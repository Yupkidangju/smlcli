use crate::providers::types::ChatMessage;
use serde::{Deserialize, Serialize};

// [v3.6.0] Phase 46 Task S-1: 세션 메타데이터 구조체.
// 워크스페이스 기반 세션 격리를 위한 인덱스 모델.
// sessions_index.json에 직렬화되어 세션 목록 조회 시 전체 파일 파싱 없이 사용됨.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionMetadata {
    /// 고유 세션 식별자 (UUID v4)
    pub session_id: String,
    /// 세션이 생성된 워크스페이스의 절대 경로
    pub workspace_root: String,
    /// 세션 제목 (첫 프롬프트 요약 또는 Auto-Titling LLM이 생성한 제목)
    pub title: String,
    /// 세션 생성 시각 (Unix 밀리초)
    pub created_at_unix_ms: u64,
    /// 세션 마지막 갱신 시각 (Unix 밀리초)
    pub updated_at_unix_ms: u64,
    /// 세션 로그 파일명 (session_{timestamp}.jsonl)
    pub log_filename: String,
}

// [v3.6.0] Phase 46 Task S-3: 세션 관련 사용자 액션.
// /resume, /new, /session 명령어에서 사용되는 세션 동작 열거형.
// [v3.7.0] SessionAction은 /resume, /new, /session 명령어 핸들러 연결 시 활성화 예정.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SessionAction {
    /// 새 세션 시작: 현재 컨텍스트 초기화 후 새 세션 할당
    NewSession,
    /// 기존 세션 이어하기: session_id로 세션 복원
    ResumeSession(String),
    /// 세션 목록 조회: 현재 워크스페이스의 세션 목록 표시
    ListSessions,
}

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

        // [v0.1.0-beta.22] 시스템 프롬프트 개편:
        // - 첫 턴 자연어 응답 가드 추가 (도구 호출 억제)
        // - 도구 카탈로그를 간결화하고 예시 JSON 제거 (스키마 노출 방지)
        // - 비작업성 입력(인삿말, 질문)에는 도구를 사용하지 않도록 명시
        // - 작업 요청이면 첫 프롬프트라도 즉시 도구 사용 (Run 모드 계약과 일관)
        let system_prompt = format!(
            "You are **smlcli**, a professional CLI agent in the user's terminal.\n\
            \n\
            ## Core Rules\n\
            - If the user's message is a greeting, question, or general conversation: \
            respond in natural language ONLY. No tools.\n\
            - If the user's message is an explicit work request \
            (read a file, run a command, write/modify code, etc.): \
            use the appropriate tool immediately, even on the very first message.\n\
            - When you use a tool, always explain what you are about to do BEFORE the tool call.\n\
            - After a tool produces output, summarize the result in natural language.\n\
            - The user sees your full response. Never output raw JSON without context.\n\
            \n\
            ## Environment\n\
            - OS: {} / {}\n\
            - Shell: {}\n\
            \n\
            ## Communication Style\n\
            - Be direct, professional, and concise.\n\
            - Respond in the same language the user writes in.\n\
            - Format output using markdown when helpful.\n\
            - When uncertain, ask clarifying questions.\n\
            \n\
            ## Tools\n\
            You have access to: ExecShell, ReadFile, WriteFile, ReplaceFileContent, \
            ListDir, GrepSearch, Stat, SysInfo.\n\
            When you need a tool, output EXACTLY ONE fenced ```json block containing \
            the tool call object with a \"tool\" field.\n\
            Only call ONE tool per response. Wait for the result before calling another.",
            os_type, arch, shell_name
        );

        Self {
            messages: vec![ChatMessage {
                role: crate::providers::types::Role::System,
                content: Some(system_prompt),
                tool_calls: None,
                tool_call_id: None,
                pinned: true,
            }],
            // [v0.1.0-beta.22] 기본 모드를 Run으로 변경.
            // 코딩 에이전트로서 파일 읽기/쓰기가 기본 동작.
            // Plan 모드는 분석/설명 전용으로 Tab 키로 전환.
            mode: AppMode::Run,
            token_budget_used: 0,
            max_token_budget: 128_000,
            needs_auto_compaction: false,
        }
    }

    pub fn add_message(&mut self, msg: ChatMessage) {
        self.messages.push(msg);

        // [v1.3.0] Sliding Window: 컨텍스트 토큰 부하 초과 시 자동 압축(요약) 요청
        if self.get_context_load_percentage() > 75 && self.messages.len() > 20 {
            self.needs_auto_compaction = true;
        }

        // [v1.3.0] 강제 Sliding Window: 메시지 수가 극단적으로 많아지면 오래된 메시지를 즉시 드랍하여 RAM 점유율 폭주 방지
        let max_messages = 200;
        if self.messages.len() > max_messages {
            let mut keep = Vec::new();
            let drop_count = self.messages.len() - (max_messages - 50); // 여유분을 위해 50개 더 삭제
            let mut dropped = 0;

            for msg in &self.messages {
                if msg.pinned || dropped >= drop_count {
                    keep.push(msg.clone());
                } else {
                    dropped += 1;
                }
            }
            self.messages = keep;
        }
    }

    pub fn estimate_current_tokens(&self) -> u32 {
        self.messages
            .iter()
            .map(|m| {
                if let Some(s) = &m.content {
                    // [v1.4.0] 토큰 계산 휴리스틱 고도화: 영문 4글자당 1토큰, 비-ASCII 1글자당 1.5토큰
                    let mut tokens: f32 = 0.0;
                    for c in s.chars() {
                        if c.is_ascii() {
                            tokens += 0.25;
                        } else {
                            tokens += 1.5;
                        }
                    }
                    (tokens.ceil() as u32).max(1)
                } else {
                    1
                }
            })
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
        // [v1.6.0] Protected Range: 첫 3개 메시지 보존
        let protected_count = self.messages.len().min(3);
        for i in 0..protected_count {
            new_messages.push(self.messages[i].clone());
        }

        let keep_count = 4;
        let drop_end = self.messages.len().saturating_sub(keep_count);

        for i in protected_count..drop_end {
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
            content: Some("[Summary Pending...]".to_string()),
            tool_calls: None,
            tool_call_id: None,
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
            if msg.content.as_deref() == Some("[Summary Pending...]") {
                msg.content = Some(format!("[Context Compaction Summary]\n{}", summary));
                msg.pinned = true;
                break;
            }
        }
    }

    #[allow(dead_code)] // [v3.7.0] 토큰 예산 UI 표시 시 활성화 예정
    pub fn get_budget_percentage(&self) -> u32 {
        if self.max_token_budget == 0 {
            return 0;
        }
        ((self.token_budget_used as f64 / self.max_token_budget as f64) * 100.0) as u32
    }
}
