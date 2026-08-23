use crate::tools::registry::{Tool, ToolContext};
use serde_json::json;
use std::path::Path;
use std::process::{Command, Output};

fn git(cwd: &Path, args: &[&str]) -> Output {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("git 실행 실패");
    assert!(
        output.status.success(),
        "git {:?} 실패: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn initialized_repo() -> tempfile::TempDir {
    let repo = tempfile::tempdir().expect("temp repo 생성 실패");
    git(repo.path(), &["init"]);
    git(repo.path(), &["config", "user.email", "audit@smlcli.test"]);
    git(repo.path(), &["config", "user.name", "Audit Test"]);
    std::fs::write(repo.path().join("base.txt"), "base\n").unwrap();
    git(repo.path(), &["add", "base.txt"]);
    git(repo.path(), &["commit", "-m", "initial"]);
    repo
}

#[test]
fn fin_f001_auto_commit_preserves_pre_staged_user_index() {
    let repo = initialized_repo();
    std::fs::write(repo.path().join("user.txt"), "user staged\n").unwrap();
    git(repo.path(), &["add", "user.txt"]);
    std::fs::write(repo.path().join("tool.txt"), "tool change\n").unwrap();

    crate::infra::git_engine::GitEngine::auto_commit(
        repo.path().to_str().unwrap(),
        "WriteFile",
        &["tool.txt"],
        "smlcli: ",
    )
    .expect("target path commit은 성공해야 함");

    let committed = git(
        repo.path(),
        &["show", "--pretty=format:", "--name-only", "HEAD"],
    );
    let committed = String::from_utf8_lossy(&committed.stdout);
    assert!(committed.lines().any(|line| line == "tool.txt"));
    assert!(
        !committed.lines().any(|line| line == "user.txt"),
        "도구 커밋이 기존 staged user.txt를 포함하면 안 됨"
    );

    let staged = git(repo.path(), &["diff", "--cached", "--name-only"]);
    assert!(
        String::from_utf8_lossy(&staged.stdout)
            .lines()
            .any(|line| line == "user.txt"),
        "기존 staged user.txt는 index에 그대로 남아야 함"
    );
}

#[test]
fn fin_f001_hard_reset_rollback_is_disabled_and_preserves_wip() {
    let repo = initialized_repo();
    let tracked = repo.path().join("base.txt");
    std::fs::write(&tracked, "concurrent user wip\n").unwrap();

    let result = crate::tools::git_checkpoint::rollback_checkpoint(
        repo.path().to_str().expect("UTF-8 temp path"),
    );

    assert!(
        result.is_err(),
        "자동 hard-reset rollback은 비활성화되어야 함"
    );
    assert_eq!(
        std::fs::read_to_string(tracked).unwrap(),
        "concurrent user wip\n",
        "실패 복구가 concurrent WIP를 변경하면 안 됨"
    );
}

fn workspace_tempdir() -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix(".smlcli-audit-")
        .tempdir_in(std::env::current_dir().expect("current dir"))
        .expect("workspace tempdir 생성 실패")
}

fn tool_context<'a>(
    token: &'a crate::domain::permissions::PermissionToken,
    settings: &'a crate::domain::settings::PersistedSettings,
) -> ToolContext<'a> {
    ToolContext {
        token,
        cancel_token: tokio_util::sync::CancellationToken::new(),
        settings,
        event_tx: None,
    }
}

async fn which_python() -> Option<String> {
    for candidate in ["python3", "python"] {
        if tokio::process::Command::new(candidate)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await
            .is_ok_and(|status| status.success())
        {
            return Some(candidate.to_string());
        }
    }
    None
}

#[tokio::test]
async fn fin_f010_write_file_false_is_create_only() {
    let dir = workspace_tempdir();
    let path = dir.path().join("existing.txt");
    std::fs::write(&path, "original").unwrap();
    let token = crate::domain::permissions::PermissionToken::grant();
    let settings = crate::domain::settings::PersistedSettings::default();

    let result = crate::tools::file_ops::WriteFileTool
        .execute(
            json!({"path": path, "content": "replacement", "overwrite": false}),
            &tool_context(&token, &settings),
        )
        .await
        .expect("계약 오류는 typed ToolResult로 반환");

    assert!(
        result.is_error,
        "overwrite=false existing path는 거부해야 함"
    );
    assert_eq!(std::fs::read_to_string(path).unwrap(), "original");
}

#[test]
fn fin_f010_write_file_schema_requires_overwrite() {
    let schema = crate::tools::file_ops::WriteFileTool.schema();
    let required = schema["function"]["parameters"]["required"]
        .as_array()
        .expect("required array");
    assert!(required.iter().any(|entry| entry == "overwrite"));
}

#[tokio::test]
async fn fin_f010_replace_rejects_empty_or_ambiguous_target_without_mutation() {
    let dir = workspace_tempdir();
    let token = crate::domain::permissions::PermissionToken::grant();
    let settings = crate::domain::settings::PersistedSettings::default();
    let tool = crate::tools::file_ops::ReplaceFileContentTool;

    let empty_path = dir.path().join("empty-target.txt");
    std::fs::write(&empty_path, "abc").unwrap();
    let empty = tool
        .execute(
            json!({
                "path": empty_path,
                "target_content": "",
                "replacement_content": "x"
            }),
            &tool_context(&token, &settings),
        )
        .await
        .expect("계약 오류는 typed ToolResult로 반환");
    assert!(empty.is_error);
    assert_eq!(std::fs::read_to_string(empty_path).unwrap(), "abc");

    let duplicate_path = dir.path().join("duplicate-target.txt");
    std::fs::write(&duplicate_path, "same / same").unwrap();
    let duplicate = tool
        .execute(
            json!({
                "path": duplicate_path,
                "target_content": "same",
                "replacement_content": "changed"
            }),
            &tool_context(&token, &settings),
        )
        .await
        .expect("계약 오류는 typed ToolResult로 반환");
    assert!(duplicate.is_error);
    assert_eq!(
        std::fs::read_to_string(duplicate_path).unwrap(),
        "same / same"
    );
}

#[cfg(unix)]
#[test]
fn fin_f003_file_context_rejects_outside_symlink_binary_and_oversize() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("workspace fixture");
    let outside = tempfile::NamedTempFile::new().expect("outside fixture");
    std::fs::write(outside.path(), "outside secret").unwrap();

    let inside = root.path().join("inside.txt");
    std::fs::write(&inside, "inside text").unwrap();
    let inside_text =
        crate::tools::file_ops::read_file_context(inside.to_str().unwrap(), root.path(), 32)
            .expect("inside UTF-8 text는 허용");
    assert_eq!(inside_text, "inside text");

    assert!(
        crate::tools::file_ops::read_file_context(
            outside.path().to_str().unwrap(),
            root.path(),
            32,
        )
        .is_err(),
        "absolute outside file은 거부"
    );

    let link = root.path().join("outside-link.txt");
    symlink(outside.path(), &link).unwrap();
    assert!(
        crate::tools::file_ops::read_file_context(link.to_str().unwrap(), root.path(), 32).is_err(),
        "workspace 내부 symlink가 outside를 가리키면 거부"
    );

    let binary = root.path().join("binary.dat");
    std::fs::write(&binary, b"abc\0def").unwrap();
    assert!(
        crate::tools::file_ops::read_file_context(binary.to_str().unwrap(), root.path(), 32)
            .is_err(),
        "NUL binary는 거부"
    );

    let invalid_utf8 = root.path().join("invalid.txt");
    std::fs::write(&invalid_utf8, [0xff, 0xfe, 0xfd]).unwrap();
    assert!(
        crate::tools::file_ops::read_file_context(invalid_utf8.to_str().unwrap(), root.path(), 32,)
            .is_err(),
        "invalid UTF-8은 lossy 변환하지 않고 거부"
    );

    let oversized = root.path().join("oversized.txt");
    std::fs::write(&oversized, "x".repeat(33)).unwrap();
    assert!(
        crate::tools::file_ops::read_file_context(oversized.to_str().unwrap(), root.path(), 32,)
            .is_err(),
        "byte cap을 넘는 file은 거부"
    );
}

#[tokio::test]
async fn fin_f005_missing_or_unsupported_custom_provider_fails_closed() {
    use crate::domain::provider::{CustomProviderConfig, ProviderKind, ToolDialect};

    let mut registry = crate::providers::registry::ProviderRegistry::new();
    let missing = registry.get_adapter(&ProviderKind::Custom("missing".to_string()));
    assert!(
        missing.validate_credentials("sentinel-key").await.is_err(),
        "missing custom adapter가 built-in/Mock adapter로 fallback하면 안 됨"
    );

    let registration = registry.register_custom_providers(&[CustomProviderConfig {
        id: "unsupported-gemini".to_string(),
        base_url: "https://custom.example.invalid/v1".to_string(),
        auth_type: "Bearer".to_string(),
        auth_header_name: None,
        dialect: ToolDialect::Gemini,
    }]);
    assert!(registration.is_err(), "custom Gemini 등록은 거부해야 함");
    let unsupported = registry.get_adapter(&ProviderKind::Custom("unsupported-gemini".to_string()));
    assert!(
        unsupported
            .validate_credentials("sentinel-key")
            .await
            .is_err(),
        "configured base URL을 무시하는 custom Gemini dialect는 zero-request error여야 함"
    );
}

#[test]
fn fin_f005_persisted_provider_name_is_strictly_resolved() {
    let settings = crate::domain::settings::PersistedSettings::default();
    assert!(
        crate::app::chat_runtime::resolve_provider_kind(&settings, "typo-provider").is_err(),
        "unknown provider name을 OpenRouter로 fallback하면 안 됨"
    );
    assert!(
        crate::app::chat_runtime::resolve_provider_kind(&settings, "Custom: missing").is_err(),
        "settings에 없는 custom id는 zero-request error여야 함"
    );
}

#[cfg(unix)]
#[test]
fn fin_f004_preexisting_deterministic_temp_symlink_cannot_modify_target() {
    use std::os::unix::fs::symlink;

    let dir = workspace_tempdir();
    let target = dir.path().join("victim.txt");
    let external = tempfile::NamedTempFile::new().expect("external sentinel 생성");
    std::fs::write(external.path(), "sentinel").unwrap();
    let deterministic_tmp = dir.path().join("victim.txt.tmp");
    symlink(external.path(), &deterministic_tmp).unwrap();

    let result = crate::tools::file_ops::write_file_commit(target.to_str().unwrap(), "new content")
        .expect("write result");

    assert!(!result.is_error, "unique temp write 자체는 성공해야 함");
    assert_eq!(
        std::fs::read_to_string(external.path()).unwrap(),
        "sentinel",
        "pre-existing sibling temp symlink의 대상은 절대 수정되면 안 됨"
    );
}

#[cfg(unix)]
#[test]
fn fin_f010_atomic_replace_preserves_existing_mode() {
    use std::os::unix::fs::PermissionsExt;

    let dir = workspace_tempdir();
    let target = dir.path().join("executable.sh");
    std::fs::write(&target, "old").unwrap();
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o755)).unwrap();

    let result = crate::tools::file_ops::write_file_commit(target.to_str().unwrap(), "new")
        .expect("atomic replace result");
    assert!(!result.is_error);
    assert_eq!(
        std::fs::metadata(target).unwrap().permissions().mode() & 0o777,
        0o755
    );
}

#[test]
fn fin_f007_fetch_permission_denies_non_public_destinations() {
    let tool = crate::tools::fetch::FetchUrlTool;
    let settings = crate::domain::settings::PersistedSettings {
        network_policy: crate::domain::permissions::NetworkPolicy::AllowAll,
        ..Default::default()
    };

    for url in [
        "http://127.0.0.1:8080/",
        "http://[::1]/",
        "http://169.254.169.254/latest/meta-data/",
        "http://10.0.0.1/",
    ] {
        let decision = tool.check_permission(&json!({"url": url}), &settings);
        assert!(
            matches!(
                decision,
                crate::domain::permissions::PermissionResult::Deny(_)
            ),
            "non-public URL은 permission 단계에서 Deny해야 함: {url}"
        );
    }
}

#[test]
fn fin_f007_utf8_truncation_is_char_boundary_safe_and_truthful() {
    let content = "한".repeat(4_000);
    let (truncated, was_truncated) = crate::tools::fetch::truncate_utf8_output(&content, 10_000);
    assert!(was_truncated);
    assert!(truncated.len() <= 10_000);
    assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
    assert!(truncated.chars().all(|character| character == '한'));
}

#[test]
fn fin_f023_html_conversion_has_no_copyleft_runtime_dependency_behavior_gap() {
    let html =
        "<style>.hidden{}</style><h1>Title &amp; More</h1><script>secret()</script><p>Body</p>";
    let text = crate::tools::fetch::html_to_text(html);
    assert_eq!(text, "Title & More\nBody");
    assert!(!text.contains("secret"));
}

#[test]
fn fin_f003_chat_mentions_enforce_per_turn_count() {
    let root = tempfile::tempdir().expect("mention root");
    let mut prompt = String::new();
    for index in 0..17 {
        let name = format!("file-{index}.txt");
        std::fs::write(root.path().join(&name), "x").unwrap();
        prompt.push_str(&format!("@{name} "));
    }
    let (expanded, errors) =
        crate::app::chat_runtime::expand_chat_mentions(&prompt, "", root.path());
    assert_eq!(
        expanded.matches("--- End of file-").count(),
        16,
        "최대 16개 파일만 context에 포함"
    );
    assert!(errors.iter().any(|error| error.contains("최대 16개")));
    assert!(expanded.contains("@file-16.txt"));
}

#[tokio::test]
async fn fin_f002_mcp_child_does_not_inherit_parent_pwd() {
    assert!(
        std::env::var_os("PWD").is_some(),
        "fixture는 parent PWD가 있는 환경에서 실행되어야 함"
    );
    let Some(python) = which_python().await else {
        eprintln!("[SKIP] python3 미설치로 MCP environment E2E를 건너뜀");
        return;
    };
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("mock_mcp_server.py");
    let client = crate::infra::mcp_client::McpClient::spawn(
        "env_boundary",
        &python,
        &[script.to_string_lossy().to_string()],
    )
    .await
    .expect("MCP fixture spawn");
    client.list_tools().await.expect("fixture tools/list");

    let child_pwd = client
        .call_tool("read_env", json!({"name": "PWD"}))
        .await
        .expect("read_env call");
    client.shutdown().await;
    assert_eq!(child_pwd, "<unset>", "parent PWD까지 통째로 상속하면 안 됨");
}

async fn mcp_fixture(name: &str) -> Option<crate::infra::mcp_client::McpClient> {
    let python = which_python().await?;
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("mock_mcp_server.py");
    let client = crate::infra::mcp_client::McpClient::spawn(
        name,
        &python,
        &[script.to_string_lossy().to_string()],
    )
    .await
    .expect("MCP fixture spawn");
    client.list_tools().await.expect("fixture tools/list");
    Some(client)
}

#[tokio::test]
async fn fin_f006_mcp_call_is_cancellable_and_clears_pending_request() {
    let Some(client) = mcp_fixture("cancel_fixture").await else {
        return;
    };
    let cancellation = tokio_util::sync::CancellationToken::new();
    let trigger = cancellation.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        trigger.cancel();
    });

    let started = std::time::Instant::now();
    let result = client
        .call_tool_cancellable("hang", json!({}), cancellation)
        .await;
    assert!(result.is_err(), "cancelled MCP call은 error여야 함");
    assert!(started.elapsed() < std::time::Duration::from_secs(1));
    assert_eq!(client.pending_request_count().await, 0);
    client.shutdown().await;
}

#[tokio::test]
async fn fin_f006_mcp_oversized_response_is_bounded() {
    let Some(client) = mcp_fixture("huge_fixture").await else {
        return;
    };
    let result = client.call_tool("huge_output", json!({})).await;
    assert!(
        result.is_err(),
        "1 MiB 초과 JSON-RPC line/result는 거부해야 함"
    );
    assert_eq!(client.pending_request_count().await, 0);
    client.shutdown().await;
}

#[test]
fn fin_f006_mcp_schema_requires_object_shape_and_bounded_size() {
    let malformed = crate::infra::mcp_client::McpToolInfo {
        name: "bad".to_string(),
        description: "bad schema".to_string(),
        input_schema: json!({"type": "array"}),
    };
    assert!(
        crate::infra::mcp_client::McpClient::validate_tool_info(&malformed).is_err(),
        "MCP tool inputSchema는 object contract여야 함"
    );

    let huge = crate::infra::mcp_client::McpToolInfo {
        name: "huge".to_string(),
        description: "x".repeat(70 * 1024),
        input_schema: json!({"type": "object"}),
    };
    assert!(
        crate::infra::mcp_client::McpClient::validate_tool_info(&huge).is_err(),
        "bounded schema/description 크기를 강제해야 함"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn fin_f006_mcp_shutdown_terminates_descendant_process_group() {
    let Some(client) = mcp_fixture("descendant_fixture").await else {
        return;
    };
    let pid = client
        .call_tool("spawn_child", json!({}))
        .await
        .expect("spawn_child result")
        .parse::<i32>()
        .expect("child pid");
    client.shutdown().await;

    let mut alive = true;
    for _ in 0..20 {
        alive = unsafe { libc::kill(pid, 0) } == 0;
        if !alive {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    if alive {
        unsafe {
            libc::kill(pid, libc::SIGKILL);
        }
    }
    assert!(!alive, "MCP shutdown 후 descendant process가 남으면 안 됨");
}

#[test]
fn fin_f008_safe_only_denies_read_paths_outside_workspace() {
    let tool = crate::tools::shell::ExecShellTool;
    let mut settings = crate::domain::settings::PersistedSettings {
        shell_policy: crate::domain::permissions::ShellPolicy::SafeOnly,
        ..Default::default()
    };
    let root = crate::infra::workspace_harness::canonical_workspace_root();
    settings.set_workspace_trust(
        &root,
        crate::domain::settings::WorkspaceTrustState::Trusted,
        false,
    );

    for command in ["cat /etc/passwd", "grep root /etc/passwd", "ls /etc"] {
        let decision = tool.check_permission(
            &json!({"command": command, "safe_to_auto_run": true}),
            &settings,
        );
        assert!(
            matches!(
                decision,
                crate::domain::permissions::PermissionResult::Deny(_)
            ),
            "SafeOnly command가 workspace 밖 read path를 허용하면 안 됨: {command}"
        );
    }
}

#[test]
fn fin_f008_extra_binds_are_trusted_read_only_workspace_extra_mounts() {
    let trusted = tempfile::tempdir().expect("trusted extra root");
    let nested = trusted.path().join("nested");
    std::fs::create_dir(&nested).unwrap();
    let allow_roots = vec![trusted.path().to_string_lossy().to_string()];
    let valid = vec![format!("{}:/workspace-extra/reference", nested.display())];
    let validated = crate::infra::sandbox::validate_extra_binds(&valid, &allow_roots)
        .expect("trusted extra bind");
    assert_eq!(validated.len(), 1);
    assert_eq!(validated[0].0, nested.canonicalize().unwrap());
    assert_eq!(validated[0].1, Path::new("/workspace-extra/reference"));

    assert!(
        crate::infra::sandbox::validate_extra_binds(
            &["/etc:/workspace-extra/etc".to_string()],
            &allow_roots,
        )
        .is_err(),
        "trusted extra root 밖 host bind는 거부"
    );
    assert!(
        crate::infra::sandbox::validate_extra_binds(
            &[format!("{}:/etc", nested.display())],
            &allow_roots,
        )
        .is_err(),
        "guest system path shadow bind는 거부"
    );
}

#[test]
fn fin_f008_global_pid_environment_reaper_is_disabled() {
    assert!(
        !crate::infra::process_reaper::is_global_reaping_enabled(),
        "spoofable SMLCLI_PID 기반 global process kill은 비활성화되어야 함"
    );
}

#[test]
fn fin_f009_terminal_transition_is_exactly_once_and_releases_write_queue() {
    let mut runtime = crate::app::state::RuntimeState::new();
    let cancellation = tokio_util::sync::CancellationToken::new();
    runtime
        .active_tool_cancel_tokens
        .insert("call-1".to_string(), cancellation);
    runtime.register_tool_execution("call-1".to_string(), true);
    runtime.write_tool_queue.push_back((
        crate::domain::tool_result::ToolCall {
            name: "WriteFile".to_string(),
            args: json!({"path": "next.txt", "content": "next", "overwrite": false}),
        },
        Some("call-2".to_string()),
        1,
    ));

    let first = runtime
        .begin_tool_terminal_transition("call-1")
        .expect("first terminal transition");
    assert_eq!(
        first.as_ref().and_then(|(_, id, _)| id.as_deref()),
        Some("call-2")
    );
    assert!(!runtime.is_write_tool_running);
    assert!(runtime.active_write_execution_key.is_none());
    assert!(!runtime.active_tool_cancel_tokens.contains_key("call-1"));
    assert!(
        runtime.begin_tool_terminal_transition("call-1").is_none(),
        "duplicate terminal event는 처리하지 않음"
    );
}

#[tokio::test]
async fn fin_f009_approval_revalidates_policy_and_promotes_next_request() {
    let (tx, _rx) = tokio::sync::mpsc::channel(8);
    let mut app = crate::app::App {
        state: crate::app::state::AppState::new_for_test(),
        action_tx: tx,
    };
    let root = crate::infra::workspace_harness::canonical_workspace_root();
    let mut settings = crate::domain::settings::PersistedSettings {
        shell_policy: crate::domain::permissions::ShellPolicy::Deny,
        ..Default::default()
    };
    settings.set_workspace_trust(
        &root,
        crate::domain::settings::WorkspaceTrustState::Trusted,
        false,
    );
    app.state.domain.settings = Some(settings);
    app.state.runtime.pending_tool_executions = 2;
    app.state.runtime.approval.pending_tool = Some(crate::domain::tool_result::ToolCall {
        name: "ExecShell".to_string(),
        args: json!({"command": "echo first", "safe_to_auto_run": false}),
    });
    app.state.runtime.approval.pending_tool_call_id = Some("call-1".to_string());
    app.state.runtime.approval.pending_tool_index = Some(0);
    app.state.runtime.approval.queued_approvals.push_back((
        crate::domain::tool_result::ToolCall {
            name: "ExecShell".to_string(),
            args: json!({"command": "echo second", "safe_to_auto_run": false}),
        },
        Some("call-2".to_string()),
        1,
    ));

    app.handle_tool_approval(true);

    assert_eq!(
        app.state.runtime.approval.pending_tool_call_id.as_deref(),
        Some("call-2"),
        "강화된 Deny로 첫 요청이 거부되어도 다음 approval은 승격"
    );
    assert!(!app.state.runtime.is_write_tool_running);
}

#[tokio::test]
async fn fin_f009_approval_keyboard_contract_accepts_enter_and_rejects_escape() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let (tx, _rx) = tokio::sync::mpsc::channel(8);
    let mut app = crate::app::App {
        state: crate::app::state::AppState::new_for_test(),
        action_tx: tx,
    };
    let mut settings = crate::domain::settings::PersistedSettings {
        shell_policy: crate::domain::permissions::ShellPolicy::Deny,
        ..Default::default()
    };
    let root = crate::infra::workspace_harness::canonical_workspace_root();
    settings.set_workspace_trust(
        &root,
        crate::domain::settings::WorkspaceTrustState::Trusted,
        false,
    );
    app.state.domain.settings = Some(settings);

    app.state.runtime.approval.pending_tool = Some(crate::domain::tool_result::ToolCall {
        name: "ExecShell".to_string(),
        args: json!({"command": "echo enter"}),
    });
    app.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(app.state.runtime.approval.pending_tool.is_none());
    assert!(!app.state.should_quit);

    app.state.runtime.approval.pending_tool = Some(crate::domain::tool_result::ToolCall {
        name: "ExecShell".to_string(),
        args: json!({"command": "echo escape"}),
    });
    app.handle_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(app.state.runtime.approval.pending_tool.is_none());
    assert!(
        !app.state.should_quit,
        "approval Esc는 app quit가 아니라 reject"
    );
}

#[test]
fn fin_f011_malformed_encrypted_nonce_returns_error_without_panic() {
    use secrecy::{SecretBox, SecretString};

    let key = SecretBox::new(vec![7_u8; 32].into());
    let encrypted = crate::infra::secret_store::encrypt_value(
        &key,
        &SecretString::new("secret".to_string().into()),
    )
    .expect("fixture encryption");
    let ciphertext = encrypted.split_once(':').unwrap().1;
    let malformed = format!("00:{ciphertext}");
    assert!(
        crate::infra::secret_store::decrypt_value(&key, &malformed).is_err(),
        "nonce length mismatch는 panic이 아니라 typed error"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn fin_f011_config_store_is_private_atomic_and_no_follow() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let root = tempfile::tempdir().expect("config root");
    std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o777)).unwrap();
    let path = root.path().join("config.toml");
    let settings = crate::domain::settings::PersistedSettings::default();
    crate::infra::config_store::save_config_to_path(&settings, &path)
        .await
        .expect("private atomic save");
    assert_eq!(
        std::fs::metadata(root.path()).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );

    std::fs::remove_file(&path).unwrap();
    let sentinel = tempfile::NamedTempFile::new().expect("sentinel");
    std::fs::write(sentinel.path(), "sentinel").unwrap();
    symlink(sentinel.path(), &path).unwrap();
    assert!(
        crate::infra::config_store::save_config_to_path(&settings, &path)
            .await
            .is_err(),
        "config symlink target은 거부"
    );
    assert_eq!(
        std::fs::read_to_string(sentinel.path()).unwrap(),
        "sentinel"
    );
}

#[cfg(unix)]
#[test]
fn fin_f011_master_key_is_private_and_exclusive() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().expect("secret root");
    std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o777)).unwrap();
    let first = crate::infra::secret_store::get_or_create_master_key_in(root.path())
        .expect("master key create");
    let second = crate::infra::secret_store::get_or_create_master_key_in(root.path())
        .expect("master key reuse");
    use secrecy::ExposeSecret;
    assert_eq!(first.expose_secret(), second.expose_secret());
    assert_eq!(
        std::fs::metadata(root.path()).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        std::fs::metadata(root.path().join(".master_key"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
}

#[test]
fn fin_f011_wizard_patch_preserves_unowned_settings_fields() {
    let mut existing = crate::domain::settings::PersistedSettings::default();
    existing.denied_roots.push("/denied".to_string());
    existing.extra_workspace_dirs.push("/extra".to_string());
    existing.allowed_env_vars.push("RUST_LOG".to_string());
    existing.git_integration.auto_commit = true;
    existing.sandbox.enabled = true;
    existing
        .encrypted_keys
        .insert("sentinel".to_string(), "cipher".to_string());
    existing
        .mcp_servers
        .push(crate::domain::settings::McpServerConfig {
            name: "server".to_string(),
            command: "server-bin".to_string(),
            args: Vec::new(),
            allowed_env_vars: Vec::new(),
        });

    let patched = crate::app::wizard_controller::merge_wizard_settings(
        Some(&existing),
        "OpenAI".to_string(),
        "gpt-test".to_string(),
        None,
    );
    assert_eq!(patched.default_provider, "OpenAI");
    assert_eq!(patched.default_model, "gpt-test");
    assert_eq!(patched.denied_roots, existing.denied_roots);
    assert_eq!(patched.extra_workspace_dirs, existing.extra_workspace_dirs);
    assert_eq!(patched.allowed_env_vars, existing.allowed_env_vars);
    assert_eq!(
        patched.git_integration.auto_commit,
        existing.git_integration.auto_commit
    );
    assert_eq!(patched.sandbox.enabled, existing.sandbox.enabled);
    assert_eq!(patched.encrypted_keys, existing.encrypted_keys);
    assert_eq!(patched.mcp_servers.len(), 1);
}

fn session_metadata(id: &str, log_filename: &str) -> crate::domain::session::SessionMetadata {
    crate::domain::session::SessionMetadata {
        session_id: id.to_string(),
        workspace_root: "/workspace".to_string(),
        title: id.to_string(),
        created_at_unix_ms: 1,
        updated_at_unix_ms: 1,
        log_filename: log_filename.to_string(),
    }
}

#[cfg(unix)]
#[test]
fn fin_f012_session_store_is_private_and_contained() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let root = tempfile::tempdir().expect("session root");
    std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o777)).unwrap();
    let (logger, metadata) = crate::infra::session_log::SessionLogger::new_workspace_session_in(
        root.path(),
        "/workspace",
    )
    .expect("isolated session create");
    assert_eq!(
        std::fs::metadata(root.path()).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        std::fs::metadata(&logger.file_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert!(
        crate::infra::session_log::is_valid_log_filename(&metadata.log_filename),
        "generated log filename은 containment grammar를 만족"
    );

    let outside = tempfile::NamedTempFile::new().expect("outside log");
    assert!(
        crate::infra::session_log::SessionLogger::from_file_in(
            outside.path().to_path_buf(),
            root.path(),
        )
        .is_err(),
        "absolute outside session path는 거부"
    );
    let link = root.path().join("session_1_abcdef.jsonl");
    symlink(outside.path(), &link).unwrap();
    assert!(
        crate::infra::session_log::SessionLogger::from_file_in(link, root.path()).is_err(),
        "session symlink는 거부"
    );
}

#[test]
fn fin_f012_restore_rejects_oversized_line_without_unbounded_read() {
    let root = tempfile::tempdir().expect("session root");
    let path = root.path().join("session_1_abcdef.jsonl");
    crate::infra::secure_fs::atomic_write_private(&path, "x".repeat(1024 * 1024 + 1).as_bytes())
        .unwrap();
    let logger = crate::infra::session_log::SessionLogger::from_file_in(path, root.path())
        .expect("bounded logger open");
    let (messages, errors) = logger.restore_messages().expect("bounded restore");
    assert!(messages.is_empty());
    assert_eq!(errors, 1, "oversized line은 한 건의 parse error로 격리");
}

#[test]
fn fin_f012_corrupt_index_is_not_silently_overwritten() {
    let root = tempfile::tempdir().expect("session root");
    let index = root.path().join("sessions_index.json");
    crate::infra::secure_fs::atomic_write_private(&index, b"{corrupt").unwrap();
    let result = crate::infra::session_log::SessionIndex::upsert_in(
        root.path(),
        &session_metadata("id-1", "session_1_abcdef.jsonl"),
    );
    assert!(result.is_err(), "corrupt index는 empty로 간주하지 않음");
    assert_eq!(std::fs::read_to_string(index).unwrap(), "{corrupt");
}

#[test]
fn fin_f012_concurrent_index_writers_preserve_all_entries() {
    let root = tempfile::tempdir().expect("session root");
    let root = std::sync::Arc::new(root);
    let mut threads = Vec::new();
    for index in 0..8 {
        let root = root.clone();
        threads.push(std::thread::spawn(move || {
            crate::infra::session_log::SessionIndex::upsert_in(
                root.path(),
                &session_metadata(
                    &format!("id-{index}"),
                    &format!("session_{index}_abcdef.jsonl"),
                ),
            )
            .unwrap();
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }
    let entries = crate::infra::session_log::SessionIndex::load_all_in(root.path()).unwrap();
    assert_eq!(entries.len(), 8);
}

fn chat_message(content: &str) -> crate::providers::types::ChatMessage {
    crate::providers::types::ChatMessage {
        role: crate::providers::types::Role::User,
        content: Some(content.to_string()),
        tool_calls: None,
        tool_call_id: None,
        pinned: false,
    }
}

#[test]
fn fin_f013_compaction_failure_or_drift_preserves_original_messages() {
    let mut session = crate::domain::session::SessionState::new();
    for index in 0..10 {
        session.add_message(chat_message(&format!("message-{index}")));
    }
    let original = session.messages.clone();
    let extracted = session.prepare_compaction();
    assert!(!extracted.is_empty());
    assert_eq!(
        session.messages, original,
        "prepare는 원본을 mutate하지 않음"
    );
    session.abort_compaction();
    assert_eq!(session.messages, original);

    session.prepare_compaction();
    session.add_message(chat_message("arrived-during-summary"));
    let drifted = session.messages.clone();
    assert!(session.commit_compaction("summary").is_err());
    assert_eq!(
        session.messages, drifted,
        "drift failure도 byte-equivalent state 유지"
    );
}

#[test]
fn fin_f013_compaction_commits_only_after_nonempty_success() {
    let mut session = crate::domain::session::SessionState::new();
    for index in 0..10 {
        session.add_message(chat_message(&format!("message-{index}")));
    }
    let original = session.messages.clone();
    session.prepare_compaction();
    assert!(session.commit_compaction("three bullet summary").is_ok());
    assert_ne!(session.messages, original);
    assert!(session.messages.iter().any(|message| {
        message
            .content
            .as_deref()
            .is_some_and(|content| content.contains("three bullet summary"))
    }));
}

#[tokio::test]
async fn fin_f013_malformed_and_valid_tool_calls_are_counted_in_one_turn() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    let mut app = crate::app::App {
        state: crate::app::state::AppState::new_for_test(),
        action_tx: tx,
    };
    let response = crate::providers::types::ChatMessage {
        role: crate::providers::types::Role::Assistant,
        content: None,
        tool_calls: Some(vec![
            crate::providers::types::ToolCallRequest {
                id: "bad".to_string(),
                r#type: "function".to_string(),
                function: crate::providers::types::FunctionCall {
                    name: "ReadFile".to_string(),
                    arguments: "{bad-json".to_string(),
                },
            },
            crate::providers::types::ToolCallRequest {
                id: "good".to_string(),
                r#type: "function".to_string(),
                function: crate::providers::types::FunctionCall {
                    name: "ReadFile".to_string(),
                    arguments: json!({"path": "Cargo.toml"}).to_string(),
                },
            },
        ]),
        tool_call_id: None,
        pinned: false,
    };

    app.process_tool_calls_from_response(&response);
    assert_eq!(
        app.state.runtime.pending_tool_executions, 2,
        "malformed call도 동일 turn의 terminal count에 포함"
    );
    assert!(matches!(
        rx.try_recv(),
        Ok(crate::app::event_loop::Event::Action(
            crate::app::action::Action::ToolError(_, Some(id), 0)
        )) if id == "bad"
    ));
}

#[test]
fn fin_f014_sse_decoder_handles_chunk_boundaries_incrementally() {
    let mut decoder = crate::providers::streaming::SseDecoder::new();
    assert!(
        decoder
            .push(b"data: {\"choices\":[{\"delta\":{")
            .unwrap()
            .is_empty()
    );
    let lines = decoder.push(b"\"content\":\"hello\"}}]}\n\n").unwrap();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("hello"));
}

#[test]
fn fin_f014_central_redaction_masks_exact_header_query_and_json_secrets() {
    let secret = "super-secret-token";
    let input = format!(
        "Authorization: Bearer {secret} https://example.test?key={secret} {{\"api_key\":\"{secret}\"}}"
    );
    let redacted = crate::infra::redaction::redact_sensitive_text(&input, &[secret]);
    assert!(!redacted.contains(secret));
    assert!(redacted.matches("[REDACTED]").count() >= 3);
}

#[test]
fn fin_f014_streaming_masker_never_emits_split_secret_prefix() {
    let (tx, _rx) = tokio::sync::mpsc::channel(1);
    let mut app = crate::app::App {
        state: crate::app::state::AppState::new_for_test(),
        action_tx: tx,
    };
    app.state.runtime.secret_mask_regex = regex::Regex::new("supersecret").ok();
    app.state.runtime.streaming_masker.max_match_len = "supersecret".len();

    let mut first = "prefix super".to_string();
    app.mask_stream_chunk(&mut first);
    assert!(!first.contains("super"));

    let mut second = "secret suffix\n01234567890".to_string();
    app.mask_stream_chunk(&mut second);
    let emitted = format!("{first}{second}");
    assert!(emitted.starts_with("prefix "));
    assert!(emitted.contains("[REDACTED]"));
    assert!(!emitted.contains("supersecret"));
    assert!(
        !first.contains("super"),
        "첫 chunk에서 secret prefix를 emit하지 않음"
    );
}

#[tokio::test]
async fn fin_f014_exec_shell_tool_path_emits_live_output_events() {
    let settings = crate::domain::settings::PersistedSettings::default();
    let (tx, mut rx) = tokio::sync::mpsc::channel(8);
    let task = tokio::spawn(async move {
        crate::tools::shell::execute_shell_streaming(
            "printf 'live-output\\n'",
            Some("."),
            &settings,
            Some(tx),
            tokio_util::sync::CancellationToken::new(),
        )
        .await
    });
    let event = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
        .await
        .expect("live event timeout")
        .expect("live event channel");
    assert!(matches!(
        event,
        crate::app::event_loop::Event::Action(
            crate::app::action::Action::ToolOutputChunk(chunk)
        ) if chunk.contains("live-output")
    ));
    let result = task.await.unwrap().unwrap();
    assert!(!result.is_error);
}

#[test]
fn fin_f014_grep_truncation_metadata_matches_actual_limit() {
    let dir = workspace_tempdir();
    let path = dir.path().join("many.txt");
    std::fs::write(&path, "hit\n".repeat(101)).unwrap();
    let result = crate::tools::grep::grep_search("hit", dir.path().to_str().unwrap(), false)
        .expect("grep fixture");
    assert!(result.is_truncated);
    assert!(result.original_size_bytes.is_some());
    assert!(result.stdout.contains("결과 잘림"));
}

#[test]
fn fin_f015_repo_map_discards_stale_worker_revision() {
    let mut state = crate::domain::repo_map::RepoMapState::new();
    state.mark_stale();
    let first = state.begin_refresh().expect("first revision");
    state.mark_stale();
    assert!(
        !state.finish_success(first, "stale map".to_string()),
        "old worker result는 적용하지 않음"
    );
    assert!(state.cached.is_none());
    let latest = state.begin_refresh().expect("latest revision");
    assert!(latest > first);
    assert!(state.finish_success(latest, "latest map".to_string()));
    assert_eq!(state.cached.as_deref(), Some("latest map"));
    assert_eq!(state.applied_revision, latest);
}

#[tokio::test]
async fn fin_f015_event_loop_shutdown_closes_all_producers() {
    let (mut events, tx) =
        crate::app::event_loop::EventLoop::new(std::time::Duration::from_millis(5));
    events.shutdown().await;
    assert!(
        tx.send(crate::app::event_loop::Event::Tick).await.is_err(),
        "shutdown 후 event receiver가 닫혀야 함"
    );
}

#[test]
fn fin_f016_harness_baseline_detects_all_enforced_drift_fields() {
    let baseline = crate::infra::workspace_harness::WorkspaceHarnessSnapshot::collect(None);
    let mut current = baseline.clone();
    current.canonical_root.push_str("-moved");
    current.trust_state = crate::domain::settings::WorkspaceTrustState::Trusted;
    current.denied = !current.denied;
    current.sandbox_enabled = !current.sandbox_enabled;
    current.sandbox_guest_root = "/different".to_string();
    assert_eq!(
        crate::infra::workspace_harness::harness_drift_fields(&baseline, &current),
        vec![
            "canonical_root",
            "trust_state",
            "denied",
            "sandbox_enabled",
            "sandbox_guest_root"
        ]
    );
}

#[test]
fn fin_f016_doctor_network_probe_respects_network_policy() {
    let mut settings = crate::domain::settings::PersistedSettings {
        network_policy: crate::domain::permissions::NetworkPolicy::Deny,
        ..Default::default()
    };
    assert!(crate::infra::doctor::doctor_probe_url(&settings).is_none());
    settings.network_policy = crate::domain::permissions::NetworkPolicy::ProviderOnly;
    assert_eq!(
        crate::infra::doctor::doctor_probe_url(&settings)
            .and_then(|url| url.host_str().map(ToString::to_string))
            .as_deref(),
        Some("openrouter.ai")
    );
}

#[test]
fn fin_f016_wizard_success_reopens_trust_gate_with_safe_preset() {
    let (tx, _rx) = tokio::sync::mpsc::channel(8);
    let mut app = crate::app::App {
        state: crate::app::state::AppState::new_for_test(),
        action_tx: tx,
    };
    app.state.ui.trust_gate.popup = crate::app::state::TrustGatePopup::Closed;
    let settings = crate::domain::settings::PersistedSettings::default();
    assert_eq!(
        settings.network_policy,
        crate::domain::permissions::NetworkPolicy::ProviderOnly
    );
    assert_eq!(
        settings.shell_policy,
        crate::domain::permissions::ShellPolicy::Ask
    );
    app.handle_action(crate::app::action::Action::WizardSaveFinished(Ok(settings)));
    assert!(matches!(
        app.state.ui.trust_gate.popup,
        crate::app::state::TrustGatePopup::Open { .. }
    ));
}

#[test]
fn fin_f017_command_registry_drives_slash_palette_and_help_surfaces() {
    let slash = crate::app::state::SlashMenuState::new();
    let palette = crate::app::state::CommandPaletteState::new();
    let registry = crate::commands::COMMANDS
        .iter()
        .map(|command| command.id)
        .collect::<std::collections::HashSet<_>>();
    let slash_ids = slash
        .matches
        .iter()
        .map(|(id, _)| *id)
        .collect::<std::collections::HashSet<_>>();
    let palette_ids = palette
        .all_commands
        .iter()
        .filter(|command| command.id.starts_with('/'))
        .map(|command| command.id)
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(slash_ids, registry);
    assert_eq!(palette_ids, registry);
    assert!(registry.contains("/mcp"));
    assert!(registry.contains("/undo"));
    assert!(registry.contains("/quit"));
}

#[test]
fn fin_f017_status_and_clear_mutate_visible_timeline() {
    let (tx, _rx) = tokio::sync::mpsc::channel(8);
    let mut app = crate::app::App {
        state: crate::app::state::AppState::new_for_test(),
        action_tx: tx,
    };
    app.state.domain.settings = Some(crate::domain::settings::PersistedSettings::default());
    app.handle_slash_command("/status");
    assert!(
        app.state
            .ui
            .timeline
            .iter()
            .any(|block| block.title == "Session Status")
    );
    app.state
        .ui
        .timeline
        .push(crate::app::state::TimelineBlock::new(
            crate::app::state::TimelineBlockKind::Conversation,
            "old visible message",
        ));
    app.handle_slash_command("/clear");
    assert_eq!(app.state.ui.timeline.len(), 1);
    assert_eq!(app.state.ui.timeline[0].title, "Chat cleared");
}

#[test]
fn fin_f017_key_only_focus_and_inspector_shortcuts_are_reachable() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let (tx, _rx) = tokio::sync::mpsc::channel(8);
    let mut app = crate::app::App {
        state: crate::app::state::AppState::new_for_test(),
        action_tx: tx,
    };
    app.state.ui.show_inspector = true;
    app.state.ui.focused_pane = crate::app::state::FocusedPane::Composer;
    app.handle_input(KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL));
    assert_eq!(
        app.state.ui.focused_pane,
        crate::app::state::FocusedPane::Timeline
    );
    app.handle_input(KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL));
    assert_eq!(
        app.state.ui.focused_pane,
        crate::app::state::FocusedPane::Inspector
    );
    app.handle_input(KeyEvent::new(KeyCode::Char('4'), KeyModifiers::ALT));
    assert_eq!(
        app.state.ui.active_inspector_tab,
        crate::app::state::InspectorTab::Logs
    );
}

#[test]
fn fin_f018_unicode_truncation_is_width_bounded_and_boundary_safe() {
    use unicode_width::UnicodeWidthStr;
    for value in [
        "/경로/매우긴파일이름.rs",
        "workspace/🦀/emoji/非常に長い名前.rs",
        "ascii/very/long/path/to/file.rs",
    ] {
        let truncated = crate::tui::layout::truncate_middle(value, 12);
        assert!(UnicodeWidthStr::width(truncated.as_str()) <= 12);
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
        assert!(truncated.contains('…'));
    }
}

#[test]
fn fin_f018_layout_geometry_and_hit_test_share_all_breakpoints() {
    for width in [80, 90, 99, 100, 103, 104, 120, 140] {
        let geometry = crate::tui::layout::LayoutGeometry::new(
            ratatui::layout::Rect::new(0, 0, width, 30),
            true,
        );
        let inspector = geometry.inspector.expect("active inspector");
        assert!(inspector.x + inspector.width <= width);
        assert!(crate::tui::layout::LayoutGeometry::contains(
            inspector,
            inspector.x,
            inspector.y
        ));
        if width < 100 {
            assert_eq!(
                geometry.timeline, geometry.body,
                "compact drawer overlays body"
            );
            assert_eq!(inspector.width, 32.min(width));
        } else {
            assert_eq!(geometry.timeline.width + inspector.width, width);
            assert!(geometry.timeline.width >= 72);
        }
    }
}

#[test]
fn fin_f018_compact_inspector_mouse_tabs_map_to_six_actions() {
    let geometry =
        crate::tui::layout::LayoutGeometry::new(ratatui::layout::Rect::new(0, 0, 90, 30), true);
    let inspector = geometry.inspector.unwrap();
    let first = geometry.inspector_tab_at(inspector.x + 2, inspector.y);
    let fourth = geometry.inspector_tab_at(inspector.x + 2, inspector.y + 1);
    assert_eq!(first, Some(0));
    assert_eq!(fourth, Some(3));
}

#[tokio::test]
async fn fin_f018_overlay_render_and_input_priority_match() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let (tx, _rx) = tokio::sync::mpsc::channel(8);
    let mut app = crate::app::App {
        state: crate::app::state::AppState::new_for_test(),
        action_tx: tx,
    };
    app.state.ui.show_help_overlay = true;
    app.state.ui.questionnaire = Some(crate::domain::questionnaire::QuestionnaireState::new(
        vec![crate::domain::questionnaire::ClarificationQuestion {
            id: "q".to_string(),
            title: "Question".to_string(),
            options: vec!["A".to_string()],
            allow_custom: false,
        }],
        Some("call".to_string()),
        0,
    ));
    app.handle_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(
        app.state.ui.questionnaire.is_none(),
        "top questionnaire consumes Esc"
    );
    assert!(
        app.state.ui.show_help_overlay,
        "underlying help remains open"
    );
}

#[test]
fn fin_f019_locale_normalization_supports_real_lang_formats() {
    assert_eq!(crate::tui::i18n::normalize_locale("ko_KR.UTF-8"), "ko");
    assert_eq!(crate::tui::i18n::normalize_locale("ja-JP@calendar"), "ja");
    assert_eq!(crate::tui::i18n::normalize_locale("zh-TW.UTF-8"), "zh_TW");
    assert_eq!(
        crate::tui::i18n::normalize_locale("zh_Hans_CN.UTF-8"),
        "zh_CN"
    );
    assert_eq!(crate::tui::i18n::normalize_locale("C.UTF-8"), "en");
}

#[test]
fn fin_f019_all_supported_locales_own_common_surface_keys() {
    let manager = crate::tui::i18n::I18nManager::new("en");
    for locale in ["ko", "en", "ja", "zh_TW", "zh_CN"] {
        for key in [
            "config_title",
            "config_dashboard",
            "wizard_title",
            "wizard_step_provider",
            "loading_models",
            "help_title",
            "no_active_blocks",
            "terminal_too_small",
            "api_key_required",
        ] {
            assert!(manager.has_translation(locale, key), "{locale}:{key}");
        }
    }
}

#[test]
fn fin_f019_wizard_and_config_render_each_locale_catalog() {
    use ratatui::{Terminal, backend::TestBackend};

    for locale in ["ko", "en", "ja", "zh_TW", "zh_CN"] {
        let mut state = crate::app::state::AppState::new_for_test();
        state.i18n = crate::tui::i18n::I18nManager::new(locale);
        state.ui.is_wizard_open = true;
        let expected_wizard = state.i18n.tr("wizard_title").to_string();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| crate::tui::layout::draw(frame, &state))
            .unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(
            rendered
                .replace(' ', "")
                .contains(&expected_wizard.replace(' ', "")),
            "wizard locale {locale}"
        );

        state.ui.is_wizard_open = false;
        state.ui.config.is_open = true;
        let expected_config = state.i18n.tr("config_title").to_string();
        terminal
            .draw(|frame| crate::tui::layout::draw(frame, &state))
            .unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(
            rendered
                .replace(' ', "")
                .contains(&expected_config.replace(' ', "")),
            "config locale {locale}"
        );
    }
}

#[test]
fn fin_f020_cli_run_contract_is_interactive_without_prompt_argument() {
    use clap::{CommandFactory, Parser};
    let command = crate::Cli::command();
    let run = command
        .get_subcommands()
        .find(|subcommand| subcommand.get_name() == "run")
        .expect("run subcommand");
    assert!(run.get_arguments().next().is_none());
    assert!(crate::Cli::try_parse_from(["smlcli", "run", "prompt"]).is_err());
}

#[test]
fn fin_f020_provider_and_command_public_contract_matches_runtime() {
    use crate::domain::provider::ProviderKind;
    let builtins = [
        ProviderKind::OpenAI,
        ProviderKind::Anthropic,
        ProviderKind::Xai,
        ProviderKind::OpenRouter,
        ProviderKind::Google,
        ProviderKind::LmStudio,
    ];
    let registry = crate::providers::registry::ProviderRegistry::new();
    for provider in builtins {
        let adapter = registry.get_adapter(&provider);
        let pointer = &*adapter as *const _ as *const ();
        assert!(!pointer.is_null());
    }
    assert!(!crate::commands::COMMANDS.is_empty());
    assert!(crate::commands::is_known_command("/mcp"));
}

#[test]
fn fin_f021_identity_version_and_adr_authority_are_unique() {
    assert_eq!(env!("CARGO_PKG_NAME"), "smlcli");
    assert_eq!(env!("CARGO_PKG_VERSION"), "3.9.0");
    let agents = include_str!("../../AGENTS.md");
    assert!(agents.contains("**Project Name:** smlcli"));
    assert!(agents.contains("**Project Version:** 3.9.0"));

    let decisions = include_str!("../../DESIGN_DECISIONS.md");
    let mut ids = std::collections::HashSet::new();
    for line in decisions.lines().filter(|line| line.starts_with("## ADR-")) {
        let id = line.split(':').next().unwrap().trim_start_matches("## ");
        assert!(ids.insert(id.to_string()), "duplicate ADR id: {id}");
    }
    assert!(ids.contains("ADR-014"));
    assert!(ids.contains("ADR-041"));
}

#[test]
fn fin_f023_lockfile_contains_patched_advisory_versions() {
    let lock = include_str!("../../Cargo.lock");
    for (name, version) in [
        ("anyhow", "1.0.103"),
        ("crossbeam-epoch", "0.9.20"),
        ("h2", "0.4.16"),
        ("quinn-proto", "0.11.15"),
    ] {
        let package = format!("name = \"{name}\"\nversion = \"{version}\"");
        assert!(
            lock.contains(&package),
            "patched lock entry missing: {package}"
        );
    }
    assert!(!lock.contains("name = \"html2md\""));
    assert!(!lock.contains("name = \"shadow-rs\""));
}

#[test]
fn fin_f024_workflow_controls_are_full_sha_pinned_and_locked() {
    let status = std::process::Command::new("python3")
        .arg("scripts/check-workflows.py")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("workflow check 실행");
    assert!(status.success());
}

#[test]
fn fin_f025_package_scope_excludes_local_and_reference_residue() {
    let manifest = include_str!("../../Cargo.toml");
    assert!(manifest.contains("include = ["));
    for excluded in [
        "scratch.py",
        ".gemini",
        ".codex",
        ".antigravitycli",
        "stitch_modern_tui_redesign",
        "__pycache__",
    ] {
        assert!(!manifest.contains(&format!("\"/{excluded}")));
    }
}
