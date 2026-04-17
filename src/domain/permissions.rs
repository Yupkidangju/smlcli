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
        if let Some(tool) = crate::tools::registry::GLOBAL_REGISTRY.get_tool(&call.name) {
            tool.check_permission(&call.args, settings)
        } else {
            PermissionResult::Deny(format!("Unknown tool: {}", call.name))
        }
    }
}
