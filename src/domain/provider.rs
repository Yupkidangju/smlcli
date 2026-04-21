use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ProviderKind {
    OpenRouter,
    Google,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProviderProfile {
    pub id: String,
    pub kind: ProviderKind,
    pub api_key_alias: String,
    pub default_model: String,
}

/// [Phase 16] Provider 호환성을 맞추기 위한 방언(Dialect) 설정
#[derive(Debug, Clone, PartialEq)]
pub enum ToolDialect {
    OpenAICompat, // 기본 JSON Schema
    Anthropic,    // strict XML/JSON 혼합 구조 (미래 대비)
    Gemini,       // function parameter required fields 제약 엄격
}
