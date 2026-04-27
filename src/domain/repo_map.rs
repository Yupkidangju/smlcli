use anyhow::Result;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

/// [v2.3.0] Phase 31: RepoMap Disk Caching
#[derive(Serialize, Deserialize)]
struct RepoMapCache {
    hash: u64,
    content: String,
}

fn get_cache_path(cwd: &str) -> PathBuf {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    cwd.hash(&mut hasher);
    crate::infra::config_store::get_config_dir()
        .join(format!("repo_map_cache_{:x}.json", hasher.finish()))
}

/// 디렉토리 내 중요 파일 개수와 최종 수정 시간을 결합한 가벼운 해시를 생성.
fn cheap_hash(cwd: &str) -> u64 {
    let mut count: u64 = 0;
    let mut mtime_sum: u64 = 0;

    // 워크스페이스 내 모든 파일(재귀 x, src/ 만 간단히)을 스캔
    // WalkBuilder를 얕게(depth=3) 수행하여 빠르게 해시.
    let walker = WalkBuilder::new(cwd)
        .hidden(true)
        .ignore(true)
        .git_ignore(true)
        .max_depth(Some(3))
        .build();

    for entry in walker.into_iter().flatten() {
        if let Ok(meta) = entry.metadata()
            && meta.is_file()
        {
            count += 1;
            if let Ok(mtime) = meta.modified()
                && let Ok(duration) = mtime.duration_since(std::time::UNIX_EPOCH)
            {
                mtime_sum = mtime_sum.wrapping_add(duration.as_secs());
            }
        }
    }

    // mtime 합계 + 파일 개수를 섞어 해시
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    format!("{}:{}", count, mtime_sum).hash(&mut hasher);
    hasher.finish()
}

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
    // [v1.5.0] 스캔 성능 최적화: 깊이 10 제한 적용
    builder
        .hidden(true)
        .ignore(true)
        .git_ignore(true)
        .max_depth(Some(10));
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

    // [v2.1.0] Phase 29: 최근 수정된 파일 우선 처리 (Context Compression)
    let mut files = Vec::new();
    for entry in walker.flatten() {
        if entry.path().is_file()
            && let Some(ext) = entry.path().extension()
            && ext == "rs"
        {
            let mod_time = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(std::time::UNIX_EPOCH);
            files.push((entry.path().to_path_buf(), mod_time));
        }
    }

    // 최신순 정렬
    files.sort_by(|a, b| b.1.cmp(&a.1));

    for (path, _) in files {
        if let Ok(content) = fs::read_to_string(&path) {
            let rel_path = path.strip_prefix(cwd).unwrap_or(&path);
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

                // [v2.1.0] Phase 29: Max Token Guard (약 4000 토큰 = 16000 문자)
                if repo_map.len() > 16000 {
                    repo_map.push_str(
                        "\n... (Truncated: Context limit reached for low-priority files)",
                    );
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
    tokio::task::spawn_blocking(move || {
        let current_hash = cheap_hash(&cwd);
        let cache_path = get_cache_path(&cwd);

        // 캐시 로드 시도
        if let Ok(cache_str) = fs::read_to_string(&cache_path)
            && let Ok(cache) = serde_json::from_str::<RepoMapCache>(&cache_str)
            && cache.hash == current_hash
        {
            return Ok(cache.content);
        }

        let content = generate_repo_map(&cwd)?;

        // 백그라운드 캐시 저장
        let cache = RepoMapCache {
            hash: current_hash,
            content: content.clone(),
        };
        if let Ok(serialized) = serde_json::to_string(&cache) {
            let _ = fs::write(cache_path, serialized);
        }

        Ok(content)
    })
    .await
    .unwrap_or_else(|e| Err(anyhow::anyhow!("Task join error: {}", e)))
}
