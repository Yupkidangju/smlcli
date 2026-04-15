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
- [x] **Task 5: Keyring 연동과 암호화 파일 설정**
  - 로컬 `master-key` 생성 및 운영체제 Keyring 등록 로직
  - XChaCha20Poly1305 기반 로컬 구성 정보(`settings.toml`) 암호화 및 복호화
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
- [x] **Task 15: 설정 및 Vault 암호화 연동**
  - 운영체제 Keyring과 XChaCha20 적용한 구성 영구 저장 기능
- [x] **Task 16: Inspector 반응형 분할 및 UI/UX 폴리싱**
  - 화면 폭/단축키(`Ctrl + I`)에 대응하는 동적 Split 레이아웃
  - `similar` Diff의 라인별 `초록색/빨간색` 렌더링 스팬 처리

## Phase 6: 슬래시 커맨드 및 설정 결합 (Commands & Config)
- [x] **Task 17: `/config` 종합 마스터 대시보드 구현**
  - TUI 오버레이를 통해 Provider, Model, Permission 등 모든 설정 내역을 방향키로 이동 및 수정
  - 변경 시 즉시 OS Keyring 및 암호화 파일(`settings.enc`)에 반영
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
  - [Architecture] `mod.rs` God Object(773줄)를 `command_router.rs`, `chat_runtime.rs`로 분해. WizardStep 미사용 variant 제거.
  - [Quality] `cargo fmt` 전체 적용. `cargo check && cargo test && cargo clippy` 전수 통과.
