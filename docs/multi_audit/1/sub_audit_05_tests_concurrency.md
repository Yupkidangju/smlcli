# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 1
- Perspective: A05 — 테스트 품질·동시성·실패 복원·데이터 무결성
- User Goal: `$multi-audit` 프로젝트의 모든 문제점을 파악하여 안정화하고 완성도를 높이는 전체감사 개시. 프로젝트의 모든 문서 및 구현내용을 먼저 파악후 작업을 분해하여 상세 감사 후 근본적인 문제를 해결할 수 있도록 합니다.
- Audit Basis: Standard-backed
- Standard Path: `/mnt/Projects_SSD/rust/smlcli/AI_AUDIT_DOC_STANDARD.md`
- Report Contract: `/home/eunho1/.codex/skills/multi-audit/references/report-contract.md`

## 2. Assigned Scope

이번 관점은 다음을 독립적으로 확인했다.

- `src/tests` 전체와 소스 내부 `#[test]`/`#[tokio::test]` 인벤토리 및 현재 테스트 실행 결과
- 문서의 `105개`, `115개`, `121개`, `100%`, `완벽`, `보장` 테스트·품질 주장의 현재성
- 테스트가 실제 호출 경로·소비·상태 변화·파일/프로세스 부작용을 검증하는지 여부
- Tokio task/channel, cancellation token, 병렬 도구 결과 순서, 스트리밍 출력·마스킹, Auto-Verify
- config/session 저장, MCP lifecycle, shell timeout/kill/output cap, UI event routing의 양성·음성·경계 테스트
- full test, ignored test, doctest, target/feature 범위와 CI/local 명령 차이

## 3. Excluded and Uninspected Scope

- `docs/multi_audit/1`의 다른 에이전트 보고서는 독립성 유지를 위해 읽지 않았다.
- `.git/`, 프로젝트 `target/`, 캐시 및 생성·벤더 산출물은 조사 범위에서 제외했다.
- 실제 외부 LLM/API 네트워크 호출은 수행하지 않았다. MCP 검증은 저장소의 mock 프로세스 경로만 사용했다.
- `build.sh`, `cargo fmt` 수정 모드, 소스·테스트·설정·제품 문서 수정은 수행하지 않았다. 이 보고서 파일만 작성 대상이다.
- 물리 TTY 렌더링, Windows/musl 실행, 실제 비루프백 네트워크는 현재 환경에서 검증하지 못했다.
- `/tmp`에 감사용으로 생성한 정확한 `CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target`은 검증 후 삭제했다. 프로젝트 파일과 기존 사용자 변경은 보존했다.

## 4. Evidence Examined

### 문서 및 설정

- `AI_AUDIT_DOC_STANDARD.md` 전체
- `/home/eunho1/.codex/skills/multi-audit/references/report-contract.md` 전체
- `spec.md`, `designs.md`, `BUILD_GUIDE.md`, `README.md`
- `IMPLEMENTATION_SUMMARY.md`, `CHANGELOG.md`, `LESSONS_LEARNED.md`, `audit_roadmap.md`
- 과거 PASS 주장 대조용 `audit_report_8.md`, `audit_report_9.md`
- `Cargo.toml`, `Cargo.lock`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`

### 구현 및 테스트

- `src/main.rs`, `src/tests/mod.rs`
- `src/tests/audit_regression.rs`, `provider_validation.rs`, `settings_flow.rs`, `shell_permissions.rs`
- 내부 테스트가 있는 `src/infra/workspace_utils.rs`, `src/tui/layout.rs`, `src/tui/widgets/inspector_tabs.rs`
- `src/app/event_loop.rs`, `src/app/mod.rs`, `src/app/state.rs`, `src/app/tool_runtime.rs`, `src/app/chat_runtime.rs`
- `src/tools/executor.rs`, `src/tools/shell.rs`, `src/tools/grep.rs`, `src/tools/fetch.rs`
- `src/infra/config_store.rs`, `src/infra/session_log.rs`, `src/infra/mcp_client.rs`, `src/infra/secret_store.rs`
- `scripts/mock_mcp_server.py`

### 실행 명령 및 결과

| 명령 | 결과 |
| --- | --- |
| `CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo test --all-targets --locked --no-fail-fast` | Exit 101. `running 121 tests`; `119 passed; 2 failed; 0 ignored`. 실패: `test_bwrap_mounts_workspace_as_canonical_guest_root`, `test_session_harness_record_is_skipped_on_restore`. |
| `CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo test -- --list` | Exit 0. `121 tests, 0 benchmarks`. |
| `CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo test --doc --locked --no-fail-fast` | Exit 101: `no library targets found in package smlcli`. |
| `CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo test --all-features --all-targets --locked -- --list` | Exit 0. 기능 목록은 `features:{}`이고 다시 121개 목록만 확인됨. |
| 컴파일된 테스트 바이너리 `--list --ignored` | `0 tests, 0 benchmarks` (ignored 테스트 없음). |
| 각 실패 테스트를 컴파일된 바이너리로 `--exact --nocapture` 재실행 | bwrap은 `No permissions to create a new namespace`, 세션은 `Read-only file system (os error 30)`로 각각 단독 재현. |
| `cargo metadata --no-deps --format-version 1` | 테스트 가능한 대상은 `bin` 하나(`src/main.rs`), `doctest:false`, feature `{}`. |

현재 선언된 테스트 수는 `src/tests/audit_regression.rs` 112개 + `provider_validation.rs` 1개 + `settings_flow.rs` 2개 + `shell_permissions.rs` 1개 + `workspace_utils.rs` 3개 + `tui/layout.rs` 1개 + `inspector_tabs.rs` 1개 = 121개다. 이는 목록 수와 일치하지만, 실행 성공 수와는 다르다.

## 5. Findings

### [A05-F001] 현재 전체 테스트 게이트는 121개 중 2개 실패하며 과거 PASS 주장은 재현되지 않음

- Pass: Debug / Engineering Quality
- Pattern: `TEST-001`, `BUILD-001`
- Area: 전체 테스트 게이트·문서 주장·환경 재현성
- Severity: Major
- Status: Confirmed
- Summary: 사용자가 지정한 locked/all-targets/no-fail-fast 명령이 현재 트리에서 실패했다. 테스트 목록은 121개지만 실제 결과는 119 pass/2 fail이므로 `121 passed`, `105개/115개 100%` 또는 `완벽` 주장을 현재 증거로 유지할 수 없다.
- Evidence:
  - `CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo test --all-targets --locked --no-fail-fast`가 Exit 101로 종료했고, `test_bwrap_mounts_workspace_as_canonical_guest_root`와 `test_session_harness_record_is_skipped_on_restore`가 실패했다.
  - `src/tests/audit_regression.rs:1701-1721`은 bwrap `pwd` 성공을 무조건 assert한다. 단독 재실행도 `bwrap: No permissions to create a new namespace`로 실패했다.
  - `src/tests/audit_regression.rs:1853-1869`는 실제 홈의 세션 경로를 여는 `new_workspace_session(...).unwrap()`을 사용한다. 단독 재실행도 `Read-only file system (os error 30)`로 실패했다.
  - `IMPLEMENTATION_SUMMARY.md:104,111` 및 `CHANGELOG.md:47`은 각각 115개/105개 전체 100% 패스를 주장한다. `audit_report_8.md:55-59`, `audit_report_9.md:57-61`도 121 pass와 완벽한 게이트를 주장하지만 현재 재실행으로 반증된다.
  - `.github/workflows/ci.yml:41-48`의 CI는 `cargo test --all-targets`만 실행하며 bwrap user namespace capability 또는 테스트용 writable HOME을 준비하지 않는다. `BUILD_GUIDE.md:31-38`도 명령만 나열하고 해당 런타임 전제를 고정하지 않는다.
- Expected Basis: 사용자 지정 검증 명령, `AI_AUDIT_DOC_STANDARD.md`의 `TEST-001`/`BUILD-001` 및 현재 실행 결과가 문서 주장과 일치해야 한다. 환경 의존 테스트는 capability와 제외 조건을 명시해야 한다.
- Actual: 등록 수와 성공 수가 다르고, 두 실패는 test process가 요구하는 환경 capability를 선언·격리하지 않은 상태에서 무조건 실패한다. 과거 PASS 리포트는 현재 재현 증거가 아니다.
- Impact: 전체 PASS/100% 주장이 품질 게이트로 사용될 수 없고, CI와 지원 환경에서 재현되지 않는 실패가 릴리스 판단을 오염시킨다.
- Suggested Action: bwrap 테스트를 지원 capability가 명시된 CI job으로 격리하거나 user namespace 미지원 시 silent pass가 아닌 명시적 `Ignored`/환경 게이트로 분리한다. 세션 테스트는 주입 가능한 임시 HOME/session root를 사용한다. 121/121 결과를 다시 얻은 뒤 오래된 105/115/121 문서와 역사 리포트를 현재 기준으로 정리한다.
- Re-audit Method: 깨끗한 writable test HOME과 명시적 user namespace capability에서 동일한 locked/all-targets/no-fail-fast 명령을 실행하고, `--list`, `--ignored`, 실행 요약, CI job 환경을 함께 기록한다.
- Owner: Coder / Architect
- Confidence: High
- Notes: 이 finding은 현재 환경 탓이라고 단정하는 finding이 아니라, 지원 실행 조건이 코드·CI·문서에 고정되지 않아 현재 게이트가 재현 불가능하다는 판정이다.

### [A05-F002] 상태·문자열 복제 테스트가 실제 production 호출과 부작용을 검증하지 않음

- Pass: Debug / Engineering Quality
- Pattern: `TEST-001`, `IMP-003`
- Area: 회귀 테스트의 호출 경로·소비·부작용 검증
- Severity: Major
- Status: Confirmed
- Summary: 중요한 설정·보안·UI 테스트 여러 개가 production handler를 호출하지 않고 구현 branch를 테스트 안에서 다시 작성하거나 enum/string만 확인한다. 이런 테스트는 구현이 깨져도 통과할 수 있어 100% 회귀 주장에 포함될 근거가 약하다.
- Evidence:
  - `src/tests/audit_regression.rs:15-31`의 `test_api_key_masking`은 `"*".repeat(...)`만 확인한다. 실제 렌더러는 `src/tui/widgets/setting_wizard.rs:84-87`의 `InputField::with_password(true).render()`를 사용하며, 해당 renderer나 secret 값 소비 경로를 호출하지 않는다.
  - `src/tests/audit_regression.rs:36-52`의 `test_provider_switch_resets_model`은 `settings.default_provider`와 `default_model`을 테스트 본문에서 직접 대입한다. 실제 async 검증·rollback·save 경로를 호출하지 않는다.
  - `src/tests/audit_regression.rs:56-82`의 `test_network_policy_deny_blocks_chat`은 `NetworkPolicy` enum 값을 비교할 뿐이다. 실제 네트워크 차단은 `src/app/chat_runtime.rs:118-137`의 `resolve_credentials()` 경로에 있다.
  - `src/tests/audit_regression.rs:86-118`의 wizard Esc 테스트는 `if wizard.err_msg.is_some()` branch를 테스트 안에서 복제하고, `test_wizard_no_error_esc_quits`는 `should_quit` 또는 `handle_input`을 전혀 호출하지 않는다.
  - `src/tests/audit_regression.rs:122-143`의 저장 실패 테스트는 `is_loading_models`/`save_config`/이벤트를 실행하지 않고 `if err.is_some()`를 시뮬레이션한다.
  - `src/tests/audit_regression.rs:4419-4488`의 LM Studio “E2E” 테스트는 `available_models`와 `WizardStep::ModelSelection`을 직접 설정하고 `Saving` 진입에서 끝난다. 문서의 “최종 Settings 영속화 저장” 주장을 실제 `save_config`로 검증하지 않는다.
  - `src/tests/audit_regression.rs:4218-4274`의 MCP persistence 테스트도 `/mcp add/remove`와 `save_config`가 아니라 `Vec::push/iter_mut/retain`만 직접 수행한다.
- Expected Basis: `AI_AUDIT_DOC_STANDARD.md`의 `TEST-001` 및 `IMP-003`에 따라 완료·보안·실패 주장은 실제 호출 경로와 결정적 side effect assertion으로 잠겨야 한다.
- Actual: 구현 branch의 복사본, enum equality, 문자열 포함 여부가 production 동작을 대신한다. 특히 실제 저장·이벤트·provider/network call·renderer 소비가 검증되지 않는다.
- Impact: 회귀가 발생해도 테스트가 계속 PASS할 수 있고, 문서의 “E2E/100%/완벽” 표현이 실제 coverage보다 강하다.
- Suggested Action: `handle_input`, `handle_action`, wizard/config command, `save_config`, provider adapter seam을 호출하는 테스트로 교체하고, state/timeline/event/disk 결과를 assert한다. branch를 테스트에 재작성하지 말고 테스트가 production 함수의 contract를 소비하게 한다.
- Re-audit Method: 각 대표 테스트의 구현 branch를 임시로 깨뜨렸을 때 실패하는지 mutation-style 확인을 수행하고, 실제 `ConfigSaveFinished`, provider guard, renderer output, persisted TOML/JSONL을 관찰한다.
- Owner: Coder
- Confidence: High
- Notes: 단순 domain value test가 모두 무가치하다는 의미는 아니다. 현재 테스트 이름과 문서 주장이 통합/E2E 수준을 내세우는 항목에 대해 실제 호출·부작용이 없는 점이 finding이다.

### [A05-F003] 테스트와 런타임이 실제 홈·전역 저장소·고정 임시 경로를 공유함

- Pass: Debug / Engineering Quality
- Pattern: `TEST-001`, `ARCH-001`
- Area: 테스트 격리·병렬 결정성·사용자 데이터 무결성
- Severity: Major
- Status: Confirmed
- Summary: 세션·secret·App 초기화 테스트가 process 전용 임시 root가 아니라 실제 `dirs::home_dir()`의 `~/.smlcli`를 사용한다. 병렬 test harness에서 전역 상태와 파일이 공유되고, 현재 read-only 환경에서 실제 실패가 발생했다.
- Evidence:
  - `src/infra/session_log.rs:328-366,389-396`의 `new_workspace_session`과 `SessionIndex`는 `dirs::home_dir()/ .smlcli/sessions`에 직접 쓰며, 세션 생성 시 `SessionIndex::upsert`까지 수행한다.
  - `src/app/state.rs:139-185`의 `DomainState::new_async()`는 앱 초기화 때 config를 읽고 workspace session/harness record를 생성한다. `App::new()`를 사용하는 테스트(`src/tests/audit_regression.rs:2368,2414,3848,3916`)는 이 부작용 경로를 피하지 않는다.
  - `src/tests/audit_regression.rs:1853-1869`는 실제 workspace session을 `unwrap()`하고, 현재 실행에서 read-only 오류로 실패했다.
  - `src/infra/secret_store.rs:15-31`은 master key를 실제 `~/.smlcli/.master_key`에 생성한다. `src/tests/audit_regression.rs:2422-2429`의 `test_block_lifecycle_roles_runtime`가 이를 호출한다.
  - `src/tests/audit_regression.rs:587-627`, `634-646`, `654-687`은 고정된 `/tmp/smlcli_test_session_1/2/3` 이름과 무시되는 cleanup 결과를 사용한다. panic/동시 실행/잔여 파일이 다음 실행에 영향을 줄 수 있다.
- Expected Basis: 테스트는 병렬 실행되어도 서로의 파일·환경·전역 설정을 훼손하지 않아야 하며, secret/session 데이터는 사용자 저장소를 오염시키지 않아야 한다. `AI_AUDIT_DOC_STANDARD.md`의 재현성·데이터 무결성 원칙을 적용한다.
- Actual: writable/read-only HOME 여부에 따라 결과가 달라지고, 실제 user home과 고정 temp path를 공유한다. 앱 초기화와 secret 생성도 테스트 격리 seam이 없다.
- Impact: false negative/positive, flaky race, 사용자 세션·master key 오염, 권한/CI 차이에 따른 테스트 실패가 발생한다. 현재 실패가 이 문제를 직접 드러냈다.
- Suggested Action: config/session/secret root를 dependency injection 또는 test-only path provider로 분리하고 각 테스트마다 `tempfile::TempDir`를 사용한다. `HOME` 전역 변경은 serial guard로 감싸거나 더 안전한 명시적 path seam을 사용하며, cleanup 실패를 무시하지 않는다.
- Re-audit Method: 독립 writable HOME에서 병렬 1회·반복 10회 및 `--test-threads=1`을 비교하고, 실제 사용자 `~/.smlcli`에 파일이 생성되지 않는지 확인한다.
- Owner: Coder / Architect
- Confidence: High
- Notes: 세션 기능 자체의 제품 요구를 부정하는 것이 아니라, 테스트가 제품 전역 저장소를 직접 공유하는 격리 결함이다.

### [A05-F004] MCP 실행 경로가 cancellation token을 무시하고 lifecycle 음성 테스트도 없음

- Pass: Debug / Engineering Quality
- Pattern: `DBG-002`, `TEST-001`
- Area: MCP cancellation·timeout·EOF·child lifecycle
- Severity: Major
- Status: Confirmed
- Summary: 모든 도구에 취소 토큰을 등록하지만 MCP 분기에서는 토큰을 `select!`에 연결하지 않는다. Ctrl+C/ESC가 토큰을 취소하고 map을 지워도 `client.call_tool(...).await`는 자체 10초 timeout 또는 서버 응답까지 계속될 수 있다. 테스트는 정상 initialize/list/call만 검증한다.
- Evidence:
  - `src/app/tool_runtime.rs:345-353`은 도구별 `CancellationToken`을 생성·등록한다.
  - 같은 파일 `:409-432`의 MCP route는 `client.call_tool(...).await`를 직접 기다리며 `cancel_token.cancelled()`를 경쟁시키지 않는다. 비-MCP branch `:463-464`만 executor에 token을 전달한다.
  - `src/app/mod.rs:1955-1968,2039-2049`의 Ctrl+C/ESC는 map의 토큰을 cancel하고 즉시 clear한다. MCP call 자체에 대한 취소 완료·shutdown 대기·결과 회수 검증이 없다.
  - `src/infra/mcp_client.rs:224-231`에는 10초 response timeout이 있지만, 이는 사용자 취소와 다른 동작이다. `:176-203`의 EOF pending drain도 직접 테스트되지 않는다.
  - `src/tests/audit_regression.rs:4021-4082,4089-4150`은 정상 mock list/call만 수행한다. Python 또는 mock script가 없으면 `return`으로 성공 처리한다(`:4023-4038`, `:4090-4102`).
  - `spec.md:2526-2530`은 mock 왕복뿐 아니라 서버 crash의 graceful error와 10초 timeout 검증을 요구한다.
- Expected Basis: 사용자 지정 동시성·실패 복원 목표와 `spec.md` MCP 테스트 요구사항. 취소 가능한 모든 활성 도구는 사용자 취소 시 bounded completion을 가져야 한다.
- Actual: MCP route는 token cancel을 관찰하지 않으며, crash/EOF/timeout/child 종료를 테스트하지 않는다. 정상 E2E 두 건의 PASS만으로 lifecycle을 보장할 수 없다.
- Impact: 사용자가 MCP 요청을 취소해도 최대 10초 동안 pending 상태가 남고 UI/Auto-Verify 경합이 발생할 수 있다. 서버 hang/EOF 시 child·pending map·event 상태의 복원 여부도 미검증이다.
- Suggested Action: MCP call을 `tokio::select!`로 `cancel_token.cancelled()`와 경쟁시키고 취소 시 `shutdown()` 및 pending 정리를 보장한다. hang/EOF/초기화 실패/isError 응답을 내는 mock mode를 추가해 timeout, pending empty, child 종료, ToolError 이벤트를 assert한다. 환경 dependency가 없을 때는 silent return 대신 명시적 skip/capability gate를 사용한다.
- Re-audit Method: mock server를 `sleep`/EOF/invalid response/error response 모드로 실행하고 call 도중 token cancel을 수행하여 bounded return, pending map empty, child exit, 정확한 UI action을 확인한다.
- Owner: Coder
- Confidence: High
- Notes: 현재 mock E2E가 실제 Python 프로세스 왕복을 확인하는 범위는 인정하지만, 그것은 lifecycle 전체가 아니다.

### [A05-F005] 셸 streaming·timeout·process-group kill·output cap 경로가 production에 연결되었는지 테스트되지 않음

- Pass: Debug / Engineering Quality
- Pattern: `DBG-002`, `TEST-001`
- Area: shell failure recovery·streaming·process resource bounds
- Severity: Major
- Status: Confirmed
- Summary: 명세와 요약 문서는 stdout/stderr streaming, 30초 timeout, cancel 시 child 종료, 5MB cap을 완료로 설명하지만 실제 Tool trait 경로는 `tx=None` buffer mode를 사용한다. 테스트는 sandbox 경로 허용/차단만 수행하고 timeout·cancel·descendant kill·cap 경계를 실행하지 않는다.
- Evidence:
  - `src/tools/shell.rs:38-46`의 `execute_shell`은 `execute_shell_streaming(..., None, cancel_token)`을 호출한다.
  - `src/tools/shell.rs:116-124`의 streaming tx는 optional이고, `src/tools/shell.rs:516-533`의 `ExecShellTool::execute`는 `execute_shell`을 호출한다. 같은 주석(`:527-530`)도 ToolContext에 action tx가 아직 미연동임을 인정한다.
  - `src/tools/shell.rs:195-250,253-300`에는 5MB/1MB cap과 `ToolOutputChunk` 전송 코드가 있으나 production Tool trait에서 tx를 전달하지 않는다. `src/app/mod.rs:987-1005`의 소비 handler는 직접 Action을 받았을 때만 동작한다.
  - `src/tools/shell.rs:303-356`에는 process-group kill, 30초 timeout, cancellation select가 있으나 `src/tests`에는 `sleep`, `cancel_token.cancel()`, `kill_process_group`, 5MB/1MB 경계 assertion이 없다. shell 관련 실행 테스트는 `src/tests/audit_regression.rs:1881-1929`의 `/etc` 차단과 workspace 쓰기뿐이다.
  - `spec.md:1176-1183`은 timeout 30초·stdout/stderr streaming·cancel child 종료를, `spec.md:2014-2018`은 100MB 출력의 5MB cap 검증을 요구한다. `IMPLEMENTATION_SUMMARY.md:273-275,808-812`는 이를 완료 또는 검증된 기능처럼 적는다.
- Expected Basis: `spec.md` Shell Exec 계약과 `AI_AUDIT_DOC_STANDARD.md`의 `TEST-001`/`DBG-002` 결정적 failure-mode 검증.
- Actual: live output path는 현재 Tool trait에서 연결되지 않고, timeout/kill/cap 실패 모드에 직접 증거가 없다. sandbox allow/deny PASS는 process lifecycle PASS를 의미하지 않는다.
- Impact: 긴 출력, child가 descendant를 남기는 명령, 취소 중 channel/reader 경합, timeout 이후 zombie가 발생해도 회귀 테스트가 잡지 못할 수 있다. `ToolOutputChunk` 및 streaming masker의 실제 runtime coverage도 없다.
- Suggested Action: streaming이 shipped 범위라면 `ToolContext`에 bounded event sink을 연결하고, 아니면 문서에서 deferred로 분리한다. `sleep 100`, background child, stdout/stderr interleave, 정확히 cap-1/cap/cap+1 및 1MB line 경계를 별도 fixture로 테스트한다.
- Re-audit Method: external network 없이 shell fixture를 실행하여 live Action 순서, timeout boundedness, process-group descendant 종료, `is_truncated/original_size_bytes` 및 channel close를 측정한다.
- Owner: Coder
- Confidence: High
- Notes: 현재 테스트에서 `execute_shell` 자체가 호출되는 것은 확인했지만, 그 호출은 tx 없는 buffer mode다.

### [A05-F006] 스트리밍 secret masker의 chunk-boundary 동작에 회귀 테스트가 없고 중복·누출 가능성이 있음

- Pass: Debug / Engineering Quality
- Pattern: `TEST-001`, `SEC-005` 보조 관점
- Area: streaming masker·secret redaction·output integrity
- Severity: Major
- Status: Probable
- Summary: `test_api_key_masking`은 wizard 표시용 별표 생성만 확인하고 실제 `mask_secrets`/`ToolOutputChunk`를 호출하지 않는다. 구현은 이전 trailing window와 현재 chunk를 합친 결과 전체를 caller의 `text`로 되돌린 뒤 caller가 다시 전체를 append하므로, secret이 chunk 경계를 가로지를 때 이전 출력 중복 또는 미마스킹 prefix 잔류가 가능하다.
- Evidence:
  - `src/app/mod.rs:1692-1749`의 `mask_secrets`는 `trailing_buffer`를 `window`에 prepend(`:1723-1725`)하고, match가 있으면 `*text = masked_window`로 caller 입력 전체를 교체(`:1727-1733`)한다.
  - `src/app/mod.rs:987-990`의 `ToolOutputChunk` handler는 반환된 `chunk` 전체를 `stream_accumulator`에 append한다. 이전 chunk의 trailing 부분은 이미 accumulator에 들어갔을 수 있다.
  - `src/app/mod.rs:752-760`의 flush는 남은 trailing만 별도로 append하므로 window의 “이미 소비된 부분”을 구별하지 않는다.
  - `rg` 인벤토리상 `src/tests`에는 `mask_secrets`, `StreamingMasker`, `ToolOutputChunk`를 호출하는 테스트가 없다. `src/tests/audit_regression.rs:15-31`은 `"*".repeat`와 문자열 assertion만 수행한다.
- Expected Basis: 사용자 지정 streaming masker·데이터 무결성 목표와 secret redaction hard-boundary의 경계별 양성/음성 테스트.
- Actual: 단일 chunk의 wizard 표시만 검증되고, secret split at boundary, Unicode boundary, repeated secret, newline flush, no-duplication/no-leak 조건은 검증되지 않는다. 정적 흐름상 이전 tail과 새 masked window가 함께 append될 수 있다.
- Impact: 로그/타임라인에 중복 출력이 생기거나 chunk 첫 부분의 secret 조각이 평문으로 남을 수 있다. 전체 결과가 나중에 마스킹되는 경로가 있더라도 live streaming 경로의 노출·정합성을 보장하지 못한다.
- Suggested Action: masker API를 “현재 chunk에 실제로 방출할 부분”과 “아직 보류할 tail”로 분리하고, logical stream 기준으로 한 번만 append한다. secret 길이의 모든 split 위치, UTF-8 경계, newline/flush, 여러 secret을 fixture로 검증한다.
- Re-audit Method: test-only encrypted settings seam으로 known secret을 주입하고 `ToolOutputChunk`를 1바이트씩 분할하여 최종 logs/timeline에 secret 원문·중복·누락이 없는지 assert한다.
- Owner: Coder / Security reviewer
- Confidence: Medium-High
- Notes: 실제 live streaming wiring 자체가 A05-F005에서 미연결로 확인되므로, 이 finding의 현재 production impact는 wiring 여부에 따라 달라질 수 있다. 구현 경계 결함과 테스트 공백은 독립적으로 남는다.

### [A05-F007] Grep/Fetch 출력 절단이 ToolResult truncation metadata와 불일치함

- Pass: Debug / Engineering Quality
- Pattern: `TEST-001`, `DBG-002`
- Area: 결과 데이터 무결성·LLM context truncation
- Severity: Major
- Status: Confirmed
- Summary: Grep과 Fetch는 결과를 제한하거나 절단하지만 `ToolResult.is_truncated`와 `original_size_bytes`를 false/None으로 반환한다. 해당 경계·metadata를 검증하는 테스트도 없다.
- Evidence:
  - `src/tools/grep.rs:91-117`은 100건에서 `truncated=true` 및 사용자 문구를 만들지만 `:120-130`의 `ToolResult`는 `is_truncated:false`, `original_size_bytes:None`이다.
  - `src/tools/fetch.rs:83-103`은 응답 bytes가 500,000을 넘으면 읽기를 중단하고, markdown이 10,000 bytes를 넘으면 절단 문구를 붙이지만 `:105-115`에서 동일하게 false/None을 반환한다.
  - `src/domain/tool_result.rs:9-27`에는 절단 여부와 원래 크기를 전달하는 계약 필드가 명시되어 있다.
  - `src/tests/audit_regression.rs:2499-2546`은 Fetch의 permission과 invalid scheme만, `:2548-2601`은 Grep sandbox와 invalid regex만 확인한다. 성공 응답의 cap-경계나 metadata assertion은 없다.
  - `spec.md:2004-2005,2045-2046`은 5MB cap 시 LLM에 절단 사실과 원래 크기 metadata를 전달하는 계약을 설명한다.
- Expected Basis: `ToolResult` typed contract, `spec.md` truncation metadata rule, 사용자 지정 데이터 무결성 목표.
- Actual: 사람이 읽는 일부 문구는 있을 수 있지만 typed result metadata가 false/None이어서 공통 `flush_pending_tool_outcomes`의 truncation 처리(`src/app/mod.rs:1649-1667`)가 이 도구들에는 적용되지 않는다.
- Impact: LLM과 후속 자동화가 결과가 완전하다고 오인하고 누락된 파일/검색 결과를 근거로 판단할 수 있다. cap 동작이 바뀌어도 회귀 테스트가 잡지 못한다.
- Suggested Action: source bytes/매칭 상한을 넘긴 경우 `is_truncated=true`, 실제 원본 크기 또는 측정 가능한 상한 metadata를 채우고, local fixture로 cap-1/cap/cap+1 및 malformed/large input을 검증한다. 외부 네트워크 없이 Fetch executor seam을 주입한다.
- Re-audit Method: 100+ matches fixture와 500KB/10KB HTML fixture를 실행해 stdout, flag, original size, LLM tool message metadata가 일치하는지 확인한다.
- Owner: Coder
- Confidence: High
- Notes: 외부 URL 호출은 수행하지 않았지만 truncation metadata 불일치는 소스와 typed result만으로 확인된다.

### [A05-F008] config/session 동시 저장·실패 원자성에 대한 직접 검증이 없고 SessionIndex는 plain read-modify-write임

- Pass: Debug / Engineering Quality
- Pattern: `ARCH-001`, `TEST-001`
- Area: config/session persistence·concurrent writers·crash recovery
- Severity: Major
- Status: Confirmed
- Summary: config 저장에는 lock/temp/rename 코드가 있으나 이를 직접 호출하는 테스트가 없고, session index는 lock/temp/rename 없이 전체 JSON을 plain write한다. 두 writer 또는 중간 실패에서 데이터 유실·손상 가능성을 검증하지 않았다.
- Evidence:
  - `src/infra/config_store.rs:24-120`은 config lock과 `.tmp` rename을 구현하지만 `src/tests`에는 `save_config`, `cleanup_tmp_files`, 기본 `load_config`를 호출하는 테스트가 없다. 테스트는 `load_config_from_path` parse error(`src/tests/audit_regression.rs:366-391`)와 clone 후 TOML 직렬화(`:2389-2404`)만 수행한다.
  - `src/infra/session_log.rs:423-433`의 `SessionIndex::upsert`는 `load_all`→메모리 수정→`save_all`의 read-modify-write다.
  - `src/infra/session_log.rs:470-484`의 `save_all`은 `std::fs::write(&path, json)`으로 직접 덮어쓰며 lock, temp rename, fsync가 없다. `load_all`은 JSON parse 실패를 `unwrap_or_default()`로 빈 목록으로 바꾼다(`:399-409`).
  - `src/tests/audit_regression.rs:4218-4274`의 MCP “persistence” 테스트는 `PersistedSettings` Vec만 조작하고 실제 config/session persistence를 호출하지 않는다. `SessionIndex::` 직접 테스트도 없다.
  - `spec.md:2049,2053-2055`는 config concurrent write lock/atomicity 및 결과 metadata 검증을 명시하며, Phase 46 문서는 session index를 사용자-facing 목록의 기반으로 사용한다.
- Expected Basis: 사용자 지정 config/session 원자성·데이터 무결성 목표와 `spec.md` File Locking/Write-and-Rename 계약.
- Actual: config path는 구현 흔적만 있고 failure/concurrency evidence가 없으며, SessionIndex는 두 프로세스가 동시에 upsert하면 마지막 writer가 다른 세션을 덮어쓸 수 있고 중간 plain write가 JSON을 손상시킬 수 있다.
- Impact: 세션 목록 유실·빈 목록 복구 오인·resume 대상 누락, 설정 저장 경합 회귀가 PASS 테스트를 통과할 수 있다.
- Suggested Action: SessionIndex에도 lock + temp + fsync + atomic rename을 적용하고 parse corruption을 빈 목록으로 조용히 reset하지 않도록 오류/backup 정책을 정한다. config/session에 주입 가능한 temp root와 concurrent writer/failure fixture를 추가한다.
- Re-audit Method: 동일 index에 여러 async/process writer를 동시에 실행하고 모든 session id 보존, JSON 유효성, 중간 write/rename 실패 후 이전 파일 보존, `.tmp` cleanup을 확인한다.
- Owner: Coder / Architect
- Confidence: High
- Notes: config 구현의 lock이 올바른지 별도 확정한 것이 아니라, 현재 테스트가 그 계약을 잠그지 않는다는 finding과 SessionIndex plain-write 위험을 함께 기록한다.

### [A05-F009] test target/feature/doctest/CI 범위가 좁은데 121개 전체 검증처럼 보고됨

- Pass: Debug / Engineering Quality
- Pattern: `TEST-001`, `BUILD-001`
- Area: coverage scope·CI matrix·PASS wording
- Severity: Minor
- Status: Confirmed
- Summary: 현재 121개는 하나의 binary test target에 포함된 단위/모듈 테스트 수다. doctest는 실행 대상 자체가 없고, feature는 빈 set이며, ignored는 0개다. Windows/musl release build는 있지만 해당 target의 test 실행은 없다.
- Evidence:
  - `cargo metadata --no-deps --format-version 1`은 `src/main.rs`의 `bin` target 하나와 `doctest:false`, `features:{}`를 반환했다.
  - `cargo test -- --list`는 121개/0 benchmark, `--list --ignored`는 0개를 반환했다.
  - `cargo test --doc --locked --no-fail-fast`는 `no library targets found`로 종료했다. 따라서 doctest PASS가 아니다.
  - `cargo test --all-features --all-targets --locked -- --list`도 feature set이 없으므로 같은 121개 binary 목록뿐이다.
  - `.github/workflows/ci.yml:47-48`은 Ubuntu에서 `cargo test --all-targets`만 수행한다. `.github/workflows/release.yml:31-73`의 Windows/musl job은 build만 수행하고 target별 test는 하지 않는다. `BUILD_GUIDE.md:40-42`는 cross-platform testing을 필수라고 쓰지만 matrix 검증은 없다.
- Expected Basis: `AI_AUDIT_DOC_STANDARD.md`의 scope inventory와 TEST/BUILD gate 규칙. PASS 문구는 실제 실행 범위와 제외 범위를 함께 명시해야 한다.
- Actual: 121개 binary test PASS/FAIL 수가 project-wide, doctest, feature, cross-target assurance처럼 읽힐 수 있다. 이전 리포트의 `cargo test -- -D warnings`도 일반 test harness 범위만 기록한다.
- Impact: 미검증 target/문서 예시/교차 플랫폼 lifecycle이 release PASS에 포함된 것으로 오인될 수 있다.
- Suggested Action: CI/local matrix를 `bin unit`, doctest unavailable, ignored=0, feature set empty, Linux runtime integration, Windows/musl build-only로 명시한다. cross-target runtime test가 비목표라면 PASS 문구에서 제거한다.
- Re-audit Method: 최종 report에 `--list`, ignored, doctest error, metadata features/targets, CI job별 명령을 표로 기록하고, 필요한 target에는 별도 test job을 추가해 실행 결과를 남긴다.
- Owner: Architect / CI owner
- Confidence: High
- Notes: binary-only 프로젝트에서 doctest가 없다는 사실 자체를 결함으로 단정하지 않는다. 문제는 범위가 명시되지 않은 채 “전체/완벽”으로 확장 해석되는 점이다.

### [A05-F010] 실제 EventLoop·다중 producer·종료 경합은 테스트되지 않으며 TTY 크기를 test 전용 상수로 고정함

- Pass: Debug / Engineering Quality
- Pattern: `DBG-001`, `TEST-001`
- Area: UI event ordering·shutdown·terminal capability
- Severity: Minor
- Status: Confirmed
- Summary: `EventLoop::new()`가 생성하는 tick, blocking crossterm input, signal producer와 receiver 종료를 검증하는 테스트가 없다. 대신 대부분의 테스트가 `handle_input`/`handle_action`을 직접 호출하고, mouse test는 production terminal size를 `cfg!(test)`로 `(100,30)`에 고정한다.
- Evidence:
  - `src/app/event_loop.rs:29-90`은 세 producer와 `spawn_blocking` polling loop를 생성하지만 `src/tests`에 `EventLoop::new` 또는 `EventLoop::next` 호출 테스트가 없다.
  - `src/app/event_loop.rs:47-70`의 blocking loop는 channel send가 실패해야만 빠져나오며, Quit/receiver drop과 poll task 종료 경합을 직접 검증할 seam이 없다.
  - `src/tests/audit_regression.rs`의 입력/Action 테스트는 `handle_input`/`handle_action` 직접 호출(`:1943-2053`, `:2701-2785`)이고 실제 event queue ordering을 거치지 않는다.
  - `src/app/mod.rs:2615-2626`은 `cfg!(test)`일 때 terminal size를 강제로 `(100,30)`으로 만든다. `src/tests/audit_regression.rs:2277-2357`의 mouse test는 이 고정값에만 맞춰져 실제 80/120/140 또는 resize sequence를 검증하지 않는다.
- Expected Basis: `AI_AUDIT_DOC_STANDARD.md`의 `DBG-001` startup/event chain 및 `TEST-001` deterministic failure-mode 원칙, `spec.md`의 resize/mouse/event 계약.
- Actual: reducer 함수의 단일 호출은 통과하지만 producer 간 event ordering, Quit 중 pending Action, resize 직후 mouse target, blocking task shutdown은 미검증이다.
- Impact: UI race, 종료 시 task 잔류, 실제 terminal geometry에서의 routing drift가 121개 테스트 PASS에 숨을 수 있다.
- Suggested Action: crossterm source를 주입할 수 있는 deterministic EventLoop harness를 두고 Tick/Input/Action/Resize/Quit 순서를 재현한다. test-only size override 대신 명시적 geometry parameter를 사용해 여러 크기 경계를 검증한다.
- Re-audit Method: synthetic event source로 queue saturation, Quit+ToolFinished ordering, resize before mouse, receiver drop 후 task 종료를 bounded timeout으로 확인한다.
- Owner: Coder
- Confidence: High
- Notes: 이 finding은 물리 TTY 자체를 요구하는 것이 아니라, 현재 direct handler 테스트만으로는 event architecture의 경쟁 상태를 판정할 수 없다는 뜻이다.

## 6. Uncertainties and Clarifications Needed

- 지원 CI/배포 환경이 Linux unprivileged user namespace를 반드시 제공하는지, 아니면 bwrap 테스트가 capability-gated integration인지 명세가 없다. 이 선택에 따라 A05-F001의 실행 정책을 확정해야 한다.
- `ToolOutputChunk`/streaming shell이 현재 shipped Phase인지, `ToolContext` event sink 연동 전의 deferred surface인지 문서가 서로 다르게 표현한다. `src/tools/shell.rs:527-530`의 deferred 주석과 `spec.md`/`IMPLEMENTATION_SUMMARY.md`의 완료 주장을 조정해야 한다.
- “전체 테스트”의 의미가 main binary unit tests만인지, doctest/target/feature/실제 외부 lifecycle까지인지 명시가 없다. 요구사항을 창작하지 않고 현재 실행 범위를 문서로 닫아야 한다.
- Fetch/Grep의 truncation metadata가 공통 `ToolResult` 계약에 반드시 적용되는지 제품 명세에서 도구별로 분리되어 있지 않다. 현재 구현과 `spec.md:2045-2046`은 공통 적용을 시사하지만 확인이 필요하다.
- 테스트 전용 HOME/session root 주입 방식과 사용자 데이터 보호 경계가 문서화되어 있지 않다. 현재 환경 실패를 단순 skip할지, 지원 환경 요구사항으로 승격할지 결정이 필요하다.

## 7. Perspective Decision

- Decision: `HOLD` (A05 범위)
- Rationale: 현재 지정 전체 테스트 게이트가 121개 중 2개 실패했고, 테스트 성공을 주장하는 핵심 경로 중 일부가 실제 production 호출을 하지 않는다. MCP cancellation, shell timeout/kill/output cap, streaming redaction, config/session concurrent persistence에 대한 결정적 실패-mode 증거가 없으며, SessionIndex에는 plain write 경합 위험이 확인된다.
- PASS limitation: 이 판정은 A05 관점의 테스트·동시성·복원·무결성 범위에 대한 것이며, 다른 관점의 결과를 대신하지 않는다. 현재 증거로 프로젝트 전체 PASS 또는 “100%/완벽”을 지지하지 않는다.
- Required re-audit gate:
  1. writable isolated HOME과 명시적 bwrap capability에서 동일 locked/all-targets 명령이 121/121로 재현되거나, 환경-gated 결과가 보고서에 명시되어야 한다.
  2. 복제형 테스트를 production handler/side effect 기반으로 교체하고, streaming/cancellation/MCP/shell/config/session failure fixtures를 추가해야 한다.
  3. truncation metadata와 streaming redaction의 경계 invariant를 직접 검증해야 한다.
  4. doctest/feature/target/CI 범위를 PASS 문구와 함께 명시해야 한다.
