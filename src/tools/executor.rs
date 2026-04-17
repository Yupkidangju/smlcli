use crate::domain::permissions::PermissionToken;
use crate::domain::tool_result::{ToolCall, ToolResult};
use anyhow::Result;

pub async fn execute_tool(call: ToolCall, token: &PermissionToken) -> Result<ToolResult> {
    if let Some(tool) = crate::tools::registry::GLOBAL_REGISTRY.get_tool(&call.name) {
        let ctx = crate::tools::registry::ToolContext { token };
        tool.execute(call.args, &ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))
    } else {
        Err(anyhow::anyhow!("Unknown tool: {}", call.name))
    }
}
