use std::fs;

fn main() {
    let path = "src/providers/registry.rs";
    let content = fs::read_to_string(path).unwrap();
    
    let mut new_content = content.replace(
        "let err_text = response.text().await.unwrap_or_default();\n                return Err(ProviderError::ApiResponse { code: response.status().as_u16(), message: format!(\"OpenRouter Error: {}\", err_text) });",
        "let code = response.status().as_u16();\n                let err_text = response.text().await.unwrap_or_default();\n                return Err(ProviderError::ApiResponse { code, message: format!(\"OpenRouter Error: {}\", err_text) });"
    );

    new_content = new_content.replace(
        "let err_text = response.text().await.unwrap_or_default();\n                return Err(ProviderError::ApiResponse { code: response.status().as_u16(), message: format!(\"OpenRouter Stream Error: {}\", err_text) });",
        "let code = response.status().as_u16();\n                let err_text = response.text().await.unwrap_or_default();\n                return Err(ProviderError::ApiResponse { code, message: format!(\"OpenRouter Stream Error: {}\", err_text) });"
    );

    new_content = new_content.replace(
        "let err_text = response.text().await.unwrap_or_default();\n                return Err(ProviderError::ApiResponse { code: response.status().as_u16(), message: format!(\"Gemini Error: {}\", err_text) });",
        "let code = response.status().as_u16();\n                let err_text = response.text().await.unwrap_or_default();\n                return Err(ProviderError::ApiResponse { code, message: format!(\"Gemini Error: {}\", err_text) });"
    );

    new_content = new_content.replace(
        "let err_text = response.text().await.unwrap_or_default();\n                return Err(ProviderError::ApiResponse { code: response.status().as_u16(), message: format!(\"Gemini Stream Error: {}\", err_text) });",
        "let code = response.status().as_u16();\n                let err_text = response.text().await.unwrap_or_default();\n                return Err(ProviderError::ApiResponse { code, message: format!(\"Gemini Stream Error: {}\", err_text) });"
    );

    fs::write(path, new_content).unwrap();
}
