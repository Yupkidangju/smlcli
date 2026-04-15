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
Accepted

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
- `command_router.rs`: 12개 슬래시 커맨드의 파싱과 실행 (165줄)
- `chat_runtime.rs`: LLM 요청 조립, API 키 조회, Provider 디스패치 (78줄)
- `mod.rs`: 이벤트 루프, 최상위 디스패치, Fuzzy Finder만 보유 (~530줄→실행 흐름만)

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
