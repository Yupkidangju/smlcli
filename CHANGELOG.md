# Changelog

모든 중요한 변경 사항은 이 문서에 기록됩니다.
이 프로젝트는 [Semantic Versioning](https://semver.org/) 기준을 따릅니다.

## [Unreleased]

### Docs
- **Phase 15: 2026 CLI UX 현대화 로드맵 문서화**: `spec.md`, `designs.md`, `IMPLEMENTATION_SUMMARY.md`, `DESIGN_DECISIONS.md`, `audit_roadmap.md`에 최신 CLI/TUI UX 패턴을 반영한 리팩토링 및 기능 강화 계획을 추가. 블록 기반 타임라인, 커맨드 팔레트, 입력 툴벨트, 반응형 상태바, 절제된 ASCII 애니메이션, 포커스/스크롤 상태 머신을 구현 전용 스펙으로 동결.

### Changed/Improved (Phase 15-A: TimelineBlock 마이그레이션)
- **블록 기반 타임라인 도입**: 기존 `TimelineEntry` 기반 단일 텍스트 렌더링에서 `TimelineBlock`, `BlockSection`, `BlockStatus` 상태 머신 기반의 모듈식 아키텍처로 완전히 교체. 
- **컴파일/의존성**: 고유 식별자 할당을 위한 `uuid v4` 의존성 추가.
- **렌더링 시스템 교체**: `src/tui/layout.rs` 및 `src/tui/widgets/inspector_tabs.rs`가 새로운 `TimelineBlock` 모델을 순회하여 렌더링하도록 재작성. (기존 `TimelineEntry` 및 `ToolStatus` 완전히 제거)

### Added (Phase 15-B/C/D/E: UX / State Machine & Inspector Workspace)
- **포커스 상태 머신 (`FocusedPane`)**: 타임라인, 인스펙터, 컴포저, 팔레트 등 포커스 기반 독립 스크롤링 및 키보드 이벤트 라우팅 도입 (`src/app/mod.rs`). 활성화된 패널은 Accent 색상 경계선으로 시각화.
- **커맨드 팔레트 (`Command Palette`)**: `Ctrl+K` 입력 시 팝업되는 퍼지 검색 기반 빠른 실행 명령 레이어(`src/tui/layout.rs`) 도입. (현재 `toggle_inspector`, `compact`, `clear`, `help` 지원)
- **Composer Toolbar**: 하단 입력창(Composer) 상단에 현재 작업 맥락(`[RUN]/[PLAN]`, `CWD`, `Shell Policy`) 및 `[Ctrl+K]` 힌트 칩을 렌더링하는 툴바 영역 도입.
- **다중 라인 프롬프트 지원**: `Shift+Enter` 를 통해 Composer 버퍼에 줄바꿈(`\n`)을 삽입할 수 있도록 멀티라인 입력 처리 추가 (`src/app/mod.rs`).
- **Adaptive Header**: 윈도우 폭에 맞춰 상단 바 정보가 생략되는 반응형 정책(Adaptive Header) 적용 완료.
- **타임라인 커서 및 Inspector Preview (`Phase 15-E`)**: 타임라인 내 블록 이동(`Up`/`Down`)을 위한 커서를 추가하고, 선택된 블록의 전체 마크다운 및 코드 펜스를 Inspector의 `Preview` 탭에서 확인할 수 있도록 재구성 (`src/tui/widgets/inspector_tabs.rs`).
- **Inspector Diff 탭 (`Phase 15-E`)**: 파일 수정 등 승인 대기 중인 변경사항(Diff)을 직관적으로 확인할 수 있도록 `Diff` 탭의 렌더링 구현 추가.
- **Motion Polish 애니메이션 개선 (`Phase 15-F`)**: LLM 생성 중 보여지는 Thinking 스피너를 점자(`⠁⠂⠄⡀⢀⠠⠐⠈`)로, 도구 실행 상태 배지를 `▶/▷`로 변경하여 `designs.md`의 모션 요구사항을 준수.

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

