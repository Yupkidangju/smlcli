use serde::{Deserialize, Serialize};
use crate::providers::types::ChatMessage;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum AppMode {
    #[default]
    Plan,
    Run,
}

pub struct SessionState {
    pub messages: Vec<ChatMessage>,
    pub mode: AppMode,
    pub token_budget_used: u32,
    pub max_token_budget: u32,
    pub needs_auto_compaction: bool,
}

impl SessionState {
    pub fn new() -> Self {
        let os_type = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        let shell_name = if cfg!(target_os = "windows") { "powershell" } else { "sh" };
        
        let system_prompt = format!(
            "You are smlcli, an intelligent autonomous CLI agent.\n\
            System Info: {} / {}\n\
            Shell Executor: {} (Default)\n\
            \n\
            [AVAILABLE TOOLS]\n\
            1) ExecShell(command, cwd, safe): Executes shell commands directly.\n\
            2) ReadFile(path, start, end): Views specific line range of a file.\n\
            3) WriteFile(path, content, overwrite): Overwrites or creates new file.\n\
            4) ReplaceFileContent(path, target_content, replacement_content): Modifies specific lines.\n\
            5) ListDir(path, depth): Shows directory structure.\n\
            6) GrepSearch(pattern, path, case_insensitive): High-speed file search.\n\
            7) SysInfo(): Fetches live memory/CPU metrics.\n\
            \n\
            [TOOL CALL PROTOCOL]\n\
            If you need to use a tool, you MUST output exactly ONE JSON block in your response like this:\n\
            ```json\n\
            {{\n\
              \"tool\": \"ExecShell\",\n\
              \"command\": \"ls -al\",\n\
              \"cwd\": null,\n\
              \"safe_to_auto_run\": true\n\
            }}\n\
            ```\n\
            * Remember: You have explicit access to the system. Do not write the tool signature as text, use ONLY the JSON format inside ```json ... ```.",
            os_type, arch, shell_name
        );

        Self {
            messages: vec![ChatMessage {
                role: crate::providers::types::Role::System,
                content: system_prompt,
                pinned: true,
            }],
            mode: AppMode::Plan,
            token_budget_used: 0,
            max_token_budget: 128_000,
            needs_auto_compaction: false,
        }
    }
    
    pub fn add_message(&mut self, msg: ChatMessage) {
        self.messages.push(msg);
        
        if self.get_context_load_percentage() > 75 && self.messages.len() > 50 {
            self.needs_auto_compaction = true;
        }
    }
    
    pub fn estimate_current_tokens(&self) -> u32 {
        self.messages.iter().map(|m| (m.content.len() as u32 / 3).max(1)).sum()
    }

    pub fn get_context_load_percentage(&self) -> u32 {
        if self.max_token_budget == 0 { return 0; }
        ((self.estimate_current_tokens() as f64 / self.max_token_budget as f64) * 100.0) as u32
    }
    
    pub fn extract_for_summary(&mut self) -> Vec<ChatMessage> {
        if self.messages.len() <= 5 {
             return vec![];
        }
        
        let mut to_drop = Vec::new();
        let mut new_messages = Vec::new();
        new_messages.push(self.messages[0].clone()); // 시스템 메인 프롬프트 유지
        
        let keep_count = 4;
        let drop_end = self.messages.len() - keep_count;
        
        for i in 1..drop_end {
            if self.messages[i].pinned {
                new_messages.push(self.messages[i].clone());
            } else {
                to_drop.push(self.messages[i].clone());
            }
        }
        
        if to_drop.is_empty() {
            return vec![];
        }
        
        new_messages.push(ChatMessage {
            role: crate::providers::types::Role::System,
            content: "[Summary Pending...]".to_string(),
            pinned: true,
        });
        
        for i in drop_end..self.messages.len() {
            new_messages.push(self.messages[i].clone());
        }
        
        self.messages = new_messages;
        to_drop
    }
    
    pub fn apply_summary(&mut self, summary: &str) {
        for msg in &mut self.messages {
            if msg.content == "[Summary Pending...]" {
                msg.content = format!("[Context Compaction Summary]\n{}", summary);
                msg.pinned = true;
                break;
            }
        }
    }

    pub fn get_budget_percentage(&self) -> u32 {
        if self.max_token_budget == 0 { return 0; }
        ((self.token_budget_used as f64 / self.max_token_budget as f64) * 100.0) as u32
    }
}
