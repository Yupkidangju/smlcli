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

- [2026-04-21] : **[Implemented - Phase 25 Ultimate Polish & Security Hardening]**
  - ✅ **UTF-8 안전성 보장 (UX/UI)**: TUI 렌더링 시 `unicode-width` 크레이트를 적용하여 한국어/이모지 멀티바이트 문자가 깨지거나 패닉이 발생하는 현상 수정.
  - ✅ **심볼릭 링크 샌드박스 탈옥 방지 (Security)**: `file_ops`에서 `std::fs::canonicalize`를 통해 파일 절대 경로를 확인하여 Workspace 외부 경로 접근(Path Traversal/Symlink) 차단.
  - ✅ **네트워크 타임아웃 및 재시도 (Robustness)**: LLM API 호출(429/5xx) 및 행(Hanging) 현상 방어를 위해 지수 백오프 기반의 재시도(Retry)와 60초 타임아웃 적용.
  - ✅ **ENOSPC 디스크 에러 그레이스풀 폴백 (Reliability)**: 로그(`session_log`) 및 설정(`config_store`) 기록 시 디스크 용량 부족(`StorageFull`) 오류를 캡처하여 패닉 방지 및 보존 정책 적용.
  - ✅ **터미널 프로세스 잔상 제거 (UX)**: `ExecShell`을 통한 서브 프로세스 실행 후 TUI 복귀 시, 화면 잔상(Ghosting) 방지를 위한 `terminal.clear()` 및 커서 재설정 로직 반영.

- [2026-04-21] : **[Implemented - Windows Host Shell / Workspace Trust Gate]** 계획된 Workspace Trust Gate 및 호스트 셸 정렬 구현 완료.
  - ✅ **Task 1: Workspace root 결정 유틸리티 통합**: `.git` 또는 `Cargo.toml` 상향 탐색이 성공하면 root를 확정하고, 실패 시 현재 디렉터리를 사용하도록 통합.
  - ✅ **Task 2: Trust/Workspace 정책 모델 + 설정 영속화**: `Unknown/Trusted/Restricted` 상태와 `workspace_trust`, `denied_roots` 필드를 `PersistedSettings`에 추가하여 `config.toml`에 영속화.
  - ✅ **Task 3: Startup Trust Gate UI**: 시작 시 `Unknown` 상태인 경우 `Trust Once / Trust & Remember / Restricted` 선택을 강제하는 Popup UI를 렌더링하고 입력을 라우팅.
  - ✅ **Task 4: Permission Engine 연동**: `PermissionEngine::check()`에서 `WriteFile`, `ReplaceFileContent`, `ExecShell` 도구에 대해 `Restricted` 또는 `Unknown` 상태 시 강력한 차단(Deny) 로직 적용.
  - ✅ **Task 5: Windows exec shell 정렬**: `ExecShell` 도구에서 빈 명령어 차단 및 호스트 셸 정렬. (Linux는 bash 기반, Windows는 PowerShell 기반으로 처리되도록 구조화)
  - ✅ **Task 6: REPL/설정 관리 surface 추가**: `/workspace show`, `/workspace trust`, `/workspace deny`, `/workspace clear` 커맨드를 추가하여 터미널 내에서 신뢰 상태를 동적으로 관리할 수 있도록 연동.
  - ✅ **Task 7: 상태바/진단 출력 반영**: `/status` 명령어에 현재 Workspace Trust 상태 및 Denied 여부 표기 반영.

- [2026-04-20] : **[Scroll UX Remediation]** 타임라인/인스펙터 스크롤 동작을 일반적인 CLI 기대치에 맞게 보정.
  - 마우스 휠이 구식 `timeline_scroll_offset` 대신 실제 렌더 필드 `timeline_scroll`을 조작하도록 수정.
  - `follow_tail`이 최하단 복귀 시 다시 활성화되어 새 콘텐츠가 자동으로 보이도록 보정.
  - 마우스 클릭 포커스를 row+column 기준으로 재판정하여 Composer 클릭 시 Timeline이 잘못 선택되지 않도록 수정.
  - 선택된 타임라인 블록 전체 반전을 제거하고 첫 줄만 약하게 강조하도록 변경.
- [2026-04-20] : **[Workspace/Inspector UX Fix]** 실행 위치와 Inspector 보조 UI를 정리.
  - `target/release` 같은 빌드 산출물 디렉터리에서 실행되면 저장소 루트로 작업 디렉터리를 자동 보정하여 `ReadFile`/`Stat`/`ListDir`가 올바른 프로젝트 파일을 대상으로 동작하도록 수정.
  - Composer Toolbar에 `F2 Inspector` 힌트를 복원.
  - Inspector 상단 탭 영역을 적응형 1줄/2줄 헤더로 바꿔 좁은 폭에서도 탭명이 잘리지 않도록 수정.

- [2026-04-20] : **[2nd Audit Remediation - Context / Config / Edge Tests]** 2차 감사의 심화 이슈를 재검토하고 실제 결함만 수정.
  - Auto-Verify가 240자 요약만 다시 보내던 문제를 수정하여 `stderr` 우선, `stdout` 보조의 확장 실패 컨텍스트를 모델 재전송에 사용하도록 개선.
  - `load_config()` 오류를 `DomainState::new_async()`에서 삼키지 않도록 수정하고, 손상된 `config.toml`은 Setup Wizard 첫 화면과 로그 버퍼에 복구/삭제 가이드로 표시.
  - `logs_buffer`는 별도 락 추가 대신 Event Loop 직렬화 계약을 문서화. 비동기 태스크는 직접 공유 버퍼를 만지지 않고 `Event::Action`으로만 전달됨을 명시.
  - 실패 경로 테스트 확충: 손상된 TOML, 시작 시 설정 오류, Auto-Verify tail context, 비-Git checkpoint no-op.
- [2026-04-20] : **[3rd Audit Remediation - Sandbox / Repo Map / HITL TTL]** 3차 감사 항목을 재검증 후 실제 미구현 항목만 보강.
  - `Sliding Window`는 이미 `/compact` 드라이버와 자동 compact tick 경로가 연결된 상태임을 재확인하고 미수정.
  - Linux `ExecShell`을 `bwrap` 기반 실제 샌드박스로 전환하여 호스트 루트는 읽기 전용, 요청 `cwd`만 `/workspace`로 쓰기 가능하도록 제한.
  - `Repo Map`은 비동기 `spawn_blocking` worker와 캐시 상태(`cached/is_loading/stale`)를 도입하고, 준비된 캐시를 실제 채팅 요청 system message에 주입하도록 연결.
  - `Approval` 요청에 5분 TTL을 도입하여 응답이 없으면 자동 거부/정리/시스템 알림이 수행되도록 수정.
  - 회귀 테스트 추가: Linux 샌드박스 쓰기 차단/허용, Repo Map cache lifecycle, 요청 주입, 승인 시간 초과.

- [2026-04-20] : **[Audit Remediation - Auto-Verify / Tree Depth / Guardrail]** 문서-구현 불일치 4건 중 실제 결함을 교정.
  - `AutoVerifyState`가 정의만 되고 사용되지 않던 문제를 수정하여 `ToolFinished(is_error=true)`와 `ToolError` 모두에서 힐링 상태 전이 및 최대 3회 제한을 적용.
  - `send_chat_message_internal()` 후속 재전송 경로에 Tool Schema 주입을 공통화하여, 자가 복구 턴에서도 모델이 후속 도구 호출을 지속할 수 있게 함.
  - `TimelineBlock.depth`와 `tui/layout.rs` 인덴트 렌더링을 연결하여 `ToolRun`/`Approval`/`Auto-Verify Notice`가 실제로 `└─` 계층 구조로 표시되도록 수정.
  - `is_actionable_input()` 기반 선제 차단을 제거하고, 비작업성 입력에서도 모델이 구조화된 `tool_calls`를 반환하면 Permission Engine 아래에서 계속 처리되도록 완화.
  - 회귀 테스트 추가: Auto-Verify 재시도 상한, depth 생성, tool schema 주입, 완화된 가드레일 동작 검증.

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
cargo test  ✅ 46 passed (0 failed)
```

---

## Phase 15: 2026 CLI UX 현대화 로드맵 (완료)

이 페이즈는 이미 구현된 Phase 13~14의 기반을 유지하면서, `smlcli`를 **블록 기반 작업 콘솔**로 승격시키는 계획 단계다. 목적은 "더 화려한 TUI"가 아니라, 빠른 명령 발견, 작업 재참조, 긴 출력 관리, 독립 패널 포커스, 절제된 모션을 갖춘 실사용 CLI UX를 만드는 것이다.

### 15.1 시스템 분해표 및 파일 책임 (계획)
| 시스템 | 파일 경로 | 예정 책임 |
| --- | --- | --- |
| **Block Timeline Layer** | `src/app/state.rs`, `src/tui/layout.rs` | [✅ Phase 15-A 완료] `TimelineBlock`, `BlockSection`, `BlockStatus` 구조체 도입 및 렌더링 교체 완료 |
| **Focus / Scroll State Machine** | `src/app/state.rs`, `src/app/mod.rs` | `FocusedPane`, pane별 scroll/selection/follow 상태, 키/마우스 라우팅 |
| **Command Palette Layer** | `src/app/state.rs`, `src/app/mod.rs`, `src/tui/layout.rs` | [✅ Phase 15-C 완료] `Ctrl+K` 기반 Quick Actions palette, 카테고리별 상태 연동(`PaletteCategory` 도입) |
| **Composer Toolbar Layer** | `src/app/state.rs`, `src/tui/layout.rs` | [✅ Phase 15-D 완료] `ComposerToolbarState` 연동, mode/path/policy/hint chip 동적 렌더링 및 multiline 표시 |
| **Adaptive Header Layer** | `src/tui/layout.rs` | [✅ 완료] 세그먼트 우선순위 기반 상단 바 렌더링, 좌우 정렬, 폭별 중략 |
| **Inspector Workspace Layer** | `src/tui/widgets/inspector_tabs.rs` | [✅ 완료] 블록 상세, diff, logs, recent, search를 작업형 패널로 재구성 |
| **Motion Layer** | `src/tui/layout.rs` | [✅ Phase 15-F 완료] `MotionProfile` 도입, `Running` 스피너 렌더링 및 `NeedsApproval` 펄스 적용 완료 |

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

---

## Phase 16: Deep UI Interactivity & Provider Hardening (v0.1.0-beta.26)

이 페이즈는 Phase 15의 블록 기반 TUI 위에 접기/펼치기(Fold/Unfold) 상호작용과 도메인 에러(`ProviderError`) 일원화를 추가하는 것을 목표로 구현된다.

### 16.1 시스템 분해표 및 파일 책임
| 시스템 | 파일 경로 | 예정 책임 |
| --- | --- | --- |
| **Block State Layer** | `src/app/state.rs` | `BlockDisplayMode` enum 추가. `TimelineBlock`에 `toggle_collapse()` 헬퍼 구현. |
| **Input Routing Layer** | `src/app/mod.rs` | 타임라인 포커스 상태에서 `Enter` 입력 시 선택된 블록의 상태를 토글하는 로직 추가. |
| **Timeline Renderer Layer** | `src/tui/layout.rs` | `display_mode == Collapsed`일 때 Diff를 그리지 않고 1줄짜리 요약 스팬(`[ +N / -M ]`)만 그리도록 분기. |

### 16.2 경계 계약 요약 (Typed Contracts & Formulas)
- **Diff 접기/펼치기 공식**:
  - `additions`: Diff 중 `+`로 시작하는 라인 수 (단, `+++`는 제외)
  - `deletions`: Diff 중 `-`로 시작하는 라인 수 (단, `---`는 제외)
  - **Threshold**: `additions + deletions > 10`
  - 초과 시 `display_mode = BlockDisplayMode::Collapsed` 기본 할당.
- **렌더링 데이터(Real Data)**: `[ +14 lines / -3 lines ] (Enter 키로 펼치기)`

### 16.3 구현 순서 권장 (Execution Path)
- [x] **Step 1: State Extension** - `BlockDisplayMode` Enum 및 `TimelineBlock.display_mode/diff_summary` 필드, 토글 메서드 추가 완료 (`src/app/state.rs`).
- [x] **Step 2: Diff Summary Calculation** - `ReplaceFileContent` 블록 추가 시 `additions`와 `deletions`를 카운트하여 10줄을 초과하면 `Collapsed`로 설정 (`src/app/mod.rs` 및 `src/tools/file_ops.rs`).
- [x] **Step 3: Render Logic 분기** - `src/tui/layout.rs`의 `TimelineBlockKind::ToolRun` 렌더러에서 `display_mode`에 따라 요약 라벨만 렌더링하거나 원본 스팬을 렌더링하도록 분기 완료.
- [x] **Step 4: Input Binding** - `src/app/mod.rs`에서 `Enter` 입력 시 현재 선택된 타임라인 블록의 상태 토글 완료.

### 16.4 Task 2: 에러 시그니처 완전 구조화 및 전파 (Error Types Unification)
- [x] **Step 1: ConfigError Unification** - `src/infra/config_store.rs`의 `anyhow::Result` 반환을 `Result<T, ConfigError>`로 변경하고 에러 종류별 세분화.
- [x] **Step 2: ProviderError Unification** - `src/providers/registry.rs`의 `ProviderAdapter` 트레잇과 어댑터 내부의 `anyhow::Result`를 `Result<T, ProviderError>`로 전환하고 `NetworkFailure`, `ApiResponse`, `AuthenticationFailed` 등으로 구조화.

### 16.5 Task 3: Provider-Specific Native Tool Call 세분화 (Dialect 추상화)
- [x] **Step 1: ToolDialect Enum** - `src/domain/provider.rs`에 `ToolDialect` Enum(`OpenAICompat`, `Anthropic`, `Gemini`) 추가.
- [x] **Step 2: Schema Processing** - `src/tools/registry.rs`의 `all_schemas` 메서드에서 `ToolDialect`를 주입받아, Gemini의 경우 `parameters.required` 배열이 없으면 명시적으로 `[]`를 삽입하도록 예외 처리.
- [x] **Step 3: Chat Runtime 연동** - `src/app/chat_runtime.rs`의 `build_streaming_chat_request`에서 `ProviderKind`를 통해 알맞은 방언을 추론하여 Schema를 제공.

## Phase 17: Windows Shell Host Alignment & Workspace Trust Gate
이 페이즈는 Windows 환경에서의 셸 런타임 분리(Host vs Exec) 및 작업 디렉터리에 대한 명시적 신뢰(Trust) 상태를 관리하는 보안 게이트를 구축한다. 신뢰되지 않은 환경에서는 파일 쓰기 및 파괴적 셸 실행이 엄격하게 차단된다.

### 17.1 시스템 분해표 및 파일 책임
| 시스템 | 파일 경로 | 예정 책임 |
| --- | --- | --- |
| **Trust Model & Storage** | `src/domain/settings.rs`<br>`src/domain/workspace.rs` | `WorkspaceTrustState`, `WorkspaceTrustRecord` 정의 및 `PersistedSettings` 내 신뢰 디렉터리(`trusted_workspaces`, `denied_roots`) 필드 추가. |
| **Trust Gate UI** | `src/app/wizard_controller.rs`<br>`src/tui/widgets/trust_gate.rs` | 시작 시 현재 작업 디렉터리가 `Unknown` 상태일 경우, 3가지 선택지(Trust Once, Trust & Remember, Restricted)를 제공하는 프롬프트 렌더링. |
| **Permission Engine 연동** | `src/domain/permissions.rs` | 권한 검사 시 현재 `trust_state`를 조회하여 `Restricted` 또는 `Denied` 상태면 쓰기/실행 도구(`WriteFile`, `ExecShell`)의 권한을 차단. |
| **Shell Host 추론 및 실행** | `src/tools/shell.rs`<br>`src/app/state.rs` | Windows 환경에서 현재 호스트 셸과 무관하게 실행 셸(`exec_shell`)을 `pwsh` 또는 `powershell.exe`로 강제 지정 및 런타임 탐지 로직 추가. |
| **REPL 명령어 및 상태 표시** | `src/app/command_router.rs`<br>`src/tui/layout.rs` | `/workspace show/trust/deny/clear` 슬래시 명령어 구현 완료. `/workspace add/remove`는 **v3.0에서 구현 예정**. 상태바 및 `/status` 명령어 출력에 현재 Host/Exec 셸 정보 및 Trust 상태 노출. |

### 17.2 경계 계약 요약 (Typed Contracts)
- **신뢰 상태 모델 (WorkspaceTrustState)**:
  - `Unknown`: 아직 평가되지 않음 (프롬프트 노출 대상).
  - `Trusted`: 쓰기 및 셸 실행 허용 (기존 PermissionPolicy 적용).
  - `Restricted`: 읽기/탐색 전용 모드. `WriteFile`, `ReplaceFileContent`, `ExecShell` 도구 원천 차단.
- **Windows Exec Shell 결정 로직**:
  - `pwsh` (PowerShell Core) 확인 -> 존재 시 최우선 사용.
  - `powershell.exe` (Windows PowerShell) 확인 -> fallback.
  - 둘 다 실패 시 에러 반환 후 `ExecShell` 도구 실행 거부.

### 17.3 구현 순서 권장 (Execution Path)
- [x] **Task 1: Workspace root 결정 유틸리티** - `src/infra/workspace_utils.rs` (신설)에서 현재 디렉터리 기반 상향 탐색(`.git` 또는 `Cargo.toml` 기준)으로 루트 경로 도출.
- [x] **Task 2: Trust 정책 모델 영속화** - `PersistedSettings` 구조체 확장 및 로컬 스토리지(`config.toml`) 연동.
- [x] **Task 3: Trust Gate UI** - `Unknown` 상태일 때의 차단 화면/프롬프트 작성 및 상태 머신(Wizard) 전이 로직 작성.
- [x] **Task 4: Permission Engine 연동** - `Restricted` 상태일 경우 `PermissionResult::Denied` 반환과 함께 "Workspace is not trusted..." 안내문 출력.
- [x] **Task 5: Windows Exec Shell 강제화** - `ExecShell`의 Windows `cmd` 분기를 `pwsh` 탐색 로직으로 교체.
- [x] **Task 6: REPL 및 상태바 연동** - `/workspace` 서브 명령어 라우팅 추가 및 `layout.rs` 상단 상태바에 셸/Trust 상태 노출.

## Phase 18: Multi-Provider Expansion & Advanced Agentic Tools
이 페이즈는 2026년 4월 기준 최신 모델(GPT-5.4, Claude 4.7, Grok 4.20)들을 네이티브로 지원하고, 에이전트의 상황 인지 능력을 대폭 끌어올릴 수 있는 구조화된 시스템 도구(ListDir, GrepSearch, FetchURL)를 도입한다.

### 18.1 시스템 분해표 및 파일 책임
| 시스템 | 파일 경로 | 예정 책임 |
| --- | --- | --- |
| **Provider Registry & Adapters** | `src/domain/provider.rs`<br>`src/providers/openai.rs`<br>`src/providers/anthropic.rs`<br>`src/providers/xai.rs` | 각 Provider별 BaseURL, 모델명 리스트 관리 및 SDK 규격(또는 REST API) 연동. `ProviderAdapter` 트레이트 구현체 분리. |
| **Advanced Tools** | `src/tools/list_dir.rs`<br>`src/tools/grep_search.rs`<br>`src/tools/fetch_url.rs` | 신규 도구 구조체 생성 및 `Tool` 트레이트 구현. JSON 결과물 포매팅 최적화. |
| **Tool Registry 연동** | `src/tools/registry.rs` | 새로 만들어진 3개의 도구를 글로벌 레지스트리에 등록 및 각 도구의 `schema()` 정의. |
| **Setup Wizard 업데이트** | `src/tui/widgets/setting_wizard.rs` | 기존 OpenRouter 외에 신규 Provider들의 API 키를 입력받을 수 있는 Wizard 플로우 확장. |

### 18.2 경계 계약 요약 (Typed Contracts)
- **ToolDialect 확장**:
  ```rust
  pub enum ToolDialect {
      OpenAICompat,
      AnthropicNative,
      Gemini,
      XAI,
  }
  ```
- **신규 도구 입력 스키마**:
  - `ListDirectory`: `{ "path": "string" }`
  - `GrepSearch`: `{ "query": "string", "path": "string", "is_regex": "boolean" }`
  - `FetchURL`: `{ "url": "string" }`

### 18.3 구현 순서 권장 (Execution Path)
- [x] **Task 1: Provider 모델 및 Dialect 확장** - `provider.rs` 및 `registry.rs`에 신규 Provider 열거형, 2026.04 최신 모델 리스트(`gpt-5.4` 등) 추가.
- [x] **Task 2: API Adapter 구현** - `openai.rs`, `anthropic.rs`, `xai.rs` 생성 및 `ProviderAdapter` 구현.
- [x] **Task 3: Advanced Tools 구현** - `list_dir.rs`, `grep_search.rs`, `fetch_url.rs` 3종 도구 구현 로직 작성.
- [x] **Task 4: Registry 및 Wizard 연동** - 도구들을 시스템에 등록하고 초기 마법사 화면에서 API 키 입력 UI 업데이트.

---

## Phase 19: v1.0.0 Audit Remediation (완료)
**목표(Scope):** v1.0.0 출시 전 식별된 9가지 시스템 결함(상태 의존성, 데드락 위험, 권한 우회 등)을 해결하여 완전한 무상태 도구 실행과 이벤트 루프의 동시성 안정성을 확보합니다.
**비목표(Non-Scope):** 새로운 AI 모델 연동이나 TUI의 신규 레이아웃 추가 등은 이 단계에 포함되지 않습니다.

### 19.1 해결 대상 및 결함 (Concrete Analysis)
1. **[Logic] Wizard 설정 누락 및 상태 전이 결함**: 필수 필드 검증 누락으로 UI 마비.
2. **[Architecture] 도구 실행 상태 의존성 (Stateless 위반)**: `ToolRuntime`이 전역 상태 직접 수정.
3. **[Concurrency] 이벤트 루프 데드락 위험**: 취소 입력(`AppAction::Cancel`) 시 채널 블로킹.
4. **[Error] 에러 타입 파편화**: `infra/` 계층의 `Box<dyn Error>` 남용으로 복구 불가.
5. **[UX/UI] TUI 로그 렌더링 블로킹**: 로그 크기에 비례한 프레임 드랍.
6. **[Interaction] 위저드 탭 포커스 순환 오류**: 탭 순서 꼬임.
7. **[Sync] 상태바 갱신 지연**: 도구 종료 후 처리중 메시지 잔류.
8. **[Resource] 로그 파일 핸들 누수**: 매번 Open/Close 하여 핸들 고갈 위험.
9. **[Security] 권한 검증 와일드카드 우회 결함**: `rm -rf *` 등 위험 패턴 검열 누락.

### 19.2 경계 계약 요약 (Typed Contracts & Concrete Numbers)
- **TUI 렌더링 상한**: `MAX_LOG_LINES = 5000` (FIFO). Window size = `terminal_height - 4`.
- **에러 규격 (`SmlError`)**:
  ```rust
  pub enum SmlError {
      // ... 기존 에러 ...
      InfraError(String),
      IoError(std::io::Error),
  }
  impl From<std::io::Error> for SmlError { ... }
  ```
- **위저드 포커스 계약 (`WizardField`)**:
  ```rust
  pub enum WizardField { ApiKey, Provider, Model, SaveButton }
  ```
- **세션 로거 계약 (`SessionLogger`)**: `BufWriter<std::fs::File>` 필드를 소유하며 `Drop`에서 `flush()` 수행.
- **상태바 갱신 주기**: 기존 대비 틱(tick) 레이트 `100ms`로 강제 동기화.

### 19.3 구현 및 검증 경로 (Execution & Verification Path)
- [x] **Phase 1: Core Error & Config** (에러 통합 및 리소스 누수 방지)
  - `src/domain/error.rs` 리팩토링 및 `infra` 반환형 강제 변환. `SessionLogger` `BufWriter` 적용.
  - *검증*: 손상된 설정 파일 로드시 `SmlError::IoError`가 UI에 정상 전파되는지 확인. 도구 100회 실행 후 `lsof -p <PID>`로 파일 핸들 누수 확인.
- [x] **Phase 2: Logic & Security** (보안 강화 및 위저드 무결성)
  - `setting_wizard.rs` 필수값 검증 및 에러 시 현재 탭 유지. `glob` 기반 `is_dangerous()` 권한 검증 도입.
  - *검증*: `smlcli exec "rm -rf .git/*"` 실행 시 권한 거부 출력 확인.
- [x] **Phase 3: Runtime & Concurrency** (도구 무상태화 및 데드락 해소)
  - `ToolRuntime::execute()` 반환형 변경 및 상태 직접 변경 제거. `tokio_util::sync::CancellationToken` 도입.
  - `execute_tool` 시그니처에서 `App` 상태 참조를 제거하고 순수 비동기 이벤트 래핑 방식으로 분리하여 데드락 해소 완료.
- [x] **Phase 4: TUI & UX** (로그 렌더링 최적화 및 상호작용 개선)
  - `inspector_tabs.rs`에 윈도우 기반 렌더링 도입. 위저드 `[ApiKey, Provider, Model, SaveButton]` 포커스 강제.
  - Inspector 탭(Logs 등) 렌더링 시 버퍼의 전체를 포매팅하지 않고 `inspector_scroll` 기반으로 `area.height`만큼 슬라이스하여 처리함. (20,000줄 출력 프레임 드랍 완벽 해결)
  - Inspector의 다른 탭들에서 발생하던 스크롤 방향 역전 문제(Up 방향키 입력 시 스크롤이 내려가는 증상)를 `top_offset` 변환으로 수정.
  - 위저드 탭(Tab) 키 및 Shift+Tab 키 이벤트를 통한 `Provider <-> ApiKey <-> Model <-> Saving` 간 포커스 순환을 구현하여 단축 이동 편의성 제공.
  - *검증*: 20,000줄 출력 시 스크롤이 프레임 드랍 없이 부드럽게 동작하는지 확인. 위저드 내 탭 순환 확인.

- [x] **Phase 5: 2차 정밀 감사 및 잔여 결함 수정** (안정성 총점검)
  - **[Concurrency]**: `tokio::process::Command`에 `kill_on_drop(true)`를 설정하고, `execute_shell_streaming` 내 `select!` 블록에서 `cancel_token.cancelled()` 감지 시 `child.kill().await`를 명시적으로 호출하여 좀비 프로세스 발생 원천 차단.
  - **[State]**: `WizardController` 내 인증/검증 실패 시 `err_msg` 갱신과 함께 `api_key_input.clear()`를 수행하여 입력 버퍼 잔류를 초기화. 첫 문자 입력 또는 백스페이스 입력 시 에러 메시지 자동 초기화 로직 보완.
  - **[Performance]**: `InspectorTabs` 윈도우 기반 렌더링에서 `start_idx` 계산 시 `clamp(0, total_lines.saturating_sub(display_height))` 및 `total_lines == 0` 가드 조건을 도입하여 Out of Bounds Panic 방지. 수학적으로 정확한 bottom-up 스크롤 오프셋 변환 완수.
  - **[Security]**: `PermissionEngine::is_dangerous`에서 `regex` 모듈을 통한 `[;&|>]` 쉘 인젝션 체이닝 탐지 기능 도입. `ExecShell` 도구에 명시적 바이너리 화이트리스트(`git`, `ls`, `grep` 등)를 적용하여 미인가 바이너리 실행 시 무조건 사용자 승인(Ask) 강제.
  - **[Memory]**: `SessionLogger` 동기 `append_message` 및 비동기 `append_message_async` 내부에 `writer.flush()` 명시적 호출 추가. 비정상 패닉 종료 시점에도 파일 유실이 없도록 OS 수준의 Page Cache 저장 강제.
  - *검증*: 전 단위 테스트(`cargo test`) 66개 통과 완료. Clippy 단일 경고 없는 완벽 준수 검증.

---

## Phase 20: v1.2.0-rc.1 Final Polish (심층 결함 수정)
**목표(Scope):** 실제 운영 환경에서 발생할 수 있는 보안 취약점(고도화 인젝션), TUI UX 결함(스크롤 점핑), 로직 결함(동적 설정 갱신 누락) 및 성능 저하(파이프 블로킹) 이슈를 최종 해결합니다. (Completed)

### 20.1 구현 및 검증 경로 (Execution Path)
- [x] **Phase 1: Logic & Performance** (프로바이더 핫리로딩 및 로그 쓰로틀링)
  - `src/providers/registry.rs` 및 `chat_runtime.rs`에 `OnceLock<RwLock>` 도입하여 `reload_providers` 구현.
  - `src/tools/shell.rs`에 `yield_now().await` 및 1MB 라인 커팅 가드 적용.
- [x] **Phase 2: Security** (명령어 치환 차단 및 파일 권한)
  - `src/domain/permissions.rs`에 `$()`, `  `, `\n` 정규식 추가. `sudo`, `rm` 명령어 사용 시 `/etc`, `/var` 경로 접근 차단하는 PathGuard 추가.
  - `src/infra/secret_store.rs` 및 `config_store.rs` 설정 저장 시 UNIX `chmod 600` (OpenOptionsExt) 원자적 적용.
- [x] **Phase 3: UX** (스크롤 점핑 방지)
  - `logs_buffer` 가지치기(pruning) 시, 삭제된 줄 수 N만큼 `state.ui.inspector_scroll` 값을 동기화하여 Sticky Scroll 방식을 구현.

---

## Phase 21: v1.3.0 Final Industrial Polish (완성도 향상 및 엣지 케이스 수정)
**목표(Scope):** 도구 출력의 ANSI 이스케이프 코드 처리, 비정상 종료 시 터미널 복구 보장, 동기 I/O의 완벽한 비동기 전환, 채팅 컨텍스트의 메모리 상한 관리, 그리고 API Key 입력 마스킹 등 UX와 시스템 안정성을 상용(Industrial) 수준으로 끌어올립니다.

### 21.1 구현 및 검증 경로 (Execution Path)
- [x] **Phase 1: Stability** (터미널 복구 및 비동기 I/O 리팩토링)
  - `src/main.rs` 및 `src/tui/terminal.rs`에 Panic Hook 및 RAII 기반의 터미널 상태 자동 복구(Raw 모드 해제) 로직 구현.
  - `src/domain/repo_map.rs` 등의 모든 동기 파일 I/O를 `tokio::fs` 기반으로 전환하여 비동기 블로킹 최소화.
- [x] **Phase 2: UX & Security** (API Key 마스킹 및 ANSI 코드 처리)
  - `src/tui/widgets/setting_wizard.rs` 내 API Key 입력 시 화면에 `*`로 마스킹되어 표시되도록 컴포넌트 수정.
  - `src/tui/widgets/inspector_tabs.rs`에 정규식 또는 ANSI 파싱 라이브러리를 도입하여, 도구 출력의 ANSI 코드를 렌더링 가능한 Span 구조로 치환하거나 필터링.
- [x] **Phase 3: Optimization** (컨텍스트 윈도잉 및 메모리 관리)
  - `src/domain/session.rs` 및 `src/app/state.rs`에 채팅 기록 무한 증식을 방지하기 위한 최대 컨텍스트 길이/메시지 수 기반의 Sliding Window 및 자동 요약(Summarize) 구조 도입.

## Phase 22: v1.4.0 Production Hardening (시스템 안정화 및 프로덕션 폴리싱)
**목표(Scope):** 설정 파일 저장의 원자성(Atomicity) 확보, 종료 신호(SIGINT/SIGTERM) 수신 시 Graceful Shutdown 구현, 스트리밍 시 ANSI 시퀀스 분절 현상 해결, UI 라인 래핑 성능 최적화, 정교한 토큰 추정을 통한 메모리 관리 고도화.

### 22.1 구현 및 검증 경로 (Execution Path)
- [x] **Phase 1: Data Integrity** (설정 저장 원자성 및 종료 신호 처리)
  - `src/infra/config_store.rs`: Write-and-Rename 패턴(`.tmp` 확장자로 임시 저장 후 `fs::rename`) 및 `fsync` 호출로 원자적 파일 쓰기 구현.
  - `src/main.rs`, `src/tui/terminal.rs`: `tokio::signal::ctrl_c` 캡처를 통한 이벤트 루프 종료(`AppAction::Quit`) 및 Graceful Shutdown 구현 (자식 프로세스 정리 포함).
- [x] **Phase 2: Streaming** (ANSI 시퀀스 분절 처리)
  - `src/app/tool_runtime.rs`, `src/tui/widgets/inspector_tabs.rs`: `vte` 파서 또는 Stateful Byte Accumulator를 도입하여 버퍼 경계에서 발생하는 ANSI 코드 분절에 의한 깨짐 현상 완벽 대응.
- [x] **Phase 3: Optimization** (라인 래핑 최적화 및 토큰 계산 정교화)
  - `src/tui/widgets/inspector_tabs.rs`: `ratatui`의 `Wrap`으로 인한 CPU 스파이크를 방지하기 위해 가로 스크롤(Horizontal Scroll)을 도입하거나 사전 Hard Wrap 로직 구축.
  - `src/domain/session.rs`: 영문 4문자당 1토큰, 한글 1문자당 1~2토큰 등의 가중치를 부여한 토큰 계산 휴리스틱 고도화 (오차율 10% 이내 목표).

## Phase 23: v1.5.0 Final Refinement (시스템 고도화 및 최종 품질 보증)
**목표(Scope):** 터미널 리사이징 대응, LLM 의 비정형/오류 도구 호출 자동 복구, RepoMap 스캔 지연 해소, 서브 프로세스 데드락을 방지하는 하드 타임아웃, 세션 로그 파일의 10MB 분할 및 유지 정책(Rotation)을 적용하여 완성도를 극대화합니다.

### 23.1 구현 및 검증 경로 (Execution Path)
- [x] **Phase 1: Robustness** (리사이즈 가드 및 도구 호출 검증)
  - `src/tui/terminal.rs`, `src/app/mod.rs`: `Event::Resize` 수신 시 `terminal.autoresize()` 호출 및 `area.width/height` 기반으로 80x24 미만 시 경고 화면 출력, scroll offset 클램핑 적용.
  - `src/app/tool_runtime.rs`: 도구 인자가 유효한 JSON 스키마를 따르지 않거나 필드가 누락된 경우 즉각 에러로 종료하지 않고 LLM에게 복구를 요청하는 피드백 루프 구현.
- [x] **Phase 2: Scalability** (스캔 최적화 및 타임아웃)
  - `src/domain/repo_map.rs`: `WalkBuilder`에 기본 스캔 깊이(`max_depth`) 제한을 두어 `node_modules` 등 거대 트리가 깊게 들어가는 것을 방지.
  - `src/tools/executor.rs`: 도구 실행 비동기 블록을 `tokio::time::timeout(Duration::from_secs(30))`으로 감싸서 타임아웃 발생 시 강제 종료(`SIGKILL`) 처리.
- [x] **Phase 3: Sustainability** (로그 로테이션)
  - `src/infra/session_log.rs`: 로그 파일 저장 시 10MB 초과를 감지하고, 초과 시 파일 롤오버 및 최신 5개만 유지하는 Retention 로직 적용.

## Phase 24: v1.6.0 Final Integrity Hardening (시스템 무결성 확정 및 최종 고도화)
**목표(Scope):** 워크스페이스 변화 동적 감지, 쉘 도구 인터렉티브 블로킹 방지, 스마트 컨텍스트 요약, 민감 정보(Secret) 마스킹, 그리고 유닛 테스트 지원을 위한 Mocking 구조를 도입하여 최종적인 무결성을 확정합니다.

### 24.1 구현 및 검증 경로 (Execution Path)
- [x] **Phase 1: Security & Safety** (민감 정보 마스킹 및 인터렉티브 쉘 가드)
  - `src/tools/shell.rs`: `Stdio::null()`을 `stdin`으로 명시적 할당하여, LLM이 입력을 대기하는 명령(`git commit` 등)을 실행 시 행(Hang)이 걸리지 않고 즉각 에러로 반환되게 함.
  - `src/app/mod.rs` & `src/infra/secret_store.rs`: UI 및 로그에 출력되는 텍스트 스트림을 가로채어, 로드된 Secret Key들을 `[REDACTED]`로 치환(Lazy Regex)하는 마스킹 파이프라인 구축.
- [x] **Phase 2: Data Consistency** (RepoMap 갱신 결함 해결)
  - `src/app/state.rs`, `src/app/tool_runtime.rs`: `state.repo_map_dirty` 플래그를 추가. 파일시스템 조작 도구 실행 시 플래그를 `true`로 켜고, 다음 채팅 전송 직전에 백그라운드로 RepoMap을 재빌드하는 Lazy Refresh 로직.
- [x] **Phase 3: Intelligence & Architecture** (컨텍스트 압축 및 DI 도입)
  - `src/domain/session.rs`: 세션 메시지 히스토리 정리 시 초기 3개(System, User Goal)는 절대 지워지지 않도록 보호 구역(Protected Range) 지정 및 요약 주입 로직 도입.
  - `src/providers/mod.rs`: `LlmProvider` 트레이트를 분리하고, 네트워크 요청을 배제한 단위 테스트용 `MockProvider` 의존성 주입 패턴 적용.

## Phase 30: v2.2.0 The Ultimate Hardening (시스템 운영 무결성 확정 및 배포 준비)
**상태**: ✅ 완료
**관련 문서**: spec.md §30

### 30.1 구현 및 검증 경로 (Execution Path)
- [x] **Phase 1: Resilience** (설정 마이그레이션 및 상태 복원력)
  - `src/infra/config_store.rs`: `settings.json` 스키마 변경 시의 하위 호환성을 위해 `Settings::migrate()` 기반의 버전 관리 파이프라인 도입. 프로그램 실행 시 고립된 좀비 `.tmp` 파일을 제거하는 `cleanup_tmp_files` 구현.
- [x] **Phase 2: Diagnostics** (smlcli doctor 도입)
  - `src/infra/doctor.rs`: `smlcli doctor` 커맨드 추가. Git, TTY, 환경 설정, API 접근성 등 핵심 요구사항을 사전에 테스트하고 리포트를 터미널에 출력.
- [x] **Phase 3: UX & Performance** (Windows 프로세스 정리 및 클립보드)
  - `src/app/mod.rs` & `src/tools/shell.rs`: Windows 환경 하위 프로세스 그룹 정리를 위해 `taskkill /F /T /PID` 호출 적용. `arboard` 라이브러리를 통해 TUI `y` 키 입력 시 타임라인/인스펙터 텍스트 복사 연동.

## Phase 31: v2.3.0 The Final Polish & Resilience (운영 안정성 고도화 및 최종 릴리즈 품질 확정)
**상태**: ✅ 완료
**관련 문서**: spec.md §31

### 31.1 구현 및 검증 경로 (Execution Path)
- [x] **Phase 1: Data Integrity** (설정 파일 롤백 및 네트워크 타임아웃)
  - `src/infra/config_store.rs`: 마이그레이션 실패 시를 대비하여 `.bak` 파일을 선 백업하고, 실패 시 원상 복구(Rollback)하는 트랜잭션 개념 도입.
  - `src/infra/doctor.rs`: 네트워크 API 진단 시 `tokio::time::timeout` 5초 제한을 걸어 행(Hang) 현상 방지.
- [x] **Phase 2: UX Notification & Policy** (Toast 알림 및 환경변수 화이트리스트)
  - `src/app/state.rs` & `src/tui/layout.rs`: 클립보드 복사 등 주요 동작에 대해 2초 만료(`expires_at`)를 가지는 하단 Toast Notification 팝업 구현.
  - `src/domain/settings.rs` & `src/tools/shell.rs`: 실행 환경 제어를 위해 `allowed_env_vars` 화이트리스트를 추가하고, `exec_shell_stream` 동작 시 해당 목록만 노출하도록 보안 제어 강화.
- [x] **Phase 3: Performance** (RepoMap 디스크 캐싱)
  - `src/domain/repo_map.rs`: 파일 개수와 수정시간(`mtime`)을 조합한 가벼운 해시(`cheap_hash`) 알고리즘을 도입. `repo_map_cache_{hash}.json` 형태로 디스크에 저장하여, 매번 AST 파싱 비용이 발생하는 대형 리포지토리의 성능 저하 해결.

## Phase 32: v2.4.0 Final Release Candidate (시스템 운영 무결성 확정 및 배포 준비)
**상태**: ✅ 완료
**관련 문서**: spec.md §32

### 32.1 구현 및 검증 경로 (Execution Path)
- [x] **Phase 1: Performance** (다중 도구 비동기 실행 및 쓰기 직렬화)
  - `src/app/tool_runtime.rs` 및 `state.rs`: `VecDeque`를 활용한 다중 도구 비동기 실행을 구현하고, `write_tool_queue`를 통한 쓰기 전용 도구(WriteFile, ExecShell) 순차 처리 로직(Mutex) 확보.
  - `TimelineBlock` 에 `tool_call_id` 연결로 비동기 렌더링 무결성 확보.
- [x] **Phase 2: UX** (TUI Help Overlay 및 CLI Auto-Completion)
  - `src/tui/help_overlay.rs`: 현재 활성화된 패널(Composer, Timeline 등)에 따라 컨텍스트에 맞는 단축키 헬프 오버레이 모달 창 렌더링.
  - `src/main.rs`: `clap_complete` 크레이트를 통해 다중 셸 지원(bash, zsh, fish 등) 오프라인 자동 완성 스크립트 출력 서브 커맨드 구현.
- [x] **Phase 3: Reliability & Security** (무소음 건강 검진 및 취약점 패치)
  - `src/app/mod.rs` & `infra/doctor.rs`: `App::new_async` 구동 시 토키오 태스크로 `DoctorReport::run_diagnostics()` 수행, 문제 발생 시 Toast 알림(`SilentHealthCheckFailed` 액션 트리거) 표시.
  - `Cargo.toml`: rand 의존성 및 패키지 버전을 `2.4.0`으로 갱신, 취약점 경고 회피 패치 완료.

## Phase 35: v2.5.0 System Hardening & Metadata (시스템 무결성 확정)
**상태**: ✅ 완료
**관련 문서**: spec.md §35

### 35.1 구현 및 검증 경로 (Execution Path)
- [x] **Phase 1: Process Management & DevOps** (고아 프로세스 및 메타데이터)
  - `src/infra/process_reaper.rs` 및 `src/tools/shell.rs`: `sysinfo` 기반으로 `SMLCLI_PID` 환경 변수가 일치하지 않는 고아 셸 프로세스를 감지하여 종료(Reap)하는 시스템 자원 정리.
  - `build.rs`, `src/infra/doctor.rs`: `shadow-rs`를 연동하여 릴리즈 바이너리에 Git 해시 및 빌드 타임 내장.
- [x] **Phase 2: UX Locale & Concurrency** (어댑티브 UI 보더 및 비동기 출력 순서)
  - `src/tui/widgets/mod.rs`: `ratatui::symbols::border::Set<'static>` 반환 구조체에서 `use_ascii_borders`나 `LANG` 환경변수 지원 불가 시 순수 ASCII(`+-|`)로 보더 fallback 변환.
  - `src/app/mod.rs`: 병렬 도구 실행(ToolFinished)이 순서없이 도착하더라도, 렌더링 시에는 호출된 순서(tool_index)에 맞추어 `pending_tool_outcomes`에 캐싱해두었다가 순서대로 로그/타임라인에 쓰도록 Ordered Aggregation 정렬 적용.
- [x] **Phase 3: Log Reliability** (세션 로거 안정성)
  - `src/infra/session_log.rs`: `read_to_string()` 대신 `BufReader::lines()`로 `SessionLogger::restore()`를 변경하여, 대용량 로그 파일 파싱 시 메모리 초과/패닉 문제를 구조적으로 차단.

---

## v3.0 Roadmap — 경쟁력 확보 태스크 리스트

> **관련 문서**: spec.md §v3.0 Roadmap (Phase 40-45)
> **배경**: v2.5.0 평가 리포트에서 도출된 5대 약점을 순차적으로 해소.
> **상태**: ⏳ 계획 (v2.5.0 마무리 후 착수)

### Phase 40: Git-Native Integration (v3.0.0)
- [x] **Task G-1: GitCheckpointTool 레지스트리 등록**
  - `tools/git_checkpoint.rs` → `Tool` trait 래핑 + `GLOBAL_REGISTRY` 등록
  - `is_write_tool()`/guard 테스트 `known_unregistered`에서 `GitCheckpoint` 제거
- [x] **Task G-2: GitEngine 자동 커밋 엔진**
  - `infra/git_engine.rs` 신규 생성: `auto_commit()`, `undo_last()`, `list_history()` API
  - `domain/settings.rs`에 `GitIntegrationConfig` 추가 + config.toml 영속화
- [x] **Task G-3: ToolFinished 자동 커밋 훅**
  - `app/mod.rs` ToolFinished 핸들러에서 `GitEngine::auto_commit()` 연동
  - 타임라인 `GitCommit` 블록 추가
- [x] **Task G-4: `/undo` 슬래시 명령어**
  - `commands/mod.rs` 라우팅 + `GitEngine::undo_last()` 호출 + Revert 블록
- [x] **Task G-5: Inspector Git 히스토리 탭**
  - Inspector 패널에 `Git` 탭 추가 + `list_history()` 리스트 렌더링 + diff 프리뷰

### Phase 41: Provider 확장성 (v3.1.0)
- [x] **Task P-1: ProviderKind Custom 변형 추가**
  - `domain/provider.rs`에 `Custom(String)` enum 변형 + `CustomProviderConfig` 타입
  - `PersistedSettings`에 `custom_providers` 필드 + config.toml 영속화
- [x] **Task P-2: OpenAICompatAdapter 커스텀 인스턴스화**
  - `get_adapter()`에서 `Custom` → 기존 `OpenAICompatAdapter`/`AnthropicAdapter` base_url 교체
  - `ToolDialect` 자동 감지 분기
- [x] **Task P-3: `/provider add/remove/list` 명령어**
  - TUI 오버레이 또는 CLI 인자로 커스텀 provider CRUD
  - `validate_credentials()` smoke test 연동

### Phase 42: OS-Level Sandbox (v3.2.0)
- [x] **Task S-1: 샌드박스 백엔드 감지**
  - `infra/sandbox.rs` 신규 생성: `detect_backend()`, `doctor` 출력 반영
- [x] **Task S-2: bubblewrap 래퍼**
  - `wrap_command_bwrap()` 구현: ro-bind/bind/proc/dev/unshare-net
- [x] **Task S-3: ExecShellTool 통합**
  - `execute()` 내부에서 sandbox 활성화 시 `bwrap` 래핑, 비활성화 시 기존 폴백
- [x] **Task S-4: `/config` Sandbox 섹션**
  - config 대시보드에 Sandbox 토글/경로 편집 UI + config.toml `[sandbox]` 테이블

### Phase 43: MCP 클라이언트 (v3.3.0) ⚠️ 인프라 구현 완료 / E2E 테스트 미비
- [x] **Task M-1: MCP JSON-RPC 클라이언트**
  - `infra/mcp_client.rs` 신규 생성: `McpClient` 구조체 (`Debug`, `Clone` 파생)
  - mpsc::channel(32) 기반 요청 큐 + oneshot::channel 기반 응답 매칭
  - Stdin Writer Task: `AtomicU64` 자동 ID 채번 → JSON-RPC 2.0 직렬화 → `\n` 구분 전송
  - Stdout Reader Task: `BufReader::read_line()` 라인 파싱 → `Arc<Mutex<HashMap<u64, Sender>>>` pending 역탐색
  - `initialize()` → `notifications/initialized` → `list_tools()` → `call_tool()` 프로토콜 구현
  - `tokio::time::timeout(10초)` 래퍼로 응답 무한 대기 차단
  - `McpToolInfo` 구조체: `name`, `description`, `inputSchema(rename)` 필드
  - **[v2.5.3]** `Arc<Mutex<Option<Child>>>` 핸들 보관 → `shutdown()` 메서드로 앱 종료 시 명시적 kill
  - **[v3.3.1]** 감사 HIGH-1: `App::run()` 종료 직후 `mcp_clients.values().shutdown().await` 호출 연동 완료. 모든 종료 경로(Quit/quit/Ctrl-C/SIGTERM)에서 프로세스 누수 완전 방지.
  - **[v2.5.3]** stderr drain task: 별도 `tokio::spawn`으로 stderr 소비하여 OS 파이프 버퍼 블로킹 방지
- [x] **Task M-2: 동적 도구 등록**
  - `app/mod.rs` 초기화 시 `settings.mcp_servers` 순회 → `tokio::spawn` 비동기 로드
  - `McpClient::spawn(name, cmd, args)` → `initialize()` → `list_tools()` → OpenAI tools JSON Schema 변환
  - **[v2.5.3]** MCP `{name, description, inputSchema}` → OpenAI `{type: "function", function: {...}}` 형식 래핑 적용
  - 네임스페이스 접두사: `mcp_{server_name}_{tool_name}` (OpenAI 도구명 규칙 호환)
  - `Action::McpToolsLoaded(server_name, schemas, client)` 이벤트로 이벤트 루프에 전달
  - **[v3.3.1]** 감사 MEDIUM-1: `Action::McpLoadFailed(name, error)` 액션 추가. spawn/list_tools 실패 시 타임라인에 에러 Notice 블록 표시. 기존 `if let Ok && let Ok` 침묵 처리 제거.
  - **[v3.3.1]** 감사 MEDIUM-2: `sanitize_tool_name_part()` 정규화 함수 도입. OpenAI tool name 규격(^[a-zA-Z0-9_-]+$) 위반 문자를 '_'로 치환.
  - **[v3.3.2]** 감사 HIGH-3: `mcp_tool_name_map: HashMap<sanitized_full_name, (sanitized_server, original_tool_name)>` 역매핑 테이블 도입. `mcp_clients` key를 정규화 서버명으로 저장하여 라우팅 일관성 확보. 이전 longest prefix match 방식 제거.
  - **[v3.3.3]** 감사 MEDIUM-2: `/mcp add` 시 정규화 서버명 충돌 검사 추가. `foo.bar`/`foo_bar` 같은 충돌 시 등록 거부.
  - **[v3.3.5]** 감사 HIGH-1: `build_mcp_full_name()` 도입. 서버/도구 파트를 각 최대 27자로 truncate하여 OpenAI 64자 제한 준수. `MAX_TOOL_NAME_LEN = 64`, 접두사 5자 + 접미사 예비 4자.
  - **[v3.3.6]** 감사 HIGH-1: `McpToolsLoaded` 핸들러에서 `extend()` → 전역 충돌 검사 + suffix 부여로 교체. 서버 간 truncation 충돌(앞 27자 동일) 방지. suffix 포함 64자 초과 시 base truncation 적용.
  - **[v3.3.7]** 감사 HIGH-1: 전역 충돌 해소 시 `mcp_tools_cache`의 schema `function.name`도 변경된 key와 동기화. schemas를 충돌 해소 완료 후 cache에 push. suffix 한계(9999) 초과 시 skip + 경고. `filter_map`으로 서버 내 skip도 동일 적용.
  - **[v3.3.8]** 감사 MEDIUM-1: skip 시 `schemas.retain()`으로 해당 schema 즉시 제거. cache에 라우팅 불가 도구 잔류 방지. skip 경고를 타임라인 Notice로도 표시.
  - **[v3.3.9]** 감사 MEDIUM-1: `McpClient::dummy()` 테스트 전용 생성자 도입. `handle_action(McpToolsLoaded)` 관통 테스트 2건 추가: 정상 로드 동기화 + 서버 간 충돌 suffix·schema·map 일관성 검증.
  - `RuntimeState.mcp_clients: HashMap<String, McpClient>` + `mcp_tools_cache: Vec<Value>` + `mcp_tool_name_map` 캐싱
  - `chat_runtime.rs`에서 `build_streaming_chat_request()` 시 `mcp_tools_cache`를 기존 도구에 합류
- [x] **Task M-3: Permission 통합 + `/mcp` 명령어**
  - `domain/permissions.rs`: `call.name.starts_with("mcp_")` → `PermissionResult::Ask` 강제 반환
  - `app/tool_runtime.rs`: `mcp_` 접두사 판별 → `mcp_tool_name_map` 직접 조회 → `(sanitized_server, original_tool_name)` 획득 → `call_tool(original_name)` JSON-RPC 위임 → `ToolResult`/`ToolError` 래핑
  - **[v3.3.3]** 감사 HIGH-1: `call_tool()` 응답에서 `isError`를 `content`보다 먼저 검사. MCP 공식 스키마(CallToolResult) 준수.
  - `app/command_router.rs`: `/mcp list` (서버 목록 표시), `/mcp add <name> <command> [args...]` (upsert + 정규화 충돌 검사 + 비동기 save_config), `/mcp remove <name>` (제거 + 비동기 save_config)
  - **⚠️ `/mcp add`/`remove`는 설정 저장 후 앱 재시작이 필요합니다. 런타임 즉시 반영은 미지원.**
  - `/help` 도움말 메뉴에 `/mcp` 항목 등록
- [x] **Task M-4: MCP E2E 테스트** ✅ (v3.7.0)
  - `scripts/mock_mcp_server.py`: JSON-RPC 2.0 mock 서버 (initialize/tools/list/tools/call)
  - `test_mcp_e2e_initialize_and_list_tools`: 실제 프로세스 spawn → initialize + list_tools 왕복 (2도구 반환 검증)
  - `test_mcp_e2e_call_tool`: get_weather/read_file tools/call 왕복 (응답 내용 검증)
  - `test_mcp_permission_engine_always_ask`: PermissionEngine mcp_ 접두사 → Ask 강제 반환
  - `test_mcp_namespace_strip_roundtrip`: sanitize → mcp_{server}_{tool} → 역매핑 복원 왕복
  - `test_mcp_config_add_remove_persistence`: Vec<McpServerConfig> push/upsert/retain 영속화
  - `test_ask_clarification_tool_registered`: GLOBAL_REGISTRY 등록 + 스키마 + Allow 검증
  - `test_questionnaire_state_submit_and_build`: 3문항 순차 답변 → build_result 조립 검증
  - `test_questionnaire_total_options`: allow_custom에 따른 옵션 수 계산 검증

### Phase 44: DeleteFile 및 TECH-DEBT 정리 (v3.4.0)
- [x] **Task D-1: DeleteFileTool 구현** ✅ (v3.4.0)
  - `tools/file_ops.rs`에 `Tool` trait 구현 + `GLOBAL_REGISTRY` 등록 완료
  - `domain/permissions.rs` PermissionEngine에 DeleteFile 쓰기 도구 검사 추가 (Workspace Trust Gate + 경로 횡단)
  - `app/tool_runtime.rs` format_tool_name에 DeleteFile 표시 포맷 추가
  - `tests/audit_regression.rs`의 `known_unregistered`에서 `DeleteFile` 제거 완료
  - guard 테스트가 자동으로 sandbox 검증 포함 (path_write_count ≥ 3)
- [x] **Task D-2: TECH-DEBT 일괄 정리** ✅ (v3.4.0)
  - `tui/mod.rs`, `tools/mod.rs`, `domain/mod.rs`, `app/mod.rs`, `infra/mod.rs` 모듈 레벨 `#[allow(dead_code)]` 전수 제거 (7건)
  - `infra/git_engine.rs`, `infra/mcp_client.rs` 파일 레벨 `#![allow(dead_code)]` 제거 (2건)
  - `providers/registry.rs`의 `ProviderRegistry` 1건만 cfg(test) 구조적 사유로 유지 (사유 주석 갱신)
  - `tools/shell.rs`의 `[ROADMAP/v3.0]` 주석을 실제 구현 상태 반영으로 갱신
  - 빌드 경고 0건, 94개 테스트 전부 통과 확인
  - (v2.5.0 감사 LOW-1) `FUTURE`/`ROADMAP` 주석을 코드에서 제거하고 `spec.md` Future Work 또는 이슈 트래커로 이관

### Phase 45: 빌드 & 배포 파이프라인 (v3.5.0) ✅
- [x] **Task CI-1: GitHub Actions CI 워크플로** ✅ (v3.5.0)
  - `.github/workflows/ci.yml`: fmt/clippy/test 게이트 + cargo cache 최적화
  - `version-sync` job: 버전 동기화 스크립트 CI 연동
- [x] **Task CI-2: Release 워크플로** ✅ (v3.5.0)
  - `.github/workflows/release.yml`: 태그(v*) push → quality-gate → Linux musl / Windows msvc 크로스 빌드 → GitHub Releases 자동 업로드
  - `softprops/action-gh-release@v2` 사용, release notes 자동 생성
  - musl-tools 자동 설치 포함 (정적 링크 바이너리)
- [x] **Task CI-3: 버전 동기화 검증** ✅ (v3.5.0)
  - `scripts/check-version-sync.sh`: Cargo.toml ↔ CHANGELOG.md ↔ Git Tag 버전 일치 검증
  - 로컬 실행 검증 완료 (v3.4.0 동기화 통과 확인)

### Phase 46: Workspace-scoped Session Management (v3.6.0) ✅
- [x] **Task S-1: Session Metadata & Workspace 격리** ✅ (v3.6.0)
  - `domain/session.rs`에 `SessionMetadata` 구조체 추가 (session_id, workspace_root, title, timestamps, log_filename)
  - `SessionAction` 열거형 추가 (NewSession, ResumeSession, ListSessions)
  - `infra/session_log.rs`에 `SessionIndex` 구조체 추가 (sessions_index.json CRUD)
  - `SessionLogger::new_workspace_session()`: 워크스페이스 연동 세션 생성 + 인덱스 등록
  - `DomainState.current_session_metadata` 필드 추가하여 활성 세션 추적
- [x] **Task S-2: Auto-Titling 파이프라인** ✅ (v3.6.0)
  - `chat_runtime.rs::submit_chat_request()`에서 첫 UserMessage 감지 시 프롬프트 앞 50자를 임시 제목으로 설정
  - 기존 세션의 경우 `updated_at` 타임스탬프만 갱신 (`SessionIndex::touch()`)
- [x] **Task S-3: TUI Session Picker (`/resume`, `/session`)** ✅ (v3.6.0)
  - `command_router.rs`에 `/resume`, `/session` 명령어 라우팅 추가
  - 현재 워크스페이스의 세션 목록을 KeyValueTable로 렌더링 (현재 세션 마커, 상대 시간 표시)
  - `/resume <번호>` 형태로 세션 전환: 메시지 복원, 로거 교체, 인덱스 touch
- [x] **Task S-4: `/new` 명령어 연동** ✅ (v3.6.0)
  - `/new` 명령어: 타임라인/세션 상태/스트림 어큐뮬레이터 초기화 후 새 세션 할당
  - SlashMenuState, CommandPaletteState, /help 도움말에 세션 관리 명령어 3건 추가

### Phase 47: Interactive Planning Questionnaire (v3.7.0) ✅
- [x] **Task Q-1: AskClarification 도구 스키마 정의 및 하네싱** ✅ (v3.7.0)
  - `domain/questionnaire.rs` 생성: `ClarificationQuestion`, `AskClarificationArgs`, `AskClarificationResult`, `QuestionnaireState` 도메인 타입
  - `tools/questionnaire.rs` 생성: `AskClarificationTool` (Tool trait 구현, OpenAI Function Calling 스키마)
  - `GLOBAL_REGISTRY`에 AskClarification 도구 정식 등록
  - PLAN 모드 시스템 프롬프트에 AskClarification 강제 사용 지침 주입 (하네싱)
- [x] **Task Q-2: Questionnaire TUI 렌더러 구현** ✅ (v3.7.0)
  - `tui/widgets/questionnaire.rs` 생성: 화면 중앙 모달 오버레이 위젯
  - 객관식 옵션 커서 렌더링 (▸ 마커, Cyan 하이라이트)
  - 주관식 텍스트 입력 필드 (▏ 커서 표시)
  - allow_custom: "✏ 직접 입력..." 옵션 렌더링
  - 진행률 표시 ("질문 N/M"), 하단 키보드 힌트
  - `layout.rs::draw()`에서 help_overlay 뒤에 questionnaire 오버레이 렌더링
- [x] **Task Q-3: State Machine 연동** ✅ (v3.7.0)
  - `action.rs`에 `ShowQuestionnaire` / `QuestionnaireCompleted` Action 추가
  - `UiState.questionnaire: Option<QuestionnaireState>` 필드 추가
  - `tool_runtime.rs`: AskClarification 도구명 감지 시 비동기 실행 대신 ShowQuestionnaire Action 발행
  - `mod.rs::handle_action()`: ShowQuestionnaire(QuestionnaireState 생성 + Approval 블록) / QuestionnaireCompleted(ToolResult 조립 + ToolFinished 전달)
  - `mod.rs::handle_questionnaire_key()`: ↑↓ 옵션 탐색, Enter 선택/제출, Esc 취소, 문자 입력/Backspace
### Phase 48: 1st & 2nd Audit Remediation (v3.7.1) ✅
- [x] **[Finding 1] GrepSearch 샌드박스 우회 차단**
  - 원시 경로 직접 검색을 `file_ops::validate_sandbox()`를 거치도록 수정.
  - `/etc`, `../` 등 외부 탐색 원천 차단.
  - `audit_regression.rs`에 `test_grep_search_sandbox_bypass` 회귀 테스트 추가.
- [x] **[Finding 2] Approval 타임아웃 큐 고립 방지**
  - 대기 중인 승인 요청이 만료되었을 때, 큐(`queued_approvals`)에 대기 중인 다음 요청을 정상적으로 팝업하도록 승격 로직 적용.
  - `audit_regression.rs`에 `test_approval_timeout_promotes_queue` 회귀 테스트 추가.
- [x] **[Finding 3] MCP pending 요청 타임아웃/EOF 메모리 누적 방지**
  - `McpClient`의 `pending_requests` 맵을 구조체 멤버로 승격.
  - 10초 타임아웃 시 펜딩 맵에서 명시적으로 엔트리 제거.
  - EOF(서버 종료) 시 펜딩 중인 모든 요청에 즉시 통지하여 무한 대기 블로킹 방지.
- [x] **[Finding 4] TUI 테마 색상 정책 통일**
  - `questionnaire.rs` 및 `help_overlay.rs` 내 직접 선언된 `Color::Cyan`, `Color::Rgb` 하드코딩 제거.
  - `state.palette()`에서 가져오는 테마 색상(`accent`, `bg_panel`, `text_primary`)으로 전면 교체하여 `/theme` 반영.
- [x] **[Finding 5] 직접 셸 실행(!) 에러 재전송 방지**
  - `! command` 등으로 발생한 실패 시 `ToolError`에서 `tool_call_id`가 없는 경우, LLM으로 불필요한 오류 피드백이 전송되지 않도록 방어 로직 추가.
- [x] **[Finding 6] 미연결 데드 코드 제거**
  - `providers/sanitize.rs` 모듈 삭제 및 관련 불필요 임포트 제거.
- [x] **[Finding 7] 파일 포맷 공백 및 EOF 정비**
  - MD 파일들의 trailing whitespace 및 EOF newline 처리 완료 (`git diff --check` 통과).
