// [v3.7.0] Phase 47 Task Q-1: AskClarification 도구 구현.
// PLAN 모드에서 AI가 모호한 요구사항을 발견할 경우 호출하는 구조화된 질문 도구.
// 일반 도구와 달리 execute()에서 직접 실행하지 않고,
// ShowQuestionnaire Action을 통해 TUI 모달로 전환한 뒤
// 사용자 답변을 수집하여 ToolResult로 반환하는 이벤트 기반 패턴.

use async_trait::async_trait;
use serde_json::Value;

use crate::domain::error::ToolError;
use crate::domain::permissions::PermissionResult;
use crate::domain::settings::PersistedSettings;
use crate::domain::tool_result::ToolResult;
use crate::tools::registry::{Tool, ToolContext};

/// AskClarification 도구.
/// LLM이 PLAN 모드에서 모호한 요구사항을 발견할 때 호출함.
/// 사용자에게 객관식/주관식 질문을 제시하고 답변을 수집.
pub struct AskClarificationTool;

#[async_trait]
impl Tool for AskClarificationTool {
    fn name(&self) -> &'static str {
        "AskClarification"
    }

    fn description(&self) -> &'static str {
        "사용자에게 명확화 질문을 제시합니다. 모호한 요구사항이 있을 때 선택지와 함께 질문하세요."
    }

    fn schema(&self) -> Value {
        // OpenAI Function Calling 형식의 JSON 스키마.
        // questions 배열에 각 질문의 id, title, options, allow_custom을 포함.
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "AskClarification",
                "description": "사용자에게 명확화 질문을 제시하여 모호한 요구사항을 해소합니다. 각 질문에 선택지를 제공하거나, 자유 입력을 허용합니다.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "questions": {
                            "type": "array",
                            "description": "명확화가 필요한 질문 목록",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "id": {
                                        "type": "string",
                                        "description": "질문 고유 ID (예: 'q1', 'framework_choice')"
                                    },
                                    "title": {
                                        "type": "string",
                                        "description": "사용자에게 표시될 질문 본문"
                                    },
                                    "options": {
                                        "type": "array",
                                        "items": { "type": "string" },
                                        "description": "선택지 목록. 빈 배열이면 자유 입력(주관식)"
                                    },
                                    "allow_custom": {
                                        "type": "boolean",
                                        "description": "true면 선택지 외에 직접 입력 허용"
                                    }
                                },
                                "required": ["id", "title", "options", "allow_custom"]
                            }
                        }
                    },
                    "required": ["questions"]
                }
            }
        })
    }

    fn check_permission(&self, _args: &Value, _settings: &PersistedSettings) -> PermissionResult {
        // AskClarification은 읽기 전용 도구이므로 항상 허용.
        // 파일 시스템이나 네트워크 접근 없이 TUI 모달만 렌더링함.
        PermissionResult::Allow
    }

    fn format_detail(&self, args: &Value) -> String {
        // 승인 대기 시 간략한 질문 수 표시
        let count = args
            .get("questions")
            .and_then(|q| q.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        format!("AskClarification — {} 개의 질문", count)
    }

    /// AskClarification 도구는 직접 실행하지 않음.
    /// tool_runtime에서 이 도구명을 감지하면 ShowQuestionnaire Action으로 분기하고,
    /// 사용자 답변 수집 후 ToolFinished Action으로 결과를 전송.
    /// 이 execute()는 호출되지 않아야 하나, 안전장치로 에러를 반환.
    async fn execute(&self, args: Value, _ctx: &ToolContext<'_>) -> Result<ToolResult, ToolError> {
        // 이 경로는 정상적으로는 도달하지 않아야 함.
        // tool_runtime에서 AskClarification을 감지하여 별도 처리하기 때문.
        // 만약 도달하면 인자를 파싱하여 기본 텍스트 응답으로 변환.
        let parsed: Result<crate::domain::questionnaire::AskClarificationArgs, _> =
            serde_json::from_value(args);
        match parsed {
            Ok(clarification) => {
                // 폴백: 질문 목록을 텍스트로 변환하여 반환
                let mut text = String::from("다음 질문에 대해 답변해주세요:\n\n");
                for (i, q) in clarification.questions.iter().enumerate() {
                    text.push_str(&format!("{}. {}\n", i + 1, q.title));
                    if q.options.is_empty() {
                        text.push_str("   (자유 입력)\n");
                    } else {
                        for (j, opt) in q.options.iter().enumerate() {
                            text.push_str(&format!("   {}. {}\n", j + 1, opt));
                        }
                    }
                    text.push('\n');
                }
                Ok(ToolResult {
                    tool_name: "AskClarification".to_string(),
                    tool_call_id: None,
                    stdout: text,
                    stderr: String::new(),
                    exit_code: 0,
                    is_error: false,
                    is_truncated: false,
                    original_size_bytes: None,
                    affected_paths: Vec::new(),
                })
            }
            Err(e) => Err(ToolError::InvalidArguments(format!(
                "AskClarification 인자 파싱 실패: {}",
                e
            ))),
        }
    }
}
