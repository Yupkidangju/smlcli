# Changelog

모든 중요한 변경 사항은 이 문서에 기록됩니다.
이 프로젝트는 [Semantic Versioning](https://semver.org/) 기준을 따릅니다.

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
