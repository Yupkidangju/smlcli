# Design Decisions (ADRs)

이 문서는 프로젝트의 주요 아키텍처 결정(Architecture Decision Records, ADRs)을 기록합니다.
코드의 형태보다는 **왜 이러한 기술, 도구, 방법론을 선택했는지**를 기록하여 향후 투입되는 팀원과 에이전트에게 맥락(Context)을 제공합니다.

---

## ADR-001: UI 프레임워크로 Ratatui 채택

### Status
Accepted

### Date
2026-04-14 (초기 선언)

### Context
`smlcli`는 순수 터미널 환경에서 시각적으로 풍부한 피드백(파일 내용, Diff, 상태바, 검색 목록)을 제공해야 합니다.
키보드 단축키, 창 크기 변환, 타임라인 스트리밍 지원이 필수적입니다.

### Decision
Rust의 터미널 컴포넌트 프레임워크인 `ratatui`와 터미널 이스케이프 관리에 `crossterm` 백엔드를 활용합니다.

### Alternatives Considered
- **Cursive**: 콜백 기반 이벤트 시스템이나, 비동기 상태 관리 및 모던 레이아웃 렌더링을 구현하기는 조금 낡은 설계임.
- **Iced (TUI 모드)**: GUI 멀티 대응이 가능하나, CLI 위주의 무거운 의존성 때문에 탈락.

### Consequences
- 안전한 드로잉을 위해 앱 상태를 불변성에 가깝게 관리하고, 매 틱(tick)마다 상태 기반 UI 재렌더링 수행 필요.
- 사용자 이벤트 관리에 복잡도가 소폭 증가하므로 `AppAction` 이벤트 채널 설계가 필수적임.

---

## ADR-002: 민감성 데이터 관리를 위한 OS Keyring과 암호화 결합

### Status
Superseded by ADR-007 (v0.1.0-beta.14)

### Date
2026-04-14 (초기 선언)

### Context
사용자의 LLM API Key는 로컬 컴퓨터에 안전하게 보관되어야 하나, 평문 설정 파일에 저장할 경우 탈취 가능성과 우발적 Git 업로드가 우려됨.

### Decision
`keyring` 크레이트를 이용해 OS 의존적 보안 저장소(Windows Credential Manager, Linux Secret Service/KWallet)에 `master-key` 및 API Provider 별 인증키를 저장. 일반 설정 파일은 `master-key`를 활용해 `XChaCha20-Poly1305`로 암호화함.

### Alternatives Considered
- 설정 파일 평문 저장 (최악의 보안)
- `.env` 파일 관리 (다른 툴에서 읽을 가능성 농후)

### Consequences
- 패키지 이식성에 OS 데몬 및 시스템 라이브러리 (pkg-config, libdbus 등) 제약 사항이 생길 수 있음.
- 빌드 가이드에 해당 필수 라이브러리를 명시해야 함.

---

## ADR-003: "Inspector" 및 "Composer" 기반 정보 구조 통합

### Status
Accepted

### Date
2026-04-14 (초기 선언)

### Context
기능이 너무 많을 때 화면의 모든 요소를 띄워놓으면 터미널 공간 제약 문제(인지 과부하)가 발생함.

### Decision
우측 작업을 전담하는 `Inspector`를 도입하고 타임라인 이벤트를 통합, 하단의 긴 명령 공간을 `Composer`로 지칭함.
이러한 Vertical Slicing은 100행 미만의 터미널 공간을 최적화. `designs.md`의 규칙 정의로 확약됨.

### Consequences
- 모든 로그와 탭은 필요시에만 토글되며, 복잡도는 UI 렌더링 측 트리에 숨김.

---

## ADR-004: 하이브리드 컨텍스트 압축 시스템 (Intelligent Compression)

### Status
Accepted

### Date
2026-04-15

### Context
긴 채팅 히스토리로 인해 컨텍스트 한도(Token Limit) 초과 에러가 빈번해질 수 있으며, 한계치에 다다랐을 때 단순히 오래된 메시지 절반을 삭제하게 되면 프로젝트의 핵심 설계 맥락(spec.md, 방향성)이 망각되어 AI의 환각(Hallucination) 위험이 증대됨.

### Decision
OpenHands, Aider, Devika 등 선도적인 Coder Agent의 방식을 하이브리드로 채택함.
1. 단순 메시지 개수가 아닌 동적 토큰 임계점(Limits)의 75% 도달 시 압축 트리거.
2. 백그라운드 LLM 프롬프팅을 통한 `Summarizing Condenser` 방식으로 메시지 뭉치를 단순 삭제 대신 작은 `[Summary]` 로 대체 보존.
3. `/tokens`를 통해 사용자가 소비를 통제하고 중요 세션은 보호(Pinning)할 수 있도록 함.

### Alternatives Considered
- **전체 요약 본문 유지**: 매 API 콜마다 과거 내역 전체를 요약시켜 Token을 줄이지만 API 호출 오버헤드와 지연 시간 극증.
- **Tree-sitter 기반 Repository Map (Aider)**: 문맥의 코드를 구조도로 줄여주지만, Parser 통합의 개발 비용이 무겁고 현재 MVP 스펙을 상회함.

### Consequences
- `AppState`에 단순 Array 형태의 Message Box 대신, 중요도 기반 속성과 요약을 관리할 수 있는 메타데이터 구조로 고도화가 요구됨.
- 요약 생성 API 요청 비용 및 비동기 처리(Tokio Runtime) 상의 복잡성 증대.

---

## ADR-005: mod.rs God Object 분해 (v0.1.0-beta.7)

### Status
Accepted

### Date
2026-04-15

### Context
`src/app/mod.rs`가 773줄의 단일 파일로, 입력 처리, 위자드 컨트롤러, 채팅 런타임, 슬래시 커맨드 엔진, 도구 승인 런타임, Config 팝업 오케스트레이션 등 6개 책임을 모두 담당하고 있었습니다. 새 기능 추가나 버그 수정 시 회귀(regression) 위험이 높고, 코드 리뷰 시 변경 영향 범위를 특정하기 어려운 상태였습니다.

### Decision
`mod.rs`를 책임 단위로 분해하여 다음 모듈로 분리:
- `command_router.rs` (215줄): 12개 슬래시 커맨드의 파싱과 실행
- `chat_runtime.rs` (90줄): LLM 요청 조립, API 키 조회, Provider 디스패치
- `tool_runtime.rs` (173줄): 도구 JSON 파싱, 권한 검사, 비동기 실행, 승인 y/n, 직접 셸 실행
- `wizard_controller.rs` (222줄): Setup Wizard 상태 전이, Config 팝업 Enter 처리
- `mod.rs` (422줄): 이벤트 루프 오케스트레이터, 입력 핸들러(키별 소형 메서드), Fuzzy Finder

각 모듈은 `impl App` 블록을 분산 구현하는 Rust 패턴을 사용하여, `App` 구조체 정의 변경 없이 메서드를 물리적으로 분리했습니다.

### Alternatives Considered
- **Trait 기반 분리**: 각 책임을 별도 trait로 추출. 그러나 `&mut self`에 대한 공유 상태 접근이 빈번하여, trait 경계가 의미 없음.
- **별도 구조체**: `CommandRouter`, `ChatRuntime` 등을 독립 구조체로 만들고 `App`이 소유. 그러나 `action_tx`, `state` 등을 모두 전달해야 하여 과도한 파라미터 체인 발생.

### Consequences
- 각 책임의 변경 범위가 물리적 파일 수준에서 격리되어 병렬 작업 가능.
- `mod.rs`는 순수 이벤트 루프 오케스트레이션으로 축소되어 가독성 개선.
- Rust의 `impl` 분산 패턴을 사용했으므로 API 호환성 완전 유지.

---

## ADR-006: Provider 자격 검증 2단계 방식 (v0.1.0-beta.7)

### Status
Accepted

### Date
2026-04-15

### Context
OpenRouter의 `/api/v1/models` 엔드포인트는 공개 엔드포인트이므로 인증 없이도 응답합니다. 기존 위자드는 이 엔드포인트로만 검증했기 때문에, 잘못된 API 키도 설정이 "성공"하고 실제 채팅(`/api/v1/chat/completions`) 시에야 401 에러가 발생했습니다.

### Decision
위자드에서 API 키 입력 후 다음 2단계를 순차 진행:
1. `validate_credentials()` 호출 — 실제 인증이 필요한 엔드포인트로 키 유효성 확인
2. 성공 시에만 `fetch_models()` 호출 — 모델 목록 조회 진행

`CredentialValidated` 비동기 이벤트를 `Action` enum에 추가하여, 검증 결과에 따라 위자드 단계를 진행하거나 에러 메시지를 표시합니다.

### Consequences
- 잘못된 키로 설정이 저장되는 것을 원천 차단.
- 위자드 UX에 "검증 중..." → "성공" 또는 "실패: 재입력" 피드백 제공.
- 비동기 이벤트가 하나 추가되어 이벤트 핸들러 복잡도 소폭 증가.

---

## ADR-007: Credential Store 재설계 — keyring 제거, 파일 기반 암호화 (v0.1.0-beta.14)

### Status
Accepted (ADR-002 Superseded)

### Date
2026-04-16

### Context
`keyring` 크레이트는 Linux에서 gnome-keyring(D-Bus Secret Service)에 의존하여:
1. `sync-secret-service` feature 미지정 시 mock store만 사용되어 키 영속화 실패 (beta.13 긴급 버그).
2. D-Bus 미설치 환경(WSL, headless 서버, Docker)에서 빌드/실행 자체가 불가.
3. Windows에서는 Credential Manager 백엔드가 별도이므로 크로스플랫폼 테스트 부담 증가.

### Decision
keyring 크레이트를 완전 제거하고 파일 기반 솔루션 도입:
- **마스터 키**: `~/.smlcli/.master_key` (32바이트 랜덤, hex 인코딩, chmod 600).
- **설정 파일**: `~/.smlcli/config.toml` (TOML 평문, chmod 600).
- **API 키**: `PersistedSettings.encrypted_keys: HashMap<String, String>`에 ChaCha20Poly1305 암호화된 값으로 저장.
- `save_config()` / `load_config()` 시그니처에서 `master_key` 파라미터 제거 (내부에서 자동 조회).

### Alternatives Considered
- **keyring feature 수정만**: Linux gnome-keyring 의존을 고치더라도 Windows/headless 환경 문제는 잔존.
- **dot-env 환경변수**: 평문 키 저장으로 보안 저하.
- **YAML 설정(serde_yml)**: 도입했으나 RUSTSEC-2025-0067/0068 unsound 경고로 즉시 교체 → 기존 `toml` 크레이트로 전환.

### Consequences
- 외부 OS 데몬(D-Bus) 의존 완전 해소. Docker, WSL, headless 환경에서도 동일하게 동작.
- 마스터 키 파일의 물리적 보안이 OS 키링보다 다소 약할 수 있으나, chmod 600 + 사용자 홈 디렉토리 격리로 실용적 수준 확보.
- 의존성 4개 제거 (keyring, dbus, dbus-secret-service, libdbus-sys) → 빌드 시간 및 바이너리 크기 감소.

---

## ADR-008: TUI UX 4건 개선 — 도구 JSON 필터링, 추론 인디케이터, 슬래시 메뉴, 페르소나 (v0.1.0-beta.16)

### Status
Accepted

### Date
2026-04-16

### Context
실제 TUI 사용 테스트에서 4건의 UX 결함 발견:
1. AI가 도구 호출 시 원시 JSON 스키마가 타임라인에 그대로 노출.
2. AI 추론 중 아무런 시각적 피드백 없음.
3. `/` 슬래시 커맨드를 직접 타이핑해야 하며 자동완성 없음.
4. AI에게 CLI 에이전트로서의 역할 정의(페르소나)가 부재.

### Decision
1. **filter_tool_json()**: 타임라인 렌더링 시 ```json 도구 호출 블록을 `⚙️ [도구명] 도구 호출 실행 중...` 형태로 대체.
2. **is_thinking 플래그**: dispatch 시 true, 응답 수신 시 false, 타임라인 하단에 `✨ AI가 응답을 생성하고 있습니다...` 표시.
3. **SlashMenuState**: Composer에서 `/` 입력 시 11개 명령어 팝업, 방향키+Enter 선택, Esc 닫기, 부분 일치 필터링.
4. **시스템 프롬프트 강화**: ~1K 토큰 페르소나 정의. 사용자 입력 언어 미러링 지시.

### Consequences
- 사용자 경험 대폭 개선: 도구 호출이 자연어로 설명되고, 추론 상태가 가시화됨.
- 슬래시 메뉴로 명령어 진입 장벽 감소.
- 시스템 프롬프트 토큰 소비가 ~300 → ~1K로 증가하나, 응답 품질 개선으로 총 토큰 효율은 향상.

---

## ADR-009: UX 아키텍처 전면 개편 (v0.1.0-beta.18 계획)

**상태**: 승인됨 (구현 예정)
**일자**: 2026-04-16

### Context
beta.17까지의 구조는 `Action` 7종 + `session.messages` 단일 배열이었다. 이 구조로는:
- 도구/채팅의 시작·진행·완료를 구분할 수 없어 Codex 스타일 진행 표시 불가
- LLM 컨텍스트와 UI 표시가 혼재되어 작업 카드/승인 카드/로그 분리 불가
- Inspector가 enum만 정의되고 탭 콘텐츠 미구현
- 색상이 하드코딩되어 UI 일관성 부재
- 전체 응답 일괄 수신(batch)으로 긴 응답 시 무반응

### Decision
4단계 개편 (Phase 9-A/B/C/D):

**Phase 9-A (기반)**:
1. `Action` enum 7종 → 14종 확장 (ChatStarted, ChatDelta, ToolQueued, ToolStarted, ToolOutputChunk, ToolSummaryReady 추가)
2. `timeline: Vec<TimelineEntry>` 도입 — session.messages(LLM)와 분리
3. Semantic Palette (`tui/palette.rs`) — info/success/warning/danger/muted + bg 3계층
4. tick 기반 애니메이션 (스피너, 배지 깜빡임, pulse)
5. Inspector 탭별 실체 구현 (`widgets/inspector_tabs.rs`)
6. ToolFinished 출력 요약 분리 (2~4줄 타임라인 + 원문 Logs 탭)
7. SSE 스트리밍 (`chat_stream()` + `ChatDelta`)

**Phase 9-B (기능)**: CLI Entry Modes, 세션 영속성, SafeOnly, Blocked Command, Structured Tool Call, File Read 안전장치, Grep UX
**Phase 9-C (품질)**: Shell 스트리밍, Diff UI, 테스트 22건+

### Alternatives Considered
- 기존 구조를 유지하고 UI만 리터치 → 구조적 한계로 거부
- 별도 TUI 프레임워크 사용 → ratatui 생태계 숙련도와 기존 투자 고려하여 거부

### Consequences
- 코드베이스 ~1,200줄 추가 (Phase 9+10 전체)
- 신규 의존성: `clap 4` (derive feature, CLI 서브커맨드)
- 테스트: 14건 → 24건, Clippy: 0 warnings
- 신규 파일: `tui/palette.rs`, `infra/session_log.rs`

---

## ADR-010: Phase 10 SSE 스트리밍 + 세션 영속성 + CLI Entry

### Status
Accepted

### Date
2026-04-16

### Context
Phase 9까지의 이벤트 아키텍처(ChatDelta, TimelineEntry)가 완성되었으므로, LLM 응답의 실시간 스트리밍, 대화 영속성, CLI 서브커맨드가 자연스럽게 구현 가능한 상태.

### Decision
1. **SSE 스트리밍**: ProviderAdapter trait에 `chat_stream()` 메서드 추가. stream:true로 요청 후 SSE `data:` 라인을 파싱하여 delta 토큰을 mpsc 채널로 전송. delta_forwarder 태스크가 `ChatDelta` 이벤트를 UI로 라우팅.
2. **세션 영속성**: `infra/session_log.rs` — JSONL append-only 기록. 외부 의존성 없이 `std::time::UNIX_EPOCH` 기반 타임스탬프.
3. **CLI Entry Modes**: `clap 4` derive로 `run`(기본)/`doctor`/`sessions` 서브커맨드 구현.
4. **코드 위생**: 전역 `#![allow(unused_imports/unused_variables)]` 제거, `dead_code`만 유지.

### Alternatives Considered
- **bytes_stream 진정한 청크 스트리밍**: reqwest의 `bytes_stream()` + 라인 버퍼링 방식. 현 단계에서는 `text().await` 후 라인 파싱으로도 SSE 호환 동작하므로 단순한 방식 채택. 향후 대용량 응답 시 bytes_stream 전환 가능.
- **chrono 의존성**: 타임스탬프에 chrono 사용 검토 → UNIX epoch 초 단위로 충분하므로 외부 의존성 최소화.

### Consequences
- 사용자가 실시간으로 AI 응답을 확인 가능 (기존 batch 대비 UX 크게 향상)
- 대화 기록이 `~/.smlcli/sessions/`에 자동 보존 → 세션 복원 기반 마련
- `smlcli doctor`로 설정 문제 사전 진단 가능

---

## ADR-011: 감사 대응 — 세션 로거 이중 API + 테마 시스템 + thiserror 연동 (v0.1.0-beta.20)

### Status
Accepted

### Date
2026-04-17

### Context
v0.1.0-beta.19에서 `SessionLogger::append_message()`를 비동기(`async fn`)로 전환했으나, 호출부(`chat_runtime.rs`, `mod.rs`)에서 반환된 `Future`를 `.await`나 `tokio::spawn` 없이 버려서 로그가 실제로 디스크에 기록되지 않는 치명적 결함이 발생했습니다. 또한 `from_file`/`restore_messages`도 삭제되어 회귀 테스트 전체가 실패했습니다.

동시에 감사 리포트에서 Inspector Search 탭 미구현, 테마 시스템 부재, thiserror 미사용 등 3건의 MEDIUM 이슈가 지적되었습니다.

### Decision

**1. 세션 로거 이중 API 전략**
- 비동기 API(`append_message_async`)는 향후 대용량 로깅이나 네트워크 기반 로그 전송 시나리오를 위해 유지.
- 동기 API(`append_message`, `from_file`, `restore_messages`)를 복원하여 현재의 TUI 이벤트 루프 내에서 안전하게 호출.
- 런타임 호출 경로(`chat_runtime.rs`, `mod.rs`)에서는 동기 API를 사용하여 Future 누락 문제를 원천 차단.

**2. 테마 전환 시스템**
- `PersistedSettings.theme` 필드를 `serde(default)` 어노테이션과 함께 추가하여 하위 호환성 유지.
- `palette.rs`에 `Palette` 구조체와 `DEFAULT_PALETTE`/`HIGH_CONTRAST_PALETTE` 정적 상수를 정의하고, `get_palette(&str)` 함수로 참조 반환.
- `/theme` 슬래시 커맨드를 통해 토글 전환하며, `tokio::spawn`으로 비동기 config 저장.

**3. thiserror 점진적 연동**
- `config_store::load_config()`에서 `ConfigError::NotFound`/`ParseFailure`를 실제 코드 경로에 연결.
- 반환 타입은 `anyhow::Result`를 유지하되, 내부에서 `map_err`를 통해 구조화된 에러를 생성하여 향후 UI 분기 처리 기반 마련.

### Alternatives Considered
- **모든 호출을 async + .await로 전환**: 이벤트 루프 내의 `handle_action`이 동기 메서드이므로 전면 async 전환은 아키텍처 대규모 변경을 요구. 현 단계에서는 비용 대비 효과가 낮음.
- **tokio::spawn으로 async 감싸기**: 가능하나, 에러 핸들링이 caller에게 즉시 전달되지 않아 로그 실패를 UI에 반영하기 어려움.
- **전체 anyhow 제거**: 단번에 모든 에러를 thiserror로 전환하면 변경 범위가 과도. 점진적 마이그레이션이 안전.

### Consequences
- 세션 로그가 실제로 디스크에 기록되어 세션 복원 기능의 기반이 확보됨.
- 테마 전환이 실시간으로 동작하며 재시작 시에도 유지됨.
- 28개 회귀 테스트 전부 통과, clippy 경고 0건으로 릴리스 게이트 충족.
- 에러 체계가 구조화되어 향후 UI에서 에러 유형별 메시지 분기 가능.

---

## ADR-012: 테마 렌더링 주입 아키텍처 및 에러 타입 전면 구조화

### Status
수락 (v0.1.0-beta.21)

### Context
v0.1.0-beta.20에서 테마 시스템의 데이터 모델(`Palette` 구조체, `get_palette()`)과 커맨드(`/theme`)는 구현되었으나, 실제 TUI 렌더링 코드는 여전히 `pal::ACCENT`, `pal::SUCCESS` 등 정적 상수를 직접 참조하여 테마 전환이 화면에 반영되지 않았다. 또한 `Action` enum의 에러 경로가 `String` 기반이라 에러 종류별 분기 처리가 불가능했다.

### Decision

**1. 테마 렌더링 주입 — `AppState::palette()` 헬퍼 패턴**
- `AppState`에 `palette() -> &'static Palette` 메서드를 추가하여 현재 설정의 `theme` 값에 따른 `Palette` 참조를 반환.
- 모든 렌더링 함수(`draw_top_bar`, `draw_timeline`, `draw_inspector`, `draw_composer`, `render_logs`, `render_search`, `render_recent`, `draw_config`, `draw_wizard`)의 진입점에서 `let p = state.palette();`를 선언.
- 기존 `pal::CONSTANT` 참조 50+곳을 `p.field`로 일괄 전환.
- `SPINNER_FRAMES`, `TOOL_BADGE` 같은 유틸리티 상수는 테마 무관이므로 `pal::` 직접 참조 유지.

**2. 에러 타입 전면 구조화**
- `Action` enum의 4개 에러 경로를 도메인 타입으로 전환:
  - `ChatResponseErr(String)` → `ChatResponseErr(ProviderError)`
  - `ToolError(String)` → `ToolError(ToolError)`
  - `ModelsFetched(Err(String))` → `ModelsFetched(Err(ProviderError))`
  - `CredentialValidated(Err(String))` → `CredentialValidated(Err(ProviderError))`
- `ProviderError`, `ToolError`, `ConfigError`, `AppError`에 `Clone` derive 추가 (Action의 Clone 요구).
- `AppError::Io`/`Unknown` variant를 `#[from] std::io::Error`/`anyhow::Error`에서 `String` 기반으로 단순화 (Clone 호환).
- 수신 핸들러에서 UI 표시 시 `e.to_string()` (Display trait) 사용으로 기존 동작 유지.

### Alternatives Considered
- **trait object 기반 동적 디스패치** (`Box<dyn Error>`): Clone 불가, Action enum과 호환 불가.
- **각 함수에 palette 파라미터 전달**: 시그니처 변경이 과도하고, state에서 이미 접근 가능하므로 불필요.
- **전역 `static` 테마 변수**: 멀티 인스턴스 확장성 저해, 런타임 변경 시 동기화 문제.

### Consequences
- `/theme` 명령어가 화면 전체에 즉시 반영되어 "실시간 테마 전환" 감사 항목 완료.
- 에러 발생 시 `match` 패턴으로 에러 종류별 UI 분기가 가능해짐 (예: AuthenticationFailed → 재인증 안내).
- 향후 새 테마 추가 시 `Palette` 상수 1개 + `get_palette()` 분기 1줄만 추가하면 됨.
- 28개 회귀 테스트 전부 통과, clippy 경고 0건 유지.

---

## ADR-013: 하네스 도구 격리, 빈 명령 차단, UI Wrap, PLAN/RUN 계약

### Status
수락 (v0.1.0-beta.22)

### Context
v0.1.0-beta.21까지의 구현에서 다음 구조적 결함이 발견됨:
1. LLM 응답의 bare JSON이 도구 호출로 자동 실행되거나 raw 텍스트로 노출됨
2. 빈 ExecShell 명령이 SafeOnly 정책에서 자동 허용됨 (`is_safe_command` 빈 토큰 → true)
3. 타임라인, 컴포저, 설정 팝업, 위자드에 word wrap이 없어 긴 텍스트가 가로로 넘침
4. PLAN/RUN 모드가 UI 토글만 있고 LLM 행동에 반영되지 않음
5. 승인 카드가 도구 정보를 30자로 절단

### Decision

**1. 3단계 도구 호출 필터링**
- bare JSON(fenced가 아닌) 응답은 도구로 인식하지 않고 로그만 남김
- fenced JSON 블록 내에 `"tool"` 키가 없으면 건너뜀 (코드 예시 JSON 보호)
- `ToolCall` 역직렬화 성공 후에도 `ExecShell.command.trim().is_empty()`면 즉시 거부

**2. 빈 ExecShell 하드 가드**
- `PermissionEngine::check()`에서 permission 분기 이전에 빈 명령 즉시 Deny
- `is_safe_command()` 빈 토큰 목록 → `false` 반환 (이전: `true`)

**3. 전체 UI Wrap + 스크롤**
- 타임라인, 컴포저, 설정 팝업, 위자드 4곳에 `Wrap { trim: false }` 적용
- `UiState::timeline_scroll: u16` 필드로 세로 스크롤 오프셋 관리

**4. PLAN/RUN 시스템 프롬프트 주입**
- `dispatch_chat_request()`에서 현재 모드를 감지하여 모드별 시스템 메시지를 주입
- PLAN: 분석/설명 위주, 파일 쓰기 자제
- RUN: `WriteFile`/`ReplaceFileContent` 우선 사용 지시

**5. 승인 카드 전체 경로**
- `format_tool_name()`: 도구별 의미 있는 이름 (전체 경로, 최대 120자)
- `format_tool_detail()`: 승인 카드에 명령/경로/동작을 축약 없이 표시

**6. 첫 턴 자연어 가드 (재감사 대응)**
- 시스템 프롬프트에서 도구 필드 스키마와 예시 JSON을 제거
- "첫 응답은 반드시 자연어", "비작업성 입력에는 도구 미사용" 정책을 Core Rules로 명시
- 도구 카탈로그는 이름만 나열 (ExecShell, ReadFile, ... 등)

**7. bare JSON 렌더링 필터 (재감사 대응)**
- `filter_tool_json()`에 bare JSON 감지 로직 추가
- `"tool"` 키가 있는 bare JSON은 `⚙️ [ToolName] 도구 호출 감지됨` 요약으로 대체
- `"tool"` 키가 없는 일반 JSON은 원문 그대로 유지

**8. 타임라인 스크롤 키 바인딩 (재감사 대응)**
- `handle_input()`에 `KeyCode::PageUp`/`KeyCode::PageDown` 분기 추가
- `timeline_scroll`을 ±5씩 조작 (saturating_add/sub)
- 위자드, Fuzzy, 설정 팝업이 열려 있을 때는 비활성

### Alternatives Considered
- **별도 machine channel 프로토콜**: native tool call API(OpenAI function calling)로 전환하면 근본적으로 해결되나, 현재 OpenRouter/Gemini 어댑터가 markdown 기반이므로 단계적 전환 결정
- **PLAN에서 도구 완전 차단**: UX 유연성 저하 → 시스템 프롬프트 수준 제어로 타협
- **무한 스크롤(가상화)**: ratatui의 `Paragraph::scroll()`이 충분하므로 현 단계에서는 불필요
- **도구 스키마를 별도 프롬프트로 분리**: 모델이 도구를 호출할 때만 지연 주입하는 방식도 고려했으나, 현재 단일 시스템 프롬프트 구조에서는 불필요한 복잡도

### Consequences
- 첫 턴에 도구 JSON이 자동 실행되거나 스키마가 노출되는 결함 해소
- bare JSON 도구 응답이 사용자 친화적 요약으로 대체됨
- 빈 명령이 어떤 정책에서도 실행되지 않음
- 모든 텍스트 영역에서 가로 넘침 없이 읽기 가능
- PageUp/PageDown으로 긴 응답을 세로 탐색 가능
- RUN 모드에서 코드 작성 요청 시 파일 도구 우선 사용으로 일관된 UX
- 승인 카드에서 전체 파일 경로와 동작을 한눈에 확인 가능
- 33개 회귀 테스트 전부 통과, clippy 경고 0건

---

## ADR-015: `@` 멘션 및 `!` 뱅 커맨드 상태 관리와 탐색 분리

### Status
Accepted

### Date
2026-04-17

### Background & Context
초기 `smlcli`의 `@` 기능은 `std::fs::read_dir(".")`에 의존하여 1 Depth 파일만 노출했으며, `!` 기능은 자동완성이나 히스토리 없이 단순 텍스트 매칭에 불과했다. 터미널 기반 에이전트로서 LLM 컨텍스트 주입 속도와 셸 실행 속도를 높여야 했으나, 복잡한 UI 요소를 추가하면 터미널의 가시성을 해칠 위험이 있었다.

### Decision (Frozen Decisions)
새로운 패널을 추가하는 대신, 기존 **Fuzzy Finder 위젯을 오버로딩(Overloading)하여 재사용**하기로 결정했다.

1. **상태 분리 (`FuzzyMode`)**: `state.rs`에 `FuzzyMode { Files, Macros }` 열거형을 동결하여 도입한다. TUI 렌더링 측(`layout.rs`)은 모드를 알 필요 없이 `matches` 배열만 렌더링하고, 입력 제어기(`app/mod.rs`)만 모드에 따라 매칭 알고리즘을 분기한다.
2. **탐색 라이브러리 교체**: 표준 라이브러리 대신 `ignore` 크레이트를 동결 사용한다. 하위 디렉터리 탐색 시 필연적으로 발생하는 `.git`, `node_modules`, `target` 등의 스캔 부하를 `.gitignore` 룰셋으로 회피하기 위함이다.
3. **히스토리 버퍼 동결**: 셸 히스토리 버퍼는 SQLite나 디스크 영속화를 하지 않고, **메모리(Vec<String>)**에만 보존한다. 에이전트 CLI 특성상, 세션이 끝나면 날아가는 것이 보안 및 관리 비용 측면에서 이득이라 판단했다.

### Alternatives Considered & Rejected
- **슬래시 커맨드(`/`) 스페이스에 통합 시도 (기각)**
  - 이유: `/`는 `smlcli` 내부 동작 설정(`Provider`, `Model`, `Mode` 토글 등)으로 예약되어 있다. 여기에 셸 명령어 자동완성이나 파일명을 섞을 경우 검색 스페이스가 오염되어 오작동을 유발한다. 따라서 관습적인 `@`(파일), `!`(셸)을 강제 분리했다.
- **히스토리 영속화 (기각)**
  - 이유: `~/.bash_history`나 별도 DB 파일에 히스토리를 저장하는 방안도 검토했으나, AI 에이전트에게 내리는 "테스트 셸"은 일회성인 경우가 많아 파일 I/O 비용 대비 효용성이 떨어진다.

### Consequences
- `update_fuzzy_matches()` 로직이 분기되면서 O(N) 탐색 부하가 발생하지만, `truncate(100)`으로 최대 항목 수를 제한하여 UI 버벅임을 차단함.

---

## ADR-016: Native Tool Call JSON Schema Migration

### Status
Accepted

### Date
2026-04-17

### Background & Context
이전 마일스톤까지 `smlcli`는 시스템 프롬프트 안에 도구 사용법(JSON 규격)을 텍스트로 밀어 넣고, LLM 응답을 정규식(` ```json ... ``` `)으로 캡처하는 방식을 사용했다. 이 방식은 모델이 인삿말("네, 파일을 생성하겠습니다.")을 앞에 붙일 경우 파싱이 복잡해지며, 억지로 프롬프트 지시("오직 JSON만 응답해")를 내려도 통제력을 벗어나는(Hallucination) 일이 빈번했다. 또한, 도구 호출 오류 시 다시 자연어로 에러를 먹여야 하는 등 루프 복잡도가 높았다.

### Decision (Frozen Decisions)
모든 LLM Provider의 상호작용 레이어를 **OpenAI 호환 Native Tool Calling API (Structured Outputs)** 로 완전히 교체한다.

1. **스키마 주입 동결**: 도구의 정의는 더 이상 시스템 프롬프트(텍스트)에 포함되지 않으며, `ChatRequest.tools` 배열에 JSON Schema 객체로 동결되어 전송된다. 
2. **응답 Role 동결**: LLM이 도구를 선택하면 자연어가 아닌 `ChatResponse.message.tool_calls` 배열로 수신된다. 이후 실행 결과는 반드시 `Role::Tool` 메시지로 포장하여 다음 턴에 전달한다.
3. **스트리밍 조립 버퍼 채택**: SSE(Server-Sent Events) 환경에서는 `tool_calls`도 청크(Chunk) 단위로 쪼개져 온다. 델타를 파싱 중인 `action_tx`와 별개로, 내부 상태에 `ToolCallDelta`를 모아두는 10MB 크기 제한의 메모리 버퍼를 두어 스트림 종료(`[DONE]`) 시점에 한 번에 역직렬화(Deserialize)한다.

### Alternatives Considered & Rejected
- **LangChain / LlamaIndex 등 외부 프레임워크 도입 (기각)**
  - 이유: `smlcli`는 의존성을 최소화한 경량 TUI 애플리케이션이다. 무거운 파이썬/JS 기반의 추상화 레이어를 러스트 포팅 버전으로 가져오면 실행 바이너리 크기 및 빌드 타임이 폭증한다. `reqwest` 기반 직접 파싱을 유지한다.
- **Provider별 독자 규격 지원 (기각)**
  - 이유: Gemini의 독자적인 `function_declarations` 규격이나 Anthropic의 규격을 따로 맞추면 `ChatRequest` 모델이 극도로 복잡해진다. 다행히 Google(Gemini)과 OpenRouter 모두 OpenAI 호환 엔드포인트를 제공하므로, OpenAI 규격(Tools API) 단일 통일안을 채택한다.

### Consequences
- 프롬프트 파싱 버그(정규식 누수) 원천 제거.
- 시스템 프롬프트의 토큰 크기가 대폭 축소되어 1턴 당 API 비용 및 지연율(Latency) 감소 기대.
- SSE 델타 파싱 로직(`chat_stream`)의 난이도가 대폭 상승(청크 조립 필요).
- 에러 발생 시 `Role::Tool` 로 즉각 피드백이 가므로 자가 치유(Auto-healing) 확률 비약적 상승.
---

## [ADR-013] Agentic Autonomy via Polymorphic Tool Registry & Git Checkpoints

- **Date:** 2026-04-18
- **Context:**
  단순한 질의응답 기반의 CLI 도구에서 벗어나, 파일 시스템을 자율적으로 수정하고 오류를 검증하며 스스로 복구하는 에이전트 시스템이 필요했다. 기존의 `match` 기반 하드코딩 도구 실행 구조는 새로운 도구 추가 시 `executor.rs`, `tool_runtime.rs` 등 여러 파일을 수정해야 하는 확장의 병목이 있었다. 또한 AI가 실수로 코드를 망가뜨렸을 때 안전망 없이 코드가 덮어씌워지는 위험이 컸다.
- **Decision:**
  1. **Polymorphic Tool Registry**: `Tool` 트레이트를 도입하고, 구조체(struct) 기반 동적 디스패치(`GLOBAL_REGISTRY`) 구조로 전환하여 개방-폐쇄 원칙(OCP)을 준수하도록 설계했다. 도구의 권한 검증(`is_destructive`), 실행, 파싱 로직을 각 도구 구현체 내부로 캡슐화했다.
  2. **Automated Git Checkpoint**: 파괴적인 도구(`WriteFile`, `ReplaceFileContent`, `ExecShell`) 실행 전후로 Git Checkpoint(임시 커밋 및 `reset --hard`)를 자동 생성하여 에러 발생 시 즉각적으로 시스템 코드를 이전 정상 상태로 복구하는 자기 보호망을 구축했다.
  3. **Auto-Verify State Machine**: `ToolFinished` 발생 시 에러가 감지되면, `AutoVerifyState::Healing` 상태로 전환하고 실패 원인을 LLM 프롬프트에 자동으로 피드백하여 AI가 스스로 문제를 인식하고 자가 복구 도구를 재호출하도록 구현했다.
  4. **Tree-sitter Repo Map**: `tree-sitter`를 사용해 소스 파일(`.rs`)의 함수/구조체 AST를 추출하고 이를 시스템 프롬프트 상단에 배치(Context Injection)하여, AI가 전체 코드베이스의 관계를 파악하고 수정하게 만들었다.
- **Consequences:**
  - **Positive:**
    - 새로운 도구를 추가할 때 레지스트리에 등록만 하면 될 정도로 확장성이 비약적으로 향상되었다.
    - Git Checkpoint를 통한 안전망 덕분에 AI가 잘못된 코드를 주입해도 즉각 롤백되고 수정 피드백이 주어지므로 작업 안정성이 극대화되었다.
  - **Negative:**
    - Tree-sitter AST 파싱 로직이 매 프롬프트 전송 시마다 실행되므로 파일 개수가 많아질 경우 성능 병목 우려가 있으나, 현재 최대 8KB 길이 제한을 두어 안전판을 마련했다.

---

## ADR-017: 2026 CLI UX 현대화 방향 채택 (Phase 15 로드맵)

### Status
Accepted (구현 예정)

### Date
2026-04-18

### Background & Context
Phase 13~14를 통해 `smlcli`는 도구 호출, 자가 복구, 멀티라인 렌더링, 스크롤 분리, 반응형 레이아웃의 기반을 확보했다. 그러나 현재 UI는 여전히 "메시지 나열형 TUI" 성격이 강하고, 최신 작업형 CLI가 제공하는 다음 패턴은 아직 구조적으로 부족하다.

- 입력/출력/도구 결과를 한 단위로 묶는 **블록 기반 히스토리**
- 긴 도움말보다 빠른 액션 발견을 돕는 **명령 팔레트**
- 입력창 주변에 상태를 드러내는 **입력 툴벨트 / 컨텍스트 칩**
- 패널별 포커스와 독립 스크롤을 갖는 **작업형 레이아웃**
- 정보 밀도를 높이되 시각적 소음을 늘리지 않는 **절제된 ASCII 모션**

외부 조사 기준으로는 Warp의 Blocks/Universal Input, Textual의 Command Palette, Ratatui의 Layout/Style 계층이 가장 현실적인 참조점이었다.

### Decision (Frozen Decisions)
1. **프레임워크 유지**
   - `ratatui + crossterm`을 유지한다.
   - 최소 1차 구현에서는 Textual/Bubble Tea로 교체하지 않는다.

2. **Block-first Timeline**
   - 대화 기록은 `TimelineBlock` 중심으로 재구성한다.
   - 입력/AI/도구 결과를 하나의 작업 블록으로 묶는다.

3. **Command Palette 우선**
   - `Ctrl+K`를 전역 Quick Actions palette로 동결한다.
   - `Ctrl+P`는 provider/model 빠른 전환 역할을 유지한다.

4. **Composer Toolbar 도입**
   - mode, cwd, context, policy, hint를 chip 형태로 표시한다.
   - 입력창은 단순 텍스트 버퍼가 아니라 작업 툴벨트로 취급한다.

5. **Focused Pane 상태 머신**
   - `Timeline`, `Inspector`, `Composer`, `Palette`를 독립 포커스 패널로 취급한다.
   - 키보드/마우스 라우팅은 포커스 또는 포인터 위치에 종속된다.

6. **모션 예산 제한**
   - 상태 전달용 ASCII 모션만 허용한다.
   - 과한 점멸과 전체 화면 전환 애니메이션은 금지한다.

### Alternatives Considered & Rejected
- **현재 구조 유지 + 표면적 스타일 수정 (기각)**
  - 이유: 색상과 스피너만 바꾸면 최신 CLI처럼 "보일" 수는 있어도, 블록 히스토리/커맨드 발견/포커스 분리 같은 구조 문제는 해결되지 않는다.
- **Textual 또는 Bubble Tea 기반 재작성 (기각)**
  - 이유: 구현 속도와 일관성보다 비용이 크다. 현재 코드 자산과 테스트 자산을 버리지 않고 진화시키는 편이 실용적이다.
- **웹 UI 스타일을 강하게 흉내내는 방향 (기각)**
  - 이유: 터미널 폭 제한과 글자 격자 특성을 무시한 디자인은 금방 깨진다. CLI의 장점은 정보 밀도와 조작 속도다.

### Consequences
- **Positive**
  - UX 개선이 개별 버그 수정이 아니라 구조 개선으로 이어진다.
  - 코더는 `block/focus/palette/toolbar/layout` 축으로 작업을 분리할 수 있다.
  - 향후 멀티세션/북마크/재실행 같은 기능을 얹기 쉬워진다.
- **Negative**
  - `TimelineEntry` 중심 구현을 `TimelineBlock` 중심으로 재해석해야 하므로 초기 리팩토링 비용이 크다.
  - 포커스/스크롤 상태가 늘어나면서 상태 머신 테스트가 필수화된다.

---

## ADR-018: Auto-Verify 상태 머신 실연결과 LLM 우선 도구 판정

### Status
Accepted

### Date
2026-04-20

### Context
Phase 13 문서는 `AutoVerifyState::Healing`, 후속 재전송의 tool schema 유지, Tree of Thoughts depth 렌더링을 정의했지만 실제 구현은 이 세 축이 모두 느슨하게 연결되어 있었다.

- `AutoVerifyState`는 `state.rs`에만 존재하고 `ToolFinished`/`ToolError` 경로에서 전혀 사용되지 않았다.
- `send_chat_message_internal()` 재전송 경로는 초기 요청과 달리 tool schema를 포함하지 않아, 실패 이후 후속 도구 호출 능력이 약화됐다.
- `is_actionable_input()` 휴리스틱이 모델의 구조화된 `tool_calls`를 런타임에서 차단하여 문서의 자율성 원칙과 충돌했다.
- 타임라인 블록에는 `depth`가 없어 문서의 `└─` 계층 UI가 렌더링되지 않았다.

### Decision
1. 도구 실패는 모두 `advance_auto_verify_after_failure()`로 수렴시켜 동일한 상태 머신을 사용한다.
   - 1차/2차 실패: `Healing { retries: 1|2 }`로 전이 후 재전송
   - 3차 실패: `Abort` Notice를 남기고 `Idle`로 복귀, 추가 재전송 금지
2. 초기 요청과 후속 재전송은 모두 `build_streaming_chat_request()`를 통해 동일한 tool schema 집합을 포함한다.
3. 입력 의도 분류는 참고 신호만 제공하고, 구조화된 `tool_calls`는 모델 판단을 우선하여 실행 파이프라인으로 전달한다.
4. `TimelineBlock.depth`를 도입하고 `ToolRun`/`Approval`/`Auto-Verify Notice`에 `depth: 1`을 부여하여 TUI에서 계층 인덴트를 렌더링한다.

### Alternatives Considered
- **문서를 구현에 맞춰 하향 조정**
  - 감사 리포트가 지적한 실제 결함을 남겨둔 채 설명만 바꾸는 방식이므로 기각.
- **휴리스틱 차단 유지 + 옵션 플래그 추가**
  - 기본 경로가 계속 자율성 원칙을 어기므로 즉시 효과가 없다고 판단해 기각.

### Consequences
- 자가 복구 성공/중단 조건이 명시적 상태 전이로 정리되어 회귀 테스트가 쉬워진다.
- 안전성 책임이 `is_actionable_input()`가 아니라 Permission Engine과 blocked command 정책으로 더 명확하게 수렴한다.
- `mod.rs`의 액션 처리 책임이 다시 비대해졌으므로, 후속 단계에서 action handler 분리는 여전히 필요하다.

---

## ADR-019: 손상된 설정 파일 가시화와 Auto-Verify 상세 컨텍스트 분리

### Status
Accepted

### Date
2026-04-20

### Context
2차 감사 기준으로 보면 이전 수정 이후에도 두 가지 운영상 빈틈이 남아 있었다.

- `load_config()` 자체는 `Err`를 반환하지만, 앱 초기화가 이를 `ok().flatten()`으로 삼켜서 손상된 `config.toml`이 단순한 "설정 없음"처럼 보였다.
- Auto-Verify는 실패 후 재전송에서 UI용 짧은 요약을 그대로 사용해, 긴 Rust 컴파일 에러의 후반부 원인을 잃을 수 있었다.

### Decision
1. `DomainState::new_async()`는 설정 로드 실패를 삼키지 않고 `config_load_error`로 보존한다.
2. `AppState::apply_startup_config_error()`가 Setup Wizard 첫 화면과 로그 버퍼에 복구/삭제 가이드를 즉시 주입한다.
3. Auto-Verify는 두 종류의 실패 표현을 분리한다.
   - 사용자 UI: 240자 Notice 요약
   - 모델 재전송: `stderr` 우선, `stdout` 보조의 확장 실패 컨텍스트(앞/뒤 보존형)
4. `logs_buffer`는 별도 락을 도입하지 않는다. 비동기 태스크가 직접 버퍼를 공유하지 않고 모두 `Event::Action`으로 직렬화되기 때문이다.

### Alternatives Considered
- **설정 파일 손상 시 즉시 앱 종료**
  - 오류는 명확하지만, 사용자가 같은 화면에서 곧바로 복구를 시작할 수 없으므로 기각.
- **logs_buffer에 `RwLock` 추가**
  - 현재 소유권 모델상 경쟁 상태를 줄이지 못하고 복잡성만 늘리므로 기각.

### Consequences
- 사용자는 설정 파일 손상을 "API 키가 왜 없지?"가 아니라 구체적인 복구 지시로 받게 된다.
- Auto-Verify가 긴 실패 로그의 후반부 단서를 더 잘 보존하여 무의미한 재시도를 줄일 수 있다.
- 이벤트 루프 직렬화 계약이 코드와 문서 양쪽에서 더 분명해졌다.

---

## ADR-020: Linux Shell Sandbox 실체화와 Repo Map 비동기 캐시

### Status
Accepted

### Date
2026-04-20

### Context
3차 감사 기준으로 보면 두 가지 "이름만 있는" 기능이 실제로 남아 있었다.

- `ExecShell`은 Permission 정책만 있었고 OS 레벨 격리가 전혀 없어, 승인 우회 시 호스트 전체에 쓰기를 시도할 수 있었다.
- `Repo Map`은 생성 함수만 있고 실제 요청 주입 경로가 없었으며, 구현 자체도 동기 전체 스캔이었다.

### Decision
1. Linux `ExecShell`은 `bubblewrap (bwrap)`를 기본 샌드박스 백엔드로 사용한다.
   - 호스트 루트는 읽기 전용 바인드
   - 요청 `cwd`만 `/workspace`로 읽기/쓰기 허용
   - `/tmp`는 tmpfs
   - 네트워크는 유지(`--share-net`)하되 PID/IPC/UTS 등은 격리
2. `Repo Map`은 `tokio::task::spawn_blocking`으로 생성하고, `RuntimeState.repo_map`에 캐시/로딩/오류/오염(stale) 상태를 유지한다.
3. 채팅 요청은 준비된 Repo Map cache가 있을 때만 system message로 주입한다.
4. `WriteFile`/`ReplaceFileContent`/`ExecShell` 후에는 Repo Map을 stale 처리하고 백그라운드 재생성을 예약한다.
5. HITL 승인 대기는 5분 TTL을 두고 Tick 루프에서 자동 만료 처리한다.

### Alternatives Considered
- **블랙리스트 기반 PermissionEngine만 유지**
  - 정책 우회 시 물리적 보호막이 없으므로 기각.
- **Repo Map을 요청 직전에 동기 생성**
  - 정확하지만 TUI 프리징을 피할 수 없으므로 기각.
- **파일 감시 기반 증분 인덱서**
  - 장기적으로는 더 좋지만 현재 슬라이스에서 구현 비용이 과도해 우선 stale+background refresh로 절충.

### Consequences
- Linux에서는 shell 실행이 실제 격리 계층을 갖게 되어 "정책만 있는 샌드박스" 상태를 벗어난다.
- Repo Map은 이제 실제 요청에 주입되며, UI 루프를 막지 않는다.
- Windows/macOS의 격리 parity는 아직 남아 있는 후속 과제다.

---

## ADR-021: Phase 17 Host Shell과 Exec Shell 분리 + Workspace Trust Gate 도입 계획

### Status
Implemented

### Date
2026-04-21

### Context
현재 구현은 Windows에서 `ExecShell`이 PowerShell로 실행되지만, 사용자가 실제로 어떤 호스트 콘솔(`cmd.exe`, PowerShell, Windows Terminal) 위에서 앱을 띄웠는지와는 분리되어 있다. 또한 현재 작업 루트가 신뢰된 폴더인지 확인하는 단계가 없어, 사용자 관점에서 “어느 범위까지 제어권을 위임했는지”가 명확하지 않다.

### Decision
1. `Host shell`과 `Exec shell`은 별도 개념으로 문서와 UI에 구분해 노출한다.
2. Windows exec shell은 `pwsh.exe`를 우선 사용하고, 없으면 `powershell.exe`로 fallback 한다.
3. 앱 시작 시 현재 workspace root에 대한 `Workspace Trust Gate`를 통과해야 쓰기/셸 실행이 가능하다.
4. Trust 상태는 `Unknown / Trusted / Restricted` 3상태로 동결한다.
5. `Restricted`는 읽기 전용 탐색 모드이며, 쓰기/셸 실행은 Permission Engine 단계에서 강제 차단한다.
6. trust 상태는 시작 시점 1회 선택만으로 끝내지 않고, REPL 명령과 설정 UI에서 계속 관리할 수 있게 한다.
7. `denied_roots`는 trust 상태보다 우선하는 전역 차단 규칙으로 취급한다.
8. 추가 workspace 디렉터리(`extra_workspace_dirs`)는 Gemini CLI의 multi-directory 지원 패턴처럼 별도 목록으로 관리한다.

### Alternatives Considered
- **호스트 콘솔을 강제로 PowerShell 창으로 재실행**
  - 사용자의 기존 터미널 세션을 끊고 UX가 과도하게 침습적이므로 보류.
- **Trust Gate 없이 경고 메시지만 표시**
  - 사용자 의사결정이 구조화되지 않아 보호 루트 개념이 약해지므로 기각.
- **시작 시 1회 trust prompt만 두고 이후 수정 불가**
  - 운영 중 workspace 범위와 차단 루트를 조정할 수 없어 실제 사용성/관리성이 떨어지므로 기각.

### Consequences
- 작업 루트에 대한 신뢰 여부가 명시적 상태가 되어, 추후 보안 정책과 권한 모델을 더 정교하게 확장하기 쉬워진다.
- trust workspace / deny roots / extra dirs를 REPL과 설정에서 함께 관리할 수 있어 장기 세션 운영성이 높아진다.

---

## ADR-022: Race Condition Prevention & Block Isolation in Async LLM Responses

### Status
Implemented

### Date
2026-04-21

### Context
초기 구현에서는 LLM 응답 스트리밍(`ChatDelta`, `ChatResponseOk`)을 렌더링할 때 무조건 `timeline.last_mut()`에 의존했다. 이로 인해 LLM이 응답을 생성하는 도중(`is_thinking == true`) 사용자가 새 프롬프트를 제출하거나, 도구 실행(`ToolFinished`) 후 LLM에 후속 요청을 보내는 과정에서 새 블록이 타임라인의 맨 끝에 추가되면, 진행 중이던 이전 응답 스트림이 잘못된 블록(예: 사용자의 새 입력 블록이나 다른 도구 블록)에 덮어써지는 심각한 Race Condition(블록 오염)이 발생했다.

### Decision
1. **`active_chat_block_idx` 도입**: LLM 요청(자연어 전송 및 도구 후속 전송)이 발생하는 시점에 `timeline`의 길이를 측정하여 갱신할 타겟 블록의 인덱스를 `RuntimeState`에 저장한다.
2. **명시적 인덱스 라우팅**: 모든 비동기 응답 액션(`ChatDelta`, `ChatResponseOk`, `ChatResponseErr`)은 `last_mut()`가 아닌 `active_chat_block_idx`에 해당하는 블록만을 찾아가서 갱신한다.
3. **`is_thinking` 철저한 잠금**: 스트리밍 중간(`ChatDelta` 수신 시점)에 `is_thinking`을 해제하던 버그를 제거하여, 요청이 시작되고 끝날 때까지 완전한 `true` 상태를 유지하도록 변경한다.
4. **사용자 입력 원천 차단**: `is_thinking == true`일 때 사용자가 Composer에서 `Enter`를 누르거나 명령어를 실행하려고 하면 즉각 거부(`[Warning]`)하여 동시 요청으로 인한 상태 머신 교란을 아키텍처 레벨에서 차단한다.
5. **통합 헬퍼 사용**: `dispatch_chat_request`와 `send_chat_message_internal` 양쪽 모두 블록 생성과 인덱스 할당을 통일하기 위해 `spawn_chat_request` 헬퍼 메서드로 리팩토링한다.

### Alternatives Considered
- **액션 메시지에 블록 ID/Request ID 포함**
  - 안전한 방법이지만, Provider Adapter 규격을 변경하고 기존 이벤트 구조체를 대규모 수정해야 하므로 오버헤드가 크다고 판단하여 기각. 
- **`chat_in_flight` 플래그 도입**
  - 단순 플래그만으로는 "어디에 기록해야 하는지"를 지정할 수 없으므로 근본 해결책이 아님.

### Consequences
- 동시 요청으로 인한 블록 텍스트 섞임 현상이 원천 차단되었다.
- LLM과 통신 중 사용자의 예기치 못한 단축키/명령어 난입으로 인한 런타임 Crash 및 예외 상황 방어력이 대폭 상승했다.
- 네트워크 비동기 렌더링의 타겟이 확정적이므로 안정성이 극대화되었다.

---

## ADR-023: Phase 18 Multi-Provider Architecture & 2026.04 Grounding

### Status
Accepted (구현 예정)

### Date
2026-04-21

### Context
SMLCLI는 초기 개발 시 OpenAI와 Gemini만을 타겟으로 구성되었으나, 2026년 4월 기준 AI 생태계는 Anthropic의 `claude-4.7` 및 xAI의 `grok-4.3-beta` 등 강력한 멀티모달/추론 모델들이 약진하고 있다. 이에 따라 다양한 Provider의 API 구조를 추상화할 수 있는 Registry/Adapter 패턴의 도입이 필수적이며, 동시에 파일 탐색과 외부 정보 수집을 돕는 `ListDir`, `GrepSearch`, `FetchURL` 도구를 추가하여 에이전트 성능을 고도화해야 한다.

### Decision
1. **Provider Registry Pattern**: 기존의 모놀리식 API 호출 로직을 분리하여 `ProviderAdapter` 트레이트를 구현한 각 Provider별 구조체(`OpenAIAdapter`, `AnthropicAdapter`, `xAIAdapter`)로 라우팅한다.
2. **Anthropic Messages API 지원**: OpenAI 호환 포맷이 아닌 Anthropic 전용의 Messages API 및 고유의 SSE 스트리밍 청크 구조를 파싱하는 어댑터를 독립 구현한다.
3. **2026.04 최신 라인업 그라운딩**: `gpt-5.4-pro`, `claude-4.7`, `grok-4.3-beta` 등 2026년 4월 현재 기준의 최신 모델을 설정의 기본 선택지로 업데이트한다.
4. **Advanced Tools**: 디렉터리 브라우징을 위한 `ListDir`, 정규표현식 지원 검색 `GrepSearch`, 외부 문서를 Markdown으로 요약하는 `FetchURL` 도구를 추가한다.

### Alternatives Considered
- **서드파티 LLM 라우팅 라이브러리 사용**
  - 의존성을 줄이고 SSE 스트리밍의 세밀한 UI 제어를 위해 자체 어댑터 패턴을 구축하기로 함.
- **모든 Provider를 OpenAI 호환으로 강제**
  - Anthropic 등 일부 Provider는 독자적인 기능(예: 캐싱, 특수 도구 호출 방식)을 지원하므로 Native API를 존중하는 것이 장기적으로 유리함.

### Consequences
- 사용자는 `config.toml`이나 `/setting`에서 자유롭게 OpenAI, Anthropic, xAI 등을 넘나들며 최신 모델을 사용할 수 있다.
- 에이전트는 파일의 상세 검색 및 웹 정보 검색 능력을 갖춰 로컬 코드베이스 외의 영역까지 작업을 확장할 수 있다.

---

## ADR-024: 시스템 결함 분석 및 수정 지침 (v1.0.0 Audit Remediation)

### Status
Accepted (구현 예정)

### Date
2026-04-21

### Context
v1.0.0 출시를 앞두고 외부 감사(Audit)에서 9개의 핵심 결함이 식별되었습니다.
1. Wizard 필수 필드 미입력 시 상태 전이 오류
2. 도구 런타임(ToolRuntime)의 글로벌 상태 직접 조작으로 인한 무상태 원칙 위반
3. 이벤트 루프 내 `Ctrl+C` 취소 처리 시 채널 블로킹으로 인한 데드락
4. `infra` 계층의 에러가 `Box<dyn Error>`로 처리되어 타입 정보를 잃는 문제
5. 수만 줄의 TUI 로그 렌더링 시 발생하는 프레임 블로킹
6. 위저드 설정 탭 순환 포커스 오류
7. 도구 실행 종료 후 StatusBar 미갱신 지연
8. `SessionLogger` 파일 핸들 미반환 리소스 누수
9. 권한 검증 시 와일드카드(`rm -rf *` 등) 우회 가능성

### Decision
4단계(Phase 1~4)에 걸쳐 결함을 수정하기로 결정했습니다:
1. **Phase 1 (Error & Resource)**: `SmlError`를 통합하여 `InfraError`, `IoError` 확장. `SessionLogger` 내부 필드로 `BufWriter<File>` 소유 및 `Drop` 시 명시적 flush 처리.
2. **Phase 2 (Logic & Security)**: `glob` 기반 블랙리스트(`*`, `../`) 감지 로직 `is_dangerous()` 도입. 위저드에서 필수값 미입력 시 `WizardError::MissingRequiredField` 반환 후 현재 탭 강제 유지.
3. **Phase 3 (Runtime & Concurrency)**: `ToolRuntime::execute()` 반환형을 `Result<ToolResult, ToolError>`로 제한해 전역 상태 수정 권한 박탈. `tokio_util::sync::CancellationToken`으로 select 경합 처리하여 데드락 제거.
4. **Phase 4 (TUI & UX)**: `inspector_tabs.rs`에 `terminal_height`에 맞춘 윈도우 슬라이싱 적용 (`MAX_LOG_LINES = 5000`). `WizardField` 열거형 배열 기반 강제 포커스 순환 구현. `StatusBar::clear()` 명시적 트리거.

### Consequences
- 아키텍처의 무상태 원칙(Stateless)과 단방향 이벤트 플로우가 엄격히 보장됩니다.
- 대규모 로그 환경 및 강제 종료 상황에서의 멈춤(Freezing)이 원천 차단됩니다.
- 보안 우회 경로(디렉터리 횡단, 와일드카드 폭격)를 차단하여 시스템 신뢰도가 상승합니다.
