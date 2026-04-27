// [v3.7.0] Phase 47 Task Q-1: Interactive Planning Questionnaire 도메인 타입.
// PLAN 모드에서 AI가 모호성을 발견할 경우, 구조화된 폼(Form) 도구를 호출하여
// 사용자에게 객관식/주관식 질문을 제시하고 답변을 수집하는 구조체들.
// spec.md §47.2 Typed Contracts에 기반.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 개별 명확화 질문.
/// `options`가 빈 배열이면 주관식(자유 입력) 텍스트로 간주됨.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationQuestion {
    /// 질문 고유 식별자 (LLM이 생성한 ID, 예: "q1", "lang_choice")
    pub id: String,
    /// 질문 본문 (사용자에게 표시됨)
    pub title: String,
    /// 선택지 목록. 빈 배열 = 주관식(자유 입력)
    pub options: Vec<String>,
    /// true일 경우 정해진 옵션 외에 "직접 입력" 선택지를 추가로 허용
    pub allow_custom: bool,
}

/// AskClarification 도구의 인자 구조체.
/// LLM이 도구 호출 시 이 형태로 JSON을 전달함.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskClarificationArgs {
    /// 명확화가 필요한 질문 목록 (1개 이상)
    pub questions: Vec<ClarificationQuestion>,
}

/// 사용자 답변 결과.
/// 모든 질문에 대한 답변이 수집된 후 LLM에게 ToolResult로 전달됨.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskClarificationResult {
    /// key: question id, value: 사용자가 선택하거나 입력한 답변
    pub answers: HashMap<String, String>,
}

/// TUI에서 Questionnaire 폼의 상태를 관리하는 구조체.
/// 사용자가 질문에 순차적으로 답변하는 과정을 추적함.
#[derive(Debug, Clone)]
pub struct QuestionnaireState {
    /// 질문 목록
    pub questions: Vec<ClarificationQuestion>,
    /// 현재 표시 중인 질문 인덱스 (0-based)
    pub current_index: usize,
    /// 객관식 질문에서 현재 커서가 가리키는 옵션 인덱스
    pub option_cursor: usize,
    /// 수집된 답변 (question id → answer)
    pub answers: HashMap<String, String>,
    /// 주관식 입력 시 사용하는 텍스트 버퍼
    pub custom_input: String,
    /// "직접 입력" 모드인지 여부 (allow_custom + Enter)
    pub is_custom_input_mode: bool,
    /// 도구 호출 ID (ToolResult 반환 시 사용)
    pub tool_call_id: Option<String>,
    /// 도구 인덱스 (병렬 도구 추적용)
    pub tool_index: usize,
}

impl QuestionnaireState {
    /// 새 Questionnaire 상태 생성.
    pub fn new(
        questions: Vec<ClarificationQuestion>,
        tool_call_id: Option<String>,
        tool_index: usize,
    ) -> Self {
        Self {
            questions,
            current_index: 0,
            option_cursor: 0,
            answers: HashMap::new(),
            custom_input: String::new(),
            is_custom_input_mode: false,
            tool_call_id,
            tool_index,
        }
    }

    /// 현재 질문 참조.
    pub fn current_question(&self) -> Option<&ClarificationQuestion> {
        self.questions.get(self.current_index)
    }

    /// 현재 질문이 주관식(자유 입력)인지 여부.
    pub fn is_current_freeform(&self) -> bool {
        self.current_question()
            .map(|q| q.options.is_empty())
            .unwrap_or(false)
    }

    /// 현재 질문에서 표시할 전체 옵션 수 (allow_custom이면 +1).
    pub fn total_options(&self) -> usize {
        self.current_question()
            .map(|q| {
                let base = q.options.len();
                if q.allow_custom { base + 1 } else { base }
            })
            .unwrap_or(0)
    }

    /// 현재 질문에 대한 답변을 기록하고 다음 질문으로 이동.
    /// 모든 질문에 답변 완료 시 true 반환.
    pub fn submit_answer(&mut self, answer: String) -> bool {
        if let Some(q) = self.current_question() {
            self.answers.insert(q.id.clone(), answer);
        }
        self.current_index += 1;
        self.option_cursor = 0;
        self.custom_input.clear();
        self.is_custom_input_mode = false;
        self.current_index >= self.questions.len()
    }

    /// 답변을 AskClarificationResult 형태로 조립.
    pub fn build_result(&self) -> AskClarificationResult {
        AskClarificationResult {
            answers: self.answers.clone(),
        }
    }
}
