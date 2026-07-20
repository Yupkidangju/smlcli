# SMLCLI Audit Roadmap

본 문서는 `smlcli`의 무결성을 유지하기 위한 4단계(정합성, 위험요소, 아키텍처, 로드맵) 감사 프로세스입니다. 버전이 변경되거나 기능 규모가 커질 때, AI 에이전트는 `/audit` 트리거 발동 시 이 문서를 기반으로 코드 리포트를 출력해야 합니다.

## 1단계: 정합성 유지 (Consistency Audit)
개발 도중 코드와 스펙 문서가 어긋나지 않는지 점검합니다.
- `spec.md` vs 실제 코드의 권한 모델 일치 확인 (Safe, Confirm, Blocked 모드)
- `designs.md` vs 실제 TUI 인터랙션 키보드 맵 매칭 확인 (Tab 작동 여부 등)
- Rust 의존성의 버전 호환 테스트(`cargo check && cargo clippy`)
- 설정 마법사의 상태 다이어그램이 코드 흐름과 정확히 일치하는지 분석

## 2단계: 위험요소 및 보안 확인 (Risk & Security Audit)
명시적인 보안 홀을 점검하고, 사용자 동의 여부 절차를 검사합니다.
- 파일 기반 암호화 저장소(~/.smlcli/)를 우회하여 API 키가 평문 노출되는지 확인
- `diff` 검토나 `Ask` 확인 로그 없이 파일 쓰기(write)나 셸 명령이 통과되는 하드코딩 여부 추적
- 심볼릭 링크 공격 탐지 및 범위를 벗어난 홈 디렉터리 수정 시도 검열
- 과도한 stdout 스트리밍으로 인한 메모리 누수 점검

## 3단계: 아키텍처 결함 감지 (Architecture Audit)
애플리케이션의 모듈 분리가 `spec.md`를 잘 따르고 있는지 평가합니다.
- 도메인 레이어: `permissions`, `session`, `settings`가 UI 로직에 얽혀 있지 않은지 점검.
- 터미널 렌더러 분리: `tui/` 폴더 밖에서 터미널 제어 코드가 불리지 않는지 확인.
- Provider Layer: 상호 연결 인터페이스(trait)를 제대로 사용하여 하드 코딩 방지 여부 점검.

## 4단계: 액션 로드맵 (Remediation Roadmap)
감사 이후 수정이 필요한 항목들을 즉시 도출하고 실행 가능한 계획으로 변경합니다.
1. **Critical:** 빌드 에러, 보안 누수, 문서와 극명하게 어긋나는 동작 -> 즉각 수정
2. **High:** TUI 오작동, 치명적인 패닉 발생, 권한 바이패스 -> Phase 내 수정
3. **Medium:** `similar` 기반 Diff 가시성 문제, 디자인 틀어짐 -> UI 수정
4. **Low:** 문서 오탈자, 추가 로그 레벨 도입 -> 보완

---

### Audit Trigger Command
사용자가 명령어로 `/audit` 지정 또는 "감사 실행" 관련 지시를 내리면, AI 에이전트는 소스 루트를 탐색한 수 위 4단계의 통과/실패 항목과 진행 권고를 출력하십시오.

---

## Phase 9 UX 아키텍처 개편 감사 기준 (v0.1.0-beta.18+)

### 9-A 이벤트 기반 구조 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Action enum 14종 | `action.rs` 내 variant 수 확인 | ChatStarted, ChatDelta, ToolQueued, ToolStarted, ToolOutputChunk, ToolSummaryReady 존재 |
| TimelineEntry 분리 | `state.rs` 에 `timeline: Vec<TimelineEntry>` 필드 존재 | session.messages와 별도 관리. timeline이 비어있을 때만 session.messages 폴백 허용 (하위 호환) |
| Semantic Palette | `tui/palette.rs` 상수 존재 | layout.rs/widgets에서 하드코딩 Color 사용 0건 |
| tick 기반 애니메이션 | `tick_count` 기반 스피너/깜빡임 | thinking indicator가 4프레임 회전 |
| Inspector 탭 실체 | `widgets/inspector_tabs.rs` 존재 | Preview/Diff/Search/Logs/Recent 각 탭에 실제 콘텐츠 |
| Tool 출력 요약 분리 | ToolFinished 핸들러 검사 | 타임라인에 2~4줄 요약, logs_buffer에 원문 |
| SSE 스트리밍 | `chat_stream()` 메서드 존재 | ChatDelta 이벤트로 토큰 단위 수신 |

### 9-B 기능 완성 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| CLI Entry Modes | `main.rs`에 clap 파싱 | `run`, `doctor`, `sessions` 서브커맨드 동작 |
| 세션 영속성 | `session_log.rs` 존재 | JSONL 저장/복원 round-trip |
| SafeOnly 화이트리스트 | `permissions.rs` 검사 | safe_commands 미매칭 시 Deny |
| Blocked Command | `permissions.rs` 검사 | sudo/rm -rf 등 무조건 차단 |
| Structured Tool Call | `registry.rs` 검사 | fenced JSON 스크래핑 외 native contract 존재 |
| File Read 안전장치 | `file_ops.rs` 검사 | 경로 정규화 + 1MB 제한 |
| Grep UX | `grep.rs` 검사 | context_lines + max_results |
| 프롬프트 커맨드 확장 (@, !) | `FuzzyMode` enum 존재 | `ignore::WalkBuilder` 사용 유무, `history_idx` 상태 보존 유무 검사 |

### 9-C 품질 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Shell 스트리밍 | `shell.rs` 검사 | spawn + BufReader + ToolOutputChunk |
| 테스트 확장 | `cargo test` | 22건+ (secret round-trip, cancel/rollback, tool lifecycle, layout snapshot 포함) |
| 전역 allow 제거 | `main.rs` 검사 | `#![allow(dead_code)]` 등 **crate-level 전역** allow 0건. 모듈 단위 국소 `#[allow(dead_code)]`는 예약 코드 사유 주석과 함께 허용 |

---

## Phase 13 Agentic Autonomy 개편 감사 기준

### 13-A 도구 및 실행 아키텍처 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Tool Registry | `src/tools/registry.rs` 코드 확인 | `Tool` 트레이트를 구현하는 개별 도구 구조체(`ReadFile`, `WriteFile` 등)가 존재하며 다형성 호출 보장 |
| Automated Git Checkpoints | `git_checkpoint.rs` 소스 검사 + `test_git_checkpoint_source_has_no_git_clean` 테스트 | `create_checkpoint()`는 강제 커밋 없이 워킹 트리 clean 여부만 반환(`Result<bool>`). `rollback_checkpoint()`에 `git clean -fd` 없음. `git reset --hard` 종료 코드 검사 필수. WIP 존재 시 롤백 건너뜀 |
| ExecShell 파괴 판정 | `shell.rs` 코드 확인 + `test_exec_shell_is_not_destructive` 테스트 | `ExecShell`의 `is_destructive()`는 기본값(`false`)을 반환하며, 쉘 명령 실패가 Git 롤백을 트리거하지 않음 |
| Auto-Verify Loop | `mod.rs` ToolFinished/ToolError 분기 확인 + `test_auto_verify_state_transitions` + `test_auto_verify_abort_stops_resend` 테스트 | `ToolFinished(is_error=true)`와 `ToolError` 양쪽 경로 모두에서 힐링 프롬프트 주입 + `send_chat_message_internal()` 재전송. 최대 3회(`retries < 3`) 초과 시 `Idle`로 전환(Abort)하고 **LLM 재전송을 중단** |
| Tool Schema 재전송 | `chat_runtime.rs` `send_chat_message_internal()` 확인 | 초기 요청과 내부 재전송 양쪽 모두 `GLOBAL_REGISTRY.all_schemas()`를 `req.tools`에 주입하여 LLM이 후속 도구 호출 가능 |

### 13-B 에이전트 인텔리전스 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Tree-sitter Repo Map | `smlcli` 부팅 후 디버그 로그 점검 + `test_repo_map_generation` 테스트 | `System` 프롬프트 하단에 `[Repo Map]` 헤더(`repo_map.push_str("[Repo Map]\n")`)로 추출된 주요 함수/구조체 시그니처 8,000바이트 이하 주입 확인 |
| Tree of Thoughts UI | 타임라인 렌더링 확인 | 메인 응답 텍스트 블록 아래에 도구 실행과 에러 이력이 인덴트(`└─`)로 계층화되어 표시됨 |
| PLAN/RUN 모드 분리 | `chat_runtime.rs`에서 `AppMode::Plan`/`AppMode::Run` 프롬프트 지시문 존재 확인 + `test_plan_run_mode_toggle` 테스트 | PLAN 모드에서는 분석·설명에 집중하고 도구 호출을 자제하라는 시스템 프롬프트 주입. RUN 모드에서는 파일 생성/수정 도구를 적극 사용하라는 시스템 프롬프트 주입. 모드 전환 시 dedupe 방식으로 기존 지시를 교체. ※ 코드 아키텍처 수준의 Planner/Executor 분리(별도 Action variant)는 향후 로드맵 항목 |
| 직접 셸(`!`) 정책 | `tool_runtime.rs` `handle_direct_shell_execution` 코드 확인 + `test_direct_shell_safe_to_auto_run_is_false` 테스트 | `safe_to_auto_run: false`가 하드코딩되어 SafeOnly 모드의 allowlist 정책을 우회하지 않음 |
| 회귀 테스트 커버리지 | `cargo test` | Phase 13 전용 테스트 8건 이상 통과: `git_checkpoint_dirty_tree_skip`, `rollback_non_git_repo_is_noop`, `git_checkpoint_source_has_no_git_clean`, `auto_verify_state_transitions`, `auto_verify_abort_stops_resend`, `exec_shell_is_not_destructive`, `repo_map_generation`, `direct_shell_safe_to_auto_run_is_false` |

---

## Phase 14: TUI UX/UI 고도화 감사 (v0.1.0-beta.24)

### 14-A 멀티라인 렌더링 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| /help 구조화 렌더링 | `layout.rs` 소스 확인 + `/help` 입력 | `TimelineBlockKind::Help` + `BlockSection::KeyValueTable` 구조 사용. 명령어 Span(고정 11칸, accent)과 설명 Span(text_secondary)이 분리된 Line 렌더링. Paragraph wrap 시 명령어 컬럼 유지 |
| AI 응답 줄바꿈 보존 | 멀티라인 응답 수신 후 타임라인 확인 | 빈 줄, 줄바꿈, 문단 구분이 그대로 보존됨 |
| render_multiline_text 존재 | `layout.rs` 소스 확인 | `render_multiline_text(text, style) -> Vec<Line<'static>>` 헬퍼 존재 확인 |

### 14-B 스크롤 모델 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| 스크롤 상태 분리 | `state.rs` 필드 확인 | `timeline_scroll`, `inspector_scroll`, `timeline_follow_tail` 세 필드 독립 존재 |
| 마우스 캡처 | `terminal.rs` 소스 확인 | `EnableMouseCapture` / `DisableMouseCapture` 호출 존재 |
| 마우스 이벤트 전달 | `event_loop.rs` 소스 확인 | `CrosstermEvent::Mouse` → `Event::Mouse` 매핑 존재 |
| Auto-follow 동작 | PageUp → PageDown → End 키 순서 테스트 | 바닥에서 자동 추적, 위로 스크롤 시 고정, End 시 복귀 |

### 14-C 키바인딩 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Ctrl+I 바인딩 제거 | `mod.rs` 소스 검색 | `KeyCode::Char('i')` + `CONTROL` 매칭 없음 |
| F2 인스펙터 토글 | `mod.rs` 소스 확인 | `KeyCode::F(2) => show_inspector` 토글 존재 |
| 상태 바 문구 동기화 | `layout.rs` 소스 확인 | `"(Tab) 모드 전환 | (F2) 인스펙터 토글"` 문자열 존재 |

### 14-D 반응형 레이아웃 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| cwd 중략 | `layout.rs` 소스 확인 | `truncate_middle()` 헬퍼 존재 + 상단 바에서 cwd에 적용 |
| provider/model 중략 | `layout.rs` 소스 확인 | provider(`truncate_middle(12)`)와 model(`truncate_middle(20)`) 적용 |
| 적응형 상단 바 | `layout.rs` 소스 확인 | 세그먼트별 Span 분리. 폭 부족 시 cwd → shell policy 순으로 생략. `bar_width` 기반 조건 분기 존재 |
| 인스펙터 폭 클램프 | `layout.rs` 소스 확인 | `Constraint::Percentage(30)` 제거, `clamp(32, 48)` 로직 존재 |
| 탭 라벨 축약 | `layout.rs` 소스 확인 | 폭 < 40 시 축약형 라벨 사용 확인 |

---

## Phase 15: 2026 CLI UX 현대화 감사 기준 (계획)

### 15-A Block Timeline 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| TimelineBlock 타입 | `state.rs` 소스 확인 | `TimelineBlock`, `BlockSection`, `BlockStatus` 타입 존재 |
| 블록 헤더 렌더링 | `layout.rs` 렌더링 확인 | 블록 제목, 상태 배지, 부제목이 독립 구조로 렌더링 |
| stdout/stderr 접힘 | 긴 도구 출력 시뮬레이션 | 12줄 초과 시 기본 접힘 + 펼치기 동작 존재 |

### 15-B Command Palette 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Palette 상태 타입 | `state.rs` 소스 확인 | `CommandPaletteState`와 `PaletteCommand` 타입 존재 |
| Ctrl+K 바인딩 | `mod.rs` 소스 확인 | `Ctrl+K`가 palette 토글로 매핑됨 |
| Fuzzy 검색 | 검색어 입력 후 결과 확인 | 최대 8개 기본 노출, category/shortcut hint 표시 |

### 15-C Composer Toolbar 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Toolbar 칩 구조 | `state.rs` / `layout.rs` 확인 | mode/path/context/policy/hint 칩 존재 |
| Shift+Enter 멀티라인 | 입력 시나리오 검증 | `Shift+Enter`는 줄바꿈, `Enter`는 제출 |
| Context chip 표시 | `@` 파일 첨부 후 확인 | 최대 5개 칩 표시, 길이 초과 시 중략 |

### 15-D Focus & Scroll 상태 머신 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| FocusedPane 타입 | `state.rs` 확인 | `Timeline/Inspector/Composer/Palette` 4종 존재 |
| 포커스별 키 라우팅 | `mod.rs` 확인 | 포커스된 pane에만 스크롤/선택 입력 적용 |
| 마우스 패널 라우팅 | 수동 검증 | 포인터가 올라간 패널만 스크롤 |

### 15-E 반응형 / 모션 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Breakpoint 레이아웃 | 100/120/140 cols 스냅샷 | compact/standard/wide 레이아웃이 모두 안정적 |
| Adaptive top bar | 상단 바 확인 | provider/model/mode/ctx가 우선 표시되고 덜 중요한 정보는 점진적으로 생략 |
| ASCII 모션 정책 | 실행/승인/오류 상태 확인 | 상태 전달용 모션만 존재, 과한 깜빡임 없음 |

---

## Phase 16: Deep UI Interactivity & Provider Hardening 감사 기준

### 16-A Collapsed Diff UI 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| State 무결성 | `state.rs` 소스 확인 | `BlockDisplayMode` 타입 존재 및 `TimelineBlock`에 적용됨 |
| 접기/펼치기 기본 조건 | 12줄 이상 Diff 생성 시뮬레이션 | 10줄을 초과하는 변경 사항은 타임라인 추가 시 `Collapsed`로 설정됨 |
| 렌더링 표기 | 렌더링 결과 확인 | `[ +N lines / -M lines ] (Enter 키로 펼치기)` 형식의 `muted` 스타일 라인이 노출됨 |
| 상태 스왑 라우팅 | 타임라인 포커스 후 Enter 입력 | Enter 입력 시 블록의 `Collapsed` ↔ `Expanded` 상태가 즉시 토글됨 |

### 16-B Provider & Config Error 구조화 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| ConfigError 타입 | `config_store.rs` 확인 | `anyhow::Result` 대신 `Result<T, ConfigError>`가 사용되며 NotFound/ParseFailure 등이 명확히 분리됨 |
| ProviderError 타입 | `registry.rs` 확인 | `ProviderAdapter`가 `Result<T, ProviderError>`를 반환하며 Network/Api/Auth 에러가 도메인 레벨에서 구분됨 |
| 에러 노출 무결성 | 설정 로드 실패 시뮬레이션 | 에러가 뭉뚱그려지지 않고 명시적인 복구 가이드와 함께 UI로 노출됨 |

### 16-C Tool Dialect 추상화 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Dialect Enum | `provider.rs` 확인 | `ToolDialect` enum (`Anthropic`, `OpenAICompat`, `Gemini`) 존재 |
| 스키마 변환 적용 | `tools/registry.rs` 확인 | `all_schemas(&dialect)` 호출 시 Provider에 맞춰 JSON Schema가 패치됨 (예: Gemini의 빈 `required` 배열 강제 삽입) |
| Runtime 연동 | `chat_runtime.rs` 확인 | `ProviderKind`에 따라 올바른 `ToolDialect`를 추론하여 스키마 조립에 사용함 |

---

## Phase 17: Workspace Trust Gate & Shell Alignment 감사 기준

### 17-A Workspace Trust Gate 감사

| 항목 | 검증 초점 | 합격 기준 |
|------|-----------|-----------|
| 시작 차단 프롬프트 | `Unknown` 워크스페이스에서 앱 실행 | 메인 렌더링 전 Trust 선택(3옵션) 프롬프트가 노출되며 선택 전까지 입력 불가 |
| Restricted 격리 | `Restricted` 선택 후 도구 동작 시뮬레이션 | `WriteFile`, `ReplaceFileContent`, `ExecShell` 권한 검사 시 `Denied` 반환 및 차단 알림 발생 |
| Trust 영속화 | `Trust & Remember` 후 앱 재시작 | `config.toml`에 정책 저장되어 다음 실행 시 프롬프트 노출 없이 `Trusted` 상태 유지 |
| REPL 및 상태 연동 | `/workspace` 커맨드 입력 | 현재 루트, 권한 상태가 상태바에 명시되며 슬래시 명령어로 신뢰/차단/조회 가능 |

### 17-B Windows Shell Host Alignment 감사

| 항목 | 검증 초점 | 합격 기준 |
|------|-----------|-----------|
| Exec Shell 환경 추론 | Windows에서 실행 후 `cmd` 또는 `pwsh` 환경 감지 | `pwsh` 혹은 `powershell.exe`가 최우선으로 `ExecShell`의 셸 백엔드로 동작함 |
| 정책 예외 테스트 | Linux의 bwrap 모드 구동 확인 | OS별 분기(linux/windows)가 올바르게 분리 적용됨 |

---

## Phase 18: Multi-Provider Expansion & Advanced Agentic Tools 감사 기준 (계획)

### 18-A Multi-Provider & Model Grounding 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| 2026.04 모델명 유효성 | `src/domain/provider.rs` 및 `config.toml` 확인 | `gpt-5.4`, `claude-opus-4.7`, `grok-4.20` 등 최신 모델 라인업이 명시되어야 함 |
| Base URL 무결성 | 각 Provider Adapter 네트워크 호출 인터셉트 | OpenAI(`api.openai.com/v1`), Anthropic(`api.anthropic.com/v1/messages`), xAI(`api.x.ai/v1`) 엔드포인트 올바른 전송 확인 |
| Dialect 호환성 | `chat_runtime.rs` 및 Tool JSON Schema 조립 확인 | 각 Provider의 요구사항(Gemini의 빈 배열 required 등)이 Dialect를 통해 정확히 패치됨 |

### 18-B Advanced Tools (에이전트 부가 도구) 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| ListDirectory | `ListDirectory` ToolCall을 빈 디렉터리 및 다중 파일 디렉터리에 실행 | 파일 크기, 종류가 포함된 JSON 트리가 정상 반환되며 무한 루프나 권한 패닉이 발생하지 않음 |
| GrepSearch | `GrepSearch` ToolCall로 정규표현식 매칭 실행 (`is_regex: true`) | 일치하는 파일 경로와 라인 넘버, 텍스트 일부분이 정확히 도출됨 |
| FetchURL | `FetchURL` ToolCall로 외부 문서 URL 요청 시뮬레이션 | HTML/데이터가 Markdown 텍스트로 적절히 파싱되어 컨텍스트에 삽입됨 |

---

## Phase 19: v1.0.0 Audit Remediation 감사 기준 (완료)

### 19-A Core Error & Resource 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| `SmlError` 통합 여부 | `infra/` 폴더 내 함수 반환형 검사 | `Box<dyn Error>`가 존재하지 않고 `SmlError`만 반환됨 |
| `BufWriter` 핸들 누수 | 도구 100회 실행 후 파일 핸들 검사 | `lsof -p <PID>` 결과 열린 파일 디스크립터 수가 일정하게 유지됨 |

### 19-B Logic & Security 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Wizard 빈값 상태 전이 | Wizard에서 API Key 입력 없이 Next 이동 | 이동이 제한되고 에러 메시지(Missing Required Field)가 출력됨 |
| `is_dangerous` 검열 | `rm -rf *` 도구 실행 시도 | 권한 검사 엔진에서 `PermissionResult::Deny`로 차단됨 |

### 19-C Runtime Concurrency & TUI 성능 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| `ToolRuntime` 무상태화 | `execute()` 함수 서명 검사 | `Result<ToolResult, ToolError>`만 반환하며 `&mut state` 참조가 없음 |
| 데드락 해소 (Select Race) | 무한 대기 도구 실행 중 `Ctrl+C` 입력 | 채널 블로킹 없이 즉시 이벤트 루프로 취소 신호가 전파됨 |
| TUI Windowed Rendering | 20,000줄의 stdout 로그 생성 | `terminal_height` 기준 슬라이싱으로 프레임 드랍 없이 스크롤됨 |

---

## Phase 21: v1.3.0 Final Industrial Polish (완성도 향상 및 엣지 케이스 수정)

### 21-A Stability & I/O 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Panic 터미널 복구 | 코드 내 임의 패닉 발생 후 앱 강제 종료 | Raw Mode가 해제되고 커서와 입력 에코가 정상적으로 동작함 |
| 비동기 I/O 블로킹 검증 | 방대한 크기의 프로젝트 폴더 탐색 (repo_map 생성) | TUI 이벤트 루프가 멈추지 않고 키보드/마우스 입력에 매끄럽게 반응함 |

### 21-B UX & Memory 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| ANSI 제어 문자 렌더링 | `ls --color=always` 또는 오류 스택 트레이스 등 색상 코드가 포함된 도구 출력 | `[31m` 같은 코드 문자열이 평문 노출되지 않고, 텍스트가 깔끔하게 렌더링되거나 실제 색상으로 매핑됨 |
| API Key 입력 마스킹 | 설정 마법사나 팝업에서 새로운 API 키 입력 | 평문이 뷰에 나타나지 않고 `*` 문자로 치환되어 렌더링됨 |
| 채팅 컨텍스트 Sliding Window | 임계치 이상의 반복적인 도구/채팅 메시지 생성 | 전체 기록이 무한정 메모리에 쌓이지 않고, 자동으로 요약되거나 오래된 항목이 제거되어 RAM 점유율이 안정됨 |

---

## Phase 22: v1.4.0 Production Hardening (시스템 안정화 및 프로덕션 폴리싱)

### 22-A Data Integrity & System 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| 설정 저장 원자성(Atomicity) | `config_store.rs` 저장 중 프로세스 강제 종료(Kill) 시도 | 기존 설정 파일(`settings.json`)이 손상(Corrupted)되지 않고 유지되며, 임시 파일만 생성/삭제됨 |
| Graceful Shutdown (SIGINT) | 도구 실행 중이거나 이벤트 대기 중 `Ctrl+C` 입력 | 패닉 시와 동일하게 터미널이 안전하게 복구되고, 자식 프로세스가 정리된 후 앱이 종료됨 |

### 22-B Streaming & Optimization 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| ANSI 시퀀스 분절 처리 | 도구의 비동기 스트리밍 출력 버퍼를 아주 작게 줄이고 색상 코드 출력 | 버퍼 경계에서 `\x1b[` 등이 잘려도 렌더링 시 깨진 문자가 보이지 않고 완벽한 스트림으로 결합됨 |
| 가로 방향 라인 래핑 성능 | 10만 자 이상의 미니파이된 JSON/JS 코드 출력 렌더링 | Soft Wrap에 의한 CPU 스파이크가 발생하지 않고, 지연 없이 TUI 인스펙터 창이 즉각 반응함 |
| 정교한 토큰 계산 추정 | 한글, 특수문자, 영문이 혼합된 장문 대화 진행 | 단순 글자 수 기반이 아닌 가중치 기반으로 계산되어 실제 LLM 토큰 수량과의 오차율이 10% 내외로 유지됨 |

---

## Phase 23: v1.5.0 Final Refinement (시스템 고도화 및 최종 품질 보증)

### 23-A Robustness & Scalability 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| 터미널 리사이즈 가드 | TUI 실행 중 창 크기를 80x24 미만으로 축소 | 패닉 없이 경고 화면("터미널 크기가 너무 작습니다")이 정상 노출되며, 복구 시 레이아웃이 정상 렌더링됨 |
| 잘못된 도구 호출 복구 | LLM에게 `{"invalid": "json"` 등 고의로 깨진 형식 전송 시뮬레이션 | 시스템이 크래시되지 않고 LLM에게 "Invalid JSON format" 에러 피드백을 주어 재시도를 유도함 |
| RepoMap 스캔 제한 | `target`, `node_modules`가 포함된 깊이 10 이상의 폴더에서 스캔 실행 | `max_depth` 제한 및 Ignore 규칙이 적용되어 2초 이내에 스캔이 완료됨 |

### 23-B System & Infra 감사
| 항목 | 검증 정법 | 합격 기준 |
|------|-----------|-----------|
| 서브 프로세스 타임아웃 | `sleep 100` 도구 실행 시도 | 기본 타임아웃(30초) 도달 시 프로세스에 SIGKILL이 전송되고 런타임이 블로킹에서 벗어남 |
| 세션 로그 로테이션 | 10MB 이상의 세션 로그 파일을 반복 기록 시도 | 기존 로그 파일이 `.1` 등의 백업으로 롤오버되며, 최대 5개까지만 유지됨 |

---

## Phase 24: v1.6.0 Final Integrity Hardening (시스템 무결성 확정 및 최종 고도화)

### 24-A Security & Consistency 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| 인터렉티브 쉘 블로킹 방지 | `git commit` 등 사용자 입력을 대기하는 명령어 실행 | 터미널이 응답 없이 멈추지 않고, 입력 대기 없이 즉시 에러 상태로 프로세스가 종료됨 |
| 민감 정보 마스킹 | 도구로 `echo $GEMINI_API_KEY` 등을 실행하여 API 키 출력 시도 | 화면 및 파일 로그에 실제 키 대신 `[REDACTED]`로 치환되어 저장/표출됨 |
| RepoMap 동적 갱신 | `WriteFile` 도구로 새 파일 생성 직후 `RepoMap` 관련 도구 없이 바로 채팅 질의 | 파일 시스템 변경 시 `dirty` 플래그가 켜져 다음 턴에 자동으로 `RepoMap`이 최신화되고 LLM이 이를 인지함 |

### 24-B Intelligence & Architecture 감사
| 항목 | 검증 정법 | 합격 기준 |
|------|-----------|-----------|
| 스마트 컨텍스트 요약 | 메시지가 토큰 한계를 초과하여 컨텍스트 압축이 발생하도록 긴 대화 진행 | 시스템 가이드와 초기 목표는 삭제되지 않고 유지되며, 삭제된 구간의 요약본이 삽입됨 |
| Provider Mocking 구조 | `cargo test` 명령어를 오프라인 상태에서 실행 | API 서버 연결 없이도 `MockProvider`를 통해 ChatRuntime과 ToolRuntime 로직이 성공적으로 테스트됨 |

---

## Phase 32: v2.4.0 Final Release Candidate 감사 기준 (진행 완료)

### 32-A Performance & Tools 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| 다중 도구 비동기 실행 | LLM이 여러 개의 읽기 전용 도구를 제안 시 | 병렬 큐 시스템(`VecDeque`)을 통해 비동기적으로 도구가 동시 실행되며, Timeline이 개별 업데이트됨 |
| 쓰기 작업 순차 제어 (Write Lock) | 쓰기 관련 도구(WriteFile 등) 포함 여러 도구 제안 | 쓰기 도구는 `write_tool_queue`에 들어가며 이전 작업이 끝나기 전까지 블로킹됨 |
| CLI 자동 완성 | `smlcli completions bash` 실행 | `clap_complete`가 정상적으로 셸 스크립트를 stdout으로 출력함 |

### 32-B UX & Reliability 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| TUI Help Overlay | `F1` 키 입력 | 현재 포커스된 창(Composer, Timeline 등)에 따라 적절한 컨텍스트 단축키 모달이 오버레이됨 |
| Silent Health Check | 설정 파일의 API 키나 모델을 고의로 훼손한 후 실행 | 앱 기동 시 백그라운드로 `Doctor`가 돌며, 실패 시 우측 상단이나 하단에 Toast로 경고가 노출됨 |
| 취약점 점검 패치 | `cargo audit` 결과 확인 | rand 및 의존성 업데이트가 완료되어 취약점 경고 0건 |

---

## Phase 35: v2.5.0 System Hardening & Metadata 감사 기준 (진행 완료)

### 35-A Process & Log Reliability 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| 고아 프로세스 정리 | `ExecShell` 로 긴 sleep을 주고 강제 종료(`kill -9 smlcli`) 후 재실행 시도 | 부모를 잃은 이전 `sleep` 프로세스가 자동으로 `reap_orphans`에 의해 색출되어 종료됨 |
| 세션 복원 OOM 방어 | 50MB 이상의 거대한 세션 `logs/` 파일을 생성하고 TUI 재실행 | `BufReader::lines`를 통해 OOM 발생 없이 안정적으로 파싱 및 불러오기가 성공함 |
| 동시성 정렬 무결성 | 병렬 도구를 동시에 3개 실행시키고, 고의로 실행 지연 시간에 차이를 둠 | 먼저 끝난 순서가 아니라 원래 요청된 도구 인덱스 순서대로 타임라인에 블록이 렌더링됨 |

### 35-B UX Locale & Build DevOps 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| 어댑티브 ASCII Fallback | `LANG=C smlcli` 실행 혹은 `use_ascii_borders=true` 적용 후 TUI 진입 | 유니코드 테두리 대신 `+`, `-`, `|` 기반의 안전한 ASCII 박스가 모든 레이아웃에 깨짐 없이 그려짐 |
| 빌드 메타데이터 증명 | `smlcli doctor` 커맨드 실행 | 보고서 상단에 `v2.5.0 (커밋해시 - 빌드날짜)`가 정상적으로 노출됨 |

---

## Phase 40: v3.0.0 Git-Native Integration 감사 기준 (완료)

### 40-A Git 자동 커밋 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| GitEngine 워킹트리 감지 | `git init` 있는/없는 폴더에서 실행 | Git 리포 없는 경우 Noop, 있는 경우 자동 커밋 가능 |
| Auto-Commit 트리거 | WriteFile/DeleteFile 도구 성공 후 상태 확인 | `git_integration.auto_commit=true` 설정 시 affected_paths만 stage + 커밋 |
| `/undo` 되돌리기 | 자동 커밋 후 `/undo` 입력 | `git reset --hard HEAD~1`으로 마지막 커밋 취소, 타임라인에 롤백 알림 |
| Inspector Git 탭 | F2 인스펙터 열기 후 Git 탭 확인 | 현재 브랜치, 최근 커밋 목록, diff 요약이 표시됨 |

---

## Phase 41: v3.1.0 Custom Provider 확장 감사 기준 (완료)

### 41-A 커스텀 Provider 등록 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| ProviderKind::Custom | `domain/provider.rs` 확인 | `Custom` 변형이 존재하며 base_url, auth_strategy 포함 |
| `/provider add/remove/list` | 각 명령어 실행 | 설정 파일에 영속화되며 재시작 없이 목록 반영 |
| Auth Strategy | 커스텀 Provider로 요청 전송 | Bearer/X-API-Key/Custom 헤더가 정확히 적용됨 |

---

## Phase 42: v3.2.0 OS-Level Sandbox 감사 기준 (완료)

### 42-A Sandbox 격리 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| bubblewrap 래퍼 | `sandbox.enabled=true` 후 ExecShell 실행 | `bwrap` 프로세스 내부에서 명령 실행, 워크스페이스 외부 접근 차단 |
| `/config` Sandbox 섹션 | 설정 UI에서 Sandbox 토글 | enabled/allow_network/extra_binds 3개 필드 노출 |
| 비-Linux 폴백 | macOS/Windows에서 sandbox 활성화 | 경고 메시지 출력 후 일반 모드로 폴백 |

---

## Phase 43: v3.3.0 MCP 클라이언트 인프라 감사 기준 (인프라 완료)

### 43-A MCP JSON-RPC 2.0 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| McpClient::spawn | mock_mcp_server.py로 E2E 테스트 | initialize → tools/list → tools/call 왕복 성공 |
| 네임스페이스 정규화 | 특수문자 포함 서버명/도구명 | `mcp_{sanitized_server}_{sanitized_tool}` 64자 제한 준수 |
| isError 우선 검사 | CallToolResult에 isError:true+content 동시 존재 | isError를 먼저 검사하여 Err 반환 |
| PermissionEngine mcp_ Ask | mcp_ 접두사 도구 실행 시도 | 신뢰 설정과 무관하게 항상 Ask 반환 |
| Child 프로세스 정리 | shutdown() 호출 후 프로세스 목록 확인 | kill()로 자식 프로세스 즉시 종료 |
| 충돌 해소 (Dedup) | 동일 정규화 결과 도구 2개 등록 | 두 번째 도구에 `_2` suffix 자동 부여 |
| 스키마 동기화 가드 | mcp_tools_cache와 mcp_tool_name_map 키 수 | 양쪽의 mcp_ 항목 수가 정확히 일치 |

---

## Phase 44: v3.4.0 DeleteFile 및 TECH-DEBT 정리 감사 기준 (완료)

### 44-A DeleteFile 도구 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| GLOBAL_REGISTRY 등록 | `get_tool("DeleteFile")` 호출 | Tool trait 구현체 반환, 스키마 유효 |
| 샌드박스 검증 | 워크스페이스 외부 경로 삭제 시도 | validate_sandbox()에서 Deny 반환 |
| 디렉토리 삭제 방지 | 디렉토리 경로 전달 | "디렉토리는 삭제 불가" 에러 반환 |
| is_destructive() | DeleteFile의 is_destructive() 호출 | true 반환, Git 체크포인트 트리거 |

### 44-B TECH-DEBT 정리 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| 전역 allow(dead_code) | `grep -r "allow(dead_code)" src/` | ProviderRegistry 1건(cfg(test) 사유)만 잔존, 나머지 0건 |
| 빌드 경고 | `cargo build` | dead_code 관련 경고 0건 |

---

## Phase 45: v3.5.0 CI/CD 파이프라인 감사 기준 (완료)

### 45-A CI 워크플로 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| ci.yml 품질 게이트 | GitHub Actions 로그 확인 | fmt → clippy → test 순서 실행, 하나라도 실패 시 전체 실패 |
| version-sync job | Cargo.toml ↔ CHANGELOG 불일치 push | CI에서 실패 감지 |
| cargo cache | 두 번째 실행 시간 비교 | 캐시 적중으로 의존성 다운로드 스킵 |

### 45-B Release 워크플로 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| release.yml 태그 트리거 | `v3.5.0` 태그 push | quality-gate → 크로스 빌드 → Releases 업로드 |
| Linux musl 바이너리 | 릴리스 자산 확인 | `smlcli-x86_64-unknown-linux-musl` 존재 |
| Windows msvc 바이너리 | 릴리스 자산 확인 | `smlcli-x86_64-pc-windows-msvc.exe` 존재 |

---

## Phase 46: v3.6.0 Workspace-scoped Session Management 감사 기준 (완료)

### 46-A 세션 영속화 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| SessionMetadata | `domain/session.rs` 확인 | id, title, workspace_root, created_at, updated_at 필드 존재 |
| SessionIndex CRUD | `infra/session_log.rs` 확인 | sessions_index.json 파일 생성/읽기/갱신/삭제 API 존재 |
| 워크스페이스 격리 | 다른 폴더에서 `/session` 실행 | 해당 폴더의 세션만 목록에 노출 |

### 46-B 세션 명령어 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Auto-Titling | 첫 메시지 전송 후 `/session` 확인 | 메시지 앞 50자가 세션 제목으로 자동 설정 |
| `/resume <번호>` | 이전 세션 번호로 resume | 메시지 복원 + 로거 교체 + 인덱스 touch |
| `/new` | 세션 진행 중 `/new` 입력 | 타임라인·세션 상태·스트림 초기화 후 새 세션 할당 |
| `/session` | 명령어 실행 | 현재 워크스페이스의 세션 목록이 KeyValueTable로 렌더링 |
| SlashMenu 등록 | `/` 입력 후 메뉴 확인 | `/session`, `/resume`, `/new` 3건이 메뉴에 노출 |

---

## Phase 47: v3.7.0 Interactive Planning Questionnaire 감사 기준 (완료)

### 47-A AskClarification 도구 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| GLOBAL_REGISTRY 등록 | `get_tool("AskClarification")` 호출 | Tool trait 구현체 반환, 스키마의 function.name이 "AskClarification" |
| 스키마 구조 | schema() JSON 확인 | questions 배열 (id/title/options/allow_custom), summary 문자열 파라미터 존재 |
| check_permission | 임의 인자로 호출 | 항상 PermissionResult::Allow (읽기 전용 도구) |
| PLAN 모드 하네싱 | chat_runtime.rs 시스템 프롬프트 확인 | PLAN 모드에서 AskClarification 강제 사용 지침 존재 |

### 47-B Questionnaire TUI 위젯 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| 모달 오버레이 | layout.rs draw() 확인 | help_overlay 뒤에 questionnaire 오버레이 렌더링 |
| 객관식 커서 | ↑↓ 키 입력 후 확인 | ▸ 마커가 현재 옵션에 표시, Cyan 하이라이트 |
| 주관식 입력 | 빈 옵션 질문에서 문자 입력 | ▏ 커서 표시, Backspace 삭제 동작 |
| allow_custom | allow_custom=true 옵션 선택 | "✏ 직접 입력..." 옵션이 목록 끝에 노출, 선택 시 텍스트 입력 모드 전환 |
| 진행률 표시 | 다중 질문 폼 진행 | "질문 N/M" 형식으로 현재 진행 상태 표시 |
| Esc 취소 | Esc 입력 | 설문 취소, questionnaire 상태 None으로 복원, 취소 ToolResult 전달 |

### 47-C State Machine 연동 감사
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| ShowQuestionnaire Action | tool_runtime.rs 인터셉트 확인 | AskClarification 도구명 감지 시 비동기 실행 대신 Action 발행 |
| QuestionnaireCompleted Action | 모든 답변 완료 후 확인 | ToolResult 조립 (JSON answers) → ToolFinished로 LLM 피드백 |
| UiState.questionnaire | state.rs 확인 | Option<QuestionnaireState> 필드 존재, None 초기화 |
| handle_questionnaire_key | mod.rs 키 입력 핸들러 확인 | is_some() 일 때 최우선 인터셉트, ↑↓/Enter/Esc/문자입력 라우팅 |

### 47-D MCP E2E 테스트 완성도 감사 (Task M-4)
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Mock MCP 서버 | scripts/mock_mcp_server.py 존재 | JSON-RPC 2.0 initialize/tools/list/tools/call 응답 |
| E2E initialize+list | `cargo test test_mcp_e2e_initialize_and_list_tools` | 2도구 반환, 스키마 검증 통과 |
| E2E call_tool | `cargo test test_mcp_e2e_call_tool` | get_weather/read_file 응답 내용 일치 |
| Permission Ask 강제 | `cargo test test_mcp_permission_engine_always_ask` | mcp_ 접두사 도구 2건 모두 Ask |
| 네임스페이스 왕복 | `cargo test test_mcp_namespace_strip_roundtrip` | sanitize → 합성 → 역매핑 복원 일치 |
| 설정 CRUD | `cargo test test_mcp_config_add_remove_persistence` | Vec push/upsert/retain 동작 |
| AskClarification 등록 | `cargo test test_ask_clarification_tool_registered` | GLOBAL_REGISTRY + 스키마 + Allow |
| Questionnaire 로직 | `cargo test test_questionnaire_state_submit_and_build` | 3문항 순차 답변 → build_result 조립 |
| total_options 계산 | `cargo test test_questionnaire_total_options` | allow_custom 포함/미포함 옵션 수 정확 |
| 전체 테스트 수 | `cargo test` | 102건 이상 통과 |

---

## Phase 48: v3.7.1 1st & 2nd Audit Remediation 감사 기준 (완료)
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| GrepSearch 샌드박스 차단 | `grep.rs` 소스 및 회귀 테스트 확인 | `file_ops::validate_sandbox()`를 통해 외부 경로(`/etc`, `../`) 탐색 원천 차단 및 `test_grep_search_sandbox_bypass` 통과 |
| Approval 큐 고립 방지 | `mod.rs` 및 `test_approval_timeout_promotes_queue` 확인 | 타임아웃 만료 시 큐에서 제거되고 다음 대기 중인 승인이 정상적으로 팝업 승격됨 |
| MCP 펜딩 맵 관리 | `McpClient` 소스 확인 | `pending_requests` 맵이 구조체 필드로 존재, 10초 타임아웃 및 EOF 시 펜딩 해제 |
| TUI 테마 색상 정책 통일 | `questionnaire.rs`, `help_overlay.rs` 확인 | 하드코딩된 Color 대신 `state.palette()`에서 가져온 테마 색상 적용 |
| 직접 셸 에러 재전송 방어 | `tool_runtime.rs` `ToolError` 분기 확인 | `tool_call_id` 누락 시 LLM 재전송 큐(`pending_tool_outcomes`)에 넣지 않고 드롭 |

## Phase 49: v3.7.2 3rd Audit Remediation 감사 기준 (완료)
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| 헤드리스 가상 터미널 격리 | `src/app/mod.rs` `handle_mouse` 확인 | `cfg!(test)` 플래그 감지 시 가상 TTY 윈도우 크기를 `(100, 30)`으로 강제 하이재킹 모킹(Mocking)하여 클릭/스크롤 좌표 단언문 격리 확보 |

## Phase 50: v3.8.0 LM Studio 공식 프로바이더 및 위저드 Fallback 감사 기준 (완료)
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| LmStudio 프로바이더 지원 | `src/domain/provider.rs` 및 `registry.rs` 확인 | `ProviderKind::LmStudio`가 정식 등록되었으며, 기본 URL(`http://localhost:1234/v1`)이 영속화 및 매핑됨 |
| 위저드 Base URL 입력 분기 | `src/app/wizard_controller.rs` 확인 | LM Studio 선택 시 API Key 입력을 생략하고 `BaseUrlInput` 단계를 중간 삽입하여 동작 |
| 실시간 어댑터 URL 동기화 | `src/providers/registry.rs` `update_lmstudio_base_url` 확인 | TUI 위저드 입력값에 기반해 `ProviderRegistry` 내 static RwLock `lmstudio` 어댑터의 base_url이 실시간 갱신됨 |
| 수동 직접 입력 Fallback UI | `src/tui/widgets/setting_wizard.rs` 확인 | API 핑`/models`의 로딩 및 실패 여부에 관계없이 목록 최하단에 항상 `"✏ 직접 입력..."`을 제공하고, 선택 시 전용 수동 입력 모달(`is_custom_model_mode`) 전개 |
| E2E 통합 회귀 테스트 통과 | `cargo test test_lm_studio_wizard_flow` | LmStudio 선택 -> Base URL 입력 -> API 핑 실패 시 수동 입력 -> 최종 settings 저장 영속화 과정이 무결하게 검증되며, 전체 105개 테스트 무결 통과 |

## Phase 51: v3.8.1 LM Studio 런타임 연동 및 무인증 예외 처리 감사 기준 (완료)

### 51.1 감사 항목 및 합격 검증 기준표
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| LM Studio 런타임 크레덴셜 예외 | `src/app/chat_runtime.rs` `resolve_credentials` 확인 | LmStudio 프로바이더의 경우 API Key 조회 및 에러 처리를 스킵하고 빈 문자열(`""`)로 즉시 통과 처리 |
| LM Studio 프로바이더 연동 가드 | `src/app/chat_runtime.rs` `resolve_credentials_for_provider` 확인 | LmStudio에 대해 `needs_key` 플래그를 `false`로 제어하여 암호 저장소 키 수집 단계를 우회 |
| 대시보드 리스트 연동 | `src/tui/widgets/config_dashboard.rs` 확인 | `/config` 대시보드 내 "Select Provider" 목록에 `"LM Studio"` 문자열 옵션이 정상 렌더링됨 |
| 대시보드 인덱스 교정 | `src/app/wizard_controller.rs` 및 `mod.rs` 확인 | 대시보드 키 핸들러 `idx => 5` 분기 처리, custom_providers 오프셋 보정(`saturating_sub(6)`), 팝업 네비게이션 최대값 상향(`5 + ...`) 정상 반영 |

### 51.2 감사 검증 명령어 및 출력 결과 샘플 (Execution Path)
감사자는 터미널에서 아래 통합 테스트 검증 명령어를 수행하고 100% 성공 여부를 단언 판단해야 합니다.

```bash
# 1. 핫픽스 패치에 관한 단위/통합 테스트 집중 실행
cargo test tests::audit_regression::test_lm_studio_wizard_flow

# 2. 전체 회귀 테스트 스위트 105개 전체 실행 및 환경 노이즈 격리 여부 확인
cargo test
```

#### 정상 통과 시 터미널 출력 결과 샘플 (Standard Output Spec)
```
running 105 tests
.........................................................................................................
test result: ok. 105 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
```

### 51.3 잔여 리스크 (Residual Risks)
* **로컬 LLM 포트/서버 설정 불일치 리스크**:
  - **내용**: LM Studio가 실행 중인 로컬 포트가 기본값(`1234`)이 아니거나, 런타임 중에 서버가 다운되는 경우 챗 연동이 거부됩니다.
  - **대응책**: 설정 마법사 Step 2 단계에서 포트 편집 편집기를 유연하게 제공하고, 검증 실패 시 즉시 에러 알림 카드와 수동 모델명 직접 입력(`✏ 직접 입력...`) 카드를 제공해 마법사를 중단 없이 이어가도록 조치했습니다.
* **TUI 위젯 최소 크기 초과 시 오버플로우 리스크**:
  - **내용**: 팝업 크기 상향(`MAX_HEIGHT = 6`)으로 인해 가로 80칸, 세로 24칸 미만의 초소형 터미널 창에서 `/config`를 구동할 시 UI 패널이 깨질 위험이 있습니다.
  - **대응책**: 터미널 최소 권장 해상도(80x24) 미만 진입 시 UI 렌더링 오류 피드백 카드를 화면 최상단에 안전하게 그리도록 예외 렌더러가 설정되어 있어, 오버플로우 충돌을 미연에 방지합니다.

---

## Phase 52: v3.9.0 TUI Modernization & Responsive Multi-viewport Redesign 감사 기준 (완료)

### 52.1 감사 항목 및 합격 검증 기준표
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| RGB 고스케일 세맨틱 팔레트 | `src/tui/palette.rs` 및 `designs.md` 확인 | 11대 RGB 고스케일 세맨틱 팔레트 명세(`bg_lowest`, `bg_panel`, `outline`, `text_primary`, `text_secondary` 등)가 완전 매핑되고 하드코딩 색상이 제거됨 |
| 다국어 5대어 실연결 | `src/tui/i18n.rs` 및 설문조사 위젯 번역 확인 | `I18nManager` 내 설문조사 전용 다국어 키 매핑(`question_progress`, `input_prompt`, `custom_input_prompt`, `input_hint` 등)이 올바르게 통합 및 tr() 바인딩 보장됨 |
| 반응형 3분할 및 Tree of Thoughts 트리 | `src/tui/layout.rs` 소스 확인 | `unicode-width` 기반 래핑 처리, `Borders::LEFT` 2px 세로바(`┃`) 적용, `└─ ⚙️` 회색 깊이 들여쓰기 트리 구조 렌더링이 정확하게 구현됨 |
| 단축키 인스펙터 탭 및 Diff 캐시 가속 | `src/tui/widgets/inspector_tabs.rs` 확인 | `Alt+1` ~ `Alt+6` 단축키 바인딩 인스펙터 탭바 연동, `DIFF_RENDER_CACHE` thread_local 5000라인급 Diff 라인 캐싱 가속 구현 완료 |
| Composer Inset & Fuzzy Command Palette | `src/tui/layout.rs` 내 커서 렌더링 확인 | `draw_composer`에 `bg_lowest` 배경과 보라색 `❯` 프롬프트 적용, `draw_command_palette` 오버레이에 `bg_panel` 스타일 및 `Clear` 위젯 소거 연동. `tick_count` 기반 500ms 깜빡임 `█` 커서 점멸 연동 완료 |
| 설문조사 모달 수학적 정렬 | `src/tui/widgets/questionnaire.rs` 확인 | 가로 60%, 세로 45% 수학적 중앙 정렬 공식 보정 및 `Clear` 잔상 소거 적용 완료 |
| Clippy warning 및 빌드 무결성 | `cargo clippy --all-targets -- -D warnings` 및 `cargo build --release` | 0 warnings, 0 errors로 완벽하게 빌드 및 무결 통과함 |

### 52.2 감사 검증 명령어 및 출력 결과 샘플 (Execution Path)
감사자는 터미널에서 아래 clippy, 빌드, 및 테스트 명령어를 수행하여 100% 무결성을 정밀 확인해야 합니다.

```bash
# 1. Clippy 경고 엄격 필터링 실행
cargo clippy --all-targets -- -D warnings

# 2. Release 프로필 빌드 무결성 검증
cargo build --release

# 3. 전체 회귀 테스트 스위트 105개 전체 실행
cargo test
```

#### Clippy 및 빌드 정상 완료 시 출력 결과 샘플 (Standard Output Spec)
```
   Compiling smlcli v3.9.0 (/mnt/Projects_SSD/rust/smlcli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.25s
```

```
   Compiling smlcli v3.9.0 (/mnt/Projects_SSD/rust/smlcli)
    Finished `release` profile [optimized] target(s) in 21.11s
```

### 52.3 잔여 리스크 (Residual Risks)
* **대용량 Diff 라인 캐시 무효화 정합성 리스크**:
  - **내용**: 5000라인 이상의 대용량 Diff 파싱 캐싱 중, `approval.diff_preview` 값이 런타임에 미세하게 변경되거나 캐시 만료 조건이 꼬여 이전 diff 잔상이 유지될 수 있습니다.
  - **대응책**: `DIFF_RENDER_CACHE` thread_local은 단순히 diff 텍스트 참조값(`*diff`)을 기준으로 완격히 동등 여부(`==`)를 비교하여 다를 경우 즉시 새로운 파싱 결과를 무효화(invalidate)하고 덮어씌우는 메커니즘을 이식하여 데이터 불일치 위험을 완벽히 격리했습니다.
* **100 cols 미만의 반응형 탭 전환 오인 리스크**:
  - **내용**: 화면 폭이 좁은 가상 터미널 환경에서 인스펙터 서브 드로어가 닫혔을 때, 탭 단축키(`Alt+1`~`Alt+6`) 입력 시 탭 상태 전이가 제대로 동작하지 않는 것처럼 오해할 수 있습니다.
  - **대응책**: 탭 상태는 내부적으로 항상 동기화되어 인스펙터 활성화 시 즉각 반영되도록 설계되었으며, Composer 하단 도움말에 탭 상태 인디케이터가 명확히 명시되도록 보강 조치했습니다.

---

## Phase 53: v3.9.1 Workspace Harness Snapshot & OS/Sandbox 정합화 감사 기준

### 53.1 감사 항목 및 합격 검증 기준표
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Snapshot 단일 계약 | `src/infra/workspace_harness.rs` 확인 | OS, arch, host_shell, exec_shell, canonical_root, trust_state, denied, extra_workspace_dirs, sandbox 필드가 하나의 구조체로 닫힘 |
| canonical trust key | `src/domain/permissions.rs`, `/workspace` 라우터 확인 | raw `current_dir()` 대신 canonical workspace root 문자열로 trust/deny를 조회 |
| Linux guest root | `src/infra/sandbox.rs` 및 테스트 확인 | `bwrap`가 host cwd를 `/workspace`에 bind하고 `--chdir /workspace`로 실행 |
| doctor 하네싱 진단 | `cargo run --quiet -- doctor` | `Workspace Harness 상태` 섹션에 OS, shell, root, trust, sandbox backend, guest root, network 정책 출력 |
| `/workspace show` 진단 | TUI 명령 테스트 또는 슬래시 명령 실행 | 출력에 `Workspace Harness`와 `Sandbox Guest Root: /workspace` 포함 |

### 53.2 감사 검증 명령어 및 출력 결과 샘플 (Execution Path)
```bash
cargo test workspace --all-targets --no-fail-fast
cargo test sandbox --all-targets --no-fail-fast
cargo run --quiet -- doctor
cargo fmt --check
cargo check --all-targets
cargo test --all-targets --no-fail-fast
cargo clippy --all-targets --all-features -- -D warnings
git diff --check
```

#### 정상 출력 샘플
```text
--- Workspace Harness 상태 ---
OS: linux (x86_64)
Host Shell: /bin/bash
Exec Shell: sh (bwrap:/workspace)
Workspace Root: /mnt/Projects_SSD/rust/smlcli
Trust Level: Trusted
Denied: false
Sandbox Enabled: true
Sandbox Backend: bubblewrap
Sandbox Guest Root: /workspace
Sandbox Network: isolated
```

### 53.3 잔여 리스크 (Residual Risks)
* **비-Linux 파일시스템 격리 parity 리스크**:
  - **내용**: Windows/macOS에서는 Linux `bwrap`와 같은 mount namespace 기반 격리가 아직 없다.
  - **대응책**: 이번 Phase에서는 doctor가 backend `none`을 명시적으로 표시하고, Windows AppContainer/Job Object 파일시스템 격리는 별도 Phase로 분리한다.

---

## Phase 54: v3.9.2 Workspace Harness Enforcement & Model Grounding 감사 기준 (완료)

### 54.1 감사 항목 및 합격 검증 기준표
| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Prompt harness 주입 | chat request system prompt 확인 | OS, arch, host shell, exec shell, canonical root, trust, sandbox 정보가 12줄 이하 block으로 중복 없이 1회 포함 |
| Tool preflight | `ExecShell`/쓰기 도구 실행 전 decision 테스트 | workspace 밖 `cwd`/path, denied root, Unknown/Restricted trust가 `Deny` 또는 `Ask`로 승격 |
| Session snapshot 기록 | 새 세션 JSONL 또는 metadata 확인 | session id와 함께 OS/root/trust/sandbox snapshot이 첫 메타 블록에 저장 |
| Sandbox 비활성 문구 구분 | `smlcli doctor`, `/workspace show` 출력 확인 | `Sandbox Enabled=false`일 때 `/workspace`가 현재 mount가 아니라 정책상 guest root임을 명시 |
| OS mismatch 감지 | Linux/Windows 명령 패턴 테스트 | Linux에서 PowerShell 전용 명령, Windows에서 POSIX-only 명령이 즉시 자동 실행되지 않고 Notice/Ask로 승격 |

### 54.2 감사 검증 명령어 및 출력 결과 샘플 (Execution Path)
```bash
cargo test harness_prompt --all-targets --no-fail-fast
cargo test harness_preflight --all-targets --no-fail-fast
cargo test session_harness --all-targets --no-fail-fast
cargo test workspace --all-targets --no-fail-fast
cargo run --quiet -- doctor
cargo fmt --check
cargo check --all-targets
cargo test --all-targets --no-fail-fast
cargo clippy --all-targets --all-features -- -D warnings
git diff --check
```

#### 정상 출력 샘플
```text
[Workspace Harness]
OS=linux arch=x86_64
HostShell=/bin/bash ExecShell=sh
WorkspaceRoot=/mnt/Projects_SSD/rust/smlcli
Trust=Trusted Denied=false
SandboxEnabled=false Backend=bubblewrap GuestRoot=/workspace Network=allowed
Rule: host paths are for local file APIs; sandbox shell cwd is /workspace only when sandbox is enabled.
```

### 54.3 잔여 리스크 (Residual Risks)
* **명령 패턴 오탐 리스크**:
  - **내용**: OS mismatch validator가 일부 합법적인 cross-shell 명령을 Ask로 승격할 수 있다.
  - **대응책**: v1에서는 자동 Deny 대신 Ask를 기본으로 사용하고, 명시적으로 위험한 cwd/path 이탈만 Deny한다.
* **Prompt 중복 리스크**:
  - **내용**: system prompt dedupe 실패 시 harness block이 누적되어 context budget을 낭비할 수 있다.
  - **대응책**: block header `[Workspace Harness]` 기준으로 기존 block을 교체하는 dedupe 테스트를 필수로 둔다.
