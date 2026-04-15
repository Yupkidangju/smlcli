#[derive(Debug, Clone)]
pub enum Action {
    ToolFinished(crate::domain::tool_result::ToolResult),
    ToolError(String),
    ChatResponseOk(crate::providers::types::ChatResponse),
    ChatResponseErr(String),
    ModelsFetched(Result<Vec<String>, String>),
    /// [v0.1.0-beta.7] API 키 검증 결과 이벤트
    CredentialValidated(Result<(), String>),
    ContextSummaryOk(String),
    ContextSummaryErr(String),
}
