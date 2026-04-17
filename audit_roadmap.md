# SMLCLI Audit Roadmap

본 문서는 `smlcli`의 무결성을 유지하기 위한 4단계(정합성, 위험요소, 아키텍처, 로드맵) 감사 프로세스입니다. 버전이 변경되거나 기능 규모가 커질 때, AI 에이전트는 `/audit` 트리거 발동 시 이 문서를 기반으로 코드 리포트를 출력해야 합니다.

## 1단계: 정합성 유지 (Consistency Audit)
개발 도중 코드와 스펙 문서가 어긋나지 않는지 점검합니다.
- `spec.md` vs 실제 코드의 권한 모델 일치 확인 (Safe, Confirm, Blocked 모드)
- `designs.md` vs 실제 TUI 인터랙션 키보드 맵 매칭 확인 (Tab 작동 여부 등)
- Rust 의존성의 버전 호환 테스트(`cargo check && cargo clippy`)
- 설정 마법사의 상태 다이어그램이 코드 흐름과 정확히 일치하는지 분석

## 2단계: 위험요소 및 보안 확인 (Risk & Security Audit)
명시적인 보안 홀을 점검하고, 사용자 동의 여부 절차를 검사합니다.
- 파일 기반 암호화 저장소(~/.smlcli/)를 우회하여 API 키가 평문 노출되는지 확인
- `diff` 검토나 `Ask` 확인 로그 없이 파일 쓰기(write)나 셸 명령이 통과되는 하드코딩 여부 추적
- 심볼릭 링크 공격 탐지 및 범위를 벗어난 홈 디렉터리 수정 시도 검열
- 과도한 stdout 스트리밍으로 인한 메모리 누수 점검

## 3단계: 아키텍처 결함 감지 (Architecture Audit)
애플리케이션의 모듈 분리가 `spec.md`를 잘 따르고 있는지 평가합니다.
- 도메인 레이어: `permissions`, `session`, `settings`가 UI 로직에 얽혀 있지 않은지 점검. 
- 터미널 렌더러 분리: `tui/` 폴더 밖에서 터미널 제어 코드가 불리지 않는지 확인. 
- Provider Layer: 상호 연결 인터페이스(trait)를 제대로 사용하여 하드 코딩 방지 여부 점검. 

## 4단계: 액션 로드맵 (Remediation Roadmap)
감사 이후 수정이 필요한 항목들을 즉시 도출하고 실행 가능한 계획으로 변경합니다.
1. **Critical:** 빌드 에러, 보안 누수, 문서와 극명하게 어긋나는 동작 -> 즉각 수정
2. **High:** TUI 오작동, 치명적인 패닉 발생, 권한 바이패스 -> Phase 내 수정
3. **Medium:** `similar` 기반 Diff 가시성 문제, 디자인 틀어짐 -> UI 수정
4. **Low:** 문서 오탈자, 추가 로그 레벨 도입 -> 보완

---

### Audit Trigger Command
사용자가 명령어로 `/audit` 지정 또는 "감사 실행" 관련 지시를 내리면, AI 에이전트는 소스 루트를 탐색한 수 위 4단계의 통과/실패 항목과 진행 권고를 출력하십시오.

---

## Phase 9 UX 아키텍처 개편 감사 기준 (v0.1.0-beta.18+)

### 9-A 이벤트 기반 구조 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Action enum 14종 | `action.rs` 내 variant 수 확인 | ChatStarted, ChatDelta, ToolQueued, ToolStarted, ToolOutputChunk, ToolSummaryReady 존재 |
| TimelineEntry 분리 | `state.rs` 에 `timeline_entries` 필드 존재 | session.messages와 별도 관리, timeline 렌더링에 messages 직접 접근 없음 |
| Semantic Palette | `tui/palette.rs` 상수 존재 | layout.rs/widgets에서 하드코딩 Color 사용 0건 |
| tick 기반 애니메이션 | `tick_count` 기반 스피너/깜빡임 | thinking indicator가 4프레임 회전 |
| Inspector 탭 실체 | `widgets/inspector_tabs.rs` 존재 | Preview/Diff/Search/Logs/Recent 각 탭에 실제 콘텐츠 |
| Tool 출력 요약 분리 | ToolFinished 핸들러 검사 | 타임라인에 2~4줄 요약, logs_buffer에 원문 |
| SSE 스트리밍 | `chat_stream()` 메서드 존재 | ChatDelta 이벤트로 토큰 단위 수신 |

### 9-B 기능 완성 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| CLI Entry Modes | `main.rs`에 clap 파싱 | `run`, `doctor`, `export-log` 서브커맨드 동작 |
| 세션 영속성 | `session_log.rs` 존재 | JSONL 저장/복원 round-trip |
| SafeOnly 화이트리스트 | `permissions.rs` 검사 | safe_commands 미매칭 시 Deny |
| Blocked Command | `permissions.rs` 검사 | sudo/rm -rf 등 무조건 차단 |
| Structured Tool Call | `registry.rs` 검사 | fenced JSON 스크래핑 외 native contract 존재 |
| File Read 안전장치 | `file_ops.rs` 검사 | 경로 정규화 + 1MB 제한 |
| Grep UX | `grep.rs` 검사 | context_lines + max_results |
| 프롬프트 커맨드 확장 (@, !) | `FuzzyMode` enum 존재 | `ignore::WalkBuilder` 사용 유무, `history_idx` 상태 보존 유무 검사 |

### 9-C 품질 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Shell 스트리밍 | `shell.rs` 검사 | spawn + BufReader + ToolOutputChunk |
| 테스트 확장 | `cargo test` | 22건+ (secret round-trip, cancel/rollback, tool lifecycle, layout snapshot 포함) |
| 전역 allow 제거 | `main.rs` 검사 | #[allow(dead_code)] 등 0건 |

---

## Phase 13 Agentic Autonomy 개편 감사 기준

### 13-A 도구 및 실행 아키텍처 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Tool Registry | `src/tools/registry.rs` 코드 확인 | `Tool` 트레이트를 구현하는 개별 도구 구조체(`ReadFile`, `WriteFile` 등)가 존재하며 다형성 호출 보장 |
| Automated Git Checkpoints | `ReplaceFileContent` 실행 시뮬레이션 | 성공적인 파일 쓰기 직후 백그라운드에서 `git commit` 생성 및 `AI: Auto-checkpoint...` 메시지 확인 |
| Auto-Verify Loop | `smlcli run --auto-verify "테스트 실패 유도"` | 셸 에러가 발생했을 때 최대 3회 이내로 자동 복구 프롬프트가 주입되어 루프를 도는지 검증 |

### 13-B 에이전트 인텔리전스 감사

| 항목 | 검증 방법 | 합격 기준 |
|------|-----------|-----------|
| Tree-sitter Repo Map | `smlcli` 부팅 후 디버그 로그 점검 | `System` 프롬프트 하단에 `[Repo Map]` 블록으로 추출된 주요 함수/클래스 시그니처 2,000 토큰 이하 주입 확인 |
| Tree of Thoughts UI | 타임라인 렌더링 확인 | 메인 응답 텍스트 블록 아래에 도구 실행과 에러 이력이 인덴트(`└─`)로 계층화되어 표시됨 |
| Planner-Executor 분리 | 복합 프롬프트 실행 | 여러 파일 변경 지시 시, Planner의 계획 수립 단계와 Executor의 도구 실행 단계가 분리되어 Action 스트림에 흐르는지 확인 |
