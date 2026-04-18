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
| CLI Entry Modes | `main.rs`에 clap 파싱 | `run`, `doctor`, `export-log` 서브커맨드 동작 |
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
| 전역 allow 제거 | `main.rs` 검사 | #[allow(dead_code)] 등 0건 |

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
