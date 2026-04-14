use anyhow::Result;
use crate::domain::provider::ProviderKind;
use reqwest::Client;
use std::pin::Pin;
use std::future::Future;

pub trait ProviderAdapter: Send + Sync {
    /// 해당 Provider에 대한 API 인증 정보를 최소한으로 검증하는 smoke test 함수
    fn validate_credentials<'a>(&'a self, api_key: &'a str) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
    
    /// Provider에 맞추어 채팅 요청을 전송하고 응답을 반환
    fn chat<'a>(&'a self, api_key: &'a str, req: crate::providers::types::ChatRequest) -> Pin<Box<dyn Future<Output = Result<crate::providers::types::ChatResponse>> + Send + 'a>>;

    /// 지원하는 모델 목록을 동적으로 가져옴
    fn fetch_models<'a>(&'a self, api_key: &'a str) -> Pin<Box<dyn Future<Output = Result<Vec<String>>> + Send + 'a>>;
}

pub struct OpenRouterAdapter {
    client: Client,
}

impl OpenRouterAdapter {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl ProviderAdapter for OpenRouterAdapter {
    fn validate_credentials<'a>(&'a self, api_key: &'a str) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let response = self.client
                .get("https://openrouter.ai/api/v1/auth/key")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await?;
            
            if response.status().is_success() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Invalid OpenRouter API Key"))
            }
        })
    }
    
    fn chat<'a>(&'a self, api_key: &'a str, req: crate::providers::types::ChatRequest) -> Pin<Box<dyn Future<Output = Result<crate::providers::types::ChatResponse>> + Send + 'a>> {
        Box::pin(async move {
            #[derive(serde::Serialize)]
            struct Payload<'a> {
                model: &'a str,
                messages: &'a Vec<crate::providers::types::ChatMessage>,
            }
            let payload = Payload {
                model: &req.model,
                messages: &req.messages,
            };

            let response = self.client
                .post("https://openrouter.ai/api/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&payload)
                .send()
                .await?;

            if !response.status().is_success() {
                let err_text = response.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!("OpenRouter Error: {}", err_text));
            }

            #[derive(serde::Deserialize)]
            struct OpenRouterRes {
                choices: Vec<Choice>,
            }
            #[derive(serde::Deserialize)]
            struct Choice {
                message: Message,
            }
            #[derive(serde::Deserialize)]
            struct Message {
                content: String,
            }

            let mut parsed: OpenRouterRes = response.json().await?;
            let reply_content = if !parsed.choices.is_empty() {
                parsed.choices.remove(0).message.content
            } else {
                "No response from model.".to_string()
            };

            let reply = crate::providers::types::ChatMessage {
                role: crate::providers::types::Role::Assistant,
                content: reply_content,
                pinned: false,
            };
            
            
            Ok(crate::providers::types::ChatResponse {
                message: reply,
                input_tokens: 0,
                output_tokens: 0,
            })
        })
    }

    fn fetch_models<'a>(&'a self, api_key: &'a str) -> Pin<Box<dyn Future<Output = Result<Vec<String>>> + Send + 'a>> {
        Box::pin(async move {
            let response = self.client
                .get("https://openrouter.ai/api/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await?;
            if !response.status().is_success() {
                return Err(anyhow::anyhow!("Failed to fetch OpenRouter models"));
            }
            #[derive(serde::Deserialize)]
            struct ModelObj { id: String }
            #[derive(serde::Deserialize)]
            struct ModelRes { data: Vec<ModelObj> }
            
            let parsed: ModelRes = response.json().await?;
            Ok(parsed.data.into_iter().map(|m| m.id).collect())
        })
    }
}

pub struct GeminiAdapter {
    client: Client,
}

impl GeminiAdapter {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl ProviderAdapter for GeminiAdapter {
    fn validate_credentials<'a>(&'a self, api_key: &'a str) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={}", api_key);
            let response = self.client.get(&url).send().await?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Invalid Gemini API Key"))
            }
        })
    }
    
    fn chat<'a>(&'a self, api_key: &'a str, req: crate::providers::types::ChatRequest) -> Pin<Box<dyn Future<Output = Result<crate::providers::types::ChatResponse>> + Send + 'a>> {
        Box::pin(async move {
            // Gemini의 OpenAI 호환 엔드포인트를 사용하여 완벽한 구조체 호환 통신 수행
            #[derive(serde::Serialize)]
            struct Payload<'a> {
                model: &'a str,
                messages: &'a Vec<crate::providers::types::ChatMessage>,
            }
            let payload = Payload {
                model: &req.model,
                messages: &req.messages,
            };

            let response = self.client
                .post("https://generativelanguage.googleapis.com/v1beta/openai/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&payload)
                .send()
                .await?;

            if !response.status().is_success() {
                let err_text = response.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!("Gemini Error: {}", err_text));
            }

            #[derive(serde::Deserialize)]
            struct GeminiRes { choices: Vec<Choice> }
            #[derive(serde::Deserialize)]
            struct Choice { message: Message }
            #[derive(serde::Deserialize)]
            struct Message { content: String }

            let mut parsed: GeminiRes = response.json().await?;
            let reply_content = if !parsed.choices.is_empty() {
                parsed.choices.remove(0).message.content
            } else {
                "No response from Gemini API.".to_string()
            };

            let reply = crate::providers::types::ChatMessage {
                role: crate::providers::types::Role::Assistant,
                content: reply_content,
                pinned: false,
            };
            
            Ok(crate::providers::types::ChatResponse {
                message: reply,
                input_tokens: 0,
                output_tokens: 0,
            })
        })
    }

    fn fetch_models<'a>(&'a self, api_key: &'a str) -> Pin<Box<dyn Future<Output = Result<Vec<String>>> + Send + 'a>> {
        Box::pin(async move {
            let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={}", api_key);
            let response = self.client.get(&url).send().await?;
            if !response.status().is_success() {
                return Err(anyhow::anyhow!("Failed to fetch Gemini models"));
            }
            #[derive(serde::Deserialize)]
            struct ModelObj { name: String }
            #[derive(serde::Deserialize)]
            struct ModelRes { models: Vec<ModelObj> }
            
            let parsed: ModelRes = response.json().await?;
            // Gemini API 리턴은 "models/gemini-1.5-pro" 형태
            Ok(parsed.models.into_iter().map(|m| m.name).collect())
        })
    }
}

pub fn get_adapter(kind: &ProviderKind) -> Box<dyn ProviderAdapter> {
    match kind {
        ProviderKind::OpenRouter => Box::new(OpenRouterAdapter::new()),
        ProviderKind::Google => Box::new(GeminiAdapter::new()),
    }
}
