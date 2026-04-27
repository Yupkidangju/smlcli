// [v0.1.0-beta.14] 아키텍처 변경: keyring 제거 → 파일 기반 설정 + 암호화 키.
// PersistedSettings에 encrypted_keys 필드 추가.
// 설정은 ~/.smlcli/config.toml에 TOML 평문으로 저장되되,
// API 키만 ChaCha20Poly1305로 암호화하여 encrypted_keys 맵에 보관.
// [v0.1.0-beta.20] theme 필드 추가: "default" 또는 "high_contrast".
//   designs.md §21.4 요구사항 반영.

use super::permissions::{FileWritePolicy, NetworkPolicy, ShellPolicy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceTrustState {
    #[default]
    Unknown,
    Trusted,
    Restricted,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceTrustRecord {
    pub root_path: String,
    pub state: WorkspaceTrustState,
    pub remember: bool,
    pub updated_at_unix_ms: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersistedSettings {
    #[serde(default = "default_version")]
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
    /// [v0.1.0-beta.20] UI 테마 설정.
    /// "default" 또는 "high_contrast". designs.md §21 참조.
    #[serde(default = "default_theme")]
    pub theme: String,
    /// [Workspace Trust] Workspace root별 신뢰 상태 레코드 목록.
    #[serde(default)]
    pub trusted_workspaces: Vec<WorkspaceTrustRecord>,
    #[serde(default)]
    pub denied_roots: Vec<String>,
    #[serde(default)]
    pub extra_workspace_dirs: Vec<String>,
    /// [v2.3.0] Phase 31: Configurable Whitelist for Environment Variables
    #[serde(default)]
    pub allowed_env_vars: Vec<String>,
    /// [v2.5.0] Phase 33: ASCII Border Fallback (로케일 호환성)
    #[serde(default)]
    pub use_ascii_borders: bool,
    /// [v3.0.0] Phase 40: Git-Native Integration 설정
    #[serde(default)]
    pub git_integration: GitIntegrationConfig,
    /// [v3.1.0] Phase 41: 커스텀 Provider 리스트
    #[serde(default)]
    pub custom_providers: Vec<crate::domain::provider::CustomProviderConfig>,
    /// [v3.2.0] Phase 42: OS-Level Sandbox 설정
    #[serde(default)]
    pub sandbox: SandboxConfig,
    /// [v3.3.0] Phase 43: MCP 클라이언트 서버 설정
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

/// [v3.0.0] Git 통합 설정
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitIntegrationConfig {
    /// 도구 실행 성공 시 자동 커밋 여부 (기본: false, 명시적 opt-in 필요)
    pub auto_commit: bool,
    /// 커밋 메시지 접두사 (기본: "smlcli: ")
    pub commit_prefix: String,
    /// 자동 커밋 대상 도구 목록
    pub commit_tools: Vec<String>,
}

impl Default for GitIntegrationConfig {
    fn default() -> Self {
        Self {
            // [v2.5.2] 감사 MEDIUM-2: 기본값 false로 변경.
            // 사용자 워크트리에 직접 영향을 주는 자동 커밋은 명시적 opt-in이 더 안전한 UX.
            auto_commit: false,
            commit_prefix: "smlcli: ".to_string(),
            commit_tools: vec![
                "WriteFile".to_string(),
                "ReplaceFileContent".to_string(),
                "DeleteFile".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enabled: bool,
    #[serde(default)]
    pub allow_network: bool,
    #[serde(default)]
    pub extra_binds: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allow_network: true,
            extra_binds: Vec::new(),
        }
    }
}

/// theme 필드의 기본값: "default"
fn default_theme() -> String {
    "default".to_string()
}

fn default_version() -> u32 {
    1
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
            theme: default_theme(),
            trusted_workspaces: Vec::new(),
            denied_roots: Vec::new(),
            extra_workspace_dirs: Vec::new(),
            allowed_env_vars: Vec::new(),
            use_ascii_borders: false,
            git_integration: GitIntegrationConfig::default(),
            custom_providers: Vec::new(),
            sandbox: SandboxConfig::default(),
            mcp_servers: Vec::new(),
        }
    }
}

impl PersistedSettings {
    pub fn get_workspace_trust(&self, root: &str) -> WorkspaceTrustState {
        self.trusted_workspaces
            .iter()
            .find(|r| r.root_path == root)
            .map(|r| r.state.clone())
            .unwrap_or(WorkspaceTrustState::Unknown)
    }

    pub fn set_workspace_trust(&mut self, root: &str, state: WorkspaceTrustState, remember: bool) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if let Some(record) = self
            .trusted_workspaces
            .iter_mut()
            .find(|r| r.root_path == root)
        {
            record.state = state;
            record.remember = remember;
            record.updated_at_unix_ms = now;
        } else {
            self.trusted_workspaces.push(WorkspaceTrustRecord {
                root_path: root.to_string(),
                state,
                remember,
                updated_at_unix_ms: now,
            });
        }
    }

    pub fn remove_workspace_trust(&mut self, root: &str) {
        self.trusted_workspaces.retain(|r| r.root_path != root);
    }

    /// [v2.2.0] Phase 30: Config Schema Auto-Migration
    pub fn migrate(&mut self) -> bool {
        let mut migrated = false;
        if self.version < 1 {
            // v0 -> v1 승격 로직
            self.version = 1;
            migrated = true;
        }
        migrated
    }
}
