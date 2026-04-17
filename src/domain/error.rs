use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("설정 오류: {0}")]
    Config(#[from] ConfigError),

    #[error("도구 실행 오류: {0}")]
    Tool(#[from] ToolError),

    #[error("공급자 오류: {0}")]
    Provider(#[from] ProviderError),

    #[error("입출력 오류: {0}")]
    Io(String),

    #[error("알 수 없는 오류: {0}")]
    Unknown(String),
}

#[derive(Error, Debug, Clone)]
pub enum ConfigError {
    #[error("설정 파일을 찾을 수 없습니다")]
    NotFound,
    #[error("설정 파일 파싱 실패: {0}")]
    ParseFailure(String),
    #[error("마스터 키 유효성 검사 실패")]
    InvalidMasterKey,
    #[error("비밀번호 복호화 실패")]
    DecryptionFailure,
}

#[derive(Error, Debug, Clone)]
pub enum ToolError {
    #[error("허용되지 않은 명령입니다: {0}")]
    PermissionDenied(String),
    #[error("도구 실행 시간 초과")]
    Timeout,
    #[error("도구 실행 실패: {0}")]
    ExecutionFailure(String),
    #[error("잘못된 도구 인자: {0}")]
    InvalidArguments(String),
}

#[derive(Error, Debug, Clone)]
pub enum ProviderError {
    #[error("인증 실패: {0}")]
    AuthenticationFailed(String),
    #[error("네트워크 연결 실패: {0}")]
    NetworkFailure(String),
    #[error("API 응답 오류 ({code}): {message}")]
    ApiResponse { code: u16, message: String },
    #[error("지원되지 않는 모델입니다: {0}")]
    UnsupportedModel(String),
}
