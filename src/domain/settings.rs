use serde::{Deserialize, Serialize};
use super::permissions::{FileWritePolicy, NetworkPolicy, ShellPolicy};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersistedSettings {
    pub version: u32,
    pub default_provider: String,
    pub default_model: String,
    pub shell_policy: ShellPolicy,
    pub file_write_policy: FileWritePolicy,
    pub network_policy: NetworkPolicy,
    #[serde(default)]
    pub safe_commands: Option<Vec<String>>,
}

impl Default for PersistedSettings {
    fn default() -> Self {
        Self {
            version: 1,
            default_provider: "OpenRouter".to_string(),
            default_model: "auto".to_string(),
            shell_policy: ShellPolicy::Ask,
            file_write_policy: FileWritePolicy::AlwaysAsk,
            network_policy: NetworkPolicy::ProviderOnly,
            safe_commands: None,
        }
    }
}
