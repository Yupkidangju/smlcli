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
2. `timeline_entries: Vec<TimelineEntry>` 도입 — session.messages(LLM)와 분리
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
