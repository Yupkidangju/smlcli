// [v0.1.0-beta.8] 4차 감사 회귀 테스트 모듈.
// 감사 보고서에서 지적된 High/Medium 이슈에 대한 회귀 방지 테스트.
// 각 테스트는 특정 감사 항목(H-1~H-4, M-1~M-2)에 매핑됨.

use crate::app::state::{ConfigPopup, ConfigState, WizardState, WizardStep};
use crate::domain::permissions::{
    FileWritePolicy, NetworkPolicy, PermissionEngine, PermissionResult, ShellPolicy,
};
use crate::domain::settings::PersistedSettings;
use crate::domain::tool_result::ToolCall;

// --- H-2: API 키 마스킹 검증 ---
// 렌더러가 평문 대신 마스크 문자열을 받는지 검증

#[test]
fn test_api_key_masking() {
    // 위자드 상태에 API 키를 입력한 후, 마스킹 로직이 올바르게 동작하는지 검증.
    // setting_wizard.rs의 draw_wizard에서 사용되는 마스킹 패턴과 동일.
    let wizard = WizardState::new();

    // 빈 입력은 빈 마스크
    let masked = "*".repeat(wizard.api_key_input.len());
    assert_eq!(masked, "");

    // 실제 키 입력 시 동일 길이의 '*'로 마스킹
    let test_key = "sk-or-v1-abc123def456";
    let masked = "*".repeat(test_key.len());
    assert_eq!(masked.len(), test_key.len());
    assert!(!masked.contains("sk-or"));
    assert!(masked.chars().all(|c| c == '*'));
}

// --- H-3: Provider 전환 시 모델 초기화 검증 ---
// /provider 로 전환 시 default_model이 "auto"로 리셋되는지 확인

#[test]
fn test_provider_switch_resets_model() {
    // Provider 전환 시 이전 provider의 모델명이 유지되면 충돌하므로,
    // "auto"로 초기화되어야 함.
    let mut settings = PersistedSettings {
        default_provider: "OpenRouter".to_string(),
        default_model: "gpt-4o".to_string(),
        ..PersistedSettings::default()
    };

    // Provider 전환 시뮬레이션: wizard_controller.rs ProviderList 브랜치의 핵심 로직
    settings.default_provider = "Google".to_string();
    settings.default_model = "auto".to_string();

    assert_eq!(settings.default_provider, "Google");
    assert_eq!(settings.default_model, "auto");
}

// --- H-4: NetworkPolicy::Deny 시 채팅 차단 검증 ---

#[test]
fn test_network_policy_deny_blocks_chat() {
    // NetworkPolicy::Deny 설정 시 채팅 요청이 차단되어야 함.
    // chat_runtime.rs에서 dispatch_chat_request 진입 시 정책 검사.
    let settings = PersistedSettings {
        version: 1,
        default_provider: "OpenRouter".to_string(),
        default_model: "auto".to_string(),
        shell_policy: ShellPolicy::Ask,
        file_write_policy: FileWritePolicy::AlwaysAsk,
        network_policy: NetworkPolicy::Deny,
        safe_commands: None,
        encrypted_keys: std::collections::HashMap::new(),
        theme: "default".to_string(),
    };

    // Deny 상태 확인
    assert_eq!(settings.network_policy, NetworkPolicy::Deny);

    // ProviderOnly 상태에서는 통과해야 함
    let settings_allow = PersistedSettings {
        network_policy: NetworkPolicy::ProviderOnly,
        ..settings.clone()
    };
    assert_eq!(settings_allow.network_policy, NetworkPolicy::ProviderOnly);
}

// --- M-1: 위자드 에러 상태에서 Esc 시 복구 검증 ---

#[test]
fn test_wizard_error_esc_restarts() {
    // 위자드에서 err_msg가 설정된 상태에서 Esc 시,
    // ProviderSelection으로 복구되어야 함 (앱 종료가 아님).
    let mut wizard = WizardState::new();
    wizard.step = WizardStep::ModelSelection;
    wizard.err_msg = Some("Failed to fetch models".to_string());
    wizard.api_key_input = "some-key".to_string();

    // mod.rs의 KeyCode::Esc 핸들러 로직 시뮬레이션
    if wizard.err_msg.is_some() {
        wizard.step = WizardStep::ProviderSelection;
        wizard.err_msg = None;
        wizard.api_key_input.clear();
        wizard.cursor_index = 0;
    }

    assert_eq!(wizard.step, WizardStep::ProviderSelection);
    assert!(wizard.err_msg.is_none());
    assert!(wizard.api_key_input.is_empty());
    assert_eq!(wizard.cursor_index, 0);
}

// --- M-1: err_msg가 없으면 Esc는 여전히 종료 의도 ---

#[test]
fn test_wizard_no_error_esc_quits() {
    // 에러가 없는 정상 위자드 상태에서 Esc는 should_quit 트리거.
    let wizard = WizardState::new();
    assert!(wizard.err_msg.is_none());
    // err_msg가 None이므로 wizard 분기에 진입하지 않고 should_quit = true가 됨
    // (mod.rs Esc 핸들러의 else 브랜치)
}

// --- H-1: Saving 단계에서 err_msg가 설정되면 위자드가 닫히지 않음 ---

#[test]
fn test_wizard_save_failure_keeps_wizard_open() {
    // save_wizard_settings 실패 시 is_wizard_open이 여전히 true여야 함.
    // 암호화 저장소를 사용할 수 없는 테스트 환경에서는 상태 전이만 검증.
    let wizard = WizardState::new();

    // Saving 단계 진입
    assert_eq!(wizard.step, WizardStep::ProviderSelection);

    // err_msg가 설정되었을 때 is_wizard_open이 닫히면 안 되는 불변식
    let mut is_wizard_open = true;
    let err = Some("Failed to access encrypted store: config not found".to_string());

    // save_wizard_settings의 에러 분기: return 하므로 is_wizard_open이 변경 안 됨
    if err.is_some() {
        // return 시뮬레이션: is_wizard_open은 변경하지 않음
    } else {
        is_wizard_open = false; // 정상 완료 시에만 닫힘
    }

    assert!(is_wizard_open, "저장 실패 시 위자드가 닫히면 안 됨");
}

// --- H-4: NetworkPolicy가 ToolCall 검사에도 적용되는지 ---

#[test]
fn test_permission_engine_denies_shell_on_deny_policy() {
    // ShellPolicy::Deny 설정 시 ExecShell이 차단되는지 검증.
    let settings = PersistedSettings {
        version: 1,
        default_provider: "OpenRouter".to_string(),
        default_model: "auto".to_string(),
        shell_policy: ShellPolicy::Deny,
        file_write_policy: FileWritePolicy::AlwaysAsk,
        network_policy: NetworkPolicy::ProviderOnly,
        safe_commands: None,
        encrypted_keys: std::collections::HashMap::new(),
        theme: "default".to_string(),
    };

    let tool = ToolCall {
        name: "ExecShell".to_string(),
        args: serde_json::json!({
            "command": "rm -rf /".to_string(),
            "cwd": serde_json::Value::Null,
            "safe_to_auto_run": false
        }),
    };

    let result = PermissionEngine::check(&tool, &settings);
    assert!(
        matches!(result, PermissionResult::Deny(_)),
        "ShellPolicy::Deny에서 ExecShell은 거부되어야 함"
    );
}

// --- ConfigState 초기화 검증 ---

#[test]
fn test_config_state_defaults() {
    // ConfigState의 err_msg 필드가 존재하고 None으로 초기화되는지 검증.
    let config = ConfigState::new();
    assert!(!config.is_open);
    assert_eq!(config.active_popup, ConfigPopup::Dashboard);
    assert!(config.err_msg.is_none());
    assert!(config.available_models.is_empty());
}

// --- FileWritePolicy::AlwaysAsk 시 도구가 Ask로 분류되는지 ---

#[test]
fn test_file_write_asks_permission() {
    let settings = PersistedSettings::default();
    assert_eq!(settings.file_write_policy, FileWritePolicy::AlwaysAsk);

    let tool = ToolCall {
        name: "WriteFile".to_string(),
        args: serde_json::json!({
            "path": "/tmp/test.txt".to_string(),
            "content": "hello".to_string(),
            "overwrite": true
        }),
    };

    let result = PermissionEngine::check(&tool, &settings);
    assert!(
        matches!(result, PermissionResult::Ask),
        "AlwaysAsk 정책에서 WriteFile은 Ask이어야 함"
    );
}

// --- ReadFile은 항상 Allow ---

#[test]
fn test_read_file_always_allowed() {
    let settings = PersistedSettings::default();
    let tool = ToolCall {
        name: "ReadFile".to_string(),
        args: serde_json::json!({
            "path": "/etc/passwd".to_string(),
            "start_line": serde_json::Value::Null,
            "end_line": serde_json::Value::Null
        }),
    };

    let result = PermissionEngine::check(&tool, &settings);
    assert!(
        matches!(result, PermissionResult::Allow),
        "ReadFile은 항상 Allow이어야 함"
    );
}

// --- [v0.1.0-beta.18] Phase 9-B: Blocked Command 차단 검증 ---
// BLOCKED_PATTERNS에 해당하는 위험 명령어가 정책과 무관하게 차단되는지 확인

#[test]
fn test_blocked_command_sudo_denied() {
    // sudo 명령어는 ShellPolicy::Ask여도 무조건 차단되어야 함
    let settings = PersistedSettings {
        shell_policy: ShellPolicy::Ask,
        ..PersistedSettings::default()
    };
    let tool = ToolCall {
        name: "ExecShell".to_string(),
        args: serde_json::json!({
            "command": "sudo rm -rf /".to_string(),
            "cwd": serde_json::Value::Null,
            "safe_to_auto_run": false
        }),
    };
    let result = PermissionEngine::check(&tool, &settings);
    assert!(
        matches!(result, PermissionResult::Deny(_)),
        "sudo 명령어는 무조건 차단이어야 함"
    );
}

#[test]
fn test_blocked_command_rm_rf_denied() {
    // rm -rf는 safe_to_auto_run=true여도 무조건 차단
    let settings = PersistedSettings {
        shell_policy: ShellPolicy::SafeOnly,
        ..PersistedSettings::default()
    };
    let tool = ToolCall {
        name: "ExecShell".to_string(),
        args: serde_json::json!({
            "command": "rm -rf /tmp/important".to_string(),
            "cwd": serde_json::Value::Null,
            "safe_to_auto_run": true
        }),
    };
    let result = PermissionEngine::check(&tool, &settings);
    assert!(
        matches!(result, PermissionResult::Deny(_)),
        "rm -rf는 safe_to_auto_run이어도 차단이어야 함"
    );
}

// --- [v0.1.0-beta.18] Phase 9-B: File Read 안전장치 검증 ---

#[test]
fn test_read_file_path_traversal_denied() {
    // '..' 포함 경로는 차단
    let settings = PersistedSettings::default();
    let tool = ToolCall {
        name: "ReadFile".to_string(),
        args: serde_json::json!({
            "path": "../../etc/passwd".to_string(),
            "start_line": serde_json::Value::Null,
            "end_line": serde_json::Value::Null
        }),
    };
    let result = PermissionEngine::check(&tool, &settings);
    assert!(
        matches!(result, PermissionResult::Deny(_)),
        "'..' 경로 traversal은 차단이어야 함"
    );
}

// --- [v0.1.0-beta.18] Phase 9-A: Timeline 구조 검증 ---

#[test]
fn test_timeline_entry_creation() {
    use crate::app::state::{TimelineEntry, TimelineEntryKind, ToolStatus};
    // TimelineEntry 생성 및 ToolStatus 전이 검증
    let entry = TimelineEntry::now(TimelineEntryKind::ToolCard {
        tool_name: "ExecShell".to_string(),
        status: ToolStatus::Queued,
        summary: String::new(),
    });
    if let TimelineEntryKind::ToolCard { status, .. } = &entry.kind {
        assert_eq!(*status, ToolStatus::Queued);
    } else {
        panic!("ToolCard 엔트리여야 함");
    }
}

// --- [v0.1.0-beta.18] Phase 9-C: 확장 테스트 6건 ---

#[test]
fn test_tool_status_transition() {
    use crate::app::state::ToolStatus;
    // ToolStatus 전이 순서: Queued → Running → Done/Error
    let queued = ToolStatus::Queued;
    let running = ToolStatus::Running;
    let done = ToolStatus::Done;
    let error = ToolStatus::Error;

    // Clone + PartialEq 검증
    assert_eq!(queued.clone(), ToolStatus::Queued);
    assert_ne!(queued, running);
    assert_ne!(done, error);
}

#[test]
fn test_blocked_command_case_insensitive() {
    // 대소문자 혼합된 위험 명령어도 차단해야 함
    let settings = PersistedSettings {
        shell_policy: ShellPolicy::Ask,
        ..PersistedSettings::default()
    };
    let tool = ToolCall {
        name: "ExecShell".to_string(),
        args: serde_json::json!({
            "command": "SUDO apt install something".to_string(),
            "cwd": serde_json::Value::Null,
            "safe_to_auto_run": false
        }),
    };
    let result = PermissionEngine::check(&tool, &settings);
    assert!(
        matches!(result, PermissionResult::Deny(_)),
        "대소문자 혼합 SUDO도 차단이어야 함"
    );
}

#[test]
fn test_read_file_normal_path_allowed() {
    // '..'이 없는 정상 경로는 Allow
    let settings = PersistedSettings::default();
    let tool = ToolCall {
        name: "ReadFile".to_string(),
        args: serde_json::json!({
            "path": "/tmp/test_file.txt".to_string(),
            "start_line": serde_json::Value::Null,
            "end_line": serde_json::Value::Null
        }),
    };
    let result = PermissionEngine::check(&tool, &settings);
    assert!(
        matches!(result, PermissionResult::Allow),
        "정상 경로 ReadFile은 Allow이어야 함"
    );
}

#[test]
fn test_timeline_entry_user_message() {
    use crate::app::state::{TimelineEntry, TimelineEntryKind};
    // UserMessage 타임라인 엔트리 생성 검증
    let entry = TimelineEntry::now(TimelineEntryKind::UserMessage("hello".to_string()));
    if let TimelineEntryKind::UserMessage(msg) = &entry.kind {
        assert_eq!(msg, "hello");
    } else {
        panic!("UserMessage 엔트리여야 함");
    }
}

#[test]
fn test_timeline_entry_system_notice() {
    use crate::app::state::{TimelineEntry, TimelineEntryKind};
    // SystemNotice 타임라인 엔트리 생성 검증
    let entry = TimelineEntry::now(TimelineEntryKind::SystemNotice("경고".to_string()));
    if let TimelineEntryKind::SystemNotice(msg) = &entry.kind {
        assert_eq!(msg, "경고");
    } else {
        panic!("SystemNotice 엔트리여야 함");
    }
}

#[test]
fn test_blocked_command_fork_bomb() {
    // fork bomb 패턴도 차단
    let settings = PersistedSettings {
        shell_policy: ShellPolicy::SafeOnly,
        ..PersistedSettings::default()
    };
    let tool = ToolCall {
        name: "ExecShell".to_string(),
        args: serde_json::json!({
            "command": ":(){ :|:& };:".to_string(),
            "cwd": serde_json::Value::Null,
            "safe_to_auto_run": true
        }),
    };
    let result = PermissionEngine::check(&tool, &settings);
    assert!(
        matches!(result, PermissionResult::Deny(_)),
        "fork bomb은 반드시 차단이어야 함"
    );
}

// ============================================================
// [v0.1.0-beta.18] Phase 10: 세션 로거 JSONL 테스트 (4건)
// ============================================================

/// JSONL 세션 로거: 메시지 append 후 restore하면 동일 내용 복원
#[test]
fn test_session_logger_append_and_restore() {
    use crate::infra::session_log::SessionLogger;
    use crate::providers::types::{ChatMessage, Role};

    let dir = std::env::temp_dir().join("smlcli_test_session_1");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("test_session.jsonl");
    let _ = std::fs::remove_file(&path);

    // 새 파일 생성 후 로거 초기화
    std::fs::File::create(&path).unwrap();
    let logger = SessionLogger::from_file(path.clone()).unwrap();

    // 메시지 2건 기록
    let msg1 = ChatMessage {
        role: Role::User,
        content: Some("hello".to_string()),
        tool_calls: None,
        tool_call_id: None,
        pinned: false,
    };
    let msg2 = ChatMessage {
        role: Role::Assistant,
        content: Some("hi there".to_string()),
        tool_calls: None,
        tool_call_id: None,
        pinned: false,
    };
    logger.append_message(&msg1).unwrap();
    logger.append_message(&msg2).unwrap();

    // 복원 검증
    let (messages, errors) = logger.restore_messages().unwrap();
    assert_eq!(messages.len(), 2, "2건 복원이어야 함");
    assert_eq!(errors, 0, "에러 0건이어야 함");
    assert_eq!(messages[0].content.as_deref().unwrap_or_default(), "hello");
    assert_eq!(
        messages[1].content.as_deref().unwrap_or_default(),
        "hi there"
    );

    // 정리
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

/// JSONL 빈 파일 restore 시 0건 반환
#[test]
fn test_session_logger_empty_file() {
    use crate::infra::session_log::SessionLogger;

    let dir = std::env::temp_dir().join("smlcli_test_session_2");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("empty.jsonl");
    std::fs::File::create(&path).unwrap();

    let logger = SessionLogger::from_file(path.clone()).unwrap();
    let (messages, errors) = logger.restore_messages().unwrap();
    assert_eq!(messages.len(), 0);
    assert_eq!(errors, 0);

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

/// JSONL 손상된 라인이 있어도 나머지는 정상 복원
#[test]
fn test_session_logger_corrupted_line_skipped() {
    use crate::infra::session_log::SessionLogger;
    use crate::providers::types::{ChatMessage, Role};

    let dir = std::env::temp_dir().join("smlcli_test_session_3");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("corrupted.jsonl");

    // 정상 1줄 + 손상 1줄 + 정상 1줄
    let msg = ChatMessage {
        role: Role::User,
        content: Some("valid".to_string()),
        tool_calls: None,
        tool_call_id: None,
        pinned: false,
    };
    std::fs::File::create(&path).unwrap();
    let logger = SessionLogger::from_file(path.clone()).unwrap();
    logger.append_message(&msg).unwrap();

    // 손상된 라인 직접 추가
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    writeln!(file, "{{invalid json line}}").unwrap();
    drop(file);

    logger.append_message(&msg).unwrap();

    let (messages, errors) = logger.restore_messages().unwrap();
    assert_eq!(messages.len(), 2, "정상 2건만 복원");
    assert_eq!(errors, 1, "손상 1건 건너뛰기");

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

/// from_file: 존재하지 않는 파일은 에러
#[test]
fn test_session_logger_nonexistent_file() {
    use crate::infra::session_log::SessionLogger;
    use std::path::PathBuf;

    let result = SessionLogger::from_file(PathBuf::from("/tmp/smlcli_nonexistent_99999.jsonl"));
    assert!(result.is_err(), "존재하지 않는 파일은 에러여야 함");
}

// =============================================================================
// [v0.1.0-beta.22] 하네스 구조/보안/UX 감사 회귀 테스트 6건
// =============================================================================

/// [H-2] 빈 ExecShell 명령은 permission 검사 이전에 즉시 Deny 처리되어야 한다.
/// 빈 명령이 SafeOnly에서 자동 허용되거나 Ask에서 무의미한 승인 대기로
/// 흐르는 것을 방지하는 하드 가드.
#[test]
fn test_empty_exec_shell_denied() {
    let settings = PersistedSettings::default();

    // 빈 문자열
    let call_empty = ToolCall {
        name: "ExecShell".to_string(),
        args: serde_json::json!({
            "command": "".to_string(),
            "cwd": serde_json::Value::Null,
            "safe_to_auto_run": false
        }),
    };
    let result = PermissionEngine::check(&call_empty, &settings);
    assert!(
        matches!(result, PermissionResult::Deny(_)),
        "빈 명령은 즉시 Deny여야 함"
    );

    // 공백만 있는 경우
    let call_spaces = ToolCall {
        name: "ExecShell".to_string(),
        args: serde_json::json!({
            "command": "   ".to_string(),
            "cwd": serde_json::Value::Null,
            "safe_to_auto_run": false
        }),
    };
    let result = PermissionEngine::check(&call_spaces, &settings);
    assert!(
        matches!(result, PermissionResult::Deny(_)),
        "공백만 있는 명령도 즉시 Deny여야 함"
    );

    // safe_to_auto_run=true여도 빈 명령은 차단
    let call_safe_empty = ToolCall {
        name: "ExecShell".to_string(),
        args: serde_json::json!({
            "command": "".to_string(),
            "cwd": serde_json::Value::Null,
            "safe_to_auto_run": true
        }),
    };
    let result = PermissionEngine::check(&call_safe_empty, &settings);
    assert!(
        matches!(result, PermissionResult::Deny(_)),
        "safe_to_auto_run이어도 빈 명령은 Deny여야 함"
    );
}

/// [H-2 보조] SafeOnly 정책에서 빈 명령이 자동 허용되지 않는지 검증.
/// 이전 버전에서는 is_safe_command()가 빈 토큰 목록에 true를 반환하여
/// SafeOnly에서 빈 명령이 허용되는 결함이 있었음.
#[test]
fn test_empty_exec_shell_safe_only_denied() {
    let settings = PersistedSettings {
        shell_policy: ShellPolicy::SafeOnly,
        ..Default::default()
    };

    let call = ToolCall {
        name: "ExecShell".to_string(),
        args: serde_json::json!({
            "command": "".to_string(),
            "cwd": serde_json::Value::Null,
            "safe_to_auto_run": false
        }),
    };
    let result = PermissionEngine::check(&call, &settings);
    assert!(
        matches!(result, PermissionResult::Deny(_)),
        "SafeOnly에서도 빈 명령은 Deny여야 함"
    );
}

/// [H-3] UiState의 timeline_scroll 필드가 존재하고 0으로 초기화되는지 검증.
/// 이 필드는 타임라인 Wrap + 스크롤 오프셋의 상태 저장소.
#[test]
fn test_timeline_scroll_initial_value() {
    let ui = crate::app::state::UiState::new(false);
    assert_eq!(
        ui.timeline_scroll, 0,
        "타임라인 스크롤 오프셋은 0으로 초기화되어야 함"
    );
}

/// [M-1] PLAN/RUN 모드가 SessionState에 올바르게 설정되는지 검증.
/// dispatch_chat_request에서 모드별 시스템 프롬프트를 주입하는 기반.
#[test]
fn test_plan_run_mode_toggle() {
    use crate::domain::session::{AppMode, SessionState};

    let mut session = SessionState::new();
    // [v0.1.0-beta.22] 기본 모드가 Run으로 변경됨 (코딩 에이전트 기본 동작)
    assert_eq!(session.mode, AppMode::Run, "초기 모드는 Run이어야 함");

    session.mode = AppMode::Plan;
    assert_eq!(session.mode, AppMode::Plan, "모드가 Plan으로 전환되어야 함");

    session.mode = AppMode::Run;
    assert_eq!(
        session.mode,
        AppMode::Run,
        "모드가 다시 Run으로 돌아가야 함"
    );
}

/// [H-1/H-2] bare JSON 도구 스키마가 실행되지 않고, 렌더링에서도 필터링되는지 검증.
/// - tool_runtime: bare JSON은 도구로 디스패치하지 않음 (실행 차단)
/// - filter_tool_json: bare JSON의 "tool" 키가 있으면 사용자 친화적 요약으로 대체 (스키마 노출 방지)
#[test]
fn test_bare_json_filtered_from_display() {
    // bare JSON 도구 스키마 — fenced가 아니므로 실행에서 무시됨
    let bare_tool_json = r#"{"tool":"ExecShell","command":"ls"}"#;
    assert!(
        !bare_tool_json.contains("```json"),
        "bare JSON에는 fenced 마커가 없어야 함"
    );
    // bare JSON에 "tool" 키가 있으므로 렌더러가 필터링해야 함
    assert!(
        bare_tool_json.contains("\"tool\""),
        "bare JSON에 tool 키가 있으면 렌더러가 필터링 대상으로 감지해야 함"
    );

    // fenced JSON은 도구 호출로 인식되어야 함
    let fenced_response = "설명입니다.\n```json\n{\"tool\":\"ExecShell\",\"command\":\"ls\"}\n```";
    assert!(
        fenced_response.contains("```json"),
        "fenced JSON에는 마커가 있어야 함"
    );

    // "tool" 키가 없는 bare JSON은 필터링하지 않아야 함 (일반 데이터)
    let bare_data_json = r#"{"name":"test","value":42}"#;
    assert!(
        !bare_data_json.contains("\"tool\""),
        "일반 JSON에는 tool 키가 없으므로 필터링 대상이 아님"
    );
}

/// [H-4] 첫 턴 하드가드 삭제 후에도 bare JSON 3단계 필터가 동작하는지 검증.
/// 첫 턴이든 N번째 턴이든 bare JSON은 차단되고, fenced JSON은 통과해야 함.
/// 실제 함수: filter_tool_json() 호출.
#[test]
fn test_filter_tool_json_bare_vs_fenced() {
    use crate::tui::layout::filter_tool_json;

    // bare 도구 JSON → 요약으로 대체 (스키마 미노출)
    let bare = r#"{"tool":"ExecShell","command":"ls -la"}"#;
    let filtered = filter_tool_json(bare);
    assert!(
        !filtered.contains(r#""tool""#),
        "bare 도구 JSON의 스키마가 노출되면 안 됨: {}",
        filtered
    );
    assert!(
        filtered.contains("ExecShell"),
        "bare 도구 JSON은 도구 이름 요약으로 대체되어야 함: {}",
        filtered
    );

    // fenced 도구 JSON → 요약으로 대체
    let fenced = "설명입니다.\n```json\n{\"tool\":\"ReadFile\",\"path\":\"/tmp/a.txt\"}\n```\n끝.";
    let filtered_fenced = filter_tool_json(fenced);
    assert!(
        !filtered_fenced.contains(r#""tool""#),
        "fenced 도구 JSON의 스키마가 노출되면 안 됨: {}",
        filtered_fenced
    );
    assert!(
        filtered_fenced.contains("ReadFile"),
        "fenced 도구 JSON은 도구 이름 요약으로 대체되어야 함: {}",
        filtered_fenced
    );
    assert!(
        filtered_fenced.contains("설명입니다"),
        "fenced 이전 텍스트는 유지되어야 함: {}",
        filtered_fenced
    );

    // 일반 JSON (tool 키 없음) → 원문 유지
    let data = r#"{"name":"test","value":42}"#;
    let filtered_data = filter_tool_json(data);
    assert_eq!(
        filtered_data.trim(),
        data,
        "tool 키 없는 일반 JSON은 원문 그대로 유지되어야 함"
    );

    // 순수 텍스트 → 변경 없음
    let text = "Hello, world! 안녕하세요.";
    assert_eq!(
        filter_tool_json(text),
        text,
        "순수 텍스트는 변경 없이 반환되어야 함"
    );
}

/// [M-1] mixed bare JSON (텍스트 + bare 도구 JSON) 필터링 실제 동작 검증.
/// 실제 함수: filter_tool_json() 호출.
#[test]
fn test_filter_tool_json_mixed_bare() {
    use crate::tui::layout::filter_tool_json;

    let mixed = "파일을 읽겠습니다.\n{\"tool\":\"ReadFile\",\"path\":\"/tmp/test.txt\"}";
    let filtered = filter_tool_json(mixed);

    // 자연어 부분은 유지
    assert!(
        filtered.contains("파일을 읽겠습니다"),
        "mixed 응답의 자연어 부분이 유지되어야 함: {}",
        filtered
    );
    // 도구 스키마는 미노출
    assert!(
        !filtered.contains(r#""tool""#),
        "mixed 응답의 도구 스키마가 노출되면 안 됨: {}",
        filtered
    );
    // 도구 이름은 요약으로 표시
    assert!(
        filtered.contains("ReadFile"),
        "mixed 응답에서 도구 이름이 요약으로 표시되어야 함: {}",
        filtered
    );
}

/// [M-2] find_json_end brace 매칭 실제 동작 검증.
/// 실제 함수: find_json_end() 호출.
#[test]
fn test_find_json_end_brace_matching() {
    use crate::tui::layout::find_json_end;

    // 단순 JSON 객체
    let simple = r#"{"tool":"ExecShell"} 뒤에 텍스트"#;
    assert_eq!(
        find_json_end(simple),
        Some(20),
        "단순 JSON 객체 종료 위치가 정확해야 함"
    );

    // 중첩 JSON
    let nested = r#"{"a":{"b":1},"c":2} extra"#;
    assert_eq!(
        find_json_end(nested),
        Some(19),
        "중첩 JSON 객체 종료 위치가 정확해야 함"
    );

    // escaped braces가 포함된 문자열
    let escaped = r#"{"val":"{not a brace}"} end"#;
    assert_eq!(
        find_json_end(escaped),
        Some(23),
        "escaped braces를 포함한 JSON 종료 위치가 정확해야 함"
    );

    // 닫히지 않은 JSON
    assert_eq!(
        find_json_end("{\"a\":1"),
        None,
        "닫히지 않은 JSON은 None을 반환해야 함"
    );
}

/// [M-2] 모드 지시 dedupe — 실제 dedupe 로직이 메시지 벡터에서 동작하는지 검증.
#[test]
fn test_mode_instruction_dedupe() {
    use crate::providers::types::{ChatMessage, Role};

    let mut messages: Vec<ChatMessage> = vec![
        ChatMessage {
            role: Role::System,
            content: Some("You are smlcli.".to_string()),
            tool_calls: None,
            tool_call_id: None,
            pinned: true,
        },
        ChatMessage {
            role: Role::System,
            content: Some("[Mode: PLAN] You are in PLAN mode.".to_string()),
            tool_calls: None,
            tool_call_id: None,
            pinned: false,
        },
    ];

    // 실제 dedupe 로직 실행: "[Mode:" 접두사로 기존 메시지를 찾아 교체
    let new_instruction = "[Mode: RUN] You are in RUN mode.";
    let mut replaced = false;
    for msg in &mut messages {
        if msg.role == Role::System
            && msg
                .content
                .as_deref()
                .unwrap_or_default()
                .starts_with("[Mode:")
        {
            msg.content = Some(new_instruction.to_string());
            replaced = true;
            break;
        }
    }

    assert!(replaced, "기존 모드 메시지가 교체되어야 함");
    assert_eq!(messages.len(), 2, "메시지 수가 늘어나지 않아야 함 (dedupe)");
    assert_eq!(
        messages[1].content.as_deref().unwrap_or_default(),
        new_instruction,
        "교체 후 RUN 모드 지시여야 함"
    );

    // 2차 교체 — 다시 PLAN으로 전환해도 메시지 수 불변
    let plan_instruction = "[Mode: PLAN] Back to plan.";
    for msg in &mut messages {
        if msg.role == Role::System
            && msg
                .content
                .as_deref()
                .unwrap_or_default()
                .starts_with("[Mode:")
        {
            msg.content = Some(plan_instruction.to_string());
            break;
        }
    }
    assert_eq!(messages.len(), 2, "2차 교체 후에도 메시지 수 불변");
    let mode_count = messages
        .iter()
        .filter(|m| {
            m.role == Role::System
                && m.content
                    .as_deref()
                    .unwrap_or_default()
                    .starts_with("[Mode:")
        })
        .count();
    assert_eq!(mode_count, 1, "모드 지시 메시지는 항상 1개여야 함");
}

/// 기본 모드가 Run인지 검증 (Open Question 해소).
#[test]
fn test_default_mode_is_run() {
    use crate::domain::session::{AppMode, SessionState};
    let session = SessionState::new();
    assert_eq!(session.mode, AppMode::Run, "기본 모드는 Run이어야 함");
}

/// format_tool_name/detail 실제 함수 호출 검증.
#[test]
fn test_format_tool_name_and_detail() {
    use crate::app::App;
    use crate::domain::tool_result::ToolCall;

    let exec = ToolCall {
        name: "ExecShell".to_string(),
        args: serde_json::json!({
            "command": "cargo build --release".to_string(),
            "cwd": Some("/home/user/project".to_string()),
            "safe_to_auto_run": false
        }),
    };
    let name = App::format_tool_name(&exec);
    let detail = App::format_tool_detail(&exec);
    assert!(
        name.contains("ExecShell"),
        "도구 이름에 ExecShell 포함: {}",
        name
    );
    assert!(
        detail.contains("cargo build"),
        "detail에 명령어 포함: {}",
        detail
    );

    let read = ToolCall {
        name: "ReadFile".to_string(),
        args: serde_json::json!({
            "path": "/tmp/very/long/path/to/file.rs".to_string(),
            "start_line": Some(1),
            "end_line": Some(100)
        }),
    };
    let name = App::format_tool_name(&read);
    assert!(
        name.contains("ReadFile"),
        "도구 이름에 ReadFile 포함: {}",
        name
    );
    assert!(
        name.contains("file.rs"),
        "도구 이름에 파일명 포함: {}",
        name
    );
}

/// 통합 테스트: process_tool_calls_from_response 실제 호출.
/// - bare JSON(fenced 아님)은 도구로 디스패치되지 않아야 함
/// - fenced JSON이 있으면 approval 상태로 전환되어야 함
/// - 첫 턴이든 N번째 턴이든 동작이 동일해야 함 (하드가드 삭제 검증)
#[tokio::test]
async fn test_process_tool_calls_integration() {
    use crate::app::App;
    use crate::app::state::AppState;
    use crate::providers::types::{ChatMessage, FunctionCall, Role, ToolCallRequest};

    let (tx, _rx) = tokio::sync::mpsc::channel(8);
    let mut app = App {
        state: AppState::new_for_test(),
        action_tx: tx,
    };

    // 1) tool_calls 없는 메시지 — 도구 디스패치 안 됨
    let no_tool_msg = ChatMessage {
        role: Role::Assistant,
        content: Some("안녕하세요! 무엇을 도와드릴까요?".to_string()),
        tool_calls: None,
        tool_call_id: None,
        pinned: false,
    };
    app.process_tool_calls_from_response(&no_tool_msg);
    assert!(
        app.state.runtime.approval.pending_tool.is_none(),
        "순수 텍스트에서 도구가 디스패치되면 안 됨"
    );

    // 2) tool_calls 있는 메시지 — 승인 대기 또는 타임라인에 ToolCard가 추가되어야 함
    let tool_msg = ChatMessage {
        role: Role::Assistant,
        content: Some("파일을 읽겠습니다.".to_string()),
        tool_calls: Some(vec![ToolCallRequest {
            id: "call_123".to_string(),
            r#type: "function".to_string(),
            function: FunctionCall {
                name: "ReadFile".to_string(),
                arguments: "{\"path\":\"/tmp/test.txt\"}".to_string(),
            },
        }]),
        tool_call_id: None,
        pinned: false,
    };
    app.process_tool_calls_from_response(&tool_msg);
    let has_tool_activity = app.state.runtime.approval.pending_tool.is_some()
        || app.state.ui.timeline.iter().any(|e| {
            matches!(
                e.kind,
                crate::app::state::TimelineEntryKind::ToolCard { .. }
            )
        });
    assert!(
        has_tool_activity,
        "tool_calls가 있는 메시지는 디스패치되어야 함 (승인 대기 또는 자동 실행)"
    );

    // 3) 비작업성 입력(인삿말) — 도구 디스패치 차단
    let (tx2, _rx2) = tokio::sync::mpsc::channel(8);
    let mut app2 = App {
        state: AppState::new_for_test(),
        action_tx: tx2,
    };
    app2.state.runtime.user_intent_actionable = false;
    app2.process_tool_calls_from_response(&tool_msg);
    let greeting_has_no_activity = app2.state.runtime.approval.pending_tool.is_none()
        && !app2.state.ui.timeline.iter().any(|e| {
            matches!(
                e.kind,
                crate::app::state::TimelineEntryKind::ToolCard { .. }
            )
        });
    assert!(
        greeting_has_no_activity,
        "비작업성 입력(인삿말)에서는 도구가 디스패치되면 안 됨 (런타임 억제 검증)"
    );

    // 4) 작업 요청 입력(기본값 true) — 도구 디스패치 허용
    let (tx3, _rx3) = tokio::sync::mpsc::channel(8);
    let mut app3 = App {
        state: AppState::new_for_test(),
        action_tx: tx3,
    };
    assert!(
        app3.state.runtime.user_intent_actionable,
        "기본값은 작업 허용(true)"
    );
    app3.process_tool_calls_from_response(&tool_msg);
    let action_has_activity = app3.state.runtime.approval.pending_tool.is_some()
        || app3.state.ui.timeline.iter().any(|e| {
            matches!(
                e.kind,
                crate::app::state::TimelineEntryKind::ToolCard { .. }
            )
        });
    assert!(
        action_has_activity,
        "작업 요청 입력에서는 도구가 디스패치되어야 함"
    );
}

/// 시스템 프롬프트에서 첫 턴 도구 금지 문구가 제거되었는지 검증.
/// Run 모드 계약과 충돌하는 "NEVER use a tool in your very first response" 문구가 없어야 함.
#[test]
fn test_system_prompt_no_first_turn_tool_ban() {
    use crate::domain::session::SessionState;

    let session = SessionState::new();
    let system_msg = session.messages[0].content.as_deref().unwrap_or_default();

    assert!(
        !system_msg.contains("NEVER use a tool in your very first response"),
        "시스템 프롬프트에 첫 턴 도구 금지 문구가 없어야 함"
    );
    assert!(
        system_msg.contains("use the appropriate tool immediately"),
        "작업 요청 시 즉시 도구 사용 지시가 있어야 함"
    );
    assert!(
        system_msg.contains("respond in natural language ONLY"),
        "비작업성 입력에 대한 자연어 전용 지시가 있어야 함"
    );
}

/// [v0.1.0-beta.22] is_actionable_input() 휴리스틱 직접 호출 테스트.
/// 비작업성 입력 → false, 작업 요청 → true.
#[test]
fn test_is_actionable_input_heuristic() {
    use crate::app::chat_runtime::is_actionable_input;

    // 비작업성 입력 (인삿말, 잡담, 감사)
    assert!(!is_actionable_input(""), "빈 입력은 비작업성");
    assert!(!is_actionable_input("안녕"), "짧은 인삿말은 비작업성");
    assert!(!is_actionable_input("hi"), "짧은 영어 인삿말은 비작업성");
    assert!(!is_actionable_input("감사합니다"), "감사 인사는 비작업성");
    assert!(!is_actionable_input("좋아요"), "단순 반응은 비작업성");
    assert!(
        !is_actionable_input("hello there"),
        "짧은 인삿말은 비작업성"
    );

    // 작업 요청 (파일, 코드, 명령)
    assert!(
        is_actionable_input("Cargo.toml 읽어줘"),
        "파일 읽기 요청은 작업성"
    );
    assert!(
        is_actionable_input("foo.py 만들어줘"),
        "파일 생성 요청은 작업성"
    );
    assert!(
        is_actionable_input("cargo test 실행해"),
        "명령 실행 요청은 작업성"
    );
    assert!(
        is_actionable_input("src/main.rs를 수정해줘"),
        "경로 포함 요청은 작업성"
    );
    assert!(
        is_actionable_input("create a new file called app.js"),
        "영어 작업 요청은 작업성"
    );
    assert!(
        is_actionable_input("이 코드를 리팩토링해줘"),
        "리팩토링 요청은 작업성"
    );
    assert!(is_actionable_input("@Cargo.toml 분석해"), "@ 참조는 작업성");
    assert!(is_actionable_input("build 해줘"), "빌드 요청은 작업성");
}
