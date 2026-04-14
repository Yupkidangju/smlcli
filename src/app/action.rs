#[derive(Debug, Clone)]
pub enum Action {
    ToolFinished(crate::domain::tool_result::ToolResult),
    ToolError(String),
    ChatResponseOk(crate::providers::types::ChatResponse),
    ChatResponseErr(String),
    ModelsFetched(Result<Vec<String>, String>),
    ContextSummaryOk(String),
    ContextSummaryErr(String),
}
