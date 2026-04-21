use crate::domain::permissions::PermissionToken;
use crate::domain::tool_result::{ToolCall, ToolResult};
use anyhow::Result;

pub async fn execute_tool(call: ToolCall, token: &PermissionToken) -> Result<ToolResult> {
    if let Some(tool) = crate::tools::registry::GLOBAL_REGISTRY.get_tool(&call.name) {
        let is_dest = tool.is_destructive(&call.args);
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string());

        let mut safe_to_rollback = false;
        if is_dest {
            safe_to_rollback =
                crate::tools::git_checkpoint::create_checkpoint(&cwd, tool.name()).unwrap_or(false);
        }

        let ctx = crate::tools::registry::ToolContext { token };
        let mut result = tool
            .execute(call.args, &ctx)
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e));

        if is_dest {
            let should_rollback = match &result {
                Ok(res) => res.is_error,
                Err(_) => true,
            };

            if should_rollback && safe_to_rollback {
                let _ = crate::tools::git_checkpoint::rollback_checkpoint(&cwd);
                if let Ok(mut res) = result {
                    res.stderr.push_str("\n[Auto-Verify] 도구 실행이 실패하여 코드 변경사항이 자동으로 롤백되었습니다.");
                    result = Ok(res);
                }
            } else if should_rollback
                && !safe_to_rollback
                && let Ok(mut res) = result
            {
                res.stderr.push_str("\n[Auto-Verify] 워킹 트리에 저장되지 않은 변경사항(WIP)이 있어 롤백을 건너뛰었습니다.");
                result = Ok(res);
            }
        }

        result
    } else {
        Err(anyhow::anyhow!("Unknown tool: {}", call.name))
    }
}
