use async_trait::async_trait;
use serde_json::Value;

use crate::domain::error::ToolError;
use crate::domain::permissions::PermissionResult;
use crate::domain::settings::PersistedSettings;
use crate::domain::tool_result::ToolResult;

/// 도구 실행에 필요한 문맥(Context)
pub struct ToolContext<'a> {
    pub token: &'a crate::domain::permissions::PermissionToken,
}

/// [v0.1.0-beta.23] Phase 13: Agentic Autonomy
/// 다형성 기반 도구 인터페이스. 기존 match 분기를 대체합니다.
#[async_trait]
pub trait Tool: Send + Sync {
    /// 도구의 이름 (예: "ReadFile")
    fn name(&self) -> &'static str;

    /// 도구의 설명
    fn description(&self) -> &'static str;

    /// Provider에 전달될 JSON Schema
    fn schema(&self) -> Value;

    /// 실행 전 권한 검사 (PermissionEngine에서 호출됨)
    fn check_permission(&self, args: &Value, settings: &PersistedSettings) -> PermissionResult;

    /// 승인 대기 시 사용할 포맷팅 (옵션) - 예: "승인 대기 (y/n) — 명령: 'cargo test'"
    fn format_detail(&self, _args: &Value) -> String {
        format!("승인 대기 (y/n) — 도구: {}", self.name())
    }

    /// 승인 대기 시 보여줄 Diff Preview (옵션)
    fn generate_diff_preview(&self, _args: &Value) -> Option<String> {
        None
    }

    /// 이 도구 실행이 코드 베이스를 파괴적(destructive)으로 변경하는지 여부
    /// (Phase 13: Git Checkpoint 자동 생성 트리거에 활용됨)
    fn is_destructive(&self, _args: &Value) -> bool {
        false
    }

    /// 도구 실제 실행
    async fn execute(&self, args: Value, ctx: &ToolContext<'_>) -> Result<ToolResult, ToolError>;
}

/// 도구를 관리하는 레지스트리
pub struct ToolRegistry {
    tools: std::collections::HashMap<&'static str, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name(), tool);
    }

    pub fn get_tool(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    pub fn all_schemas(&self, dialect: &crate::domain::provider::ToolDialect) -> Vec<Value> {
        self.tools
            .values()
            .map(|t| {
                let mut schema = t.schema();
                apply_dialect(&mut schema, dialect);
                schema
            })
            .collect()
    }
}

/// Provider 방언(Dialect)에 맞게 JSON 스키마를 가공한다.
fn apply_dialect(schema: &mut Value, dialect: &crate::domain::provider::ToolDialect) {
    if dialect == &crate::domain::provider::ToolDialect::Gemini {
        // Gemini: parameters 객체가 있을 경우, required 배열이 없으면 명시적으로 빈 배열이라도 넣어주는 것이 안전함
        if let Some(func) = schema.get_mut("function")
            && let Some(params) = func.get_mut("parameters")
                && params.get("required").is_none() {
                    params["required"] = serde_json::json!([]);
                }
    }
}

pub static GLOBAL_REGISTRY: std::sync::LazyLock<ToolRegistry> = std::sync::LazyLock::new(|| {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(crate::tools::file_ops::ReadFileTool));
    registry.register(Box::new(crate::tools::file_ops::WriteFileTool));
    registry.register(Box::new(crate::tools::file_ops::ReplaceFileContentTool));
    registry.register(Box::new(crate::tools::sys_ops::ListDirTool));
    registry.register(Box::new(crate::tools::sys_ops::SysInfoTool));
    registry.register(Box::new(crate::tools::sys_ops::StatTool));
    registry.register(Box::new(crate::tools::grep::GrepSearchTool));
    registry.register(Box::new(crate::tools::shell::ExecShellTool));
    registry
});
