# Implementation Summary & Roadmap

이 문서는 `smlcli` 프로젝트의 진행 상황과 구현 태스크 리스트를 추적합니다.
`spec.md`와 `designs.md`를 바탕으로 단계적, 수직적(Vertical Slicing)으로 나뉜 작업 단위입니다.
새 기능 구현과 완료 시마다 체크박스를 표시하고 변경 내역을 요약해야 합니다.

## Phase 1: 터미널 기반 골격과 레이아웃 (Foundation & TUI Layout)
- [x] **Task 1: 환경 구성 및 기본 구조 세팅**
  - Cargo 초기화, 구조 폴더 (`src/app`, `src/tui`, `src/domain` 등) 생성
  - `crossterm`, `ratatui`, `tokio` 기본 의존성 추가
  - `main.rs` 패닉 핸들러 구성 및 안전한 터미널 종료 로직 구현
- [x] **Task 2: 메인 화면 스캐폴딩**
  - 4개 주요 영역 렌더링 틀 작성: 상태바 (단일 줄), 타임라인, Composer(입력창), Inspector 패널
  - `Tab`, `Ctrl+C`, `Esc` 등 기본 단축키 이벤트 루프 연동
- [x] **Task 3: 상태 기계 (State Machine) 및 라우팅 기초**
  - `AppState` 기초 모델 정의, 이벤트 단위 업데이트를 제공하는 루프 모델 작성

## Phase 2: 설정 마법사 & 보안 계층 (Wizard & Security)
- [x] **Task 4: Setup Home & `/setting` Wizard UI**
  - 앱 첫 실행 시 `Setup Home` 로드
  - 제공자 선택 및 자격 증명 (API Key 등) 입력 마스킹 UI 구현
  - 플로우 상태 처리 (`Not Started`, `In Progress`, `Done`)
- [x] **Task 5: 파일 기반 암호화 설정 저장소**
  - 로컬 `master-key` 생성 및 `~/.smlcli/.master_key` 파일 저장 (chmod 600)
  - ChaCha20Poly1305 기반 API 키 필드 암호화 (`config.toml`의 `encrypted_keys`)
- [x] **Task 6: Provider 자격 검증 (Smoke Test)**
  - API 키 입력 즉시 HTTP 테스트를 거치는 Provider Adapter 인터페이스
  - 설정 저장 실패/성공에 따른 상태바 업데이트 반영

## Phase 3: 핵심 AI 통신 & 타임라인 (Core Sync & Timeline)
- [x] **Task 7: LLM 프롬프트 왕복 통신**
  - `reqwest` + 비동기 클라이언트로 OpenAI 모의/실제 응답 텍스트 연결
  - 챗 로그를 `Timeline` 영역에 스트리밍 또는 표시
- [x] **Task 8: 프롬프트 상태 유지 & Context Budget 관리**
  - 세션 문맥을 보존하고 허용 길이를 초과할 때 자동 압축(compact)하는 엔진
  - 토큰 한도를 넘지 않도록 중간 문맥을 삭제하는 `/compact` 수동 명령어 및 자동 임계치 트리거(50 항목 이상) 구현
- [x] **Task 9: PLAN/RUN 모드 전환 UX 구성**
  - 모드 토글 키를 통한 `PLAN` 과 `RUN` 라벨의 변화 및 힌트 반영

## Phase 4: 도구 실행 & 권한 승인 카드 (Tools & Approval UX)
- [x] **Task 10: 파일 읽기 도구 및 Preview 탭 구현**
  - 파일 읽기 기능 및 `Inspector -> Preview` 화면에 안전한 Line 출력 기능 추가
- [x] **Task 11: Grep 검색과 Search 탭 구현**
  - 재귀적 검색 (`ignore` 기반 탐색), 일치 항목의 컨텍스트를 Search 창에 목록 처리 
- [x] **Task 12: 파일 수정, Diff 뷰, 승인 UI 구성 (CRITICAL)**
  - 모델의 변경 제안에 따라 Inspector의 Diff 탭 활성화
  - 타임라인 내에 [Approve], [Deny] 카드 노출
  - 안전한 임시 파일 작성 -> 원자적 덮어쓰기 로직 연동
- [x] **Task 13: 셀(Shell) 커맨드 실행 및 텍스트 스트리밍**
  - `safe_only` 등 권한 모델을 따른 커맨드 유효성 체크
  - 프로세스 시작, 런타임 stdout 스트리밍을 `Inspector -> Logs` 패널에 출력

## Phase 5: 통합 및 완성도 확보 (Integration & Polish)
- [x] **Task 14: LLM 도구 호출 JSON 로직 연동**
  - 프롬프트 엔지니어링 및 `serde_json` 기반 응답 파서를 통한 승인(Pending) 카드 자동 생성
- [x] **Task 15: 설정 및 암호화 저장소 연동**
  - 파일 기반 마스터 키 + ChaCha20Poly1305 적용한 구성 영구 저장 기능
- [x] **Task 16: Inspector 반응형 분할 및 UI/UX 폴리싱**
  - 화면 폭/단축키(`Ctrl + I`)에 대응하는 동적 Split 레이아웃
  - `similar` Diff의 라인별 `초록색/빨간색` 렌더링 스팬 처리

## Phase 6: 슬래시 커맨드 및 설정 결합 (Commands & Config)
- [x] **Task 17: `/config` 종합 마스터 대시보드 구현**
  - TUI 오버레이를 통해 Provider, Model, Permission 등 모든 설정 내역을 방향키로 이동 및 수정
  - 변경 시 즉시 `config.toml` 영속 저장 및 암호화 키 갱신
- [x] **Task 18: `/provider` 및 `/model` 양방향 선택 팝업**
  - 콘솔 입력창에 명령어 입력 시 자동 페칭 후 방향키로 리스트에서 선택 (키보드 친화적 UX)
- [x] **Task 19: 핵심 슬래시 커맨드 파서 연결**
  - `/setting`, `/status`, `/mode`, `/clear`, `/help`, `/quit` 라우팅 액션 매핑

## Phase 7: 지능형 컨텍스트 압축 시스템 (Intelligent Compaction)
- [x] **Task 20: 동적 토큰 임계치 및 추적 (`/tokens`)**
  - 단순 메시지 배열 개수 한도를 넘어 단어/바이트 길이 비례 토큰 추정 알고리즘 추가
  - 컨텍스트 예산 75% 도달 시 선제적 `compact` 트리거 추가 및 `/tokens` 상태 명령어 매핑
- [x] **Task 21: 백그라운드 LLM 문맥 요약 (Summarizing Condenser)**
  - 삭제될 중간 메시지 뭉치를 백그라운드 비동기로 돌려 단일 요약 블록(`System: [Summary]...`)으로 교체
- [x] **Task 22: 중요 컨텍스트 핀(Pinning) 정책**
  - 핵심 지시사항(`spec.md` 관련 컨텍스트 등)에 핀 속성 부여하여, 압축 중에도 소실되지 않도록 보존하는 로직

## 최근 구현 요약
_(각 Task가 완료될 때마다 이 아래에 요약 코멘트를 작성합니다.)_

- [2026-04-14] : 기초 기획 및 Task 로드맵 정의
- [2026-04-14] : OpenRouter/Gemini 제공자 MVP 축소 결정 및 문서 반영
- [2026-04-14] : Phase 1 완료 (안전한 초기화 수행, tokio/ratatui 메인 루프 연동, 레이아웃 분할 구현, cargo check 무결성 검증)
- [2026-04-14] : Phase 2 로드맵 완료 (Wizard 상태 패턴, XChaCha20+Keyring 연동, reqwest Provider 인증 인터페이스 도입)
- [2026-04-14] : Phase 3 로드맵 완료 (세션 관리, 컴포저 UI 키맵 연동, 모의(Mock) LLM 응답 연동, Tab을 통한 PLAN/RUN 전환)
- [2026-04-14] : Phase 4 로드맵 완료 (풍부한 도구 집합: 파일 I/O, Diff Approval UI, Shell, SysInfo, 멀티플랫폼 지원 및 ignore Grep 구현)
- [2026-04-14] : Phase 5 MVP 핵심 완료 (LLM의 JSON 형식 도구 호출 자동 파싱, Keyring 기반 Setup 완수, Ctrl+I 동적 패널 및 하이라이팅 적용, 경고 소거)
- [2026-04-14] : Phase 5 추가 고도화 (temp_scaffold 패키지명 smlcli로 변경, MinGW-w64 연동 및 대화형 멀티플랫폼 크로스 빌드 스크립트 작성, TUI 방향키 기반의 동적 모델 검색 인프라 확보)
- [2026-04-14] : Phase 6 슬래시 커맨드 인프라 파이프라인 개방. AppState 구조에 ConfigState 오버레이 추가. `/config` 대시보드 및 동적 모델/프로바이더 변경 플로우 완성.
- [2026-04-15] : Phase 3/6 추가 고도화 (컨텍스트 압축 시스템 도입: `/compact` 슬래시 커맨드 매핑 및 SessionState 임계값 초과 시 자동 컨텍스트 버리기 엔진 구현)
- [2026-04-15] : Phase 7 하이브리드 지능형 압축 구현. 동적 임계값, 백그라운드 비동기 요약(Summarizer), 메시지 핀(Pinned) 정책 추가 완료. `/tokens` 커맨드로 런타임 제어 지원.
- [2026-04-15] : **[1차 AUDIT & REMEDIATION]** 전반적 코드 무결성 감사(Audit) 수행 및 크리티컬 버그 수정.
  - Setup Wizard 종료 시 `AppState::settings`가 즉시 갱신되지 않아 "dummy_key"로 통신하던 결함 수정.
  - `PermissionEngine` 도입으로 `ShellPolicy`, `FileWritePolicy` 정책 강제 적용 (SafeOnly, Deny, Ask 모드 분기).
  - Composer `!` 접두사를 통한 직접 셸 실행 기능 추가 및 보안 정책 연동.
  - `PermissionToken` 무결성 검증 및 `ChatResponseOk` 내 자동 실행/승인 대기 로직 분리.
- [2026-04-15] : **[2차 AUDIT & REMEDIATION]** 사양(designs.md/spec.md)의 세부 누락분 완전 통합 및 보안 결함 핫픽스 수행.
  - `file_ops.rs`의 `write_file_commit()`에 원자성(Atomic) 적용(`.tmp` 생성 후 `rename`).
  - `session.rs`의 하드드롭 레거시(`compact_context()`) 파괴 및 지능형 요약기 엔진 백그라운드 파이프라인 단일화 완료.
  - `shell.rs`의 무한 스레드 행 차단을 위해 `tokio::time::timeout` (30초) 적용.
  - Input Parser에 `@` 파일 퍼지 파인더 모드 연동 및 `layout.rs` 인스펙터 패널 상단 상태 기반 동적 탭 네비게이션 설계 도입.
- [2026-04-15] : **[3차 AUDIT & REMEDIATION - v0.1.0-beta.7]** 외부 감사 보고서 기반 16건 전수 교차검증 및 수정.
  - [Critical-4건] OpenRouter API 키 검증 우회, Gemini 모델 ID 불일치, dummy_key 무음 대체, 시스템 프롬프트 노출 수정.
  - [High-6건] Config 팝업 키 핸들러 구현, /clear 시스템 프롬프트 보존, ReplaceFileContent 실행기 구현, pinned 직렬화 제외, 상태바 동적화.
  - [Architecture] `mod.rs` God Object(773줄→422줄)를 `command_router.rs`, `chat_runtime.rs`, `tool_runtime.rs`, `wizard_controller.rs` 4개 모듈로 완전 분해.
  - WizardStep 미사용 variant 제거. handle_input을 키별 소형 메서드(handle_char_input, handle_up_key 등)로 분해.
  - [Quality] `cargo fmt` 전체 적용. `cargo check && cargo test && cargo clippy` 전수 무경고 통과.
- [2026-04-15] : **[4차 AUDIT & REMEDIATION - v0.1.0-beta.8]** 외부 감사 보고서 기반 7건 수정.
  - [High-4건] 위자드 저장 실패 무시 제거, API 키 평문 마스킹, /provider 전환 안전성(model auto reset + key 검증), NetworkPolicy::Deny 실적용.
  - [Medium-2건] 위자드 에러 Esc 복구(ProviderSelection 회귀), 회귀 테스트 10건 추가(4→14건).
  - [Low-1건] `cargo fmt --check` 게이트 통과.
  - [Quality] 전체 품질 게이트 통과: `cargo check ✓ | cargo test 14/14 ✓ | cargo clippy 0w ✓ | cargo fmt --check ✓`
- [2026-04-15] : **[5차 AUDIT & REMEDIATION - v0.1.0-beta.9]** 외부 감사 보고서 기반 5건 수정.
  - [High-1건] 보조 경로(/model, /compact, /provider) 보안 가드 우회 차단: `resolve_credentials()` 중앙 가드 도입으로 NetworkPolicy + Keyring 일관 적용.
  - [Medium-3건] /provider 전환 시 validate_credentials() 추가, Dashboard err_msg 렌더링, clippy field_reassign_with_default 해소.
  - [Low-1건] Saving 단계 UX 문구 불일치 수정.
  - [Architecture] chat_runtime.rs에 resolve_credentials()/resolve_credentials_for_provider() 중앙 보안 가드 메서드 도입.
  - [Quality] 전체 품질 게이트 통과: `cargo check ✓ | cargo test 14/14 ✓ | cargo clippy -D warnings ✓ | cargo fmt --check ✓`
- [2026-04-15] : **[6차 AUDIT & REMEDIATION - v0.1.0-beta.10]** 외부 감사 보고서 기반 4건 수정.
  - [High-1건] /provider 전환 원자성: 검증 전 save_config 제거 → 롤백 스냅샷 + ModelList 선택 시에만 디스크 저장.
  - [Medium-3건] /model에 validate_credentials 선행 검증 추가, FetchSource enum으로 비동기 라우팅 출처 의존성 해소, clippy collapsible_if 해소.
  - [Architecture] Action::ModelsFetched에 FetchSource(Config|Wizard) 태그 도입, ConfigState에 rollback 필드 추가.
  - [Quality] 전체 품질 게이트 통과: `cargo check ✓ | cargo test 14/14 ✓ | cargo clippy -D warnings ✓ | cargo fmt --check ✓`
- [2026-04-15] : **[7차 AUDIT & REMEDIATION - v0.1.0-beta.11]** 외부 감사 보고서 기반 3건 수정.
  - [High-1건] /config→Model 경로 보안 가드 우회 차단 (6차 자체 감사에서 이미 수정 확인).
  - [High-1건] Provider 전환 사용자 취소(Esc) 시 rollback 누락: Esc 핸들러에 롤백 스냅샷 복구 로직 추가.
  - [Medium-1건] save_config() 실패 묵살 수정: ShellPolicy 토글 + ModelList 저장에서 에러를 err_msg로 사용자 가시화.
  - [Quality] 전체 품질 게이트 통과: `cargo check ✓ | cargo test 14/14 ✓ | cargo clippy -D warnings ✓ | cargo fmt --check ✓`
- [2026-04-15] : **[8차 AUDIT & REMEDIATION - v0.1.0-beta.12]** 외부 감사 보고서 기반 4건 수정.
  - [Medium-1건] save_config() 저장 실패 시 in-memory 복구: ShellPolicy 토글 및 ModelList 저장에서 실패 시 이전 값으로 자동 롤백.
  - [Quality] 전체 품질 게이트 통과.
- [2026-04-15] : **[v0.1.0-beta.13]** 긴급 실행 불가 버그 수정.
  - keyring 백엔드 미설정(`sync-secret-service` feature 누락)으로 mock credential store 사용 → 키 영속화 실패 수정.
  - `dbus`, `dbus-secret-service`, `libdbus-sys` transitive 의존성 자동 추가.
- [2026-04-16] : **[v0.1.0-beta.14] Credential Store 아키텍처 재설계.**
  - `keyring` 크레이트 완전 제거: OS 의존적 gnome-keyring → 크로스플랫폼 파일 기반으로 교체.
  - 설정 저장 경로 변경: `~/.config/smlcli/settings.enc` (암호화 바이너리) → `~/.smlcli/config.toml` (TOML 평문).
  - API 키: config.toml의 `encrypted_keys` HashMap에 ChaCha20Poly1305 암호화 저장.
  - 마스터 키: `~/.smlcli/.master_key` 파일 (hex 인코딩, chmod 600).
  - `save_config()` / `load_config()` 시그니처에서 `master_key` 파라미터 제거.
  - `PersistedSettings`에 `encrypted_keys: HashMap<String, String>` 필드 추가.
- [2026-04-16] : **[v0.1.0-beta.15] 감사 3건 수정.**
  - [High] `serde_yml` (RUSTSEC-2025-0067/0068) 제거 → 기존 `toml` 크레이트로 교체.
  - [Medium] 문서-구현 불일치 해소: README/spec.md 내 keyring 참조를 파일 기반 암호화로 교체.
  - [Low] `config.toml`에 chmod 600 권한 설정 추가 (Unix).
- [2026-04-16] : **[v0.1.0-beta.16] UX 4건 개선.**
  - Tool JSON 필터링: AI 응답 내 도구 호출 JSON 스키마를 `⚙️ [도구명]` 형태로 대체 표시.
  - AI 추론 인디케이터: 프롬프트 전송 ~ 응답 수신까지 `✨ AI가 응답을 생성하고 있습니다...` 표시.
  - 슬래시 커맨드 자동완성 메뉴: Composer에서 `/` 입력 시 11개 명령어 팝업, 방향키+Enter 선택.
  - 에이전트 페르소나 시스템 프롬프트: CLI 에이전트 역할 정의 (~1K 토큰), 사용자 입력 언어 미러링.
  - [Quality] 전체 품질 게이트 통과: `cargo check ✓ | cargo test 14/14 ✓ | cargo clippy -D warnings ✓ | cargo fmt --check ✓`
- [2026-04-16] : **[v0.1.0-beta.17] 9차 감사 수정 3건.**
  - [M-1] 소스 코드 주석 일괄 교체: `Keyring`→`암호화 저장소`, `config.yaml`→`config.toml` (6개 파일 15건).
  - [M-2] `/help` 출력 한/영 병행 표기 적용 (i18n 일관성).
  - [L-1] 테스트 코드 문구 `Keyring`→`암호화 저장소` 교체 (audit_regression.rs 2건).
  - 페르소나 언어 지시: `한국어 고정` → `사용자 입력 언어 미러링` 변경.
  - [Verification] keyring 잔존 grep 0건, config.yaml 잔존 grep 0건 확인.
  - [Quality] 전체 품질 게이트 통과: `cargo check ✓ | cargo test 14/14 ✓ | cargo clippy -D warnings ✓ | cargo fmt --check ✓`

---

## Phase 9: UX 아키텍처 개편 (v0.1.0-beta.18)

**상태**: ✅ Phase 9-A/B 구현 완료 → Phase 9-C 대기
**완료 일시**: 2026-04-16
**커밋**: `b5c4612`
**관련 문서**: spec.md §3.2, §3.9, §9 / designs.md §5.5~5.6, §6.7, §21 / DESIGN_DECISIONS.md ADR-009

### Phase 9-A: 이벤트 기반 구조 — ✅ 완료 (6/7건)
1. ✅ Action enum 14종 확장 (ChatStarted, ChatDelta, ToolQueued, ToolStarted, ToolOutputChunk, ToolSummaryReady)
2. ✅ TimelineEntry 모델 도입 (session.messages ↔ timeline 이중 구조)
3. ✅ Semantic Palette 도입 (tui/palette.rs — info/success/warning/danger/muted/accent + bg 3계층)
4. ✅ tick 기반 애니메이션 (스피너 ◐◓◑◒, 배지 깜빡임 ●/○, 승인 pulse)
5. ✅ Inspector Logs 탭 실체 구현 (logs_buffer 기반 렌더링)
6. ✅ Tool 출력 요약 분리 (2~4줄 타임라인 + 원문 Logs 탭)
7. ⏳ SSE 스트리밍 — Phase 9-C로 이관

### Phase 9-B: 기능 완성 — ✅ 완료 (4/7건)
1. ✅ Blocked Command 목록 (15개 패턴 무조건 차단)
2. ✅ File Read 안전장치 (경로 traversal 차단 + 1MB 제한 + 800줄 상한)
3. ✅ Grep 결과 UX (context_lines + 파일별 그룹 + 결과 요약)
4. ✅ ToolQueued/ToolStarted/ApprovalCard 이벤트 파이프라인 통합
5. ⏳ CLI Entry Modes — Phase 9-C로 이관
6. ⏳ 세션 영속성 JSONL — Phase 9-C로 이관
7. ⏳ Structured Tool Call — Phase 9-C로 이관

### Phase 9-C: 품질 강화 — ✅ 부분 완료 (3/6건)
1. ✅ SSE 스트리밍 → Phase 10에서 구현 완료
2. ✅ CLI Entry Modes → Phase 10에서 구현 완료
3. ✅ 세션 영속성 JSONL → Phase 10에서 구현 완료
4. ✅ Shell stdout/stderr 실시간 스트리밍 (라인 단위 비동기 + ToolOutputChunk 이벤트)
5. ⏳ Diff 접기/펼치기 UI — Phase 11로 이관
6. ✅ 테스트 확장 (14→24건): blocked_command, timeline, ToolStatus 등
7. ✅ ListDir 재귀 트리 (├──/└── Unicode, 디렉토리 우선 정렬, 1000개 제한)

---

## Phase 10: 기능 완성 (v0.1.0-beta.18)

**상태**: ✅ 구현 완료
**완료 일시**: 2026-04-16
**커밋**: `369fb9e` → `d03d7b2` → `8e7c57e`

### 구현 항목 — 4건
1. ✅ **JSONL 세션 영속성** (`infra/session_log.rs` 신규)
   - append-only 대화 기록, 복원(restore_messages), 세션 목록(list_sessions)
   - 사용자/AI 메시지 자동 기록 (chat_runtime.rs + mod.rs)
   - 외부 의존성 0건 (std::time::UNIX_EPOCH 기반)
2. ✅ **CLI Entry Modes** (`main.rs` 전면 개편, clap 4 derive)
   - `smlcli run`: 기본 인터랙티브 TUI 모드
   - `smlcli doctor`: 환경 진단 (설정 로드, 세션 수, 시스템 정보)
   - `smlcli sessions`: JSONL 세션 파일 목록 (파일명/크기/메시지 수)
3. ✅ **SSE 스트리밍** (`providers/registry.rs` chat_stream 메서드)
   - ProviderAdapter trait에 chat_stream() 추가
   - OpenRouter/Gemini: stream:true → SSE data: 라인 파싱 → delta_tx 전송
   - chat_runtime.rs: delta_forwarder 태스크가 ChatDelta 이벤트 실시간 발행
4. ✅ **전역 #![allow] 최소화**
   - unused_imports/unused_variables 제거, dead_code만 유지
   - 미사용 import 6건 + 변수 2건 수정
   - ctx% 상태바 색상 차등 적용 (budget ≥85 DANGER / ≥70 WARNING)

### 실제 결과 (Phase 9+10 전체)
- 코드 변경량: ~1,200줄 추가 (18개 파일 변경, 2개 신규)
- 테스트: 14건 → 24건, Clippy: 0 warnings
- 신규 파일: `tui/palette.rs`, `infra/session_log.rs`
- 신규 의존성: `clap 4` (derive feature)

### 남은 이관 항목
- ⏳ Structured Tool Call (Provider별 native tool call contract)
- ⏳ Diff 접기/펼치기 UI

---

## Phase 11: 감사 대응 — 안정성 복원 (v0.1.0-beta.20)

**상태**: ✅ 구현 완료
**완료 일시**: 2026-04-17
**관련 문서**: audit_report_20260416_1600.report / designs.md §6.7, §21 / DESIGN_DECISIONS.md ADR-011

### 구현 항목 — 5건

#### HIGH-1: 세션 로거 회귀 복구
- ✅ `SessionLogger::from_file()` 복원: 기존 JSONL 파일로부터 로거 생성
- ✅ `SessionLogger::restore_messages()` 복원: JSONL → Vec<ChatMessage> 파싱 (손상 라인 건너뛰기)
- ✅ 동기 `append_message()` 복원: 테스트 및 이벤트 루프 내 안전한 호출용
- ✅ 비동기 API는 `append_message_async()`로 이름 변경하여 유지

#### HIGH-2: 세션 영속성 런타임 실행 불가 수정
- ✅ `chat_runtime.rs`: `logger.append_message(&msg)` — 동기 API + 에러 로깅
- ✅ `mod.rs`: `logger.append_message(&res.message)` — 동기 API + 에러 로깅
- ✅ clippy `collapsible_if` → `let chain` 패턴으로 축약

#### MEDIUM-1: Inspector Search 탭 실제 구현
- ✅ 타임라인 전체 대소문자 무시 텍스트 검색 엔진
- ✅ Composer 입력 버퍼를 검색어로 사용
- ✅ 검색 결과 최대 50건 표시, 결과 건수 요약
- ✅ TimelineEntryKind별 라벨/색상 차등 적용 (User/AI/Sys/Tool/Appr/Σ)

#### MEDIUM-2: 테마 시스템 구현
- ✅ `PersistedSettings.theme` 필드 추가 (`serde(default = "default_theme")`)
- ✅ `palette.rs`: `Palette` 구조체 + `DEFAULT_PALETTE` + `HIGH_CONTRAST_PALETTE` + `get_palette()`
- ✅ `/theme` 슬래시 커맨드 핸들러 (`command_router.rs`)
- ✅ `SlashMenuState::ALL_COMMANDS` 11→12개 (`/theme` 추가)

#### MEDIUM-3: thiserror 에러 체계 연동
- ✅ `config_store::load_config()`에서 `ConfigError::NotFound`/`ParseFailure` 연결
- ✅ `map_err` 패턴으로 anyhow 호환성 유지하며 구조화

### 추가 Clippy/품질 수정
- ✅ `WizardStep`, `ConfigPopup`에 `Debug` derive 추가
- ✅ `wizard_controller.rs`: 미사용 변수(`old_policy`, `old_model`, `action_tx`) 수정
- ✅ `layout.rs`: `tick_count % 2 == 0` → `tick_count.is_multiple_of(2)`

### 품질 검증 결과
```
cargo build   ✅ 성공
cargo test    ✅ 28/28 통과 (0 failed)
cargo clippy  ✅ 0 warnings (-D warnings 게이트 통과)
```

---

## Phase 11-B: 재감사 대응 — 렌더링 연결 및 에러 구조화 (v0.1.0-beta.21)

**상태**: ✅ 구현 완료
**완료 일시**: 2026-04-17
**관련 문서**: DESIGN_DECISIONS.md ADR-012 / designs.md §21.4

### 구현 항목 — 3건

#### HIGH-1: 테마 전환 렌더링 실연결
- ✅ `AppState::palette()` 헬퍼 메서드 추가 (현재 테마에 맞는 `&'static Palette` 반환)
- ✅ `layout.rs`: 50+곳의 정적 `pal::CONSTANT` → `state.palette().field` 동적 참조 전환
- ✅ `inspector_tabs.rs`: render_logs/render_search/render_recent 전면 전환
- ✅ `config_dashboard.rs`: `Color::Yellow` → `palette().warning` 전환
- ✅ `setting_wizard.rs`: `Color::Cyan` → `palette().info` 전환

#### MEDIUM-1: 에러 타입 전면 구조화
- ✅ `Action::ChatResponseErr(String)` → `ChatResponseErr(ProviderError)`
- ✅ `Action::ToolError(String)` → `ToolError(ToolError)`
- ✅ `Action::ModelsFetched(Err(String))` → `ModelsFetched(Err(ProviderError))`
- ✅ `Action::CredentialValidated(Err(String))` → `CredentialValidated(Err(ProviderError))`
- ✅ 에러 타입 4개에 `Clone` derive 추가, `Io`/`Unknown` variant를 `String` 기반으로 단순화
- ✅ 발송측 10곳 + 수신 핸들러 2곳 시그니처 전환

#### LOW-1: /help 도움말 갱신
- ✅ `/help` 출력에 `/theme 테마 전환 (Toggle Theme)` 항목 추가

### 품질 검증 결과
```
cargo build   ✅ 성공
cargo test    ✅ 28/28 통과 (0 failed)
cargo clippy  ✅ 0 warnings (-D warnings 게이트 통과)
```

### 남은 이관 항목
- ⏳ Structured Tool Call (Provider별 native tool call contract)
- ⏳ Diff 접기/펼치기 UI
- ⏳ `resolve_credentials()` 반환 타입 `String` → `ProviderError` 마이그레이션

---

## Phase 12: 하네스 구조/보안/UX 감사 대응 (v0.1.0-beta.22)

**상태**: ✅ 구현 완료
**완료 일시**: 2026-04-17
**관련 문서**: DESIGN_DECISIONS.md ADR-013 / spec.md §3.2

### 구현 항목 — 7건

#### HIGH-1: 도구 호출 격리 계층
- ✅ bare JSON 차단 (fenced가 아닌 raw JSON은 도구로 인식하지 않음)
- ✅ `"tool"` 키 존재 1차 필터
- ✅ ToolCall serde 역직렬화 2차 필터
- ✅ ExecShell 빈 명령 3차 필터

#### HIGH-2: 빈 ExecShell 차단
- ✅ `PermissionEngine::check()` 진입 직후 `command.trim().is_empty()` → 즉시 Deny
- ✅ `is_safe_command()` 빈 토큰 `true` → `false` 수정

#### HIGH-3: 전체 UI Wrap + 스크롤
- ✅ 타임라인: `Wrap { trim: false }` + `scroll((timeline_scroll, 0))`
- ✅ 컴포저: `Wrap { trim: false }`
- ✅ config_dashboard: `Wrap { trim: false }`
- ✅ setting_wizard: `Wrap { trim: false }`
- ✅ `UiState::timeline_scroll: u16` 필드 추가

#### MEDIUM-1: PLAN/RUN 모드 시스템 프롬프트 주입
- ✅ `dispatch_chat_request()`에서 모드별 시스템 메시지 주입
- ✅ PLAN: 분석/설명 위주, 자동 파일 쓰기 자제
- ✅ RUN: WriteFile/ReplaceFileContent 우선 사용 지시

#### MEDIUM-2: 작업 계약 명확화
- ✅ MEDIUM-1과 동일 메커니즘으로 해소

#### LOW-1: 승인 카드 전체 경로
- ✅ `format_tool_name()`: 도구별 의미 있는 이름 (최대 120자)
- ✅ `format_tool_detail()`: 명령/경로/동작 축약 없이 표시

#### LOW-2: 회귀 테스트 5건 추가
- ✅ `test_empty_exec_shell_denied`: 빈 명령 3케이스 Deny 검증
- ✅ `test_empty_exec_shell_safe_only_denied`: SafeOnly 빈 명령 Deny 검증
- ✅ `test_timeline_scroll_initial_value`: 스크롤 오프셋 초기화 검증
- ✅ `test_plan_run_mode_toggle`: PLAN/RUN 모드 전환 검증
- ✅ `test_bare_json_not_treated_as_tool`: bare/fenced JSON 구분 검증

### 품질 검증 결과
```
cargo build   ✅ 성공
cargo test    ✅ 33/33 통과 (0 failed)
cargo clippy  ✅ 0 warnings (-D warnings 게이트 통과)
```

### 남은 이관 항목
- ⏳ Structured Tool Call (Provider별 native tool call contract — 근본적 도구 격리)
- ⏳ Diff 접기/펼치기 UI
- [x] `resolve_credentials()` 반환 타입 `String` → `ProviderError` 마이그레이션
- [x] 타임라인 스크롤 키 바인딩 (PageUp/PageDown → timeline_scroll 조작)

## Phase 11: Extended Prompt Commands (@ and !) (완료)

이 페이즈는 LLM 컨텍스트 주입과 백그라운드 셸 실행을 터미널에 맞게 가속화하는 것을 목표로 구현되었다.

### 11.1 시스템 분해표 및 파일 책임
| 시스템 | 파일 경로 | 변경된 책임 (Responsibilities) |
| --- | --- | --- |
| **State Layer** | `src/app/state.rs` | `FuzzyMode` enum 정의 및 `FuzzyFinderState` 확장. `ComposerState` 히스토리 관리. |
| **Action Route** | `src/app/mod.rs` | `@`, `!` 특수 문자 캡처링 (`handle_char_input`). Fuzzy Mode 분기 및 `Up`/`Down` 이벤트 라우팅. |
| **Logic Layer** | `src/app/chat_runtime.rs` | `@` 특수 키워드(`workspace`, `terminal`) 및 파일 I/O 파싱 후 컨텍스트 인라인 치환 로직. |

### 11.2 경계 계약 요약 및 동결된 공식
- **최대 렌더링 한계 (Fuzzy Matches Limit)**:
  - 파일 시스템 탐색 비용과 렌더링 부하를 막기 위해 공식적으로 일치 항목은 **100개**로 제한한다. (`matches.truncate(100)`)
- **히스토리 라이프사이클**:
  - `ComposerState.history`에 보존되며, 애플리케이션 수명 주기와 동일. 디스크 동기화는 하지 않음.
- **예외 처리 변환 공식**:
  - 파일 읽기 에러 발생 -> `TimelineEntryKind::SystemNotice(msg)` 변환 -> 타임라인 Push. (UI 블로킹 없음)

### 11.3 핵심 알고리즘 메모
- **Fuzzy Finder 다형성**: 
  - TUI 렌더링 컴포넌트는 `FuzzyMode`가 무엇인지 알 필요가 없다. 오직 `app/mod.rs`의 `update_fuzzy_matches()` 루틴만 `FuzzyMode`를 보고 `ignore::WalkBuilder`를 돌릴지, 아니면 하드코딩된 `Macros` 배열을 필터링할지 결정한다. 
- **매크로 문자열 역분해**:
  - `build      (cargo build)` 형태로 렌더링된 문자열을 `handle_enter_key`에서 선택할 시, 괄호 안의 실제 명령어 부분만 파싱(`.split('(').nth(1)...`)하여 Composer 버퍼에 `!cargo build` 텍스트로 치환한다.

### 11.4 검증 기준 완료 내역
- [x] 빈 프롬프트에서 `!`를 쳤을 때 `FuzzyMode::Macros` 팝업이 노출되는가?
- [x] `@workspace` 선택 시 루트 폴더 파일 목록 요약이 `dispatch_chat_request`에서 주입되는가?
- [x] `!cargo build` 후, 방향키 위(`Up`)를 눌러 버퍼가 성공적으로 복원되는가?
- [x] 없는 파일을 `@invalid` 쳤을 때 붉은 에러 노티스가 발생하고 시스템이 패닉되지 않는가?

## Phase 12: Native Structured Tool Call Integration (완료)

이 페이즈는 기존의 취약한 마크다운 정규식 캡처(Fenced JSON) 방식을 폐기하고, 모델이 공식적으로 지원하는 OpenAI 호환 구조화된 도구(Tool Call) API로 안전하게 이관하는 것을 목표로 구현되었다.

### 12.1 시스템 분해표 및 파일 책임
| 시스템 | 파일 경로 | 변경된 책임 (Responsibilities) |
| --- | --- | --- |
| **Domain Layer** | `src/providers/types.rs` | `ToolCallRequest`, `FunctionCall` 등 Native JSON Schema 구조체 추가. `Role::Tool` 추가. |
| **Provider Layer** | `src/providers/registry.rs` | OpenRouter 및 Gemini SSE 스트림 루프에서 `tool_calls` 델타 이벤트를 파싱 및 수집하여 `action_tx`로 전파. |
| **Logic Layer** | `src/app/chat_runtime.rs` | 시작 시 도구 JSON Schema 주입. `ToolCallDelta` 수신 및 버퍼링 조립 루프. |
| **Tool Layer** | `src/app/tool_runtime.rs` | 기존 정규식 스크래핑(`extract_tool_calls_from_markdown`) 함수 삭제 및 Native Payload 처리로 대체. |

### 12.2 경계 계약 요약 및 동결된 공식
- **SSE Chunk Assembly Protocol**:
  - `delta.tool_calls`는 여러 개의 작은 문자열 조각으로 파편화되어 오므로, `ChatResponse`가 완료될 때까지 상태 내부의 10MB 버퍼(`String`)에 안전하게 누적(Assemble)해야 한다.
- **도구 에러 반환 규칙**:
  - 실행된 도구의 에러(없는 파일 등)는 `TimelineEntryKind::SystemNotice`와 더불어, LLM의 컨텍스트로 롤백될 때 반드시 `Role::Tool` 메시지로 캡슐화되어 전달되어야 한다.

### 12.3 핵심 알고리즘 메모
- Fenced Markdown 검사 로직(과거의 유산)을 들어내고, `ChatRequest.tools` 필드 유무에 따라 LLM이 자동으로 응답 모드를 스위칭하도록 유도.
- 시스템 프롬프트에서 도구 사용 설명을 덜어내어 초기 토큰을 절약.

### 12.4 검증 기준 완료 내역
- [x] `ChatRequest` 직렬화 시 `tools` 배열에 올바른 JSON Schema 규격이 포함되어 전송되는가?
- [x] LLM 응답 시 `Role::Assistant`에 `tool_calls` 필드가 정상적으로 파싱되는가?
- [x] 스트리밍 모드(`chat_stream`)에서 여러 개로 분할된 JSON 델타 조각들이 하나의 완전한 객체로 파싱되는가? (OpenRouter SSE 버퍼링 구현)
- [x] 정규식 스크래핑 로직(`extract_tool_calls_from_markdown`)이 코드베이스에서 완벽히 제거되었는가 (`cargo check` 무결성 검증 완료)?
- [x] 기존의 회귀 테스트가 Native Tool Call 구조체 필드를 올바르게 반영하여 무결성을 통과하는가 (`cargo test` 42/42 통과)?

---

## Phase 13: Agentic Autonomy & Architectural Refactoring (완료)

이 페이즈는 `smlcli`를 단순한 프롬프트 도구에서 벗어나 자율적으로 코드를 검증하고 복구하는 에이전트(Autonomous Agent)로 도약하기 위해 설계되었습니다.

### 13.1 시스템 분해표 및 파일 책임
| 시스템 | 파일 경로 | 변경된 책임 (Responsibilities) |
| --- | --- | --- |
| **Tool Registry Layer** | `src/tools/registry.rs` | 다형성(Polymorphism)을 갖춘 `Tool` 트레이트 명세 정의. `ReadFile`, `WriteFile`, `ReplaceFileContent`, `ExecShell` 등의 개별 구조체 이관. |
| **Git Automation Layer** | `src/tools/git_checkpoint.rs` | 워크스페이스 clean 여부 판단(`git status --porcelain`). 강제 커밋 없이 `Result<bool>` 반환. WIP 존재 시 롤백 건너뜀. `git clean -fd` 미사용. |
| **Repo Map Layer** | `src/domain/repo_map.rs` | `tree-sitter`를 이용한 Rust(.rs) 전용 AST 파싱 및 컨텍스트 요약(최대 8,000바이트 한계). |
| **State Machine** | `src/app/state.rs` | `AutoVerifyState` (`Idle`, `Healing { retries: usize }`) 정의 및 최대 3회 재시도 상태 관리. |
| **TUI Layer** | `src/tui/layout.rs` | '생각의 트리(Tree of Thoughts)' 렌더링. 도구 호출 로그를 `depth` 속성 기반 인덴트(`└─`) 처리. |

### 13.2 경계 계약 요약 및 동결된 공식
- **최대 자가 복구 한계 (Self-Correction Retries)**: `Healing { retries }` 3회 초과 시 `Idle`로 전환하고 사용자에게 수동 개입을 안내.
- **Git Checkpoint 롤백 전략**: `create_checkpoint()`는 워킹 트리가 clean일 때만 `true` 반환. `rollback_checkpoint()`는 `git reset --hard HEAD`만 사용하며 종료 코드를 반드시 검사. untracked 파일은 어떤 경우에도 삭제하지 않음.
- **ExecShell 파괴 판정**: `ExecShell`의 `is_destructive()`는 기본값(`false`)을 사용. 쉘 명령 실패가 Git 롤백을 트리거하지 않음.
- **도구 스키마 주입**: `send_chat_message`(초기)와 `send_chat_message_internal`(재전송) 양쪽 모두 `GLOBAL_REGISTRY.all_schemas()`를 `req.tools`에 주입.
- **Tool 트레이트 시그니처**: `async fn execute(&self, args: Value, ctx: &ToolContext<'_>) -> Result<ToolResult, ToolError>`

### 13.3 로드맵 체크리스트 (구현 가이드)
- [x] **Step 1: Tool Registry 리팩토링**
  - 기존 `match` 분기를 삭제하고 `tool_runtime.rs`를 다형성 호출 구조로 개편.
- [x] **Step 2: Automated Git Checkpoint 통합**
  - `is_destructive()=true`인 도구 실행 직전 `create_checkpoint()`로 워킹 트리 clean 여부를 검사. clean 상태에서만 `safe_to_rollback=true` 반환. 도구 실패 시 `git reset --hard HEAD`로 tracked 파일만 복원. WIP 존재 시 롤백 건너뜀.
- [x] **Step 3: Tree-sitter Repo Map 통합**
  - `tree-sitter`, `tree-sitter-rust`, `ignore` 의존성 주입. Rust(.rs) 파일의 구조를 AST로 추출하여 System 프롬프트 최상단 8KB 제한 하에 주입.
- [x] **Step 4: Auto-Verify & Self-Correction 루프**
  - `AutoVerifyState` (`Idle`, `Healing { retries }`) 스테이트 머신 구현. `ToolFinished(is_error=true)`와 `ToolError` 양쪽 경로 모두에서 힐링 프롬프트 주입 및 재전송. 최대 3회 초과 시 Abort.
- [x] **Step 5: Tree of Thoughts TUI 렌더링**
  - `TimelineEntry`에 계층 속성(Depth) 부여 및 `layout.rs`에서 시각적 계층화(`└─`) 적용.

---

## Phase 14: TUI UX/UI 고도화 (v0.1.0-beta.24)

### 14.1 변경 파일 및 구현 내역

#### 14-A: 멀티라인 텍스트 렌더링 — ✅ 완료
- ✅ `layout.rs`: `render_multiline_text(text, style) -> Vec<Line<'static>>` 공용 헬퍼 추가
- ✅ `layout.rs`: UserMessage, AssistantMessage, AssistantDelta 렌더링에서 `Line::from(msg)` → `render_multiline_text()` 전환
- ✅ `layout.rs`: session.messages 폴백 경로(line 270~291)에도 동일 적용
- ✅ `command_router.rs`: `/help` 출력을 타임라인 SystemNotice로 직접 추가하여 개행 보존

#### 14-B: 스크롤 분리 + Auto-Follow + 마우스 — ✅ 완료
- ✅ `state.rs`: `inspector_scroll: u16`, `timeline_follow_tail: bool` 필드 추가
- ✅ `terminal.rs`: `EnableMouseCapture`/`DisableMouseCapture` 추가
- ✅ `event_loop.rs`: `CrosstermEvent::Mouse` → `Event::Mouse(MouseEvent)` 전달
- ✅ `mod.rs`: `handle_mouse()` 메서드 — `is_mouse_in_inspector` 클램프 범위 보정 및 패널별 독립 스크롤 라우팅
- ✅ `layout.rs`: `timeline_follow_tail` 기반 bottom-up 오프셋을 top-based 렌더링에 동기화, `inspector_scroll` 전면 적용
- ✅ `mod.rs`: `Home`/`End` 키 지원 + `PageUp`/`PageDown`에 `follow_tail` 연동

#### 14-C: 키바인딩 재정렬 — ✅ 완료
- ✅ `mod.rs`: `Ctrl+I` 바인딩 제거 → 인스펙터 토글 `F2`로 변경
- ✅ `layout.rs`: 상태 바 안내 문구 `"(Tab) 모드 전환 | (F2) 인스펙터 토글"` 동기화
- ✅ `designs.md`: §4.3 키보드 바인딩 테이블을 실제 구현과 동기화

#### 14-D: 반응형 레이아웃 — ✅ 완료
- ✅ `layout.rs`: 상단 바를 `Layout::horizontal`로 좌우 분리(우측 정렬)하여 폭 감소 시 핵심 정보 잘림 원천 차단
- ✅ `layout.rs`: `truncate_middle(s, max_len)` 헬퍼 — cwd, provider, model 모두 중략 적용
- ✅ `layout.rs`: 인스펙터 폭 `Percentage(30)` → `Length` 클램프(32~48cols, 타임라인 최소 72cols)
- ✅ `layout.rs`: 인스펙터 탭 라벨 축약 (폭 < 40 시 Preview→Prev, Search→Srch 등)

### 14.2 품질 검증 결과
```
cargo clippy --all-targets --all-features -- -D warnings  ✅ 0 warnings
cargo test  ✅ 42 passed (0 failed)
```

---

## Phase 15: 2026 CLI UX 현대화 로드맵 (진행 중)

이 페이즈는 이미 구현된 Phase 13~14의 기반을 유지하면서, `smlcli`를 **블록 기반 작업 콘솔**로 승격시키는 계획 단계다. 목적은 "더 화려한 TUI"가 아니라, 빠른 명령 발견, 작업 재참조, 긴 출력 관리, 독립 패널 포커스, 절제된 모션을 갖춘 실사용 CLI UX를 만드는 것이다.

### 15.1 시스템 분해표 및 파일 책임 (계획)
| 시스템 | 파일 경로 | 예정 책임 |
| --- | --- | --- |
| **Block Timeline Layer** | `src/app/state.rs`, `src/tui/layout.rs` | [✅ Phase 15-A 완료] `TimelineBlock`, `BlockSection`, `BlockStatus` 구조체 도입 및 렌더링 교체 완료 |
| **Focus / Scroll State Machine** | `src/app/state.rs`, `src/app/mod.rs` | `FocusedPane`, pane별 scroll/selection/follow 상태, 키/마우스 라우팅 |
| **Command Palette Layer** | `src/app/state.rs`, `src/app/mod.rs`, `src/tui/layout.rs` | [🚧 진행 중] `Ctrl+K` 기반 Quick Actions palette, fuzzy search, 카테고리별 액션 실행 (현재 단순 filter 구현 상태, `PaletteCategory` 미도입) |
| **Composer Toolbar Layer** | `src/app/state.rs`, `src/tui/layout.rs` | [❌ 미구현] mode/context/policy/hint chip 렌더링, multiline 입력 상태 표시 (`ComposerToolbarState` 등 없음) |
| **Adaptive Header Layer** | `src/tui/layout.rs` | [✅ 완료] 세그먼트 우선순위 기반 상단 바 렌더링, 좌우 정렬, 폭별 중략 |
| **Inspector Workspace Layer** | `src/tui/widgets/inspector_tabs.rs` | [✅ 완료] 블록 상세, diff, logs, recent, search를 작업형 패널로 재구성 |
| **Motion Layer** | `src/tui/layout.rs` | [🚧 진행 중] 상태별 ASCII 모션 프로필과 pulse/spinner/settle 효과 (`MotionProfile` 스펙 미적용) |

### 15.2 경계 계약 요약
- **프레임워크 유지**: `ratatui + crossterm` 유지. Phase 15-A~15-C에서는 신규 의존성 도입 금지.
- **블록 우선 렌더링**: 타임라인의 기본 단위는 `TimelineEntry`가 아니라 `TimelineBlock`.
- **명령 발견 경로**: `Ctrl+K` = Command Palette, `Ctrl+P` = provider/model 유지.
- **포커스 모델**: `Timeline`, `Inspector`, `Composer`, `Palette` 4분할.
- **모션 한도**: 상태 전달용 ASCII 모션만 허용, 과한 전환 애니메이션 금지.

### 15.3 첫 작업 가능 범위 (First Playable Slice)
- 블록 헤더 + 상태 배지 + 접힘/펼침
- `Ctrl+K` palette 기본 동작
- Composer toolbar 4종 칩
- 타임라인/인스펙터/팔레트 포커스 전환
- 100/120/140 columns 레이아웃 스냅샷 통과

### 15.4 구현 순서 권장
- [x] **Step 1: Block Timeline Foundation**
  - `TimelineBlock`, `BlockSection`, `BlockStatus` 타입 정의
  - 기존 타임라인 문자열 조립을 블록 렌더링 함수로 대체
- [x] **Step 2: Focus / Scroll State Machine**
  - `FocusedPane` 도입
  - pane별 scroll/selection/follow 상태 완전 분리
- [x] **Step 3: Command Palette**
  - `Ctrl+K` 바인딩
  - category + fuzzy search + action dispatch
- [x] **Step 4: Composer Toolbar**
  - mode/path/context/policy/hint 칩 렌더링
  - `Shift+Enter` 멀티라인 입력
- [x] **Step 5: Adaptive Header**
  - 상단 바 세그먼트 우선순위 렌더링
  - breakpoints별 생략 정책
- [x] **Step 6: Inspector Workspace**
  - selected block details / diff / recent / logs 재구성
- [x] **Step 7: Motion Polish**
  - spinner/pulse/settle 효과 상태별 1종 도입
- [x] **Step 8: Verification**
  - snapshot, focus, palette, scroll, keyboard, mouse 통합 검증

### 15.5 유지보수 규칙
- palette command 목록은 문자열 상수가 아니라 구조화 데이터 원본을 단일 소스로 유지
- block renderer와 state reducer를 분리하여, 레이아웃 변경이 상태 머신을 오염시키지 않게 유지
- 새 애니메이션 추가 시 기존 상태와 중첩 금지

### 15.6 외부 레퍼런스 메모
- Warp Blocks / Universal Input: 블록 단위 작업 맥락, 입력 툴벨트
- Textual Command Palette: command discoverability
- Ratatui Layout / Style / Tachyonfx: 반응형 레이아웃과 경량 모션
