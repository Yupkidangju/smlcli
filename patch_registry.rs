use std::fs;

fn main() {
    let path = "src/providers/registry.rs";
    let content = fs::read_to_string(path).unwrap();
    let mut new_content = content.replace("use anyhow::Result;", "use crate::domain::error::ProviderError;");
    
    // Trait definition
    new_content = new_content.replace("Output = Result<()>", "Output = Result<(), ProviderError>");
    new_content = new_content.replace("Output = Result<crate::providers::types::ChatResponse>", "Output = Result<crate::providers::types::ChatResponse, ProviderError>");
    new_content = new_content.replace("Output = Result<Vec<String>>", "Output = Result<Vec<String>, ProviderError>");

    // OpenRouter errors
    new_content = new_content.replace("Err(anyhow::anyhow!(\"Invalid OpenRouter API Key\"))", "Err(ProviderError::AuthenticationFailed(\"Invalid OpenRouter API Key\".into()))");
    new_content = new_content.replace("Err(anyhow::anyhow!(\"OpenRouter Error: {}\", err_text))", "Err(ProviderError::ApiResponse { code: response.status().as_u16(), message: format!(\"OpenRouter Error: {}\", err_text) })");
    new_content = new_content.replace("Err(anyhow::anyhow!(\"OpenRouter Stream Error: {}\", err_text))", "Err(ProviderError::ApiResponse { code: response.status().as_u16(), message: format!(\"OpenRouter Stream Error: {}\", err_text) })");
    new_content = new_content.replace("Err(anyhow::anyhow!(\"Failed to fetch OpenRouter models\"))", "Err(ProviderError::NetworkFailure(\"Failed to fetch OpenRouter models\".into()))");

    // Gemini errors
    new_content = new_content.replace("Err(anyhow::anyhow!(\"Invalid Gemini API Key\"))", "Err(ProviderError::AuthenticationFailed(\"Invalid Gemini API Key\".into()))");
    new_content = new_content.replace("Err(anyhow::anyhow!(\"Gemini Error: {}\", err_text))", "Err(ProviderError::ApiResponse { code: response.status().as_u16(), message: format!(\"Gemini Error: {}\", err_text) })");
    new_content = new_content.replace("Err(anyhow::anyhow!(\"Gemini Stream Error: {}\", err_text))", "Err(ProviderError::ApiResponse { code: response.status().as_u16(), message: format!(\"Gemini Stream Error: {}\", err_text) })");
    new_content = new_content.replace("Err(anyhow::anyhow!(\"Failed to fetch Gemini models\"))", "Err(ProviderError::NetworkFailure(\"Failed to fetch Gemini models\".into()))");

    // Network error mapping for reqwest::Error
    new_content = new_content.replace(".await?;", ".await.map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;");
    new_content = new_content.replace("response.text().await?;", "response.text().await.map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;");
    new_content = new_content.replace("response.json().await?;", "response.json().await.map_err(|e| ProviderError::NetworkFailure(e.to_string()))?;");

    fs::write(path, new_content).unwrap();
}
