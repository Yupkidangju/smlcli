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
    pub fn check(call: &ToolCall, settings: &PersistedSettings) -> PermissionResult {
        match call {
            ToolCall::ReadFile { .. }
            | ToolCall::ListDir { .. }
            | ToolCall::GrepSearch { .. }
            | ToolCall::Stat { .. }
            | ToolCall::SysInfo => {
                // Read-only tools are always allowed
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
            } => match settings.shell_policy {
                ShellPolicy::Deny => {
                    PermissionResult::Deny("Shell execution is disabled by policy.".to_string())
                }
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
            },
        }
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
