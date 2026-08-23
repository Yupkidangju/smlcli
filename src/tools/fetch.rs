use crate::domain::error::ToolError;
use crate::domain::permissions::PermissionResult;
use crate::domain::settings::PersistedSettings;
use crate::domain::tool_result::ToolResult;
use crate::tools::registry::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

const MAX_REDIRECTS: usize = 5;
const MAX_WIRE_BYTES: usize = 5 * 1024 * 1024;
const MAX_RENDERED_BYTES: usize = 10_000;

#[derive(Clone)]
struct PublicDnsResolver;

impl reqwest::dns::Resolve for PublicDnsResolver {
    fn resolve(&self, name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        let host = name.as_str().to_string();
        Box::pin(async move {
            let addresses = tokio::net::lookup_host((host.as_str(), 0))
                .await
                .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> { Box::new(error) })?
                .collect::<Vec<_>>();
            if addresses.is_empty() {
                return Err(Box::new(std::io::Error::other(format!(
                    "DNS 결과가 없습니다: {host}"
                )))
                    as Box<dyn std::error::Error + Send + Sync>);
            }
            if let Some(blocked) = addresses.iter().find(|address| !is_public_ip(address.ip())) {
                return Err(Box::new(std::io::Error::other(format!(
                    "non-public DNS destination이 차단되었습니다: {}",
                    blocked.ip()
                )))
                    as Box<dyn std::error::Error + Send + Sync>);
            }
            Ok(Box::new(addresses.into_iter()) as reqwest::dns::Addrs)
        })
    }
}

pub(crate) fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_public_ipv4(ip),
        IpAddr::V6(ip) => is_public_ipv6(ip),
    }
}

fn is_public_ipv4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !matches!(
        (a, b, c),
        (0, _, _)
            | (10, _, _)
            | (100, 64..=127, _)
            | (127, _, _)
            | (169, 254, _)
            | (172, 16..=31, _)
            | (192, 0, 0 | 2)
            | (192, 88, 99)
            | (192, 168, _)
            | (198, 18 | 19, _)
            | (198, 51, 100)
            | (203, 0, 113)
            | (224..=255, _, _)
    )
}

fn is_public_ipv6(ip: Ipv6Addr) -> bool {
    if let Some(mapped) = ip.to_ipv4_mapped() {
        return is_public_ipv4(mapped);
    }
    let segments = ip.segments();
    !(ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_multicast()
        || segments[0] & 0xfe00 == 0xfc00
        || segments[0] & 0xffc0 == 0xfe80
        || segments[0] & 0xffc0 == 0xfec0
        || (segments[0] == 0x2001 && segments[1] == 0x0db8))
}

fn validate_public_url(url: &str) -> Result<reqwest::Url, ToolError> {
    let parsed = reqwest::Url::parse(url)
        .map_err(|error| ToolError::InvalidArguments(format!("유효하지 않은 URL: {error}")))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(ToolError::InvalidArguments(
            "URL scheme은 http 또는 https여야 합니다".to_string(),
        ));
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(ToolError::InvalidArguments(
            "URL credential(userinfo)은 허용되지 않습니다".to_string(),
        ));
    }
    let Some(host) = parsed.host_str() else {
        return Err(ToolError::InvalidArguments(
            "URL host가 필요합니다".to_string(),
        ));
    };
    let ip_literal = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(host);
    if let Ok(ip) = ip_literal.parse::<IpAddr>() {
        if !is_public_ip(ip) {
            return Err(ToolError::PermissionDenied(format!(
                "non-public IP destination이 차단되었습니다: {ip}"
            )));
        }
    } else {
        let lower = host.trim_end_matches('.').to_ascii_lowercase();
        if lower == "localhost"
            || lower.ends_with(".localhost")
            || lower.ends_with(".local")
            || lower.ends_with(".internal")
            || lower.ends_with(".home")
            || lower.ends_with(".lan")
            || lower == "metadata.google.internal"
        {
            return Err(ToolError::PermissionDenied(format!(
                "local/metadata hostname이 차단되었습니다: {host}"
            )));
        }
    }
    Ok(parsed)
}

pub(crate) fn truncate_utf8_output(value: &str, max_bytes: usize) -> (String, bool) {
    if value.len() <= max_bytes {
        return (value.to_string(), false);
    }
    let mut boundary = max_bytes;
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    (value[..boundary].to_string(), true)
}

pub(crate) fn html_to_text(html: &str) -> String {
    static SCRIPT_STYLE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static BLOCK_TAG: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static ANY_TAG: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let without_scripts = SCRIPT_STYLE
        .get_or_init(|| {
            regex::Regex::new(r"(?is)<(?:script|style)\b[^>]*>.*?</(?:script|style)\s*>")
                .expect("script/style regex")
        })
        .replace_all(html, "");
    let with_lines = BLOCK_TAG
        .get_or_init(|| {
            regex::Regex::new(
                r"(?i)</?(?:p|br|div|li|h[1-6]|tr|section|article|header|footer)\b[^>]*>",
            )
            .expect("block tag regex")
        })
        .replace_all(&without_scripts, "\n");
    let text = ANY_TAG
        .get_or_init(|| regex::Regex::new(r"(?s)<[^>]*>").expect("HTML tag regex"))
        .replace_all(&with_lines, "");
    let decoded = text
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");
    decoded
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

pub struct FetchUrlTool;

#[async_trait]
impl Tool for FetchUrlTool {
    fn name(&self) -> &'static str {
        "FetchURL"
    }

    fn description(&self) -> &'static str {
        "Fetch content from a URL via HTTP request."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "FetchURL",
                "description": "Fetch content from a URL via HTTP request. Converts HTML to markdown for readability. Use when extracting text from public pages or documentation.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to fetch content from (must be absolute, e.g. https://...)"
                        }
                    },
                    "required": ["url"]
                }
            }
        })
    }

    // [v2.5.0] ProviderOnly 정책 보안 의미론 정합:
    // ProviderOnly = "오직 LLM 프로바이더 API 엔드포인트만 허용".
    // FetchURL은 사용자가 지정한 임의의 외부 URL을 호출하므로,
    // ProviderOnly 환경에서는 SSRF(Server-Side Request Forgery) 방지를 위해
    // Deny로 처리한다. FetchURL을 사용하려면 AllowAll 정책을 선택해야 한다.
    fn check_permission(&self, args: &Value, settings: &PersistedSettings) -> PermissionResult {
        match settings.network_policy {
            crate::domain::permissions::NetworkPolicy::AllowAll => {
                let url = args.get("url").and_then(|value| value.as_str()).unwrap_or("");
                match validate_public_url(url) {
                    Ok(_) => PermissionResult::Allow,
                    Err(error) => PermissionResult::Deny(error.to_string()),
                }
            }
            crate::domain::permissions::NetworkPolicy::ProviderOnly => {
                PermissionResult::Deny(
                    "FetchURL은 ProviderOnly 정책에서 차단됩니다. 임의 외부 URL 호출은 SSRF 위험이 있으므로, AllowAll 정책으로 변경 후 사용하세요.".to_string(),
                )
            }
            crate::domain::permissions::NetworkPolicy::Deny => {
                PermissionResult::Deny("FetchURL is blocked by NetworkPolicy::Deny.".to_string())
            }
        }
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext<'_>) -> Result<ToolResult, ToolError> {
        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut current_url = validate_public_url(&url)?;
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .dns_resolver(PublicDnsResolver)
            .no_proxy()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(|error| {
                ToolError::ExecutionFailure(format!("Fetch client 생성 실패: {error}"))
            })?;

        let mut redirects = 0usize;
        let mut response = loop {
            let response = client
                .get(current_url.clone())
                .send()
                .await
                .map_err(|error| ToolError::ExecutionFailure(format!("URL fetch 실패: {error}")))?;
            let remote = response.remote_addr().ok_or_else(|| {
                ToolError::ExecutionFailure(
                    "연결된 remote address를 검증할 수 없어 Fetch를 중단합니다".to_string(),
                )
            })?;
            if !is_public_ip(remote.ip()) {
                return Err(ToolError::PermissionDenied(format!(
                    "non-public remote address가 차단되었습니다: {}",
                    remote.ip()
                )));
            }

            if response.status().is_redirection() {
                if redirects >= MAX_REDIRECTS {
                    return Err(ToolError::ExecutionFailure(format!(
                        "redirect가 {MAX_REDIRECTS}회를 초과했습니다"
                    )));
                }
                let location = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .ok_or_else(|| {
                        ToolError::ExecutionFailure(
                            "redirect 응답에 Location header가 없습니다".to_string(),
                        )
                    })?
                    .to_str()
                    .map_err(|_| {
                        ToolError::ExecutionFailure(
                            "redirect Location header가 유효한 UTF-8이 아닙니다".to_string(),
                        )
                    })?;
                let next = current_url.join(location).map_err(|error| {
                    ToolError::ExecutionFailure(format!("redirect URL 해석 실패: {error}"))
                })?;
                current_url = validate_public_url(next.as_str())?;
                redirects += 1;
                continue;
            }
            break response;
        };

        let declared_size = response.content_length().map(|size| size as usize);
        if declared_size.is_some_and(|size| size > MAX_WIRE_BYTES) {
            return Err(ToolError::ExecutionFailure(format!(
                "응답 Content-Length가 {MAX_WIRE_BYTES} bytes를 초과했습니다"
            )));
        }

        let mut body_bytes = Vec::with_capacity(declared_size.unwrap_or(64 * 1024));
        let mut wire_truncated = false;
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| ToolError::ExecutionFailure(format!("Failed to read chunk: {}", e)))?
        {
            let remaining = MAX_WIRE_BYTES.saturating_sub(body_bytes.len());
            if chunk.len() > remaining {
                body_bytes.extend_from_slice(&chunk[..remaining]);
                wire_truncated = true;
                break;
            }
            body_bytes.extend_from_slice(&chunk);
        }

        let content = String::from_utf8_lossy(&body_bytes).to_string();

        let clean_text = html_to_text(&content);

        let original_rendered_size = clean_text.len();
        let (mut rendered, rendered_truncated) =
            truncate_utf8_output(&clean_text, MAX_RENDERED_BYTES);
        if rendered_truncated || wire_truncated {
            rendered.push_str("\n\n... (Content truncated due to size limit) ...");
        }

        Ok(ToolResult {
            tool_name: "FetchURL".to_string(),
            stdout: rendered,
            stderr: String::new(),
            exit_code: 0,
            is_error: false,
            tool_call_id: None,
            is_truncated: rendered_truncated || wire_truncated,
            original_size_bytes: (rendered_truncated || wire_truncated)
                .then_some(declared_size.unwrap_or(original_rendered_size)),
            affected_paths: vec![],
        })
    }
}
