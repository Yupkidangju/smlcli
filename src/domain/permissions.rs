use crate::domain::settings::PersistedSettings;
use crate::domain::tool_result::ToolCall;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ShellPolicy {
    Ask,
    SafeOnly,
    Deny,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum FileWritePolicy {
    AlwaysAsk,
    SessionAllow,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum NetworkPolicy {
    #[default]
    AllowAll,
    ProviderOnly,
    Deny,
}

#[derive(Debug, Clone)]
pub struct PermissionToken {
    _private: (),
}

impl PermissionToken {
    pub(crate) fn grant() -> Self {
        Self { _private: () }
    }
}

#[derive(Debug)]
pub enum PermissionResult {
    Allow,
    Ask,
    Deny(String),
}

pub struct PermissionEngine;

impl PermissionEngine {
    fn is_dangerous(args_str: &str) -> bool {
        // [v1.2.0] 고도화된 쉘 인젝션 차단 (명령어 치환 및 멀티라인 실행 우회)
        if let Ok(re) = regex::Regex::new(r"[;&|>`\$\(\)\n\r]")
            && re.is_match(args_str)
        {
            return true;
        }

        use glob::Pattern;
        // 파괴적인 명령과 와일드카드 조합
        // [*]를 사용하여 리터럴 * 매칭
        let rm_wildcard = Pattern::new("*rm *-rf *[*]*").unwrap();
        let rm_root = Pattern::new("*rm *-rf /*").unwrap();

        // 디렉토리 횡단 공격
        let dir_traversal = Pattern::new("*../*").unwrap();

        // 홈 디렉토리 무단 접근
        let home_access = Pattern::new("*~/*").unwrap();

        rm_wildcard.matches(args_str)
            || rm_root.matches(args_str)
            || dir_traversal.matches(args_str)
            || home_access.matches(args_str)
    }

    /// [v2.5.0] 경로 전용 횡단/홈 접근 검사.
    /// 쓰기 도구의 path 인자에만 적용하여, 셸 인젝션 문자(;, &, | 등)와 분리.
    fn has_path_traversal(path: &str) -> bool {
        path.contains("../") || path.contains("..\\") || path.starts_with("~/")
    }

    pub fn check(call: &ToolCall, settings: &PersistedSettings) -> PermissionResult {
        // [v2.5.0] 도구별 위험 패턴 검사 분리.
        // ExecShell: 전체 command 인자에 인젝션 패턴 검사.
        // WriteFile/ReplaceFileContent: path 인자만 디렉토리 횡단 검사.
        // FetchURL/GrepSearch 등 읽기 전용 도구: 위험 패턴 미적용 (URL query string, regex 과차단 방지).
        match call.name.as_str() {
            "ExecShell" => {
                // 셸 명령어 전체에 인젝션 패턴 적용
                if call
                    .args
                    .get("command")
                    .and_then(|v| v.as_str())
                    .is_some_and(Self::is_dangerous)
                {
                    return PermissionResult::Deny(
                        "보안상 허용되지 않은 명령 형식이 포함되었습니다. (Dangerous pattern detected). Execution blocked.".to_string(),
                    );
                }
                // [v2.5.0] cwd 인자에도 경로 횡단/workspace 밖 접근 검사.
                // resolve_shell_cwd()가 런타임에서 추가 검증하지만, 권한 엔진에서도 선제 차단.
                if let Some(cwd_val) = call.args.get("cwd").and_then(|v| v.as_str()) {
                    // 1) 상대 경로 횡단 패턴(../, ~/) 선제 차단
                    if Self::has_path_traversal(cwd_val) {
                        return PermissionResult::Deny(
                            "ExecShell의 cwd에 디렉토리 횡단 패턴이 감지되어 차단합니다."
                                .to_string(),
                        );
                    }
                    // [v2.5.1] 2) 절대경로 workspace 이탈 선제 차단.
                    // resolve_shell_cwd/sandbox에도 방어층이 있지만, 권한 엔진에서도
                    // 명시적으로 검사하여 이중 방어(Defense-in-Depth)를 형성.
                    if std::path::Path::new(cwd_val).is_absolute()
                        && let Ok(workspace) = std::env::current_dir()
                    {
                        let canon_ws = std::fs::canonicalize(&workspace).unwrap_or(workspace);
                        let path_to_check = std::fs::canonicalize(cwd_val)
                            .unwrap_or_else(|_| std::path::PathBuf::from(cwd_val));

                        if !path_to_check.starts_with(&canon_ws) {
                            return PermissionResult::Deny(format!(
                                "ExecShell의 cwd '{}'가 workspace '{}' 밖의 절대경로입니다. 차단합니다.",
                                cwd_val,
                                canon_ws.display()
                            ));
                        }
                    }
                }
            }
            "WriteFile" | "ReplaceFileContent" | "DeleteFile" => {
                // 쓰기/삭제 도구는 path에 대해서만 디렉토리 횡단/홈 접근 검사
                if call
                    .args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .is_some_and(Self::has_path_traversal)
                {
                    return PermissionResult::Deny(
                        "경로에 디렉토리 횡단 패턴이 감지되어 차단합니다.".to_string(),
                    );
                }
            }
            _ => {
                // FetchURL, GrepSearch, ReadFile 등 읽기 전용 도구는 위험 패턴 미적용
            }
        }

        if matches!(
            call.name.as_str(),
            "WriteFile" | "ReplaceFileContent" | "DeleteFile" | "ExecShell"
        ) {
            let root = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            if settings.denied_roots.contains(&root) {
                return PermissionResult::Deny(
                    "Workspace is not trusted. Modifying files or executing commands is blocked."
                        .to_string(),
                );
            }
            match settings.get_workspace_trust(&root) {
                crate::domain::settings::WorkspaceTrustState::Restricted => {
                    return PermissionResult::Deny("Workspace is not trusted. Modifying files or executing commands is blocked.".to_string());
                }
                crate::domain::settings::WorkspaceTrustState::Unknown => {
                    return PermissionResult::Deny(
                        "Workspace trust is unknown. Please select trust level in the UI."
                            .to_string(),
                    );
                }
                crate::domain::settings::WorkspaceTrustState::Trusted => {}
            }
        }

        let base_result = if call.name.starts_with("mcp_") {
            PermissionResult::Ask
        } else if let Some(tool) = crate::tools::registry::GLOBAL_REGISTRY.get_tool(&call.name) {
            tool.check_permission(&call.args, settings)
        } else {
            PermissionResult::Deny(format!("Unknown tool: {}", call.name))
        };

        // [v1.0.0] Whitelist 방식 병행: 허용된 바이너리 외의 커맨드는 명시적 사용자 승인(AskUser)을 거치도록 강제
        if call.name == "ExecShell"
            && let PermissionResult::Allow = base_result
        {
            let command = call
                .args
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // [v1.2.0] PathGuard: 민감 명령어 + 민감 경로 조합 차단
            if command.contains("sudo ") || command.contains("rm ") {
                let sensitive_paths = [
                    "/etc", "/var", "/bin", "/sbin", "/usr", "/boot", "/dev", "/sys", "/proc",
                ];
                for path in &sensitive_paths {
                    if command.contains(path) {
                        return PermissionResult::Deny(format!(
                            "접근 금지 경로를 포함하는 권한 상승/파일 삭제 명령은 차단됩니다: {}",
                            path
                        ));
                    }
                }
            }

            let first_word = command.split_whitespace().next().unwrap_or("");
            let whitelist = [
                "git", "ls", "grep", "cat", "echo", "pwd", "date", "df", "free", "uname", "dir",
                "type", "find",
            ];
            if !whitelist.contains(&first_word) {
                return PermissionResult::Ask;
            }
        }

        base_result
    }
}
