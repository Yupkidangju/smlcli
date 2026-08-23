use crate::domain::settings::{PersistedSettings, WorkspaceTrustState};
use serde::{Deserialize, Serialize};

pub const WORKSPACE_GUEST_ROOT: &str = "/workspace";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceHarnessSnapshot {
    pub os: String,
    pub arch: String,
    pub host_shell: String,
    pub exec_shell: String,
    pub canonical_root: String,
    pub trust_state: WorkspaceTrustState,
    pub denied: bool,
    pub extra_workspace_dirs: Vec<String>,
    pub sandbox_enabled: bool,
    pub sandbox_backend: String,
    pub sandbox_guest_root: String,
    pub sandbox_allow_network: bool,
    pub sandbox_extra_binds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessPromptContext {
    pub snapshot: WorkspaceHarnessSnapshot,
    pub prompt_block: String,
    pub generated_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessPreflightInput {
    pub tool_name: String,
    pub command: Option<String>,
    pub requested_cwd: Option<String>,
    pub requested_paths: Vec<String>,
    pub snapshot: WorkspaceHarnessSnapshot,
    pub baseline: Option<WorkspaceHarnessSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessPreflightDecision {
    Allow,
    Ask { reason: String },
    Deny { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHarnessRecord {
    pub kind: String,
    pub session_id: String,
    pub snapshot: WorkspaceHarnessSnapshot,
    pub recorded_at_unix_ms: u64,
}

impl WorkspaceHarnessSnapshot {
    pub fn collect(settings: Option<&PersistedSettings>) -> Self {
        let canonical_root = canonical_workspace_root();
        let sandbox_enabled = settings.is_some_and(|s| s.sandbox.enabled);
        let sandbox_allow_network = settings.map(|s| s.sandbox.allow_network).unwrap_or(true);
        let sandbox_extra_binds = settings
            .map(|s| s.sandbox.extra_binds.clone())
            .unwrap_or_default();
        let trust_state = settings
            .map(|s| s.get_workspace_trust(&canonical_root))
            .unwrap_or_default();
        let denied = settings
            .map(|s| s.denied_roots.iter().any(|root| root == &canonical_root))
            .unwrap_or(false);
        let extra_workspace_dirs = settings
            .map(|s| s.extra_workspace_dirs.clone())
            .unwrap_or_default();

        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            host_shell: detect_host_shell(),
            exec_shell: detect_exec_shell(sandbox_enabled),
            canonical_root,
            trust_state,
            denied,
            extra_workspace_dirs,
            sandbox_enabled,
            sandbox_backend: detect_sandbox_backend(),
            sandbox_guest_root: WORKSPACE_GUEST_ROOT.to_string(),
            sandbox_allow_network,
            sandbox_extra_binds,
        }
    }

    pub fn format_report(&self) -> String {
        let guest_root = if self.sandbox_enabled {
            self.sandbox_guest_root.clone()
        } else {
            format!("{} (policy; inactive)", self.sandbox_guest_root)
        };
        format!(
            "OS: {} ({})\nHost Shell: {}\nExec Shell: {}\nWorkspace Root: {}\nTrust Level: {:?}\nDenied: {}\nExtra Workspace Dirs: {}\nSandbox Enabled: {}\nSandbox Backend: {}\nSandbox Guest Root: {}\nSandbox Network: {}\nSandbox Extra Binds: {}",
            self.os,
            self.arch,
            self.host_shell,
            self.exec_shell,
            self.canonical_root,
            self.trust_state,
            self.denied,
            format_list(&self.extra_workspace_dirs),
            self.sandbox_enabled,
            self.sandbox_backend,
            guest_root,
            if self.sandbox_allow_network {
                "allowed"
            } else {
                "isolated"
            },
            format_list(&self.sandbox_extra_binds)
        )
    }

    pub fn prompt_block(&self) -> String {
        format!(
            "[Workspace Harness]\nOS={} arch={}\nHostShell={} ExecShell={}\nWorkspaceRoot={}\nTrust={:?} Denied={}\nSandboxEnabled={} Backend={} GuestRoot={} Network={}\nRule: host paths are for local file APIs; sandbox shell cwd is /workspace only when sandbox is enabled.",
            self.os,
            self.arch,
            self.host_shell,
            self.exec_shell,
            self.canonical_root,
            self.trust_state,
            self.denied,
            self.sandbox_enabled,
            self.sandbox_backend,
            self.sandbox_guest_root,
            if self.sandbox_allow_network {
                "allowed"
            } else {
                "isolated"
            }
        )
    }
}

impl HarnessPromptContext {
    pub fn collect(settings: Option<&PersistedSettings>) -> Self {
        let snapshot = WorkspaceHarnessSnapshot::collect(settings);
        let prompt_block = snapshot.prompt_block();
        Self {
            snapshot,
            prompt_block,
            generated_at_unix_ms: unix_time_ms(),
        }
    }
}

impl SessionHarnessRecord {
    pub fn new(session_id: String, snapshot: WorkspaceHarnessSnapshot) -> Self {
        Self {
            kind: "session_harness".to_string(),
            session_id,
            snapshot,
            recorded_at_unix_ms: unix_time_ms(),
        }
    }
}

impl HarnessPreflightInput {
    pub fn from_tool_call(
        call: &crate::domain::tool_result::ToolCall,
        settings: Option<&PersistedSettings>,
        baseline: Option<&WorkspaceHarnessSnapshot>,
    ) -> Self {
        let command = call
            .args
            .get("command")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        let requested_cwd = call
            .args
            .get("cwd")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        let requested_paths = ["path", "target_path", "old_path", "new_path"]
            .iter()
            .filter_map(|key| {
                call.args
                    .get(*key)
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string)
            })
            .collect();

        Self {
            tool_name: call.name.clone(),
            command,
            requested_cwd,
            requested_paths,
            snapshot: WorkspaceHarnessSnapshot::collect(settings),
            baseline: baseline.cloned(),
        }
    }
}

pub fn canonical_workspace_root() -> String {
    crate::infra::workspace_utils::get_current_workspace_root()
        .canonicalize()
        .unwrap_or_else(|_| crate::infra::workspace_utils::get_current_workspace_root())
        .to_string_lossy()
        .to_string()
}

pub fn evaluate_preflight(input: &HarnessPreflightInput) -> HarnessPreflightDecision {
    if !matches!(
        input.tool_name.as_str(),
        "ExecShell" | "WriteFile" | "ReplaceFileContent" | "DeleteFile"
    ) {
        return HarnessPreflightDecision::Allow;
    }

    if input.snapshot.denied {
        return HarnessPreflightDecision::Deny {
            reason: "workspace is in denied_roots".to_string(),
        };
    }
    if input.snapshot.trust_state != WorkspaceTrustState::Trusted {
        return HarnessPreflightDecision::Deny {
            reason: format!("workspace trust is {:?}", input.snapshot.trust_state),
        };
    }

    if let Some(baseline) = &input.baseline {
        let drift = harness_drift_fields(baseline, &input.snapshot);
        if !drift.is_empty() {
            return HarnessPreflightDecision::Deny {
                reason: format!("workspace harness baseline drift: {}", drift.join(", ")),
            };
        }
    }

    if let Some(cwd) = &input.requested_cwd
        && let Some(reason) = path_outside_workspace(cwd, &input.snapshot.canonical_root)
    {
        return HarnessPreflightDecision::Deny { reason };
    }
    for path in &input.requested_paths {
        if let Some(reason) = path_outside_workspace(path, &input.snapshot.canonical_root) {
            return HarnessPreflightDecision::Deny { reason };
        }
    }

    if input.tool_name == "ExecShell"
        && let Some(command) = &input.command
        && let Some(reason) = detect_command_os_mismatch(&input.snapshot.os, command)
    {
        return HarnessPreflightDecision::Ask { reason };
    }

    HarnessPreflightDecision::Allow
}

pub fn harness_drift_fields(
    baseline: &WorkspaceHarnessSnapshot,
    current: &WorkspaceHarnessSnapshot,
) -> Vec<&'static str> {
    let mut drift = Vec::new();
    if baseline.canonical_root != current.canonical_root {
        drift.push("canonical_root");
    }
    if baseline.trust_state != current.trust_state {
        drift.push("trust_state");
    }
    if baseline.denied != current.denied {
        drift.push("denied");
    }
    if baseline.sandbox_enabled != current.sandbox_enabled {
        drift.push("sandbox_enabled");
    }
    if baseline.sandbox_guest_root != current.sandbox_guest_root {
        drift.push("sandbox_guest_root");
    }
    drift
}

pub fn detect_command_os_mismatch(os: &str, command: &str) -> Option<String> {
    let lower = command.to_lowercase();
    let powershell_only = [
        "get-childitem",
        "set-location",
        "write-host",
        "new-item",
        "remove-item",
        "copy-item",
        "move-item",
        "select-string",
        "test-path",
        "get-content",
    ];
    let posix_only = [
        "sudo ", "apt ", "chmod ", "chown ", "grep ", "sed ", "awk ", "bash ", "sh ", "/bin/",
    ];

    if os == "linux"
        && powershell_only
            .iter()
            .any(|pattern| lower.contains(pattern))
    {
        return Some("PowerShell-specific command proposed in linux workspace".to_string());
    }
    if os == "windows" && posix_only.iter().any(|pattern| lower.contains(pattern)) {
        return Some("POSIX-specific command proposed in windows workspace".to_string());
    }
    None
}

pub fn detect_host_shell() -> String {
    if cfg!(target_os = "windows") {
        std::env::var("ComSpec").unwrap_or_else(|_| "cmd.exe".to_string())
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string())
    }
}

pub fn detect_exec_shell(sandbox_enabled: bool) -> String {
    if cfg!(target_os = "windows") {
        if crate::tools::shell::command_in_path("pwsh.exe").is_some()
            || crate::tools::shell::command_in_path("pwsh").is_some()
        {
            "pwsh".to_string()
        } else if crate::tools::shell::command_in_path("powershell.exe").is_some() {
            "powershell.exe".to_string()
        } else {
            "Not Found".to_string()
        }
    } else if cfg!(target_os = "linux") && sandbox_enabled {
        "sh (bwrap:/workspace)".to_string()
    } else {
        "sh".to_string()
    }
}

fn detect_sandbox_backend() -> String {
    if cfg!(target_os = "linux") {
        if crate::infra::sandbox::detect_backend() {
            "bubblewrap".to_string()
        } else {
            "bubblewrap-missing".to_string()
        }
    } else {
        "none".to_string()
    }
}

fn format_list(values: &[String]) -> String {
    if values.is_empty() {
        "(none)".to_string()
    } else {
        values.join(", ")
    }
}

fn unix_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn path_outside_workspace(path: &str, canonical_root: &str) -> Option<String> {
    let raw = std::path::Path::new(path);
    let candidate = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        std::path::PathBuf::from(canonical_root).join(raw)
    };
    let resolved = candidate.canonicalize().unwrap_or(candidate);
    let root = std::path::PathBuf::from(canonical_root);
    if resolved.starts_with(&root) {
        None
    } else {
        Some(format!(
            "requested path '{}' is outside canonical workspace '{}'",
            path, canonical_root
        ))
    }
}
