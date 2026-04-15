// [v0.1.0-beta.14] 아키텍처 변경: keyring 제거 → 파일 기반 설정 + 암호화 키.
// PersistedSettings에 encrypted_keys 필드 추가.
// 설정은 ~/.smlcli/config.yaml에 YAML 평문으로 저장되되,
// API 키만 ChaCha20Poly1305로 암호화하여 encrypted_keys 맵에 보관.

use super::permissions::{FileWritePolicy, NetworkPolicy, ShellPolicy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// API 키를 암호화된 형태로 보관하는 맵.
    /// 키: "openrouter_key", "google_key" 등 provider별 alias.
    /// 값: "hex_nonce:hex_ciphertext" 형식의 암호화된 문자열.
    /// 복호화는 ~/.smlcli/.master_key를 사용하여 infra::secret_store에서 수행.
    #[serde(default)]
    pub encrypted_keys: HashMap<String, String>,
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
            encrypted_keys: HashMap::new(),
        }
    }
}
