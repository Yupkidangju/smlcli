// [v0.1.0-beta.10] 6차 감사: FetchSource 추가로 비동기 결과 라우팅의 출처 의존성 해소.
// 이전에는 ModelsFetched 이벤트가 "현재 UI 상태"(config.is_open)로 라우팅되어,
// 사용자가 팝업을 닫으면 결과가 엉뚱한 상태에 반영되는 결함이 있었음.

/// 비동기 모델 조회 요청의 출처를 식별하는 열거형.
/// ModelsFetched 이벤트에 동봉되어 정확한 상태 슬롯으로 라우팅함.
#[derive(Debug, Clone, PartialEq)]
pub enum FetchSource {
    /// Config 팝업(/config, /model, /provider)에서 발생한 요청
    Config,
    /// Setup Wizard에서 발생한 요청
    Wizard,
}

#[derive(Debug, Clone)]
pub enum Action {
    ToolFinished(crate::domain::tool_result::ToolResult),
    ToolError(String),
    ChatResponseOk(crate::providers::types::ChatResponse),
    ChatResponseErr(String),
    // [v0.1.0-beta.10] FetchSource 추가: 비동기 결과가 정확한 상태 슬롯으로 라우팅됨
    ModelsFetched(Result<Vec<String>, String>, FetchSource),
    /// [v0.1.0-beta.7] API 키 검증 결과 이벤트
    CredentialValidated(Result<(), String>),
    ContextSummaryOk(String),
    ContextSummaryErr(String),
}
