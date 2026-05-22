# D3D Audit Report (audit_report_4.md)

- **감사 대상 프로젝트:** smlcli (Terminal-native AI Agent CLI)
- **감사 기준 문서:** `AI_AUDIT_DOC_STANDARD.md`, `AI_IMPLEMENTATION_DOC_STANDARD.md`, `spec.md`
- **프로젝트 버전:** v3.8.0
- **감사 일시:** 2026-05-22T12:41:13+09:00
- **감사 결과:** **PASS WITH RESOLVED FIXES**
- **수정 요약:** LM Studio 어댑터 컴파일 오류, Cargo/문서 버전 불일치, Rustfmt 불일치 수정 완료

## 1. Audit Scope

이번 감사는 전체 구현 상태, 문서-코드 정합성, 빌드 오류, 테스트/Clippy/보안 감사 게이트를 현재 작업 트리 기준으로 재검증했다.

확인한 주요 문서:

- `spec.md`
- `designs.md`
- `README.md`
- `CHANGELOG.md`
- `IMPLEMENTATION_SUMMARY.md`
- `DESIGN_DECISIONS.md`
- `BUILD_GUIDE.md`
- `audit_roadmap.md`
- `AI_AUDIT_DOC_STANDARD.md`
- `AI_IMPLEMENTATION_DOC_STANDARD.md`

확인한 주요 소스:

- `src/providers/registry.rs`
- `src/app/mod.rs`
- `src/app/command_router.rs`
- `src/app/wizard_controller.rs`
- `src/tui/widgets/setting_wizard.rs`
- `src/tests/audit_regression.rs`
- `Cargo.toml`
- `Cargo.lock`

## 2. Excluded Scope

- `target/`, `.git/`, 외부 서비스 실제 API 호출은 감사 범위에서 제외했다.
- `cargo audit`는 최초 샌드박스 실행에서 `~/.cargo/advisory-db..lock` 쓰기 제한으로 실패했으나, 승인된 외부 권한 재실행으로 통과를 확인했다.
- `cargo run -- auth --help`는 현재 공개 CLI에 `auth` 서브커맨드가 없어 실패했다. 현재 `audit_roadmap.md`와 `README.md`의 공개 CLI 표면은 `run`, `doctor`, `sessions`, `completions` 중심이므로 본 건은 빌드 결함이 아니라 과거 감사 체크리스트 항목의 노후화로 분류했다.

## 3. Pass 1: Implementation Compliance Findings

### [IMP-F001] v3.8.0 문서-매니페스트 버전 불일치 수정

- **Pass:** Implementation
- **Pattern:** IMP-001
- **Area:** Version / Documentation Sync
- **Severity:** Major
- **Status:** Verified
- **Summary:** `CHANGELOG.md` 최신 릴리스는 `3.8.0`이었지만 `Cargo.toml`과 `spec.md`의 현재 버전은 `3.7.1`로 남아 있어 버전 동기화 게이트가 실패했다.
- **Evidence:** `scripts/check-version-sync.sh` 최초 실행 결과 `Cargo.toml (3.7.1) != CHANGELOG.md (3.8.0)` 실패.
- **Expected:** 릴리스 버전은 `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, `spec.md`에서 동일해야 한다.
- **Actual:** 수정 전에는 `Cargo.toml`/`spec.md`가 `3.7.1`이었다.
- **Fix:** `Cargo.toml`, `Cargo.lock`, `spec.md`를 `3.8.0`으로 동기화했다.
- **Re-audit Method:** `./scripts/check-version-sync.sh` 재실행.
- **Verification:** `scripts/check-version-sync.sh` 통과, `cargo run -- --version` 출력 `smlcli 3.8.0`.

### [IMP-F002] LM Studio 위저드 기능 계약 유지 확인

- **Pass:** Implementation
- **Pattern:** IMP-003
- **Area:** Provider / Wizard Flow
- **Severity:** Info
- **Status:** Verified
- **Summary:** LM Studio 선택 시 API Key 단계를 건너뛰고 Base URL 입력 및 수동 모델 입력 fallback으로 이어지는 계약은 테스트로 유지되고 있다.
- **Evidence:** `tests::audit_regression::test_lm_studio_wizard_flow` 통과.
- **Expected:** LM Studio는 `AuthStrategy::None`, 기본 URL `http://localhost:1234/v1`, 수동 모델 입력 fallback을 제공해야 한다.
- **Actual:** `ProviderKind::LmStudio`, `WizardStep::BaseUrlInput`, `is_custom_model_mode` 경로가 통합 테스트에서 검증됐다.
- **Suggested Fix:** 추가 수정 불필요.
- **Re-audit Method:** `cargo test --all-targets --no-fail-fast`.

## 4. Pass 2: Debug / Engineering Quality Findings

### [DBG-F001] LM Studio 어댑터 Clone 누락으로 인한 빌드 오류 수정

- **Pass:** Debug / Engineering Quality
- **Pattern:** DBG-001
- **Area:** Rust Compile / Provider Registry
- **Severity:** Critical
- **Status:** Verified
- **Summary:** `ProviderRegistry::get_adapter(ProviderKind::LmStudio)`가 `RwLockReadGuard<OpenAICompatAdapter>`에서 `clone()`을 호출하지만 `OpenAICompatAdapter`가 `Clone`을 구현하지 않아 `cargo check --all-targets`가 실패했다.
- **Evidence:** 최초 `cargo check --all-targets`에서 `src/providers/registry.rs:859:61`, `no method named clone found for struct RwLockReadGuard<'_, OpenAICompatAdapter>` 오류 발생.
- **Expected:** LM Studio 어댑터는 `Arc<dyn ProviderAdapter>`로 반환될 수 있어야 한다.
- **Actual:** `OpenAICompatAdapter`가 `Clone`이 아니어서 반환 경로가 컴파일되지 않았다.
- **Fix:** `OpenAICompatAdapter`에 `#[derive(Clone)]`을 추가했다. 내부 필드인 `reqwest::Client`, `String`, `AuthStrategy`는 모두 Clone 가능하다.
- **Re-audit Method:** `cargo check --all-targets`, `cargo clippy --all-targets --all-features -- -D warnings`.
- **Verification:** 두 명령 모두 통과.

### [DBG-F002] Rustfmt 불일치 수정

- **Pass:** Debug / Engineering Quality
- **Pattern:** DBG-002
- **Area:** Formatting Gate
- **Severity:** Minor
- **Status:** Verified
- **Summary:** LM Studio 관련 신규 코드 일부가 rustfmt 기준과 달라 `cargo fmt --check`가 실패했다.
- **Evidence:** `src/app/command_router.rs`, `src/app/mod.rs`, `src/app/wizard_controller.rs`, `src/providers/registry.rs`, `src/tests/audit_regression.rs`, `src/tui/widgets/setting_wizard.rs`에서 포맷 diff 발생.
- **Expected:** `cargo fmt --check`가 변경 필요 없이 통과해야 한다.
- **Actual:** 최초 실행에서 포맷 diff가 출력됐다.
- **Fix:** `cargo fmt`를 적용했다.
- **Re-audit Method:** `cargo fmt --check`, `git diff --check`.
- **Verification:** 두 명령 모두 통과.

### [DBG-F003] 전체 테스트 및 공개 CLI 표면 검증

- **Pass:** Debug / Engineering Quality
- **Pattern:** TEST-001
- **Area:** Test / Runtime CLI
- **Severity:** Info
- **Status:** Verified
- **Summary:** 수정 후 전체 테스트와 주요 CLI 명령이 정상 동작한다.
- **Evidence:** `cargo test --all-targets --no-fail-fast`에서 105개 테스트 통과.
- **Checked Commands:**
  - `cargo run -- --help`: `run`, `doctor`, `sessions`, `completions` 표시
  - `cargo run -- --version`: `smlcli 3.8.0`
  - `cargo run -- sessions --help`: 정상 출력
  - `cargo run --quiet -- doctor`: 진단 실행 완료, 비TTY 및 네트워크 불안정 경고만 표시
- **Suggested Fix:** 공개 CLI에는 `auth` 서브커맨드가 없으므로 향후 감사 체크리스트에서 `auth --help` 항목은 제거하거나 현재 명령 구조로 교체해야 한다.
- **Re-audit Method:** 위 명령 재실행.

## 5. Pass 3: Security Findings

### [SEC-F001] RustSec 보안 감사 통과

- **Pass:** Security
- **Pattern:** SEC-002
- **Area:** Dependency Vulnerability Scan
- **Severity:** Info
- **Status:** Verified
- **Summary:** `Cargo.lock` 462개 crate dependency에 대해 RustSec advisory scan을 수행했고 취약점 보고 없이 종료 코드 0으로 통과했다.
- **Evidence:** `cargo audit` 승인된 외부 권한 실행, advisory 1096개 로드 후 scan 완료.
- **Expected:** 알려진 RustSec 취약점이 없어야 한다.
- **Actual:** 취약점 보고 없음.
- **Suggested Fix:** 추가 수정 불필요.
- **Re-audit Method:** `cargo audit`.

### [SEC-F002] LM Studio AuthStrategy::None 경계 유지

- **Pass:** Security
- **Pattern:** SEC-003
- **Area:** Local Provider Authentication Boundary
- **Severity:** Info
- **Status:** Verified
- **Summary:** LM Studio 로컬 프로바이더는 API Key를 요구하지 않는 설계이므로 `AuthStrategy::None`으로 명시되어 있고, 이번 Clone 수정은 인증 헤더 처리나 secret 저장 경계를 변경하지 않는다.
- **Evidence:** `ProviderRegistry::new()`와 `update_lmstudio_base_url()`이 `OpenAICompatAdapter::with_auth(..., AuthStrategy::None)`을 유지한다.
- **Expected:** LM Studio 경로는 토큰을 요구하거나 저장하지 않아야 한다.
- **Actual:** 인증 없음 전략 유지.
- **Suggested Fix:** 추가 수정 불필요.
- **Re-audit Method:** `src/providers/registry.rs` 확인 및 `test_lm_studio_wizard_flow` 재실행.

## 6. Cross-Pass Conflicts

- 없음. 빌드 수정, 버전 동기화, 문서 동기화가 서로 같은 v3.8.0 릴리스 상태를 가리킨다.

## 7. Required Fixes Before PASS

- 없음. 이번 감사 중 발견된 필수 수정 3건은 모두 반영 후 재검증했다.

## 8. Accepted Risks

- `doctor`는 현재 환경에서 네트워크 불안정 경고와 비TTY 경고를 표시한다. 이는 실행 환경 조건이며 빌드/테스트 결함은 아니다.
- `cargo audit`는 샌드박스 내부에서 advisory DB lock 파일 생성 권한 문제로 실패할 수 있다. 승인된 외부 권한 실행 경로는 정상 통과했다.

## 9. Needs Spec Clarification

- 과거 감사 체크리스트에 남아 있던 `cargo run -- auth --help` 검증 항목은 현재 CLI 명령 구조와 맞지 않는다. 현재 문서상 공개 명령은 `run`, `doctor`, `sessions`, `completions`이므로 후속 문서 정리 시 감사 체크리스트에서 `auth` 항목을 제거하거나 대체해야 한다.

## 10. Re-audit Checklist

| Command | Result |
| --- | --- |
| `cargo fmt --check` | PASS |
| `cargo check --all-targets` | PASS |
| `cargo test --all-targets --no-fail-fast` | PASS, 105 passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |
| `./scripts/check-version-sync.sh` | PASS |
| `git diff --check` | PASS |
| `cargo run -- --help` | PASS |
| `cargo run -- --version` | PASS, `smlcli 3.8.0` |
| `cargo run -- sessions --help` | PASS |
| `cargo run --quiet -- doctor` | PASS with environment warnings |
| `cargo audit` | PASS after escalated advisory DB access |

## 11. Final Decision

**PASS WITH RESOLVED FIXES.**

현재 작업 트리는 v3.8.0 기준으로 전체 빌드, 테스트, Clippy, 포맷, 버전 동기화, RustSec 보안 감사 게이트를 통과한다. 최초 발견된 빌드 오류와 버전 불일치는 수정 완료됐고, 결과는 본 보고서에 별도 기록했다.
