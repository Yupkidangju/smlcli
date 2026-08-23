// [v3.4.0] Phase 44 Task D-2: TECH-DEBT 정리 완료. 파일 레벨 allow(dead_code) 제거.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{Mutex, mpsc, oneshot};

const MAX_RPC_LINE_BYTES: usize = 1024 * 1024;
const MAX_RPC_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_MCP_TOOLS: usize = 128;
const MAX_TOOL_INFO_BYTES: usize = 64 * 1024;
const MAX_TOOL_RESULT_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInfo {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// [v2.5.3] 감사 HIGH-2: Child 핸들 보관 + shutdown 지원.
/// Arc<Mutex<Option<Child>>>로 자식 프로세스를 추적하여
/// 앱 종료 시 명시적 kill이 가능하고, stderr drain으로 블로킹 방지.
#[derive(Debug, Clone)]
pub struct McpClient {
    #[allow(dead_code)] // [v3.7.0] MCP 서버 로그 식별자로 사용 예정
    name: String,
    request_tx: mpsc::Sender<RpcRequest>,
    child_handle: Arc<Mutex<Option<Child>>>,
    pending_requests: Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value>>>>>,
    request_id_counter: Arc<AtomicU64>,
    tool_schemas: Arc<Mutex<HashMap<String, Value>>>,
    tasks: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

struct RpcRequest {
    method: String,
    params: Option<Value>,
    id: Option<u64>,
}

async fn read_bounded_line<R>(reader: &mut R, max_bytes: usize) -> std::io::Result<Option<Vec<u8>>>
where
    R: AsyncBufRead + Unpin,
{
    let mut line = Vec::new();
    loop {
        let available = reader.fill_buf().await?;
        if available.is_empty() {
            return if line.is_empty() {
                Ok(None)
            } else {
                Ok(Some(line))
            };
        }
        let take = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |index| index + 1);
        if line.len().saturating_add(take) > max_bytes {
            reader.consume(take);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("MCP JSON-RPC line이 {max_bytes} bytes를 초과했습니다"),
            ));
        }
        let complete = available[..take].last() == Some(&b'\n');
        line.extend_from_slice(&available[..take]);
        reader.consume(take);
        if complete {
            return Ok(Some(line));
        }
    }
}

impl McpClient {
    #[cfg(test)]
    pub async fn spawn(name: &str, cmd: &str, args: &[String]) -> Result<Self> {
        Self::spawn_with_env_allowlist(name, cmd, args, &[]).await
    }

    pub async fn spawn_with_env_allowlist(
        name: &str,
        cmd: &str,
        args: &[String],
        allowed_env_vars: &[String],
    ) -> Result<Self> {
        let mut command = Command::new(cmd);
        command.args(args);
        Self::configure_child_environment(&mut command, allowed_env_vars)?;
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command.kill_on_drop(true);
        #[cfg(unix)]
        command.process_group(0);

        let mut child = command.spawn()?;
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take();

        // [v2.5.3] Child 핸들을 Arc<Mutex>로 보관
        let child_handle = Arc::new(Mutex::new(Some(child)));

        let (request_tx, request_rx) = mpsc::channel(32);

        let pending_requests: Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value>>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let request_id_counter = Arc::new(AtomicU64::new(1));
        let tool_schemas = Arc::new(Mutex::new(HashMap::new()));
        let tasks = Arc::new(Mutex::new(Vec::new()));

        // Stdin Writer Task
        let writer = tokio::spawn(async move {
            Self::writer_task(stdin, request_rx).await;
        });

        // Stdout Reader Task
        let pending_clone2 = pending_requests.clone();
        let reader = tokio::spawn(async move {
            Self::reader_task(stdout, pending_clone2).await;
        });

        let stderr_drain = stderr.map(|mut stderr| {
            tokio::spawn(async move {
                // stderr is untrusted and may contain secrets or unbounded lines.
                // Drain to a sink without retaining or rendering it.
                let _ = tokio::io::copy(&mut stderr, &mut tokio::io::sink()).await;
            })
        });
        {
            let mut task_guard = tasks.lock().await;
            task_guard.push(writer);
            task_guard.push(reader);
            if let Some(stderr_drain) = stderr_drain {
                task_guard.push(stderr_drain);
            }
        }

        let client = Self {
            name: name.to_string(),
            request_tx,
            child_handle,
            pending_requests,
            request_id_counter,
            tool_schemas,
            tasks,
        };

        // [v3.3.2] 감사 HIGH-2 수정: initialize() 실패 시 child process leak 방지.
        // 이전: initialize().await? 실패 시 child가 그대로 남아 좀비 프로세스화.
        // 수정: 실패 시 child_handle로 명시적 kill 후 에러 반환.
        if let Err(e) = client.initialize().await {
            client.shutdown().await;
            return Err(e);
        }

        Ok(client)
    }

    fn configure_child_environment(
        command: &mut Command,
        allowed_env_vars: &[String],
    ) -> Result<()> {
        const BASE_ENV: &[&str] = &[
            "PATH", "HOME", "USER", "LOGNAME", "SHELL", "TERM", "LANG", "TMPDIR",
        ];

        command.env_clear();
        for name in BASE_ENV {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        for (name, value) in std::env::vars_os() {
            if name.to_string_lossy().starts_with("LC_") {
                command.env(name, value);
            }
        }

        for name in allowed_env_vars {
            Self::validate_allowed_env_name(name)?;
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        Ok(())
    }

    fn validate_allowed_env_name(name: &str) -> Result<()> {
        let valid = !name.is_empty()
            && name.bytes().enumerate().all(|(idx, byte)| {
                byte == b'_' || byte.is_ascii_alphabetic() || (idx > 0 && byte.is_ascii_digit())
            });
        if !valid {
            return Err(anyhow::anyhow!(
                "MCP environment 변수 이름이 유효하지 않습니다: {name}"
            ));
        }

        let upper = name.to_ascii_uppercase();
        if ["KEY", "TOKEN", "SECRET", "PASSWORD", "CREDENTIAL"]
            .iter()
            .any(|marker| upper.contains(marker))
        {
            return Err(anyhow::anyhow!(
                "MCP environment allowlist는 secret-like 변수 이름을 허용하지 않습니다: {name}"
            ));
        }
        Ok(())
    }

    #[allow(dead_code)] // [v3.7.0] MCP 서버 로그 식별자로 사용 예정
    pub fn name(&self) -> &str {
        &self.name
    }

    /// [v3.3.9] 테스트 전용 더미 McpClient 생성자.
    /// 실제 프로세스를 spawn하지 않고, handle_action 관통 테스트에서
    /// McpToolsLoaded 액션을 구성하기 위해 사용.
    #[cfg(test)]
    pub(crate) fn dummy(name: &str) -> Self {
        let (tx, _rx) = mpsc::channel(1);
        Self {
            name: name.to_string(),
            request_tx: tx,
            child_handle: Arc::new(Mutex::new(None)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            request_id_counter: Arc::new(AtomicU64::new(1)),
            tool_schemas: Arc::new(Mutex::new(HashMap::new())),
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// [v2.5.3] 감사 HIGH-2: 앱 종료 시 MCP 서버 자식 프로세스를 명시적으로 종료.
    /// 호출하지 않으면 자식 프로세스가 좀비로 남을 수 있음.
    pub async fn shutdown(&self) {
        {
            let mut pending = self.pending_requests.lock().await;
            for (_, sender) in pending.drain() {
                let _ = sender.send(Err(anyhow::anyhow!("MCP Client shutdown")));
            }
        }

        let child = self.child_handle.lock().await.take();
        if let Some(mut child) = child {
            #[cfg(unix)]
            if let Some(pid) = child.id() {
                unsafe {
                    libc::kill(-(pid as i32), libc::SIGTERM);
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                unsafe {
                    libc::kill(-(pid as i32), libc::SIGKILL);
                }
                let _ = child.wait().await;
            }
            #[cfg(not(unix))]
            {
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
        }

        let tasks = {
            let mut tasks = self.tasks.lock().await;
            std::mem::take(&mut *tasks)
        };
        for task in tasks {
            task.abort();
            let _ = task.await;
        }
    }

    async fn writer_task(mut stdin: ChildStdin, mut rx: mpsc::Receiver<RpcRequest>) {
        while let Some(req) = rx.recv().await {
            let mut msg = json!({
                "jsonrpc": "2.0",
                "method": req.method,
            });

            if let Some(p) = req.params {
                msg.as_object_mut().unwrap().insert("params".to_string(), p);
            }

            if let Some(request_id) = req.id {
                msg.as_object_mut()
                    .unwrap()
                    .insert("id".to_string(), json!(request_id));
            }

            let mut out = msg.to_string();
            out.push('\n');

            if stdin.write_all(out.as_bytes()).await.is_err() {
                break;
            }
        }
    }

    async fn reader_task(
        stdout: tokio::process::ChildStdout,
        pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value>>>>>,
    ) {
        let mut reader = BufReader::new(stdout);
        while let Ok(Some(line)) = read_bounded_line(&mut reader, MAX_RPC_LINE_BYTES).await {
            if let Ok(parsed) = serde_json::from_slice::<Value>(&line)
                && let Some(id_val) = parsed.get("id").and_then(|v| v.as_u64())
            {
                let is_error = parsed.get("error").is_some();
                let mut p = pending.lock().await;
                if let Some(tx) = p.remove(&id_val) {
                    if is_error {
                        let _ = tx.send(Err(anyhow::anyhow!("MCP Error: {}", parsed["error"])));
                    } else if let Some(result) = parsed.get("result") {
                        let _ = tx.send(Ok(result.clone()));
                    } else {
                        let _ = tx.send(Err(anyhow::anyhow!("No result field in response")));
                    }
                }
            }
        }

        // [v3.7.1] EOF 시 pending 요청들 모두 에러 처리
        let mut p = pending.lock().await;
        for (_, tx) in p.drain() {
            let _ = tx.send(Err(anyhow::anyhow!(
                "MCP Client disconnected or response exceeded the size limit"
            )));
        }
    }

    async fn send_request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        self.send_request_with_cancellation(method, params, None)
            .await
    }

    async fn send_request_with_cancellation(
        &self,
        method: &str,
        params: Option<Value>,
        cancellation: Option<tokio_util::sync::CancellationToken>,
    ) -> Result<Value> {
        let request_size =
            method.len() + params.as_ref().map_or(0, |value| value.to_string().len());
        if request_size > MAX_RPC_REQUEST_BYTES {
            return Err(anyhow::anyhow!(
                "MCP request가 {} bytes 제한을 초과했습니다",
                MAX_RPC_REQUEST_BYTES
            ));
        }
        let request_id = self.request_id_counter.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        self.pending_requests.lock().await.insert(request_id, tx);

        if self
            .request_tx
            .send(RpcRequest {
                method: method.to_string(),
                params,
                id: Some(request_id),
            })
            .await
            .is_err()
        {
            self.pending_requests.lock().await.remove(&request_id);
            return Err(anyhow::anyhow!("MCP Client disconnected"));
        }

        let response = async {
            match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
                Ok(Ok(res)) => res,
                Ok(Err(error)) => Err(error.into()),
                Err(_) => Err(anyhow::anyhow!("MCP Request timeout")),
            }
        };
        let result = if let Some(cancellation) = cancellation {
            tokio::select! {
                result = response => result,
                _ = cancellation.cancelled() => Err(anyhow::anyhow!("MCP Request cancelled")),
            }
        } else {
            response.await
        };
        if result.is_err() {
            self.pending_requests.lock().await.remove(&request_id);
        }
        result
    }

    async fn send_notification(&self, method: &str, params: Option<Value>) -> Result<()> {
        let request_size =
            method.len() + params.as_ref().map_or(0, |value| value.to_string().len());
        if request_size > MAX_RPC_REQUEST_BYTES {
            return Err(anyhow::anyhow!(
                "MCP notification이 {} bytes 제한을 초과했습니다",
                MAX_RPC_REQUEST_BYTES
            ));
        }
        self.request_tx
            .send(RpcRequest {
                method: method.to_string(),
                params,
                id: None,
            })
            .await
            .map_err(|_| anyhow::anyhow!("MCP Client disconnected"))?;
        Ok(())
    }

    pub async fn initialize(&self) -> Result<()> {
        let params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "smlcli",
                "version": env!("CARGO_PKG_VERSION")
            }
        });
        let _res = self.send_request("initialize", Some(params)).await?;
        self.send_notification("notifications/initialized", None)
            .await?;
        Ok(())
    }

    pub async fn list_tools(&self) -> Result<Vec<McpToolInfo>> {
        let res = self.send_request("tools/list", None).await?;
        if let Some(tools_arr) = res.get("tools").and_then(|v| v.as_array()) {
            if tools_arr.len() > MAX_MCP_TOOLS {
                return Err(anyhow::anyhow!(
                    "MCP tool count가 {}개 제한을 초과했습니다",
                    MAX_MCP_TOOLS
                ));
            }
            let mut tools = Vec::new();
            for t in tools_arr {
                let info: McpToolInfo = serde_json::from_value(t.clone())
                    .map_err(|error| anyhow::anyhow!("MCP tool schema parse 실패: {error}"))?;
                Self::validate_tool_info(&info)?;
                tools.push(info);
            }
            let schemas = tools
                .iter()
                .map(|tool| (tool.name.clone(), tool.input_schema.clone()))
                .collect();
            *self.tool_schemas.lock().await = schemas;
            Ok(tools)
        } else {
            Err(anyhow::anyhow!(
                "MCP tools/list result에 tools array가 없습니다"
            ))
        }
    }

    pub(crate) fn validate_tool_info(info: &McpToolInfo) -> Result<()> {
        let encoded_size = serde_json::to_vec(info)?.len();
        if encoded_size > MAX_TOOL_INFO_BYTES {
            return Err(anyhow::anyhow!(
                "MCP tool info가 {} bytes 제한을 초과했습니다: {}",
                MAX_TOOL_INFO_BYTES,
                info.name
            ));
        }
        if info.name.is_empty() || info.name.len() > 256 {
            return Err(anyhow::anyhow!("MCP tool name 길이가 유효하지 않습니다"));
        }
        let schema = info
            .input_schema
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("MCP inputSchema는 object여야 합니다"))?;
        if schema.get("type").and_then(Value::as_str) != Some("object") {
            return Err(anyhow::anyhow!(
                "MCP inputSchema.type은 object여야 합니다: {}",
                info.name
            ));
        }
        if schema
            .get("properties")
            .is_some_and(|properties| !properties.is_object())
        {
            return Err(anyhow::anyhow!(
                "MCP inputSchema.properties는 object여야 합니다: {}",
                info.name
            ));
        }
        if schema.get("required").is_some_and(|required| {
            required
                .as_array()
                .is_none_or(|entries| entries.iter().any(|entry| !entry.is_string()))
        }) {
            return Err(anyhow::anyhow!(
                "MCP inputSchema.required는 string array여야 합니다: {}",
                info.name
            ));
        }
        Ok(())
    }

    async fn validate_tool_arguments(&self, name: &str, arguments: &Value) -> Result<()> {
        let schemas = self.tool_schemas.lock().await;
        let schema = schemas
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("advertised schema가 없는 MCP tool입니다: {name}"))?;
        let arguments = arguments
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("MCP tool arguments는 object여야 합니다"))?;
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for name in required.iter().filter_map(Value::as_str) {
                if !arguments.contains_key(name) {
                    return Err(anyhow::anyhow!("필수 MCP argument가 없습니다: {name}"));
                }
            }
        }
        if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
            for (name, value) in arguments {
                let Some(expected) = properties
                    .get(name)
                    .and_then(|property| property.get("type"))
                    .and_then(Value::as_str)
                else {
                    continue;
                };
                let valid = match expected {
                    "string" => value.is_string(),
                    "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
                    "number" => value.is_number(),
                    "boolean" => value.is_boolean(),
                    "object" => value.is_object(),
                    "array" => value.is_array(),
                    "null" => value.is_null(),
                    _ => false,
                };
                if !valid {
                    return Err(anyhow::anyhow!(
                        "MCP argument type 불일치: {name}은 {expected}여야 합니다"
                    ));
                }
            }
        }
        Ok(())
    }

    /// [v3.3.3] 감사 HIGH-1 수정: MCP tools/call 응답 처리.
    /// MCP 공식 스키마(CallToolResult)에 따르면 `content`와 `isError`는 동시에 존재할 수 있다.
    /// `isError: true`이면 content 내용은 에러 메시지이므로, isError를 먼저 검사해야 한다.
    /// 이전: content가 있으면 즉시 Ok 반환 → isError:true 에러가 성공으로 전파됨.
    /// 수정: isError 검사를 최우선으로 수행하고, content에서 에러 메시지를 추출하여 전달.
    #[cfg(test)]
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<String> {
        self.call_tool_cancellable(name, arguments, tokio_util::sync::CancellationToken::new())
            .await
    }

    pub async fn call_tool_cancellable(
        &self,
        name: &str,
        arguments: Value,
        cancellation: tokio_util::sync::CancellationToken,
    ) -> Result<String> {
        self.validate_tool_arguments(name, &arguments).await?;
        let params = json!({
            "name": name,
            "arguments": arguments
        });
        let res = self
            .send_request_with_cancellation("tools/call", Some(params), Some(cancellation))
            .await?;
        Self::parse_call_tool_result(&res, name)
    }

    /// [v3.3.4] 감사 MEDIUM-3 수정: CallToolResult 파싱 로직을 별도 함수로 추출.
    /// 실제 MCP 서버 연결 없이도 isError/content 파싱 로직을 직접 단위 테스트 가능.
    /// - isError:true이면 content를 에러 메시지로 활용하여 Err 반환.
    /// - isError가 없거나 false이면 content를 성공 출력으로 반환.
    /// - content와 isError 모두 없으면 raw 응답 JSON을 문자열로 반환.
    pub(crate) fn parse_call_tool_result(res: &Value, tool_name: &str) -> Result<String> {
        // isError 검사를 최우선으로 수행
        let is_error = res
            .get("isError")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // content 배열에서 text 항목 수집
        let mut output = String::new();
        if let Some(content) = res.get("content").and_then(|v| v.as_array()) {
            for item in content {
                if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                    if output.len().saturating_add(text.len()) > MAX_TOOL_RESULT_BYTES {
                        return Err(anyhow::anyhow!(
                            "MCP tool result가 {} bytes 제한을 초과했습니다",
                            MAX_TOOL_RESULT_BYTES
                        ));
                    }
                    output.push_str(text);
                    output.push('\n');
                }
            }
        }
        let output = output.trim().to_string();

        if is_error {
            // content가 있으면 에러 메시지로 활용, 없으면 일반 메시지
            let err_msg = if output.is_empty() {
                format!("MCP 도구 '{}' 실행 실패 (상세 없음)", tool_name)
            } else {
                format!("MCP 도구 '{}' 실행 실패: {}", tool_name, output)
            };
            Err(anyhow::anyhow!(err_msg))
        } else if !output.is_empty() {
            Ok(output)
        } else {
            // content도 isError도 없는 경우: raw 응답 반환
            let raw = res.to_string();
            if raw.len() > MAX_TOOL_RESULT_BYTES {
                Err(anyhow::anyhow!(
                    "MCP tool result가 {} bytes 제한을 초과했습니다",
                    MAX_TOOL_RESULT_BYTES
                ))
            } else {
                Ok(raw)
            }
        }
    }

    #[cfg(test)]
    pub(crate) async fn pending_request_count(&self) -> usize {
        self.pending_requests.lock().await.len()
    }
}
