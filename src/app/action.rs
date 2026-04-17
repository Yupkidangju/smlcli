// [v0.1.0-beta.18] Phase 9-A: 이벤트 시스템 14종 확장.
// 이전: 7종 (ToolFinished, ToolError, ChatResponseOk, ChatResponseErr,
//              ModelsFetched, CredentialValidated, ContextSummaryOk/Err)
// 현재: 14종 — 채팅/도구 라이프사이클(시작·진행·완료·에러)을 세분화하여
//       Codex 스타일 진행 표시(스피너, 스트리밍, 작업 카드)를 구현 가능하게 함.
// 관련 문서: spec.md §3.9, DESIGN_DECISIONS.md ADR-009
// [v0.1.0-beta.21] ChatResponseErr/ToolError/ModelsFetched/CredentialValidated에
//   String 대신 도메인 에러 타입(ProviderError/ToolError)을 사용하도록 전환.
//   이를 통해 UI에서 에러 종류별 분기 처리와 내부 진단 정보 분리가 가능해짐.

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
    ChatResponseOk(Box<crate::providers::types::ChatResponse>),

    /// [v0.1.0-beta.21] Provider 호출 실패 — ProviderError 타입으로 구조화.
    /// UI 표시용 메시지는 Display 트레이트, 내부 진단은 Debug/패턴매칭으로 분리.
    ChatResponseErr(crate::domain::error::ProviderError),

    // === 도구 라이프사이클 ===
    /// [v0.1.0-beta.18] JSON 파싱 완료 → 권한 검사 전. 타임라인에 "대기중" 카드 추가.
    ToolQueued(Box<crate::domain::tool_result::ToolCall>, Option<String>),

    /// [v0.1.0-beta.18] 실행 시작. 타임라인의 해당 카드 상태를 "실행중"으로 갱신.
    ToolStarted(String),

    /// [v0.1.0-beta.18] shell stdout 스트리밍 청크. Logs 탭에 append.
    ToolOutputChunk(String),

    /// 도구 실행 완료. 결과를 세션 메시지에 추가.
    ToolFinished(Box<crate::domain::tool_result::ToolResult>),

    /// [v0.1.0-beta.18] 2~4줄 요약 생성 완료. 타임라인 카드의 summary 갱신.
    ToolSummaryReady(String),

    /// [v0.1.0-beta.21] 도구 실행 실패 — ToolError 타입으로 구조화.
    ToolError(crate::domain::error::ToolError),

    // === 기존 유지 ===
    /// [v0.1.0-beta.21] 비동기 모델 목록 조회 결과 — 실패 시 ProviderError 사용.
    ModelsFetched(
        Result<Vec<String>, crate::domain::error::ProviderError>,
        FetchSource,
    ),

    /// [v0.1.0-beta.21] API 키 검증 결과 — 실패 시 ProviderError 사용.
    CredentialValidated(Result<(), crate::domain::error::ProviderError>),

    /// 컨텍스트 요약 성공
    ContextSummaryOk(String),

    /// 컨텍스트 요약 실패
    ContextSummaryErr(String),
}
