# smlcli Implementation Spec (v0.1 BETA)

## 0. Global Documentation Rules (Git Policy)

**Priority Over Code**
문서 업데이트는 소스코드 작성보다 우선되는 절대 규칙이다. 구현 전과 구현 후 모두 `spec.md`, `audit_roadmap.md`, `implementation_summary.md`, `designs.md`를 먼저 갱신한다.

**Git Allowlist**
Git에는 `README.md`, `CHANGELOG.md`, `BUILD_GUIDE.md`만 업로드한다.

**Local Update Enforcement**
`.gitignore`에 의해 Git에 올라가지 않는 `spec.md` 및 내부 문서도 로컬에서는 항상 최신 상태를 유지한다.

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

이 프로젝트는 Linux와 Windows를 함께 지원해야 하고, 방향키·Enter·ESC 중심의 TUI 조작이 핵심이다. 현재 설계 방향은 `ratatui` 기반 UI, `crossterm` 기반 터미널 이벤트 처리, `tokio` + `reqwest` 기반 비동기 provider 호출, `keyring` 기반 secure store, `grep` + `ignore` 기반 검색, `similar` 기반 diff로 구성한다.

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
`keyring` + `chacha20poly1305`

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
│   │   ├── mod.rs (Event Loop & Top-level Dispatch)
│   │   ├── state.rs
│   │   ├── event_loop.rs
│   │   ├── action.rs
│   │   ├── command_router.rs (슬래시 커맨드 엔진)
│   │   └── chat_runtime.rs (LLM 요청 조립 & Provider 디스패치)
│   ├── tui/
│   │   ├── mod.rs
│   │   ├── terminal.rs
│   │   ├── layout.rs
│   │   └── widgets/
│   │       ├── mod.rs
│   │       ├── config_dashboard.rs
│   │       └── setting_wizard.rs
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── session.rs (Context Budget & Compaction logic)
│   │   ├── provider.rs
│   │   ├── settings.rs
│   │   ├── permissions.rs (Permission Engine)
│   │   └── tool_result.rs
│   ├── providers/
│   │   ├── mod.rs
│   │   ├── registry.rs
│   │   └── types.rs
│   ├── tools/
│   │   ├── mod.rs
│   │   ├── file_ops.rs
│   │   ├── shell.rs
│   │   ├── grep.rs
│   │   ├── sys_ops.rs
│   │   └── executor.rs
│   ├── infra/
│   │   ├── mod.rs
│   │   ├── config_store.rs
│   │   ├── secret_store.rs
│   │   └── ...
│   ├── tests/
│   └── types/
│       └── mod.rs
└── assets/
    └── examples/
```

### 3.2 Runtime Architecture

애플리케이션은 단일 `AppState`를 중심으로 동작한다. 입력 이벤트, 렌더 요청, AI 응답, 툴 결과, modal 상태를 모두 `Action` 단위로 정규화한다. 이벤트 루프는 TUI 렌더링과 비동기 작업을 분리하되, 사용자에게는 하나의 연속된 터미널 경험처럼 보이게 유지한다.

핵심 계층은 아래처럼 분리한다.

* `tui/*`: 화면 그리기와 키 입력 해석
* `app/*`: 라우팅, 상태 전이, context budget
* `providers/*`: provider/model 호출 추상화
* `tools/*`: 파일/셸/grep/diff 등 실행 가능한 도구
* `infra/*`: 저장소, 암호화, OS 상호작용
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

* 민감정보는 keyring에 저장
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
}

pub struct PersistedSettings {
    pub version: u32,
    pub default_provider: String,
    pub default_model: String,
    pub cwd_policy: CwdPolicy,
    pub shell_policy: ShellPolicy,
    pub file_write_policy: FileWritePolicy,
    pub network_policy: NetworkPolicy,
    pub theme: ThemeMode,
}

pub enum ProviderKind {
    OpenAI,
    Anthropic,
    Google,
    OpenRouter,
    OpenAICompatible,
}

pub struct ProviderProfile {
    pub id: String,
    pub kind: ProviderKind,
    pub base_url: Option<String>,
    pub api_key_alias: String,
    pub model: String,
    pub enabled: bool,
}

pub enum ToolCall {
    ReadFile { path: String },
    WriteFile { path: String, content: String, require_diff: bool },
    ExecShell { command: String, cwd: String, timeout_ms: u64 },
    Grep { pattern: String, root: String, case_sensitive: bool },
    Diff { old_text: String, new_text: String },
    ListDir { path: String },
    Pwd,
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

---

## 4. Environment-Specific Configuration (Agent Rules)

**Config Filename:** `.antigravityrules`

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
2. master secret를 keyring에 저장

   * service: `smlcli`
   * username: `master-key`
3. 일반 설정은 `settings.toml` 구조체로 직렬화
4. 직렬화 결과를 XChaCha20Poly1305로 암호화
5. 파일에는 `version`, `nonce`, `ciphertext`만 저장
6. provider별 실제 API key는 keyring에 별도 저장
7. config 파일에는 `api_key_alias`만 저장

**Decryption Flow**

1. 시작 시 keyring에서 master secret 조회
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

* API key는 keyring에 저장한다.
* 로컬 설정은 암호화 파일로만 저장한다.
* master secret는 keyring에 있고, 파일에는 없다.
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
* **Tech Stack Mastery**: Rust, ratatui, crossterm, tokio, reqwest, keyring, grep, diff, secure config storage
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

`/setting` wizard, keyring 저장, encrypted config

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

---

## 10. Final Implementation Notes

* 이 프로젝트는 **채팅 UI**가 아니라 **작업형 터미널 에이전트**다.
* 핵심 완성도 기준은 답변 품질보다 **설정 신뢰성**, **권한 통제**, **파일 변경 가시성**, **종료 복구 안정성**이다.
* MVP 범위에서는 플러그인 시스템, LSP, 멀티에이전트, 원격 서버 모드는 넣지 않는다.
* v0.1 BETA의 성공 기준은 “매일 쓸 수 있는 안전한 `smlcli`”다.
