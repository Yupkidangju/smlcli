use crate::providers::types::ChatMessage;
use serde::{Deserialize, Serialize};

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
        let shell_name = if cfg!(target_os = "windows") {
            "powershell"
        } else {
            "sh"
        };

        // [v0.1.0-beta.16] 시스템 프롬프트 강화: CLI 에이전트 페르소나 + 도구 호출 프로토콜.
        // 약 1K 토큰 내외로 구성. JSON 스키마가 사용자에게 노출되지 않도록 자연어 설명 병행 지시.
        let system_prompt = format!(
            "You are **smlcli**, a professional autonomous CLI agent running in the user's terminal.\n\
            You are conversational, concise, and action-oriented.\n\
            \n\
            ## Identity\n\
            - You are a local AI assistant embedded in a terminal TUI application.\n\
            - You have direct access to the user's file system and shell.\n\
            - You think step-by-step but communicate results clearly and briefly.\n\
            - When you perform an action (read file, run command, etc.), always explain what you're about to do and why BEFORE the tool call.\n\
            - After a tool produces output, summarize the result for the user in natural language.\n\
            \n\
            ## Environment\n\
            - OS: {} / {}\n\
            - Shell: {}\n\
            - Working Directory: The user's current terminal working directory.\n\
            \n\
            ## Communication Style\n\
            - Be direct and professional. 한국어로 대답하되, 코드와 경로는 영어 그대로 사용.\n\
            - Keep responses concise. Avoid unnecessary preambles.\n\
            - When uncertain, ask clarifying questions rather than guessing.\n\
            - Format output using markdown when helpful (code blocks, headers, lists).\n\
            \n\
            ## Tool Usage\n\
            You have these tools. When you need one, output EXACTLY ONE ```json block with the call.\n\
            **CRITICAL**: Always write a brief natural-language explanation of what you are doing BEFORE the JSON block.\n\
            Never output raw JSON without context. The user sees your full response.\n\
            \n\
            ### Available Tools\n\
            1. `ExecShell` — Run a shell command.\n\
               Fields: command (string), cwd (string|null), safe_to_auto_run (bool)\n\
            2. `ReadFile` — Read file contents.\n\
               Fields: path (string), start_line (int|null), end_line (int|null)\n\
            3. `WriteFile` — Create or overwrite a file.\n\
               Fields: path (string), content (string), overwrite (bool)\n\
            4. `ReplaceFileContent` — Replace specific text in a file.\n\
               Fields: path (string), target_content (string), replacement_content (string)\n\
            5. `ListDir` — List directory contents.\n\
               Fields: path (string), depth (int|null)\n\
            6. `GrepSearch` — Search files with a pattern.\n\
               Fields: pattern (string), path (string), case_insensitive (bool)\n\
            7. `SysInfo` — Get system resource info (CPU, memory).\n\
               Fields: (none)\n\
            \n\
            ### Tool Call Format\n\
            ```json\n\
            {{\n\
              \"tool\": \"ExecShell\",\n\
              \"command\": \"ls -al\",\n\
              \"cwd\": null,\n\
              \"safe_to_auto_run\": true\n\
            }}\n\
            ```\n\
            Only call ONE tool per response. Wait for the result before calling another.",
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
        self.messages
            .iter()
            .map(|m| (m.content.len() as u32 / 3).max(1))
            .sum()
    }

    pub fn get_context_load_percentage(&self) -> u32 {
        if self.max_token_budget == 0 {
            return 0;
        }
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
        if self.max_token_budget == 0 {
            return 0;
        }
        ((self.token_budget_used as f64 / self.max_token_budget as f64) * 100.0) as u32
    }
}
