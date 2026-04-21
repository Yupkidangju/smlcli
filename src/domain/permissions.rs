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

pub enum PermissionResult {
    Allow,
    Ask,
    Deny(String),
}

pub struct PermissionEngine;

impl PermissionEngine {
    fn is_dangerous(args_str: &str) -> bool {
        // [v1.2.0] 고도화된 쉘 인젝션 차단 (명령어 치환 및 멀티라인 실행 우회)
        if let Ok(re) = regex::Regex::new(r"[;&|>`\$\(\)\n\r]") && re.is_match(args_str) {
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

        rm_wildcard.matches(args_str) || rm_root.matches(args_str) || dir_traversal.matches(args_str) || home_access.matches(args_str)
    }

    pub fn check(call: &ToolCall, settings: &PersistedSettings) -> PermissionResult {
        // [v1.0.0] 보안 검증: 와일드카드 우회, 디렉토리 횡단, 쉘 체이닝 문법 차단
        if let Ok(args_str) = serde_json::to_string(&call.args) && Self::is_dangerous(&args_str) {
            return PermissionResult::Deny("보안상 허용되지 않은 명령 형식이 포함되었습니다. (Dangerous pattern detected: [;&|>`$()\\n\\r], *, ../, ~/). Execution blocked.".to_string());
        }

        if matches!(call.name.as_str(), "WriteFile" | "ReplaceFileContent" | "ExecShell") {
            let root = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            if settings.denied_roots.contains(&root) {
                return PermissionResult::Deny("Workspace is not trusted. Modifying files or executing commands is blocked.".to_string());
            }
            match settings.get_workspace_trust(&root) {
                crate::domain::settings::WorkspaceTrustState::Restricted => {
                    return PermissionResult::Deny("Workspace is not trusted. Modifying files or executing commands is blocked.".to_string());
                }
                crate::domain::settings::WorkspaceTrustState::Unknown => {
                    return PermissionResult::Deny("Workspace trust is unknown. Please select trust level in the UI.".to_string());
                }
                crate::domain::settings::WorkspaceTrustState::Trusted => {}
            }
        }

        let base_result = if let Some(tool) = crate::tools::registry::GLOBAL_REGISTRY.get_tool(&call.name) {
            tool.check_permission(&call.args, settings)
        } else {
            PermissionResult::Deny(format!("Unknown tool: {}", call.name))
        };

        // [v1.0.0] Whitelist 방식 병행: 허용된 바이너리 외의 커맨드는 명시적 사용자 승인(AskUser)을 거치도록 강제
        if call.name == "ExecShell" && let PermissionResult::Allow = base_result {
            let command = call.args.get("command").and_then(|v| v.as_str()).unwrap_or("");
            
            // [v1.2.0] PathGuard: 민감 명령어 + 민감 경로 조합 차단
            if command.contains("sudo ") || command.contains("rm ") {
                let sensitive_paths = ["/etc", "/var", "/bin", "/sbin", "/usr", "/boot", "/dev", "/sys", "/proc"];
                for path in &sensitive_paths {
                    if command.contains(path) {
                        return PermissionResult::Deny(format!("접근 금지 경로를 포함하는 권한 상승/파일 삭제 명령은 차단됩니다: {}", path));
                    }
                }
            }

            let first_word = command.split_whitespace().next().unwrap_or("");
            let whitelist = [
                "git", "ls", "grep", "cat", "echo", "pwd", "date", "df", "free", "uname", "dir", "type", "find"
            ];
            if !whitelist.contains(&first_word) {
                return PermissionResult::Ask;
            }
        }

        base_result
    }
}
