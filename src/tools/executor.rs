use crate::domain::permissions::PermissionToken;
use crate::domain::tool_result::{ToolCall, ToolResult};
use anyhow::Result;
use tokio_util::sync::CancellationToken;

pub async fn execute_tool(
    call: ToolCall,
    token: &PermissionToken,
    cancel_token: CancellationToken,
    settings: std::sync::Arc<crate::domain::settings::PersistedSettings>,
    event_tx: Option<tokio::sync::mpsc::Sender<crate::app::event_loop::Event>>,
) -> Result<ToolResult> {
    if let Some(tool) = crate::tools::registry::GLOBAL_REGISTRY.get_tool(&call.name) {
        let ctx = crate::tools::registry::ToolContext {
            token,
            cancel_token: cancel_token.clone(),
            settings: &settings,
            event_tx,
        };

        let exec_future = tool.execute(call.args, &ctx);

        tokio::select! {
            res = exec_future => res.map_err(|e| anyhow::anyhow!("{:?}", e)),
            _ = cancel_token.cancelled() => {
                Err(anyhow::anyhow!("Tool execution cancelled by user"))
            }
        }
    } else {
        Err(anyhow::anyhow!("Unknown tool: {}", call.name))
    }
}
