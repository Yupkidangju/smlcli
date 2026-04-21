use anyhow::Result;
use ignore::WalkBuilder;
use std::fs;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

#[derive(Debug, Clone, Default)]
pub struct RepoMapState {
    pub cached: Option<String>,
    pub is_loading: bool,
    pub stale: bool,
    pub last_error: Option<String>,
}

impl RepoMapState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn should_refresh(&self) -> bool {
        self.cached.is_none() || self.stale
    }

    pub fn begin_refresh(&mut self) -> bool {
        if self.is_loading || !self.should_refresh() {
            return false;
        }
        self.is_loading = true;
        self.last_error = None;
        true
    }

    pub fn mark_stale(&mut self) {
        self.stale = true;
    }

    pub fn finish_success(&mut self, repo_map: String) {
        self.cached = Some(repo_map);
        self.is_loading = false;
        self.stale = false;
        self.last_error = None;
    }

    pub fn finish_error(&mut self, error: String) {
        self.is_loading = false;
        self.last_error = Some(error);
    }
}

/// 워킹 디렉토리 내 `.rs` 파일들의 구조(struct, enum, fn)를 추출하여 요약본(Repo Map)을 생성합니다.
pub fn generate_repo_map(cwd: &str) -> Result<String> {
    let mut builder = WalkBuilder::new(cwd);
    builder.hidden(true).ignore(true).git_ignore(true);
    let walker = builder.build();

    let mut repo_map = String::new();
    repo_map.push_str("[Repo Map]\n");

    let language = tree_sitter_rust::LANGUAGE.into();
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .map_err(|e| anyhow::anyhow!("Parser setup failed: {}", e))?;

    // 구조체, 열거형, 함수 선언부를 찾는 Query
    let query_source = r#"
        (struct_item name: (type_identifier) @name) @struct
        (enum_item name: (type_identifier) @name) @enum
        (function_item name: (identifier) @name) @func
    "#;
    let query = Query::new(&language, query_source)
        .map_err(|e| anyhow::anyhow!("Query parse failed: {}", e))?;

    let mut cursor = QueryCursor::new();

    let mut file_count = 0;

    // [v0.1.0-beta.23] clippy::collapsible_if, manual_flatten 해소: flatten + let-chain 적용
    for entry in walker.flatten() {
        if entry.path().is_file()
            && let Some(ext) = entry.path().extension()
            && ext == "rs"
            && let Ok(content) = fs::read_to_string(entry.path())
        {
            let rel_path = entry.path().strip_prefix(cwd).unwrap_or(entry.path());
            let path_str = rel_path.to_string_lossy();

            if let Some(tree) = parser.parse(&content, None) {
                let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

                let mut items = Vec::new();
                while let Some(m) = matches.next() {
                    for cap in m.captures {
                        let node: tree_sitter::Node = cap.node;
                        if let Ok(text) = node.utf8_text(content.as_bytes()) {
                            let kind = node.kind();
                            if (kind == "type_identifier" || kind == "identifier")
                                && let Some(parent) = node.parent()
                            {
                                let parent_kind = parent.kind();
                                if parent_kind == "struct_item" {
                                    items.push(format!("struct {}", text));
                                } else if parent_kind == "enum_item" {
                                    items.push(format!("enum {}", text));
                                } else if parent_kind == "function_item" {
                                    items.push(format!("fn {}", text));
                                }
                            }
                        }
                    }
                }

                if !items.is_empty() {
                    repo_map.push_str(&format!("\nFile: {}\n", path_str));
                    items.dedup();
                    for item in items {
                        repo_map.push_str(&format!("  - {}\n", item));
                    }
                    file_count += 1;
                }

                // 8KB 크기 제한으로 자름
                if repo_map.len() > 8000 {
                    repo_map.push_str("\n... (Truncated due to size limits)");
                    break;
                }
            }
        }
    }

    if file_count == 0 {
        repo_map.push_str("\n(No Rust source files found or AST extraction failed)");
    }

    Ok(repo_map)
}

/// [v0.1.0-beta.27] 대형 저장소에서도 TUI를 막지 않기 위해 Repo Map 생성을 blocking worker로 분리한다.
pub async fn generate_repo_map_async(cwd: String) -> Result<String> {
    tokio::task::spawn_blocking(move || generate_repo_map(&cwd))
        .await
        .map_err(|e| anyhow::anyhow!("Repo Map worker join 실패: {}", e))?
}
