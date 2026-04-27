use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ProviderKind {
    OpenAI,
    Anthropic,
    #[serde(rename = "xAI")]
    Xai,
    OpenRouter,
    Google,
    // [v3.1.0] Phase 41: 커스텀 Provider 지원 (Ollama, LMStudio, VLLM 등 API 호환 백엔드)
    Custom(String),
}

/// [v3.1.0] Phase 41: 커스텀 Provider 구성 설정
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CustomProviderConfig {
    pub id: String,                       // 고유 식별자 (예: "local-ollama")
    pub base_url: String,                 // API 엔드포인트 기본 URL
    pub auth_type: String,                // "Bearer", "None", "CustomHeader" 등
    pub auth_header_name: Option<String>, // 커스텀 헤더 사용시 헤더 이름
    pub dialect: ToolDialect,             // API 호환 타입 (대부분 OpenAICompat)
}

// [v3.7.0] ProviderProfile은 멀티 프로바이더 전환 UI에서 사용 예정.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct ProviderProfile {
    pub id: String,
    pub kind: ProviderKind,
    pub api_key_alias: String,
    pub default_model: String,
}

/// [Phase 16] Provider 호환성을 맞추기 위한 방언(Dialect) 설정
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ToolDialect {
    OpenAICompat, // 기본 JSON Schema
    Anthropic,    // strict XML/JSON 혼합 구조 (미래 대비)
    Gemini,       // function parameter required fields 제약 엄격
}
