use thiserror::Error;

// [v2.1.0] Phase 29: Actionable Error 확장을 위한 구조
#[derive(Debug, Clone)]
pub struct ActionableError {
    pub message: String,
    pub suggestion: Option<String>,
}

impl std::fmt::Display for ActionableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(sug) = &self.suggestion {
            write!(f, "{} (제안: {})", self.message, sug)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

// [v3.7.0] Unknown variant는 예상치 못한 에러 분류용 대비 variant.
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum SmlError {
    #[error("설정 오류: {0}")]
    Config(#[from] ConfigError),

    #[error("도구 실행 오류: {0}")]
    Tool(#[from] ToolError),

    #[error("공급자 오류: {0}")]
    Provider(#[from] ProviderError),

    #[error("입출력 오류: {0}")]
    IoError(#[from] std::io::Error),

    #[error("인프라 오류: {0}")]
    InfraError(String),

    #[error("알 수 없는 오류: {0}")]
    Unknown(String),
}

impl SmlError {
    pub fn to_actionable(&self) -> ActionableError {
        let msg = self.to_string();
        let suggestion = match self {
            SmlError::Config(ConfigError::InvalidMasterKey) => Some("설정 메뉴(/setting)에서 API 키를 다시 확인하거나 볼트 비밀번호를 점검하세요.".to_string()),
            SmlError::Provider(ProviderError::AuthenticationFailed(_)) => Some("제공자의 API 키가 유효한지 또는 만료되지 않았는지 확인하세요.".to_string()),
            SmlError::Provider(ProviderError::NetworkFailure(_)) => Some("인터넷 연결 상태를 확인하고 프록시나 방화벽 설정을 점검하세요.".to_string()),
            SmlError::Config(ConfigError::NotFound) => Some("smlcli 초기 설정을 진행하거나 ~/.smlcli 디렉토리 권한을 확인하세요.".to_string()),
            SmlError::Config(ConfigError::ParseFailure(_)) => Some("설정 파일 형식이 잘못되었습니다. URL 형식이나 오타를 확인하세요.".to_string()),
            SmlError::Tool(ToolError::ExecutionFailure(_)) => Some("해당 명령어가 시스템에 설치되어 있는지, 그리고 실행 권한이 충분한지 확인하세요.".to_string()),
            SmlError::Tool(ToolError::Timeout) => Some("도구 실행이 너무 오래 걸립니다. 명령을 최적화하거나 불필요한 백그라운드 작업을 줄이세요.".to_string()),
            _ => None,
        };
        ActionableError {
            message: msg,
            suggestion,
        }
    }
}

// [v3.7.0] InvalidMasterKey, DecryptionFailure는 Vault 복호화 로직 연동 시 활성화 예정.
#[derive(Error, Debug, Clone)]
#[allow(dead_code)]
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

// [v3.7.0] PermissionDenied, Timeout은 PermissionEngine 세분화 시 활성화 예정.
#[derive(Error, Debug, Clone)]
#[allow(dead_code)]
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

impl ToolError {
    pub fn to_actionable(&self) -> ActionableError {
        SmlError::Tool(self.clone()).to_actionable()
    }
}

// [v3.7.0] UnsupportedModel은 Provider 모델 검증 강화 시 활성화 예정.
#[derive(Error, Debug, Clone)]
#[allow(dead_code)]
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

impl ProviderError {
    pub fn to_actionable(&self) -> ActionableError {
        SmlError::Provider(self.clone()).to_actionable()
    }
}
