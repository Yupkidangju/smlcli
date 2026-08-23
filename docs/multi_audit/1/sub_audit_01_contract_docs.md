# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 1
- Perspective: A01 — 목표·계약·문서·구현·기능 정합성
- User Goal: `$multi-audit 프로젝트의 모든 문제점을 파악하여 안정화하고 완성도를 높이는 전체감사 개시. 프로젝트의 모든 문서 및 구현내용을 먼저 파악후 작업을 분해하여 상세 감사 후 근본적인 문제를 해결할 수 있도록 합니다.`
- Audit Basis: Standard-backed
- Standard Path: `/mnt/Projects_SSD/rust/smlcli/AI_AUDIT_DOC_STANDARD.md`
- Report Contract Path: `/home/eunho1/.codex/skills/multi-audit/references/report-contract.md`
- Project Root: `/mnt/Projects_SSD/rust/smlcli`
- Audit Commit: `55d1c33` (`feat(harness): implement Workspace Harness Enforcement and standardize /workspace sandbox mount`)
- Working-tree baseline: `AGENTS.md`와 `AI_AUDIT_DOC_STANDARD.md` 수정, `AI_CODING_STANDARD.md` 및 `docs/` 미추적. 이 사용자 기존 변경은 보존하고 현재 증거로만 반영했다.

## 2. Assigned Scope

문서 authority, 버전·Phase·완료 주장, 기능 계약과 실제 소비 경로의 양방향 정합성을 조사했다. 다음 표면을 함께 대조했다.

- 통제/표준 문서: `AGENTS.md`, `AI_AUDIT_DOC_STANDARD.md`, `AI_IMPLEMENTATION_DOC_STANDARD.md`, `AI_CODING_STANDARD.md`
- 제품/설계/운영 문서: `spec.md`, `designs.md`, `IMPLEMENTATION_SUMMARY.md`, `DESIGN_DECISIONS.md`, `README.md`, `BUILD_GUIDE.md`, `CHANGELOG.md`, `LESSONS_LEARNED.md`, `audit_roadmap.md`, `redesign_plan.md`
- 역사 감사 기록: 루트 `audit_report_1.md`부터 `audit_report_9.md`까지. 현재 상태의 권위로 승격하지 않고 당시 주장과 현재 증거의 대조 자료로만 사용했다.
- 매니페스트·빌드·CI: `Cargo.toml`, `Cargo.lock`, `build.rs`, `build.sh`, `scripts/check-version-sync.sh`, `scripts/mock_mcp_server.py`, `.cargo/config.toml`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`
- 런타임 구현: `src/main.rs`와 `src/app`, `src/domain`, `src/infra`, `src/providers`, `src/tools`, `src/tui`, `src/types`, `src/commands` 전체 파일 인벤토리 및 계약 관련 호출 경로
- 테스트: `src/tests`와 모듈 내부 테스트의 선언·이름·주요 assertion 및 실행 결과

## 3. Excluded and Uninspected Scope

- `.git/`, `target/`, 캐시와 생성 빌드 산출물은 검사하지 않았다.
- 외부 유료/API 호출, 실제 LLM endpoint 호출, `build.sh` 실행, 네트워크 접근은 하지 않았다.
- `cargo fmt` 수정 모드와 소스 변경을 유발할 수 있는 포매터는 실행하지 않았다.
- `docs/multi_audit/1`의 동료 보고서와 `audit_run.json`은 읽지 않았다. 본 보고서 파일만 새로 작성한다.
- `stitch_modern_tui_redesign`의 PNG/HTML은 제품 런타임이 아닌 참조 자산으로 분류하고 파일 목록·저장소 추적 여부만 확인했다. 실제 화면 픽셀 검증은 하지 않았다.
- `.agents`는 지원/참조 트리로 분류하고 파일 목록·추적된 생성 파일 여부만 확인했다. skill 데이터의 동작 검증은 하지 않았다.
- Windows 실제 빌드/실행, 물리 TUI와 화면 캡처, `cargo audit`/`cargo deny`는 실행하지 않았다. 이 미실행 영역은 PASS 증거로 사용하지 않았다.

## 4. Evidence Examined

### 4.1 Inventory and read-only commands

- `rg --files -g '!target/**' -g '!.git/**' -g '!docs/multi_audit/1/**'`: 루트 문서, `src/` 구현·테스트, CI, 스크립트, 참조 트리 인벤토리.
- `git status --short --untracked-files=all`, `git log -8 --oneline --decorate`, `git diff --stat`, `git diff --check`.
- `cargo metadata --no-deps --format-version 1 --locked`: package `smlcli` `3.9.0`, edition `2024`, binary target `src/main.rs` 확인.
- `./scripts/check-version-sync.sh`: `Cargo.toml 버전: 3.9.0`, `CHANGELOG.md 최신 버전: 3.9.0`, 종료 성공. 이 스크립트가 `spec.md`, ADR, roadmap, `AGENTS.md`, `build.sh`를 검사하지 않는 것도 확인했다.
- `CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo test --all-targets --no-fail-fast --locked --offline`: `running 121 tests`, `119 passed; 2 failed`. 실패는 `test_bwrap_mounts_workspace_as_canonical_guest_root` (`src/tests/audit_regression.rs:1715`, user namespace 권한 부족)와 `test_session_harness_record_is_skipped_on_restore` (`src/tests/audit_regression.rs:1856`, read-only home filesystem)이다. 소스 트리에는 변경이 없었다.
- `git diff --check`: 현재 사용자 변경 `AGENTS.md:4-6`의 Markdown trailing spaces를 보고했다.
- `rg -n '^\\s*#\\[test\\]' src`: 정적 test 선언 101개를 확인했다. 실제 실행 테스트 수 121개와 다른 숫자를 문서의 고정 수치로 사용하지 않았다.

### 4.2 핵심 문서·구현 근거

- 프로젝트 정체성: `AGENTS.md:4-6,25-30`, `spec.md:1,20-31`, `Cargo.toml:1-3`, `README.md:1`.
- 최신 Phase/버전: `CHANGELOG.md:7-20`, `spec.md:3001-3098`, `audit_roadmap.md:636-723`, `DESIGN_DECISIONS.md:1354-1416`, `src/app/state.rs:512-557`.
- i18n 계약/소비: `redesign_plan.md:22,301-313`, `CHANGELOG.md:18-20`, `IMPLEMENTATION_SUMMARY.md:95-104`, `src/tui/i18n.rs:19-215`, `src/tui/widgets/questionnaire.rs:67-170`, `src/tui/layout.rs:429,879-913`, `src/tui/widgets/setting_wizard.rs:25-176`, `src/tui/widgets/config_dashboard.rs:48-173`, `src/main.rs:90-136`, `src/infra/doctor.rs:75-193`.
- Provider 계약: `designs.md:466-500`, `spec.md:2294-2347,2800-2833`, `README.md:16,68,116,160,204`, `src/domain/provider.rs:3-42`, `src/tui/widgets/setting_wizard.rs:27-38`, `src/tui/widgets/config_dashboard.rs:118-132`.
- `/undo`와 모듈 책임: `spec.md:2271-2290`, `IMPLEMENTATION_SUMMARY.md:947-960`, `src/commands/mod.rs:1`, `src/types/mod.rs:1`, `src/app/command_router.rs:387-394,770-808`, `src/infra/git_engine.rs:100-151`.
- 설계/shortcut drift: `designs.md:146-149,1206-1210,1109-1142,1248-1254`, `redesign_plan.md:108-126,342-347`, `audit_roadmap.md:588-632`, `src/app/state.rs:8-15`, `src/app/mod.rs:2133-2141,2451-2475`.
- Phase 54 연결: `spec.md:3001-3098`, `DESIGN_DECISIONS.md:1387-1416`, `IMPLEMENTATION_SUMMARY.md:83-93`, `src/infra/workspace_harness.rs:162-243`, `src/app/tool_runtime.rs:93-127,489-539`, `src/app/state.rs:165-185`.
- Build/release gates: `BUILD_GUIDE.md:20-30`, `.github/workflows/ci.yml:7-60`, `.github/workflows/release.yml:7-88`, `scripts/check-version-sync.sh:8-54`.
- Path/case drift: `spec.md:98-108,2875-2901`, `AI_IMPLEMENTATION_DOC_STANDARD.md:29-43`, `AGENTS.md:12.2`, `src/infra/config_store.rs:12-21`, `README.md:17,69`, `IMPLEMENTATION_SUMMARY.md:890`.
- ADR authority: `DESIGN_DECISIONS.md:262-290,371-439,501-516`, `IMPLEMENTATION_SUMMARY.md:249,409`, `AI_IMPLEMENTATION_DOC_STANDARD.md:271-285`.
- Orphan/support hygiene: `src/domain/provider.rs:26-34`, `src/domain/session.rs:24-30`, `.gitignore:1`, `git ls-files` 결과의 `.agents/**/__pycache__/*.pyc`, `scratch.py`, `scratch2.py`, `.antigravitycli/*`, `.gemini/settings.json`, `stitch_modern_tui_redesign/**`.

### 4.3 Inspected file inventory summary

루트 Markdown 23개(`AGENTS.md`, 두 AI 표준, `BUILD_GUIDE.md`, `CHANGELOG.md`, `DESIGN_DECISIONS.md`, `IMPLEMENTATION_SUMMARY.md`, `LESSONS_LEARNED.md`, `README.md`, `audit_report_1..9.md`, `audit_roadmap.md`, `designs.md`, `redesign_plan.md`, `spec.md`)와 참조 `stitch_modern_tui_redesign/terminal_precision/DESIGN.md`를 인벤토리했다. `src/`의 app/domain/infra/providers/tools/tui/tests 및 매니페스트·스크립트·CI를 대상으로 문서에 등장한 계약과 실제 심볼/호출 경로를 검색했다.

## 5. Findings

### [A01-F001] 프로젝트 정체성 authority가 `doomlike 0.5.1`과 `smlcli 3.9.x`로 충돌한다

- Pass: Implementation
- Pattern: `IMP-004`, `SPEC-GAP-001`
- Area: 프로젝트 identity, 문서 authority, 감사 범위
- Severity: Major
- Status: Needs Clarification
- Summary: 현재 사용자 변경 `AGENTS.md`는 프로젝트를 `doomlike` `0.5.1`로 선언하지만, `Cargo.toml`, `cargo metadata`, `spec.md`, README, 런타임 binary는 모두 `smlcli`를 기준으로 한다. 어느 문서가 이 저장소의 프로젝트명·버전 authority인지 현재 트리만으로 닫히지 않는다.
- Evidence: `AGENTS.md:4-6,25-30`의 `doomlike/0.5.1`; `Cargo.toml:1-3`의 `smlcli/3.9.0`; `spec.md:1,20-31`의 `smlcli v3.9.0`; `cargo metadata --no-deps --format-version 1 --locked` 결과의 package/id `smlcli 3.9.0`; `src/main.rs:25-30,112` 및 `README.md:1,12`의 `smlcli`.
- Expected Basis: `AGENTS.md:9-19,23-32,198-200`의 authority 우선순위와 프로젝트 메타데이터 규칙, `AI_AUDIT_DOC_STANDARD.md:178-181`의 범위 인벤토리 규칙. 사용자 감사 지시가 이 충돌을 미확정으로 명시했다.
- Expected: 프로젝트 identity와 현재 release authority가 하나로 선언되거나, `AGENTS.md`가 외부 템플릿/미확정 메타데이터임을 명시해야 한다.
- Actual: `AGENTS.md`에는 가정/미확정 표기가 없고, version-sync 스크립트는 `AGENTS.md`를 검사하지 않는다.
- Impact: 잘못된 프로젝트를 대상으로 문서·코드·릴리스 범위를 판단할 수 있고, 이후 finding의 owner와 version gate가 흔들린다.
- Suggested Fix: 사람/아키텍트가 `AGENTS.md`의 프로젝트 identity를 `smlcli`로 확정할지 외부 운영 문서로 분리할지 결정하고, 모든 문서·CI authority를 그 결정에 맞춰 동기화한다. 현재 보고서는 사용자 변경을 수정하지 않았다.
- Re-audit Method: `AGENTS.md`, `spec.md`, `Cargo.toml`, `cargo metadata`, README의 name/version 검색 및 authority 표를 다시 대조하고, version-sync에 포함할 파일을 명시적으로 확인한다.
- Owner: Human / Architect
- Confidence: High
- Notes: AI 표준상 `Needs Spec Clarification` 분류다. `AGENTS.md`의 사용자 기존 변경 자체를 오류라고 단정하지 않고 authority 미확정으로 보고한다.

### [A01-F002] package `3.9.0`과 완료된 Phase 53/54 `v3.9.1/v3.9.2`의 release 경계가 닫히지 않았다

- Pass: Implementation
- Pattern: `IMP-003`, `IMP-004`, `BUILD-001`
- Area: versioning, Phase gate, release scope
- Severity: Major
- Status: Needs Clarification
- Summary: Cargo와 Changelog의 latest released header는 `3.9.0`이지만, 같은 트리의 `spec.md`/roadmap/ADR/구현 요약은 Phase 53과 Phase 54를 `v3.9.1`/`v3.9.2`로 구현 완료로 기록한다. `Unreleased` 표기는 일부 변경에만 있어 현재 binary가 어느 Phase까지 포함하는지 release authority가 불명확하다.
- Evidence: `Cargo.toml:3` 및 metadata 결과 `3.9.0`; `spec.md:1,3001-3004`의 v3.9.0 문서 제목과 Phase 54 `v3.9.2`; `audit_roadmap.md:636,681`의 Phase 53/54; `DESIGN_DECISIONS.md:1354-1357,1387-1393`의 ADR-039/040 `Implemented`; `IMPLEMENTATION_SUMMARY.md:83-93`의 Phase 54 구현 완료; `CHANGELOG.md:7-20`의 `[Unreleased]`; `src/app/state.rs:512`의 v3.9.1 주석; `build.sh:12`의 낡은 v0.1.0 banner.
- Expected Basis: `AI_AUDIT_DOC_STANDARD.md:245-263,280-293`의 Phase gate·완료 주장 규칙, `AI_IMPLEMENTATION_DOC_STANDARD.md:117-126`의 검증 결과 기반 완료 규칙.
- Expected: `3.9.1/3.9.2`가 미출시 Phase라면 문서에 명확히 `Unreleased` 범위를 닫고, 출시 포함이면 package/README/Changelog/태그와 동기화해야 한다.
- Actual: `scripts/check-version-sync.sh:8-41`은 Cargo와 첫 Changelog release header만 검사하므로 spec/roadmap/ADR/build banner의 drift를 통과시킨다. 현재 스크립트는 성공했지만 Phase 경계를 증명하지 않는다.
- Impact: 3.9.0 binary에 포함되지 않은 기능을 shipped 완료로 홍보하거나, 반대로 다음 release 범위를 잘못 감사할 수 있다.
- Suggested Fix: release authority를 명시적으로 결정하고, Phase 53/54를 `Unreleased` 하위 목록으로 묶거나 package version을 승격한다. version-sync/CI가 결정된 authority 집합을 검사하도록 확장한다.
- Re-audit Method: package version, latest Changelog header, Unreleased 목록, spec/roadmap/ADR Phase 상태, `build.sh` banner를 같은 표로 재검증하고 tag release workflow에서 실행되는지 확인한다.
- Owner: Human / Architect
- Confidence: High
- Notes: 사용자가 알려준 `spec/Phase54 3.9.2 vs Cargo/README 3.9.0` 미확정 사항을 증거로 확인했다. 결론은 release 정책 결정 후 확정해야 한다.

### [A01-F003] 5개 언어 “완전 연동” 완료 주장이 실제 UI 소비 범위를 초과한다

- Pass: Implementation
- Pattern: `IMP-001`, `IMP-003`
- Area: i18n, user-facing UI, completion evidence
- Severity: Major
- Status: Confirmed
- Summary: 사전의 5개 locale key completeness는 구현됐지만, 제품 전체의 정적 UI/CLI 문자열이 `I18nManager`를 소비하지 않는다. 문서는 모든 정적 라벨이 다국어라고 읽히는 반면 실제로는 questionnaire, 상태 badge/탭 일부만 `tr()`을 사용한다.
- Evidence: `redesign_plan.md:22`는 “TUI 내 노출되는 모든 정적 라벨”을 다국어 사전에 연동한다고 한다. `CHANGELOG.md:18-20` 및 `IMPLEMENTATION_SUMMARY.md:95-104`는 5대 언어 실연결/완전 매핑을 완료로 주장한다. 실제 `src/tui/i18n.rs:19-215`는 정적 사전을 제공하지만 호출은 `src/tui/widgets/questionnaire.rs:67-170`과 `src/tui/layout.rs:429,879-913` 중심이다. `src/tui/widgets/setting_wizard.rs:25-176`, `src/tui/widgets/config_dashboard.rs:48-173`, `src/main.rs:90-136`, `src/infra/doctor.rs:75-193`, `src/app/command_router.rs`에는 직접 영어/한국어 UI 문자열이 남아 있다. `src/tests/audit_regression.rs:4496-4538`의 `test_v3_9_0_i18n_key_completeness`는 사전 간 key 대칭만 검증한다.
- Expected Basis: `AGENTS.md:6.4,8.1`의 언어 정책과 `redesign_plan.md:22,301-313`; `AI_AUDIT_DOC_STANDARD.md:272-293`의 완료 주장에 결정적 검증 기준 요구.
- Expected: 제품 범위를 “questionnaire/status/tabs 부분 번역”으로 좁히거나, 선언한 모든 사용자 노출 문자열을 locale key와 소비 경로로 닫고 coverage 테스트를 제공해야 한다.
- Actual: locale 사전 key completeness가 전체 UI 번역 완료의 대리 증거로 사용되고, renderer/CLI coverage는 검증되지 않는다.
- Impact: 비영어 설정에서 wizard, doctor, command, error 메시지가 혼재하며, README/Changelog의 다국어 완료 주장이 실제 UX를 과대대표한다.
- Suggested Fix: 지원 범위를 문서에 명시적으로 분리하고, 전체 번역을 유지하려면 사용자 노출 문자열을 단일 i18n source로 이동한 뒤 locale별 renderer/CLI 회귀 테스트를 추가한다.
- Re-audit Method: `rg`로 user-facing literal과 `tr()` 호출을 전수 분류하고, 각 locale에서 wizard/config/doctor/help/error 화면을 headless fixture 또는 수동 TUI로 확인한다.
- Owner: Coder / Architect
- Confidence: High
- Notes: AI 표준 분류는 요구사항을 그대로 유지할 경우 `Needs Fix`다. 이 finding은 번역 품질(자연스러움)이 아니라 계약 소비 경로의 부재를 지적한다.

### [A01-F004] Ollama가 내장 Provider인지 Custom Provider인지 문서와 구현이 합의되지 않았다

- Pass: Implementation
- Pattern: `IMP-001`, `SPEC-GAP-001`
- Area: provider contract, setup wizard, public feature scope
- Severity: Major
- Status: Needs Clarification
- Summary: `designs.md`와 Phase 48-A 예시 타입은 `Ollama`를 지원 항목/enum variant로 제시하지만, 실제 `ProviderKind`와 wizard/config 메뉴에는 Ollama가 없다. 다른 문서는 Ollama를 `Custom(String)`으로 지원한다고 설명하고 README는 6개 provider만 공개한다.
- Evidence: `designs.md:466-498` wireframe와 지원 목록의 `Ollama`; `spec.md:2812-2824` typed contract의 `Ollama`; 실제 `src/domain/provider.rs:3-13`에는 `OpenAI, Anthropic, Xai, OpenRouter, Google, LmStudio, Custom(String)`만 존재; `src/tui/widgets/setting_wizard.rs:31-38` 및 `src/tui/widgets/config_dashboard.rs:120-132` 목록에도 Ollama 없음; `spec.md:2294-2299`는 Ollama/Ollama 호환 endpoint를 Custom Provider로 설명; `README.md:16,68,116,160,204`는 Ollama를 공개 provider 목록에 넣지 않는다.
- Expected Basis: `AI_AUDIT_DOC_STANDARD.md:272-293,321-342`의 문서-도메인/UI 분리 및 Spec Gap 규칙.
- Expected: Ollama를 내장 provider로 할지 `Custom` 등록 경로로만 제공할지 하나를 동결하고 enum, wizard, docs, tests가 같은 계약을 사용해야 한다.
- Actual: 사용자-facing 설계와 typed example은 내장 지원처럼 보이지만, runtime/UI는 Custom-only 경로다.
- Impact: 사용자는 wizard에서 Ollama를 찾지 못하고, 구현자는 없는 enum을 기준으로 작업할 수 있다. provider 지원 완료 판정도 재현할 수 없다.
- Suggested Fix: 제품 결정 후 한 경로를 canonical로 선언한다. Custom-only라면 designs/spec의 내장 Ollama 항목을 “Custom Provider 예시”로 바꾸고 CRUD/fetch/chat 테스트를 연결한다.
- Re-audit Method: `ProviderKind`, provider registry factory, wizard/config menu, `/provider add` help, README와 provider tests를 한 mapping 표로 재검증한다.
- Owner: Human / Architect
- Confidence: High
- Notes: 현재 증거만으로 내장 추가 요구를 창작하지 않고 `Needs Spec Clarification`으로 남긴다.

### [A01-F005] `/undo` 구현 책임 파일과 빈 `commands` 모듈이 문서 계약과 어긋난다

- Pass: Implementation
- Pattern: `IMP-003`, `IMP-004`, `DOC-BACKFILL-001`
- Area: file responsibility, slash command routing, orphan module
- Severity: Major
- Status: Confirmed
- Summary: Git `/undo` 기능 자체는 `app/command_router.rs`에 구현되어 있지만, `spec.md`와 `IMPLEMENTATION_SUMMARY.md`는 `commands/mod.rs`가 라우팅한다고 완료 선언한다. 실제 `src/commands/mod.rs`와 `src/types/mod.rs`는 빈 모듈이며, `main.rs`가 이를 모듈로 포함할 뿐 소비 경로가 없다.
- Evidence: `spec.md:2271-2273`와 `IMPLEMENTATION_SUMMARY.md:957-958`의 `commands/mod.rs` 책임 주장; `src/commands/mod.rs:1`, `src/types/mod.rs:1`은 내용이 없음; 실제 command catalog는 `src/app/command_router.rs:387-394`, `/undo` handler는 `:770-808`, `GitEngine::undo_last`는 `src/infra/git_engine.rs:100-151`; `rg -n "crate::commands|commands::|crate::types|types::" src`에서 해당 모듈 소비 호출이 없다. Git history에서도 `src/commands/mod.rs`는 빈 파일로 생성된 뒤 채워지지 않았다.
- Expected Basis: `AI_IMPLEMENTATION_DOC_STANDARD.md:182-198,257-269`의 파일 책임/교차 문서 closure, `AI_AUDIT_DOC_STANDARD.md:272-293,631-660`의 orphan 분류 규칙.
- Expected: 완료 기능의 실제 owner 파일과 호출 경로가 문서에 기록되고, 빈 scaffold는 의도적 deferred 범위와 추적 ID를 가져야 한다.
- Actual: 문서가 존재하지 않는 routing owner를 완료로 기록하고, 빈 모듈은 현재 제품 구조에 포함되어 있다.
- Impact: 다음 코더가 잘못된 파일을 수정하며, commands/types 레이어가 실제 경계인지 오판한다. `/undo`의 구현 누락과 문서 누락을 구분하기 어렵다.
- Suggested Fix: `/undo` 책임을 `app/command_router.rs`로 복구하거나 승인된 아키텍처로 실제 모듈을 연결한다. 빈 모듈은 삭제/유지 결정을 문서화하고, file-responsibility 표와 route tests를 갱신한다.
- Re-audit Method: slash command catalog→router→GitEngine 호출 그래프와 문서 책임표를 다시 대조하고, 빈 모듈의 참조가 0인지 확인한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: runtime 기능 부재가 아니라 “생성됐으나 소비되지 않는 계약/상태”와 잘못된 파일 책임을 보고한다.

### [A01-F006] 구식 설계/로드맵의 Phase·shortcut 계약이 현재 코드와 역사 감사 주장에 남아 있다

- Pass: Implementation
- Pattern: `IMP-004`, `IMP-003`
- Area: stale secondary docs, TUI interaction contract, audit reproducibility
- Severity: Major
- Status: Confirmed
- Summary: 현재 코드와 `designs.md`는 Inspector 포커스에서 `Tab/Shift+Tab`으로 6개 탭을 순환하지만, `redesign_plan.md`와 `audit_roadmap.md`는 `Alt+1..5/6`을 완료 기준으로 남긴다. `designs.md`와 `IMPLEMENTATION_SUMMARY.md`는 Phase 13/15의 상태도 각각 “진행 예정/계획”과 “완료”로 다르게 기록한다.
- Evidence: 현재 interaction은 `designs.md:146-149,1206-1210`, `src/app/mod.rs:2451-2475`; 현재 enum/renderer는 `src/app/state.rs:8-15`, `src/tui/layout.rs:836-842,1005-1021`의 `Preview, Diff, Search, Logs, Recent, Git`이다. 구식 계약은 `redesign_plan.md:108-126,342-347`의 5탭/Alt+1..5, `audit_roadmap.md:596,631`의 Alt+1..6이다. 상태 충돌은 `designs.md:1109-1142,1248-1254`의 진행 예정/계획과 `IMPLEMENTATION_SUMMARY.md:524-648`의 완료 선언이다. `audit_report_5.md:81`, `audit_report_6.md:83`, `audit_report_9.md:34-45`도 현재 shortcut/Phase를 독립 재검증하지 않고 PASS 근거로 사용했다.
- Expected Basis: `AI_AUDIT_DOC_STANDARD.md:631-679`의 secondary document 독립 감사와 `IMP-003/IMP-004`; `AI_IMPLEMENTATION_DOC_STANDARD.md:257-269`의 cross-document closure.
- Expected: 현재 동작을 검증하는 canonical design/roadmap을 하나로 정하고, 과거 계획은 snapshot/obsolete 표기를 가져야 한다.
- Actual: 동일 저장소에서 코더가 서로 다른 키맵·Phase 상태를 읽을 수 있으며, audit checklist가 실제 입력 경로를 검사하지 않는다.
- Impact: 잘못된 수동 검증을 통과시키거나 이미 완료된 기능을 중복 구현한다. PASS 보고서의 재현성과 다음 작업의 안전성이 저하된다.
- Suggested Fix: `redesign_plan.md`와 roadmap의 현재 기준을 Tab/Shift+Tab·6탭으로 갱신하거나 historical snapshot으로 명시하고, Phase 13/15의 완료/잔여를 한 authority에 반영한다. 역사 보고서는 덮어쓰지 말고 정정 supplement로 연결한다.
- Re-audit Method: `Alt+`, `Phase 13`, `Phase 15`, 탭 이름을 전 문서에서 검색하고 현재 `InspectorTab`·key routing·tests와 대조한다.
- Owner: Architect / Auditor
- Confidence: High
- Notes: 기존 audit_report_7의 `Settings` 탭 Accepted Risk는 audit_report_8에서 “밀려나 해소”로 처리됐지만, shortcut/Phase drift 전체가 해소됐다는 근거는 없다.

### [A01-F007] Phase 54의 snapshot drift 검사와 세션 기록은 모든 경로에서 보장되지 않는다

- Pass: Implementation
- Pattern: `IMP-001`, `IMP-003`, `SEC-005`
- Area: Workspace Harness contract, preflight, session audit
- Severity: Major
- Status: Confirmed
- Summary: Phase 54 문서/ADR/Changelog는 도구 실행 직전 5개 snapshot 필드의 drift를 검사하고 세션 시작 시 snapshot을 항상 기록한다고 완료 선언한다. 실제 preflight는 매번 새 snapshot을 수집해 현재 값만 검사하고 이전/기대 snapshot과 비교하지 않으며, workspace-session 생성 실패 fallback에서는 harness record를 기록하지 않는다.
- Evidence: `spec.md:3005-3009,3048-3052,3073-3078`은 drift 검사와 항상 기록을 요구; `DESIGN_DECISIONS.md:1396-1403`도 같은 결정; `src/infra/workspace_harness.rs:162-193`의 `HarnessPreflightInput::from_tool_call`은 현재 settings로 snapshot을 새로 수집하고 baseline/expected snapshot 필드가 없다; `src/infra/workspace_harness.rs:205-243`의 `evaluate_preflight`에는 `canonical_root`, `trust_state`, `denied`, `sandbox_enabled`, `sandbox_guest_root` 간 비교가 없고 `rg -n -i "drift" src`에도 구현 심볼이 없다. `src/app/state.rs:165-185`는 `new_workspace_session` 실패 시 `SessionLogger::new_session()`만 생성하고 `append_harness_record`를 호출하지 않는다.
- Expected Basis: Phase 54 success criteria와 typed contract, `AI_AUDIT_DOC_STANDARD.md:272-293`의 생성·호출·후속 효과 검증 규칙.
- Expected: 실행 시점 snapshot을 세션/작업 기준과 비교해 drift를 명시적으로 처리하고, 모든 session-start fallback에서도 기록 실패를 사용자/진단 surface에 남겨야 한다.
- Actual: current-state validation과 baseline drift detection이 혼동되어 있으며, logger fallback은 Phase 54 audit record를 생략한다.
- Impact: 실행 중 workspace/trust/sandbox 변경을 “drift”로 판정할 수 없고, 일부 세션에는 사후 환경 재구성용 증거가 없다. `Implemented`/PASS 선언이 실제 보장 범위를 넘는다.
- Suggested Fix: baseline snapshot의 source/수명/비교 규칙을 동결하고 preflight 입력에 expected snapshot 또는 generation을 연결한다. logger fallback은 harness record 기록 실패를 명시적으로 처리하거나 세션 생성을 차단하는 정책을 정한다. 해당 실패 경로 회귀 테스트를 추가한다.
- Re-audit Method: baseline과 현재 snapshot을 다르게 만든 뒤 각 5개 필드별 decision을 확인하고, `new_workspace_session` 실패 fixture에서 JSONL 첫 레코드·오류 표출을 검증한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: 이 finding은 보안 pass와 연결될 수 있으나, 본 보고서에서는 문서의 완료 계약과 실제 소비 경로 불일치로 판정했다.

### [A01-F008] 문서화된 `cargo audit`/version-sync release gate가 실제 CI·Release workflow에 연결되지 않는다

- Pass: Debug / Implementation
- Pattern: `BUILD-001`, `DEP-001`, `IMP-003`
- Area: CI/CD, supply-chain gate, release reproducibility
- Severity: Major
- Status: Confirmed
- Summary: `BUILD_GUIDE.md`는 merge 전 `cargo audit`를 포함한 품질 게이트가 항상 통과해야 한다고 쓰지만, CI는 fmt/clippy/test와 별도 version-sync만 실행하고 `cargo audit`를 실행하지 않는다. Release workflow는 tag에서 다시 fmt/clippy/test만 수행하며 version-sync도 audit도 실행하지 않는다.
- Evidence: `BUILD_GUIDE.md:20-30`의 required commands `cargo fmt --check`, clippy, test, audit; `.github/workflows/ci.yml:17-48`에는 fmt/clippy/test만 있고 `:50-60` version-sync job만 추가; `.github/workflows/release.yml:7-9,17-29`의 tag/quality gate에도 audit/version-sync 없음; `scripts/check-version-sync.sh:33-41`의 tag 검증 코드는 release workflow에서 호출되지 않는다. `CHANGELOG.md:83-85`와 `IMPLEMENTATION_SUMMARY.md:1045-1055`는 CI/CD/version-sync 완료를 주장한다.
- Expected Basis: `AI_AUDIT_DOC_STANDARD.md:314-342,683-713`의 build/dependency/supply-chain gate 규칙과 `BUILD_GUIDE.md`의 사용자 계약.
- Expected: 어떤 gate가 merge-only/local-only/release-only인지 문서와 workflow가 동일해야 하며, release tag도 version·dependency gate의 명시적 경로를 가져야 한다.
- Actual: local guide의 `cargo audit`는 CI/release에 enforcement되지 않고, release tag는 version-sync를 우회한다. `cargo audit` 자체는 외부 advisory DB 접근 금지로 이번 감사에서 실행하지 않았다.
- Impact: 문서상 security gate를 통과했다고 믿어도 실제 release artifact가 동일 gate를 통과했다는 증거가 없다. version/manifest drift와 dependency advisory가 release에서 발견될 수 있다.
- Suggested Fix: canonical gate 목록을 정한 뒤 CI와 release job에서 같은 명령/lock/DB provenance를 실행하거나, local-only임을 명시하고 “always before merge/release” 주장을 제거한다.
- Re-audit Method: GitHub workflow event별 실행 job을 표로 만들고 tag push에서 version-sync·audit가 실제 실행되는지 확인한 뒤, offline/CI 조건의 명령 결과를 기록한다.
- Owner: Coder / Release Owner
- Confidence: High
- Notes: 외부 네트워크·유료 호출 없이 workflow 정적 대조만 수행했다.

### [A01-F009] 최신 트리의 121-test PASS 주장은 현재 허용 환경에서 재현되지 않는다

- Pass: Debug / Implementation
- Pattern: `TEST-001`, `BUILD-001`, `IMP-003`
- Area: completion evidence, test isolation, historical PASS claims
- Severity: Major
- Status: Confirmed
- Summary: 루트 audit_report_8/9와 일부 최신 요약은 121개 테스트 100% PASS를 현재 상태처럼 제시하지만, 지정된 읽기 중심 조건에서 실행한 동일 all-targets suite는 121개 중 119개만 통과하고 2개가 실패했다. 실패 중 하나는 bwrap user namespace 권한, 다른 하나는 home 기반 session log의 read-only filesystem 의존성을 드러낸다.
- Evidence: `audit_report_8.md:21,55,97,105`와 `audit_report_9.md:55-59,101-109`의 121 PASS 주장; `IMPLEMENTATION_SUMMARY.md:104`의 115 PASS 주장; `spec.md:3080-3098` 및 `audit_roadmap.md:692-723`의 Phase 54 full-test gate. 실행 명령 `CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo test --all-targets --no-fail-fast --locked --offline` 결과 `running 121 tests`, `119 passed; 2 failed`: `src/tests/audit_regression.rs:1715` `test_bwrap_mounts_workspace_as_canonical_guest_root`는 “No permissions to create a new namespace”, `src/tests/audit_regression.rs:1856` `test_session_harness_record_is_skipped_on_restore`는 `InfraError("세션 로그 파일 열기 실패: Read-only file system")`이다.
- Expected Basis: `AI_AUDIT_DOC_STANDARD.md:117-126,245-263,631-679`의 실행 증거·재감사 규칙과 `AI_CODING_STANDARD.md:45-63`의 미실행 검증 기록 규칙.
- Expected: PASS 수치는 실행 시점·환경·명령과 함께 snapshot으로 남고, 환경 의존 테스트는 명확한 prerequisite 또는 deterministic seam을 가져야 한다.
- Actual: historical PASS 문서에 환경 조건/현재 유효성 표기가 없고, test가 `/home`/bwrap 권한에 직접 의존해 본 감사 환경에서 재현되지 않았다.
- Impact: release readiness와 Phase gate를 잘못 PASS로 해석할 수 있다. 실패가 제품 결함인지 환경 제한인지 분류할 증거도 현재 문서에 부족하다.
- Suggested Fix: test를 temp home/log-dir 주입으로 격리하고 bwrap user namespace prerequisite를 CI job에 명시한다. historical report는 당시 snapshot으로 라벨링하고 현재 gate 결과를 별도 기록한다.
- Re-audit Method: `HOME`/config/session path를 임시 fixture로 격리한 실행과 bwrap 권한이 보장된 CI/host 실행을 각각 수행하고 121/121 결과 및 환경 경고를 기록한다.
- Owner: Coder / CI Owner / Auditor
- Confidence: High
- Notes: 현재 소스/문서를 수정하지 않고 실행 결과만 기록했다. 외부 API는 호출하지 않았다.

### [A01-F010] 현재 사용자 변경이 `git diff --check`를 막지만 의도/허용 여부가 문서화되지 않았다

- Pass: Debug / Implementation
- Pattern: `BUILD-001`, `IMP-003`
- Area: working-tree hygiene, protected control document
- Severity: Minor
- Status: Needs Clarification
- Summary: 현재 사용자 수정 `AGENTS.md`의 Markdown hard-break trailing spaces가 `git diff --check`에서 오류로 보고된다. 과거 audit_report_5/6/7/9는 `git diff --check` PASS를 주장하지만 그 시점과 현재 사용자 변경을 구분하지 않는다.
- Evidence: `git diff --check` 결과 `AGENTS.md:4`, `AGENTS.md:5`, `AGENTS.md:6` trailing whitespace; `AGENTS.md:4-6`은 `**Template Version:** 1.3  ` 등 사용자 기존 변경; `audit_report_5.md:206-215`, `audit_report_6.md:282-295`, `audit_report_7.md:133-139`의 과거 diff-check PASS 주장.
- Expected Basis: `AI_AUDIT_DOC_STANDARD.md:117-126,245-263`의 실제 실행 증거 규칙 및 `AGENTS.md:6.3,11`의 문서 encoding/quality 정책.
- Expected: 의도한 Markdown 줄바꿈이면 gate 정책에서 허용 사유를 기록하거나, current working tree gate를 통과하는 표현으로 문서 형식을 정해야 한다.
- Actual: 현재 tree의 diff-check는 실패하며, 이 변경이 보호 문서의 의도적 서식인지 단순 공백인지 authority 문서에 없다.
- Impact: scoped commit/release gate가 차단되거나, 감사자가 오래된 PASS를 현재 결과로 오인한다.
- Suggested Fix: 사용자/문서 owner가 hard-break 서식과 diff-check 정책을 결정한 뒤 current-tree evidence와 historical evidence를 분리한다. 이 보고서는 `AGENTS.md`를 수정하지 않았다.
- Re-audit Method: owner 결정 후 동일 `git diff --check`와 변경 파일 scope 검사를 재실행하고, historical report는 snapshot date를 붙여 재참조한다.
- Owner: Human / Documentation Owner
- Confidence: High
- Notes: protected user change라서 자동 수정하지 않았다.

### [A01-F011] 설정 파일 경로와 구현 요약 파일명에 case/path drift가 남아 있다

- Pass: Implementation
- Pattern: `IMP-001`, `BUILD-001`, `DOC-BACKFILL-001`
- Area: persistence path, documentation links, case-sensitive tooling
- Severity: Major
- Status: Confirmed
- Summary: canonical runtime path는 `~/.smlcli/config.toml`인데 spec의 실데이터 sample은 `config.json`, 구현 요약의 과거 section은 `settings.json`으로 남아 있다. 또한 표준/spec이 `implementation_summary.md` 소문자 이름을 기준으로 하지만 실제 파일은 `IMPLEMENTATION_SUMMARY.md`다.
- Evidence: runtime `src/infra/config_store.rs:12-21`의 `~/.smlcli/config.toml`; `README.md:17,69`와 `CHANGELOG.md:53`의 TOML; `spec.md:2875`의 `~/.smlcli/config.json`; `IMPLEMENTATION_SUMMARY.md:890`의 `settings.json`; `spec.md:98-108` 및 `AI_IMPLEMENTATION_DOC_STANDARD.md:29-43`의 `implementation_summary.md`; 실제 root 파일 `IMPLEMENTATION_SUMMARY.md`, `AGENTS.md:12.2`의 uppercase 이름.
- Expected Basis: `AI_IMPLEMENTATION_DOC_STANDARD.md:257-269`의 cross-document closure 및 `AI_AUDIT_DOC_STANDARD.md:314-342`의 build/runtime path 대조.
- Expected: 사용자 설정 경로와 control-document 파일명은 case-sensitive 환경에서 하나의 canonical spelling으로 닫혀야 한다.
- Actual: 문서 샘플·과거 요약·표준 이름이 서로 다른 경로를 가리키며, 일부 링크/자동화가 Linux에서 깨질 수 있다.
- Impact: 사용자가 잘못된 config 파일을 편집하거나 후속 agent가 구현 요약을 찾지 못해 설정/문서 복구가 실패할 수 있다.
- Suggested Fix: config.toml과 `IMPLEMENTATION_SUMMARY.md`를 canonical path로 선언하고 모든 sample/link/표준 템플릿을 동기화한다. case-sensitive link check를 CI에 추가한다.
- Re-audit Method: `rg -n 'config\\.(json|yaml)|settings\\.json|implementation_summary\\.md'` 결과를 canonical path 목록과 대조하고 README/spec/summary/build 명령을 따라 경로를 확인한다.
- Owner: Documentation Owner / Architect
- Confidence: High
- Notes: 과거 migration 기록 자체는 보존할 수 있으나 현재 sample/책임표처럼 읽히는 항목은 분리해야 한다.

### [A01-F012] ADR ID가 중복되고 참조되지만 정의되지 않은 번호가 있다

- Pass: Implementation
- Pattern: `IMP-004`, `DOC-BACKFILL-001`
- Area: decision authority, traceability, historical lineage
- Severity: Major
- Status: Confirmed
- Summary: `DESIGN_DECISIONS.md`에 ADR-013이 두 번 다른 결정으로 정의되고 ADR-009/ADR-014 heading은 없는데 다른 문서가 이를 참조한다. 이 상태에서는 implementation summary/spec의 “관련 ADR” 링크가 어떤 결정인지 유일하게 결정되지 않는다.
- Evidence: `DESIGN_DECISIONS.md:371-439`의 `ADR-013` 하네스 결정과 `:501-516`의 `[ADR-013]` Agentic Autonomy 결정; heading 목록에는 ADR-009 및 ADR-014가 없고 `DESIGN_DECISIONS.md:262`부터 ADR-010으로 건너뛴다. 참조는 `IMPLEMENTATION_SUMMARY.md:249`의 `ADR-009`, `:409`의 `ADR-013`, `src/app/action.rs:6`의 `ADR-009`다. `AI_IMPLEMENTATION_DOC_STANDARD.md:271-285`는 문서 ID가 정의로 닫혀야 한다고 규정한다.
- Expected Basis: ID closure와 supersede 관계를 요구하는 `AI_IMPLEMENTATION_DOC_STANDARD.md:200-212,271-285`, `AI_AUDIT_DOC_STANDARD.md:631-679`의 secondary docs 추적 규칙.
- Expected: ADR ID는 immutable unique identifier여야 하고, 누락/중복을 historical alias 또는 supersede 표로 명시해야 한다.
- Actual: 동일 ID가 서로 다른 결정에 사용되고, 일부 참조는 정의를 찾지 못한다.
- Impact: 코더가 잘못된 보안/아키텍처 결정을 근거로 구현하며, 반복 실패의 원인과 supersede lineage를 추적할 수 없다.
- Suggested Fix: ADR 번호를 재사용하지 않고 canonical ID/alias/supersedes 표를 추가한다. 기존 역사 문서는 덮어쓰지 말고 참조를 명확한 immutable ID로 보강한다.
- Re-audit Method: 모든 `ADR-[0-9]+` 참조와 heading을 집합 비교하고, 중복/미정의/역참조 각각을 owner와 상태에 연결한다.
- Owner: Architect / Documentation Owner
- Confidence: High
- Notes: 번호 재배열은 역사 불변성을 해칠 수 있으므로 human review가 필요하다.

### [A01-F013] 완료 Phase가 존재하는데 typed contract가 호출되지 않는 orphan surface가 남아 있다

- Pass: Implementation
- Pattern: `IMP-002`, `IMP-004`, `DOC-BACKFILL-001`
- Area: orphan types, deferred scope, reverse documentation
- Severity: Minor
- Status: Confirmed
- Summary: `ProviderProfile`과 `SessionAction`은 multi-provider/session 완료 문맥에 등장하지만 현재 호출 경로가 없고 `allow(dead_code)`/예정 주석으로 남아 있다. empty `commands/types` 외에도 “정의=완료”와 “소비=실제 기능”을 혼동할 수 있는 표면이다.
- Evidence: `src/domain/provider.rs:26-34`의 `ProviderProfile`과 `#[allow(dead_code)]`; `src/domain/session.rs:24-30`의 `SessionAction`과 예정 주석; call-site 검색에서 두 심볼의 제품 runtime 소비가 없고 session command는 `src/app/command_router.rs:848-1030`가 직접 `SessionLogger`/`SessionIndex`를 호출한다. 반면 `IMPLEMENTATION_SUMMARY.md:962-971,1057-1073` 및 `spec.md:2294-2299,2629-2667`는 Phase 41/46을 완료로 기록한다.
- Expected Basis: `AI_AUDIT_DOC_STANDARD.md:272-293,631-679`의 실제 소비·후속 상태 검증 및 orphan 분류, `AI_IMPLEMENTATION_DOC_STANDARD.md:287-300`의 flag/contract 소비 위치 규칙.
- Expected: 의도적 future contract는 deferred Phase/추적 ID로 격리하고, 완료 기능에 필요한 타입은 실제 호출·테스트 경로에 연결해야 한다.
- Actual: dead code가 완료 Phase와 같은 표면에 남아 있어 의도적 선행 설계인지 잔재인지 문서가 구분하지 않는다.
- Impact: 다음 구현자가 이미 있는 typed contract를 연결해야 하는지 제거해야 하는지 판단할 수 없고, dead code 제거 감사가 반복될 수 있다.
- Suggested Fix: 각 orphan을 `Intentional but Undocumented`, `Deferred`, `Accidental/Orphan` 중 하나로 분류하고 roadmap/ADR에 연결하거나 제거 승인을 얻는다. 완료 체크리스트에 call-site와 failure-mode test를 포함한다.
- Re-audit Method: public type/enum/method별 정의-호출-테스트-문서 매핑을 생성하고 `allow(dead_code)` 잔여를 Phase 상태와 대조한다.
- Owner: Architect / Coder
- Confidence: Medium-High
- Notes: 단순히 dead code를 삭제하라는 finding이 아니다. 의도와 Phase 경계를 먼저 닫아야 한다.

### [A01-F014] reference/support tree와 생성 파일의 저장소 위생·권위가 문서화되지 않았다

- Pass: Implementation
- Pattern: `IMP-004`, `SEC-006`
- Area: generated/reference tree, repository hygiene, shipped scope
- Severity: Minor
- Status: Confirmed
- Summary: `stitch_modern_tui_redesign`와 `.agents`는 제품 runtime이 아닌 참조/지원 트리지만, 저장소에는 PNG/HTML과 tracked Python `__pycache__/*.pyc`, `scratch.py`, `scratch2.py`, `.antigravitycli/*`, `.gemini/settings.json` 등이 함께 존재한다. `.gitignore`는 `/target`만 제외하고, 어떤 항목이 release/shipped scope에서 제외되는지 문서가 없다.
- Evidence: `git ls-files` 결과에 `stitch_modern_tui_redesign/**`의 `code.html`/`screen.png`, `.agents/skills/ui-ux-pro-max/scripts/__pycache__/*.pyc`, `scratch.py`, `scratch2.py`, `.antigravitycli/a64b...json`, `.gemini/settings.json`이 포함; `.gitignore:1`은 `/target`만 명시; 제품 문서에는 해당 support/reference ownership 및 package exclusion 표가 없다. `redesign_plan.md:15`만 stitch를 설계 reference로 언급한다.
- Expected Basis: `AI_AUDIT_DOC_STANDARD.md:178-181`의 generated/vendor/reference-only scope 기록과 `SEC-006`의 shipped scope/provenance 규칙.
- Expected: reference/support/generated 파일의 소유·보존 이유·shipped 여부와 secret/metadata 취급을 문서화하고, 재생성 파일은 ignore 또는 생성 원본을 canonical로 둬야 한다.
- Actual: runtime source와 참조/생성 파일이 같은 저장소 namespace에 있으나 경계·위생 정책이 없다. 민감 가능 파일은 내용 확인 없이 미확정으로 남겼다.
- Impact: package/release에 의도하지 않은 대형/생성/환경 파일이 포함되거나, 감사자가 참조 asset을 제품 구현 증거로 오인할 수 있다.
- Suggested Fix: support tree manifest와 shipped-scope 정책을 추가하고, 재생성 산출물/pycache/scratch의 추적 정책을 human owner가 결정한다. 비밀 가능 metadata는 공개 전 별도 검토한다.
- Re-audit Method: `git ls-files`를 runtime/support/generated 세 분류로 재분류하고 Cargo package/release artifact 목록과 대조한다. `.antigravitycli`/`.gemini` 내용은 권한·비밀 검토 후 필요할 때만 확인한다.
- Owner: Human / Release Owner
- Confidence: High for hygiene, Low for contents of uninspected metadata
- Notes: 사용자 지시에 따라 PNG/HTML과 `.agents` 기능 자체를 제품 runtime으로 세지 않았다. 이 finding은 저장소 위생과 authority boundary만 다룬다.

## 6. Uncertainties and Clarifications Needed

- `AGENTS.md`의 `doomlike/0.5.1`이 의도적 상위 템플릿인지 잘못 병합된 프로젝트 메타데이터인지 human authority 결정이 필요하다(A01-F001).
- Phase 53/54 `v3.9.1/v3.9.2`가 `3.9.0` package에 포함된 unreleased work인지, release bump가 누락된 것인지 결정이 필요하다(A01-F002).
- Ollama를 built-in provider로 약속할지 Custom Provider 예시로만 유지할지 결정이 필요하다(A01-F004).
- `AGENTS.md` trailing spaces가 Markdown hard-break 의도인지 현재 diff-check gate에서 허용할지 결정이 필요하다(A01-F010).
- 실제 bwrap user namespace 권한과 session log home path가 제품 prerequisite인지, CI-only test seam으로 격리해야 하는지 결정이 필요하다(A01-F009).
- `.antigravitycli`/`.gemini` metadata의 shipped/secret 여부는 내용을 읽지 않아 판정하지 않았다(A01-F014).

## 7. Perspective Decision

### A01 Decision: HOLD

현재 트리의 문서·구현·기능 정합성에 대해 A01 관점의 PASS를 발행할 수 없다. 특히 프로젝트 identity와 release authority가 미확정이고(A01-F001~F002), 완료된 Phase/다국어/provider/하네스 계약의 실제 소비·경계가 문서 주장과 다르며(A01-F003~F007), CI/release gate와 최신 실행 증거가 같은 결론을 지지하지 않는다(A01-F008~F010). 따라서 통합 감사에서도 적어도 다음을 해소하기 전에는 PASS 계열 판정을 사용하지 않아야 한다.

1. 프로젝트명·버전·Unreleased/Phase 54의 canonical authority를 사람/아키텍트가 확정한다.
2. user-facing provider/i18n 범위와 실제 호출 경로를 문서·코드·테스트에서 닫는다.
3. stale design/roadmap/ADR/file-responsibility 문서를 historical snapshot과 current authority로 분리한다.
4. CI/release gate와 test isolation을 재현 가능한 명령으로 연결하고, 현재 119/121 실행 결과의 두 실패를 환경 제한 또는 수정 대상으로 명시한다.
5. config path, ADR IDs, reference/generated tree provenance를 복구한다.

이 보고서는 소스·테스트·설정·제품 문서를 수정하지 않았으며, 지정된 보고서 파일만 생성했다.
