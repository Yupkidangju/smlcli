pub fn redact_sensitive_text(text: &str, secrets: &[&str]) -> String {
    let mut redacted = text.to_string();
    for secret in secrets.iter().filter(|secret| secret.len() > 5) {
        redacted = redacted.replace(secret, "[REDACTED]");
    }

    static AUTH: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static QUERY: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static JSON_SECRET: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let auth = AUTH.get_or_init(|| {
        regex::Regex::new(r"(?i)(authorization\s*[:=]\s*(?:bearer\s+)?)[^\s,;]+")
            .expect("auth redaction regex")
    });
    let query = QUERY.get_or_init(|| {
        regex::Regex::new(r#"(?i)([?&](?:key|api_key|token|access_token)=)[^&\s\"']+"#)
            .expect("query redaction regex")
    });
    let json_secret = JSON_SECRET.get_or_init(|| {
        regex::Regex::new(r#"(?i)(\"(?:api[_-]?key|token|secret|password)\"\s*:\s*\")[^\"]+"#)
            .expect("json secret redaction regex")
    });
    redacted = auth.replace_all(&redacted, "$1[REDACTED]").to_string();
    redacted = query.replace_all(&redacted, "$1[REDACTED]").to_string();
    json_secret
        .replace_all(&redacted, "$1[REDACTED]")
        .to_string()
}
