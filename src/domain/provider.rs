use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ProviderKind {
    OpenRouter,
    Google,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProviderProfile {
    pub id: String,
    pub kind: ProviderKind,
    pub api_key_alias: String,
    pub default_model: String,
}
