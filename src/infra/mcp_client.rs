// [v3.4.0] Phase 44 Task D-2: TECH-DEBT 정리 완료. 파일 레벨 allow(dead_code) 제거.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{Mutex, mpsc, oneshot};

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
}

struct RpcRequest {
    method: String,
    params: Option<Value>,
    id: Option<u64>,
}

impl McpClient {
    pub async fn spawn(name: &str, cmd: &str, args: &[String]) -> Result<Self> {
        let mut command = Command::new(cmd);
        command.args(args);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command.spawn()?;
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        // [v2.5.3] stderr drain task: 버퍼 블로킹 방지.
        // MCP 서버가 stderr에 많은 출력을 쓰면 OS 파이프 버퍼가 차서
        // stdout 읽기가 블로킹될 수 있으므로, 별도 태스크에서 소비.
        if let Some(stderr) = child.stderr.take() {
            let server_name = name.to_string();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr);
                let mut line = String::new();
                loop {
                    match reader.read_line(&mut line).await {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            // stderr 출력은 디버그 로그로만 소비 (릴리스에서는 무시)
                            #[cfg(debug_assertions)]
                            eprintln!("[MCP:{}:stderr] {}", server_name, line.trim());
                            line.clear();
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        // [v2.5.3] Child 핸들을 Arc<Mutex>로 보관
        let child_handle = Arc::new(Mutex::new(Some(child)));

        let (request_tx, request_rx) = mpsc::channel(32);

        let pending_requests: Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value>>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let request_id_counter = Arc::new(AtomicU64::new(1));

        // Stdin Writer Task
        tokio::spawn(async move {
            Self::writer_task(stdin, request_rx).await;
        });

        // Stdout Reader Task
        let pending_clone2 = pending_requests.clone();
        tokio::spawn(async move {
            Self::reader_task(stdout, pending_clone2).await;
        });

        let client = Self {
            name: name.to_string(),
            request_tx,
            child_handle,
            pending_requests,
            request_id_counter,
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
        }
    }

    /// [v2.5.3] 감사 HIGH-2: 앱 종료 시 MCP 서버 자식 프로세스를 명시적으로 종료.
    /// 호출하지 않으면 자식 프로세스가 좀비로 남을 수 있음.
    pub async fn shutdown(&self) {
        let mut guard = self.child_handle.lock().await;
        if let Some(mut child) = guard.take() {
            let _ = child.kill().await;
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
        let mut line = String::new();
        while let Ok(bytes) = reader.read_line(&mut line).await {
            if bytes == 0 {
                break; // EOF
            }
            if let Ok(parsed) = serde_json::from_str::<Value>(&line)
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
            line.clear();
        }

        // [v3.7.1] EOF 시 pending 요청들 모두 에러 처리
        let mut p = pending.lock().await;
        for (_, tx) in p.drain() {
            let _ = tx.send(Err(anyhow::anyhow!("MCP Client disconnected (EOF)")));
        }
    }

    async fn send_request(&self, method: &str, params: Option<Value>) -> Result<Value> {
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

        match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
            Ok(Ok(res)) => res,
            Ok(Err(e)) => Err(e.into()),
            Err(_) => {
                self.pending_requests.lock().await.remove(&request_id);
                Err(anyhow::anyhow!("MCP Request timeout"))
            }
        }
    }

    async fn send_notification(&self, method: &str, params: Option<Value>) -> Result<()> {
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
            let mut tools = Vec::new();
            for t in tools_arr {
                if let Ok(info) = serde_json::from_value(t.clone()) {
                    tools.push(info);
                }
            }
            Ok(tools)
        } else {
            Ok(Vec::new())
        }
    }

    /// [v3.3.3] 감사 HIGH-1 수정: MCP tools/call 응답 처리.
    /// MCP 공식 스키마(CallToolResult)에 따르면 `content`와 `isError`는 동시에 존재할 수 있다.
    /// `isError: true`이면 content 내용은 에러 메시지이므로, isError를 먼저 검사해야 한다.
    /// 이전: content가 있으면 즉시 Ok 반환 → isError:true 에러가 성공으로 전파됨.
    /// 수정: isError 검사를 최우선으로 수행하고, content에서 에러 메시지를 추출하여 전달.
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<String> {
        let params = json!({
            "name": name,
            "arguments": arguments
        });
        let res = self.send_request("tools/call", Some(params)).await?;
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
            Ok(res.to_string())
        }
    }
}
