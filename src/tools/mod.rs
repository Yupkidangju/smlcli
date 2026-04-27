pub mod executor;
pub mod fetch;
pub mod file_ops;
pub mod git_checkpoint;
pub mod grep;
// [v3.4.0] Phase 44 Task D-2: TECH-DEBT 정리 완료. 모든 도구가 활성화됨.
// [v3.7.0] Phase 47: AskClarification 도구 추가.
pub mod questionnaire;
pub mod registry;
pub mod shell;
pub mod sys_ops;

// executor.rs 에서 도구명, 파라미터 매핑을 파싱하여 각 모듈로 라우팅합니다.
