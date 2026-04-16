// [v0.1.0-beta.18] Phase 9-A: 이벤트 시스템 14종 확장.
// 이전: 7종 (ToolFinished, ToolError, ChatResponseOk, ChatResponseErr,
//              ModelsFetched, CredentialValidated, ContextSummaryOk/Err)
// 현재: 14종 — 채팅/도구 라이프사이클(시작·진행·완료·에러)을 세분화하여
//       Codex 스타일 진행 표시(스피너, 스트리밍, 작업 카드)를 구현 가능하게 함.
// 관련 문서: spec.md §3.9, DESIGN_DECISIONS.md ADR-009

/// [v0.1.0-beta.10] 비동기 모델 조회 요청의 출처를 식별하는 열거형.
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
    // === 채팅 라이프사이클 ===

    /// [v0.1.0-beta.18] LLM 요청 발송 시점. thinking indicator 시작 트리거.
    ChatStarted,

    /// [v0.1.0-beta.18] SSE 스트리밍 토큰 단위 수신. 타임라인에 실시간 append.
    ChatDelta(String),

    /// 전체 응답 완료. 토큰 예산 갱신 + 도구 호출 감지.
    ChatResponseOk(crate::providers::types::ChatResponse),

    /// Provider 호출 실패.
    ChatResponseErr(String),

    // === 도구 라이프사이클 ===

    /// [v0.1.0-beta.18] JSON 파싱 완료 → 권한 검사 전. 타임라인에 "대기중" 카드 추가.
    ToolQueued(crate::domain::tool_result::ToolCall),

    /// [v0.1.0-beta.18] 실행 시작. 타임라인의 해당 카드 상태를 "실행중"으로 갱신.
    ToolStarted(String),

    /// [v0.1.0-beta.18] shell stdout 스트리밍 청크. Logs 탭에 append.
    ToolOutputChunk(String),

    /// 도구 실행 완료. 결과를 세션 메시지에 추가.
    ToolFinished(crate::domain::tool_result::ToolResult),

    /// [v0.1.0-beta.18] 2~4줄 요약 생성 완료. 타임라인 카드의 summary 갱신.
    ToolSummaryReady(String),

    /// 도구 실행 실패.
    ToolError(String),

    // === 기존 유지 ===

    /// [v0.1.0-beta.10] FetchSource 추가: 비동기 결과가 정확한 상태 슬롯으로 라우팅됨
    ModelsFetched(Result<Vec<String>, String>, FetchSource),

    /// [v0.1.0-beta.7] API 키 검증 결과 이벤트
    CredentialValidated(Result<(), String>),

    /// 컨텍스트 요약 성공
    ContextSummaryOk(String),

    /// 컨텍스트 요약 실패
    ContextSummaryErr(String),
}
