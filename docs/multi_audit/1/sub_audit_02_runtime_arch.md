# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 1
- Perspective: A02 — 빌드·런타임·아키텍처·데이터 무결성
- User Goal: 프로젝트의 모든 문서 및 구현 내용을 파악한 뒤, 안정성·완성도를 막는 근본 원인과 우선순위를 증거로 찾아 안정화한다.
- Audit Basis: Standard-backed
- Standard Path: /mnt/Projects_SSD/rust/smlcli/AI_AUDIT_DOC_STANDARD.md
- Report Contract: /home/eunho1/.codex/skills/multi-audit/references/report-contract.md
- Repository: /mnt/Projects_SSD/rust/smlcli

## 2. Assigned Scope

이번 보고서는 다른 에이전트의 결론을 전제하지 않고 다음을 독립적으로 추적했다.

- src/main.rs의 CLI 파싱과 TUI/doctor/sessions/completions 진입
- App::new, App::run, 이벤트·액션 라우팅, 채팅 및 도구 라이프사이클
- Provider registry와 OpenAI-compatible/Anthropic/Gemini/LM Studio/custom provider 경로
- MCP child process, JSON-RPC, 로드·라우팅·종료 lifecycle
- 설정·암호화 키·세션 JSONL/index·RepoMap·Git checkpoint/auto-commit
- workspace trust/harness와 Linux bubblewrap 실행 경로
- Cargo manifest/lock, build script, CI/release workflow, 테스트 및 구현 문서의 실행 계약

적용 패턴: DBG-001, DBG-002, ARCH-001, BUILD-001, TEST-001, IMP-001, IMP-003.

## 3. Excluded and Uninspected Scope

- .git, target/, 캐시·생성 산출물과 peer docs/multi_audit/1/sub_audit_*.md는 읽지 않았다.
- 외부 Provider/MCP/API 호출, 유료 호출, build.sh 실행, Windows native build, 실제 배포/릴리스는 수행하지 않았다.
- 현재 작업트리의 사용자 변경 AGENTS.md, AI_AUDIT_DOC_STANDARD.md, AI_CODING_STANDARD.md, docs/는 보존했다.
- 실제 Provider wire compatibility와 Windows runtime은 미검증이다.
- git diff --check는 사용자 기존 변경 AGENTS.md:4-6의 trailing whitespace 때문에 실패했으며, 이번 감사에서 수정하지 않았다.

## 4. Evidence Examined

### Documents

- AGENTS.md, AI_AUDIT_DOC_STANDARD.md, AI_IMPLEMENTATION_DOC_STANDARD.md, AI_CODING_STANDARD.md
- spec.md, designs.md, DESIGN_DECISIONS.md, IMPLEMENTATION_SUMMARY.md, BUILD_GUIDE.md
- README.md, CHANGELOG.md, LESSONS_LEARNED.md, audit_roadmap.md
- Cargo.toml, Cargo.lock, build.rs, build.sh, scripts/check-version-sync.sh
- .github/workflows/ci.yml, .github/workflows/release.yml

### Source and tests

- src/main.rs
- src/app/{mod,event_loop,action,state,chat_runtime,command_router,tool_runtime,wizard_controller}.rs
- src/domain/{permissions,provider,repo_map,session,settings,tool_result,error}.rs
- src/providers/{registry,anthropic,types}.rs
- src/infra/{config_store,doctor,git_engine,mcp_client,process_reaper,sandbox,secret_store,session_log,workspace_harness,workspace_utils}.rs
- src/tools/{executor,registry,file_ops,git_checkpoint,grep,fetch,shell,sys_ops,questionnaire}.rs
- src/tests/{audit_regression,provider_validation,settings_flow,shell_permissions}.rs

### Commands and results

1. CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo check --all-targets --locked — PASS.
2. CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo clippy --all-targets --all-features --locked -- -D warnings — PASS.
3. CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo fmt --check — PASS.
4. bash scripts/check-version-sync.sh — PASS (Cargo.toml 3.9.0 == CHANGELOG 3.9.0).
5. CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo test --all-targets --locked --no-fail-fast — FAIL: 121 tests, 119 passed, 2 failed.
   - test_bwrap_mounts_workspace_as_canonical_guest_root: bwrap reports No permissions to create a new namespace.
   - test_session_harness_record_is_skipped_on_restore: InfraError, session log open failed with Read-only file system.
6. target/debug/smlcli --version — smlcli 3.9.0.
7. target/debug/smlcli run --help — Usage: smlcli run, no prompt argument.
8. target/debug/smlcli run explain-this-repository — exit 2, unexpected argument.
9. git diff --check — only the existing AGENTS.md whitespace failure noted above.

## 5. Findings

### [A02-F001] 문서의 비대화형 run prompt 진입 계약이 실제 CLI에 없다

- Pass: Implementation / Debug
- Pattern: IMP-001, BUILD-001
- Area: main entrypoint, CLI contract, runtime mode
- Severity: Major
- Status: Needs Fix (Confirmed)
- Summary: 명세는 smlcli run "explain this repository"를 비대화형 1회 실행으로 정의하지만, 실제 Run variant는 인자를 받지 않고 항상 TUI로 분기한다.
- Evidence:
  - spec.md:217-228은 run prompt 명령과 비대화형 1회 실행을 정의한다.
  - src/main.rs:39-42의 Commands::Run에는 필드가 없고, src/main.rs:68-71은 run_interactive().await로 보낸다.
  - target/debug/smlcli run --help는 Usage: smlcli run만 출력했고, run explain-this-repository는 exit 2였다.
- Expected Basis: spec.md 3.3 Entry Modes와 BUILD-001의 문서 명령-실행 경로 일치 조건.
- Actual: run은 prompt를 받을 수 없고 TUI 진입 alias일 뿐이다.
- Impact: 자동화/스크립트/헤드리스 사용자가 문서된 단일 작업을 실행할 수 없다.
- Suggested Action: Run에 optional prompt와 headless provider/tool loop를 구현하거나, 비대화형 지원을 범위에서 제거하고 모든 문서를 실제 semantics로 동기화한다.
- Re-audit Method: run --help, run prompt no-TTY smoke, mock provider 1회 응답/도구 결과/exit code와 README/spec/completion을 재대조한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: doctor, sessions, completions 분기는 별도로 동작한다.

### [A02-F002] Provider chat_stream이 전체 HTTP body를 버퍼링해 SSE 스트리밍을 수행하지 않는다

- Pass: Debug / Engineering Quality
- Pattern: DBG-001, TEST-001
- Area: provider transport, ChatDelta lifecycle, memory/latency
- Severity: Major
- Status: Needs Fix (Confirmed)
- Summary: OpenAI-compatible, Gemini, Anthropic의 streaming 구현이 Response::text().await로 EOF까지 기다린 뒤 줄을 순회하므로 ChatDelta가 실제 network chunk 도착 시점에 전달되지 않는다.
- Evidence:
  - src/providers/registry.rs:328-352 (OpenAI-compatible), :628-649 (Gemini), src/providers/anthropic.rs:375-402 모두 response.text().await 이후 delta_tx.send를 호출한다.
  - src/app/chat_runtime.rs:577-595의 forwarder는 provider가 body를 반환하기 전까지 delta를 받을 수 없다.
  - spec.md:205-206, :1424-1425는 SSE 토큰 단위 실시간 렌더링을 요구한다.
- Expected Basis: SSE는 증분 frame을 소비해 frame마다 ChatDelta를 발행해야 한다.
- Actual: 응답 전체가 메모리에 모인 뒤 모든 delta가 한꺼번에 발행된다.
- Impact: 핵심 실시간 UX가 완료 후 일괄 표시로 퇴행하고, 긴 응답의 메모리·timeout/취소 지연이 증가한다.
- Suggested Action: bytes_stream 또는 incremental line decoder, bounded buffer, DONE/EOF/cancellation 처리를 공통 transport helper로 구현한다.
- Re-audit Method: 지연된 local mock SSE가 EOF 전 ChatDelta를 발생시키는지, body cap과 cancellation이 동작하는지 검증한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: cfg(test) MockProvider의 즉시 delta 전송은 실제 reqwest transport를 검증하지 않는다.

### [A02-F003] App 종료와 MCP 초기 로드 사이 lifecycle 경계가 없어 child process가 orphan될 수 있다

- Pass: Debug / Engineering Quality
- Pattern: DBG-001
- Area: MCP process lifecycle, cancellation, graceful shutdown
- Severity: Major
- Status: Needs Fix (Confirmed)
- Summary: App::new가 MCP 로드 task를 detached spawn으로 만들고 task handle을 보관하지 않는다. 로드 완료 전 종료하면 App 종료 정리는 아직 mcp_clients에 등록되지 않은 child를 볼 수 없다.
- Evidence:
  - src/app/mod.rs:163-173에서 서버별 detached task를 만든다.
  - src/app/mod.rs:177-212에서 child를 spawn/list하고, :286-295에서 McpToolsLoaded를 보낸다. send 실패는 버린다.
  - src/app/mod.rs:422-428은 이미 RuntimeState.mcp_clients에 삽입된 client만 shutdown한다.
  - src/infra/mcp_client.rs:135-142의 shutdown은 caller가 호출해야 하며 startup task 소유/join 구조가 없다.
- Expected Basis: Quit/SIGTERM 시 시작 task와 그 child가 모두 취소·join·shutdown되어야 한다.
- Actual: Quit가 먼저 처리되면 mcp_clients가 비어 있을 수 있고, receiver close 후 send 실패만 발생하며 client.shutdown은 호출되지 않는다.
- Impact: MCP 서버가 앱 종료 후 살아 resource/port를 점유하고 다음 실행의 orphan reaper 대상이 된다.
- Suggested Action: JoinSet/task registry와 공용 CancellationToken으로 startup을 소유하고 종료 시 취소→join→client shutdown을 수행한다. event send 실패도 finally shutdown으로 감싼다.
- Re-audit Method: 느린 mock MCP PID를 기록하고 load 완료 전 Quit/SIGTERM 후 PID·task가 사라지는지, 정상/list failure/EOF도 확인한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: list_tools 실패 경로는 명시적 shutdown하지만 Quit 중 send 실패 경로는 별도 정리가 없다.

### [A02-F004] Provider registry reload가 custom adapter와 LM Studio base URL을 소거한다

- Pass: Debug / Implementation
- Pattern: ARCH-001, IMP-001
- Area: provider registry, runtime configuration source of truth
- Severity: Major
- Status: Needs Fix (Confirmed)
- Summary: 설정 로드 시 등록한 custom providers와 LM Studio URL이 reload_providers 호출에서 기본 registry로 교체된다.
- Evidence:
  - src/app/state.rs:158-162는 startup에서 custom providers와 lmstudio_base_url을 등록한다.
  - src/providers/registry.rs:780-800의 new는 custom_adapters 빈 map과 localhost 기본 LM Studio URL을 생성한다.
  - src/providers/registry.rs:957-959의 reload_providers는 registry 전체를 new로 덮는다.
  - src/app/wizard_controller.rs:500-529의 모델 선택 완료 경로는 reload만 호출한다.
  - src/providers/registry.rs:863-868은 누락된 Custom(id)를 OpenAI adapter로 fallback한다.
- Expected Basis: custom provider와 PersistedSettings.lmstudio_base_url은 runtime 호출자와 같은 설정 source of truth를 사용해야 한다.
- Actual: 모델 변경 또는 wizard 저장 뒤 custom 호출은 OpenAI로 fallback하고 LM Studio custom URL은 기본 localhost로 돌아간다.
- Impact: 잘못된 endpoint/header로 요청되거나 로컬 서비스 대신 외부 OpenAI endpoint가 호출될 수 있다.
- Suggested Action: reload_providers(settings)가 registry 생성과 custom/LM URL 재등록을 atomic transition으로 수행하게 하고, Custom 누락은 명시적 오류로 종료한다.
- Re-audit Method: custom provider와 비기본 LM Studio URL을 등록한 뒤 /config 모델 선택, /provider remove, wizard save 후 mock 수신 URL/header를 확인한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: /provider remove 경로는 별도로 재등록하지만 모델 선택/wizard save에는 같은 보장이 없다.

### [A02-F005] 손상된 encrypted key의 nonce 길이 검증 부재로 복구 경로가 panic할 수 있다

- Pass: Debug / Data Integrity
- Pattern: DBG-001
- Area: secret/config recovery, panic safety
- Severity: Major
- Status: Needs Fix (Probable)
- Summary: 암호화 값의 nonce를 hex decode한 뒤 길이를 확인하지 않고 XNonce::from_slice에 전달한다.
- Evidence:
  - src/infra/secret_store.rs:101-109는 nonce/ciphertext의 형식과 hex만 확인한다.
  - src/infra/secret_store.rs:111-117에서 XNonce::from_slice(&nonce_bytes)를 길이 검증 없이 호출한다. 고정 24-byte 타입의 길이 불일치는 panic 경로다.
  - src/app/chat_runtime.rs:153-175와 src/app/mod.rs:1693-1701이 get_api_key를 호출한다.
  - src/app/state.rs:141-155, :717-756은 손상 config 복구 안내 경로를 정의한다.
- Expected Basis: 손상 설정은 구조화된 error로 wizard recovery에 도달해야 한다.
- Actual: nonce 길이 불일치가 from_slice assertion에 도달할 수 있어 TUI/채팅이 중단될 수 있다.
- Impact: 복구 안내 전에 앱이 종료되고 손상 config가 반복 crash를 일으킨다.
- Suggested Action: nonce 24 bytes와 ciphertext 최소 길이를 검사해 SmlError를 반환하고 malformed fixture를 추가한다.
- Re-audit Method: 길이 0/1/23/25 nonce와 잘린 ciphertext에서 panic 없이 actionable error가 나오는지 확인한다.
- Owner: Coder
- Confidence: Medium-High
- Notes: 실제 malformed fixture 실행은 재감사에서 직접 잠가야 한다.

### [A02-F006] 첫 채팅 요청이 RepoMap을 이벤트 루프에서 동기 생성해 TUI를 멈출 수 있다

- Pass: Debug / Engineering Quality
- Pattern: DBG-002
- Area: RepoMap scheduling, UI responsiveness, duplicate work
- Severity: Major
- Status: Needs Fix (Confirmed)
- Summary: 문서는 blocking worker 분리를 선언하지만 spawn_chat_request가 repo_map_dirty이면 UI event loop 안에서 동기 generate_repo_map을 실행한다.
- Evidence:
  - src/app/chat_runtime.rs:548-558에서 요청 직전 generate_repo_map(&cwd)를 직접 호출한다.
  - src/domain/repo_map.rs:210-238에는 spawn_blocking 기반 generate_repo_map_async가 별도로 있다.
  - src/app/mod.rs:440-467은 background action channel refresh도 제공한다.
  - spec.md:952-953은 UI/input loop가 RepoMap scan을 기다리지 않아야 한다고 명시한다.
- Expected Basis: 비용 있는 context generation은 background worker/cache를 사용하고 input loop를 차단하지 않아야 한다.
- Actual: startup scan 미완료 또는 write/shell 후 dirty 상태에서 첫 요청이 depth 10 AST scan을 동기 수행하며 background scan과 중복될 수 있다.
- Impact: 큰 저장소에서 입력·tick·렌더링이 멈추고 CPU/latency가 증가한다.
- Suggested Action: 요청 경로는 준비된 cache만 사용하고 dirty면 background result를 기다리는 상태를 명시한다. generation revision으로 오래된 결과 overwrite도 막는다.
- Re-audit Method: 큰 Rust fixture에서 startup/write 직후 key-to-request latency와 event loop blocking 여부를 tracing으로 검증한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: AST 추출 정확성보다 호출 scheduling이 본 finding의 범위다.

### [A02-F007] malformed tool call이 outstanding count에 포함되지 않아 조기 재전송·중복 요청을 만든다

- Pass: Debug / Engineering Quality
- Pattern: DBG-001, TEST-001
- Area: tool turn state machine, ordering, result aggregation
- Severity: Major
- Status: Needs Fix (Confirmed)
- Summary: 한 provider 응답에 malformed와 정상 tool call이 섞이면 malformed ToolError가 먼저 channel에 들어가지만 pending_tool_executions에는 정상 호출 수만 더해진다.
- Evidence:
  - src/app/tool_runtime.rs:36-60은 parse 실패마다 ToolError를 try_send하고 valid_tools에는 넣지 않는다.
  - src/app/tool_runtime.rs:83-89은 valid_tools.len()만 counter에 더한다.
  - src/app/mod.rs:1338-1350의 ToolError는 counter를 감소시키고 0이면 flush와 send_chat_message_internal을 실행한다.
- Expected Basis: 한 turn의 모든 call/result가 끝난 뒤 정확히 한 번만 flush/retry해야 한다.
- Actual: malformed event가 정상 tool 완료보다 먼저 처리되면 정상 실행 중 counter가 0이 되고 follow-up provider 요청이 발행된다.
- Impact: provider 중복 요청, 세션 순서·Auto-Verify 상한·UI block routing 오염.
- Suggested Action: 모든 call ID를 turn 시작 시 outstanding set에 등록하고 malformed도 terminal result로 집계한다. set empty에서만 flush/retry한다.
- Re-audit Method: malformed JSON + 정상 ReadFile/ExecShell mixed tool_calls fixture에서 follow-up 1회, result ordering, counter 0을 단언한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: 정상 call만 있는 기존 auto-verify test는 이 경계를 잠그지 않는다.

### [A02-F008] 첫 wizard 저장 후 Workspace Trust Gate가 재평가되지 않아 쓰기/셸이 조용히 차단된다

- Pass: Implementation / Debug
- Pattern: IMP-001, DBG-001
- Area: startup state transition, workspace trust, onboarding
- Severity: Major
- Status: Needs Fix (Confirmed)
- Summary: 최초 실행에서는 wizard가 열려 trust gate 검사를 건너뛰고, wizard 저장 성공 후에는 wizard만 닫는다. 새 settings의 root trust가 Unknown인 상태에서 도구는 거부되지만 popup은 나타나지 않는다.
- Evidence:
  - src/app/mod.rs:303-305의 check_trust_gate는 wizard가 열려 있으면 return한다.
  - src/app/mod.rs:102-109의 startup call은 wizard open 상태에서 한 번만 실행된다.
  - src/app/mod.rs:1366-1372의 WizardSaveFinished(Ok)는 reload와 is_wizard_open=false만 수행한다.
  - src/infra/workspace_harness.rs:213-222와 src/domain/permissions.rs:155-162는 Unknown trust를 Deny한다.
  - spec.md:248-257, :357-358은 최초 workspace trust 선택을 요구한다.
- Expected Basis: wizard 완료 후 trust 선택 상태로 이어지거나 즉시 명시적 trust gate가 열려야 한다.
- Actual: 첫 실행 화면은 정상처럼 보이지만 tool call은 workspace trust is Unknown으로 거부된다. 사용자가 /workspace trust를 찾아야 한다.
- Impact: onboarding의 설정→작업 흐름이 닫히지 않고 보안 gate가 silent failure가 된다.
- Suggested Action: wizard save success 후 canonical root snapshot을 재계산해 trust popup을 열거나 trust 선택을 setup 필수 단계로 편입한다.
- Re-audit Method: 격리 home에서 wizard save 후 popup과 WriteFile/ExecShell preflight 및 Trust Once/Remember/Restricted persistence를 검증한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: /workspace trust 명령이 존재한다는 사실은 자동 gate 누락을 해소하지 않는다.

### [A02-F009] bubblewrap 설치 여부만 감지하고 user-namespace 실행 가능성을 검증하지 않아 sandbox 진단과 테스트가 불결정적이다

- Pass: Debug / Build / Test
- Pattern: BUILD-001, TEST-001
- Area: Linux sandbox capability, doctor, release gate
- Severity: Major
- Status: Needs Fix (Confirmed; environment-dependent)
- Summary: backend detection은 bwrap --version 성공만 확인한다. 현재 환경에는 bwrap이 있지만 namespace 생성 권한이 없어 필수 regression test가 실패했고 doctor는 설치됨으로 표시할 수 있다.
- Evidence:
  - src/infra/sandbox.rs:3-9의 detect_backend는 version command만 확인한다.
  - src/infra/doctor.rs:64-69는 이를 bubblewrap 설치됨으로 표시한다.
  - src/tests/audit_regression.rs:1700-1724는 실제 namespace 실행을 무조건 성공해야 한다고 단언한다.
  - cargo test 결과는 bwrap: No permissions to create a new namespace로 실패했다.
- Expected Basis: BUILD-001은 runtime capability를 검증하고 doctor/CI가 설치 여부와 실행 가능 여부를 구분해야 한다.
- Actual: Sandbox Backend가 bubblewrap이어도 실제 격리가 불가능할 수 있고, 현재 전체 test gate는 119/121이다.
- Impact: sandbox enabled 시 모든 shell command가 실패할 수 있으며 CI/release 재현성이 환경 정책에 좌우된다.
- Suggested Action: no-op capability probe로 namespace/mount/chdir 가능성을 판정하고 snapshot/doctor에 unavailable reason을 표시한다. capability 없는 환경의 test는 명시적 skip/환경 실패로 분리한다.
- Re-audit Method: userns 허용, 현재 제한, bwrap 미설치 세 환경에서 doctor·command·test 결과를 비교한다.
- Owner: Architect / Coder / CI Owner
- Confidence: High
- Notes: sandbox 보안 강도 자체보다 실행 가능성 진단과 test gate의 재현성 finding이다.

### [A02-F010] Context compaction이 provider 실패 시 원본 대화를 복구하지 않고 오류 문구로 대체한다

- Pass: Debug / Data Integrity
- Pattern: DBG-002, TEST-001
- Area: session state, compaction transaction, failure rollback
- Severity: Major
- Status: Needs Fix (Confirmed)
- Summary: /compact는 LLM 요약 요청 전에 원본 메시지를 즉시 제거한다. 호출 실패 시 Summary Pending을 Fallback due to error로 바꾸므로 삭제된 대화가 복구되지 않는다.
- Evidence:
  - src/domain/session.rs:172-213은 self.messages에서 내용을 제거하고 marker를 삽입한다.
  - src/app/command_router.rs:1099-1114는 이 변경 뒤 비동기 provider 요약을 호출한다.
  - src/app/mod.rs:1414-1419의 ContextSummaryErr는 error fallback summary만 적용한다.
  - designs.md:830-835, spec.md:761-775는 목표 보존 요약 교환을 정의한다.
- Expected Basis: 실패 가능한 외부 호출 전 snapshot을 보존하고 성공 시에만 compaction을 commit해야 한다.
- Actual: timeout/network/auth/provider response 오류에서 과거 user/assistant/tool 메시지가 사라지고 오류 설명만 남는다.
- Impact: 장기 세션 목표·결정·도구 결과가 손실되어 이후 agent가 잘못된 작업을 수행할 수 있다.
- Suggested Action: pending compaction object에 원본을 보관하고 summary 성공/검증 뒤 atomic replace, 실패/취소 시 restore한다.
- Re-audit Method: timeout/429/invalid response mock에서 message IDs/content/pinned prompt 보존과 성공 시 commit을 검증한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: sliding-window drop은 별도 경로이므로 compaction rollback 테스트와 분리한다.

### [A02-F011] Git auto-commit이 이미 stage된 사용자 WIP를 AI 커밋에 혼입할 수 있다

- Pass: Debug / Data Integrity
- Pattern: TEST-001, ARCH-001
- Area: Git checkpoint/auto-commit, user data integrity
- Severity: Major
- Status: Needs Fix (Confirmed)
- Summary: GitEngine::auto_commit은 지정 경로를 git add한 뒤 index 전체를 commit한다. 기존에 사용자가 stage한 unrelated 파일을 분리하거나 snapshot/restore하지 않는다.
- Evidence:
  - src/infra/git_engine.rs:31-53은 files를 add하지만 기존 index를 초기화하지 않는다.
  - src/infra/git_engine.rs:55-83은 staged index 전체를 commit한다.
  - src/app/mod.rs:1158-1171은 affected_paths만 넘기지만 GitEngine 내부 staged 상태를 통제하지 않는다.
  - spec.md:2262-2269, :2286-2289는 affected paths만 stage해 WIP를 보호한다고 정의한다.
  - 기존 test src/tests/audit_regression.rs:3048-3079는 unrelated 파일을 unstaged로만 만들어 이 경계를 검증하지 않는다.
- Expected Basis: 선택적 자동 commit은 staged WIP를 분리하고 /undo가 사용자 변경을 되돌리지 않아야 한다.
- Actual: unrelated staged file이 target path와 함께 AI commit에 들어갈 수 있다.
- Impact: user data/history 오염과 /undo에 의한 사용자 변경 revert 위험.
- Suggested Action: temporary index 또는 index snapshot/restore로 target만 commit하고 pre-staged unrelated 회귀를 추가한다.
- Re-audit Method: 임시 repo에서 unrelated를 먼저 git add한 뒤 target만 auto_commit하고 commit tree와 post-commit index를 확인한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: clean worktree checkpoint 정책과 auto-commit index isolation은 별도 경계다.

### [A02-F012] Session logger 테스트가 실제 home에 결합되어 전체 Cargo test를 막고 사용자 데이터를 건드릴 수 있다

- Pass: Debug / Build / Test
- Pattern: BUILD-001, TEST-001
- Area: test isolation, persistence path, reproducibility
- Severity: Major
- Status: Needs Fix (Confirmed; environment-dependent)
- Summary: session 저장 경로가 dirs::home_dir에 고정되고 test가 temp root를 주입하지 않는다. 현재 read-only home에서 필수 test가 실패했다.
- Evidence:
  - src/infra/session_log.rs:265-269, :389-397은 실제 ~/.smlcli/sessions를 사용한다.
  - src/tests/audit_regression.rs:1853-1878은 new_workspace_session을 실제 home에서 호출한다.
  - cargo test 결과는 session log open failed: Read-only file system이었다.
- Expected Basis: TEST-001은 hermetic fixture와 deterministic storage seam을 요구하며 test가 user data 경계를 침범하면 안 된다.
- Actual: HOME/path injection 생성자와 test storage backend가 없고 CI/container home 권한에 따라 결과가 달라진다.
- Impact: CI quality gate가 환경에 따라 실패하고 개발자 ~/.smlcli 세션을 읽거나 cleanup할 수 있다.
- Suggested Action: SessionStoreRoot/path provider를 injection하고 test는 TempDir를 주입한다. env override가 필요하면 serial guard와 cleanup을 둔다.
- Re-audit Method: read-only home, writable temp root, 동시 test process에서 cargo test를 수행하고 실제 home 변화가 없는지 확인한다.
- Owner: Coder / CI Owner
- Confidence: High
- Notes: 이번 감사가 만든 /tmp/smlcli-multi-audit-1-target는 정리했으며 source/test는 수정하지 않았다.

### [A02-F013] 비동기 설정 저장에 순서/세대 계약이 없어 오래된 snapshot이 최신 설정을 덮을 수 있다

- Pass: Debug / Data Integrity
- Pattern: ARCH-001, TEST-001
- Area: config persistence, concurrent saves, rollback semantics
- Severity: Major
- Status: Needs Fix (Probable)
- Summary: 여러 UI 경로가 settings clone을 각각 detached task로 save_config에 넘긴다. file lock은 mutual exclusion만 제공하고 logical ordering을 보장하지 않는다.
- Evidence:
  - src/app/command_router.rs:473-494, src/app/wizard_controller.rs:330-402, src/app/command_router.rs:514-524는 변경마다 독립 tokio::spawn으로 clone을 저장한다.
  - src/infra/config_store.rs:47-68의 lock에는 revision/sequence/latest-wins 검사가 없다.
  - src/app/mod.rs:1379-1391은 실패만 표시하고 disk snapshot이 현재 in-memory state인지 검증하지 않는다.
- Expected Basis: runtime 설정과 persisted 설정은 ARCH-001의 단일 source of truth와 logical ordering을 가져야 한다.
- Actual: 빠른 연속 toggle/save에서 task completion 순서가 역전되면 오래된 clone이 마지막 rename되어 최신 설정을 덮을 수 있다.
- Impact: 재시작 후 provider/policy/theme/sandbox가 되돌아가 runtime과 disk가 달라진다.
- Suggested Action: ConfigStore 단일 writer/command queue 또는 monotonic revision을 도입하고 stale completion을 무시하거나 최신 snapshot을 재저장한다.
- Re-audit Method: delayed fake store에서 A→B→C 저장을 역순 완료시키고 disk/in-memory 최종 상태와 completion 처리 결과를 확인한다.
- Owner: Architect / Coder
- Confidence: Medium
- Notes: fs2 lock과 temp rename은 file corruption 방지에는 유효하지만 logical ordering 문제를 해결하지 않는다.

## 6. Uncertainties and Clarifications Needed

### Metadata authority conflict

- AGENTS.md:5-6 및 project-doc 블록은 doomlike / 0.5.1로 표시되어 있다.
- 실행 계약과 release 문서는 Cargo.toml:2-3, README.md, spec.md:1,20-35, CHANGELOG.md:15의 smlcli / 3.9.0을 사용한다.
- scripts/check-version-sync.sh는 Cargo와 Changelog 3.9.0 일치를 PASS했다. source에서 doomlike/0.5.1을 runtime/build 계약으로 읽는 경로는 확인되지 않았다.
- 따라서 실행·빌드에 대한 직접 영향은 확인하지 못했지만 문서 authority는 Needs Spec Clarification이다. AGENTS metadata를 임의로 수정하지 않고 project owner가 canonical identity를 결정해야 한다.

### Test/release claim drift

- 현재 test target은 121개이며 119개가 통과했다.
- spec/implementation 문서의 여러 Phase closure는 105/108/102개 전체 통과를 주장한다. 당시 snapshot일 수 있으나 기준 시점이 없어 현재 release evidence로 사용할 수 없다.
- 재감사에서 현재 test count, environment-dependent test, pass/fail SHA를 동결해야 한다.

### Scope boundary for network/security

- 실제 provider/MCP/FetchURL network 호출은 수행하지 않았다. non-loopback, shell network, MCP startup side effect, SSRF 등은 A03 독립 보고서와 최종 통합에서 별도 교차검증이 필요하다.
- A02는 해당 경계의 lifecycle/config/cancellation 연결만 판정했다.

## 7. Perspective Decision

HOLD (A02 범위).

cargo check, clippy, fmt --check, version-sync는 통과했지만 핵심 runtime 계약과 데이터 무결성 경로에 Major finding이 남아 있다. 특히 실제 SSE 비스트리밍, MCP 조기 종료 누수, provider registry reset, compaction 실패 데이터 손실, staged WIP 혼입은 구조적 수정과 회귀 검증 없이는 안정화 완료로 볼 수 없다. 전체 test gate도 현재 환경에서 119/121로 실패했으므로 이 관점에서 PASS를 부여하지 않는다.

## 8. Re-audit Checklist

- [ ] run prompt contract를 구현 또는 문서에서 닫고 no-TTY smoke를 통과시킨다.
- [ ] delayed SSE fixture에서 EOF 전 ChatDelta를 검증한다.
- [ ] MCP startup/task registry와 shutdown join으로 모든 종료 경로의 child PID 회수를 확인한다.
- [ ] provider reload 후 custom adapter와 LM Studio URL 유지 여부를 mock endpoint로 확인한다.
- [ ] malformed encrypted key가 panic 없이 복구 error로 전달되는지 확인한다.
- [ ] RepoMap이 input/event loop를 차단하지 않는지 큰 fixture로 검증한다.
- [ ] malformed + valid tool call mixed turn의 follow-up이 1회이고 outstanding set이 0인지 확인한다.
- [ ] wizard 완료 후 trust popup/명시적 trust 선택과 write/shell preflight를 검증한다.
- [ ] bwrap capability probe와 제한/허용 환경의 test/doctor 결과를 분리한다.
- [ ] compaction 실패/취소 시 원본 메시지와 pinned context가 복구되는지 확인한다.
- [ ] pre-staged unrelated Git WIP가 auto-commit에 포함되지 않는지 확인한다.
- [ ] session/config persistence에 injectable storage와 ordered writer를 적용한다.

## 9. Coder Handoff

/mnt/Projects_SSD/rust/smlcli/docs/multi_audit/1/sub_audit_02_runtime_arch.md를 먼저 읽고, 각 finding을 현재 프로젝트 문서·실제 코드·테스트에 대조하여 검증한 뒤 우선순위대로 수정하세요. 계약 변경이 필요하면 관련 문서를 먼저 갱신하고, 수정 후 지정된 Cargo build/test/clippy와 각 finding의 재감사 명령 결과를 기록하세요.

