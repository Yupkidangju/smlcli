# smlcli Implementation Spec (v3.7.1)

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
v3.7.1

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
│   │   ├── state.rs (AppState, TimelineBlock, AutoVerifyState, WizardState, ConfigState, FuzzyState)
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
`session.messages`는 LLM 컨텍스트 전용이며, 사용자 화면 표시는 `timeline: Vec<TimelineBlock>`로 분리한다. 이 분리를 통해 도구 실행 요약 카드, 승인 카드, 실행 로그, 결과 요약을 독립적으로 관리한다. timeline이 비어있을 때만 session.messages 폴백을 허용한다(하위 호환).

**이벤트 세분화 (14종+ Action)**
채팅과 도구 호출의 전체 라이프사이클(시작·진행·완료·에러)을 별도 이벤트로 정규화하여 Codex 스타일 진행 표시를 구현한다.

**도구 호출 격리 계층 (v0.1.0-beta.22)**
LLM 응답에서 도구 호출을 감지할 때 3단계 필터를 적용한다:
1. **bare JSON 차단**: fenced(`\`\`\`json`)가 아닌 raw JSON 객체는 도구로 인식하지 않는다.
2. **`"tool"` 키 검증**: fenced JSON 블록 내에 `"tool"` 필드가 존재해야만 도구 후보로 취급한다.
3. **ToolCall 역직렬화 + 빈 명령 차단**: serde 역직렬화 성공 후에도 `ExecShell.command.trim().is_empty()`이면 즉시 거부한다.

**첫 턴 자연어 가드 (v0.1.0-beta.22)**
시스템 프롬프트에 다음 정책을 명시한다:
- 인삿말, 질문, 설명 등 비작업성 입력에는 도구 없이 자연어로만 응답한다.
- 파일 읽기/수정, 명령 실행, 검색 등 명시적 작업 요청은 첫 턴이라도 즉시 도구를 사용할 수 있다.
- 도구 카탈로그는 이름만 나열하며, 필드 스키마와 예시 JSON은 시스템 프롬프트에 포함하지 않는다.

**LLM 우선 도구 판정 (v0.1.0-beta.25)**
- 입력 의도 분류(`is_actionable_input`)는 사용자 입력의 성격을 설명하는 참고 신호로만 유지한다.
- 모델이 구조화된 `tool_calls`를 반환한 경우, 런타임은 이를 선제 차단하지 않고 LLM의 판단을 우선한다.
- 비작업성으로 분류된 턴에서 도구 호출이 발생하면 런타임 로그에 기록하되, 후속 안전성은 기존 Permission Engine이 담당한다.

**동시 요청 차단 및 블록 독립 라우팅 (v0.1.0-beta.26)**
모델이 응답을 스트리밍 중이거나 도구를 실행하는 동안(`is_thinking == true`)에는 새로운 대화 제출 및 명령어 실행이 원천 차단된다. 또한 비동기 응답은 `timeline.last_mut()`에 의존하지 않고, 요청 시점에 할당된 `active_chat_block_idx`를 통해 타겟 블록으로만 정확히 라우팅되어 블록 오염(Race condition)을 방지한다.

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
smlcli sessions
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
* 설정 파일이 없거나 `config.toml` 파싱에 실패하면 자동으로 `/setting` wizard 진입
* 설정 파일이 손상된 경우, Wizard 첫 단계에서 "설정 파일을 복구하거나 삭제 후 다시 설정하라"는 안내를 즉시 노출
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
* Network: `AllowAll` / `ProviderOnly` / `Deny`
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

6. **Workspace Trust Gate**
   처음 진입한 작업 루트(workspace root)에 대해서는 사용자가 해당 폴더를 신뢰하는지 명시적으로 선택해야 한다. 신뢰 상태가 확정되기 전에는 쓰기/셸 실행을 허용하지 않는다.

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
    AllowAll,      // 프로바이더 API + FetchURL 등 임의 외부 URL 허용
    ProviderOnly,  // 프로바이더 API만 허용, FetchURL 등 임의 외부 URL은 SSRF 방지를 위해 Deny
    Deny,          // 모든 네트워크 통신 차단
}
```

### 3.10 Phase 17: Windows Shell Host Alignment & Workspace Trust Gate
(구현 완료)

### 3.11 Phase 18: Multi-Provider Expansion & Advanced Agentic Tools (계획)
이 페이즈는 다양한 최신 상용 LLM Provider를 네이티브로 지원하고, 에이전트의 자율성과 탐색 능력을 극대화할 수 있는 고급 도구(Tools)를 추가하는 것을 목표로 한다. 2026년 4월 기준 최신 모델의 역량을 최대한 끌어올리기 위한 컨텍스트 호환성에 중점을 둔다.

#### 18.1 Multi-Provider Integration (2026.04 Grounded)
기존 `OpenRouter` 및 `Gemini` 의존성을 넘어, 주요 모델 제공사의 네이티브 API 연동을 추가한다. 각 제공사별 Dialect 추상화를 확장한다.

| Provider | Base URL | 대표 지원 모델 (2026.04 기준) | 비고 |
| --- | --- | --- | --- |
| **OpenAI** | `https://api.openai.com/v1` | `gpt-5.4`, `gpt-5.4-thinking`, `gpt-5.4-mini` | 최신 GPT-5.4 제품군. 1M 컨텍스트 윈도우. |
| **Anthropic** | `https://api.anthropic.com/v1/messages` | `claude-opus-4.7`, `claude-sonnet-4.7` | Agentic Software Engineering 및 복잡한 추론 특화. |
| **xAI** | `https://api.x.ai/v1` | `grok-4.20`, `grok-4.20-reasoning` | 극도로 빠른 속도와 낮은 환각(Hallucination)율, OpenAI SDK 호환 API 제공. |

- `ToolDialect` enum 확장 (`OpenAINative`, `AnthropicNative`, `XAI`).
- 설정(`config.toml`)의 `encrypted_keys`에 `openai_key`, `anthropic_key`, `xai_key` 확장.

#### 18.2 Advanced Agentic Tools
기존의 파일 읽기/쓰기/실행 도구 외에, 컨텍스트 비용을 절감하고 에이전트의 상황 인지(Situational Awareness) 능력을 강화하는 특화 도구를 추가한다.

1. **`ListDirectory` (디렉터리 탐색)**
   - `ExecShell("ls")` 대비 JSON 형태로 구조화된 파일 크기, 타입(파일/디렉터리), 자식 노드 수를 반환하여 LLM의 파싱 오동작 방지.
2. **`GrepSearch` (코드 심층 검색)**
   - 정규식 및 글로브 필터링(`*.rs`)을 지원하여, 전체 파일을 읽지 않고도 함수 정의와 변수 사용처를 정확하게 매핑.
3. **`FetchURL` (웹 문서/API 그라운딩)**
   - 외부 공식 API 문서나 레퍼런스 페이지를 실시간으로 읽어와(Markdown 변환) 자체 컨텍스트로 편입하는 도구.

이 섹션은 **구현 전 동결하는 계획 명세**다. 아래 항목이 문서화된 이후에만 코드를 변경한다.

**참고 제품 / 공개 레퍼런스**
- OpenAI Codex CLI 계열: sandbox / approvals / workspace 범위 분리 개념
- Gemini CLI 공개 문서: 계층형 설정 파일, `/permissions trust`, `/directory add/show`, 정책 파일 경로 관리

이 구현은 특정 제품을 그대로 복제하지 않는다. 다만 아래 개념을 차용한다.
- 설정을 **user / project / admin** 수준으로 분리 가능한 구조
- REPL에서 workspace/trust를 직접 조회·추가·제거하는 관리 명령
- 현재 세션이 신뢰하는 루트와 추가 workspace 디렉터리를 명시적으로 노출하는 UX
- 일반 사용자 설정과 관리자 강제 정책을 장기적으로 분리 가능한 데이터 모델

#### A. Scope

**목표**
- Windows 11에서 앱이 `cmd.exe` 호스트에서 실행되더라도, 내부 명령 실행(`ExecShell`)은 일관되게 PowerShell 계열(`pwsh` 우선, 없으면 `powershell.exe`)로 수행한다.
- 현재 작업 루트에 대한 신뢰 여부를 사용자에게 명시적으로 묻는 `Workspace Trust Gate`를 추가한다.
- 신뢰되지 않은 루트에서는 읽기 전용 탐색만 허용하고, 파일 쓰기/셸 실행/파괴적 도구는 모두 차단한다.

**비목표**
- 이번 단계에서 Windows 콘솔 호스트 자체를 새 PowerShell 창으로 강제 재실행하지 않는다.
- 전역적인 OS 보안 모델(예: Windows Defender 정책, AppLocker)은 다루지 않는다.
- 다중 워크스페이스 동시 신뢰 편집 UI는 이번 범위에 포함하지 않는다.

#### B. Frozen Decisions

1. **Host shell과 exec shell을 분리한다.**
   - Host shell: 사용자가 앱을 띄운 실제 콘솔(`cmd.exe`, PowerShell, Windows Terminal 등)
   - Exec shell: `ExecShell` 도구가 실제 명령을 수행하는 런타임 셸
   - 상태바와 진단 메시지에는 이 둘을 구분하여 표기한다.

2. **Windows exec shell 선택 우선순위**
   - 1순위: `pwsh.exe`
   - 2순위: `powershell.exe`
   - 둘 다 없으면 명시적 오류를 표시하고 `ExecShell` 실행을 중단한다.

3. **Workspace root는 시작 시 1회 결정한다.**
   - 우선순위: 명시적 CLI 인자(향후) > 현재 디렉터리에서 상향 탐색한 저장소 루트 > 현재 디렉터리
   - 저장소 루트 판별 기준: `.git` 또는 `Cargo.toml`

4. **Trust Gate는 3상태 모델을 사용한다.**
   - `Unknown`
   - `Trusted`
   - `Restricted`

5. **Restricted 상태의 정책**
   - `ReadFile`, `ListDir`, `Stat`, `GrepSearch`, `SysInfo`만 허용
   - `WriteFile`, `ReplaceFileContent`, `ExecShell`은 모두 거부
   - UI는 명시적으로 “신뢰 전 읽기 전용 모드”임을 표시

6. **Trust persistence**
   - 신뢰 결과는 `workspace_root -> trust_state` 형태로 로컬 설정에 저장
   - 저장 위치는 기존 `config.toml` 체계를 확장한다
   - “한 번만 신뢰”는 세션 메모리에서만 유지하고 프로세스 종료 시 폐기한다

7. **Workspace policy management**
   - 신뢰된 workspace 목록, 추가 workspace 디렉터리, 접근 금지 루트(denied roots)는 시작 시점 프롬프트뿐 아니라 REPL 명령과 설정 UI에서 모두 관리 가능해야 한다.
   - denied root는 trust 상태와 무관하게 최우선 차단 규칙을 갖는다.

#### C. Typed Contracts

```rust
pub enum WorkspaceTrustState {
    Unknown,
    Trusted,
    Restricted,
}

pub struct WorkspaceTrustRecord {
    pub root_path: String,
    pub state: WorkspaceTrustState,
    pub remember: bool,
    pub updated_at_unix_ms: u64,
}

pub struct WorkspacePolicySettings {
    pub trusted_workspaces: Vec<WorkspaceTrustRecord>,
    pub denied_roots: Vec<String>,
    pub extra_workspace_dirs: Vec<String>,
}

pub struct RuntimeWorkspaceState {
    pub root_path: String,
    pub host_shell: String,
    pub exec_shell: String,
    pub trust_state: WorkspaceTrustState,
    pub trust_prompt_visible: bool,
    pub extra_workspace_dirs: Vec<String>,
}
```

#### D. Concrete Rules

- Trust Gate가 `Unknown`인 경우, 앱 시작 직후 **첫 화면**에서 선택해야 한다.
- 선택지는 3개로 고정한다.
  - `Trust Once`
  - `Trust & Remember`
  - `Restricted`
- Restricted 상태에서는 차단 메시지에 항상 다음 문구를 포함한다.
  - `"Workspace is not trusted. Enable trust before write or shell actions."`
- denied root와 매칭되는 경로는 trust 상태와 무관하게 읽기/쓰기/셸 참조가 모두 거부된다.
- `extra_workspace_dirs`는 현재 세션에서 추가로 허용한 디렉터리 집합이며, 각 경로는 개별 trust 평가 및 설정 저장 대상이다.
- Windows exec shell 진단 문자열은 아래 형식으로 통일한다.
  - `Host: cmd.exe | Exec: pwsh`
  - `Host: WindowsTerminal | Exec: powershell.exe`

#### E. Execution Path

1. **Task 1: Workspace root 결정 유틸리티**
   - 현재 디렉터리 기준으로 상향 탐색하여 루트를 결정하는 함수 추가
   - `target/release` 보정 로직과 충돌하지 않도록 통합

2. **Task 2: Trust/Workspace 정책 모델 및 설정 영속화**
   - `PersistedSettings`에 trust record 맵, denied roots, extra workspace dirs 추가
   - 시작 시 현재 root의 trust state와 관련 정책을 로드
   - user/project/admin 계층 확장을 막지 않는 저장 포맷으로 설계

3. **Task 3: Trust Gate UI**
   - Trust가 `Unknown`이면 메인 채팅 대신 startup prompt를 띄운다
   - 방향키 + Enter로 3개 선택지 제공
   - Restricted 선택 시 즉시 읽기 전용 상태로 진입

4. **Task 4: Permission Engine 연동**
   - trust state가 `Restricted`이면 쓰기/셸 도구 차단
   - denied root에 매칭되는 경로는 읽기 포함 전면 차단
   - 차단 메시지와 타임라인 Notice 추가

5. **Task 5: Windows exec shell 정렬**
   - `pwsh.exe`/`powershell.exe` 탐지 유틸리티 추가
   - `ExecShell` 실행 경로와 system prompt의 shell 표기를 동일 기준으로 맞춤

6. **Task 6: REPL 명령 / 설정 UI 추가**
   - `/workspace show`: 현재 root, 추가 workspace dirs, trust state, denied roots 표시 **(구현 완료)**
   - `/workspace trust [path]`: 현재 또는 지정 path를 `Trust Once / Trust & Remember / Restricted`로 변경 **(구현 완료: 현재 root 대상)**
   - `/workspace deny`: 현재 root를 Restricted + denied_roots에 추가 **(구현 완료)**
   - `/workspace clear`: 현재 root의 trust 및 deny 기록 초기화 **(구현 완료)**
   - `/workspace add <path>`: 추가 workspace dir 등록 **(미구현 — v3.0 계획)**
   - `/workspace remove <path>`: 추가 workspace dir 해제 **(미구현 — v3.0 계획)**
   - `/workspace deny add <path>` / `/workspace deny remove <path>` / `/workspace deny list` **(미구현 — v3.0 계획)**
   - `/config`와 `/setting`에서도 동일 정보를 읽고 수정할 수 있게 연결

7. **Task 7: 상태바/툴바/Doctor 반영**
   - Host shell, exec shell, trust state를 상태바와 `/status` 또는 doctor 출력에 노출

#### F. Verification Path

- Linux + bash/zsh
  - trust gate에서 Restricted 선택 시 `WriteFile`/`ExecShell` 차단 확인
  - denied root 추가 후 `ReadFile`/`ListDir`까지 차단되는지 확인
- Windows 11 + `cmd.exe`
  - 앱 실행 후 상태바에 `Host: cmd.exe` 또는 동등 정보 노출
  - `ExecShell`이 `pwsh` 또는 `powershell.exe`로 실제 실행되는지 확인
- Windows Terminal + PowerShell
  - host shell과 exec shell이 동일/호환 표기로 안정적으로 노출되는지 확인
- 공통 테스트
  - trust record 저장/복원 테스트
  - denied roots 저장/복원 테스트
  - `/workspace show/trust/deny/clear` 명령 테스트 (v3.7.1 구현분)
  - `/workspace add/remove` 명령 테스트 → **v3.0 구현 후 추가 예정**
  - Restricted 상태 permission deny 테스트
  - denied root permission deny 테스트
  - exec shell fallback 테스트 (`pwsh` 없음 → `powershell.exe`)

---

### 3.8 Phase 18: Multi-Provider Expansion & Advanced Agentic Tools (2026.04)

Phase 18은 LLM 어댑터 지원의 확장(OpenAI, Anthropic, xAI)과 2026년 4월 기준 최신 모델 그라운딩, 그리고 에이전트 성능을 극대화할 수 있는 추가 시스템 도구(ListDir, GrepSearch, FetchURL)를 구현하는 것을 목표로 한다.

#### A. Scope Closure

**In-Scope:**
- `ProviderAdapter` 트레이트 기반의 xAI 및 Anthropic 어댑터 구현 (OpenAI 호환 API 사용 또는 Anthropic Messages API 사용).
- `config.toml` 내 제공자 선택 및 Base URL 지원.
- 2026년 4월 기준 최신 모델 식별자 갱신 (`gpt-5.4`, `claude-4.7`, `grok-4.3-beta` 등).
- 파일 탐색 보조 도구: `ListDir` (디렉터리 브라우징), `GrepSearch` (정규표현식 파일 검색).
- 외부 데이터 수집 보조 도구: `FetchURL` (HTML to Markdown).

**Out-of-Scope:**
- 이미지 입력이나 실시간 음성/오디오(Real-time Audio) 지원 기능은 이번 페이즈에서 제외한다.
- 웹 브라우저 전체 제어(Playwright/Puppeteer) 도구는 포함하지 않고 단순 HTTP Fetch만 다룬다.

#### B. Architectural Decisions

1. **Provider Registry & Adapter Pattern**
   - 기존의 단일화된 Provider 호출 로직을 `crate::providers::registry` 패턴으로 완전히 분리한다.
   - `AnthropicAdapter`는 `api.anthropic.com/v1/messages`를 바라보고, `xAIAdapter`는 `api.x.ai/v1/chat/completions` 등 OpenAI 호환 형식을 바라본다.

2. **Model Grounding (2026.04 Baseline)**
   - OpenAI: `gpt-5.4-pro`, `gpt-5.4-thinking`, `gpt-5.4-instant`, `gpt-5.3-codex`.
   - Anthropic: `claude-4.7` (Opus), `claude-4.6` (Sonnet).
   - xAI: `grok-4.3-beta`, `grok-4.1-fast`.
   - 설정 화면에서 사용자가 선택할 수 있는 Default Model 리스트를 이 기준으로 업데이트한다.

3. **Advanced Tool Definition**
   - **ListDir**: 특정 경로의 디렉터리 구조를 JSON/텍스트로 반환. (Context 예산 절약을 위해 max_depth 및 ignore 패턴 적용).
   - **GrepSearch**: `ripgrep`이나 Rust 내장 `regex`를 사용하여 코드베이스의 특정 패턴을 빠르게 검색. (디렉터리 전체 탐색 시 필수).
   - **FetchURL**: `reqwest` 등으로 URL을 가져오고 HTML 태그를 제거해 Markdown으로 변환. (API 문서 참조 등).

#### C. Typed Contracts

```rust
pub enum ProviderKind {
    OpenRouter,
    OpenAI,
    Anthropic,
    xAI,
    Google,
}

pub struct ProviderConfig {
    pub provider: ProviderKind,
    pub base_url: Option<String>,
    pub api_key_alias: String,
}

pub trait ProviderAdapter {
    fn chat_stream<'a>(
        &'a self,
        api_key: &'a str,
        request: ChatRequest,
        delta_tx: tokio::sync::mpsc::Sender<String>,
    ) -> Pin<Box<dyn Future<Output = Result<ChatResponse, ProviderError>> + Send + 'a>>;
}
```

#### D. Concrete Rules

- Anthropic 어댑터는 OpenAI의 `ChatCompletion` 형식과 다른 `Messages API` 구조와 SSE 스트리밍 청크 구조를 사용하므로, 역/정직렬화 어댑터를 철저하게 구현한다.
- `ListDir` 실행 시 `node_modules`, `target`, `.git` 폴더는 기본으로 무시(ignore)하여 토큰 낭비를 막는다.
- `FetchURL`은 텍스트 파싱 후 결과가 10,000자를 넘어가면 앞뒤 일부만 남기고 `[...truncated]` 처리한다.

#### E. Execution Path

1. **Task 1: Provider Registry 확장**
   - `ProviderKind` enum 갱신 및 `ProviderAdapter` 트레이트 정립.
   - `AnthropicAdapter`, `OpenAIAdapter`, `xAIAdapter` 구현.
2. **Task 2: 2026.04 모델 명세 업데이트**
   - `domain/provider.rs` 또는 `config.toml` 기본값 라인업에 최신 모델 이름 반영.
   - `smlcli config` 및 `/setting` UI에 신규 Provider와 Model 드롭다운 추가.
3. **Task 3: 신규 도구 3종 구현**
   - `ListDir` 구조체 및 Execute 로직 작성.
   - `GrepSearch` 구조체 작성 (`regex` 또는 `ignore` 크레이트 연동).
   - `FetchURL` 구현 (Reqwest + 기본 HTML 파서/마크다운 변환기).
4. **Task 4: 통합 테스트 및 에러 처리**
   - Provider별 API Key 오류 메시지 등 에러 노출 무결성 점검.
   - 각 도구가 샌드박스 정책(Workspace Trust Gate)을 올바르게 존중하는지 테스트.

#### F. Verification Path

- xAI 및 Anthropic 모델을 설정에서 선택하고 "안녕"이라고 테스트 메시지를 보낼 때 정상적으로 SSE 스트리밍이 렌더링되는지 확인한다.
- `FetchURL`을 통해 웹페이지 내용이 Markdown으로 응답에 잘 요약되는지 확인한다.
- `GrepSearch`로 특정 함수명 검색 시 파일 경로와 매치되는 라인이 올바른 포맷으로 리턴되는지 확인한다.

**Future Work (v3.0 이후)**

> 고급 계획/추론 정책(Advanced Planning / Reasoning Policy)은 v3.0 이후 Phase에서 구현 예정.
> AI 런타임이 도구를 요청하더라도 최종 권한은 Permission Engine 정책 레이어에 있음.

### 3.11 Context Optimization (Phase 7: Advanced Compaction)

세션 길이가 길어질수록 컨텍스트 오염을 막기 위해 하이브리드 압축 전략(Hybrid Context Compression)을 도입한다.

1. **지능형 요약 압축 (Intelligent Condensation)**
   - 단순 메시지 드롭을 넘어, 백그라운드 LLM 호출을 통해 이전 대화를 1개의 압축된 System Block으로 요약(`summary + last intent`)하여 앞단에 주입한다.

2. **동적 토큰 임계치 (Dynamic Token Bounds)**
   - 정적 메시지 개수를 탈피하고, `tiktoken` 수준의 토큰 비용을 역산하거나 단어 기준 추정치를 사용한다.
   - 예산의 75% 도달 시 선제적 `compact_context()`가 자동 트리거된다.
   - `/tokens` 명령어를 통해 사용자에게 시각적 사용량을 UI로 제어 권한을 준다.

3. **중요 컨텍스트 핀 지정 (Pinning & Anchor)**
   - `spec.md` 등 핵심 설계 원칙이 담긴 메시지나 사용자가 핀(Pin) 처리한 컨텍스트는 수명 주기를 무한대로 유지하여 압축 대상에서 제외한다.

### 3.12 Extended Prompt Commands (@ and !)

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

### 3.13 Phase 12: Native Structured Tool Call Integration

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

### 3.14 Phase 13: Agentic Autonomy & Architectural Refactoring

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
- **자가 치유 오류 컨텍스트 보존**: Auto-Verify가 LLM에 재전송하는 실패 원문은 UI 요약과 분리한다. 모델에는 `stderr` 우선, `stdout` 보조의 풍부한 실패 컨텍스트를 **최대 1,200자(앞/뒤 보존형)** 로 전달하고, 사용자용 Notice는 240자 요약으로 제한한다.
- **Tree-sitter Repo Map**: `tree-sitter`로 추출한 저장소의 Rust(.rs) AST(함수, 구조체 시그니처) 맵을 시스템 프롬프트에 `[Repo Map]` 헤더로 주입한다. 최대 **8,000바이트**를 넘지 않도록 자른다(Truncate).
- **Repo Map 백그라운드 생성 규칙**: Repo Map 생성은 `tokio::task::spawn_blocking` 워커에서 수행한다. UI/입력 루프는 절대 동기 스캔을 기다리지 않는다. 최근 생성본은 캐시에 유지하고, `WriteFile`/`ReplaceFileContent`/`ExecShell` 이후 stale 처리 후 백그라운드로 재생성한다.
- **Git Checkpoint 안전 정책**: `create_checkpoint()`는 강제 커밋을 수행하지 **않는다**. 워킹 트리가 깨끗한지만 검사하여 `bool`을 반환하고, `true`일 때만 롤백(`git reset --hard HEAD`)이 허용된다. `git clean -fd`는 사용하지 **않으며**, untracked 파일은 어떤 경우에도 삭제되지 않는다.
- **직접 셸 실행(`!`) 정책**: 사용자가 `!` 접두사로 직접 입력한 명령에 대해서도 `safe_to_auto_run: false`를 설정하여 `SafeOnly` 모드의 allowlist 정책을 반드시 존중한다. 블랙리스트와 allowlist 모두 동일하게 적용된다.
- **Linux 샌드박스 백엔드**: Linux의 `ExecShell`은 `bubblewrap(bwrap)` 기반의 실제 프로세스 격리를 사용한다. 호스트 파일시스템은 기본 읽기 전용으로 노출하고, 요청한 작업 디렉터리만 `/workspace`로 쓰기 가능하게 bind mount 한다.
- **HITL 만료 시간**: 승인 대기(`Approval`)는 시작 시각을 기록하고 **5분**이 지나면 자동 `Abort` 처리한다. 만료 시 pending queue와 diff preview를 즉시 비우고, 타임라인/세션에 시스템 알림을 남긴다.

#### 3. Execution Path (실행 및 검증 흐름)

1. **Tool Registry 도입**: `src/tools/` 내부의 기존 도구들(`ReadFile`, `WriteFile`, `ExecShell` 등)을 `Tool` 트레이트 구현체로 일괄 리팩토링. 각 도구의 `is_destructive()` 메서드로 파괴적 여부를 판별한다. `ExecShell`은 기본값(`false`)을 사용하여 쉘 명령이 Git 롤백을 트리거하지 않도록 한다.
2. **Tree-sitter 통합**: `tree-sitter` 크레이트를 도입하여 작업 디렉터리의 Rust 소스의 함수/구조체 시그니처를 추출, `[Repo Map]` 블록으로 `System` 프롬프트에 백그라운드 주입한다. 생성은 blocking worker로 분리하고, 준비된 캐시만 요청에 삽입한다.
3. **Automated Git Checkpoints**: `WriteFile`이나 `ReplaceFileContent` 같은 파괴적 도구(`is_destructive()=true`) 실행 직전, `create_checkpoint()`가 워킹 트리 상태를 검사한다. 변경사항이 없는(clean) 상태에서만 `safe_to_rollback=true`를 반환하며, 도구 실행 실패 시 `git reset --hard HEAD`로 tracked 파일만 복원한다. WIP가 있으면 롤백 자체를 건너뛴다.
4. **Auto-Verify 루프**: 도구 실행 실패 시(`ToolFinished.is_error=true` 또는 `ToolError`), 힐링 프롬프트를 세션에 주입하고 `send_chat_message_internal()`로 LLM에 재전송한다. 이때 도구 스키마(`tools`)를 반드시 포함하여 모델이 후속 도구를 호출할 수 있게 한다. 최대 3회 실패 시 `Idle`로 전환하고 사용자에게 안내한다.
   - 1차/2차 실패는 `Healing { retries: 1|2 }`로 전이하고 재전송을 수행한다.
   - 3차 실패에서는 `Abort` Notice를 남기고 `Idle`로 되돌리며, 추가 재전송을 하지 않는다.
   - `ToolFinished(is_error=true)` 경로에서는 2~4줄 요약이 아니라 `stderr/stdout`의 확장 실패 컨텍스트를 힐링 프롬프트에 주입한다.
5. **Tree of Thoughts UI**: `tui/layout.rs` 타임라인 렌더링에 `depth` 속성 기반 들여쓰기를 적용. `ToolRun`, `Approval`, `Auto-Verify Notice`는 기본적으로 `depth: 1`을 사용하여 메인 응답 아래에 AI의 도구 호출 및 에러 수정 내역(`└─ ⚙️ ExecShell (cargo check) → Error → Retrying...`)을 트리 형태로 시각화한다.
6. **HITL TTL**: 승인 대기 카드 생성 시 `pending_since_ms`를 기록하고, Tick 루프에서 5분 초과 여부를 검사한다. 초과 시 자동 거부 처리 후 시스템 Notice와 세션 메시지를 남긴다.

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

context compaction / session restore / sessions

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

1. CLI Entry Modes: clap 파서 + run/doctor/sessions 서브커맨드
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
    pub query: String,
    pub cursor: usize,
    pub results: Vec<PaletteCommand>,
}

pub struct PaletteCommand {
    pub id: &'static str,
    pub title: &'static str,
    pub category: PaletteCategory,
    pub shortcut_hint: Option<&'static str>,
}

pub enum PaletteCategory {
    Navigation,
    Session,
    Tools,
    Settings,
    Context,
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

### Phase 16: Deep UI Interactivity & Provider Hardening (v0.1.0-beta.26)

이 페이즈는 Phase 15의 블록 기반 TUI 위에 접기/펼치기(Fold/Unfold) 상호작용과 Provider-Specific Tool Call Dialect 최적화를 더하는 것을 목표로 한다.

#### 16.1 Scope Closure
- **목표**: 10줄 이상의 긴 Diff 블록 접기/펼치기, Provider 간 호환성 파편화 해결(Dialect 추상화), 도메인 에러(`ProviderError`) 일원화.
- **성공 기준**: `ReplaceFileContent` 블록의 기본 렌더링이 변경된 줄 수(+N/-M) 요약으로 노출되며 Enter로 토글 가능. 모든 설정 저장 에러가 `ProviderError` 또는 `AppError`로 반환.
- **비목표**: 마우스 드래그를 통한 텍스트 복사, 터미널 밖의 시스템 알림 등은 구현하지 않는다.

#### 16.2 Typed Contracts (동결된 타입 계약)
```rust
// 타임라인 블록의 렌더링 상태 확장을 위한 타입
#[derive(Debug, Clone, PartialEq)]
pub enum BlockDisplayMode {
    Collapsed,
    Expanded,
}

// TimelineBlock 내부 필드 추가
pub struct TimelineBlock {
    // ... 기존 필드 ...
    pub display_mode: BlockDisplayMode,
    pub diff_summary: Option<(usize, usize)>, // (additions, deletions)
}

// Provider 호환성을 맞추기 위한 방언(Dialect) 설정
#[derive(Debug, Clone, PartialEq)]
pub enum ToolDialect {
    OpenAICompat, // 기본 JSON Schema
    Anthropic,    // strict XML/JSON 혼합 구조 (미래 대비)
    Gemini,       // function parameter required fields 제약 엄격
}
```

#### 16.3 Concrete Formulas (동결된 수치 공식)
- **Diff Collapsing Threshold**: `ReplaceFileContent` 등에서 변경 사항(추가/삭제 줄 수의 합)이 **10줄을 초과**(`additions + deletions > 10`)하면 `BlockDisplayMode::Collapsed`를 기본값으로 갖는다.
- **Collapsed View Format**: `[ +{add} lines / -{del} lines ] (Enter 키로 펼치기)` 텍스트를 Muted 스타일로 렌더링.

#### 16.4 Execution Path
1. `src/app/state.rs`: `TimelineBlock`에 `display_mode`, `diff_summary` 필드 및 상태 토글(`toggle_collapse`) 추가.
2. `src/app/mod.rs`: `FocusedPane::Timeline`일 때 현재 선택된 블록에 대해 `Enter` 키 이벤트 처리 추가 (토글 호출).
3. `src/tui/layout.rs`: `TimelineBlockKind::ToolRun(ReplaceFileContent)` 렌더러에서 `display_mode`에 따라 요약 라벨 또는 원본 Diff 노출 로직 추가.
4. (Task 2 & 3 이관) `ProviderError` 마이그레이션.

### Phase 19: v1.0.0 Audit Remediation (완료)

#### 19.1 Scope Closure
**목표(Scope)**: v1.0.0 출시 전 시스템 전반에서 식별된 9개의 핵심 결함을 해결하여 완전한 무상태(Stateless) 아키텍처와 데드락 없는 동시성 이벤트 루프를 보장합니다. 에러 캡슐화, TUI 렌더링 최적화, 리소스 누수 방지를 포함합니다.
**비목표(Non-Scope)**: 신규 AI 모델 연동이나 새로운 TUI 위젯 뷰 추가 등은 이 단계에서 제외됩니다.

#### 19.2 Typed Contracts (동결된 타입 계약)
```rust
// 1. 에러 통합 (src/domain/error.rs)
pub enum SmlError {
    // ... 기존 상태 ...
    InfraError(String),
    IoError(std::io::Error),
}
impl From<std::io::Error> for SmlError { ... }

// 2. Wizard 탭 포커스 강제 (src/tui/widgets/setting_wizard.rs)
pub enum WizardField {
    ApiKey,
    Provider,
    Model,
    SaveButton,
}

// 3. Wizard 검증 오류 (src/app/wizard_controller.rs)
pub enum WizardError {
    MissingRequiredField(String),
}
```

#### 19.3 Concrete Numbers (구체적 수치)
* `MAX_LOG_LINES = 5000`: TUI 로그 메모리 최대 한도 초과 시 FIFO 방식으로 제거.
* `chunk_size = terminal_height - 4`: Windowed Rendering 시 한 프레임에 렌더링될 라인 수.
* `StatusBar Tick = 100ms`: 반응성을 위해 상태바 업데이트 주기를 최소 100ms로 유지.

#### 19.4 Execution & Verification Path
**Phase 1: Core Error & Config**
* 실행: `SmlError` 구현 및 `SessionLogger`에 `BufWriter<File>` 도입 후 Drop 구현.
* 검증: `lsof -p <PID>`로 반복 도구 실행에도 파일 핸들이 누수되지 않음을 증명.

**Phase 2: Logic & Security**
* 실행: `glob` 크레이트로 `is_dangerous` 블랙리스트(`*`, `../`) 추가 및 Wizard 빈 필드 상태 전이 차단.
* 검증: `smlcli exec "rm -rf .git/*"` 실행 시 권한 거부 출력 확인.

**Phase 3: Runtime & Concurrency**
* 실행: `ToolRuntime::execute()` 반환형 변경 및 `tokio_util::sync::CancellationToken` 도입으로 select race 구현.
* 검증: 도구 실행 중 `Ctrl+C` 입력 시 데드락 없이 즉시 캔슬 로그 출력 증명.

**Phase 4: TUI & UX**
* 실행: `inspector_tabs.rs`에 윈도우 기반 렌더링 구현 및 상태바 즉각 갱신(`StatusBar::clear()`).
* 검증: 20,000줄의 stdout 로그를 생성하는 스크립트 실행 시 UI 프레임 드랍이 발생하지 않음을 증명.

### Phase 25: Ultimate Polish & Security Hardening (v1.7.0)

#### 25.1 Scope Closure
- **목표**: 극한 환경(네트워크 지연, 디스크 용량 부족, 잘못된 터미널 리사이즈)에서의 시스템 무결성 보장 및 샌드박스 공격 완전 방어.
- **성공 기준**: 디스크 Full(`ENOSPC`) 발생 시 패닉 없이 기존 설정 보존, LLM API 타임아웃 발생 시 백오프 재시도 성공, `ExecShell` 서브 프로세스 복귀 후 TUI 고스팅(Ghosting) 원천 차단.

#### 25.2 Typed Contracts (동결된 타입 계약)
```rust
// 타임아웃 및 재시도 상태 로직 (app/chat_runtime.rs)
pub struct RetryPolicy {
    pub max_retries: usize,
    pub base_delay_ms: u64,
}

// 터미널 잔상 강제 클리어 (app/state.rs)
pub struct UiState {
    // ... 기존 필드 ...
    pub force_clear: bool,
}
```

#### 25.3 Concrete Rules
- **문자열 슬라이싱 안전성**: `unicode-width`를 활용하여 바이트 단위 슬라이싱에서 발생하는 멀티바이트(한글/이모지) 깨짐과 패닉 방지.
- **Path Traversal 방어**: `std::fs::canonicalize`를 사용하여 심볼릭 링크 및 `../` 공격을 추적하고, 최종 경로가 `workspace_root`를 벗어나면 무조건 차단.
- **네트워크 강건성**: `tokio::time::timeout` 60초 제한 및 지수 백오프(Exponential Backoff) 적용.
- **디스크 쓰기 안전성**: `session_log.rs` 및 `config_store.rs`에서 `std::io::ErrorKind::StorageFull` 매칭 시 `InfraError`로 승격하여 TUI 렌더링 유지.
- **잔상 방지(Ghosting)**: `ToolFinished(ExecShell)` 수신 시 `force_clear = true`를 세팅하고 다음 draw 틱에서 `terminal.clear()` 및 `cursor::Show`, `cursor::MoveTo(0, 0)` 수행.

#### 25.4 Execution Path
1. `src/tui/widgets/inspector_tabs.rs`: `unicode-width` 기반의 문자열 자르기 로직 적용.
2. `src/tools/file_ops.rs`: 파일 도구에 `std::fs::canonicalize` 기반의 경로 검증 가드 추가.
3. `src/app/chat_runtime.rs`: LLM 호출에 지수 백오프 루프 및 60초 타임아웃 래퍼 적용.
4. `src/infra/session_log.rs` / `config_store.rs`: I/O 쓰기 단계에서 `StorageFull` 에러 매칭 및 `InfraError` 맵핑.
5. `src/tui/terminal.rs` 및 `src/app/mod.rs`: `TerminalGuard::clear_and_reset` 도입 및 `force_clear` 이벤트 루프 연동.

### Phase 26: The Final Polish & Stability (v1.8.0)

#### 26.1 Scope Closure
- **목표**: 스트리밍 및 파일 처리의 엣지 케이스 무결성 확보(이진 데이터 가드, 마스킹 누수 차단, Git 충돌 가드) 및 대화면 TUI 렌더링 최적화.
- **성공 기준**: 바이너리 파일 리딩 시 TUI 크래시 원천 차단, 스트리밍 경계(Chunking)에서도 민감키(API Key) 100% 마스킹, 진행 중인 Git 충돌(Merge/Rebase) 시 체크포인트 자동 중단, 대화면 로그 렌더링 CPU 점유율 최적화.

#### 26.2 Typed Contracts (동결된 타입 계약)
```rust
// TUI 로그 라인 렌더링 캐싱 구조 (tui/widgets/inspector_tabs.rs)
use std::collections::HashMap;
use ratatui::text::Line;

pub struct RenderCache {
    pub lines_cache: HashMap<usize, Line<'static>>, // index -> styled line
    pub is_dirty: bool,
}

// 스트리밍 마스킹용 윈도우 상태 (app/tool_runtime.rs)
pub struct StreamingMasker {
    // 이전 청크의 마지막 N 바이트를 보관하여 스트림 경계 단어 보존
    pub trailing_buffer: String,
    pub max_match_len: usize,
}
```

#### 26.3 Concrete Formulas & Rules
- **이진 파일 감지율**: 파일 읽기 전 최초 1024 바이트 청크를 읽어 `\0` (null byte)가 포함되어 있거나, non-printable 문자가 30%를 초과할 경우 읽기 중단 및 경고.
- **상태 기반 마스킹(Stateful Masking)**: API Key 마스킹을 위해, 스트리밍 수신 시 현재 청크와 앞선 청크의 뒷부분(키 길이만큼)을 이어 붙인 윈도우 내에서 마스킹 정규식/패턴 매칭을 수행.
- **Git 충돌 안전장치**: `git_checkpoint.rs`에서 `.git/MERGE_HEAD`, `.git/REBASE_HEAD`, `.git/CHERRY_PICK_HEAD` 파일 중 하나라도 존재 시, 작업 강제 중단 및 에러 반환.
- **렌더링 캐시 무효화**: 타임라인이나 로그 배열(Vec)에 데이터가 append/update 되는 시점에만 `RenderCache.is_dirty = true`로 세팅, 그렇지 않은 Tick(예: 커서 이동, 리사이즈 없음)에서는 캐시 된 `Line` 반환.

#### 26.4 Execution & Verification Path
1. **Phase 1 (Safety)**: `src/tools/file_ops.rs`에 1KB 헤더 바이너리 감지 로직 추가, `src/tools/git_checkpoint.rs`에 `.git/*_HEAD` 존재 검사 로직 추가.
   * 검증: 바이너리 파일 대상 `ReadFile` 시도 시 거부 확인, `git merge --no-commit` 상태에서 도구 호출 시 체크포인트 중단 확인.
2. **Phase 2 (Security & Performance)**: `src/app/tool_runtime.rs` 또는 스트리밍 로그 처리부에 Sliding Window 방식의 API Key 마스킹 구현. `src/tui/widgets/inspector_tabs.rs`에 `HashMap` 기반 라인 캐싱 도입.
   * 검증: 10바이트씩 청크로 잘린 API Key 문자열이 완벽히 마스킹되는지 단위 테스트(Unit Test). 100행 이상의 렌더링 시 CPU 사용량 프로파일링.
3. **Phase 3 (Stability)**: `src/providers/` 어댑터들이 Provider 간 변경 시 호환되지 않는 시스템 메시지 등 내부 찌꺼기 메시지를 정제하는 추상화 단계(Generic Message Format to Specific API) 보강.
   * 검증: Anthropic 대화 도중 OpenAI로 Provider 변경 시 에러 없이 이어서 질문 전송 성공 확인.

### Phase 27: Final Pre-Launch Hardening (v1.9.0)

#### 27.1 Scope Closure
- **목표**: 하위 프로세스 그룹의 완전한 제어, 환경 변수의 오염 방지, 대규모 결과값의 메모리 효율성, 터미널 인터페이스의 표준 준수(OSC 시퀀스).
- **성공 기준**: 쉘 스크립트 실행 후 남는 좀비/고아 프로세스 완전 차단, 도구에 필수 환경 변수만 격리 전달, 수십 MB의 파일 출력 시 메모리 크래시 방지 및 터미널 타이틀 동기화 확인.

#### 27.2 Typed Contracts (동결된 타입 계약)
```rust
// 환경변수 화이트리스트 (app/tool_runtime.rs)
pub const ENV_WHITELIST: &[&str] = &[
    "PATH", "HOME", "USER", "TERM", "LANG",
];

// 도구 실행 출력 캡핑 크기 상수 (domain/tool_result.rs)
pub const MAX_STDOUT_BYTES: usize = 5 * 1024 * 1024; // 5MB
```

#### 27.3 Concrete Formulas & Rules
- **Process Grouping**: `tokio::process::Command` 생성 시 `.process_group(0)`을 부여하여 독립 그룹 생성. `child.kill()` 대신 `libc::kill(-pgid, libc::SIGKILL)`을 호출하여 프로세스 트리 전체를 소멸시킴 (Unix 전용).
- **Environment Isolation**: `.env_clear()` 호출 후 `ENV_WHITELIST`에 정의된 시스템 변수 및 사용자가 Smlcli Config에 명시한 변수만 넘기도록 화이트리스트 필터링 적용.
- **Size Capping**: 실행 결과(`stdout`, `stderr`)가 `MAX_STDOUT_BYTES`를 넘으면 초과분을 잘라내고 끝부분에 `... [중략: N MB] ...` 메타데이터 문자열 추가 후 저장하여 모델과 UI에 전달.
- **Terminal OSC 시퀀스**: 도구 실행 시작 시 `\x1b]0;[smlcli] Executing: <tool_name>\x07` 및 `\x1b]9;4;1;100\x07` (Progress) 전송, 완료 시 `\x1b]0;smlcli\x07` 및 `\x1b]9;4;0;0\x07` 복구.

#### 27.4 Execution & Verification Path
1. **Phase 1 (Critical Stability)**: `src/tools/shell.rs` 및 `src/app/tool_runtime.rs`에 `cfg(unix)` 기반 PGID 전파 로직 및 `env_clear()` 기반 환경 변수 격리 기능 구현.
   * 검증: `sh -c "sleep 100 & sleep 100"` 실행 후 취소 시 백그라운드 프로세스가 남지 않는지 확인.
2. **Phase 2 (Scalability)**: `src/domain/tool_result.rs` 및 `app/tool_runtime.rs`에 스트리밍 바이트 카운터 누적 및 `MAX_STDOUT_BYTES` 초과분 절단 구현. `src/tui/terminal.rs` 등에 OSC 시퀀스 추가.
   * 검증: 100MB 텍스트 덤프 시 메모리 폭주 없이 5MB 캡핑 적용 확인. 터미널 탭 제목 변경 확인.
3. **Phase 3 (Compliance)**: `spec.md`, `README.md` 등 전체 문서 동기화 및 렌더링 검수.

### Phase 28: Final Touch & System Optimization (v2.0.0)

#### 28.1 Scope Closure
- **목표**: 대규모 출력 절단 시 메타데이터 보존, 터미널(PTY) ANSI 보존, Git 체크포인트 저장소 관리 정책 도입, 그리고 멀티 프로세스 동시 실행 시의 설정 파일 동시성 충돌 방지 및 쉘 네이티브 PATH 탐색 지원.
- **성공 기준**: 5MB 초과 출력 시 LLM이 잘림 사실을 인지, 파일 쓰기 락 기반 설정 깨짐 방지, Git Checkpoint 레퍼런스(`refs/smlcli/checkpoints/`) 50개 유지(나머지 Prune), 쉘 고유 환경 PATH 동적 복원.

#### 28.2 Typed Contracts (동결된 타입 계약)
```rust
// 체크포인트 최대 유지 개수 (tools/git_checkpoint.rs)
pub const MAX_GIT_CHECKPOINTS: usize = 50;

// 출력 절단 메타데이터 추가 필드 (domain/tool_result.rs)
pub struct ToolResult {
    pub tool_name: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub is_error: bool,
    pub tool_call_id: Option<String>,
    pub is_truncated: bool,
    pub original_size_bytes: Option<usize>,
}
```

#### 28.3 Concrete Formulas & Rules
- **Truncation Metadata**: 출력 버퍼가 `MAX_STDOUT_BYTES`를 넘으면, 데이터를 잘라내는 동시에 `[SYSTEM: 이 결과는 너무 길어서 일부가 절단되었습니다. 원래 크기: {size} bytes]`라는 메타 정보를 LLM Context 상/하단에 강제로 추가.
- **PTY & ANSI Preservation**: `tokio::process` 스트림 방식을 유지하되, TTY 에뮬레이션을 모방하여 ANSI 색상 코드를 잃지 않도록 관련 힌트 전달 또는 표준 출력 제어 유지.
- **Git Checkpoint Prune**: 새로운 체크포인트 생성 전, `git for-each-ref --sort=-committerdate refs/smlcli/checkpoints/`를 조회하여 50개가 넘는 오래된 참조들을 `git update-ref -d`로 자동 삭제.
- **File Locking**: `fs2` 혹은 `tempfile::NamedTempFile` 기반의 Write-and-Rename 전략(원자적 쓰기)을 사용하여 두 개 이상의 `smlcli` 인스턴스가 `config.toml`을 동시에 덮어쓰지 않도록 락 처리.
- **Shell-Native PATH Discovery**: 환경 변수 격리 시 `sh -c "echo $PATH"`를 1회 실행하여 현재 사용자 기본 쉘의 실제 PATH(e.g. nvm, cargo/bin)를 추출한 뒤, 도구 실행 컨텍스트에 주입.

#### 28.4 Execution & Verification Path
1. **Phase 1 (Stability & Security)**: 설정 파일 원자적 쓰기(Write-and-Rename 및 File Lock 적용) 및 쉘 PATH 동적 탐색 기능(`src/app/tool_runtime.rs` 및 `config_store.rs`) 구현.
   * 검증: 두 개의 창에서 동시에 설정 저장을 시도할 때 파일 깨짐이 없는지, `cargo`나 `nvm` 등 커스텀 PATH 환경이 보존되는지 확인.
2. **Phase 2 (Functionality)**: PTY ANSI 보존 및 대규모 출력 절단 메타데이터(`ToolResult::is_truncated`) 구현.
3. **Phase 3 (Maintenance)**: Git 체크포인트 자동 정리(Prune 로직 50개 제한) 적용 및 검수.

### Phase 29: The Final Polishing (v2.1.0)

#### 29.1 Scope Closure
- **목표**: 대규모 RepoMap 토큰 압축, 리사이징 시 스크롤 위치 앵커링, Git 예외 상태 안전 처리, 서브 프로세스 Stdin 하이재킹 차단, Actionable Error 메시지 적용.
- **성공 기준**: 1,000개 파일 프로젝트에서 5,000 토큰 이하의 RepoMap 유지, 창 크기 변경 시 읽던 로그 라인이 고정됨, Detached HEAD에서 체크포인트 무결성 유지, 파이썬 `input()` 실행 시 UI 먹통 방지.

#### 29.2 Typed Contracts (동결된 타입 계약)
```rust
// Actionable Error 확장을 위한 구조 (domain/error.rs)
pub struct ActionableError {
    pub message: String,
    pub suggestion: Option<String>,
}

// 스크롤 앵커 상태 관리 (tui/widgets/inspector_tabs.rs)
pub struct ScrollAnchor {
    pub absolute_line_index: usize,
    pub active: bool,
}
```

#### 29.3 Concrete Formulas & Rules
- **Context Compression**: RepoMap 구성 시 최근 접근/수정된 파일 위주로 토큰을 계산하며, 임계치(예: 4000 토큰) 초과 시 하위 트리를 생략(`...`) 처리.
- **Scroll Anchoring**: 창 변경 이벤트 감지 시 `scroll_offset = anchor_absolute_index`를 역계산하여 뷰포트 상단 줄을 고정.
- **Git State Detection**: `git rev-list -n 1 --all` 및 `git rev-parse --is-inside-work-tree`를 선행하여, 브랜치가 없는 경우 `HEAD`나 `Initial`이라는 Fallback Name을 사용.
- **Strict Pipe (Stdin Hijacking)**: `tokio::process::Command`에 `.stdin(Stdio::null())`을 강제 주입하여 터미널 키보드 이벤트를 TUI로 격리.
- **Actionable Errors**: 에러 메시지 포맷을 `[원인] (제안: [행동 유도])` 형태로 표준화하여 사용자가 직관적으로 후속 조치를 할 수 있도록 함.

#### 29.4 Execution & Verification Path
1. **Phase 1 (Logic & Stability)**: `src/tools/git_checkpoint.rs`에 초기화되지 않은 저장소 및 Detached HEAD 대응. `src/tools/shell.rs`에 `.stdin(Stdio::null())` 강제 추가.
2. **Phase 2 (UX/UI)**: `src/tui/widgets/inspector_tabs.rs`에 스크롤 앵커 로직 추가 및 `src/domain/error.rs` 에러 포맷 리팩토링.
3. **Phase 3 (Efficiency)**: `src/domain/repo_map.rs`에 토큰 제한 기반 트리 생략 알고리즘 적용.

### Phase 30: The Ultimate Hardening (v2.2.0)

#### 30.1 Scope Closure
- **목표**: 설정 파일 버전 관리 및 마이그레이션 적용, 원자적 쓰기 실패 시 임시 파일(.tmp) 정리, `doctor` 커맨드를 통한 자가 진단 기능 구현, TUI 로그/코드 클립보드 복사(Copy-to-Clipboard) 연동, Windows 환경 프로세스/경로 호환성 확보.
- **성공 기준**: `.tmp` 찌꺼기 파일이 앱 시작 시 지워지고, `smlcli doctor`를 통해 환경 상태를 리포트하며, 버전 0의 설정이 버전 1로 안전히 승격되고, `arboard`를 통해 TUI에서 복사가 정상 동작하며, 윈도우 환경에서도 안전하게 빌드/프로세스 제어가 이루어짐.

#### 30.2 Typed Contracts (동결된 타입 계약)
```rust
// 설정 파일 버전 관리 (domain/settings.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_version")]
    pub version: u32,
    // ...
}
fn default_version() -> u32 { 1 }

// 시스템 진단 리포트 (infra/doctor.rs)
pub enum DiagnosticStatus {
    Ok,
    Warn(String),
    Error(String),
}
pub struct DoctorReport {
    pub api_status: DiagnosticStatus,
    pub git_status: DiagnosticStatus,
    pub config_status: DiagnosticStatus,
    pub term_status: DiagnosticStatus,
}
```

#### 30.3 Concrete Formulas & Rules
- **Config Migration**: 파일 읽기 시 파싱이 성공하면 `version` 필드를 검사하고, 1 미만이면 `migrate()`를 실행하여 최신 구조로 변환 후 덮어쓴다.
- **Atomic Cleanup**: `ConfigStore` 초기화 단계에서 워크스페이스 내 `*.tmp` 파일을 `glob`이나 `fs::read_dir`을 사용해 무조건 삭제.
- **Doctor Check**: `API_KEY` 존재 유무, `git --version`, `.settings.json` 쓰기 권한, `is_terminal` 여부를 검사.
- **Windows Compat**: `#[cfg(unix)]`와 `#[cfg(windows)]` 속성 매크로를 이용해, Unix는 `nix`를 통해 process group kill을 사용하고, Windows는 `taskkill /F /T /PID` 혹은 단순 `child.kill()`로 우회한다.

#### 30.4 Execution & Verification Path
1. **Phase 1 (Compatibility & Safety)**: OS별 프로세스 관리 분기(`src/tools/shell.rs`) 및 임시 파일 정리 로직(`src/infra/config_store.rs`).
2. **Phase 2 (Scalability)**: 설정 파일 버전 관리 및 마이그레이션 로직 구현(`src/domain/settings.rs`).
3. **Phase 3 (UX & Tooling)**: 클립보드 연동(`src/tui/widgets/inspector_tabs.rs` 등) 및 `smlcli doctor` 진단 모드(`src/main.rs`).

### Phase 31: The Final Polish & Resilience (v2.3.0)

#### 31.1 Scope Closure
- **목표**: 마이그레이션 실패 시의 안전성 확보, 클립보드 피드백 제공, Doctor 모드 네트워크 의존성 분리, 환경 변수 화이트리스트 확장, 그리고 RepoMap 캐싱 도입.
- **성공 기준**: 잘못된 마이그레이션에도 원본 `.bak` 보존, `y` 키 입력 시 2초간 "복사 완료" 토스트 알림, `doctor` 커맨드 5초 타임아웃, `allowed_env_vars` 동작, RepoMap 80% 이상의 로딩 시간 단축.

#### 31.2 Typed Contracts (동결된 타입 계약)
```rust
// 환경 변수 확장 (domain/settings.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSettings {
    #[serde(default)]
    pub allowed_env_vars: Vec<String>,
    // ...
}

// 클립보드 토스트 알림 상태 (app/state.rs)
pub struct ToastNotification {
    pub message: String,
    pub expires_at: std::time::Instant,
}
```

#### 31.3 Concrete Formulas & Rules
- **Backup & Rollback**: `config_store::load_config`에서 `settings.migrate()` 시도 전 원본을 `config.toml.bak`으로 복사. 실패 시 `rename`으로 복원.
- **Doctor Timeout**: `check_api()` 호출 등을 `tokio::time::timeout(Duration::from_secs(5), ...)`로 래핑하여 블로킹 방지.
- **Configurable Whitelist**: `shell.rs`에서 환경 변수를 세팅할 때 `settings.allowed_env_vars`를 추가로 순회.
- **RepoMap Caching**: `.gemini/tmp/repo_map_cache.json`에 `(mtime_sum, map_string)`을 직렬화하여 저장하고, 빌드 전 디렉터리의 mtime 총합(또는 파일 개수+크기 hash)과 비교하여 일치 시 캐시 사용.

#### 31.4 Execution & Verification Path
1. **Phase 1 (Resilience)**: `src/infra/config_store.rs`의 `.bak` 백업 로직 및 `src/infra/doctor.rs` 타임아웃 래핑.
2. **Phase 2 (UX & Flexibility)**: `src/app/state.rs` 토스트 상태 추가, `src/app/mod.rs` 알림 설정, `src/tui/layout.rs` 렌더링. `src/domain/settings.rs` 및 `src/tools/shell.rs`의 환경 변수 연동.
3. **Phase 3 (Optimization)**: `src/domain/repo_map.rs` 파일 해싱 및 디스크 캐싱 구현.

### Phase 35: System Hardening & Metadata (v3.7.1)

#### 35.1 Scope Closure
- **목표**: 터미널 로케일 문제 완화(ASCII 대체), 비동기 이벤트 큐 안정화(Ordered Aggregation), 대용량 세션 로그 파싱 안정성, 백그라운드 자식 프로세스 누수 방지, 배포 바이너리 메타데이터 추적성 확보.
- **성공 기준**: `LANG` 환경 변수나 설정에 의해 보더가 ASCII로 변환됨, 병렬 도구 결과가 순서대로 로그에 남음, `sysinfo`가 `--clean-orphans`로 SMLCLI_PID 기반 고아 프로세스를 정리함, `doctor` 출력에 빌드 정보(shadow-rs) 포함됨.

#### 35.2 Typed Contracts (동결된 타입 계약)
```rust
// 환경 설정: ASCII 보더 활성화 옵션
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSettings {
    #[serde(default)]
    pub use_ascii_borders: bool,
    // ...
}

// 런타임: 병렬 도구 실행 결과 순서 정합성 보장 큐
pub struct RuntimeState {
    pub pending_tool_outcomes: Vec<(usize, ToolOutcome)>,
    // ...
}
```

#### 35.3 Concrete Formulas & Rules
- **Ordered Aggregation**: `Action::ToolFinished` 도달 시 큐(`pending_tool_outcomes`)에 `(tool_index, outcome)`을 넣은 후, 현재 `active_tool_calls` 내 완료된 도구 인덱스가 순서대로 나열되는지 확인하고, 일치하는 순서부터 flush.
- **ASCII Fallback**: `ratatui::symbols::border::Set` 반환 함수에서 `LANG` 내 `UTF-8` 미포함 또는 `use_ascii_borders == true` 시 `Set { horizontal_top: "-", vertical_left: "|", ... }` 반환.
- **Process Reaper**: `sysinfo::System::refresh_processes()` 후 `process.environ()` 내 `SMLCLI_PID`가 존재하고 현재 PID와 다르면 부모 잃은 고아 프로세스로 판정하여 `kill()`.
- **Build Metadata**: `build.rs`를 통해 `shadow_rs`를 호출하여 빌드 시 커밋 해시와 빌드 일시를 주입, `doctor` 커맨드가 `env!("CARGO_PKG_VERSION")` 및 `shadow::build::*` 참조.

#### 35.4 Execution & Verification Path
1. **Phase 1 (Data & System)**: `infra/process_reaper.rs` (sysinfo) 구현 및 `main.rs` (스레드 스폰 / doctor CLI 분기) 적용.
2. **Phase 2 (DevOps)**: `Cargo.toml`에 `shadow-rs` 추가, `build.rs` 작성, `infra/doctor.rs` 진단 리포트 출력 포맷 개선.
3. **Phase 3 (UX & Concurrency)**: `app/mod.rs` (ordered aggregation), `tui/widgets/mod.rs` (ASCII helper), `layout.rs` 렌더링 교체, `infra/session_log.rs` `BufReader::lines` 적용.

---

## v3.0 Roadmap — 경쟁력 확보 로드맵

> **배경**: v3.7.1 평가에서 도출된 5대 약점(Git 통합 부재, Provider 확장성, OS-level 샌드박스 없음, 에코시스템 미비, 배포 파이프라인 없음)을 순차적으로 해소한다.
> **의존성 순서**: Phase 40 → 41 → 42 → 43 → 44 → 45 (각 Phase는 이전 Phase의 인프라에 의존).

### Phase 40: Git-Native Integration (v3.0.0)

#### 40.1 Scope Closure
- **목표**: Aider 수준의 Git 네이티브 워크플로 구현. 모든 파일 변경이 자동 커밋되고, 실패 시 원자적 롤백이 가능한 구조.
- **비목표**: GitHub/GitLab API 연동, PR 자동 생성은 v3.1 이후.
- **성공 기준**: WriteFile/ReplaceFileContent 성공 시 자동 커밋 생성, `/undo` 명령어로 마지막 AI 편집 되돌리기, 체크포인트 히스토리 TUI 표시.

#### 40.2 Typed Contracts
```rust
/// Git 통합 설정 (config.toml에 영속화)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitIntegrationConfig {
    /// 도구 실행 성공 시 자동 커밋 여부 (기본: false, 명시적 opt-in 필요)
    pub auto_commit: bool,
    /// 커밋 메시지 접두사 (기본: "smlcli: ")
    pub commit_prefix: String,
    /// 자동 커밋 대상 도구 목록
    pub commit_tools: Vec<String>, // ["WriteFile", "ReplaceFileContent", "DeleteFile"]
}

/// 체크포인트 엔트리 (TUI 히스토리 표시용)
pub struct CheckpointEntry {
    pub ref_name: String,
    pub timestamp: u64,
    pub tool_name: String,
    pub files_changed: Vec<String>,
    pub commit_hash: String,
}
```

#### 40.3 구현 가이드

**Step 1: GitCheckpointTool 레지스트리 등록**
- 기존 `tools/git_checkpoint.rs`의 함수들을 `Tool` trait 구현체로 래핑.
- `GLOBAL_REGISTRY`에 등록. `is_write_tool()` 및 guard 테스트의 `known_unregistered`에서 제거.
- `check_permission()`: git repo가 아니면 `Deny`, Trusted workspace면 `Allow`.

**Step 2: 자동 커밋 엔진 (`infra/git_engine.rs`)**
```rust
// 핵심 API
pub struct GitEngine;
impl GitEngine {
    /// 변경된 파일을 stage하고 자동 커밋 생성
    /// 커밋 메시지: "{prefix}{tool_name}: {파일 경로 요약}"
    pub fn auto_commit(cwd: &str, tool_name: &str, files: &[&str], prefix: &str) -> Result<String>;

    /// 마지막 smlcli 커밋을 되돌림 (git revert --no-edit)
    pub fn undo_last(cwd: &str, prefix: &str) -> Result<String>;

    /// smlcli가 생성한 커밋 히스토리 조회
    pub fn list_history(cwd: &str, prefix: &str, limit: usize) -> Result<Vec<CheckpointEntry>>;
}
```

**Step 3: ToolFinished 후 자동 커밋 훅**
- `app/mod.rs`의 `Action::ToolFinished` 핸들러에서:
  1. `is_write_tool() && !res.is_error && git_config.auto_commit` 조건 확인.
  2. **[v2.5.1]** `res.affected_paths`가 비어있지 않은 경우에만 `GitEngine::auto_commit(&file_refs)` 호출.
     - `affected_paths`가 비어있으면 (ExecShell 등) auto-commit skip → 사용자 WIP 보호.
     - `WriteFile`/`ReplaceFileContent` 성공 시 canonical path가 `affected_paths`에 자동 기록.
  3. 커밋 성공 시 타임라인에 `GitCommit` 블록 추가 (커밋 해시 표시).
  4. 실패 시 Toast 경고만 표시 (비차단).

**Step 4: `/undo` 슬래시 명령어**
- `commands/mod.rs`에 `/undo` 라우팅 추가.
- `GitEngine::undo_last()` 호출 → 타임라인에 Revert 블록 표시.
- **[v2.5.1]** 연속 `/undo`로 여러 커밋 되돌리기 가능 (스택 방식):
  1. HEAD가 `smlcli:` prefix 자동 커밋이면 직접 `git revert HEAD`.
  2. HEAD가 Revert 커밋이면, `git log -50`에서 아직 revert되지 않은 가장 최근 smlcli 커밋을 탐색 후 `git revert --no-edit <hash>`.
  3. 되돌릴 수 있는 smlcli 커밋이 없으면 에러 반환.

**Step 5: Inspector Git 히스토리 탭**
- Inspector 패널에 `Git` 탭 추가.
- **[v2.5.1]** `GitEngine::list_history(cwd, prefix, limit)` 호출 시 prefix 인자로 `git log --grep=^{prefix}` 필터 적용.
  - smlcli 생성 커밋만 표시, 사용자 커밋 분리.
- 선택 시 해당 커밋의 diff 프리뷰 표시.

#### 40.4 테스트 요구사항
- `tempfile::TempDir` + `git init`으로 임시 repo 생성 후 auto_commit/undo/list_history E2E 테스트.
- 비-git 디렉토리에서 graceful skip 검증.
- WIP가 있는 상태에서 auto_commit이 `affected_paths` 대상만 커밋하는지 검증.
- 연속 `/undo`에서 Revert 커밋 이후에도 올바른 smlcli 커밋을 찾아 revert하는지 검증.
- guard 테스트에서 `known_unregistered`에서 `GitCheckpoint` 제거 확인.

---

### Phase 41: Provider 확장성 (v3.1.0)

#### 41.1 Scope Closure
- **목표**: OpenAI 호환 엔드포인트(Ollama, LM Studio, vLLM 등)를 `base_url + api_key`만으로 사용 가능하게 하고, 커스텀 provider 등록 UX 제공.
- **비목표**: MCP 기반 도구 확장은 Phase 43에서 별도 처리.
- **성공 기준**: `/provider add local http://localhost:11434 openai` 명령어로 커스텀 provider 추가 (positional 파서: `<id> <base_url> [dialect] [auth_type] [auth_header_name]`). 기존 3사 + 커스텀 N개 동시 사용 가능.

#### 41.2 Typed Contracts
```rust
/// 커스텀 provider 정의 (config.toml에 영속화)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomProviderConfig {
    pub id: String,                       // 고유 식별자 (예: "local-ollama")
    pub base_url: String,                 // API 엔드포인트 기본 URL
    pub auth_type: String,                // "Bearer", "None", "CustomHeader"
    pub auth_header_name: Option<String>, // CustomHeader 사용 시 헤더 이름
    pub dialect: ToolDialect,             // API 호환 타입 (OpenAICompat | Anthropic | Gemini)
}

/// [v2.5.2] 어댑터 수준 인증 전략
#[derive(Debug, Clone)]
pub enum AuthStrategy {
    Bearer,              // 표준 Bearer 토큰 인증 (기본값)
    None,                // 인증 없음 (Ollama, LMStudio 등 로컬 모델)
    CustomHeader(String), // 커스텀 헤더 이름으로 API 키 전달
}
```

#### 41.3 구현 가이드

**Step 1: `ProviderKind` 확장**
- 기존 `ProviderKind` enum에 `Custom(String)` 변형 추가.
- `PersistedSettings`에 `custom_providers: Vec<CustomProviderConfig>` 필드 추가.
- `get_adapter()` 함수에서 `Custom` → `custom_adapters` HashMap 조회.

**Step 2: `OpenAICompatAdapter` + `AuthStrategy` 주입**
- **[v2.5.2]** `OpenAICompatAdapter::with_auth(base_url, auth_strategy)` 생성자로 인증 전략 주입.
- `apply_auth()` 헬퍼로 모든 HTTP 요청에 동적 인증 적용 (Bearer/None/CustomHeader).
- `register_custom_providers()`에서 `config.auth_type` → `AuthStrategy` 변환:
  - `"none"` → `AuthStrategy::None` (로컬 모델용, 헤더 없이 전송)
  - `"customheader"` → `AuthStrategy::CustomHeader(config.auth_header_name)`
  - 기타 → `AuthStrategy::Bearer` (기본)

**Step 3: `/provider add` 및 `/provider remove` 명령어**
- `/provider add <id> <base_url> [dialect] [auth_type] [auth_header_name]` 문법으로 커스텀 provider 등록.
- `config.toml` 영속화 후 재시작 시 `register_custom_providers()` 자동 호출.
- `/provider list`에서 기본 5사 + 커스텀 N개 통합 표시.

**Step 4: 모델 Fetch 호환**
- `fetch_models()` 호출 시 `/v1/models` 엔드포인트 사용. 실패 시 빈 목록 반환 (수동 입력 허용).
- Ollama는 `/api/tags` 형식이므로 `ToolDialect::Ollama` 추가 고려 (또는 v3.2 확장).

#### 41.4 테스트 요구사항
- Mock HTTP 서버(wiremock-rs)로 커스텀 provider CRUD + chat + fetch_models E2E 테스트.
- `config.toml` round-trip 직렬화 검증.
- 잘못된 base_url에서 timeout 그레이스풀 실패 검증.

---

### Phase 42: OS-Level Sandbox (v3.2.0)

#### 42.1 Scope Closure
- **목표**: Linux에서 bubblewrap(`bwrap`) 기반 프로세스 격리, Windows에서 Job Object 기반 리소스 제한을 ExecShell에 적용.
- **비목표**: Docker 컨테이너 통합은 v4.0 이후.
- **성공 기준**: `ExecShell` 실행 시 workspace 외부 파일시스템 접근이 커널 레벨에서 차단됨. 네트워크 egress 제한 옵션 제공.

#### 42.2 Typed Contracts
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enabled: bool,           // 기본: false (opt-in)
    pub allow_network: bool,     // 기본: true
    pub readonly_paths: Vec<String>,  // ["/usr", "/lib", "/bin"]
    pub writable_paths: Vec<String>,  // ["{workspace}"]
}

pub enum SandboxBackend {
    Bubblewrap,  // Linux: bwrap
    JobObject,   // Windows: CreateJobObject
    None,        // 미지원 플랫폼 또는 비활성화
}
```

#### 42.3 구현 가이드

**Step 1: 샌드박스 백엔드 감지 (`infra/sandbox.rs`)**
- `which("bwrap")` 존재 여부로 Linux 샌드박스 가용성 판단.
- Windows: `windows-sys` 크레이트로 Job Object API 접근.
- `doctor` 커맨드에 샌드박스 가용성 표시.

**Step 2: Linux bubblewrap 래퍼**
```rust
pub fn wrap_command_bwrap(cmd: &str, config: &SandboxConfig, cwd: &str) -> Command {
    let mut bwrap = Command::new("bwrap");
    // 읽기 전용 바인드 마운트
    for path in &config.readonly_paths {
        bwrap.args(["--ro-bind", path, path]);
    }
    // 쓰기 가능 마운트 (workspace만)
    for path in &config.writable_paths {
        let resolved = path.replace("{workspace}", cwd);
        bwrap.args(["--bind", &resolved, &resolved]);
    }
    // /proc, /dev 최소 마운트
    bwrap.args(["--proc", "/proc", "--dev", "/dev"]);
    // 네트워크 격리
    if !config.allow_network {
        bwrap.arg("--unshare-net");
    }
    bwrap.args(["--", "sh", "-c", cmd]);
    bwrap.current_dir(cwd);
    bwrap
}
```

**Step 3: ExecShellTool 통합**
- `execute()` 내부에서 `SandboxConfig.enabled && backend == Bubblewrap` 시 `wrap_command_bwrap()` 사용.
- 비활성화 또는 미지원 시 기존 `build_host_command()` 폴백.

**Step 4: 설정 UI 연동**
- `/config` 대시보드에 Sandbox 섹션 추가 (enabled/network/paths).
- `config.toml`에 `[sandbox]` 테이블 영속화.

#### 42.4 테스트 요구사항
- bwrap 설치된 CI 환경에서 `/etc/passwd` 쓰기 시도 → 커널 EPERM 검증.
- bwrap 미설치 시 graceful fallback (경고 + 기존 경로) 검증.
- 네트워크 격리 시 외부 HTTP 요청 실패 검증.

> [!WARNING]
> Phase 42는 **Linux 전용** 이점이 큼. Windows Job Object는 파일시스템 격리가 아닌 리소스 제한(CPU/메모리)만 가능하므로, Windows에서는 AppContainer SID 또는 v4.0 Docker 통합을 별도 검토해야 함.

---

### Phase 43: 에코시스템 확장 — MCP 클라이언트 (v3.3.0) ⚠️ 인프라 완료 / E2E 미비

#### 43.1 Scope Closure
- **목표**: MCP(Model Context Protocol) stdio 클라이언트를 구현하여 외부 MCP 서버의 도구를 smlcli에서 사용 가능하게 함.
- **비목표**: MCP 서버 기능(smlcli가 서버 역할), LSP 통합은 v3.4 이후.
- **성공 기준**: `config.toml`에 MCP 서버 등록 후 해당 서버의 도구가 AI 모델에 자동 노출됨. 도구 호출 시 MCP 서버로 위임.
- **달성 상태**: 인프라 구현 완료. `config.toml`의 `[[mcp_servers]]` 섹션에 서버 등록 → 앱 시작 시 비동기 초기화 → LLM tools 배열에 자동 노출 → 도구 호출 시 MCP 서버로 JSON-RPC 위임. **E2E 테스트는 Phase 44 Task M-4로 이관.**

#### 43.2 Typed Contracts (구현 동결)
```rust
/// MCP 서버 설정 (config.toml의 [[mcp_servers]])
/// 파일: src/domain/settings.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,       // 서버 식별자 (예: "filesystem")
    pub command: String,    // 실행 명령어 (예: "npx")
    #[serde(default)]
    pub args: Vec<String>,  // 인자 목록 (예: ["-y", "@anthropic/mcp-filesystem"])
}

/// MCP 서버로부터 수신한 도구 메타데이터
/// 파일: src/infra/mcp_client.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInfo {
    pub name: String,              // 원본 도구명 (예: "read_file")
    pub description: String,       // 도구 설명
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,       // JSON Schema 입력 정의
}

/// MCP 클라이언트 — mpsc 채널 기반 비동기 JSON-RPC 2.0 통신
/// 파일: src/infra/mcp_client.rs
#[derive(Debug, Clone)]
pub struct McpClient {
    pub name: String,                                // 서버 식별자
    request_tx: mpsc::Sender<RpcRequest>,            // 요청 전송 채널
    pending_requests: Arc<Mutex<HashMap<u64, tokio::sync::oneshot::Sender<anyhow::Result<serde_json::Value>>>>>, // 대기 중인 응답 맵 (타임아웃/EOF 고립 방지용)
    request_id_counter: Arc<std::sync::atomic::AtomicU64>, // 요청 ID 발급기
    child_handle: Arc<Mutex<Option<Child>>>,          // 자식 프로세스 핸들 (shutdown용)
}

/// 런타임 상태 — MCP 클라이언트 및 도구 캐시
/// 파일: src/app/state.rs (RuntimeState 내부)
pub mcp_clients: HashMap<String, McpClient>,   // 서버명 → 클라이언트 인스턴스
pub mcp_tools_cache: Vec<serde_json::Value>,   // LLM tools 배열에 직접 주입되는 스키마 목록
```

#### 43.3 구현 상세 (Execution Path)

**Step 1: MCP JSON-RPC 클라이언트 (`infra/mcp_client.rs`, 232줄)**
- MCP 서버를 `Command::new(cmd).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())` 형태로 비동기 스폰.
- **[v2.5.3]** `Arc<Mutex<Option<Child>>>` 핸들 보관 → `shutdown()` 메서드로 앱 종료 시 명시적 kill.
- **[v2.5.3]** stderr drain task: 별도 `tokio::spawn`으로 stderr을 소비하여 OS 파이프 버퍼 블로킹 방지.
- `mpsc::channel(32)` 기반 요청 큐 + `oneshot::channel` 기반 응답 매칭 구조.
- Stdin Writer Task: `RpcRequest`를 수신하여 JSON-RPC 2.0 메시지로 직렬화 후 stdin에 `\n` 구분자로 전송. `AtomicU64` 기반 자동 ID 채번.
- Stdout Reader Task: `BufReader::read_line()` 기반 라인 단위 응답 수신 → `id` 매칭으로 `pending_requests: Arc<Mutex<HashMap<u64, oneshot::Sender>>>` 역탐색하여 결과 전달.
- JSON-RPC 2.0 over stdio: `initialize` → `notifications/initialized` → `tools/list` → `tools/call` 프로토콜.
- `tokio::time::timeout(10초)` 래퍼로 응답 무한 대기 차단.
- `send_notification()`: ID 없이 전송 (MCP `notifications/initialized` 등 단방향 메시지용).

**Step 2: 동적 도구 등록 (`app/mod.rs` 초기화 경로)**
- `App::new()` 초기화 시 `settings.mcp_servers` 순회.
- 각 서버에 대해 `tokio::spawn`으로 비동기 초기화:
  1. `McpClient::spawn(name, cmd, args)` → `initialize()` → `list_tools()` 호출
  2. 수신된 `McpToolInfo` 목록을 OpenAI tools 호환 `{type: "function", function: {name, description, parameters}}` 형태로 래핑 (**[v2.5.3]** 기존 미래핑 → 완전 래핑으로 수정)
  3. 도구명을 `mcp_{server_name}_{tool_name}` 형식의 네임스페이스 접두사로 래핑
  4. `McpToolsLoaded(server_name, schemas, client)` 액션으로 이벤트 루프에 전달
- `Action::McpToolsLoaded` 핸들러에서 `RuntimeState.mcp_clients`에 클라이언트 등록, `mcp_tools_cache`에 스키마 캐싱.

**Step 3: LLM 도구 노출 (`app/chat_runtime.rs`)**
- `build_streaming_chat_request()` 시 `mcp_tools_cache`의 스키마를 기존 `GLOBAL_REGISTRY` 도구 목록에 합류하여 `ChatRequest.tools` 배열에 포함.
- LLM이 `mcp_{server}_{tool}` 형식의 도구명으로 호출 시, `tool_runtime.rs`의 디스패치 로직이 접두사를 분석하여 해당 MCP 클라이언트로 라우팅.
- **[v3.3.5]** full_name 길이 제한: `build_mcp_full_name()` 함수에서 서버/도구 파트를 각 최대 27자로 truncate. 접두사 5자 + 접미사 예비 4자 = 9자를 고정 할당하여, 전체 60자 이내 base + suffix 포함 64자 이내 보장 (OpenAI function name 64자 제한).
- **[v3.3.6]** 전역 충돌 정책: `McpToolsLoaded` 핸들러에서 `mcp_tool_name_map`에 merge 시 기존 key와 충돌하면 suffix(_2, _3 등) 부여. suffix로 인해 64자 초과 시 base를 overflow만큼 줄여서 재구성.
- **[v3.3.7]** schema-map 동기화: 전역 충돌 해소 시 schema의 `function.name`도 변경된 key와 동일하게 수정. schemas를 충돌 해소 완료 후에 cache에 push. suffix 한계(9999) 초과 시 해당 도구를 skip + 경고 로그.
- **[v3.3.8]** skip schema 제거: suffix 한계 초과로 skip된 도구의 schema를 `schemas.retain()`으로 즉시 제거. `mcp_tools_cache`에 라우팅 불가능한 도구가 남지 않도록 보장. skip 경고를 타임라인 Notice로도 표시.
- **[v3.3.9]** 핸들러 관통 테스트: `App::new()` + `handle_action(McpToolsLoaded)` 직접 호출로 `mcp_tools_cache`와 `mcp_tool_name_map`의 실제 상태 동기화를 검증. `McpClient::dummy()` 테스트 전용 생성자 도입.

**Step 4: MCP 도구 실행 디스패치 (`app/tool_runtime.rs`)**
- `tool_call.name.starts_with("mcp_")` 조건으로 MCP 도구 판별.
- **[v3.3.2]** `mcp_tool_name_map`에서 정규화된 전체 도구명을 직접 조회하여 `(sanitized_server, original_tool_name)` 튜플을 획득.
  - `mcp_clients`도 정규화 서버명을 key로 사용하므로 완전 일치 보장.
  - 이전 `starts_with` prefix match 방식은 서버명 충돌 및 정규화 불일치 문제로 v3.3.2에서 교체됨.
- 역매핑에서 복원한 원본 MCP 도구명으로 `client.call_tool(original_name, args)` 호출.
- **[v3.3.3]** `call_tool()` 응답에서 `isError`를 `content`보다 먼저 검사. `isError: true`이면 content를 에러 메시지로 활용하여 `Err` 반환.
- **[v3.3.4]** 같은 서버 내 도구명 정규화 충돌(예: `foo.bar`/`foo_bar` → 동일 full_name) 시 suffix 부여로 고유성 보장.
- **[v3.3.6]** 서버 간 truncation 충돌도 전역 merge 시 suffix로 해소. suffix 포함 64자 초과 시 base truncation 적용.
- 성공 시 `ToolResult`로 래핑하여 `ToolFinished` 액션 전송, 실패 시 `ToolError::ExecutionFailure` 전파.

**Step 5: Permission 통합 (`domain/permissions.rs`)**
- `PermissionEngine::check()`에서 `call.name.starts_with("mcp_")` 감지 시 무조건 `PermissionResult::Ask` 반환.
- 모든 MCP 도구는 사용자 명시적 승인(y/n) 후에만 실행.

**Step 6: `/mcp` 슬래시 명령어 (`app/command_router.rs`)**
- `/mcp list`: 등록된 MCP 서버의 이름, 명령어, 인자를 목록 표시.
- `/mcp add <name> <command> [args...]`: 새 MCP 서버를 `settings.mcp_servers`에 추가 후 비동기 `save_config()`. 재시작 안내.
- `/mcp remove <name>`: 서버를 `settings.mcp_servers`에서 제거 후 비동기 `save_config()`. 재시작 안내.
- `/help` 도움말 메뉴에 `/mcp` 항목 등록.

#### 43.4 테스트 요구사항
- Mock MCP 서버 스크립트(간단한 echo 서버)로 `initialize` → `tools/list` → `tools/call` 왕복 검증.
- 서버 크래시 시 graceful error + timeout 10초 내 반환 검증.
- 네임스페이스 접두사 `mcp_{server}_{tool}` 적용 및 strip 복원 검증.
- `/mcp add/remove` 후 `config.toml` 영속화 검증.
- `PermissionEngine`에서 `mcp_` 접두사 도구에 대해 `Ask` 강제 반환 검증.

---

### Phase 44: DeleteFile 도구 및 TECH-DEBT 정리 (v3.4.0)

#### 44.1 Scope Closure
- **목표**: v3.7.1에서 예약된 `DeleteFile` 도구 구현, 모든 `TECH-DEBT`/`ROADMAP` 마커 정리, `#[allow(dead_code)]` 제거.
- **성공 기준**: `DeleteFile` 도구 레지스트리 등록 + sandbox 검사 + guard 테스트 자동 통과. `grep -r "TECH-DEBT\|allow(dead_code)" src/ | wc -l` → 0.

#### 44.2 구현 가이드

**Step 1: `DeleteFileTool` 구현 (`tools/file_ops.rs`)**
- `Tool` trait 구현. `check_permission()`: `validate_sandbox()` 필수 + `FileWritePolicy` 적용.
- `execute()`: `std::fs::remove_file()` + 삭제 전 `GitEngine::auto_commit()` (Phase 40 의존).
- `is_destructive()`: `true` 반환.

**Step 2: GLOBAL_REGISTRY 등록 + guard 테스트 연동**
- `registry.rs`에 `DeleteFileTool` 등록.
- `is_write_tool()`의 `DeleteFile`이 자동으로 guard 테스트에 포함됨 (자동 대조 구조).
- `known_unregistered`에서 `DeleteFile` 제거.

**Step 3: TECH-DEBT 일괄 정리**
- `grep -rn "TECH-DEBT\|allow(dead_code)\|ROADMAP" src/` 결과를 순회.
- dead_code 제거 또는 pub(crate) 전환.
- ROADMAP 주석 중 완료된 항목 삭제, 미완료 항목은 이 문서로 이관.

#### 44.3 테스트 요구사항
- `DeleteFile` sandbox 검사: `/etc/passwd` 삭제 시도 → Deny 검증.
- `tempfile::TempDir`에 파일 생성 후 삭제 → 존재하지 않음 검증.
- guard 테스트가 `DeleteFile` 자동 포함 확인 (path_write_count >= 3).

---

### Phase 45: 빌드 & 배포 파이프라인 (v3.5.0)

#### 45.1 Scope Closure
- **목표**: GitHub Actions CI/CD 파이프라인 구축. Linux/Windows 크로스 컴파일 + GitHub Releases 자동 배포.
- **성공 기준**: `git tag v3.5.0 && git push --tags` → GitHub Actions가 Linux(x86_64-unknown-linux-musl) + Windows(x86_64-pc-windows-msvc) 바이너리를 자동 빌드하여 Release에 업로드.

#### 45.2 구현 가이드

**Step 1: CI 워크플로 (`.github/workflows/ci.yml`)**
```yaml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
      - run: cargo clippy --all-targets --all-features -- -D warnings
      - run: cargo test
```

**Step 2: Release 워크플로 (`.github/workflows/release.yml`)**
```yaml
name: Release
on:
  push:
    tags: ['v*']
jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            artifact: smlcli-linux-x86_64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: smlcli-windows-x86_64.exe
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: "${{ matrix.target }}" }
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: softprops/action-gh-release@v2
        with:
          files: target/${{ matrix.target }}/release/smlcli*
```

**Step 3: 버전 자동화**
- `Cargo.toml` 버전과 `spec.md` 버전 동기화 검증 스크립트.
- `CHANGELOG.md` 최신 섹션이 태그 버전과 일치하는지 CI에서 확인.

#### 45.3 테스트 요구사항
- CI 워크플로가 PR에서 fmt/clippy/test 게이트 통과 검증.
- Release 워크플로가 태그 push 시 바이너리 생성 검증.
- musl 정적 링크 바이너리가 glibc 없는 환경에서 실행 가능 검증.

---

### Phase 46: Workspace-scoped Session Management (v3.6.0)

#### 46.1 Scope Closure
- **목표**: 현재 작업 폴더(Workspace) 단위로 세션을 격리하고, TUI 내에서 과거 대화를 불러오거나 전환할 수 있는 기능을 제공.
- **성공 기준**: `/resume` 명령어 사용 시 현재 워크스페이스의 세션 목록 모달이 노출되며, 선택 시 타임라인이 과거 대화 내역으로 즉각 교체됨.

#### 46.2 Typed Contracts
```rust
pub struct SessionMetadata {
    pub session_id: String,
    pub workspace_root: String,
    pub title: String,           // 첫 프롬프트 요약 또는 백그라운드 LLM이 생성한 제목
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

pub enum SessionAction {
    NewSession,
    ResumeSession(String), // session_id
    ListSessions,
}
```

#### 46.3 구현 가이드

**Step 1: 폴더(Workspace) 기반 세션 격리**
- `infra/session_log.rs`의 저장 경로를 워크스페이스 해시(hash) 별로 분리하거나, 세션 메타데이터에 `workspace_root` 필드를 추가하여 쿼리 시 필터링.
- 세션 메타데이터용 인덱스 파일(`sessions_index.json` 등)을 도입하여 전체 파일을 파싱하지 않고도 목록과 타이틀을 렌더링.

**Step 2: 세션 타이틀 자동 생성 (Auto-Titling)**
- 새 세션의 첫 번째 UserMessage가 들어왔을 때, 해당 텍스트의 앞부분(약 30~50자)을 딴 임시 타이틀을 부여.
- 백그라운드 비동기 태스크(LLM)를 통해 2~3단어의 명확한 제목(예: "DB 커넥션 버그 수정", "Setup UI 스캐폴딩")으로 자동 갱신.

**Step 3: `/resume`, `/session`, `/new` 명령어 라우팅**
- `app/command_router.rs`에 명령어 추가.
- `/new`: 기존 세션을 뒤로 하고, `AppState.timeline`과 `SessionState`를 텅 빈 상태로 초기화. 새로운 `session_id` 할당.
- `/resume`: TUI 내의 오버레이(명령어 팔레트 등)에 세션 목록을 렌더링.

**Step 4: TUI Session Picker UX**
- `Ctrl+K` 또는 `/resume` 팝업을 통해 방향키로 세션을 탐색.
- 선택 시 `SessionLogger::restore_messages()`를 호출하여 `AppState`의 타임라인을 해당 세션 내용으로 핫-스왑(Hot-swap).

#### 46.4 테스트 요구사항
- 서로 다른 `workspace_root` 환경에서 `smlcli` 실행 시, 서로의 세션이 목록에 노출되지 않음 검증.
- `/new` 명령어 입력 시 타임라인과 컨텍스트 예산(tokens)이 완전히 리셋됨 검증.
- 과거 세션 로드 후 프롬프트 전송 시 해당 세션 파일에 `append` 되는지 검증.

---

### Phase 47: Interactive Planning Questionnaire (v3.7.0)

#### 47.1 Scope Closure
- **목표**: `PLAN` 모드에서 AI가 모호성을 발견할 경우, 텍스트로 질문하는 대신 구조화된 폼(Form) 도구를 호출하게 하여 사용자가 TUI 내에서 화살표 키로 객관식/주관식 답변을 빠르게 입력할 수 있도록 구현.
- **성공 기준**: LLM이 `AskClarification` 도구를 호출하면 TUI 화면에 질문 리스트가 렌더링되며, 사용자의 선택 결과를 모아 `ToolResult`로 자동 전송하여 플래닝을 완료함.

#### 47.2 Typed Contracts
```rust
pub struct ClarificationQuestion {
    pub id: String,
    pub title: String,
    pub options: Vec<String>,     // 빈 배열이면 주관식(자유 입력) 텍스트로 간주
    pub allow_custom: bool,       // true일 경우 옵션 외 "직접 입력" 허용
}

pub struct AskClarificationArgs {
    pub questions: Vec<ClarificationQuestion>,
}

pub struct AskClarificationResult {
    pub answers: HashMap<String, String>, // key: question id, value: selected/typed answer
}

// State Extension
pub struct QuestionnaireState {
    pub questions: Vec<ClarificationQuestion>,
    pub current_index: usize,
    pub answers: HashMap<String, String>,
}
```

#### 47.3 구현 가이드

**Step 1: AI 프롬프트 하네싱 (Harnessing)**
- `dispatch_chat_request`에서 현재 모드가 `PLAN`일 때, 시스템 프롬프트에 `AskClarification` 도구 사용 지침을 명시적으로 주입.
- "모호한 요구사항이 있다면 텍스트로 질문하지 말고, 반드시 `AskClarification` 도구를 사용하여 선택지를 제시하라."라는 강제 지침(System Directive) 추가.

**Step 2: `AskClarificationTool` 구조체 및 스키마 등록**
- `tools/questionnaire.rs`를 신규 생성하고 `Tool` trait을 구현.
- `execute()` 내부에서 백그라운드로 실행되지 않고, 메인 TUI의 `QuestionnaireState`를 렌더링 모드로 전환하는 이벤트를 발생시키도록 설계(이벤트 루프 기반 UI 블로킹).

**Step 3: Questionnaire TUI 렌더러 (`tui/widgets/questionnaire.rs`)**
- `Action::ShowQuestionnaire` 수신 시 타임라인 위에 오버레이 모달 또는 인스펙터 패널 형태로 질문 폼(Form)을 렌더링.
- 화살표 키(`↑`, `↓`)로 옵션을 탐색하고 `Enter`로 선택. 자유 입력란은 `ComposerState`의 버퍼를 차용.

**Step 4: 답변 조립 및 결과 전송**
- 사용자가 모든 질문에 답을 완료하면, `AskClarificationResult` 구조체로 직렬화.
- 이를 `ToolFinished(Box<ToolResult>)` 액션으로 이벤트 루프에 태워 LLM으로 피드백 전송.

#### 47.4 테스트 요구사항
- `PLAN` 모드에서 시스템 프롬프트 주입 여부 검증 (unit test).
- `AskClarificationTool` 스키마가 정상적으로 OpenAI/Anthropic/Gemini 포맷으로 변환되는지 검증.
- 더미(Dummy) 질문을 포함한 `ShowQuestionnaire` 이벤트 발송 시 TUI 렌더링 및 방향키 포커스 순환 검증.

---

### v3.0 Phase 의존성 그래프

```
Phase 40 (Git)        ✅ 완료
    │
    ├──→ Phase 41 (Provider) ✅ 완료 ──→ Phase 43 (MCP) ✅ 완료
    │                                         │
    └──→ Phase 42 (Sandbox) ✅ 완료           │
              │                               │
              └──────→ Phase 44 (Cleanup) ✅ ──→ Phase 45 (Deploy) ✅ ──→ Phase 46 (Session) ✅ ──→ Phase 47 (Plan Form) ✅ ◀── 완료
```

- **Phase 40 (Git) ✅**: Git-Native Integration 완료. `GitEngine` 자동 커밋, `/undo` 되돌리기, Inspector Git 탭.
- **Phase 41 (Provider) ✅**: 커스텀 Provider 확장 완료. `ProviderKind::Custom`, `/provider add/remove/list`.
- **Phase 42 (Sandbox) ✅**: OS-Level Sandbox 완료. `bubblewrap` 래퍼, `/config` Sandbox 섹션.
- **Phase 43 (MCP) ✅**: MCP 클라이언트 인프라 완료. `McpClient` JSON-RPC 2.0, `mcp_tools_cache`, `/mcp list/add/remove`. E2E 테스트 완료 (Task M-4, v3.7.0).
- **Phase 44 (Cleanup) ✅**: `DeleteFileTool` 구현 + `TECH-DEBT`/`allow(dead_code)` 일괄 정리 완료.
- **Phase 45 (Deploy) ✅**: GitHub Actions CI/CD 파이프라인 구축 완료. fmt/clippy/test 게이트 + Release 크로스빌드.
- **Phase 46 (Session) ✅**: 워크스페이스 연동 세션 관리 완료. `/session`, `/resume`, `/new` 명령어 + Auto-Titling.
- **Phase 47 (Plan Form) ✅**: PLAN 모드 전용 Interactive Questionnaire 폼 완료. `AskClarification` 도구 + TUI 모달 + State Machine.
