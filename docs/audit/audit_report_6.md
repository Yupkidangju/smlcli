# D3D Audit Report (audit_report_6.md)

- **감사 대상 프로젝트:** smlcli (Terminal-native AI Agent CLI)
- **프로젝트 경로:** `/mnt/Projects_SSD/rust/smlcli`
- **감사 기준 문서:** `AI_AUDIT_DOC_STANDARD.md`, `AI_IMPLEMENTATION_DOC_STANDARD.md`, `spec.md`, `audit_roadmap.md`
- **프로젝트 버전:** v3.9.0
- **감사 일시:** 2026-05-23T02:12:00+09:00
- **감사 모드:** 보고서 전용 재감사. 소스 수정 없음.
- **최종 판정:** **PASS WITH KNOWN RISKS**

## 1. Audit Scope

이번 재감사는 현재 작업 트리 기준으로 `audit_report_5.md`의 PASS 주장과 실제 문서/소스/테스트/실행 증거를 독립적으로 재검증했다. 이전 HOLD 사유였던 포맷, 공백, README/구현 요약 동기화, v3.9.0 UI/i18n 테스트, `ExecShell.safe_to_auto_run` 경계 명세를 중심으로 확인했다.

확인한 주요 문서:

- `AI_AUDIT_DOC_STANDARD.md`
- `AI_IMPLEMENTATION_DOC_STANDARD.md`
- `spec.md`
- `designs.md`
- `README.md`
- `CHANGELOG.md`
- `IMPLEMENTATION_SUMMARY.md`
- `DESIGN_DECISIONS.md`
- `BUILD_GUIDE.md`
- `LESSONS_LEARNED.md`
- `audit_roadmap.md`
- `audit_report_5.md`

확인한 주요 소스 및 테스트:

- `Cargo.toml`
- `Cargo.lock`
- `src/app/mod.rs`
- `src/app/state.rs`
- `src/domain/permissions.rs`
- `src/domain/settings.rs`
- `src/tools/shell.rs`
- `src/tui/i18n.rs`
- `src/tui/layout.rs`
- `src/tui/palette.rs`
- `src/tui/widgets/inspector_tabs.rs`
- `src/tui/widgets/questionnaire.rs`
- `src/tests/audit_regression.rs`

## 2. Excluded Scope

- `target/`, `.git/`, 외부 provider 실제 API 호출, 실제 TUI 물리 스크린샷 검증은 제외했다.
- `cargo test --all-targets --no-fail-fast` 중 Git 관련 테스트가 임시 저장소에서 commit 로그를 출력하는 것은 테스트 설계상 정상 동작으로 분류했다. 감사 전후 `git status --short`는 비어 있었다.
- `cargo run -- auth --help`는 현재 공개 CLI에 `auth` 서브커맨드가 없어 제외했다. 현재 CLI 표면은 `run`, `doctor`, `sessions`, `completions`이다.

## 3. Pass 1: Implementation Compliance Findings

### [IMP-F001] v3.9.0 README 다국어 기능 설명 동기화 확인

- **Pass:** Implementation
- **Pattern:** IMP-001 / DOC-BACKFILL-001
- **Area:** README / Feature Description
- **Severity:** Info
- **Status:** Verified
- **Summary:** 이전 감사에서 누락됐던 v3.9.0 TUI 현대화 설명이 README 5개 언어 Features 섹션에 반영됐다.
- **Evidence:**
  - `README.md:32` 한국어 Features에 v3.9.0 TUI 현대화 항목 존재.
  - `README.md:79` English Features에 v3.9.0 TUI Modernization 항목 존재.
  - `README.md:118` 일본어 Features에 v3.9.0 TUI 항목 존재.
  - `README.md:157` 중국어 번체 Features에 v3.9.0 항목 존재.
  - `README.md:196` 중국어 간체 Features에 v3.9.0 항목 존재.
- **Expected:** README는 한/영/일/중(번체)/중(간체) 순서로 사용자-facing 새 기능을 설명해야 한다.
- **Actual:** 5개 언어에 v3.9.0 항목이 모두 있다.
- **Impact:** 사용자-facing 문서 동기화 차단 사유가 해소됐다.
- **Suggested Fix:** 없음.
- **Re-audit Method:** `rg -n "v3\\.9\\.0 TUI" README.md`.
- **Owner:** Auditor

### [IMP-F002] IMPLEMENTATION_SUMMARY v3.9.0 Phase 52 요약 추가 확인

- **Pass:** Implementation
- **Pattern:** IMP-001
- **Area:** Implementation Summary
- **Severity:** Info
- **Status:** Verified
- **Summary:** `IMPLEMENTATION_SUMMARY.md`에 Phase 52 / v3.9.0 구현 요약이 추가됐다.
- **Evidence:** `IMPLEMENTATION_SUMMARY.md:83-91`에 반응형 레이아웃, `Alt+1`~`Alt+6`, 500ms 커서, `centered_rect`, i18n 테스트, `DIFF_RENDER_CACHE`, `safe_to_auto_run`, 108개 테스트 패스가 기록되어 있다.
- **Expected:** 구현 완료 범위와 검증 결과가 구현 요약 문서에 남아야 한다.
- **Actual:** v3.9.0 관련 요약이 추가됐다.
- **Impact:** 이전 문서 복구 요구는 대체로 해소됐다.
- **Suggested Fix:** 아래 `IMP-F004`의 경미한 탭 명칭 drift만 정정하면 더 정확하다.
- **Re-audit Method:** `sed -n '80,94p' IMPLEMENTATION_SUMMARY.md`.
- **Owner:** Auditor

### [IMP-F003] v3.9.0 핵심 코드 표면과 테스트 존재 확인

- **Pass:** Implementation
- **Pattern:** IMP-003
- **Area:** TUI Modernization / Test Coverage
- **Severity:** Info
- **Status:** Verified
- **Summary:** 문서가 주장하는 핵심 구현 표면과 전용 회귀 테스트가 현재 소스에 존재한다.
- **Evidence:**
  - `src/app/mod.rs:1950-1968`에 `Alt+1`~`Alt+6` 인스펙터 탭 전환이 있다.
  - `src/app/state.rs:8-15`의 실제 탭은 `Preview`, `Diff`, `Search`, `Logs`, `Recent`, `Git`이다.
  - `src/tui/widgets/inspector_tabs.rs:36-46`, `src/tui/widgets/inspector_tabs.rs:151-188`에 `DIFF_RENDER_CACHE`와 캐시 히트/미스 로직이 있다.
  - `src/tui/widgets/inspector_tabs.rs:640-712`에 `test_diff_render_cache_invalidation`이 있다.
  - `src/tests/audit_regression.rs:4085-4125`에 `test_v3_9_0_i18n_key_completeness`가 있다.
  - `src/tests/audit_regression.rs:4127-4160`에 `test_v3_9_0_centered_rect_formula`가 있다.
- **Expected:** v3.9.0 주요 UI 변경에는 코드와 결정적 테스트 증거가 있어야 한다.
- **Actual:** 코드 표면과 테스트가 모두 존재하며 전체 테스트도 통과했다.
- **Impact:** 이전 테스트 부족 finding은 해소됐다.
- **Suggested Fix:** 없음.
- **Re-audit Method:** 위 파일 범위 확인 및 `cargo test --all-targets --no-fail-fast`.
- **Owner:** Auditor

### [IMP-F004] 구현 요약의 Inspector 6번째 탭 명칭이 실제 코드와 다름

- **Pass:** Implementation
- **Pattern:** IMP-001
- **Area:** Documentation Accuracy
- **Severity:** Minor
- **Status:** Accepted Risk
- **Summary:** `IMPLEMENTATION_SUMMARY.md`는 6대 탭을 `Preview, Diff, Logs, Search, Recent, Settings`라고 쓰지만 실제 코드의 6번째 탭은 `Git`이다. 또한 실제 단축키 순서는 `Preview`, `Diff`, `Search`, `Logs`, `Recent`, `Git`이다.
- **Evidence:**
  - `IMPLEMENTATION_SUMMARY.md:85`는 `Settings`를 6번째 탭으로 기술한다.
  - `src/app/state.rs:8-15`에는 `InspectorTab::Git`은 있으나 `Settings` variant는 없다.
  - `src/app/mod.rs:1953-1958`은 `Alt+6`을 `InspectorTab::Git`에 매핑한다.
- **Expected:** 구현 요약의 탭 이름과 순서는 코드의 공개 동작과 일치해야 한다.
- **Actual:** 한 줄의 탭 명칭과 순서가 실제 코드와 다르다.
- **Impact:** 사용자-facing README에는 특정 탭명이 없어 영향이 낮고, 코드/테스트/CLI 게이트에는 영향이 없다. 단, 향후 문서 정리 시 수정해야 한다.
- **Suggested Fix:** `IMPLEMENTATION_SUMMARY.md:85`를 `Preview, Diff, Search, Logs, Recent, Git`으로 정정한다.
- **Re-audit Method:** `rg -n "Preview, Diff" IMPLEMENTATION_SUMMARY.md` 및 `src/app/state.rs` 대조.
- **Owner:** Coder

## 4. Pass 2: Debug / Engineering Quality Findings

### [DBG-F001] 포맷 및 공백 게이트 통과

- **Pass:** Debug / Engineering Quality
- **Pattern:** DBG-002
- **Area:** Formatting / Whitespace
- **Severity:** Info
- **Status:** Verified
- **Summary:** 이전 HOLD의 직접 원인이었던 `cargo fmt --check`와 `git diff --check`가 모두 통과했다.
- **Evidence:**
  - `cargo fmt --check`: 종료 코드 0, 출력 없음.
  - `git diff --check`: 종료 코드 0, 출력 없음.
- **Expected:** 포맷과 diff whitespace 게이트가 클린해야 한다.
- **Actual:** 둘 다 통과했다.
- **Impact:** 이전 포맷/공백 차단 사유는 해소됐다.
- **Suggested Fix:** 없음.
- **Re-audit Method:** 위 명령 재실행.
- **Owner:** Auditor

### [DBG-F002] 빌드, 테스트, Clippy, 버전 동기화 통과

- **Pass:** Debug / Engineering Quality
- **Pattern:** BUILD-001 / TEST-001
- **Area:** Build / Test / Lint / Version
- **Severity:** Info
- **Status:** Verified
- **Summary:** 주요 Rust 품질 게이트가 현재 트리에서 모두 통과했다.
- **Evidence:**
  - `cargo check --all-targets`: PASS.
  - `cargo test --all-targets --no-fail-fast`: PASS, 108 passed, 0 failed.
  - `cargo clippy --all-targets --all-features -- -D warnings`: PASS.
  - `./scripts/check-version-sync.sh`: PASS, `Cargo.toml` 3.9.0과 `CHANGELOG.md` 3.9.0 일치.
- **Expected:** 컴파일, 테스트, lint, 버전 동기화가 재현 가능하게 통과해야 한다.
- **Actual:** 모두 통과했다.
- **Impact:** Debug/Engineering 품질 게이트는 통과 상태다.
- **Suggested Fix:** 없음.
- **Re-audit Method:** 위 명령 재실행.
- **Owner:** Auditor

### [DBG-F003] 공개 CLI 및 doctor 동작 확인

- **Pass:** Debug / Engineering Quality
- **Pattern:** RUNTIME-001
- **Area:** CLI Surface / Runtime Diagnostics
- **Severity:** Info
- **Status:** Verified
- **Summary:** 공개 CLI 표면과 기본 진단 명령이 현재 문서의 실행 표면과 일치한다.
- **Evidence:**
  - `cargo run -- --help`: `run`, `doctor`, `sessions`, `completions` 표시.
  - `cargo run -- --version`: `smlcli 3.9.0`.
  - `cargo run -- sessions --help`: 정상 출력.
  - `cargo run --quiet -- doctor`: 설정 로드, Git, bubblewrap 확인. 네트워크 불안정 및 비TTY 경고 표시.
- **Expected:** 공개 CLI 기본 명령이 실행 가능해야 한다.
- **Actual:** 정상 실행된다. `doctor`의 네트워크/비TTY 경고는 현재 감사 환경 조건이다.
- **Impact:** CLI runtime surface는 통과 상태다.
- **Suggested Fix:** 없음.
- **Re-audit Method:** 위 명령 재실행.
- **Owner:** Auditor

## 5. Pass 3: Security Findings

### [SEC-F001] RustSec 보안 감사 통과

- **Pass:** Security
- **Pattern:** SEC-002
- **Area:** Dependency Vulnerability Scan
- **Severity:** Info
- **Status:** Verified
- **Summary:** RustSec advisory scan이 취약점 보고 없이 통과했다.
- **Evidence:** `cargo audit` 종료 코드 0. 1098개 advisory 로드, 462개 crate dependency scan 완료.
- **Expected:** 알려진 RustSec 취약점이 없어야 한다.
- **Actual:** 취약점 보고 없음.
- **Impact:** dependency 보안 게이트 통과.
- **Suggested Fix:** 없음.
- **Re-audit Method:** `cargo audit`.
- **Owner:** Auditor

### [SEC-F002] `ExecShell.safe_to_auto_run` 신뢰 경계 제한 확인

- **Pass:** Security
- **Pattern:** SEC-004
- **Area:** Shell Permission Boundary
- **Severity:** Info
- **Status:** Verified
- **Summary:** 이전 감사의 `Needs Spec Clarification` 대상이었던 `safe_to_auto_run` 경계가 문서와 코드에 반영됐다.
- **Evidence:**
  - `spec.md:953-954`는 직접 셸 실행과 LLM tool-call의 `safe_to_auto_run` 신뢰 경계 제한을 명시한다.
  - `DESIGN_DECISIONS.md:1320-1350`에 ADR-038이 추가됐다.
  - `src/tools/shell.rs:546-563`은 `is_custom_safe || is_builtin_safe`와 `safe_to_auto_run`이 모두 참일 때만 `Allow`를 반환한다.
  - `src/tests/audit_regression.rs`에는 빈 명령, 위험 명령, 직접 셸 `safe_to_auto_run=false` 관련 회귀 테스트가 있고 전체 테스트가 108개 통과했다.
- **Expected:** LLM이 `safe_to_auto_run=true`를 주장해도 런타임/사용자 허용 목록 경계 밖 명령은 자동 실행되지 않아야 한다.
- **Actual:** 코드와 문서가 이 경계를 반영한다.
- **Impact:** 이전 보안 명세 불명확성은 해소됐다.
- **Suggested Fix:** 보안 강화를 위해 향후 `safe_to_auto_run=true` + 비 allowlist 명령의 정확한 `Deny` 단위 테스트 이름을 별도로 추가하면 더 명확하다.
- **Re-audit Method:** `src/tools/shell.rs:533-570`, `spec.md:953-954`, ADR-038, shell permission tests 확인.
- **Owner:** Auditor

### [SEC-F003] 파일/검색 workspace 경계 회귀 테스트 통과

- **Pass:** Security
- **Pattern:** SEC-003
- **Area:** Filesystem Boundary
- **Severity:** Info
- **Status:** Verified
- **Summary:** workspace 외부 파일 접근과 grep 우회 방어 테스트가 계속 통과한다.
- **Evidence:** `cargo test --all-targets --no-fail-fast`에서 `test_grep_search_sandbox_bypass`, `test_all_write_tools_deny_outside_workspace_paths`, `test_read_file_path_traversal_denied`, `test_write_file_sandbox_blocks_absolute_path_outside_workspace` 통과.
- **Expected:** workspace 외부 파일 접근은 거부되어야 한다.
- **Actual:** 관련 회귀 테스트가 통과한다.
- **Impact:** 파일 경계 보안은 현재 감사에서 통과 상태다.
- **Suggested Fix:** 없음.
- **Re-audit Method:** 관련 테스트 재실행.
- **Owner:** Auditor

## 6. Cross-Pass Conflicts

### [XPF-F001] 이전 HOLD 차단 사유는 해소됐으나 경미한 문서 drift가 남음

- **Pass:** Cross-Pass
- **Pattern:** XPF-001
- **Area:** Roadmap / Summary / Source Alignment
- **Severity:** Minor
- **Status:** Accepted Risk
- **Summary:** 포맷, 공백, 테스트, 보안 명세, README/구현요약 동기화의 주요 차단 사유는 해소됐다. 다만 `IMPLEMENTATION_SUMMARY.md:85`의 `Settings` 탭 명칭은 실제 `Git` 탭과 다르다.
- **Evidence:** `cargo fmt --check`, `git diff --check`, `cargo check`, `cargo test`, `cargo clippy`, `cargo audit` 통과. `src/app/state.rs:8-15` 및 `src/app/mod.rs:1953-1958`은 6번째 탭이 `Git`임을 증명한다.
- **Expected:** PASS 전 Major/Critical 차단 사유는 없어야 하며, 남는 Minor는 Known Risk로 명시되어야 한다.
- **Actual:** Major/Critical 차단 사유는 발견되지 않았다. Minor 문서 drift 1건만 남았다.
- **Impact:** release gate를 막지는 않지만 다음 문서 정리 때 수정해야 한다.
- **Suggested Fix:** `IMPLEMENTATION_SUMMARY.md:85` 한 줄 정정.
- **Re-audit Method:** 문서 한 줄과 `InspectorTab` enum/단축키 매핑 재대조.
- **Owner:** Coder

## 7. Required Fixes Before PASS

- 없음. Critical/Major 차단 finding은 현재 재감사에서 발견되지 않았다.

## 8. Accepted Risks

- `IMPLEMENTATION_SUMMARY.md:85`의 Inspector 탭 목록이 실제 코드의 `Git` 탭을 `Settings`로 잘못 표기한다. Severity는 Minor이며 다음 문서 정리 시 수정 가능하다.
- `doctor` 명령의 네트워크 불안정 및 비TTY 경고는 현재 실행 환경 조건으로 수용한다.
- 실제 외부 provider API 호출과 실제 TUI 스크린샷 검증은 이번 감사 범위에서 제외했다.

## 9. Needs Spec Clarification

- 없음. `ExecShell.safe_to_auto_run`의 신뢰 경계는 `spec.md`와 ADR-038에 반영되어 이번 감사에서는 판정 가능했다.

## 10. Re-audit Checklist

| Command / Check | Result |
| --- | --- |
| `git status --short` | PASS, 감사 전 비어 있음 |
| `cargo fmt --check` | PASS |
| `git diff --check` | PASS |
| `cargo check --all-targets` | PASS |
| `cargo test --all-targets --no-fail-fast` | PASS, 108 passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |
| `./scripts/check-version-sync.sh` | PASS |
| `cargo run -- --help` | PASS |
| `cargo run -- --version` | PASS, `smlcli 3.9.0` |
| `cargo run -- sessions --help` | PASS |
| `cargo run --quiet -- doctor` | PASS with environment warnings |
| `cargo audit` | PASS |
| `README.md` v3.9.0 다국어 feature sync | PASS |
| `IMPLEMENTATION_SUMMARY.md` v3.9.0 summary sync | PASS with Minor drift |
| `ExecShell.safe_to_auto_run` spec/ADR/code sync | PASS |

## 11. Final Decision

**PASS WITH KNOWN RISKS.**

현재 작업 트리는 v3.9.0 기준으로 포맷, 공백, 빌드, 테스트, Clippy, 버전 동기화, RustSec, 공개 CLI 기본 동작을 통과한다. 이전 HOLD의 Major 차단 사유였던 문서 동기화, UI/i18n 테스트 부족, `safe_to_auto_run` 신뢰 경계 불명확성도 현재 소스와 문서에서 해소됐다.

남은 리스크는 `IMPLEMENTATION_SUMMARY.md:85`의 Inspector 탭 목록에서 실제 `Git` 탭을 `Settings`로 표기한 Minor 문서 drift 1건이다. 이는 사용자-facing README와 실행 코드에는 영향을 주지 않으므로 PASS를 막지는 않지만, 다음 문서 정리 때 수정해야 한다.
