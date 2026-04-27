# Changelog

모든 중요한 변경 사항은 이 문서에 기록됩니다.
이 프로젝트는 [Semantic Versioning](https://semver.org/) 기준을 따릅니다.

## [3.7.1] - 2026-04-25 (Security & Stability Hotfix)

### Fixed
- **[Finding 5] ReadFile 도구 인덱스 OOB 패닉 수정**: `file_ops.rs:read_file`에서 `start_line` 및 `end_line`이 `total_lines`를 초과하여 지정될 경우 패닉이 발생하던 취약점(Out-of-Bounds)을 `min/max` 경계값 교정 로직을 통해 안전하게 수정.
- **[Finding 6] Gemini Provider 도구 파싱 로직 구현**: `OpenRouter`/`OpenAI`와 동일하게 Gemini의 OpenAI 호환 엔드포인트 응답에서도 `tool_calls` 스트리밍(SSE) 및 동기 응답을 정밀 파싱하도록 구조체 수정 및 로직 이식.
- **[Finding 7] 설정 마법사 에러 은닉 수정**: `WizardSaveFinished` 이벤트를 도입하여 `save_config` 비동기 저장 중 디스크 꽉 참 또는 권한 오류 발생 시 마법사를 조용히 종료하지 않고 UI 에러 렌더링하도록 개선.
- **[Re-Audit Finding 1] GrepSearch 샌드박스 우회 방지**: `tools/grep.rs`에서 원시 경로를 그대로 검색하던 로직을 `file_ops::validate_sandbox()`를 거치도록 수정하여 `/etc` 등 외부 탐색 원천 차단.
- **[Re-Audit Finding 2] Approval 타임아웃 큐 고립 방지**: 대기 중인 승인 요청이 만료되었을 때, 큐(`queued_approvals`)에 남은 다음 요청을 정상적으로 팝업하도록 승격 로직 적용.
- **[Re-Audit Finding 3] CI 품질 게이트 통과**: `Cargo.toml` 버전을 3.7.1로 통일하고, `tests/audit_regression.rs`의 `collapsible_if` Clippy 경고를 수정하여 CI 실패 해결.
- **[Re-Audit Finding 4] 문서 최신화 (명세 동기화)**: `spec.md`, `audit_roadmap.md`, `designs.md` 내 버전과 존재하지 않는 `export-log` 서브커맨드 표기를 `sessions`로 정정.
- **[Re-Audit Finding 5] 세션 로그 파일명 충돌 방지**: `session_{timestamp}.jsonl` 이름 생성 시 UUID 6자리를 추가하여 동시성 상황에서 동일 파일에 덮어쓰거나 덧붙이는 현상 방지.
- **[Re-Audit Finding 6] 직접 셸 실행(!) 에러 재전송 방지**: 직접 셸 호출로 인한 오류 발생 시 `ToolError` 로직에서 `tool_call_id`가 없는 경우 LLM으로 불필요한 오류 피드백이 전송되지 않도록 방어.
- **[2차 Re-Audit] MCP pending 타임아웃 정리**: `McpClient`의 `pending_requests` 맵을 구조체 레벨로 승격하여, 타임아웃(10초) 시 해당 엔트리를 즉시 제거하고, EOF 시 잔존 요청 전체에 에러를 통지하여 메모리 누적 방지.
- **[2차 Re-Audit] GrepSearch·승인 큐 회귀 테스트 보강**: `test_grep_search_sandbox_bypass` (워크스페이스 외부 경로 차단 검증) 및 `test_approval_timeout_promotes_queue` (큐 승격 검증) 추가로 보안 수정 회귀 커버리지 확보.
- **[2차 Re-Audit] TUI 색상 정책 통합**: `questionnaire.rs`, `help_overlay.rs`에서 하드코딩된 `Color::Cyan`/`Color::Rgb` 등을 `state.palette()` 기반 테마 색상으로 교체하여 `/theme` 전환 즉시 반영.
- **[2차 Re-Audit] sanitize.rs 데드 코드 제거**: `providers/sanitize.rs`가 `mod.rs`에 미등록·미호출 상태로 방치되어 있어 제거. 동일 로직은 `registry.rs`에서 인라인으로 이미 구현 중.
- **[2차 Re-Audit] 문서 공백·EOF 정비**: `AGENTS.md`, `spec.md`, `CHANGELOG.md`, `DESIGN_DECISIONS.md`, `IMPLEMENTATION_SUMMARY.md`, `audit_roadmap.md`의 trailing whitespace 및 EOF blank line 정리로 `git diff --check` 완전 통과.

## [3.7.0] - 2026-04-24 (Phase 44-47: DeleteFile·TECH-DEBT·CI/CD·세션·플래닝 폼)

### Added
- **[Phase 44/D-1] DeleteFileTool 구현**: `tools/file_ops.rs`에 `Tool` trait 구현, `GLOBAL_REGISTRY` 정식 등록. validate_sandbox() 기반 워크스페이스 외부 접근 차단, FileWritePolicy 적용, is_destructive() = true (Git 체크포인트 트리거). 디렉토리 삭제 방지 및 파일 존재 여부 선행 검증 포함.
- **[Phase 44/D-2] TECH-DEBT 일괄 정리**: 모듈 레벨 `#[allow(dead_code)]` 7건, 파일 레벨 `#![allow(dead_code)]` 2건 제거. `ProviderRegistry` 1건만 cfg(test) 구조적 사유로 유지. `[ROADMAP/v3.0]` 주석을 실제 구현 상태로 갱신.
- **[Phase 45/CI-1] GitHub Actions CI 워크플로**: `.github/workflows/ci.yml` — fmt/clippy/test 품질 게이트 + cargo cache 최적화. `version-sync` job으로 버전 동기화 자동 검증.
- **[Phase 45/CI-2] GitHub Actions Release 워크플로**: `.github/workflows/release.yml` — 태그(v*) push 시 quality-gate → Linux musl / Windows msvc 크로스 빌드 → GitHub Releases 자동 업로드. `softprops/action-gh-release@v2` 사용.
- **[Phase 45/CI-3] 버전 동기화 검증 스크립트**: `scripts/check-version-sync.sh` — Cargo.toml ↔ CHANGELOG.md ↔ Git Tag 버전 일치 자동 검증.
- **[Phase 46/S-1] SessionMetadata & Workspace 격리**: `domain/session.rs`에 `SessionMetadata` 구조체 및 `SessionAction` 열거형 추가. `infra/session_log.rs`에 `SessionIndex` (sessions_index.json CRUD) 및 `new_workspace_session()` API 추가. `DomainState`에 `current_session_metadata` 필드.
- **[Phase 46/S-2] Auto-Titling 파이프라인**: 첫 UserMessage의 앞 50자를 세션 제목으로 자동 설정. 후속 메시지는 `updated_at` 타임스탬프만 갱신.
- **[Phase 46/S-3] `/resume`, `/session` 명령어**: 현재 워크스페이스의 세션 목록을 KeyValueTable로 렌더링. `/resume <번호>`로 세션 전환 (메시지 복원 + 로거 교체 + 인덱스 touch).
- **[Phase 46/S-4] `/new` 명령어**: 타임라인·세션 상태·스트림 어큐뮬레이터 초기화 후 새 워크스페이스 세션 할당. SlashMenu·CommandPalette·Help에 세션 명령어 3건 추가.
- **[Phase 47/Q-1] AskClarification 도구 스키마 정의 및 하네싱**: `domain/questionnaire.rs`에 `ClarificationQuestion`·`AskClarificationArgs`·`AskClarificationResult`·`QuestionnaireState` 도메인 타입. `tools/questionnaire.rs`에 `AskClarificationTool` (Tool trait, GLOBAL_REGISTRY 등록). PLAN 모드 시스템 프롬프트에 강제 사용 지침 주입.
- **[Phase 47/Q-2] Questionnaire TUI 렌더러**: `tui/widgets/questionnaire.rs` 화면 중앙 모달 오버레이. 객관식 커서(▸)·주관식 텍스트 입력·allow_custom("✏ 직접 입력")·진행률 표시·키보드 힌트.
- **[Phase 47/Q-3] State Machine 연동**: `ShowQuestionnaire`·`QuestionnaireCompleted` Action 추가. `tool_runtime` AskClarification 인터셉트 → TUI 모달. `handle_questionnaire_key()` 키 입력 핸들러 (↑↓ 탐색, Enter 선택, Esc 취소). 답변 완료 시 ToolResult 조립 → LLM 피드백.
- **[Phase 46] Workspace-scoped Session Management 로드맵**: `/resume`, `/new`, `/session` 명령어 및 TUI Session Picker 기획. 폴더 기반 세션 격리, Auto-Titling 파이프라인 명세 추가.
- **[Phase 47] Interactive Planning Questionnaire 로드맵**: PLAN 모드 전용 `AskClarification` 도구 하네싱 및 TUI Questionnaire 렌더러 기획. 구조화된 객관식/주관식 폼 기반 인터랙티브 플래닝 명세 추가.
- **[Task M-4] MCP E2E 테스트**: `scripts/mock_mcp_server.py` JSON-RPC 2.0 mock 서버. `test_mcp_e2e_initialize_and_list_tools`·`test_mcp_e2e_call_tool` 프로세스 spawn 왕복 검증. `test_mcp_permission_engine_always_ask`·`test_mcp_namespace_strip_roundtrip`·`test_mcp_config_add_remove_persistence` 설정 영속화 검증. `test_ask_clarification_tool_registered`·`test_questionnaire_state_submit_and_build`·`test_questionnaire_total_options` Phase 47 도구 등록 및 도메인 로직 검증. `PermissionResult`에 `Debug` derive 추가.

### Changed
- **PermissionEngine DeleteFile 통합**: `domain/permissions.rs`에서 `DeleteFile`을 쓰기 도구 목록에 추가. Workspace Trust Gate 및 경로 횡단 검사 적용.
- **guard 테스트 예외 목록 비움**: `audit_regression.rs`의 `known_unregistered`에서 `DeleteFile` 제거. 모든 write 도구가 레지스트리에 정식 등록됨 (path_write_count ≥ 3 자동 충족).
- **spec.md 의존성 그래프 확장**: Phase 46 (Session) → Phase 47 (Plan Form) 파이프라인 추가.
- **Clippy 경고 44건 → 0건 일소**: palette.rs 레거시 상수 13건 제거, questionnaire.rs clamp 패턴 적용, collapsible if 3건 자동 수정, 나머지 28건은 사유 주석 + `#[allow(dead_code)]`로 명시적 의도 표명. `TOOL_BADGE`, `INFO`, `SUCCESS` 등 미사용 레거시 상수는 삭제 사유·삭제 버전을 주석으로 기록.

## [3.3.9] - 2026-04-23 (12차 감사 대응: 핸들러 관통 테스트·문서 정합)

### Added
- **[MCP/MEDIUM-1] handle_action(McpToolsLoaded) 관통 테스트 2건** (92→94): `App::new()` + `handle_action()` 직접 호출로 실제 `mcp_tools_cache`/`mcp_tool_name_map` 상태 검증. (1) 정상 로드: cache 2건·map 2건·모든 schema name이 map에 존재. (2) 서버 간 충돌: suffix 부여 후 schema name 동기화·원본 key는 서버A·suffix key는 서버B로 라우팅.
- **`McpClient::dummy()` 테스트 전용 생성자**: `#[cfg(test)]` 더미 McpClient 생성. 실제 프로세스 없이 McpToolsLoaded 액션 구성 가능.

### Changed
- **[Doc/LOW-1] spec.md Step 3 갱신**: v3.3.8 skip schema `retain()` 제거 정책, v3.3.9 관통 테스트·dummy 생성자를 기술.
- **[Doc/LOW-1] IMPLEMENTATION_SUMMARY Task M-2 갱신**: v3.3.8 `retain()` + 타임라인 Notice, v3.3.9 관통 테스트·dummy 생성자를 반영.

## [3.3.8] - 2026-04-23 (11차 감사 대응: skip schema 제거·schema-map 동기화 테스트·타임라인 Notice)

### Fixed
- **[MCP/MEDIUM-1] skip 시 schema cache 잔류 방지**: 전역 충돌 해소 실패(suffix 한계 초과)로 skip된 도구의 schema를 `schemas.retain()`으로 제거. 이전에는 map insert만 건너뛰고 schema는 그대로 `mcp_tools_cache`에 push되어, LLM에 노출되지만 `mcp_tool_name_map`에 없어 라우팅 불가능한 도구가 생길 수 있었음.
- **[MCP/LOW-1] suffix skip 타임라인 Notice 추가**: MCP 도구 충돌 skip 경고를 `logs_buffer`뿐 아니라 타임라인 Notice 블록(`BlockSection::Markdown`, `BlockStatus::Error`)으로도 표시. 서버 로드 실패/서버명 충돌과 동일한 UX 일관성 확보.

### Added
- **회귀 테스트 1건 추가** (91→92): skip 시 schema가 `schemas`에서 `retain`으로 제거되고, 정상 도구만 남으며, cache의 모든 schema name이 global_map에 존재하는지 검증.

## [3.3.7] - 2026-04-23 (10차 감사 대응: schema-map 동기화·suffix skip·filter_map 전환·문서)

### Fixed
- **[MCP/HIGH-1] schema function.name ↔ map key 동기화**: `McpToolsLoaded` 핸들러에서 schemas를 먼저 push하던 로직 제거. 전역 충돌 해소 시 schema의 `function.name`을 변경된 key와 동일하게 수정한 뒤, 완료된 schemas를 cache에 push. 이전에는 map key만 suffix로 변경되어 LLM에 노출되는 이름과 라우팅 key가 불일치.
- **[MCP/MEDIUM-2] suffix 한계 초과 시 overwrite 대신 skip**: 서버 내부 suffix 루프(`filter_map` + `None` 반환)와 전역 merge 루프(`continue` + 경고 로그) 모두에서 suffix > 9999 시 해당 도구를 안전하게 건너뜀. 이전에는 break 후 기존 key로 insert되어 기존 mapping이 overwrite될 수 있었음.

### Changed
- **스키마 빌드 `map` → `filter_map` 전환**: suffix 한계 초과 도구를 `None`으로 필터하여 schemas에 포함되지 않도록 변경.
- **[Doc/LOW-1] spec.md·IMPLEMENTATION_SUMMARY 갱신**: schema-map 동기화 설계, schemas 지연 push 정책, suffix skip 정책을 반영.

### Added
- **회귀 테스트 1건 추가** (90→91): 전역 충돌 시 schema `function.name`이 변경된 map key와 일치하는지 검증.

## [3.3.6] - 2026-04-23 (9차 감사 대응: 서버 간 truncation 충돌·suffix 64자 방어·전역 merge 테스트·문서)

### Fixed
- **[MCP/HIGH-1] 서버 간 truncation 전역 충돌 방지**: `McpToolsLoaded` 핸들러에서 `extend()` → 전역 충돌 검사 + suffix 부여로 교체. 앞 27자가 동일한 서로 다른 서버가 같은 도구명을 노출할 때 `mcp_tool_name_map`이 overwrite되는 결함 수정.
- **[MCP/MEDIUM-2] suffix 64자 초과 방어**: 같은 서버 내 및 서버 간 suffix 루프에서 `_1000` 이상 접미사(5자+)로 64자 초과 시, base를 overflow만큼 줄여서 재구성하는 방어 로직 추가. 안전 한계 9999회.

### Added
- **회귀 테스트 2건 추가** (88→90): 서버 간 truncation 충돌 → suffix 해소 + 역매핑 정합 검증, suffix 64자 초과 시 base truncation 동작 검증.

### Changed
- **[Doc/LOW-1] spec.md Step 3·4 MCP truncation 설계 문서화**: `build_mcp_full_name()` 64자 제한, 전역 merge 충돌 정책, suffix 한계를 기술.
- **[Doc/LOW-1] IMPLEMENTATION_SUMMARY.md Task M-2 갱신**: v3.3.5 truncation + v3.3.6 전역 충돌 정책 반영.

## [3.3.5] - 2026-04-23 (8차 감사 대응: 64자 제한·동일명 로드 보장·타임라인 Notice)

### Fixed
- **[MCP/HIGH-1] OpenAI function name 64자 제한 준수**: `sanitize_tool_name_part()`는 문자 정규화만 수행하고, 새로운 `build_mcp_full_name()` 함수에서 서버/도구 파트를 각각 최대 27자로 truncate. 접두사 "mcp_"(4자) + "_"(1자) + 접미사 예비(4자) = 9자를 뺀 55자를 파트에 할당하여, 충돌 접미사 포함 시에도 64자 이내 보장.
- **[MCP/MEDIUM-2] 동일 서버명 중복 시 최소 하나 로드 보장**: 이전에는 `skipped_servers`에 원본명을 저장하여 `name = "fs"`가 두 번 있으면 **둘 다** skip됨. index 기반 `skipped_indices`로 교체하여 첫 번째는 로드하고 후순위만 skip.
- **[MCP/LOW-1] config.toml 충돌 경고 타임라인 Notice 표시**: 충돌 경고가 `logs_buffer`에만 들어가 일반 사용자가 놓치던 문제 해소. `McpLoadFailed`와 동일하게 타임라인 Notice 블록(`BlockSection::Markdown`)으로 표시.

### Added
- **회귀 테스트 2건 추가** (86→88): `build_mcp_full_name()` 64자 truncate 검증, 동일 서버명 중복 시 첫 번째 로드·후순위 skip 검증.

## [3.3.4] - 2026-04-23 (7차 감사 대응: 도구명 충돌·서버명 충돌·테스트 관통·문서 정합)

### Fixed
- **[MCP/HIGH-1] 같은 서버 내 도구명 정규화 충돌 방지**: 서버가 `foo.bar`와 `foo_bar` 같은 도구를 동시에 노출하면 둘 다 `mcp_srv_foo_bar`로 정규화되어 뒤 도구가 앞 도구를 조용히 overwrite하던 결함 수정. 충돌 시 접미사 번호(`_2`, `_3` 등)를 부여하여 모든 도구에 고유한 full_name 보장.
- **[MCP/MEDIUM-2] config.toml 서버명 정규화 충돌 방지**: `/mcp add`만 충돌 검사하고 config.toml 직접 편집 시에는 검사하지 않던 문제 수정. 앱 시작 시 `mcp_servers` 순회하여 정규화명이 충돌하는 서버를 감지, 후순위 서버 로드를 건너뛰고 로그 경고 출력.

### Changed
- **[MCP/MEDIUM-3] isError 테스트를 `parse_call_tool_result()` 직접 호출로 교체**: `call_tool()` 내부 파싱 로직을 `pub(crate) fn parse_call_tool_result()` 함수로 추출하여 테스트에서 직접 호출. 기존 테스트는 JSON 파싱을 테스트 측에서 재현하여 내부 로직 변경 시 회귀를 놓칠 수 있었음. 추가로 isError:true + content 없음 케이스, 도구명 충돌 접미사 해소 테스트도 추가 (85→86).
- **[Doc/LOW-1] spec.md Step 3 `/provider add` 문법**: `[auth_header_name]`까지 반영하여 성공 기준(2291행)과 구현 설명(2330행)의 인자 정합성 확보.

## [3.3.3] - 2026-04-23 (6차 감사 대응: MCP isError·정규화 충돌·회귀 테스트·문서 정합성)

### Fixed
- **[MCP/HIGH-1] CallToolResult isError 우선 검사**: MCP 공식 스키마에 따라 `isError`를 `content` 파싱 전에 검사. `isError:true + content` 동시 존재 시 content를 에러 메시지로 활용하여 `Err` 반환. 기존 코드는 content가 있으면 즉시 `Ok` 반환하여 도구 에러가 성공으로 전파되는 결함이 있었음.
- **[MCP/MEDIUM-2] 정규화 서버명 충돌 방지**: `/mcp add` 시 기존 서버 중 정규화명이 충돌하는 것(`foo.bar` vs `foo_bar` → 둘 다 `foo_bar`)이 있으면 등록 거부. `sanitize_tool_name_part`를 `pub(crate)`로 변경하여 `command_router.rs`에서도 접근 가능.

### Added
- **[MCP/MEDIUM-3] MCP 회귀 테스트 5건 추가** (80→85): `sanitize_tool_name_part` 정규화 기본 동작, 정규화 충돌 감지, 역매핑 테이블 구성 및 원본명 복원, `CallToolResult` isError+content 처리, 성공 경로 검증.

### Changed
- **[Doc/LOW-1] spec.md Phase 43 Step 4 MCP 라우팅 문서 갱신**: 이전 `starts_with` prefix match → 현재 `mcp_tool_name_map` 직접 조회 설명으로 교체.
- **[Doc/LOW-1] IMPLEMENTATION_SUMMARY.md Task M-2/M-3 갱신**: 역매핑 테이블, isError 우선 검사, 정규화 충돌 검사 반영.

## [3.3.2] - 2026-04-23 (5차 감사 대응: Sandbox·MCP Lifecycle·Routing·버전 동기화·문서 정합성)

### Fixed
- **[Sandbox/HIGH-1] ExecShell bubblewrap `bash -c` 수정**: `wrap_command_bwrap()`이 `bash <script_path>`로 실행하여 raw 명령 문자열을 파일 경로로 해석하던 결함 수정. `bash -c <cmd>` 형태로 교체하여 `touch foo && echo done` 같은 실제 셸 명령이 정상 실행됨. 변수명 섀도잉(`cmd` 파라미터 vs `cmd` 빌더)도 `bwrap_cmd`로 해소.
- **[MCP/HIGH-2] MCP initialize/list_tools 실패 시 child process kill**: `McpClient::spawn()` 내부에서 `initialize().await` 실패 시 `child_handle`로 프로세스를 명시적 kill 후 에러 반환. `mod.rs`에서 `list_tools()` 실패 시에도 `client.shutdown().await` 호출 추가. timeout/초기화 실패 MCP 서버 좀비 프로세스 완전 차단.
- **[MCP/HIGH-3] MCP tool name 정규화-역매핑 일관성 확보**: 스키마 노출 시 `sanitize_tool_name_part()`로 정규화한 이름을 사용하지만, `mcp_clients` 저장/라우팅은 원본명으로 하여 `Tool not found` 발생 가능했던 결함 해소. `mcp_tool_name_map: HashMap<sanitized_full_name, (sanitized_server, original_tool_name)>` 역매핑 테이블 도입. `mcp_clients` key를 정규화 서버명으로 저장. `tool_runtime.rs`의 prefix match를 직접 테이블 조회로 교체.
- **[Version/MEDIUM-4] Cargo.toml 버전 동기화**: `Cargo.toml` 버전이 `2.5.0`으로 동결되어 CHANGELOG `3.3.x`/문서와 불일치하던 문제 해소. `3.3.2`로 갱신하여 `cargo run -- --version` 및 `doctor` 출력 정합성 확보.
- **[Spec/MEDIUM-5] Provider 성공 기준 문법 정정**: `spec.md` Phase 41의 성공 기준이 `--name`, `--url`, `--format` 플래그 방식으로 기술되어 있었으나, 실제 구현은 positional 파서. `<id> <base_url> [dialect] [auth_type] [auth_header_name]` 형태로 수정.
- **[Settings/LOW-1] auto_commit 기본값 주석 교정**: `settings.rs` 및 `spec.md` Phase 40의 `GitIntegrationConfig.auto_commit` 필드 주석이 "기본: true"로 명시되어 있었으나 실제 기본값은 `false`. "기본: false, 명시적 opt-in 필요"로 수정.

## [3.3.1] - 2026-04-23 (4차 감사 대응: MCP 라이프사이클·UX·라우팅·Provider·Git 정합성)

### Fixed
- **[MCP/HIGH-1] 앱 종료 시 MCP 서버 자식 프로세스 명시적 kill**: `App::run()` 메인 루프 종료 직후 `mcp_clients.values().shutdown().await`를 호출. Event::Quit, /quit, Ctrl-C, SIGTERM 모든 종료 경로가 이 지점을 통과하므로 프로세스 누수를 완전 방지.
- **[MCP/MEDIUM-1] MCP 로드 실패 사용자 피드백**: `if let Ok && let Ok` 패턴으로 완전히 삼켜지던 MCP spawn/list_tools 실패를 `match`로 분리. 실패 시 `McpLoadFailed` 액션을 전송하여 타임라인에 에러 Notice 블록을 표시하고 logs_buffer에 상세 사유 기록. "도구가 안 보임" 상황에서 원인 파악 가능.
- **[MCP/MEDIUM-2] 네임스페이스 라우팅 접두사 충돌 방지**: 기존 `starts_with("mcp_{name}_")` 단순 매칭에서 longest prefix match(서버명 길이 내림차순 정렬)로 전환. 'fs'와 'fs_local' 같은 서버명에서 'fs'가 'fs_local_tool'에도 매칭되는 오라우팅 차단. OpenAI tool name 규격(^[a-zA-Z0-9_-]+$) 위반 문자를 `sanitize_tool_name_part()`로 정규화하여 Provider 거절 방지.
- **[Provider/MEDIUM-3] /provider add auth_header_name 지원**: 6번째 인자로 `auth_header_name`을 지정 가능. `CustomHeader` auth_type에서 `X-API-Key` 등 비표준 헤더를 CLI로 등록 가능. 기존 `Authorization` 하드코딩 제거.
- **[Git/MEDIUM-4] undo_last 문서-코드 정합성**: doc comment의 "해시 기반 추적"이라는 과장된 표현을 실제 구현("메시지 매칭 + 해시 consumed 추적")에 맞게 교정. 테스트명 `test_git_consecutive_undo_with_duplicate_messages` → `test_git_consecutive_undo_with_different_files`로 실제 내용과 일치하도록 변경.

### Added
- **[Domain]**: `Action::McpLoadFailed(String, String)` variant 신규 도입. MCP 로드 실패 피드백의 핵심 인프라.
- **[App]**: `App::sanitize_tool_name_part()` 정규화 헬퍼 추가. OpenAI tool name 규격 준수 보장.

### Quality
- `cargo clippy --all-targets -- -D warnings`: 경고 0건
- `cargo test`: 80건 전부 통과 (0 failed)
- `cargo fmt --check`: 통과

## [2.5.3] - 2026-04-23 (3차 감사 대응: MCP·Git 무결성 완성)

### Fixed
- **[MCP/HIGH-1] 스키마 OpenAI 형식 래핑**: MCP 서버가 반환하는 `{name, description, inputSchema}` 형태를 `{type: "function", function: {...}}` 형태로 래핑하여 OpenAI 호환 provider에 올바르게 전달. 기존 미래핑 스키마로 인한 LLM 도구 비인식 문제 해소. Anthropic `apply_dialect()` 변환도 정상 동작.
- **[MCP/HIGH-2] Child Process Lifecycle 관리**: `McpClient`에 `Arc<Mutex<Option<Child>>>` 핸들 보관. stderr drain task를 별도 `tokio::spawn`으로 소비하여 OS 파이프 버퍼 블로킹 방지. `shutdown()` 메서드로 앱 종료 시 MCP 서버 자식 프로세스 명시적 kill.
- **[Git/MEDIUM-2] auto_commit 빈 파일 목록 fallback 제거**: `auto_commit(files=[])` 호출 시 기존 `git add -u` fallback을 Err 반환으로 교체. WIP 혼입 위험 원천 차단.

### Added
- **[Testing]**: `test_git_auto_commit_empty_files_skip` (auto_commit 빈 파일 skip 검증), `test_mcp_schema_openai_format` (MCP→OpenAI 스키마 래핑 + Anthropic dialect 변환 검증). 80건 테스트 통과.

## [2.5.2] - 2026-04-23 (2차 감사 대응: 테스트·인증·문서 정합)

### Fixed
- **[Git/HIGH-1] Git E2E 테스트 추가**: `tempfile + git init` 기반 5건의 실제 git repo 테스트 신규 작성. (1) auto_commit 선택적 staging, (2) WIP 보호 (unrelated 파일 미포함), (3) undo_last 직접 revert, (4) 연속 undo 동일 메시지 중복 커밋 해시 기반 구분, (5) list_history prefix 필터. 78건 테스트 통과.
- **[Provider/HIGH-2] Custom Provider auth_type 어댑터 반영**: `OpenAICompatAdapter`에 `AuthStrategy` enum 도입 (Bearer/None/CustomHeader). `register_custom_providers()`에서 `config.auth_type`을 `AuthStrategy`로 변환하여 어댑터에 주입. Bearer 하드코딩 제거, `apply_auth()` 헬퍼로 모든 HTTP 요청에 동적 인증 적용.
- **[MCP/HIGH-3] MCP 완료 상태 정정**: `IMPLEMENTATION_SUMMARY.md` Phase 43을 '✅ 완료' → '⚠️ 인프라 구현 완료 / E2E 테스트 미비'로 수정. MCP E2E 테스트를 Task M-4로 Phase 44에 이관. `/mcp add/remove` 재시작 필요 사항 명시.
- **[Git/MEDIUM-1] undo 해시 기반 추적**: 메시지 문자열 대신 커밋 해시를 추적하여 동일 메시지의 중복 자동 커밋에서도 정확한 revert 대상 식별. `git log --pretty=format:%H|%P|%s`로 parent hash 활용.
- **[Config/MEDIUM-2] auto_commit 기본값 변경**: `git_integration.auto_commit` 기본값을 `true` → `false`로 변경. 사용자 워크트리에 직접 영향을 주는 기능은 명시적 opt-in이 더 안전한 UX.

### Added
- **[Domain]**: `AuthStrategy` enum 신규 도입 (`Bearer`, `None`, `CustomHeader`). Custom Provider 인증의 핵심 인프라.
- **[Testing]**: Git E2E 테스트 5건 추가 (78건 테스트 통과). `test_git_auto_commit_selective_staging`, `test_git_auto_commit_wip_protection`, `test_git_undo_last_direct_revert`, `test_git_consecutive_undo_with_duplicate_messages`, `test_git_list_history_prefix_filter`.

## [2.5.1] - 2026-04-23 (Git 무결성 감사 대응 패치)

### Fixed
- **[Git/HIGH-1] Auto-commit WIP 보호**: `ToolResult`에 `affected_paths: Vec<String>` 필드를 도입. `WriteFile`/`ReplaceFileContent` 성공 시 실제 변경 파일 경로를 기록하고, `auto_commit()` 호출 시 해당 파일만 선택적 `git add` 수행. `affected_paths`가 비어있으면 auto-commit skip하여 사용자 WIP를 보호. 기존 `git add -u`(전체 tracked 변경 stage) 문제 해소.
- **[Git/HIGH-2] 연속 /undo 지원**: `undo_last()`를 스택 방식으로 개선. HEAD가 `Revert "smlcli: ..."` 형태의 revert 커밋이면, `git log`에서 아직 revert되지 않은 가장 최근 smlcli 자동 커밋을 찾아 `git revert --no-edit <hash>`. 연속 `/undo` 실행 시 올바르게 여러 커밋 되돌리기 가능.
- **[Git/MEDIUM-1] Inspector Git 탭 prefix 필터**: `list_history()`에 `prefix` 인자 추가. `git log --grep=^{prefix}`로 smlcli 생성 커밋만 필터링하여 Inspector Git 탭에 사용자 커밋이 섞이는 문제 해소.
- **[Git/MEDIUM-2] git add 에러 전파**: 파일 목록 기반 `git add <file>` 실패 시 무시하지 않고 `Err`를 반환하여 잘못된 파일 참조를 즉시 노출.
- **[Git/MEDIUM-3] 문서-테스트 정합성**: `IMPLEMENTATION_SUMMARY.md` Phase 44 Task D-1의 `known_unregistered` 제거 표현을 현재 테스트 상태와 일치하도록 '구현 완료 후 제거 (현재 예외 등록 유지 중)'으로 명확화.
- **[Security/Permissions]**: (감사 MEDIUM-2) `PermissionEngine`에서 ExecShell `cwd` 인자가 절대경로일 때 workspace root 밖을 가리키면 선제적으로 `Deny` 반환. 기존 `../`/`~/` 패턴 차단에 더해 절대경로 이탈까지 이중 방어(Defense-in-Depth) 구성.
- **[Documentation]**: (감사 LOW-3) `spec.md` 내 인라인 `ROADMAP` 코드 블록 주석을 `Future Work` 문맥의 blockquote 형식으로 변환하여 릴리스 문서 정리.

### Added
- **[Domain]**: `ToolResult.affected_paths` 필드 신규 도입. Git auto-commit 파일 선택적 stage의 핵심 인프라.
- **[Testing]**: ExecShell cwd 절대경로 workspace 이탈 차단 회귀 테스트 추가 (`test_exec_shell_cwd_absolute_path_outside_workspace`). 73건 테스트 통과.
- **[Documentation]**: Phase 44 Task에 v2.5.0 감사 지적 MEDIUM-1(known_unregistered 예외 제거), LOW-1(FUTURE 주석 이관), LOW-2(dead_code 모듈별 정리) 추적 항목 명시적 등록.

## [2.5.0] - 2026-04-22 (Phase 35: System Hardening & Metadata)

### Added
- **[Integration/MCP]**: Phase 43 MCP(Model Context Protocol) 클라이언트 지원 통합 완료. `infra/mcp_client.rs`(232줄)에 mpsc 채널 + oneshot 기반 비동기 JSON-RPC 2.0 over stdio 클라이언트 구현. 시작 시 `config.toml`의 `[[mcp_servers]]` 설정에 따라 MCP 서버를 동적 스폰하고 `tools/list`로 스키마를 로드하여 `RuntimeState.mcp_tools_cache`에 캐싱. 네임스페이스 접두사(`mcp_{server}_{tool}`)로 내장 도구와 충돌 방지. `PermissionEngine`에서 `mcp_` 접두사 도구에 `Ask` 정책 강제 적용. `/mcp list/add/remove` 슬래시 명령어로 런타임 MCP 서버 설정 관리.
- **[System/Process]**: 비정상 종료로 인해 잔존하는 `ExecShell` 자식 프로세스들을 방지하기 위해 `sysinfo` 기반의 고아 프로세스(Orphan Process) 정리 기능(`src/infra/process_reaper.rs`) 추가 및 시작 시/`doctor --clean-orphans` 옵션 적용.
- **[DevOps/Build]**: `shadow-rs`를 도입하여 배포 바이너리 내부에 Git 커밋 해시 및 빌드 시간을 내장하고, `smlcli doctor` 실행 시 해당 메타데이터를 출력하도록 진단 리포트 확장.
- **[UX/Locale]**: `LANG` 환경변수 또는 사용자 설정(`use_ascii_borders`)에 따라 TUI 테두리를 유니코드 대신 ASCII(`+`, `-`, `|`)로 자동 전환하는 어댑티브 UI 렌더링 지원 추가.

### Fixed
- **[Reliability]**: `smlcli` 실행 중 병렬 도구(Parallel Tool) 실행 완료 순서가 뒤섞이더라도 `pending_tool_outcomes`에 큐잉 후 인덱스 기반으로 정렬하여 일관된 출력 순서를 보장(Ordered Aggregation).
- **[Performance]**: `SessionLogger` 복원 시 대용량 세션 파일 전체를 메모리에 올리지 않고 `BufReader`를 통해 행 단위(Line-by-line) 파싱을 수행하여 메모리 점유율 최적화 및 안정성 확보.
- **[Security]**: `FetchURL` 도구의 `ProviderOnly` 정책 처리를 `Ask`에서 `Deny`로 수정. `ProviderOnly`는 "프로바이더 API만 허용"이라는 보안 의미론을 가지므로, 임의 외부 URL 호출은 SSRF 방지를 위해 차단. FetchURL을 사용하려면 `AllowAll` 정책 필요.
- **[UX/Security]**: Trust Gate 팝업 활성 상태에서 `F2` 등 전역 단축키가 가드를 무시하고 실행되던 키 입력 폴스루(Fall-through) 버그 수정. `return` 추가로 완전 차단.
- **[Security/Agentic]**: Auto-Verify 3회 초과 실패(Abort) 후에도 LLM에 무조건 재전송되던 무한 루프 위험 수정. `AutoVerifyState::Aborted` variant 추가로 abort 결정을 **runtime state에 지속 저장**. 병렬 도구 간 abort 일관성 보장 — 로컬 변수가 아닌 state 기반 체크로 전환.
- **[Security/Process]**: Orphan reaper가 다른 정상 실행 중인 `smlcli` 인스턴스의 자식 프로세스를 오살하던 위험 수정. `SMLCLI_PID`에 기록된 부모 PID가 시스템에 아직 존재하는지 확인 후, 부모가 죽은 고아만 종료하도록 안전 가드 추가.
- **[Reliability/Concurrency]**: 병렬 도구 실행 시 단일 `CancellationToken`만 저장하여 마지막 도구만 취소 가능하거나, 먼저 끝난 도구가 토큰을 `None`으로 덮어쓰던 구조적 결함 수정. `HashMap<String, CancellationToken>`으로 전환하여 도구별 독립 취소 보장.
- **[UX/Display]**: `GrepSearch` 표시 이름이 구 스키마 `pattern` 필드를 읽어 빈 검색어를 표시하던 문제 수정. Phase 18 `query` 필드를 우선 참조하되 `pattern`도 fallback으로 지원.
- **[UX/Security]**: Wizard 초기 저장 기본 네트워크 정책이 `ProviderOnly`로 하드코딩되어 designs.md Safe Starter(`AllowAll`)와 불일치하던 문제 수정. 문서와 구현을 `AllowAll`로 통일.
- **[DevOps/Lint]**: 전역 `#![allow(dead_code)]` 제거 완료. 모든 dead_code 경고를 모듈 단위 `#[allow(dead_code)]`로 전환하여 감사 기준 "전역 allow 0건" 충족.
- **[DevOps/Lint]**: `registry.rs`의 `#[allow(unused_variables)]` 제거. cfg별 분리 메서드(`_kind`)로 리팩토링.
- **[DevOps/Format]**: `cargo fmt --check` 실패를 유발하던 `shell.rs` trailing whitespace 정리. CI fmt gate 통과 확보.
- **[DevOps/Cleanup]**: 루트 디렉터리의 실험 파일(`scratch.rs`, `test_border.rs`, `test_shadow.rs`, `patch_registry*.rs`) 삭제. 릴리즈 워크트리 정리.
- **[UX/Timeline]**: Queued approval이 마지막 타임라인 블록을 재사용하여 직전 Notice/ToolRun이 Approval로 변형되던 위험 수정. 새 `Approval` 블록을 명시적으로 push하는 방식으로 전환.
- **[UX/Timeline]**: 직접 셸 실행(`!`) Ask 경로에서 Approval 타임라인 카드를 생성하여, Inspector뿐 아니라 Timeline에서도 승인 대기 맥락 표시.
- **[Testing]**: Auto-Verify 병렬 abort 회귀 테스트 추가. "도구 A 3회 실패 abort → 도구 B 완료 → 재전송 차단" 시나리오를 직접 검증 (67건 통과).
- **[UX/Timeline]**: 최초 approval 생성 경로도 queued/direct-shell과 통일. ToolRun 블록을 pop하고 새 Approval 블록을 명시 push하여 블록 변형(mutation) 패턴을 전면 제거.
- **[DevOps/Lint]**: 모든 국소 `#[allow(dead_code)]`에 `TECH-DEBT` 마커 추가. v3.0 활성화 시 allow 제거 조건을 명시하여 릴리즈 전 정리 누락 방지.
- **[Documentation]**: `IMPLEMENTATION_SUMMARY.md`의 `/workspace add` 구현 상태를 `spec.md`/`designs.md`와 일치시켜 v3.0 미구현으로 표기.
- **[Security]**: 위험 패턴 검사를 도구별로 분리. `ExecShell`은 command에 인젝션 패턴 검사, `WriteFile`/`ReplaceFileContent`는 path에 횡단 검사만 적용. `FetchURL`/`GrepSearch` 등 읽기 전용 도구는 과차단 방지를 위해 미적용.
- **[Security]**: `ReadFileTool::check_permission()`에서 `validate_sandbox(path)`를 두 번 호출하고 두 번째에 `unwrap()`하던 패턴을 단일 `match`로 통합. 보안 경로에서 `unwrap()` 제거.
- **[Architecture]**: `dispatch_tool_call()`을 "permission 확인 후 블록 생성" 구조로 재구성. push-then-pop 패턴을 제거하여 Allow→ToolRun, Ask→Approval, Deny→ToolRun(Error) 각각 적절한 블록을 직접 생성.
- **[Reliability]**: `handle_tool_approval()`의 `pending_tool.take().unwrap()`을 `let Some ... else { return }` 패턴으로 교체. 상태 경합/만료 직후 입력에서 패닉 방지.
- **[Testing]**: Auto-Verify 병렬 abort 통합 이벤트 흐름 테스트 추가. 실제 이벤트 루프의 is_error→advance→pending 감소→flush→재전송 차단 전체 경로를 시뮬레이션.
- **[Testing]**: `handle_action(Action::ToolFinished/ToolError)` 직접 호출 통합 테스트에 `action_tx` 채널 수신 검증 추가. Aborted 상태에서 `SubmitChatRequest` 이벤트가 채널에 없음을 직접 확인.
- **[Testing]**: WriteFile/ReadFile workspace 밖 절대경로(`/etc/passwd`) 차단 end-to-end 테스트 추가. PermissionEngine→도구 레지스트리→validate_sandbox 위임 경로와 path 횡단(`../`) 검사를 모두 검증.
- **[Testing]**: write 도구 sandbox guard를 `GLOBAL_REGISTRY.tool_names()` 자동 대조 구조로 리팩토링. 하드코딩 목록 제거, 새 write 도구 등록 시 자동 탐지.
- **[Security]**: `ExecShell`의 `cwd` 인자에 경로 횡단(`../`, `~/`) 검사 추가. `resolve_shell_cwd()` 런타임 검증에 더해 권한 엔진에서도 선제 차단.
- **[Testing]**: ExecShell cwd 경로 횡단 차단 테스트 추가. `../`, `~/` 패턴 Deny 및 정상 cwd 허용을 검증.
- **[Documentation]**: `is_write_tool()`에 v3.0 미등록 도구(DeleteFile, GitCheckpoint) TODO 마커 추가. guard 테스트의 known_unregistered와 연동.
- **[Documentation]**: dirty flag 대상과 `is_write_tool()` 차이를 주석으로 명시. GitCheckpoint는 스냅샷 보존 도구로 RepoMap 갱신 대상에서 의도적 제외.
- **[Documentation]**: shell.rs의 `FUTURE` 주석을 `[ROADMAP/v3.0]`으로 변환. spec.md의 `TODO`도 동일 형식으로 정리.
- **[Documentation]**: v3.0 로드맵(Phase 40-45) 작성. Git 통합, Provider 확장, OS 샌드박스, MCP 클라이언트, DeleteFile/TECH-DEBT 정리, 빌드 파이프라인 6개 Phase의 상세 구현 가이드를 `spec.md` 및 `IMPLEMENTATION_SUMMARY.md`에 추가 (72건 통과).

## [2.3.0] - 2026-04-22 (Phase 31: The Final Polish & Resilience)

### Added
- **[Resilience]**: 설정 파일 마이그레이션 실패 시를 대비한 원자적 롤백 및 `.bak` 파일 백업 로직 추가.
- **[UX/Notification]**: 클립보드 복사 등 주요 UI 액션에 대한 시각적 피드백 제공을 위한 Toast 기반 알림 시스템 도입 (`expires_at` 기준 자동 만료).
- **[Doctor/Network]**: `smlcli doctor` 진단 내 API 네트워크 상태 체크 로직 추가 및 `tokio::time::timeout` 5초 래퍼 적용 (무한 대기 방지).
- **[Security/Config]**: `smlcli run` 등에서 LLM 도구 실행 시 허용할 환경 변수를 사용자가 지정할 수 있도록 `allowed_env_vars` 화이트리스트 확장 기능 제공.
- **[Performance]**: `RepoMap` 빌드 시 매 턴마다의 대규모 AST 파싱 비용 절감을 위해 `cheap_hash`(mtime+파일개수) 기반 `.gemini/tmp/repo_map_cache_{hash}.json` 디스크 캐싱 적용.

## [2.2.0] - 2026-04-22 (Phase 30: The Ultimate Hardening)

### Added
- **[Doctor]**: `smlcli doctor` 커맨드 추가: 설정 파일 유효성, API 키 유무, Git 설치 상태, TTY 호환성 등을 자동으로 진단하여 리포트 출력.
- **[UX]**: `arboard` 통합을 통해 TUI 내에서 `y` 단축키 입력 시 현재 포커스된 창(Inspector 또는 Timeline의 마지막 AI 응답)의 내용을 클립보드로 복사.
- **[Config]**: `version` 필드를 추가하고, 구버전 설정 감지 시 자동으로 `migrate()`를 실행하여 포맷을 승격.
- **[Stability]**: 시작 시 `~/.smlcli/*.tmp` 등의 이전 실행 찌꺼기 파일(Orphan files)을 자동으로 삭제하는 `cleanup_tmp_files` 로직 추가.
- **[Compatibility]**: Windows 환경 호환성 확보: 프로세스 그룹 종료 로직에 `#[cfg(windows)]` 매크로를 분기하여 `taskkill /F /T /PID` 사용.

## [1.7.0] - 2026-04-21

### Fixed (Phase 25: Ultimate Polish & Security Hardening)
- **UTF-8 안전성 보장 (UX/UI)**: TUI 렌더링 시 바이트 단위 문자열 자르기로 인해 한국어/이모지 등 멀티바이트 문자가 깨지거나 패닉이 발생하는 현상을 수정. `unicode-width` 크레이트와 시각적 너비 기반 자르기 로직 적용.
- **심볼릭 링크 샌드박스 탈옥 방지 (Security)**: `file_ops`에서 파일 입출력 시 `std::fs::canonicalize`를 통해 최종 절대 경로를 확인하고, Workspace 루트 밖으로 나가는 경로 우회 공격(Path Traversal/Symlink)을 원천 차단.
- **네트워크 타임아웃 및 재시도 (Robustness)**: LLM API 호출 과정에서 발생하는 무한 대기(Hanging) 현상과 429/5xx 일시적 에러를 방어하기 위해 60초 타임아웃 및 지수 백오프 기반의 재시도(Retry) 로직 추가.
- **ENOSPC 디스크 에러 그레이스풀 폴백 (Reliability)**: 설정 파일(`config.toml`)이나 세션 로그 저장 중 `std::io::ErrorKind::StorageFull` 발생 시 패닉을 방지하고 TUI에 경고 메시지만 노출하며 기존 데이터를 보존하도록 예외 처리.
- **터미널 프로세스 잔상 제거 (UX)**: `ExecShell` 도구를 사용해 `vim` 등의 서브 프로세스를 실행하고 TUI로 복귀할 때 화면 버퍼에 이전 프로세스 출력이 남는 고스팅(Ghosting) 방지를 위해 프로세스 종료 즉시 `terminal.clear()` 및 커서 재설정 호출.

## [1.2.0-rc.1] - 2026-04-21

### Fixed (Phase 19 & 20 Audit Remediation)
- **고도화된 쉘 인젝션 차단 (Phase 20)**: `$()`, ``` ` ```, `\n`, `\r` 등 복잡한 쉘 인젝션 체이닝 문법 차단 정규식 보완 및 `sudo`/`rm` 명령 시 `/etc`, `/var` 등 민감 디렉토리 접근을 원천 차단하는 PathGuard 로직 구현.
- **설정 파일 생성 원자성 보장 (Phase 20)**: API 마스터 키 및 `config.toml` 생성 시 OS 기본 umask가 적용되어 보안 노출되는 현상을 방지하기 위해 UNIX `OpenOptionsExt`를 통한 `chmod 600` 원자적(Atomic) 적용.
- **프로바이더 동적 갱신 (Phase 20)**: `ProviderRegistry`에 `OnceLock<RwLock>`을 도입하여 설정 마법사나 팝업에서 모델 및 API 키 변경 시 시스템 재시작 없이 즉시 반영(Hot-reload)되도록 구현.
- **스트리밍 파이프 블로킹 완화 (Phase 20)**: 쉘 대량 출력(수만 줄) 시 이벤트 루프가 멈추는 문제를 해결하기 위해 100줄 단위 `yield_now().await` 적용 및 OOM 방지를 위한 1MB 단위 라인 절단 로직 적용.
- **로그 가지치기 스크롤 동기화 (Phase 20)**: Inspector Logs 탭에서 10,000줄 초과 로그 Pruning 시 현재 스크롤 위치가 튀는 현상을 막기 위해, 잘려나간 줄 수만큼 `inspector_scroll` 오프셋을 역연산하는 Sticky Scroll 도입.
- **도구 실행 비동기 무상태화 (Phase 3)**: `ToolRuntime::execute_tool_async`에서 전역 상태를 직접 수정하던 안티패턴을 제거하고, 이벤트를 송신하여 이벤트 루프에서 상태를 갱신하도록 분리.
- **도구 강제 취소 (Phase 3)**: `tokio_util::sync::CancellationToken`을 도입하여 도구 실행 중 `Ctrl+C` 또는 `ESC` 키 입력 시 즉시 실행을 중단(Graceful Cancellation)하도록 지원.
- **TUI 로그 렌더링 최적화 (Phase 4)**: 20,000줄 이상의 방대한 로그 렌더링 시 발생하는 프레임 드랍(블로킹) 문제를 해결하기 위해, 전체 포매팅을 피하고 `inspector_scroll` 기반으로 화면에 보이는 높이만큼만 잘라서 렌더링하는 Window-based rendering 적용.
- **인스펙터 탭 스크롤 역전 현상 수정 (Phase 4)**: 윈도우 기반 렌더링 구현 중 모든 인스펙터 탭의 스크롤 방향이 역전(Up 누르면 내려감)되어 있던 것을 발견하여, bottom-up 오프셋을 top-based 오프셋으로 정상 변환하도록 일괄 수정.
- **설정 마법사 탭 포커스 순환 지원 (Phase 4)**: `is_wizard_open` 상태에서 `Tab` 및 `Shift+Tab` 입력 시, `Provider` ↔ `ApiKey` ↔ `Model` ↔ `Saving` 순서로 포커스가 순환되도록 구현.
- **프로세스 좀비화 원천 차단 (Phase 5)**: 쉘 명령어 스트리밍 실행 중 중단 시 `CancellationToken`이 자식 프로세스를 정리하지 못하던 오류를 수정. `Command::kill_on_drop(true)` 옵션을 부여하고 `child.kill().await`를 명시적으로 호출.
- **마법사 상태 동기화 누수 방지 (Phase 5)**: API Key 검증 실패 등 유효성 오류 발생 시, 입력 버퍼의 잔여 데이터(`api_key_input`)를 `clear()`하고 오류 발생 후 첫 키 입력 시 오류 메시지가 사라지도록 UI/UX 상태 동기화 개선.
- **Inspector 인덱싱 패닉 방지 (Phase 5)**: 빈 로그(`total_lines == 0`) 렌더링 시의 조기 반환 가드 추가 및, 로그 범위를 넘어서는 스크롤 입력 시 패닉을 방지하기 위해 스크롤 오프셋을 `clamp`로 제한.
- **쉘 인젝션 체이닝 차단 화이트리스트 도입 (Phase 5)**: `PermissionEngine::is_dangerous`에 정규식 `[;&|>]` 탐지를 추가하고, `ExecShell` 도구에서 `git`, `ls`, `grep` 등 안전한 명령어 외의 모든 실행은 `Ask`(승인 요청)로 강제하는 화이트리스트 병행 구조 도입.
- **세션 로그 플러시 안전성 확보 (Phase 5)**: 비정상 패닉 시 파일 I/O 데이터가 버퍼에 남아 증발하는 현상을 막기 위해, `SessionLogger`의 `append_message` 및 `append_message_async` 내부에 명시적인 `writer.flush()` 구문 추가.

### Docs
- **Phase 15: 2026 CLI UX 현대화 로드맵 문서화**: `spec.md`, `designs.md`, `IMPLEMENTATION_SUMMARY.md`, `DESIGN_DECISIONS.md`, `audit_roadmap.md`에 최신 CLI/TUI UX 패턴을 반영한 리팩토링 및 기능 강화 계획을 추가. 블록 기반 타임라인, 커맨드 팔레트, 입력 툴벨트, 반응형 상태바, 절제된 ASCII 애니메이션, 포커스/스크롤 상태 머신을 구현 전용 스펙으로 동결.
- **Windows Host Shell / Workspace Trust Gate 구현 계획 문서화**: `spec.md`, `designs.md`, `IMPLEMENTATION_SUMMARY.md`, `DESIGN_DECISIONS.md`에 Host shell vs Exec shell 분리, Windows PowerShell fallback 규칙, Workspace Trust Gate 3상태 모델, 구현 태스크 및 검증 기준을 상세 계획으로 추가.

### Added / Changed / Improved (Phase 18: Multi-Provider & Advanced Tools)
- **신규 Provider 어댑터**: OpenAI, Anthropic, xAI 모델 지원 추가. 기존 OpenRouter, Gemini와 더불어 `claude-opus-4-6`, `gpt-5.4` 등 2026.04 최신 모델 API 네이티브 지원.
- **Anthropic 네이티브 메시지 포맷**: `ToolDialect::AnthropicNative` 지원으로 Anthropic Messages API 스펙에 맞춘 Content Block 변환 및 SSE 파싱 구현.
- **FetchURL 도구**: 임의의 웹 URL 본문을 읽어와 `html2md`로 마크다운 변환. 500KB 메모리 상한 스트리밍 다운로드 구현. `ProviderOnly` 네트워크 정책에서는 SSRF 차단을 위해 실행 거부.
- **ListDir 도구 JSON 구조화**: 단순 텍스트 트리 대신 `{"name", "type", "size", "children"}`의 계층적 JSON 직렬화 구조 반환. `node_modules`, `target`, `.git` 자동 무시 적용.
- **GrepSearch 도구 정규식 지원**: `regex` 크레이트를 활용하여 `is_regex` 파라미터가 켜져 있을 시 실제 정규표현식 매칭 수행.
- **인증 보안 강화**: OpenRouter는 `/auth/key`를 통해, Anthropic 등은 `2xx` 응답을 통해 엄격한 API 인증 검증.

### Added / Changed / Improved (Phase 16)
- **Collapsible Diff UI**: 타임라인 내 긴 Diff(추가+삭제 10줄 초과)를 접기/펼치기 할 수 있는 토글 UI 추가.
- **Provider-Specific Tool Dialect**: Provider (Gemini/OpenRouter) 특성에 맞게 도구의 JSON Schema를 변환해주는 `ToolDialect` 추상화 및 런타임 적용. Gemini 모델 호출 시 `parameters.required`가 명시적으로 존재하도록 보정하여 파싱 에러 방지.
- **Error Unification**: `ConfigError`와 `ProviderError` 구조체로의 일관성 있는 오류 반환을 위해, 어댑터 및 I/O 설정 파일 저장/읽기 과정에서 발생하던 `anyhow::Result` 문자열 에러를 도메인 에러 타입으로 전면 전환.

### Fixed
- **Auto-Verify 상태 머신 실연결**: `src/app/mod.rs`의 `ToolFinished(is_error=true)` 및 `ToolError` 경로를 `AutoVerifyState::Healing`에 연결. 1~2회 실패는 힐링 프롬프트 주입 후 재전송, 3회 실패는 `Abort` Notice를 남기고 중단하도록 수정.
- **후속 재전송 도구 스키마 누락 수정**: `send_chat_message_internal()`이 초기 요청과 동일하게 Tool Registry 스키마를 포함하도록 `build_streaming_chat_request()` 공용 헬퍼를 도입.
- **Tree of Thoughts depth 실구현**: `TimelineBlock.depth` 필드 추가, `ToolRun`/`Approval`/`Auto-Verify Notice`를 `depth: 1`로 생성하고 `tui/layout.rs`에서 `└─` 인덴트 렌더링을 연결.
- **LLM 우선 도구 판정으로 조정**: `is_actionable_input()`은 참고 신호만 남기고, 비작업성 입력으로 분류되어도 모델이 구조화된 `tool_calls`를 반환하면 런타임이 선제 차단하지 않도록 완화.
- **Auto-Verify 오류 컨텍스트 확장**: 자가 치유 프롬프트가 240자 요약만 보던 문제를 수정. `ToolFinished` 실패 시 `stderr` 우선, `stdout` 보조의 확장 실패 컨텍스트를 앞/뒤 보존형으로 재전송하고, 사용자 Notice만 짧게 유지하도록 분리.
- **설정 파일 손상 가시화**: `load_config()` 오류가 앱 초기화에서 `Ok(None)`처럼 삼켜지던 문제를 제거하고, 손상된 `config.toml`은 Setup Wizard 첫 화면과 런타임 로그에 복구/삭제 가이드로 즉시 노출.
- **로그 버퍼 정합성 근거 명시**: `logs_buffer`는 비동기 태스크가 직접 건드리지 않고 이벤트 루프에서 직렬화된다는 계약을 Inspector Logs 렌더러 주석에 명시.
- **Linux Shell Sandbox 실체화**: `ExecShell`이 Linux에서 `bwrap` 기반 실제 격리 환경으로 실행되도록 변경. 호스트 루트는 읽기 전용, 요청 `cwd`만 `/workspace`에 쓰기 가능하게 bind mount.
- **Repo Map 유령 기능 연결**: `Repo Map`을 비동기 worker + 캐시 상태로 재구성하고, 준비된 캐시를 실제 채팅 요청의 system message로 주입하도록 연결.
- **HITL TTL 도입**: 승인 대기 요청이 5분 초과 시 자동 거부되도록 Tick 루프 만료 처리 추가.
- **타임라인 마우스 스크롤 실연결**: 마우스 휠이 더 이상 사용되지 않는 `timeline_scroll_offset`이 아니라 실제 렌더링에 사용되는 `timeline_scroll`을 조작하도록 수정. Timeline/Inspector 모두 3줄 단위 스크롤.
- **follow-tail 스크롤 복구**: 최하단으로 내려왔을 때 `timeline_follow_tail`이 다시 켜져, 새 응답/로그가 추가되면 화면이 자동으로 최신 내용에 따라붙도록 보정.
- **마우스 클릭 포커스 오판 수정**: column만 보던 패널 판정을 row+column 기반으로 변경하여 Top Bar/Composer 클릭 시 Timeline이 잘못 포커스를 먹지 않도록 수정.
- **작업 루트 자동 보정**: `target/release` 같은 빌드 산출물 디렉터리에서 앱을 실행해도 저장소 루트(`Cargo.toml`/`.git` 기준)를 작업 디렉터리로 재설정하여 `ReadFile`/`Stat`/`ListDir`가 잘못된 경로를 보지 않도록 수정.
- **Composer Toolbar 힌트 복원**: 하단 툴바에 `F2 Inspector` 힌트를 다시 표시.
- **Inspector 헤더 축약/2줄화**: Inspector 상단 탭 제목이 오른쪽에서 잘리던 문제를 줄이기 위해 고정 제목 + 적응형 1줄/2줄 탭 헤더로 변경.
- **과한 반전 하이라이트 완화**: 선택된 타임라인 블록 전체를 배경 반전하던 렌더링을 첫 줄만 약하게 강조하는 방식으로 변경하여 “전체 내용이 선택된 것처럼 보이는” 현상 완화.

### Tests
- **실패 경로 회귀 테스트 확충**: 손상된 `config.toml` 파싱 실패, 시작 시점 설정 오류 가이드, Auto-Verify tail-context 보존, 비-Git 디렉토리의 Git checkpoint no-op 경로를 테스트에 추가.
- **샌드박스/TTL/Repo Map 테스트 추가**: Linux 샌드박스의 `/etc` 쓰기 차단과 workspace 쓰기 허용, 승인 만료 자동 거부, Repo Map cache lifecycle 및 실제 요청 주입 경로를 회귀 테스트로 추가.
- **스크롤/포커스 회귀 테스트 보강**: 마우스 휠의 Timeline/Inspector 라우팅, follow-tail 복구, Composer 클릭 포커스 전환을 테스트로 고정.

### Changed/Improved (Phase 15-A: TimelineBlock 마이그레이션)
- **블록 기반 타임라인 도입**: 기존 `TimelineEntry` 기반 단일 텍스트 렌더링에서 `TimelineBlock`, `BlockSection`, `BlockStatus` 상태 머신 기반의 모듈식 아키텍처로 완전히 교체.
- **컴파일/의존성**: 고유 식별자 할당을 위한 `uuid v4` 의존성 추가.
- **렌더링 시스템 교체**: `src/tui/layout.rs` 및 `src/tui/widgets/inspector_tabs.rs`가 새로운 `TimelineBlock` 모델을 순회하여 렌더링하도록 재작성. (기존 `TimelineEntry` 및 `ToolStatus` 완전히 제거)

### Added/Partially Implemented (Phase 15: UX / State Machine & Inspector Workspace)
- **포커스 상태 머신 (`FocusedPane`)**: 타임라인, 인스펙터, 컴포저, 팔레트 등 포커스 기반 독립 스크롤링 및 키보드 이벤트 라우팅 도입 (`src/app/mod.rs`). 활성화된 패널은 Accent 색상 경계선으로 시각화. (Phase 15-B 일부 반영)
- **커맨드 팔레트 (`Command Palette`, Phase 15-C 완료)**: `Ctrl+K` 입력 시 팝업되는 빠른 실행 명령 레이어(`src/tui/layout.rs`) 및 `PaletteCategory` 도입 완료. 카테고리별 상태 연동.
- **다중 라인 프롬프트 지원**: `Shift+Enter` 를 통해 Composer 버퍼에 줄바꿈(`\n`)을 삽입할 수 있도록 멀티라인 입력 처리 추가 (`src/app/mod.rs`).
- **Composer Toolbar (`ComposerToolbarState`, Phase 15-D 완료)**: Composer 입력창 상단에 mode/path/policy/hint 등을 표시하는 칩(Chip) 동적 렌더링 도입. 멀티라인 입력 상태 실시간 표시.
- **Adaptive Header**: 윈도우 폭에 맞춰 상단 바 정보가 생략되는 반응형 정책(Adaptive Header) 적용 완료.
- **타임라인 커서 및 Inspector Preview (`Phase 15-E`)**: 타임라인 내 블록 이동(`Up`/`Down`)을 위한 커서를 추가하고, 선택된 블록의 전체 마크다운 및 코드 펜스를 Inspector의 `Preview` 탭에서 확인할 수 있도록 재구성 (`src/tui/widgets/inspector_tabs.rs`).
- **Inspector Diff 탭 (`Phase 15-E`)**: 파일 수정 등 승인 대기 중인 변경사항(Diff)을 직관적으로 확인할 수 있도록 `Diff` 탭의 렌더링 구현 추가.
- **Motion Polish 애니메이션 개선 (`Phase 15-F 완료`)**: `MotionProfile`을 전면 도입하여 도구 실행(`Running`) 스피너 렌더링 및 `NeedsApproval` 상태 진입 시 주기적 Pulse 깜빡임 애니메이션 적용.

## [v0.1.0-beta.23] - 2026-04-18

### Added (Phase 13: Agentic Autonomy 개편)
- **자율 에이전트 아키텍처 (Agentic Autonomy) 도입**: 다형성 기반의 `ToolRegistry` 패턴을 도입하여 기존 하드코딩된 `match` 도구 실행 로직을 동적으로 전환 및 통합 관리 (`src/tools/registry.rs`).
- **도구 스키마 동적 주입**: AI 모델에게 도구 스키마(Tool Schemas)를 초기 요청뿐 아니라 후속 재전송(`send_chat_message_internal`)에서도 동적으로 주입하도록 `chat_runtime` 구조 개선.
- **Git 자동 체크포인트 (Automated Git Checkpoint)**: `src/tools/git_checkpoint.rs` 모듈을 추가. `create_checkpoint()`는 강제 커밋 없이 워킹 트리 clean 여부만 검사하여 `Result<bool>`을 반환. WIP 존재 시 롤백을 건너뛰어 사용자 데이터를 보호. `rollback_checkpoint()`는 `git reset --hard HEAD`만 사용하며 `git clean -fd`는 완전 제거.
- **Tree-sitter Repo Map**: Tree-sitter 기반 `repo_map.rs`를 구현하여 워킹 디렉토리 내 `.rs` 파일들의 AST 구조(struct, enum, fn)를 추출, 8KB 크기 제한 하에 요약하여 프롬프트 상단으로 자동 주입하는 컨텍스트 확장 기능 추가.
- **Auto-Verify & Self-Healing**: 도구 실행 실패 시 `AutoVerifyState` (Idle, Healing { retries }) 스테이트 머신을 사용. `ToolFinished(is_error=true)`와 `ToolError` 양쪽 경로 모두에서 힐링 프롬프트를 주입하고 LLM에 재전송. 최대 3회 재시도 후 자동 포기(Abort).
- **Tree of Thoughts TUI 렌더링**: 여러 도구가 연쇄적으로 실행되거나 자가 복구가 진행될 때 시각적으로 인덴트(`└─`)를 부여하여 타임라인에서 계층적으로 표현하도록 기능 추가 (`src/tui/layout.rs`).

### Changed/Improved
- **ToolCall 리팩토링**: 기존 Enum 기반의 `ToolCall` 구조를 직렬화(Serialization) 없는 단일 구조체(Struct)와 `Value` 파라미터 조합으로 교체하여 유연성 극대화.
- **권한 검사 책임 이관**: `PermissionEngine` 권한 검사 체계를 `Tool` trait로 위임하여 각 도구가 스스로의 위험도(`is_destructive`)와 검사 로직을 정의하도록 개선.
- **Repo Map 헤더 포맷**: `"Repository Structure Map (AST based):"` → `"[Repo Map]"` 헤더로 변경하여 감사 기준과 동기화.

### Security
- **SafeOnly allowlist 바이패스 수정**: 직접 셸 실행(`!`) 경로에서 `safe_to_auto_run: true`가 하드코딩되어 SafeOnly 모드의 allowlist를 우회하던 취약점을 `safe_to_auto_run: false`로 수정.
- **Auto-Verify Abort 재전송 중단**: 최대 재시도(3회) 도달 시 Abort 메시지만 남기고 `send_chat_message_internal()` 호출을 중단하여 LLM 재전송 무한 루프를 방지. `ToolFinished`와 `ToolError` 양쪽 경로에 동일 적용.

### Added (Phase 14: TUI UX/UI 고도화)
- **14-A 멀티라인 텍스트 렌더링**: `render_multiline_text()` 공용 헬퍼 도입. `Line::from(msg)` 단일 라인 렌더링 → `\n` 기준 분리 멀티라인 렌더링으로 전환. `/help` 명령어 출력은 `HelpTable` variant로 구조화하고, 좁은 터미널 폭에서도 명령어 컬럼이 밀리지 않도록 수동 단어 wrap 알고리즘 적용.
- **14-B 스크롤 분리 + Auto-Follow + 마우스**: `inspector_scroll`/`timeline_follow_tail` 필드 분리. `terminal.rs`에 `EnableMouseCapture` 추가. `event_loop.rs`에서 `CrosstermEvent::Mouse` 수신. 마우스 휠을 포인터 X좌표 기반 타임라인/인스펙터 독립 라우팅. Home/End 키 지원. Auto-follow: bottom-up 오프셋 변환을 통해 렌더링에 완벽하게 연동.
- **14-C 키바인딩 재정렬**: `Ctrl+I`(터미널에서 Tab과 동일한 0x09) 바인딩 제거. 인스펙터 토글을 `F2`로 변경. 상태 바 안내 문구를 실제 키맵과 동기화.
- **14-D 반응형 레이아웃**: 상단 바를 `Layout::horizontal`로 좌우 강제 분할하여 터미널 폭 감소 시 핵심 정보(mode, ctx%)가 잘리지 않고 우측 정렬을 유지하도록 구조 개선. `provider/model/cwd` 중략 헬퍼 `truncate_middle()` 적용. 인스펙터 폭 Min/Max 클램프(32~48cols) 및 탭 라벨 축약 적용.

## [0.1.0-beta.23] - 2026-04-17

### Added (Phase 12: Native Structured Tool Call Integration 완료)
- **OpenAI 호환 도구 호출 완전 이관**: 기존의 마크다운 정규식 캡처(Fenced JSON) 방식을 폐기하고, 모델이 공식적으로 지원하는 구조화된 도구(Tool Call) API로 안전하게 이관 완료.
- **스트리밍 델타 버퍼링**: `OpenRouterAdapter::chat_stream`에서 SSE로 수신되는 `delta.tool_calls`의 파편화된 조각들을 JSON 및 객체 형태로 조립하는 스트리밍 로직 구현 완료.
- **도구 호출 ID(`tool_call_id`) 추적 매핑**: LLM의 도구 호출에 대응되는 `tool_call_id`를 유지하고 결과(`ToolResult`) 반환 시 매칭하여 정확히 전달할 수 있도록 도구 라이프사이클 및 파이프라인 개편.

### Changed
- `providers/types.rs`: `ChatMessage` 및 `ChatRequest` 등 도메인 모델에 `tool_calls`, `tool_call_id` 필드를 추가하고 `content`를 `Option<String>`으로 안전하게 래핑.
- `app/command_router.rs`: `ChatMessage` 및 `ChatRequest`의 모든 초기화 지점에 누락된 필드 보충 및 타입 변경에 따른 컴파일 에러 완전 해결.
- `app/chat_runtime.rs`: System 메시지 주입 로직의 타입 불일치와 구조체 누락 필드 전면 수정.
- `tools/*.rs` (실행기 모듈): `file_ops`, `grep`, `shell`, `sys_ops`, `executor` 내 `ToolResult` 반환 시 `tool_call_id` 필드를 추가하여 타입 무결성 확보.
- `tui/layout.rs`: 변경된 모델 구조를 지원하도록 메시지 UI 렌더링 로직 수정 (`content.as_deref()`).

### Quality
- `tests/audit_regression.rs`: 이전의 Fenced JSON 파싱 테스트를 Native Tool Call 구조 전송 검증 테스트로 일괄 갱신.
- `cargo check` 및 `cargo test`: 타입 안전성 확보 및 회귀 테스트 42건 무결성 통과 (0 failed).

## [0.1.0-beta.22] - 2026-04-17

### Fixed (하네스 구조/보안/UX 감사 대응 — HIGH 5건, MEDIUM 3건, LOW 2건)
- **[H-1] 도구 호출 격리 계층 강화**: bare JSON(fenced가 아닌) 응답을 도구로 인식하지 않도록 사전 차단. `"tool"` 키 존재 여부 1차 필터 + ToolCall serde 역직렬화 2차 필터 + ExecShell 빈 명령 3차 필터 도입. 모델 인삿말에 도구 JSON이 섞여 자동 실행되는 결함 해소.
- **[H-2] 빈 ExecShell 차단**: `command.trim().is_empty()` 검사를 permission 단계 이전에 추가. `is_safe_command()`에서 빈 토큰이 `true`를 반환하던 결함도 수정. SafeOnly/Ask 정책 모두에서 빈 명령 원천 차단.
- **[H-3] 전체 UI Wrap + 스크롤**: 타임라인, 컴포저, 설정 팝업, 위자드 4곳에 `Wrap { trim: false }` 적용. `UiState::timeline_scroll` 필드 추가로 세로 스크롤 오프셋 지원. 긴 응답/도움말이 가로로 넘치지 않음.
- **[H-4] 첫 턴 자연어 가드**: 시스템 프롬프트에 "첫 응답은 반드시 자연어", "비작업성 입력에는 도구 미사용" 정책을 명시. 도구 카탈로그를 간결화하고 예시 JSON을 제거하여 스키마 노출 원인 제거.
- **[H-5] bare JSON 렌더링 필터링**: `filter_tool_json()`에 bare JSON 감지 로직 추가. `"tool"` 키가 있는 bare JSON은 사용자 친화적 요약으로 대체. 스키마가 사용자에게 그대로 노출되는 결함 해소.
- **[M-1] PLAN/RUN 모드 시스템 프롬프트 주입**: 채팅 요청 시 현재 모드에 따라 LLM에 행동 계약을 주입. PLAN에서는 분석/설명 위주, RUN에서는 WriteFile/ReplaceFileContent 우선 사용을 지시.
- **[M-2] 작업 계약 명확화**: RUN 모드에서 코드 작성 요청 시 파일 도구를 우선 사용하도록 계약화하여, "인라인 답변 → 나중에 WriteFile" 불일치 해소.
- **[M-3] 타임라인 스크롤 키 바인딩**: PageUp/PageDown 키로 `timeline_scroll` 조작. 위자드/Fuzzy/설정 팝업이 열려 있지 않을 때만 동작. wrap 적용 후 긴 응답을 탐색할 수 있는 입력 경로 확보.
- **[L-1] 승인 카드 전체 경로 표시**: 도구 이름을 `Debug` 포맷의 30자 절단에서 도구별 의미 있는 이름(전체 경로 포함, 최대 120자)으로 개선. 승인 detail에 명령어/경로/동작을 축약 없이 표시.
- **[L-2] 회귀 테스트 갱신**: bare JSON 필터링 검증을 "스키마 노출 차단" 관점으로 갱신. 33/33 통과.

### Changed
- `domain/session.rs`: 시스템 프롬프트 전면 개편 — 첫 턴 자연어 가드, 도구 카탈로그 간결화, 예시 JSON 제거
- `domain/permissions.rs`: ExecShell 빈 명령 하드 가드 추가, `is_safe_command()` 빈 토큰 `false` 반환
- `app/tool_runtime.rs`: 3단계 도구 호출 필터링 계층 구현, `format_tool_name()`/`format_tool_detail()` 추가
- `app/chat_runtime.rs`: `dispatch_chat_request()`에 PLAN/RUN 모드별 시스템 프롬프트 주입
- `app/mod.rs`: PageUp/PageDown 키 바인딩 → `timeline_scroll` 조작
- `app/state.rs`: `UiState::timeline_scroll: u16` 필드 추가
- `tui/layout.rs`: 타임라인 `Wrap + scroll()`, 컴포저 `Wrap`, bare JSON 렌더링 필터 추가
- `tui/widgets/config_dashboard.rs`: Paragraph에 `Wrap` 적용
- `tui/widgets/setting_wizard.rs`: Paragraph에 `Wrap` 적용
- `tests/audit_regression.rs`: bare JSON 필터링 테스트를 "스키마 노출 차단" 검증으로 갱신

### Quality
- **[H-6→삭제] 첫 턴 하드가드 삭제**: `assistant_turn_count <= 1` 전역 차단을 제거. UX 파괴 원인이었음.
- **[H-7] 시스템 프롬프트 계약 통일**: "첫 응답 도구 금지" 규칙을 삭제하고, "작업 요청이면 첫 프롬프트라도 즉시 도구 사용" / "비작업성 입력이면 자연어 전용"으로 통일. Run 모드 계약과의 모순 해소.
- **[M-4] mixed bare JSON 렌더링 필터**: `filter_tool_json()`을 바이트 스캔 방식으로 개편. 응답 내 어디에든 `{"tool":...}` 패턴이 있으면 brace 매칭(`find_json_end`)으로 JSON 범위를 특정하여 사용자 친화적 요약으로 대체.
- **[M-5] 모드 지시 누적 방지 (dedupe)**: `chat_runtime.rs`에서 `"[Mode:"` 접두사로 기존 메시지를 찾아 교체.
- **[M-6] 승인 Inspector `{:?}` → `format_tool_name/detail`**: `crate::app::App::format_tool_name()` + `format_tool_detail()` 사용 + `Wrap` + `scroll()` 적용. 긴 경로/diff/replacement 탐색 가능.
- **[M-7] 통합 회귀 테스트**: `process_tool_calls_from_response()` 직접 호출로 bare JSON 차단 / fenced JSON 디스패치 / 첫 턴 동작 일관성 검증. 시스템 프롬프트 계약 검증 테스트 추가.
- **[Open Q] 기본 모드 Run 전환**: `session.rs` 기본 모드를 `AppMode::Run`으로 변경 (코딩 에이전트 기본 동작).
- `domain/session.rs`: 시스템 프롬프트 재설계 — 작업/비작업 분기, 기본 모드 Run
- `app/tool_runtime.rs`: 첫 턴 하드가드 삭제, `format_tool_name/detail` pub(crate)
- `app/state.rs`: `AppState::new_for_test()` 동기 생성자 추가
- `tui/layout.rs`: 승인 Inspector `format_tool_name/detail` + Wrap + scroll, `filter_tool_json/find_json_end` pub(crate)
- **[H-8] 비작업성 입력 런타임 도구 억제**: `is_actionable_input()` 휴리스틱으로 사용자 입력 의도를 분류하고, 인삿말/잡담 시 `user_intent_actionable=false`로 설정하여 `process_tool_calls_from_response()`에서 도구 디스패치를 코드로 차단. 프롬프트 순응에만 의존하지 않음.
- **[M-8] Inspector 서브탭 scroll 적용**: Logs/Search/Recent 3개 탭에 `.scroll((timeline_scroll, 0))` 적용. PageUp/PageDown으로 Inspector 내용도 탐색 가능.
- **[L-1] assistant_turn_count 데드 코드 정리**: 차단 로직이 제거되어 의미 없는 상태 필드와 증가 코드를 삭제. 오해를 유발하는 주석 정리.
- **[Feature] Shift+Tab 모드 전환 추가**: `Tab` 키뿐만 아니라 `Shift+Tab`(`BackTab`) 단축키로도 PLAN/RUN 모드를 즉시 전환할 수 있도록 키 바인딩 추가.
- **[Feature] 프롬프트 상단 커맨드 상태바(Status Bar) 추가**: 프롬프트 입력창 상단에 1줄짜리 커맨드 안내 상태창을 신설. 현재 모드(`[PLAN]` / `[RUN]`) 및 각종 주요 단축키 안내를 항상 표시하여 터미널 인터페이스의 사용성과 직관성 대폭 향상.
- **[Bug Fix] 도구 호출 JSON 파싱 실패 무시 현상**: LLM이 선택적 boolean 필드(`overwrite`, `safe_to_auto_run`, `case_insensitive`)를 누락할 경우 `serde_json` 파싱이 실패하여 도구 실행이 중단되는 버그 수정 (`#[serde(default)]` 추가). 파싱 실패 시 LLM과 사용자 모두에게 명확한 오류 로그와 피드백 전달하도록 예외 처리(`match` 적용).
- **[Bug Fix] `/help` 다중 줄 렌더링 깨짐**: `SystemNotice` 렌더링 시 개행문자(`\n`)가 포함된 문장을 개별 `Line`으로 올바르게 분리하여 출력하도록 변경.
- `app/chat_runtime.rs`: `is_actionable_input()` 휴리스틱 함수 추가, 입력 시점 의도 분류
- `app/tool_runtime.rs`: `user_intent_actionable == false` 시 도구 디스패치 차단 가드. ToolCall 파싱 에러 런타임 피드백 처리.
- `app/state.rs`: `assistant_turn_count` → `user_intent_actionable` 교체
- `app/mod.rs`: `assistant_turn_count` 증가 코드 제거. `Tab` / `BackTab` 모드 토글 로직 추가.
- `tui/layout.rs`: `SystemNotice` 다중 줄(`msg.lines()`) 분리 지원 및 `draw_command_status_bar` 함수 신설.
- `domain/tool_result.rs`: 선택적 bool 파라미터 `#[serde(default)]` 어노테이션 추가
- `tui/widgets/inspector_tabs.rs`: Logs/Search/Recent Wrap + scroll 추가
- `tests/audit_regression.rs`: 의도 분류 테스트 + 통합 테스트 보강 (41→42건)

### Quality
- `cargo test`: 42건 전부 통과 (0 failed)
- `cargo clippy --all-targets --all-features -- -D warnings`: 경고 0건 (릴리스 게이트 통과)

## [0.1.0-beta.21] - 2026-04-17

### Fixed (재감사 대응 — HIGH 1건, MEDIUM 2건, LOW 2건)
- **[H-1] 테마 전환 렌더링 실연결**: `/theme` 명령어가 설정값만 변경하고 화면에 반영되지 않던 결함 해소. `AppState::palette()` 헬퍼를 도입하고, `layout.rs`, `inspector_tabs.rs`, `config_dashboard.rs`, `setting_wizard.rs` 4개 렌더링 파일의 모든 정적 `pal::CONSTANT` 참조(50+곳)를 `state.palette().field` 동적 참조로 전환. `/theme` 실행 즉시 화면 전체 색상이 전환됨.
- **[M-1] 에러 타입 구조화 (ProviderError/ToolError)**: `Action` enum의 `ChatResponseErr(String)`, `ToolError(String)`, `ModelsFetched(Err(String))`, `CredentialValidated(Err(String))` 4개 경로를 `ProviderError`/`ToolError` 도메인 타입으로 전환. 에러 종류별 패턴매칭과 UI 메시지 분리가 가능해짐.
- **[M-2] spec.md Action 계약 동기화**: spec.md의 Action enum 정의를 v0.1.0-beta.21 구현(Box 래핑, ProviderError/ToolError 타입)과 정확히 일치시킴.
- **[L-1] /help 도움말 갱신**: `/theme` 커맨드가 슬래시 자동완성에는 포함되어 있었으나 `/help` 출력에는 누락되어 있던 불일치 해소.
- **[L-2] config_store.rs 에러 분류 정확화**: `read_to_string` 실패를 `ConfigError::NotFound`로 일괄 매핑하던 코드를 `ErrorKind` 분기 처리로 수정 — 권한 거부·기타 I/O 오류와 파일 미존재를 정확히 구분.
- **[L-3] README 기능 목록 갱신**: 5개 언어 섹션에 `/theme` 테마 전환, Inspector Search 탭, SSE 스트리밍, JSONL 세션 로그 기능 추가.

### Changed
- `domain/error.rs`: `AppError`, `ConfigError`, `ToolError`, `ProviderError` 4개 타입에 `Clone` derive 추가 (Action Clone 호환). `Io`/`Unknown` variant를 `String` 기반으로 단순화.
- `app/action.rs`: 에러 경로 4곳을 도메인 타입(`ProviderError`, `ToolError`)으로 전환
- `app/mod.rs`: `handle_models_fetched`, `handle_credential_validated` 시그니처를 `ProviderError`로 갱신
- `app/chat_runtime.rs`: `ChatResponseErr` 발송 2곳을 `ProviderError::NetworkFailure`로 구조화
- `app/tool_runtime.rs`: `ToolError` 발송 1곳을 `ToolError::ExecutionFailure`로 구조화
- `app/wizard_controller.rs`: `ModelsFetched`/`CredentialValidated` 발송 5곳을 `ProviderError` 기반으로 전환
- `app/command_router.rs`: `ModelsFetched` 발송 2곳을 `ProviderError` 기반으로 전환, `/help` 텍스트에 `/theme` 추가
- `tui/layout.rs`: 모든 색상을 `state.palette()` 동적 참조로 전환 (50+곳)
- `tui/widgets/inspector_tabs.rs`: 모든 색상을 동적 참조로 전환
- `tui/widgets/config_dashboard.rs`: `Color::Yellow` → `palette().warning` 전환
- `tui/widgets/setting_wizard.rs`: `Color::Cyan` → `palette().info` 전환
- `app/state.rs`: `AppState::palette()` 헬퍼 메서드 추가

### Quality
- `cargo test`: 28건 전부 통과 (0 failed)
- `cargo clippy --all-targets --all-features -- -D warnings`: 경고 0건 (릴리스 게이트 통과)

## [0.1.0-beta.20] - 2026-04-17

### Fixed (감사 리포트 대응 — HIGH 2건, MEDIUM 3건)
- **[H-1] 세션 로거 회귀 복구**: `SessionLogger::from_file()`, `restore_messages()`, 동기 `append_message()` API 복원. 비동기 전환 과정에서 삭제된 세션 복원/테스트용 동기 API를 재공급하여 회귀 테스트 28건 전부 통과.
- **[H-2] 세션 영속성 실행 불가 수정**: `chat_runtime.rs` 및 `mod.rs`에서 `logger.append_message()`가 async fn인 상태에서 await/spawn 없이 버려지던 Future를 동기 API로 교체. 로그가 실제로 디스크에 기록되도록 수정.
- **[M-1] Inspector Search 탭 실제 구현**: "v0.2 예정" 안내만 표시하던 Search 탭을 타임라인 전체 대소문자 무시 검색 엔진으로 교체. Composer 입력을 검색어로 사용하며 최대 50건 표시.
- **[M-2] 테마 시스템 구현**: `PersistedSettings`에 `theme` 필드 추가, `palette.rs`에 `Palette` 구조체와 `DEFAULT_PALETTE`/`HIGH_CONTRAST_PALETTE` 정의, `/theme` 슬래시 커맨드로 Default ↔ HighContrast 실시간 전환 + config.toml 비동기 저장.
- **[M-3] thiserror 에러 체계 연동**: `config_store.rs`에서 `ConfigError::NotFound`/`ParseFailure`를 실제 코드 경로에 연결. 향후 UI에서 에러 종류별 분기 처리 가능.

### Changed
- `session_log.rs`: 비동기 `append_message` → `append_message_async`로 이름 변경, 동기 `append_message` 신규 추가
- `state.rs`: `WizardStep`, `ConfigPopup`에 `Debug` derive 추가, `SlashMenuState::ALL_COMMANDS`에 `/theme` 추가 (11→12개)
- `wizard_controller.rs`: PersistedSettings 초기화에 `theme` 필드 추가, 미사용 변수 clippy 경고 해소
- `layout.rs`: `tick_count % 2 == 0` → `tick_count.is_multiple_of(2)` clippy 준수
- `palette.rs`: `Palette` 구조체, `get_palette()` 함수, `DEFAULT_PALETTE`/`HIGH_CONTRAST_PALETTE` 상수 추가
- `command_router.rs`: `/theme` 슬래시 커맨드 핸들러 추가

### Quality
- `cargo test`: 28건 전부 통과 (0 failed)
- `cargo clippy --all-targets --all-features -- -D warnings`: 경고 0건 (릴리스 게이트 통과)

## [0.1.0-beta.18] - 2026-04-16

### Added (Phase 10: 기능 완성 — 7건)
- **JSONL 대화 로그**: `~/.smlcli/sessions/session_{ts}.jsonl` — append-only 기록, 복원, 세션 목록 조회
- **CLI Entry Modes**: `smlcli run` (기본 TUI) / `smlcli doctor` (환경 진단) / `smlcli sessions` (세션 목록)
- **SSE 스트리밍**: Provider chat_stream() — stream:true + delta_tx 채널 → ChatDelta 실시간 발행 (OpenRouter/Gemini)
- **Structured Tool Call**: 복수 ```json 블록 감지 + ToolFinished 후 LLM 자동 재전송 (Tool Loop)
- **Stat 도구 구현**: 파일 메타데이터(유형/크기/수정일/권한) 반환 — 와일드카드 제거
- **전역 #![allow] 최소화**: unused_imports/unused_variables 제거 (dead_code만 유지) — 미사용 6+2건 수정
- 신규 의존성: `clap 4` (derive feature)

### Changed (Phase 10)
- chat_runtime: batch chat() → chat_stream() 전환 (delta_forwarder 비동기 태스크)
- chat_runtime: send_chat_message_internal() 추가 — 도구 결과 후 LLM 자동 재전송
- 상태바 ctx% 색상: budget ≥ 85 → `DANGER`(빨강), ≥ 70 → `WARNING`(노랑), 기본 → `MUTED`

### Added (Phase 9-A: 이벤트 아키텍처 기반 — 7건)
- **Action enum 14종 확장**: ChatStarted, ChatDelta, ToolQueued, ToolStarted, ToolOutputChunk, ToolSummaryReady 추가
- **TimelineEntry 이중 데이터 모델**: session.messages(LLM)와 timeline(UI 카드) 분리
- **Semantic Palette**: `tui/palette.rs` 신규 — 전경 6색 + 배경 3계층 + 스피너/배지 상수
- **tick 기반 애니메이션**: thinking 스피너(◐◓◑◒), 도구 배지 깜빡임(●/○), 승인 pulse
- **Inspector Logs 탭 실체**: logs_buffer 기반 실제 로그 렌더링
- **Tool 출력 요약 분리**: raw stdout → 2~4줄 타임라인 요약 + 원문 Logs 탭
- **타임라인 렌더링 전환**: session.messages 기반 → timeline 기반 (폴백 유지)

### Added (Phase 9-B: 보안 강화 — 4건)
- **Blocked Command 목록**: sudo/rm -rf/chmod 777/mkfs/dd/fork bomb 등 15개 패턴 무조건 차단
- **File Read 안전장치**: '..' 경로 traversal 차단 + 1MB 초과 파일 읽기 차단 + 800줄 기본 상한
- **ToolQueued/ToolStarted/ApprovalCard** 이벤트 파이프라인 전면 통합
- **Grep 결과 UX**: context_lines 주변 문맥 + 파일별 그룹 헤딩 + 결과 요약 헤더

### Added (Phase 9-C: 품질 강화 — 3건)
- **Shell 실시간 스트리밍**: stdout/stderr 라인 단위 비동기 스트리밍 (ToolOutputChunk 이벤트 발행)
- **ListDir 재귀 트리**: ├──/└── Unicode tree, 디렉토리 우선 정렬, 1000개 항목 제한
- **테스트 14→24건**: blocked_command(fork bomb/대소문자), timeline(UserMessage/SystemNotice), ToolStatus 전이 등

### Changed
- `layout.rs`: 하드코딩 Color 전면 제거 → Semantic Palette 참조로 교체
- `chat_runtime.rs`: 사용자 메시지/에러를 timeline에도 동기 추가
- `grep.rs`: context_lines 주변 문맥 + 파일별 그룹 헤딩 + 결과 요약 헤더
- `file_ops.rs`: ReadFile 800줄 기본 상한 + 경로 이중 방어
- `mod.rs`: tick 이벤트에서 tick_count 증가, generate_tool_summary() 추가


## [0.1.0-beta.17] - 2026-04-16

### Fixed (감사 리포트 수정 3건)
- **[M-1] 소스 코드 주석 정합성**: `Keyring`→`암호화 저장소`, `config.yaml`→`config.toml` 일괄 교체 (6개 파일 15건)
- **[M-2] /help 다국어 병행 표기**: 영문 단독 → 한/영 병행 (예: `/config 설정 대시보드 (Settings Dashboard)`)
- **[L-1] 테스트 코드 문구 갱신**: `Keyring`→`암호화 저장소` (audit_regression.rs 2건)

### Changed
- `session.rs`: 페르소나 언어 지시를 `한국어 고정` → `사용자 입력 언어 미러링`으로 변경

## [0.1.0-beta.16] - 2026-04-16

### Added (UX 4건 — 감사 결과 반영)
- **Tool JSON 필터링**: AI 응답에서 도구 호출 JSON 스키마가 사용자에게 직접 노출되지 않고 `⚙️ [도구명] 도구 호출 실행 중...` 형태로 표시
- **AI 추론 인디케이터**: 프롬프트 전송 후 AI 응답 수신까지 `✨ AI가 응답을 생성하고 있습니다...` 표시
- **슬래시 커맨드 자동완성 메뉴**: Composer에서 `/` 입력 시 사용 가능한 11개 명령어가 팝업으로 표시, 방향키+Enter로 선택, Esc로 취소
- **에이전트 페르소나 시스템 프롬프트**: CLI 에이전트 역할 정의, 한국어 응답 지시, 도구 호출 시 자연어 설명 병행 지시 (약 1K 토큰)

### Changed
- `session.rs`: 시스템 프롬프트를 단순 도구 나열에서 전문적 페르소나 정의로 대폭 강화
- `state.rs`: `is_thinking`, `SlashMenuState` 추가
- `layout.rs`: `filter_tool_json()` 함수 추가, thinking indicator 렌더링, 슬래시 메뉴 팝업 렌더링
- `mod.rs`: 슬래시 메뉴 키보드 입력 핸들링 (char, Up/Down, Enter, Backspace, Esc)

## [0.1.0-beta.15] - 2026-04-16

### Fixed (감사 3건 수정)
- **[High]** `serde_yml` (RUSTSEC-2025-0067/0068) 제거 → 기존 `toml` 크레이트로 교체
- **[Medium]** 문서-구현 불일치 해소: README/spec.md 내 keyring 참조를 파일 기반 암호화로 교체
- **[Low]** `config.toml`에 chmod 600 권한 설정 추가 (Unix)

## [0.1.0-beta.14] - 2026-04-16

### Changed (아키텍처 변경 — Credential Store 재설계)
- **keyring 크레이트 완전 제거**: OS 의존적 gnome-keyring/secret-service/mock 백엔드 → 크로스플랫폼 파일 기반으로 교체
- **설정 저장 경로 변경**: `~/.config/smlcli/settings.enc` (암호화 바이너리) → `~/.smlcli/config.yaml` (YAML 평문)
- **API 키 저장 방식**: keyring Entry → `config.yaml`의 `encrypted_keys` 맵에 ChaCha20Poly1305 암호화된 값으로 저장
- **마스터 키 저장**: keyring → `~/.smlcli/.master_key` 파일 (hex 인코딩, chmod 600)
- `save_config()` / `load_config()` 시그니처에서 `master_key` 파라미터 제거
- `get_api_key()` / `save_api_key()` 시그니처에 `settings` 참조 추가
- `PersistedSettings`에 `encrypted_keys: HashMap<String, String>` 필드 추가

### Removed
- `keyring` 크레이트 의존성 (+ `dbus`, `dbus-secret-service`, `libdbus-sys` 등 transitive)
- `chacha20poly1305` 전체 파일 암호화 (API 키 암호화에만 계속 사용)

### Added
- `serde_yml` 의존성 (YAML 직렬화/역직렬화)
- `secret_store::encrypt_value()` / `decrypt_value()` 유틸리티 함수

## [0.1.0-beta.13] - 2026-04-15

### Fixed (Critical — 실행 불가 버그)
- **keyring 백엔드 미설정**: `keyring = "3.6.3"` feature 미지정으로 mock credential store(비영속 메모리)가 사용됨.
  - **증상**: Wizard에서 API 키 입력 → 같은 세션 또는 재시작 후 채팅 시 `[Keyring Error] No matching entry found in secure storage`
  - **원인**: keyring v3.x는 `default-features = false`이므로 feature를 명시하지 않으면 어떤 OS 백엔드도 컴파일되지 않고 mock store만 사용
  - **수정**: `features = ["sync-secret-service"]` 추가 → D-Bus Secret Service(gnome-keyring) 백엔드 활성화
  - **영향**: 기존 mock master-key로 암호화된 `settings.enc` 복호화 불가 → 앱 재시작 시 Wizard 재설정 필요

### Changed
- `dbus`, `dbus-secret-service`, `libdbus-sys` 의존성 자동 추가 (keyring feature에 의해)

## [0.1.0-beta.12] - 2026-04-15

### Fixed (High - 8차 감사)
- **[H-1]** Provider 전환 취소 시 rollback 스냅샷 조기 해제: `handle_models_fetched` 성공 시 rollback을 해제하던 것을 제거. 모델 목록 로드 성공 ≠ 사용자 선택 완료이므로, `ModelList` 선택이 완료되고 `save_config`가 성공한 시점에서만 해제.

### Fixed (Medium)
- **[M-1]** `save_config()` 실패 후 메모리-디스크 불일치 수정:
  - **ShellPolicy 토글**: 실패 시 이전 정책으로 in-memory 복구
  - **ModelList 저장**: 실패 시 rollback 스냅샷이 있으면 provider+model 전체 복구, 없으면 이전 model만 복구

### Changed
- `handle_models_fetched` Config 성공 분기에서 rollback 해제 제거
- `ModelList` 저장/`ShellPolicy` 토글에 save 실패 시 in-memory 롤백 로직 추가

## [0.1.0-beta.11] - 2026-04-15

### Fixed (High - 7차 감사)
- **[H-1]** `/config → Model` 경로 보안 가드 우회 차단: `resolve_credentials()` + `validate_credentials()` 적용 (6차 후반 자체 감사에서 수정)
- **[H-2]** Provider 전환 사용자 취소 시 롤백 누락: ModelList/ProviderList에서 Esc로 빠져나올 때 `rollback_provider/rollback_model` 스냅샷에서 이전 provider/model로 in-memory 복구

### Fixed (Medium)
- **[M-1]** `save_config()` 실패 묵살 수정: ShellPolicy 토글과 ModelList 저장에서 `let _` 대신 에러를 `err_msg`로 표시하여 사용자에게 저장 실패 가시화

### Changed
- ModelList 선택 완료 시 rollback 스냅샷 해제 (저장 성공 시에만)
- Config Esc 핸들러에서 err_msg 초기화 추가

## [0.1.0-beta.10] - 2026-04-15

### Fixed (High - 6차 감사)
- **[H-1]** `/provider` 전환 원자성 보장: 비동기 검증 전 `save_config()` 제거 → in-memory만 변경, 검증 실패 시 롤백 스냅샷으로 이전 provider/model 복구. 디스크 저장은 ModelList 선택 완료 시에만 수행.

### Fixed (Medium)
- **[M-1]** `/model` 경로에 `validate_credentials()` 선행 검증 추가: `/provider`와 동일한 검증 일관성 확보
- **[M-2]** 비동기 `ModelsFetched` 라우팅 결함 수정: `FetchSource` enum 도입으로 요청 출처(Config/Wizard) 기반 정확한 상태 슬롯 라우팅 (UI 상태 의존 제거)
- **[M-3]** clippy `collapsible_if` 해소

### Changed (Architecture)
- `Action::ModelsFetched`에 `FetchSource` 태그 추가 (Config | Wizard)
- `ConfigState`에 `rollback_provider`/`rollback_model` 필드 추가
- `handle_models_fetched()`가 source 기반 분기 + 실패 시 롤백 수행

## [0.1.0-beta.9] - 2026-04-15

### Fixed (High - 5차 감사)
- **[H-1]** 보조 경로 보안 가드 우회 차단: `resolve_credentials()` 중앙 가드를 도입하여 `/model`, `/compact`, `/provider` 전환에서도 NetworkPolicy + Keyring 검증 일관 적용
- `/model`: `unwrap_or_default()` 제거 → `resolve_credentials()` 사전 검증
- `/compact`: 동일 패턴 적용 → 빈 키로 LLM 호출하던 경로 차단
- `/provider`: `resolve_credentials_for_provider()` + `validate_credentials()` 후 `fetch_models()` 순서 보장

### Fixed (Medium)
- **[M-1]** `/provider` 전환 시 `validate_credentials()` 미호출 수정: OpenRouter `/auth/key` 엔드포인트로 키 유효성을 먼저 확인
- **[M-2]** Config Dashboard에 `err_msg` 미표시 수정: 대시보드 렌더러 하단에 에러 메시지 표시 영역 추가
- **[M-3]** clippy `field_reassign_with_default` 경고 해소: 구조체 리터럴 + `..Default` 패턴으로 변경

### Fixed (Low)
- **[L-1]** Saving 단계 문구 불일치 수정: "saved successfully" → "Press Enter to save" + 에러 시 `err_msg` 표시

### Changed (Architecture)
- `chat_runtime.rs`에 `resolve_credentials()` / `resolve_credentials_for_provider()` 중앙 보안 가드 메서드 도입
- `dispatch_chat_request()`를 동기 사전 검증 → 비동기 spawn 패턴으로 리팩토링

## [0.1.0-beta.9] - 2026-04-21

### Fixed (High - 5차/최종 감사)
- **[H-1]** 동시 채팅 요청 차단: `is_thinking` 상태일 때 사용자가 Composer에서 Enter를 눌러 새 요청을 전송하지 못하도록 차단 (응답 덮어쓰기 Race condition 방지)
- **[H-2]** 초기 사용자 턴 블록 분리: 첫 요청 시 `Assistant` 롤을 확인하고 명시적인 전용 AI 블록을 생성하여 오류가 사용자 블록에 섞이지 않도록 분리
- **[H-3]** 도구 루프 후속 렌더링 블록 오염 방지: `send_chat_message_internal()`을 통해 도구 실행 완료 후 발생하는 LLM 재전송 요청에서도 `active_chat_block_idx`를 정확히 설정하도록 통합 헬퍼(`spawn_chat_request`)로 리팩토링. 이를 통해 이전 턴의 블록을 오염시키는 경로 차단

### Fixed (Medium)
- **[M-1]** Trust Gate 마우스 우회 차단: Trust Gate 모달이 띄워져 있을 때 마우스 클릭이나 스크롤 라우팅을 차단하여 접근 통제
- **[M-2]** Windows Shell 호환성 검증: `host_shell`을 하드코딩에서 환경변수 `ComSpec`으로 개선하고, `exec_shell` 탐색 시 `pwsh`와 `powershell.exe` 존재 여부를 런타임에 직접 확인하여 없으면 에러 반환

### Fixed (Low)
- **[L-1]** Command Palette 스펙 부합 보완: 렌더러 최대 표시 개수를 8개로 제한하고 검색 로직에 순차 문자 매칭(Fuzzy search) 및 최대 50건 반환 스펙 적용

## [0.1.0-beta.8] - 2026-04-15

### Fixed (High - 4차 감사)
- **[H-1]** 위자드 저장 실패 무시 수정: `save_api_key()`/`save_config()` 실패 시 `err_msg` 설정 후 위자드 유지 (재시작 후 깨짐 방지)
- **[H-2]** API 키 평문 노출 차단: 렌더러에서 `*` 마스킹 적용, 검증 실패 `err_msg` 화면 표시 추가
- **[H-3]** `/provider` 전환 안전성 확보: Provider 변경 시 `default_model`을 `"auto"`로 초기화, API 키 존재 확인 후 자동 ModelList 전이
- **[H-4]** `NetworkPolicy::Deny` 실적용: `chat_runtime.rs`에서 채팅 요청 전 정책 검사 → Deny 시 차단 메시지 반환

### Fixed (Medium)
- **[M-1]** 위자드 오류 화면 Esc 복구: 에러 상태에서 Esc 시 앱 종료가 아닌 ProviderSelection으로 복귀
- **[M-2]** 회귀 테스트 10건 추가: 감사 항목별 상태 전이/정책 검증 테스트 (`audit_regression.rs`, 4→14건)

### Fixed (Low)
- **[L-1]** `cargo fmt --check` 게이트 통과 확인

## [0.1.0-beta.7] - 2026-04-15

### Fixed (Critical)
- **[C-1]** OpenRouter API 키 검증 우회 수정: 위자드에서 `validate_credentials()` 호출 후에만 모델 목록 진행
- **[C-2]** Gemini 모델 식별자 불일치 수정: `models/` 프리픽스를 strip하여 bare model id로 저장 (공식 문서 대조 확인)
- **[C-3]** `dummy_key` 무음 대체 제거: Keyring 조회 실패 시 명시적 에러 메시지 표시 및 채팅 요청 중단
- **[C-4]** 시스템 프롬프트 타임라인 노출 수정: `pinned System` 메시지를 렌더링에서 필터링

### Fixed (High)
- **[H-1]** `/config`, `/provider`, `/model` 팝업에 Up/Down/Enter 키 핸들러 구현 (설정 변경 및 즉시 저장)
- **[H-2]** `/clear` 명령이 시스템 프롬프트까지 삭제하던 버그 수정: `pinned` 메시지 보존
- **[H-3]** `ReplaceFileContent` 도구 실행기 구현: read → string replace → atomic write 패턴
- **[H-4]** `ChatMessage.pinned` 필드가 Provider API 페이로드에 포함되던 문제 수정 (`skip_serializing`)
- **[H-6]** 상태바 하드코딩(`/workspace`, `Shell Ask`) 제거: 실제 CWD 및 정책 동적 표시

### Changed (Architecture - Phase 3 Complete)
- **[리팩토링]** `src/app/mod.rs` God Object(773줄 → 422줄) 5개 모듈 완전 분해:
  - `command_router.rs` (215줄): 슬래시 커맨드 엔진 (12개 커맨드 파싱/실행)
  - `chat_runtime.rs` (90줄): LLM 요청 조립, API 키 조회, Provider 디스패치
  - `tool_runtime.rs` (173줄): 도구 JSON 파싱, 권한 검사(PermissionEngine), 비동기 실행, 승인 y/n, 직접 셸 실행
  - `wizard_controller.rs` (222줄): Setup Wizard 상태 전이(Provider→Key→Model→Save), Config 팝업 Enter 처리
  - `mod.rs` (422줄): 이벤트 루프 오케스트레이터 + 입력 핸들러(키별 소형 메서드) + Fuzzy Finder
- **[M-1]** WizardStep::Home, PermissionPreset 미사용 variant 제거
- **[M-5]** `cargo fmt` 적용으로 전체 코드 포매팅 통일
- `CredentialValidated` 이벤트를 Action enum에 추가하여 비동기 인증 흐름 구현

## [0.1.0-beta.6] - 2026-04-15

### Added
- **[Phase 7] 지능형 하이브리드 컨텍스트 압축 엔진(Intelligent Compaction) 도입**
- 동적인 `token` 임계값(Threshold) 추정기 및 UI 모니터링 메뉴 추가 (`/tokens`)
- `/compact` 호출 또는 한계치 돌파 시, 배경 비동기 LLM 요약기(Summarizer)를 가동해 단순 버리기가 아닌 압축 축소화(Collapse) 적용
- 프롬프트 엔지니어링 구조가 망각되지 않도록 방어하는 Pinned 메시지(보존 지시) 메타데이터 적용
- TUI 오버레이를 사용한 사용자 설정 종합 대시보드 (`/config` 명령어 추가)
- `/setting`, `/status`, `/mode`, `/clear` 등 TUI 및 모델 설정 제어를 위한 슬래시 커맨드 라우팅 파이프라인
- **[UX]** Composer 내 `@` 타이핑 시 현재 디렉터리 파일의 Fuzzy Finder 팝업 인터페이스 연동 (Enter 시 파일 참조 주입)
- **[UX]** Inspector 패널 상단에 상태 기반 동적 탭 네비게이션([Preview], [Diff], [Search], [Logs] 등) UI 도입
- SessionState 내에 컨텍스트 임계값을 넘지 않도록 관리하는 토큰 예산 관리 모듈
- 슬래시 커맨드 파싱 및 처리 엔진: 상하 방향키 조작 및 엔터로 빠른 선택 지원 (`/status`, `/mode`, `/help`, `/clear` 등)
- 컨텍스트 압축기능 추가 (`/compact`): 토큰 과소비 방지를 위해 비동기 LLM 컨덴서를 사용하여 요약 압축 수행

### Changed
- **[안정성]** `file_ops.rs`의 `write_file_commit()`이 디스크 기록 중단 시 파일 파손을 막기 위해 원자적 `.tmp` 생성 후 `rename` 하는 방식으로 개선 (Atomic Write)
- **[안정성]** `src/tools/shell.rs`의 셸 실행(`Command::output().await`) 구문에 30초 `tokio::time::timeout` 래퍼를 씌워 좀비 프로세스 방지
- **[보안]** Safe Command 하드코딩 탈피: OS 호스트 감지(Windows/Linux 분리) 적용 및 `PersistedSettings` 내 커스텀 `safe_commands` 지원 병합

### Removed
- 단순 배열 하드 드롭으로 장기 문맥을 파괴하던 기존 `compact_context()` 레거시 함수를 `session.rs`에서 완전 제거

### Fixed
- **[CRITICAL]** Setup Wizard 종료 시 `AppState::settings`가 즉시 갱신되지 않아 재부팅 전까지 초기 설정을 인식하지 못하던 버그 수정
- **[SECURITY]** `PermissionEngine` 도입으로 `ShellPolicy`, `FileWritePolicy` 정책 강제 적용 (SafeOnly, Deny, Ask 모드 분기 로직 구현)
- **[UX]** Composer `!` 접두사를 통한 직접 셸 실행 기능 추가 및 보안 정책 연동

### Changed
- `spec.md` 파일 구조를 실제 구현된 모듈 구조(session.rs, permissions.rs 등)와 일치하도록 최신화
- `PermissionToken` 무결성 검증 및 `ChatResponseOk` 내 자동 실행/승인 대기 로직 분리

## [0.1.0-beta.5] - 2026-04-14

### Added
- 대화형 TUI 마법사 고도화: 시작 화면 없이 방향키 조작만으로 Provider, API Key, Model을 끊김없이 순차적으로 선택/저장하는 자동화 플로우 도입
- API 모델 동적 호출(`reqwest` GET): 인증키 획득 직후 비동기 방식으로 Provider별 수백 개의 모델 리스트를 불러오고 스크롤 바인딩 제공
- 멀티플랫폼 대화형 크로스 컴파일(Linux Native/MinGW-w64)을 지원하는 컴파일 보조 셸 스크립트(`build.sh`) 작성

### Changed
- `Cargo.toml` 패키지 명칭을 `temp_scaffold`에서 `smlcli`로 공식 변경

## [0.1.0-beta.4] - 2026-04-14

### Added
- `OpenRouter` 및 `Gemini` 제공자와 실시간 통신하는 비동기 이벤트 루프(`Tokio` + `reqwest`)
- 프롬프트에 정의된 JSON Tool 포맷을 자동 파싱하여 `PendingTool` 승인 상태로 변환하는 중계기
- `Approve(y) / Deny(n)` 인터페이스 및 `Inspector` 동적 렌더링 레이아웃 (`Ctrl+I` 토글)
- 파일 렌더링 변경 시 출력되는 Diff 비교에 Ratatui Span 기반 초록/빨강 색상 적용
- `OS Keyring` 및 `XChaCha20`을 결합한 보안 설정 관리자(`Setup Wizard` 적용)

### Changed
- 모든 도구(Shell, File Ops) 실행부를 `pub(crate)`로 제한하여 외부 캡슐화 및 권한 토큰 분리
- Windows 환경에서 셸 실행 시 `cmd` 대신 `powershell -Command` 사용으로 보안/호환성 증대

### Security
- 무결성 없는 도구 접근을 막기 위한 `PermissionToken` 지연 승인 패턴 도입

### Deprecated
- 없음

### Removed
- 없음

### Fixed
- 없음

### Security
- 프로젝트 전반에 걸친 보안 검토 가이드 등록 (`audit_roadmap.md`)
