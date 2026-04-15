use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    // [v0.1.0-beta.7] H-4: pinned는 내부용 필드이므로 Provider API 페이로드에 포함되면 안 됨.
    // 엄격한 OpenAI 호환 서버에서 unknown field 에러를 야기할 수 있으므로 직렬화 제외.
    #[serde(default, skip_serializing)]
    pub pinned: bool,
}

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub message: ChatMessage,
    pub input_tokens: u32,
    pub output_tokens: u32,
}
