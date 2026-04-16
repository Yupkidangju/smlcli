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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum NetworkPolicy {
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
    /// [v0.1.0-beta.18] Phase 9-B: 위험 명령어 블랙리스트.
    /// 이 패턴에 매칭되는 명령어는 정책(Ask/SafeOnly)에 관계없이 무조건 차단.
    const BLOCKED_PATTERNS: &'static [&'static str] = &[
        "sudo ",
        "rm -rf",
        "rm -fr",
        "chmod 777",
        "chmod -R 777",
        "mkfs",
        "dd if=",
        "> /dev/",
        ":(){ :|:&", // fork bomb
        "shutdown",
        "reboot",
        "init 0",
        "init 6",
        "format c:",  // Windows
        "del /f /s",  // Windows
    ];

    /// [v0.1.0-beta.18] Phase 9-B: 파일 읽기 안전 제한.
    /// 1MB 초과 파일은 차단, 경로 정규화로 traversal 방지.
    const MAX_READ_FILE_SIZE: u64 = 1_048_576; // 1MB

    pub fn check(call: &ToolCall, settings: &PersistedSettings) -> PermissionResult {
        match call {
            ToolCall::ReadFile { path, .. } => {
                // [v0.1.0-beta.18] Phase 9-B: 파일 읽기 안전장치
                // 경로 정규화: .. traversal 방지
                let canonical = std::path::Path::new(path);
                if path.contains("..") {
                    return PermissionResult::Deny(format!(
                        "경로에 '..'이 포함되어 있어 차단됩니다: {}",
                        path
                    ));
                }
                // 파일 크기 제한 (1MB)
                if let Ok(meta) = std::fs::metadata(canonical) {
                    if meta.len() > Self::MAX_READ_FILE_SIZE {
                        return PermissionResult::Deny(format!(
                            "파일이 1MB를 초과합니다 ({} bytes): {}",
                            meta.len(),
                            path
                        ));
                    }
                }
                PermissionResult::Allow
            }
            ToolCall::ListDir { .. }
            | ToolCall::GrepSearch { .. }
            | ToolCall::Stat { .. }
            | ToolCall::SysInfo => {
                // 읽기 전용 도구는 항상 허용
                PermissionResult::Allow
            }
            ToolCall::WriteFile { .. } | ToolCall::ReplaceFileContent { .. } => {
                match settings.file_write_policy {
                    FileWritePolicy::AlwaysAsk => PermissionResult::Ask,
                    FileWritePolicy::SessionAllow => PermissionResult::Allow,
                }
            }
            ToolCall::ExecShell {
                command,
                safe_to_auto_run,
                ..
            } => {
                // [v0.1.0-beta.18] Phase 9-B: 블랙리스트 검사를 최우선으로 실행
                if Self::is_blocked_command(command) {
                    return PermissionResult::Deny(format!(
                        "위험 명령어로 차단됨: '{}'",
                        command.chars().take(60).collect::<String>()
                    ));
                }

                match settings.shell_policy {
                    ShellPolicy::Deny => PermissionResult::Deny(
                        "Shell execution is disabled by policy.".to_string(),
                    ),
                    ShellPolicy::Ask => PermissionResult::Ask,
                    ShellPolicy::SafeOnly => {
                        if *safe_to_auto_run || Self::is_safe_command(command, settings) {
                            PermissionResult::Allow
                        } else {
                            PermissionResult::Deny(format!(
                                "Command '{}' is blocked in SafeOnly mode.",
                                command
                            ))
                        }
                    }
                }
            }
        }
    }

    /// [v0.1.0-beta.18] Phase 9-B: 위험 명령어 블랙리스트 매칭.
    /// BLOCKED_PATTERNS 중 하나라도 포함되면 true 반환.
    fn is_blocked_command(cmd: &str) -> bool {
        let lower = cmd.to_lowercase();
        Self::BLOCKED_PATTERNS
            .iter()
            .any(|pattern| lower.contains(pattern))
    }

    fn is_safe_command(cmd: &str, settings: &PersistedSettings) -> bool {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return true;
        }

        if let Some(custom_list) = &settings.safe_commands {
            return custom_list.iter().any(|c| c == parts[0]);
        }

        let os = std::env::consts::OS;
        let safe_list = if os == "windows" {
            vec!["dir", "echo", "date", "cd", "type", "find"]
        } else {
            vec![
                "ls", "pwd", "date", "echo", "cat", "grep", "df", "free", "uname",
            ]
        };
        safe_list.contains(&parts[0])
    }
}
