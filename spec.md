# smlcli Implementation Spec (v0.1 BETA)

## 0. Global Documentation Rules (Git Policy)

**Priority Over Code**
문서 업데이트는 소스코드 작성보다 우선되는 절대 규칙이다. 구현 전과 구현 후 모두 `spec.md`, `audit_roadmap.md`, `implementation_summary.md`, `designs.md`를 먼저 갱신한다.

**Documentation First Enforcement**
모든 기능 작업은 아래 순서를 반드시 따른다.

1. `spec.md` 반영
2. `designs.md` 반영
3. `audit_roadmap.md` 단계 갱신
4. 구현
5. 테스트 작성 및 실행
6. `implementation_summary.md` 기록

---

## 1. Project Identity & Versioning

**Project Name**
smlcli

**Current Version**
v0.1 BETA

**Status**
Initial Specification & Implementation Entry

**Target Environment**
Google Antigravity

**Target Platform**
Desktop CLI for Linux + Windows

**Primary Goal**
`smlcli`는 Codex/OpenCode 계열의 사용감을 갖는 터미널 중심 AI 에이전트 CLI다. 앱 실행 시 기본적으로 TUI에 진입하며, `/setting`을 통해 provider, API key, model, 권한 정책을 동적이고 직관적인 화살표 키 조작만으로 순차적(Sequential)으로 설정한다. AI 연결 후에는 프롬프트 기반으로 터미널 작업, 파일 읽기, 파일 쓰기, 명령 실행, grep, diff를 수행한다.

**Reference Philosophy**
Antigravity에 맞는 문서 우선·검증 우선 워크플로를 따르며, UI 상호작용 패턴은 TUI 기본 진입, slash command, provider 연결, run 모드, 모델 식별자 규칙을 참고하되 구현은 Rust 네이티브 아키텍처로 새로 작성한다.

---

## 2. Environment & Tech Stack

### 2.1 Grounding Rationale

이 프로젝트는 Linux와 Windows를 함께 지원해야 하고, 방향키·Enter·ESC 중심의 TUI 조작이 핵심이다. 현재 설계 방향은 `ratatui` 기반 UI, `crossterm` 기반 터미널 이벤트 처리, `tokio` + `reqwest` 기반 비동기 provider 호출, 파일 기반 암호화 저장소 (`~/.smlcli/`, ChaCha20Poly1305), `grep` + `ignore` 기반 검색, `similar` 기반 diff로 구성한다.

### 2.2 Stack Details

**Language**
Rust stable, edition 2024

**Framework / Runtime**
`ratatui`, `crossterm`, `tokio`

**CLI / Config / Serialization**
`clap`, `serde`, `toml`

**HTTP / Provider Layer**
`reqwest`

**Secret Storage / Local Encryption**
`chacha20poly1305` — 파일 기반 마스터 키 (`~/.smlcli/.master_key`) + API 키 암호화 (`~/.smlcli/config.toml`)

**Search / Diff**
`grep`, `ignore`, `similar`

### 2.3 MVP Provider Scope

MVP에서 기본 지원하는 provider는 다음과 같다.

* OpenRouter
* Google (Gemini)
* (차후 OpenAPI 호환 및 기타 provider 추가 예정)

모델 식별자는 내부적으로 반드시 `provider/model` 문자열로 표준화한다.

### 2.4 Reference Behavior

`smlcli`는 아래 동작 패턴을 따른다.

* 인자 없이 실행 시 TUI로 진입
* 별도 `run` 모드 제공
* slash command 기반 UX
* TUI 안에서 provider 연결 및 API key 입력
* 모델 식별자 표준화
* Windows 품질 검증 경로 포함

---

## 3. Project Architecture & UX Design

### 3.1 File Directory Structure (Optimized for Antigravity)

```bash
smlcli/
├── .antigravityrules
├── spec.md
├── audit_roadmap.md
├── implementation_summary.md
├── designs.md
├── README.md
├── CHANGELOG.md
├── BUILD_GUIDE.md
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── main.rs
│   ├── app/
│   │   ├── mod.rs (Event Loop & Top-level Dispatch, ~510줄)
│   │   ├── state.rs (AppState, TimelineEntry, WizardState, ConfigState, FuzzyState)
│   │   ├── event_loop.rs (Crossterm 이벤트 + Action 채널 수신)
│   │   ├── action.rs (14종 비동기 이벤트 타입 정의)
│   │   ├── command_router.rs (슬래시 커맨드 엔진, 11개 커맨드)
│   │   ├── chat_runtime.rs (LLM 요청 조립 & Provider 디스패치 & SSE 스트리밍)
│   │   ├── tool_runtime.rs (도구 JSON 파싱, 권한 검사, 비동기 실행, 출력 요약)
│   │   └── wizard_controller.rs (Setup Wizard 상태 전이 & Config 팝업)
│   ├── tui/
│   │   ├── mod.rs
│   │   ├── terminal.rs
│   │   ├── layout.rs
│   │   ├── palette.rs (Semantic Palette + 테마 전환 API)
│   │   └── widgets/
│   │       ├── mod.rs
│   │       ├── config_dashboard.rs
│   │       ├── setting_wizard.rs
│   │       └── inspector_tabs.rs (Preview/Diff/Search/Logs/Recent 탭 렌더링)
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── session.rs (Context Budget & Compaction logic)
│   │   ├── provider.rs
│   │   ├── settings.rs
│   │   ├── permissions.rs (Permission Engine + Blocked Command List)
│   │   ├── tool_result.rs
│   │   └── repo_map.rs (Tree-sitter AST Context)
│   ├── providers/
│   │   ├── mod.rs
│   │   ├── registry.rs
│   │   └── types.rs
│   ├── tools/
│   │   ├── mod.rs
│   │   ├── registry.rs (Polymorphic Tool Registry)
│   │   ├── git_checkpoint.rs (Auto-healing & State rollback)
│   │   ├── file_ops.rs
│   │   ├── shell.rs
│   │   ├── grep.rs
│   │   ├── sys_ops.rs
│   │   └── executor.rs
│   ├── infra/
│   │   ├── mod.rs
│   │   ├── config_store.rs
│   │   ├── secret_store.rs
│   │   └── session_log.rs (JSONL 세션 영속성 — 동기/비동기 이중 API)
│   ├── tests/
│   └── types/
│       └── mod.rs
└── assets/
    └── examples/
```

### 3.2 Runtime Architecture

애플리케이션은 단일 `AppState`를 중심으로 동작한다. 입력 이벤트, 렌더 요청, AI 응답, 툴 결과, modal 상태를 모두 `Action` 단위로 정규화한다. 이벤트 루프는 TUI 렌더링과 비동기 작업을 분리하되, 사용자에게는 하나의 연속된 터미널 경험처럼 보이게 유지한다.

**이중 데이터 모델 (Dual Data Model)**
`session.messages`는 LLM 컨텍스트 전용이며, 사용자 화면 표시는 `timeline: Vec<TimelineEntry>`로 분리한다. 이 분리를 통해 도구 실행 요약 카드, 승인 카드, 실행 로그, 결과 요약을 독립적으로 관리한다. timeline이 비어있을 때만 session.messages 폴백을 허용한다(하위 호환).

**이벤트 세분화 (14종+ Action)**
채팅과 도구 호출의 전체 라이프사이클(시작·진행·완료·에러)을 별도 이벤트로 정규화하여 Codex 스타일 진행 표시를 구현한다.

**도구 호출 격리 계층 (v0.1.0-beta.22)**
LLM 응답에서 도구 호출을 감지할 때 3단계 필터를 적용한다:
1. **bare JSON 차단**: fenced(`\`\`\`json`)가 아닌 raw JSON 객체는 도구로 인식하지 않는다.
2. **`"tool"` 키 검증**: fenced JSON 블록 내에 `"tool"` 필드가 존재해야만 도구 후보로 취급한다.
3. **ToolCall 역직렬화 + 빈 명령 차단**: serde 역직렬화 성공 후에도 `ExecShell.command.trim().is_empty()`이면 즉시 거부한다.

**첫 턴 자연어 가드 (v0.1.0-beta.22)**
시스템 프롬프트에 다음 정책을 명시한다:
- 첫 응답은 반드시 자연어 인삿말/확인으로 시작하며, 도구를 사용하지 않는다.
- 인삿말, 질문, 설명 등 비작업성 입력에는 도구 없이 자연어로만 응답한다.
- 도구 카탈로그는 이름만 나열하며, 필드 스키마와 예시 JSON은 시스템 프롬프트에 포함하지 않는다.

**bare JSON 렌더링 필터 (v0.1.0-beta.22)**
`filter_tool_json()`은 fenced JSON 블록뿐 아니라 bare JSON도 감지한다. `"tool"` 키가 있는 bare JSON은 사용자 친화적 요약으로 대체하여 스키마가 사용자에게 직접 노출되지 않도록 한다.

**PLAN/RUN 모드 행동 계약 (v0.1.0-beta.22)**
채팅 요청 시 현재 모드에 따라 LLM에 행동 지시를 주입한다:
- **PLAN 모드**: 분석/설명 위주. 코드를 인라인으로 보여주되 자동 파일 쓰기는 하지 않는다.
- **RUN 모드**: 코드 작성/수정 요청 시 반드시 `WriteFile`/`ReplaceFileContent` 도구를 사용하여 디스크에 기록한다.

**타임라인 스크롤 (v0.1.0-beta.22)**
`UiState::timeline_scroll` 필드와 `PageUp`/`PageDown` 키 바인딩으로 긴 응답을 세로 탐색한다. 위자드, Fuzzy, 설정 팝업이 열려 있을 때는 비활성.

**SSE 스트리밍**
Provider 응답을 토큰 단위로 수신하여 실시간 타임라인 렌더링에 반영한다.

핵심 계층은 아래처럼 분리한다.

* `tui/*`: 화면 그리기와 키 입력 해석, Semantic Palette, Inspector 탭
* `app/*`: 라우팅, 상태 전이, context budget, timeline 관리
* `providers/*`: provider/model 호출 추상화, SSE 스트리밍
* `tools/*`: 파일/셸/grep/diff 등 실행 가능한 도구
* `infra/*`: 저장소, 암호화, OS 상호작용, 세션 로그
* `domain/*`: 순수 상태 모델과 정책

### 3.3 Entry Modes

CLI는 두 가지 진입 모드를 제공한다.

```bash
smlcli
smlcli run "explain this repository"
smlcli doctor
smlcli export-log
```

인자 없이 실행하면 TUI에 진입하고, `run`은 비대화형 1회 실행 모드다.

### 3.4 Main Screen Layout

메인 화면은 네 영역으로 고정한다.

1. **왼쪽 상태 패널**
   현재 provider, model, working directory, 권한 모드, 세션 토큰 예산

2. **중앙 대화 패널**
   사용자 입력, AI 응답, 툴 실행 요약, 오류 메시지

3. **오른쪽 작업 패널**
   파일 프리뷰, grep 결과, diff 프리뷰, 최근 대상 파일

4. **하단 입력창**
   일반 프롬프트 또는 `/` 명령 입력

### 3.5 UX/UI Flow & Interface Verification

#### User Scenario A: 최초 실행

* 사용자가 `smlcli` 실행
* 설정 파일이 없으면 자동으로 `/setting` wizard 진입
* provider 선택
* API key 입력 및 즉시 검증
* model 조회 또는 수동 입력
* 권한 정책 선택
* 저장 후 메인 채팅 화면 진입

#### User Scenario B: 일반 프롬프트

* 사용자가 자연어 프롬프트 입력
* 모델이 적절한 tool call 제안
* 툴 실행 전 권한 검사
* 결과 요약 후 중앙 패널에 응답 표시

#### User Scenario C: 파일 수정

* AI가 수정 전 원본을 읽음
* 수정안을 생성
* `similar` 기반 diff 생성
* 오른쪽 패널에 변경 프리뷰 표시
* 사용자가 `Enter`로 승인하면 원자적 쓰기 수행
* `ESC`로 취소

#### User Scenario D: grep

* AI 또는 사용자가 패턴 검색 요청
* `ignore` 기반 탐색 + `grep` 매칭 수행
* 파일별 결과 목록 표시
* 방향키로 이동
* `Enter`로 해당 파일 프리뷰 열기

#### User Scenario E: shell command

* AI가 명령 실행을 제안
* 정책이 `Ask`이면 확인 modal 표시
* 승인 시 제한된 shell executor가 실행
* stdout/stderr를 실시간 스트리밍
* `ESC` 또는 cancel 키로 중단

### 3.6 Setting Wizard Design (Sequential UX)

`/setting`은 단순 설정 화면이 아니라 마우스 없이 화살표 키보드만으로 끊김없이 완수되는 자동화 흐름이어야 한다.

**Step 1. Provider 선택**

* 항목: OpenRouter / Google (Gemini)
* 조작: 방향키 이동, `Enter` 선택, `ESC` 재진입

**Step 2. 자격 증명 입력**

* API key 입력은 마스킹
* 붙여넣기 허용
* Custom provider는 `base_url`도 함께 입력
* `Enter` 시 즉시 비동기를 통해 해당 Provider의 최신 가용 모델 목록(Models API) 페치(Fetch) 개시
* 실패 시 같은 단계에 머물며 구체적 오류 카드를 출력

**Step 3. Model 선택 (Dynamic Listing)**

* 가능하면 provider에서 받은 동적 모델 목록 조회
* 화면 높이를 고려해 10개씩 페이징/윈도잉 표출 유지
* 저장 시 방향키로 고른 정확한 `provider/model` 조합을 저장

**Step 4. 권한 정책**

* Shell: `Ask` / `SafeOnly` / `Deny`
* File Write: `AlwaysAsk` / `SessionAllow`
* Network: `ProviderOnly` / `Deny`
* Grep/Diff: 항상 enabled

**Step 5. 저장**

* 민감정보는 ~/.smlcli/.master_key 기반 암호화로 저장
* 일반 설정은 암호화된 로컬 파일에 저장
* 저장 성공 전까지 메인 대화는 비활성화

### 3.7 Keyboard Model

기본 조작은 다음과 같다.

* `↑ ↓ ← →`: 목록 이동, 패널 포커스 이동
* `Enter`: 선택, 승인, 확정
* `ESC`: 취소, 뒤로가기, modal 닫기
* `Tab`: 패널 순환
* `Ctrl+C`: 안전 종료
* `/`: slash command 시작
* `:`: 선택적 command palette 진입
* `PgUp/PgDn`: 로그 스크롤

### 3.8 Interface Verification Points

1. **Terminal Cleanup**
   panic, cancel, Ctrl+C, validation error 이후에도 raw mode와 alternate screen이 남지 않아야 한다.

2. **Windows Compatibility**
   공식 지원은 Windows 네이티브로 제공한다. 품질 보증 경로는 `Windows Terminal + PowerShell`과 `WSL`을 둘 다 포함한다.

3. **Validation Gate**
   `/setting` 완료 전에는 AI 호출을 금지한다.

4. **Write Preview**
   모든 파일 쓰기는 diff 프리뷰와 사용자 승인 후에만 반영한다.

5. **Permission Visibility**
   현재 권한 모드는 항상 상태 패널에 노출한다.

### 3.9 Data Models & Interfaces (Logical Equivalence)

```rust
pub struct AppState {
    pub ui: UiState,
    pub settings: SettingsState,
    pub session: SessionState,
    pub permissions: PermissionState,
    pub provider: ProviderState,
    pub tool_runtime: ToolRuntimeState,
    // [v0.1.0-beta.18 개편] 이중 데이터 모델
    pub timeline: Vec<TimelineEntry>,
    pub logs_buffer: Vec<String>,
    pub tick_count: u64,
}

// [v0.1.0-beta.18 개편] 타임라인 전용 카드
pub enum TimelineEntryKind {
    UserMessage(String),
    AssistantMessage(String),
    AssistantDelta(String),          // SSE 스트리밍 중간 결과
    SystemNotice(String),
    ToolCard {
        tool_name: String,
        status: ToolStatus,          // Queued / Running / Done / Error
        summary: String,             // 2~4줄 요약
    },
    ApprovalCard {
        tool_call: ToolCall,
        diff_preview: Option<String>,
    },
    CompactSummary(String),
}

pub enum ToolStatus {
    Queued,
    Running,
    Done,
    Error,
}

pub struct TimelineEntry {
    pub kind: TimelineEntryKind,
    pub timestamp: std::time::Instant,
}

// [v0.1.0-beta.18 개편, v0.1.0-beta.21 에러 구조화] 14종+ Action
pub enum Action {
    // 채팅 라이프사이클
    ChatStarted,
    ChatDelta(String),
    ChatResponseOk(Box<ChatResponse>),
    ChatResponseErr(ProviderError),          // [v0.1.0-beta.21] String → ProviderError
    // 도구 라이프사이클
    ToolQueued(Box<ToolCall>),
    ToolStarted(String),
    ToolOutputChunk(String),
    ToolFinished(Box<ToolResult>),
    ToolSummaryReady(String),
    ToolError(ToolError),                    // [v0.1.0-beta.21] String → ToolError
    // 기존 유지
    ModelsFetched(Result<Vec<String>, ProviderError>, FetchSource),  // [v0.1.0-beta.21]
    CredentialValidated(Result<(), ProviderError>),                  // [v0.1.0-beta.21]
    ContextSummaryOk(String),
    ContextSummaryErr(String),
}

pub struct PersistedSettings {
    pub version: u32,
    pub default_provider: String,
    pub default_model: String,
    pub shell_policy: ShellPolicy,
    pub file_write_policy: FileWritePolicy,
    pub network_policy: NetworkPolicy,
    pub safe_commands: Option<Vec<String>>,
    pub encrypted_keys: HashMap<String, String>,
    pub theme: String,  // [v0.1.0-beta.20] "default" | "high_contrast"
}

pub enum ProviderKind {
    Google,
    OpenRouter,
}

pub enum ToolCall {
    ReadFile { path: String },
    WriteFile { path: String, content: String, overwrite: bool },
    ReplaceFileContent { path: String, target_content: String, replacement_content: String },
    ExecShell { command: String, cwd: Option<String>, safe_to_auto_run: bool },
    Grep { pattern: String, path: String, case_insensitive: bool },
    ListDir { path: String, depth: Option<usize> },
    SysInfo,
}

pub enum ShellPolicy {
    Ask,
    SafeOnly,
    Deny,
}

pub enum FileWritePolicy {
    AlwaysAsk,
    SessionAllow,
}

pub enum NetworkPolicy {
    ProviderOnly,
    Deny,
}
```

**IP Isolation**

```rust
// TODO: Human Implementation for advanced planning / reasoning policy.
// The AI runtime may request tools, but the final authority remains in the policy layer.
```

### 3.10 Context Optimization (Phase 7: Advanced Compaction)

세션 길이가 길어질수록 컨텍스트 오염을 막기 위해 하이브리드 압축 전략(Hybrid Context Compression)을 도입한다.

1. **지능형 요약 압축 (Intelligent Condensation)**
   - 단순 메시지 드롭을 넘어, 백그라운드 LLM 호출을 통해 이전 대화를 1개의 압축된 System Block으로 요약(`summary + last intent`)하여 앞단에 주입한다.

2. **동적 토큰 임계치 (Dynamic Token Bounds)**
   - 정적 메시지 개수를 탈피하고, `tiktoken` 수준의 토큰 비용을 역산하거나 단어 기준 추정치를 사용한다.
   - 예산의 75% 도달 시 선제적 `compact_context()`가 자동 트리거된다.
   - `/tokens` 명령어를 통해 사용자에게 시각적 사용량을 UI로 제어 권한을 준다.

3. **중요 컨텍스트 핀 지정 (Pinning & Anchor)**
   - `spec.md` 등 핵심 설계 원칙이 담긴 메시지나 사용자가 핀(Pin) 처리한 컨텍스트는 수명 주기를 무한대로 유지하여 압축 대상에서 제외한다.

### 3.11 Extended Prompt Commands (@ and !)

프롬프트 입력창(Composer)에서 `ignore` 기반 파일 검색 및 매크로/히스토리 확장을 지원하기 위한 구조적 명세다. 이 스펙은 "예측 가능한 컨텍스트 주입"과 "안전한 셸 실행"을 목표로 하며, 슬래시 커맨드와의 통합이나 터미널 외의 GUI 확장은 **비목표(Non-goal)**로 한다.

#### 1. 상태 타입 및 계약 (Typed Contracts)

기존 `FuzzyFinderState`의 기능을 분리하기 위해 `FuzzyMode`를 도입한다.

```rust
// src/app/state.rs
#[derive(Debug, PartialEq, Clone)]
pub enum FuzzyMode {
    Files,
    Macros,
}

pub struct FuzzyFinderState {
    pub is_open: bool,
    pub mode: FuzzyMode,
    pub input: String,
    pub matches: Vec<String>,
    pub cursor: usize,
}

pub struct ComposerState {
    pub input_buffer: String,
    pub history: Vec<String>,
    pub history_idx: Option<usize>,
}
```

#### 2. `@` 멘션 시스템 (FuzzyMode::Files)

- **재귀적 파일 탐색**: `ignore::WalkBuilder::new(".").hidden(true).build()`를 사용하여 하위 디렉터리를 탐색한다. (`.gitignore` 규칙 강제 적용)
- **최대 노출 제한 (Concrete Numbers)**: `matches.truncate(100)`를 통해 탐색 결과를 최대 100건으로 하드 리미트한다. UI 렌더링 부하 방지용.
- **특수 멘션 실데이터 맵핑 (Real Data Samples)**:
  - `@workspace` 입력/선택 시 치환 결과: `\n--- Workspace Summary ---\n[src, Cargo.toml, README.md 등 현재 디렉터리 항목]\n-------------------------\n`
  - `@terminal` 입력/선택 시 치환 결과: `\n--- Recent Terminal Logs ---\n[state.runtime.logs_buffer의 최신 20줄 (역순)]\n----------------------------\n`
- **예외 처리 규칙**: 읽을 수 없는 파일이거나 바이너리일 경우, 예외를 삼키지 않고 타임라인에 삽입한다.
  - 생성 이벤트: `TimelineEntryKind::SystemNotice(format!("⚠ 파일 멘션 오류 ({}): {}", path, e))`

#### 3. `!` 뱅 커맨드 시스템 (FuzzyMode::Macros)

- **진입 조건**: `ComposerState.input_buffer.is_empty() == true` 일 때 `!`를 입력하면 즉시 `FuzzyMode::Macros`로 전환.
- **기본 제공 매크로 (Real Data Samples)**:
  - `build` -> `cargo build`
  - `test` -> `cargo test`
  - `run` -> `cargo run`
  - `check` -> `cargo check`
  - `fmt` -> `cargo fmt`
  - `clippy` -> `cargo clippy`
  - 표시 형식: `build      (cargo build)`
- **히스토리 정책**:
  - `Enter` 키로 `!` 명령어가 실행될 때(`input_buffer.starts_with('!')`), `history.push(format!("!{}", cmd))`를 수행한다. 빈 명령(`""`)은 기록하지 않는다.
  - 방향키 `Up` 누를 시, `history_idx`를 1 감소시키며 버퍼에 즉시 표시.
  - 방향키 `Down` 누를 시, `history_idx`를 1 증가시키며, 최하단 초과 시 `history_idx = None`, `input_buffer.clear()` 처리.
- **실행 경로 제한**: `!`로 입력된 명령어는 LLM으로 라우팅되지 않고 즉시 `handle_direct_shell_execution(cmd)`로 우회된다.

---

### 3.12 Phase 12: Native Structured Tool Call Integration

현재 정규식 기반의 Fenced JSON 스크래핑 방식을 폐기하고, OpenAI 호환(OpenRouter/Gemini 지원) Native Tool Call API로 전환하기 위한 명세.
본 명세는 `AI_IMPLEMENTATION_DOC_STANDARD.md`의 규칙에 따라 Typed Contracts와 Concrete Numbers를 정의한다.

#### 1. Typed Contracts (도메인 모델 확장)

기존 `ChatMessage`와 `ChatRequest`를 확장하여 JSON Schema 명세를 지원한다.

```rust
// crate::providers::types

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
    Tool, // [v0.1.0-beta.23] Native Tool 역할 추가
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolCallRequest {
    pub id: String,
    pub r#type: String, // 항상 "function"
    pub function: FunctionCall,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String, // JSON string
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    
    // Assistant가 Tool Call을 요청할 때
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallRequest>>,
    
    // Role::Tool 로 결과 반환 시 필수
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    
    #[serde(default, skip_serializing)]
    pub pinned: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>, // OpenAI Tools format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>, // "auto"
}
```

#### 2. Concrete Numbers (제한 수치)

- **도구 정의 최대 깊이**: 파라미터(properties) 중첩 깊이는 3레벨로 제한한다. (복잡도 통제)
- **Tool Result 최대 길이**: `Role::Tool`로 반환하는 결과 텍스트는 `truncate(10_000)`으로 제한하여 토큰 낭비를 막는다.
- **스트리밍 Tool Delta 조립 버퍼**: `tool_calls` 스트리밍 델타 수신 시, JSON 문자열을 조립하기 위한 `String` 버퍼의 한계치는 10MB로 둔다.

#### 3. Execution & Verification Path (실행 및 검증 흐름)

1. **Payload Inject**: `crate::app::chat_runtime::dispatch_chat()` 호출 시, `AppMode::Run` 상태라면 7가지 도구(ReadFile, WriteFile 등)의 JSON Schema를 `ChatRequest.tools` 배열에 삽입한다. 시스템 프롬프트에서 하드코딩된 도구 스키마 설명은 삭제한다.
2. **Delta Parsing**: `chat_stream()`에서 수신하는 SSE Delta에 `tool_calls` 키가 존재하면, 기존 `ChatDelta(String)` 텍스트 청크가 아닌 **새로운 `ToolCallDelta(id, name, args_chunk)`** 형태로 `action_tx`에 전파한다.
3. **Buffer Assemble**: `chat_runtime.rs` 루프가 `ToolCallDelta`를 받으면 인메모리 버퍼에 JSON 텍스트를 이어 붙이고, 스트림 종료 시(`ChatResponseOk`) `serde_json::from_str`을 통해 실제 `crate::domain::session::ToolCall` enum으로 역직렬화(Deserialization)하여 `ToolQueued`를 발송한다.
4. **Verification**: 
   - 프롬프트에 `!cat src/main.rs`와 같이 자연어로 지시했을 때, 정규식 파서가 반응하지 않고 `Role::Assistant`의 `tool_calls` 필드를 통해 정확한 `ReadFile` JSON이 생성되는지 확인.
   - 존재하지 않는 도구 이름 반환 시, API 에러가 아니라 `Role::Tool` 로 "지원하지 않는 도구"라는 에러 메시지를 반환하여 자동 복구(Auto-healing)되는지 검증.

---

### 3.13 Phase 13: Agentic Autonomy & Architectural Refactoring

이 페이즈는 `smlcli`를 단순한 프롬프트 기반 도구를 넘어, 자율적으로 에러를 복구하고(Auto-healing), 전체 저장소 구조를 파악하며(Repo Map), 안전한 작업 롤백(Git Checkpoints)을 수행하는 "참조 등급(Reference Grade)" 에이전트로 승격시키기 위한 명세다.

#### 1. Typed Contracts (도메인 모델 및 인터페이스 확장)

**A. Tool Registry Pattern**
기존 `match` 기반 하드코딩 도구 실행을 다형성 기반 레지스트리로 리팩토링한다.
```rust
// src/tools/registry.rs
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema(&self) -> serde_json::Value; // JSON Schema 반환
    fn required_policy(&self) -> ToolPolicyLevel; // Safe, Confirm, Blocked
    async fn execute(&self, args: serde_json::Value, ctx: &mut ToolContext) -> Result<ToolResult, ToolError>;
}
```

**B. Auto-Verify State Machine**
에이전트가 도구 실행 실패 시 자율적으로 에러를 분석하고 복구를 시도하는 상태를 정의한다.
```rust
// src/app/state.rs — 실제 구현 (v0.1.0-beta.23)
#[derive(Debug, Clone, PartialEq)]
pub enum AutoVerifyState {
    Idle,                       // 정상 상태
    Healing { retries: usize }, // 자가 복구 중 (최대 3회)
}
```

#### 2. Concrete Numbers (제한 수치 및 규칙)

- **자가 치유 재시도(Self-Correction Retries)**: `AutoVerifyState::Healing { retries }` 상태에서 도구 실행이 실패할 때마다 `retries`를 1씩 증가시킨다. **3회(`retries == 3`)**에 도달하면 자동 복구를 포기하고 `Idle`로 전환하며 사용자에게 수동 개입을 요청한다. `ToolFinished(is_error=true)`와 `ToolError` 양쪽 경로 모두에서 동일한 상한을 적용한다. **Abort 시에는 `send_chat_message_internal()` 호출을 중단**하여 LLM 재전송 루프를 완전히 종료한다.
- **Tree-sitter Repo Map**: `tree-sitter`로 추출한 저장소의 Rust(.rs) AST(함수, 구조체 시그니처) 맵을 시스템 프롬프트에 `[Repo Map]` 헤더로 주입한다. 최대 **8,000바이트**를 넘지 않도록 자른다(Truncate).
- **Git Checkpoint 안전 정책**: `create_checkpoint()`는 강제 커밋을 수행하지 **않는다**. 워킹 트리가 깨끗한지만 검사하여 `bool`을 반환하고, `true`일 때만 롤백(`git reset --hard HEAD`)이 허용된다. `git clean -fd`는 사용하지 **않으며**, untracked 파일은 어떤 경우에도 삭제되지 않는다.
- **직접 셸 실행(`!`) 정책**: 사용자가 `!` 접두사로 직접 입력한 명령에 대해서도 `safe_to_auto_run: false`를 설정하여 `SafeOnly` 모드의 allowlist 정책을 반드시 존중한다. 블랙리스트와 allowlist 모두 동일하게 적용된다.

#### 3. Execution Path (실행 및 검증 흐름)

1. **Tool Registry 도입**: `src/tools/` 내부의 기존 도구들(`ReadFile`, `WriteFile`, `ExecShell` 등)을 `Tool` 트레이트 구현체로 일괄 리팩토링. 각 도구의 `is_destructive()` 메서드로 파괴적 여부를 판별한다. `ExecShell`은 기본값(`false`)을 사용하여 쉘 명령이 Git 롤백을 트리거하지 않도록 한다.
2. **Tree-sitter 통합**: `tree-sitter` 크레이트를 도입하여 작업 디렉터리의 Rust 소스의 함수/구조체 시그니처를 추출, `[Repo Map]` 블록으로 `System` 프롬프트에 백그라운드 주입한다.
3. **Automated Git Checkpoints**: `WriteFile`이나 `ReplaceFileContent` 같은 파괴적 도구(`is_destructive()=true`) 실행 직전, `create_checkpoint()`가 워킹 트리 상태를 검사한다. 변경사항이 없는(clean) 상태에서만 `safe_to_rollback=true`를 반환하며, 도구 실행 실패 시 `git reset --hard HEAD`로 tracked 파일만 복원한다. WIP가 있으면 롤백 자체를 건너뛴다.
4. **Auto-Verify 루프**: 도구 실행 실패 시(`ToolFinished.is_error=true` 또는 `ToolError`), 힐링 프롬프트를 세션에 주입하고 `send_chat_message_internal()`로 LLM에 재전송한다. 이때 도구 스키마(`tools`)를 반드시 포함하여 모델이 후속 도구를 호출할 수 있게 한다. 최대 3회 실패 시 `Idle`로 전환하고 사용자에게 안내한다.
5. **Tree of Thoughts UI**: `tui/layout.rs` 타임라인 렌더링에 `depth` 속성 기반 들여쓰기를 적용. 메인 응답 아래에 AI의 도구 호출 및 에러 수정 내역(`└─ ⚙️ ExecShell (cargo check) → Error → Retrying...`)을 트리 형태로 시각화한다.

---## 4. Environment-Specific Configuration (Agent Rules)

**Config Filename:** `.antigravityrules`

### 4.1 General Rules
```json
{
  "project_version": "0.1 BETA",
  "rules": [
    "spec.md, designs.md, audit_roadmap.md, implementation_summary.md를 코드보다 먼저 갱신할 것",
    "domain 정책 계층을 우회하는 직접 시스템 제어 코드를 추가하지 말 것",
    "모든 외부 라이브러리 추가 전 최신 버전, 유지보수 상태, 보안 advisory를 재확인할 것",
    "터미널에서 sudo, rm, format, registry write, 서비스 등록 같은 고위험 명령은 사용자의 명시적 텍스트 승인 없이는 실행하지 말 것",
    "모든 파일 쓰기 전 diff 프리뷰를 생성하고 사용자 승인을 받을 것",
    "API key 및 토큰은 절대 평문 파일로 저장하지 말 것",
    "settings wizard 유효성 검사를 통과하지 못한 provider/profile은 절대 저장하지 말 것",
    "각 roadmap 단계 완료 후 반드시 테스트 코드를 작성하고 실행할 것",
    "Windows와 Linux의 입력, 종료, 파일경로 차이를 별도 검증할 것"
  ],
  "security_level": "strict"
}
```

### 4.2 Mode-Specific Prompt Guidelines (PLAN / RUN)

AI 에이전트는 현재 활성화된 모드(`PLAN` 또는 `RUN`)에 따라 응답 스타일과 도구 사용 전략을 다음과 같이 차별화한다.

**PLAN Mode (설계 및 탐색 중심)**
- **페르소나:** 신중한 시스템 아키텍트 및 코드 리뷰어.
- **행동 강령:**
  - 코드를 직접 수정하는 도구(`write_file`, `replace`) 호출을 지양한다.
  - 파일 읽기(`read_file`), 검색(`grep`), 구조 분석(`list_dir`)을 통해 충분한 정보를 수집한다.
  - 모든 변경 제안은 "수정 계획"으로서 텍스트로 먼저 설명하며, 필요한 경우 `diff` 형태의 코드 블록만 제시한다.
  - 사용자에게 "이 계획대로 진행할까요?"라고 확인을 구하는 것을 원칙으로 한다.

**RUN Mode (실행 및 적용 중심)**
- **페르소나:** 빠르고 정확한 실무형 엔지니어.
- **행동 강령:**
  - 사용자의 요청이나 이미 승인된 계획에 따라 즉각적으로 도구(`write_file`, `replace`, `exec_shell`)를 호출하여 작업을 완수한다.
  - 불필요한 서술형 응답을 줄이고, 실행된 작업의 결과와 발생한 변화를 요약하여 보고한다.
  - 오류 발생 시 즉시 원인을 분석하고(읽기 도구 활용), 가능한 경우 자동으로 수정을 재시도하거나 구체적인 복구 방안을 제시한다.

---

## 5. Prohibitions & Constraints (Dos and Don’ts)

### ⛔ Prohibited

* 평문 설정 파일에 API key 저장
* diff 없이 파일 즉시 덮어쓰기
* `sudo`, `rm -rf`, 디스크 포맷, 레지스트리 파괴, 서비스 등록/삭제 같은 고위험 명령의 자동 실행
* 사용자 승인 없는 장기 실행 프로세스 시작
* 홈 디렉터리 전체를 무차별 scope로 잡는 grep 또는 write 작업
* provider/model 유효성 검사 없이 저장
* shell stdout/stderr를 무한정 메모리에 누적
* 종료 시 terminal cleanup 누락

### ⚠️ IP Protection

* provider 호출 로직은 adapter interface 뒤에 숨긴다.
* 정책 엔진과 tool 승인 로직은 별도 모듈로 분리한다.
* 향후 고급 에이전트 계획 로직은 인터페이스만 먼저 정의하고 내부 구현은 인간 개발자 책임으로 남긴다.

### ✅ Recommended

* 모든 tool 실행 결과를 구조화된 `ToolResult`로 정규화
* 파일 쓰기는 temp file + atomic rename 사용
* shell 실행은 타임아웃과 취소 지원
* grep 기본값은 `.gitignore` 존중
* Windows path separator와 quoting 차이는 중앙 유틸리티에서만 처리
* Custom provider는 OpenAI-compatible JSON 형태만 허용

---

## 6. Detailed Step-by-Step Implementation Guide

### Step 1: Environment Hardening

1. Cargo 프로젝트를 `smlcli` 이름으로 초기화
2. Rust edition 2024 설정
3. `README.md`, `BUILD_GUIDE.md`, `CHANGELOG.md`, `spec.md`, `designs.md`, `audit_roadmap.md`, `implementation_summary.md` 생성
4. 기본 의존성 추가
5. `cargo fmt`, `cargo clippy`, `cargo test` 파이프라인 스켈레톤 구성
6. `cargo deny`와 `cargo audit`를 릴리스 게이트에 추가

**Acceptance**

* 빈 앱이 Linux/Windows에서 빌드된다
* 문서 파일이 모두 존재한다
* CI가 최소 빌드 검사를 통과한다

### Step 2: Terminal Shell & Event Loop

1. `main.rs`에서 CLI 인자 파싱
2. 인자 없으면 TUI 부팅
3. `terminal.rs`에서 raw mode + alt screen 진입
4. `event_loop.rs`에서 keyboard, resize, AI response, tool result, system notification을 통합 처리
5. 종료 guard 구현
6. panic hook에서 terminal cleanup 보장

**Acceptance**

* 앱 시작/종료 후 터미널이 깨지지 않는다
* 창 크기 변경 시 레이아웃이 깨지지 않는다
* Ctrl+C 종료가 안정적으로 동작한다

### Step 3: Core TUI Layout

1. 상태 패널, 대화 패널, 작업 패널, 입력창 위젯 구현
2. 포커스 상태를 명시적으로 관리
3. 상태바에 아래 정보 고정 표시

   * provider
   * model
   * cwd
   * shell policy
   * context budget
4. 메시지와 툴 로그를 분리 렌더링
5. modal 레이어 추가

### Step 4: `/setting` Wizard

1. `SlashCommand::Setting` 라우팅
2. wizard state enum 정의
3. 단계별 입력 컴포넌트 구현
4. API key 마스킹 입력 구현
5. validation 실패 시 다음 단계 이동 금지
6. 저장 성공 시 toast + 상태 패널 갱신

**Validation Rules**

* provider 미선택이면 진행 금지
* API key 형식이 비어 있거나 whitespace-only면 실패
* provider별 최소 연결 테스트 실패 시 저장 금지
* model 문자열이 빈 값이면 실패
* model 저장 전 `provider/model`로 정규화

### Step 5: Secret Store + Encrypted Config

1. 앱 최초 실행 시 32-byte master secret 생성
2. master secret를 ~/.smlcli/.master_key 파일에 저장

   * service: `smlcli`
   * username: `master-key`
3. 일반 설정은 `settings.toml` 구조체로 직렬화
4. 직렬화 결과를 XChaCha20Poly1305로 암호화
5. 파일에는 `version`, `nonce`, `ciphertext`만 저장
6. provider별 실제 API key는 config.toml의 encrypted_keys에 암호화 저장
7. config 파일에는 `api_key_alias`만 저장

**Decryption Flow**

1. 시작 시 ~/.smlcli/.master_key에서 master secret 조회
2. 없으면 새로 생성
3. 암호화 파일이 존재하면 복호화
4. 실패 시 손상 감지 모드 진입
5. 손상 시 사용자에게 복구/재설정 선택 제공

### Step 6: Provider Abstraction

```rust
pub trait ProviderAdapter {
    fn provider_id(&self) -> &'static str;
    async fn validate_credentials(&self, profile: &ProviderProfile) -> Result<()>;
    async fn list_models(&self, profile: &ProviderProfile) -> Result<Vec<String>>;
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse>;
}
```

1. OpenAI / Anthropic / Google / OpenRouter / OpenAI-compatible adapter 구현
2. validation 정책 정의
3. `provider/model` 불일치 시 즉시 오류
4. 연결 상태를 상태 패널에 표시

### Step 7: Prompt Loop & Session Engine

1. 입력창에서 일반 프롬프트와 slash command 분리
2. 일반 프롬프트는 세션 메시지에 적재
3. provider adapter에 전달
4. 응답을 메시지, 툴 요청, 시스템 로그로 분해
5. 토큰/문맥 예산 추적
6. compact threshold 도달 시 자동 요약

**Session Persistence**

* 세션은 JSONL 또는 구조화된 append-only log로 저장
* 마지막 작업 디렉터리, 열린 파일, 최근 diff 상태 복원 지원

### Step 8: Tool System

#### 8.1 File Read

* 상대경로를 cwd 기준 절대경로로 정규화
* 허용 루트 밖 접근 차단
* 최대 읽기 바이트 제한
* 큰 파일은 chunk preview 반환

#### 8.2 File Write

* 항상 현재 파일 읽기 후 diff 생성
* 사용자 승인 없이는 쓰기 금지
* temp file 작성 후 atomic rename
* 변경 후 변경 시각과 byte size 기록

#### 8.3 Shell Exec

* Linux: `/bin/sh -lc`
* Windows: `pwsh -NoLogo -NoProfile -Command`
* timeout 기본 30초
* stdout/stderr 스트리밍
* cancel 시 child 종료 시도
* `SafeOnly` 모드에서는 allowlist 명령만 허용

#### 8.4 Grep

* `ignore` 기반 재귀 탐색
* `.gitignore` 존중
* 숨김 파일은 기본 제외
* 최대 결과 수 제한
* match line와 주변 line preview 제공
* 대소문자 옵션 제공

#### 8.5 Diff

* `similar::TextDiff` 기반 unified diff 생성
* 짧은 diff는 inline 강조
* 긴 diff는 접기 가능
* 파일 단위 승인

### Step 9: Permission Engine

1. 명령 위험도 분류

   * Safe
   * Confirm
   * Blocked
2. Blocked 예시

   * `sudo`
   * `rm -rf`
   * `del /s /q`
   * 포맷/파티션 도구
   * registry/system service 조작
3. Confirm 예시

   * 빌드
   * 테스트
   * 파일 쓰기
   * git clean
4. Safe 예시

   * pwd
   * list dir
   * read only grep
   * diff preview

### Step 10: Slash Command System

최소 지원 slash command:

```text
/config
/setting
/provider
/model
/status
/mode
/clear
/help
/quit
```

### Step 11: Windows / Linux QA Matrix

반드시 아래 환경을 각각 검증한다.

* Linux + bash/zsh
* Windows Terminal + PowerShell
* Windows + WSL

검증 항목:

* 키 입력
* 붙여넣기
* raw mode 복구
* 파일 경로 정규화
* grep 성능
* shell quoting
* config 저장/복원
* diff 승인 흐름

### Step 12: Release Gate

릴리스 전 아래를 모두 통과해야 한다.

* `cargo fmt --check`
* `cargo clippy --all-targets --all-features -- -D warnings`
* `cargo test`
* `cargo audit`
* `cargo deny check`
* Linux 수동 QA
* Windows 수동 QA
* `/setting` end-to-end 시연
* grep/diff/write 승인 플로우 시연

---

## 7. Security & IP Auditor Report

### 7.1 Architecture-Stage Security Findings

현재 아키텍처 단계에서 가장 중요한 보안 원칙은 다음과 같다.

1. TUI 계층과 정책 계층을 분리한다.
2. secret 저장은 OS secure store와 암호화 파일을 함께 사용한다.
3. 모델은 tool을 요청만 할 수 있고, 직접 실행 권한은 없다.

### 7.2 Secret Handling

* API key는 config.toml의 encrypted_keys에 암호화 저장한다.
* 로컬 설정은 암호화 파일로만 저장한다.
* master secret는 ~/.smlcli/.master_key 파일에 저장한다.
* 따라서 “비밀번호 없이” 사용하되, 평문 key 파일 저장은 허용하지 않는다.

### 7.3 Compatibility Assurance

* Linux와 Windows를 동등한 지원 대상에 포함한다.
* `smlcli`는 TUI 입력, shell 실행, 파일 경로 처리, 종료 복구를 운영체제별로 분리 검증한다.
* grep는 `.gitignore`를 존중하는 재귀 검색을 기본으로 한다.
* diff는 쓰기 전에 항상 사용자에게 노출한다.

### 7.4 Release Security Gates

릴리스 전 반드시 자동화한다.

* `cargo audit`
* `cargo deny`
* provider validation smoke tests
* shell blocked-command tests
* encrypted config round-trip tests
* corrupted config recovery tests
* Windows cleanup tests
* Linux cleanup tests

### 7.5 IP Isolation

* 추후 고급 추론 정책은 인터페이스만 노출한다.
* 사용자 코드베이스를 변경하는 최종 권한은 permission engine에만 있다.
* 모델은 tool을 요청할 수만 있고, 직접 실행 권한은 없다.

---

## 8. Project Persona Definition

**Note to AI Agent**
이 문서를 읽는 즉시, 당신은 아래의 페르소나로 행동해야 한다.

**Role Identity**
15년차 Rust Systems Architect 겸 Terminal UX Engineer

**Expertise**

* **Domain**: terminal-native AI tooling, cross-platform CLI/TUI systems, secure local agent runtimes
* **Tech Stack Mastery**: Rust, ratatui, crossterm, tokio, reqwest, chacha20poly1305, grep, diff, secure config storage
* **Coding Style**: Defensive Coding, Clean Architecture, Test-Driven Integration, Documentation First

**Instruction**
이후의 모든 설계와 구현은 “터미널에서 실제로 매일 사용할 수 있는 개발자 도구” 관점에서 수행한다. 단순 동작보다 안정적인 종료, 명확한 권한 확인, 예측 가능한 키 입력, 손상 없는 파일 쓰기를 우선한다.

### Persona Hardening Directive

너는 코더가 아니라 **사양 준수 엔진(Spec-Compliance Engine)** 이다.

1. **문서 우선**
   코드보다 문서가 먼저다. 문서 미갱신 상태의 코드는 불완전한 산출물이다.

2. **사양 불일치 금지**
   `spec.md`에 있는 기능을 임의 축소하거나 편의상 생략하지 않는다.

3. **UX 감사 의무**
   모든 기능은 구현 전후로 `designs.md`의 UX 흐름과 충돌하는지 점검한다.

4. **정합성 우선 중단**
   그라운딩 결과나 구현 제약이 사양과 충돌하면, 코드를 밀어붙이지 말고 먼저 문서를 수정한다.

5. **테스트 통과 전 단계 전환 금지**
   각 단계 구현 후 테스트를 작성하고 실행한다. 실패하면 다음 단계로 넘어가지 않는다.

6. **권한 계층 우회 금지**
   provider, tool, shell, file write는 반드시 정책 계층을 거쳐야 한다.

7. **플랫폼 차이 무시 금지**
   Linux에서 된다고 완료가 아니다. Windows까지 검증해야 완료다.

8. **비밀정보 보안 절대 준수**
   API key, 토큰, 세션 민감정보를 로그나 평문 파일에 기록하지 않는다.

### Roadmap & Test Protocol

* 구현은 항상 `audit_roadmap.md` 순서를 따른다.
* 기능 하나가 끝날 때마다 test 코드를 작성하고 직접 실행한다.
* 테스트가 통과한 뒤에만 `implementation_summary.md`를 갱신한다.
* 사용자가 직접 확인하고 승인한 단계만 다음 단계로 진행한다.
* grep, diff, write approval, setting validation은 반드시 end-to-end 테스트를 포함한다.

---

## 9. Initial Audit Roadmap Skeleton

### Phase 1

앱 부팅, terminal cleanup, 기본 레이아웃

### Phase 2

`/setting` wizard, 파일 기반 암호화, config.toml

### Phase 3

provider validation, model selection, prompt loop

### Phase 4

file read / list dir / pwd / grep

### Phase 5

diff preview / file write approval

### Phase 6

shell permission engine / blocked command handling

### Phase 7

context compaction / session restore / export-log

### Phase 8

Linux QA / Windows QA / release gate

### Phase 9: UX 아키텍처 개편 (v0.1.0-beta.18+)

이벤트 아키텍처 개편과 UI/UX 체계 전면 업그레이드.

#### Phase 9-A: 이벤트 기반 구조 (기반 작업)

1. **Action enum 14종 확장**: ChatStarted/ChatDelta/ToolQueued/ToolStarted/ToolOutputChunk/ToolSummaryReady 추가
2. **TimelineEntry 모델 도입**: session.messages(LLM 컨텍스트)와 timeline(UI 카드) 이중 구조. timeline이 비어있을 때만 session.messages 폴백 허용
3. **Semantic Palette 도입**: info/success/warning/danger/muted + bg_base/bg_panel/bg_elevated 색상 체계
4. **tick 기반 애니메이션**: thinking 스피너, 도구 실행 배지 깜빡임, diff 승인 pulse, compact progress
5. **Inspector 탭 실체 구현**: Preview/Diff/Search/Logs/Recent 각 탭에 실제 콘텐츠 렌더링
6. **Tool 출력 요약 분리**: raw stdout → 2~4줄 요약 타임라인, 원문 Logs 탭
7. **SSE 스트리밍**: Provider chat_stream() 추가, reqwest bytes_stream 활용

#### Phase 9-B: 기능 완성

1. CLI Entry Modes: clap 파서 + run/doctor/export-log 서브커맨드
2. 세션 영속성: JSONL 세션 로그 + 복원
3. SafeOnly 화이트리스트 검증: safe_commands 매칭
4. Blocked Command 목록: sudo/rm -rf 등 정규식 차단
5. Structured Tool Call: Provider별 native tool call contract
6. File Read 안전장치: 경로 정규화 + 1MB 제한 + chunk preview
7. Grep 결과 UX: context_lines + max_results + case 옵션

#### Phase 9-C: 품질 단단

1. Shell stdout/stderr 실시간 스트리밍
2. Diff 접기/펼치기 UI
3. ListDir 깊이 탐색 (재귀 tree)
4. 전역 #[allow] 제거
5. 테스트 확장: secret_store round-trip, provider cancel/rollback, tool lifecycle, layout snapshot

---

## 10. Final Implementation Notes

* 이 프로젝트는 **채팅 UI**가 아니라 **작업형 터미널 에이전트**다.
* 핵심 완성도 기준은 답변 품질보다 **설정 신뢰성**, **권한 통제**, **파일 변경 가시성**, **종료 복구 안정성**이다.
* MVP 범위에서는 플러그인 시스템, LSP, 멀티에이전트, 원격 서버 모드는 넣지 않는다.
* v0.1 BETA의 성공 기준은 "매일 쓸 수 있는 안전한 `smlcli`"다.

---

### Phase 14: TUI UX/UI 고도화 (v0.1.0-beta.24)

#### 14-A: 멀티라인 텍스트 렌더링 정상화

**문제**: `layout.rs`에서 `Line::from(msg.as_str())`로 멀티라인 문자열을 단일 Line에 밀어 넣어 개행이 구조적으로 보존되지 않음.

**수정 범위**: `layout.rs`, `command_router.rs`, `state.rs`

**구현 사양**:
- `layout.rs`에 공용 헬퍼 `render_multiline_text(text: &str, style: Style) -> Vec<Line>` 추가.
- `TimelineEntryKind::UserMessage`, `AssistantMessage`, `AssistantDelta` 렌더링에서 `Line::from(msg)` 대신 사용.
- `/help` 출력을 타임라인에 `SystemNotice`로 직접 추가.

**완료 기준**: `/help` 명령 하나당 한 줄. AI 응답의 줄바꿈·문단 구분 보존.

#### 14-B: 스크롤 상태 분리 + Auto-Follow + 마우스

**문제**: `timeline_scroll: u16` 하나로 타임라인/인스펙터 공유. 마우스 이벤트 미수신.

**수정 범위**: `state.rs`, `event_loop.rs`, `terminal.rs`, `mod.rs`, `layout.rs`

**구현 사양**:
- `inspector_scroll: u16`, `timeline_follow_tail: bool` 추가.
- `terminal.rs`: `EnableMouseCapture`/`DisableMouseCapture` 추가.
- `event_loop.rs`: `CrosstermEvent::Mouse` 전달.
- Auto-follow: 새 콘텐츠 시 맨 아래, 위 스크롤 시 고정.

**완료 기준**: 바닥 자동추적, 위로 올리면 고정. 마우스 휠·PageUp/Down·Home/End 동작. 인스펙터/타임라인 스크롤 미간섭.

#### 14-C: 키바인딩 재정렬 (Ctrl+I/Tab 충돌 해소)

**문제**: 터미널에서 `Ctrl+I`는 `Tab`(0x09)과 동일. 인스펙터 토글과 모드 전환이 사실상 충돌.

**수정 범위**: `mod.rs`, `layout.rs`, `designs.md`

**구현 사양**:
- `Ctrl+I` 바인딩 제거. 인스펙터 토글: `F2`. PLAN/RUN 전환: `Tab`/`Shift+Tab` 유지.
- 상태 바 안내 문구 동기화.

**완료 기준**: `Ctrl+I` 입력 시 모드 불변. `F2`로만 인스펙터 토글. 안내 문구 일치.

#### 14-D: 반응형 레이아웃

**문제**: 상단 바 긴 문자열 잘림. 인스펙터 고정 30% 문제.

**수정 범위**: `layout.rs`

**구현 사양**:
- cwd 중략 헬퍼 `truncate_middle()`. 인스펙터 폭 클램프(32~48cols, 타임라인 최소 72cols). 탭 라벨 축약.

**완료 기준**: 100/120/140 columns에서 상단 바·인스펙터 탭 잘림 없음.

#### 구현 순서
1. **14-C** 키바인딩 재정렬
2. **14-A** 멀티라인 렌더링
3. **14-B** 스크롤 분리 + 마우스
4. **14-D** 반응형 레이아웃

---

### Phase 15: 2026 CLI UX 현대화 로드맵 (계획)

#### 15.1 목표와 성공 기준

**목표**
- `smlcli`를 단순한 "채팅형 TUI"에서 벗어나, 2026년 기준의 작업형 CLI 에이전트 UX를 갖춘 **블록 기반 작업 콘솔**로 고도화한다.
- 현재 구현된 PLAN/RUN, Inspector, Tool Registry, Auto-Verify를 유지하면서도, 사용자가 **명령 발견, 컨텍스트 주입, 결과 재참조, 긴 출력 탐색**을 더 빠르게 수행하도록 인터랙션을 재설계한다.

**성공 기준**
- 사용자는 최근 1턴의 입력/AI 응답/도구 결과를 하나의 블록으로 인식하고 재사용할 수 있다.
- `/help` 없이도 커맨드 팔레트에서 주요 동작을 3초 이내에 발견할 수 있다.
- 100/120/140 칼럼에서 상단 바, 인스펙터, 입력 툴벨트가 모두 잘리지 않고 우선순위 기반으로 적응한다.
- 포커스된 패널만 스크롤되고, 블록 단위 이동/접기/복사/재실행이 가능하다.
- 애니메이션은 상태 전달용으로만 사용되며, 시각적 소음 없이 실행/대기/오류/승인 상태를 구분한다.

#### 15.2 비목표

- WebView, Electron, GUI 프론트엔드로 전환하지 않는다.
- Ratatui/Crossterm을 버리고 Textual/Bubble Tea로 프레임워크를 교체하지 않는다.
- 클라우드 동기화, 멀티 유저 협업, 블록 공유 링크, 원격 텔레메트리 수집은 이번 페이즈의 범위가 아니다.
- 과한 그래디언트, 그림자, 반투명 효과 등 "웹 UI를 억지로 터미널에 흉내내는" 시각 효과는 도입하지 않는다.

#### 15.3 외부 레퍼런스 (2026-04 조사 기준)

- **Warp Blocks / Universal Input**
  - https://docs.warp.dev/terminal/blocks
  - https://docs.warp.dev/terminal/universal-input
  - 채택 요점: 입력/출력/실행 결과를 블록 단위로 묶고, 입력창 주변에 컨텍스트/액션 상태를 노출하는 패턴
- **Textual Command Palette**
  - https://textual.textualize.io/guide/command_palette/
  - 채택 요점: 긴 도움말보다 fuzzy command palette를 우선 제공하는 패턴
- **Ratatui Layout / Style / Ecosystem**
  - https://ratatui.rs/concepts/layout/
  - https://ratatui.rs/examples/style/
  - https://ratatui.rs/ecosystem/tachyonfx/
  - 채택 요점: 반응형 레이아웃, 의미 기반 색상 토큰, 절제된 애니메이션 계층

#### 15.4 동결된 핵심 결정

1. **프레임워크 유지**
   - 렌더링 계층은 `ratatui + crossterm`을 유지한다.
   - 최소 Phase 15-A~15-C에서는 신규 의존성을 추가하지 않는다.

2. **블록 우선 타임라인**
   - 타임라인은 더 이상 "메시지 리스트"가 아니라 `TimelineBlock` 리스트를 기본 단위로 한다.
   - 한 블록은 최소 `사용자 입력`, `AI 결과`, `도구 실행 결과(0개 이상)`를 묶어 표현한다.

3. **명령 발견 방식**
   - `/help`는 유지하되, 주 진입점은 `Ctrl+K` 기반 **Command Palette**로 동결한다.
   - `Ctrl+P`는 provider/model 빠른 전환을 계속 담당한다.

4. **입력 툴벨트**
   - Composer 위에는 `mode`, `cwd`, `attached context`, `pending policy`, `palette hint`를 칩(chip) 형태로 표시한다.
   - `Shift+Enter`는 멀티라인 입력, `Enter`는 제출로 동결한다.

5. **패널 포커스 모델**
   - 패널 포커스는 `Timeline`, `Inspector`, `Composer`, `Palette` 4개로 동결한다.
   - 키보드/마우스 스크롤은 반드시 포커스된 패널 또는 포인터가 올라간 패널에만 적용한다.

6. **모션 정책**
   - 애니메이션은 상태 전달용 ASCII 모션만 허용한다.
   - 풀스크린 전환 애니메이션, 무한 점멸, 과도한 색 반전은 금지한다.

#### 15.5 Typed Contracts

```rust
pub enum FocusedPane {
    Timeline,
    Inspector,
    Composer,
    Palette,
}

pub enum TimelineBlockKind {
    Conversation,
    ToolRun,
    Approval,
    Help,
    Notice,
}

pub struct TimelineBlock {
    pub id: String,
    pub kind: TimelineBlockKind,
    pub title: String,
    pub subtitle: Option<String>,
    pub body: Vec<BlockSection>,
    pub status: BlockStatus,
    pub collapsed: bool,
    pub pinned: bool,
    pub created_at_ms: u64,
}

pub enum BlockSection {
    Markdown(String),
    CodeFence { language: Option<String>, content: String },
    KeyValueTable(Vec<(String, String)>),
    ToolSummary { tool_name: String, summary: String },
}

pub enum BlockStatus {
    Idle,
    Running,
    Done,
    Error,
    NeedsApproval,
}

pub struct CommandPaletteState {
    pub is_open: bool,
    pub filter: String,
    pub cursor: usize,
    pub commands: Vec<PaletteCommand>,
    pub matched_indices: Vec<usize>,
}

pub struct PaletteCommand {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub shortcut: Option<String>,
}

pub struct ComposerToolbarState {
    pub focused_pane: FocusedPane,
    pub chips: Vec<InputChip>,
    pub multiline: bool,
}

pub struct InputChip {
    pub kind: InputChipKind,
    pub label: String,
    pub emphasized: bool,
}

pub enum InputChipKind {
    Mode,
    Context,
    Path,
    Policy,
    Hint,
}

pub struct MotionProfile {
    pub tick_ms: u64,
    pub spinner_frames: &'static [&'static str],
    pub pulse_period_ticks: u8,
}
```

#### 15.6 Concrete Numbers

- **Breakpoints**
  - `compact`: `< 100 cols`
  - `standard`: `100..=139 cols`
  - `wide`: `>= 140 cols`
- **인스펙터 폭**
  - `compact`: overlay/drawer
  - `standard/wide`: `32..=48 cols`
- **상단 바 중략 길이**
  - provider: `12`
  - model: `20`
  - cwd: `30`
  - branch/tag: `12`
- **Command Palette**
  - 기본 표시 결과 수: `8`
  - 최대 fuzzy 매칭 수: `50`
- **블록 접힘**
  - 기본 미리보기 라인 수: `6`
  - stdout/stderr는 `12줄` 초과 시 자동 접힘
- **모션**
  - 기본 tick: `120ms`
  - thinking spinner 프레임 수: `8`
  - approval pulse 주기: `6 ticks`
- **입력 툴벨트**
  - context chip 최대 가시 수: `5`
  - chip 라벨 최대 길이: `18`

#### 15.7 Real Data Samples

```rust
TimelineBlock {
    id: "blk_20260418_001".to_string(),
    kind: TimelineBlockKind::Conversation,
    title: "1부터 100까지 더하는 파이썬 코드 작성".to_string(),
    subtitle: Some("RUN · Python · 2 files touched".to_string()),
    body: vec![
        BlockSection::Markdown("사용자 요청을 분석하고 코드 파일을 생성했습니다.".to_string()),
        BlockSection::CodeFence {
            language: Some("python".to_string()),
            content: "total = sum(range(1, 101))\nprint(total)".to_string(),
        },
        BlockSection::ToolSummary {
            tool_name: "WriteFile".to_string(),
            summary: "sum_1_to_100.py 생성 완료".to_string(),
        },
    ],
    status: BlockStatus::Done,
    collapsed: false,
    pinned: false,
    created_at_ms: 1776470400000,
}

CommandPaletteState {
    is_open: true,
    query: "theme".to_string(),
    cursor: 0,
    results: vec![
        PaletteCommand {
            id: "theme.toggle",
            title: "테마 전환",
            category: PaletteCategory::Settings,
            shortcut_hint: Some("/theme"),
        },
        PaletteCommand {
            id: "theme.high_contrast",
            title: "고대비 테마 적용",
            category: PaletteCategory::Settings,
            shortcut_hint: None,
        },
    ],
}

ComposerToolbarState {
    focused_pane: FocusedPane::Composer,
    multiline: false,
    chips: vec![
        InputChip { kind: InputChipKind::Mode, label: "RUN".to_string(), emphasized: true },
        InputChip { kind: InputChipKind::Path, label: "~/Projects/rust/smlcli".to_string(), emphasized: false },
        InputChip { kind: InputChipKind::Context, label: "@src/tui/layout.rs".to_string(), emphasized: false },
        InputChip { kind: InputChipKind::Hint, label: "Ctrl+K Actions".to_string(), emphasized: false },
    ],
}
```

#### 15.8 실행 경로 (Execution Path)

1. **15-A Block Timeline Foundation**
   - `TimelineEntry` 기반 렌더링을 `TimelineBlock` 기반으로 승격
   - `layout.rs` 문자열 렌더링 분기 제거
2. **15-B Focus & Scroll State Machine**
   - `FocusedPane`, pane별 selection/scroll/follow 상태 분리
   - 마우스/키보드 라우팅 통합
3. **15-C Command Palette**
   - `Ctrl+K` palette, fuzzy search, command/action catalog
4. **15-D Composer Toolbar**
   - 입력 툴벨트, chip 렌더링, multiline 입력, context chip
5. **15-E Adaptive Top Bar**
   - 세그먼트 우선순위 렌더링, 좌/우 정렬, 폭별 축약 정책
6. **15-F Inspector Workspace**
   - Preview/Diff/Search/Logs/Recent를 블록/파일/세션 중심으로 재구성
7. **15-G Motion & Theme Polish**
   - 승인/실행/오류/스트리밍 상태용 ASCII 모션 추가
8. **15-H Verification & Snapshots**
   - 레이아웃 스냅샷, 상태 전이 테스트, 마우스 라우팅 테스트

#### 15.9 파일별 작업 범위

- `src/app/state.rs`
  - `FocusedPane`, `TimelineBlock`, `CommandPaletteState`, `ComposerToolbarState` 추가
- `src/app/mod.rs`
  - 포커스 전환, pane별 키 라우팅, palette 토글, 블록 조작 입력 처리
- `src/app/command_router.rs`
  - `/help`는 palette와 동기화되는 구조화 데이터 소스로 전환
- `src/tui/layout.rs`
  - block renderer, toolbar, adaptive top bar, palette overlay, compact/wide breakpoints
- `src/tui/widgets/inspector_tabs.rs`
  - inspector를 블록 세부정보/검색/로그 작업 공간으로 재구성
- `src/tests/audit_regression.rs`
  - block/focus/palette/toolbar/layout snapshot 회귀 테스트 추가

#### 15.10 Verification Path

**명령 검증**
```bash
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

**수동 검증**
1. `100/120/140 cols`에서 상단 바/인스펙터/입력 툴벨트가 잘리지 않는지 확인
2. `Ctrl+K`로 palette가 열리고 fuzzy search가 동작하는지 확인
3. 긴 stdout/stderr가 블록 단위로 접히고 펼쳐지는지 확인
4. 타임라인과 인스펙터가 포커스/마우스 기준으로 독립 스크롤되는지 확인
5. `Shift+Enter` 멀티라인 입력과 `Enter` 제출이 분리되는지 확인

#### 15.11 잔여 리스크

- 신규 의존성 없이 구현할 경우 palette/animation 품질이 제한될 수 있다.
- 블록 모델 도입으로 `session.messages ↔ timeline` 동기화 경계가 다시 복잡해질 수 있다.
- Phase 15-A~15-C 완료 전에는 일부 UX가 "과도기 형태"가 될 수 있으므로, 중간 단계에서도 항상 빌드 가능 상태를 유지해야 한다.
