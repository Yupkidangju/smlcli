# Final Multi-Audit Report — Turn 1

> 최종 판정: **HOLD**
>
> 현재 트리는 5개 Critical, 19개 Major, 1개 Minor의 통합 finding을 가진다. 확인된 Critical에는 Git rollback 데이터 손상, MCP 환경 secret 상속, @file 외부 파일 노출, deterministic temp symlink 외부 쓰기, custom provider credential 오배선이 포함된다. 전체 테스트는 현재 감사 환경에서 121개 중 119개만 통과했고, 로컬 RustSec DB 기반 cargo audit도 종료 코드 1을 반환했다.

## 1. Audit Metadata

- Audit Turn: 1
- Audit Date: 2026-08-23 (Asia/Seoul)
- Project Root: /mnt/Projects_SSD/rust/smlcli
- Git Branch / HEAD: main / 55d1c33
- Audit Mode: Standard-backed multi-agent whole-project audit
- Standard: /mnt/Projects_SSD/rust/smlcli/AI_AUDIT_DOC_STANDARD.md
- Multi-audit Contract: /home/eunho1/.codex/skills/multi-audit/references/report-contract.md
- Final Report: /mnt/Projects_SSD/rust/smlcli/docs/multi_audit/1/final_audit_report_1.md
- Initial tracked-file aggregate SHA-256: e52852a541015c968d282a56733f7e823aee86219b1e9b2ef4706f461a8670c8
- Integration-time tracked-file aggregate SHA-256: e52852a541015c968d282a56733f7e823aee86219b1e9b2ef4706f461a8670c8
- Preserved pre-existing worktree: modified AGENTS.md and AI_AUDIT_DOC_STANDARD.md, untracked AI_CODING_STANDARD.md
- Source/config/product changes by auditor: none
- Final Decision: HOLD

## 2. User Goal and Decision Basis

사용자 목표는 프로젝트의 모든 문서와 구현을 먼저 파악한 뒤 전체 문제를 독립 관점으로 상세 감사하고, 안정화와 완성도 향상을 위한 근본 수정 순서까지 도출하는 것이다.

판정 기준은 다음 우선순위를 사용했다.

1. 현재 사용자 목표와 플랫폼 안전 규칙
2. spec.md 및 승인된 설계·결정 문서
3. 현재 AGENTS.md
4. 실제 Cargo 매니페스트, 호출 경로, 테스트, 빌드·CI 증거
5. 과거 audit_report_*.md는 당시 snapshot 증거로만 사용

문서에 없는 요구사항은 만들지 않았다. 요구가 불명확한 항목은 Needs Spec Clarification으로 유지했고, 코드상 안전 불변조건이 명백한 path, secret, process, data-integrity 경계는 일반 공학 기준으로 판정했다.

## 3. Scope and Exclusions

### Included

- 프로젝트 소유 문서와 표준, 기존 감사 계보
- src/app, src/domain, src/infra, src/providers, src/tools, src/tui, src/tests 전체 작업면
- Cargo.toml, Cargo.lock, build.rs, build.sh, .cargo/config.toml
- GitHub CI/release workflow, version-sync 및 MCP mock script
- provider, MCP, network, file, path, shell, Git, secret, config, session 신뢰 경계
- TUI keyboard/mouse/focus/overlay/responsive/Unicode/i18n 표면
- dependency advisory, target matrix, release provenance, 저장소 위생

### Excluded or not directly executed

- .git 내부 객체와 target/generated cache
- 실제 외부 LLM/provider/FetchURL/DNS/redirect 호출
- 유료 API, 배포, release upload, destructive fixture
- 실제 Windows/MSVC, Linux musl, 물리 TTY 수동 검증
- 로컬 advisory DB 2026-07-17 snapshot 이후의 최신 advisory
- reference PNG/HTML의 픽셀 품질과 project-local agent skill 기능 자체

이 제외 범위는 PASS로 간주하지 않는다. 핵심 범위의 미검증 항목은 Coverage Matrix에서 Partially Covered로 표시한다.

## 4. Work-Surface Inventory

| Surface | Observed inventory | Audit treatment |
| --- | ---: | --- |
| Tracked files | 174 | 전체 분류 |
| Core/project Markdown and text docs | 24 | A01 중심, A04/A06 교차 |
| Source files excluding test modules | 57 | A02/A03/A04 중심 |
| Dedicated test modules | 5 | A05 중심 |
| Rust/script line count | 약 23,315 | symbol/call-path audit |
| Core control/product docs | 약 11,296 lines | authority/drift audit |
| GitHub workflows | 2 | CI/release audit |
| Reference HTML/PNG | 16 tracked assets | runtime 제외, shipped-scope 확인 |
| Project agent-support files | 56 | runtime 제외, hygiene 확인 |
| High-risk boundaries | file/path/shell/network/provider/MCP/Git/secret/session | 최소 2개 독립 증거 확보 |

주요 런타임 흐름은 main.rs → App/EventLoop → chat_runtime/tool_runtime → provider/MCP/tool adapters → config/session/Git/sandbox storage로 추적했다. TUI는 AppState의 domain/session과 ui.timeline이 분리되어 있고, layout/widgets가 별도 렌더링한다.

## 5. Agent Allocation and Rationale

가용 동시성 안에서 6개 독립 관점을 사용했다. 파일 수가 아니라 서로 다른 핵심 질문을 기준으로 배정했다.

| Agent | Perspective | Distinct question |
| --- | --- | --- |
| A01 | 계약·문서·기능 정합성 | 현재 authority와 완료 주장이 실제 소비 경로와 일치하는가 |
| A02 | 런타임·아키텍처·데이터 무결성 | 부팅·상태·provider·tool·storage 흐름이 안정적인가 |
| A03 | 보안·신뢰 경계·복원력 | 공격 가능한 file/network/process/secret 경계가 fail-closed인가 |
| A04 | TUI·접근성·i18n | 보이는 UI와 실제 입력·상태·locale 계약이 일치하는가 |
| A05 | 테스트·동시성·실패 복원 | 테스트가 실제 failure mode와 side effect를 잠그는가 |
| A06 | 빌드·릴리스·공급망 | locked/cross-target/release provenance gate가 닫혔는가 |

Coverage Gap Check 후 A02, A03, A05에 각각 한 번의 supplement를 요청했다. 봉인된 원본은 변경하지 않았다.

## 6. Immutable Source Report Manifest

- Manifest: /mnt/Projects_SSD/rust/smlcli/docs/multi_audit/1/source_report_manifest.json
- Manifest SHA-256: f154af8f2004732606a834a332163190ae2d3f20fffc6fa8ade2d7e19e2b3e4d
- Sidecar: /mnt/Projects_SSD/rust/smlcli/docs/multi_audit/1/source_report_manifest.sha256.json
- missing_source_reports: []

| Immutable report | Size | SHA-256 |
| --- | ---: | --- |
| sub_audit_01_contract_docs.md | 44,337 | 28f800b181bf44133b52cfd203df41b7145d0fd36abb6a61afde521b4e509488 |
| sub_audit_02_runtime_arch.md | 30,533 | 8d4f40e870e058cc5aeb97ba3d4821028f020fe28e299e4b889d62173eabff20 |
| sub_audit_02_runtime_arch_supplement_1.md | 22,492 | a748ff13d743288e9824b8229d415d0d0724b31f4f485b0390e15e07cb707b90 |
| sub_audit_03_security_resilience.md | 68,371 | 15376b719ec6823f075a2fde1dd71e1457972bc831f78eeb27994caeaaf67deb |
| sub_audit_03_security_resilience_supplement_1.md | 18,251 | 59ccc7b3046c7c0872ac77f6b7e77cd8eac44d6575ad7bf4d093c16762b78de8 |
| sub_audit_04_tui_ux_i18n.md | 48,055 | 79c2ab75ba49ed33a321f0eaf82b7671985fa904740452967637db0e4c598ac9 |
| sub_audit_05_tests_concurrency.md | 38,114 | 0300c2b14daf18eb052360a8efdd9b6e1d7027125583575ede910c3193b54515 |
| sub_audit_05_tests_concurrency_supplement_1.md | 17,780 | 8f31c3617dc3310f9d4548bbef50c8eb14e8fcf0223b84a4b994eb533826be7d |
| sub_audit_06_release_supplychain.md | 35,132 | e9ee229d4a19867cc06d126595e1c20a661b8cd80978acc64e45a0372f86738e |

## 7. Evidence and Commands

| Evidence / command | Result | Interpretation |
| --- | --- | --- |
| cargo metadata --no-deps --format-version 1 | PASS | package smlcli 3.9.0, edition 2024, one bin target |
| cargo fmt --check | PASS | source format gate only |
| cargo check --all-targets --locked | PASS in A02 isolated target | compile gate only |
| cargo clippy --all-targets --all-features --locked -- -D warnings | PASS in A02 isolated target | lint gate only |
| scripts/check-version-sync.sh | PASS | Cargo/CHANGELOG 3.9.0 only; Phase/tag authority 미검증 |
| full cargo test isolated target | FAIL | 121 total, 119 passed, 2 failed |
| current prebuilt test binary rerun by main | FAIL | same two failures reproduced |
| cargo audit --no-fetch | FAIL | 2 vulnerabilities, 3 unsound warnings; DB snapshot 2026-07-17 |
| cargo deny offline | Not Covered | config/cache 부족 |
| host cargo build --release --locked | PASS in A06 | glibc host artifact only |
| musl/MSVC build | Not Covered | target/cache/network constraints |
| git diff --check | FAIL | pre-existing AGENTS.md hard-break whitespace |
| source secret-pattern scan | no obvious plaintext token hit | exhaustive secret assurance 아님 |
| report_integrity finalize/verify | PASS | 9 reports verified, missing 0 |

메인 모델의 별도 fresh Cargo build는 /tmp disk quota 초과로 중단되었다. 따라서 A02/A06의 isolated build 결과와 메인의 source re-open 및 기존-current test binary 결과를 구분해 기록한다.

## 8. Coverage Gap Check

| Work surface / audit question | Independent evidence | Coverage | Remaining gap |
| --- | --- | --- | --- |
| Product authority and document-code sync | A01, A02, A04, A06 | Covered | human identity/version decision |
| Main/CLI/provider/tool runtime | A01, A02, A03 | Covered | real external provider wire |
| Build/lint/test reproducibility | A02, A05, A06 | Covered | clean build constrained by quota |
| File/path/write/destructive boundary | A02 supplement, A03, A05 supplement | Covered | destructive race fixture not executed |
| Shell/sandbox/process lifecycle | A02, A03, A05 | Covered | restricted vs userns-enabled host comparison |
| Secret/config/session integrity | A02, A03, A05 | Covered | real user data intentionally unread |
| Network/provider/MCP trust boundary | A02, A03, A05 supplement | Covered | live DNS/redirect/provider not executed |
| TUI/focus/responsive/Unicode/i18n | A01, A04, A05 | Covered | physical TTY Partially Covered |
| Performance/concurrency/state transactions | A02, A04, A05 | Covered | profiler/large-repo runtime absent |
| Dependencies/CI/release/provenance | A01, A03, A06 | Covered | live runner, current advisory refresh absent |
| Windows/MSVC and Linux musl | A02, A06 | Partially Covered | actual target build/runtime absent |
| Reference assets/support skills | A01, A06 | Excluded from product runtime | shipped-scope policy still required |

핵심 목표나 Critical/Major 가능 영역을 Excluded로 처리하지 않았다. 물리·외부 환경 공백은 HOLD 상태의 residual uncertainty로 유지한다.

## 9. Canonical Findings

### [FIN-F001] 자동 Git 복구·커밋이 사용자 변경을 손상하거나 혼입할 수 있음

- Sources: A03-F013, A03-F014, A02-F011
- Areas: Git checkpoint, rollback, auto-commit, data integrity
- Severity: Critical
- Status: Confirmed
- Summary: checkpoint ref를 만들지만 rollback은 ref를 사용하지 않고 현재 HEAD에 git reset --hard를 실행한다. auto-commit은 이미 stage된 사용자 파일까지 index 전체로 커밋한다.
- Verified Evidence: src/tools/git_checkpoint.rs:182-266, src/tools/executor.rs:17-55, src/infra/git_engine.rs:31-84를 메인이 재확인했다.
- Expected Basis: 실패 복구는 pre-tool snapshot과 affected paths만 복원하고 사용자 WIP/index를 보존해야 한다.
- Actual: checkpoint 이후의 concurrent tracked change가 삭제될 수 있고, pre-staged WIP가 AI commit 및 후속 undo에 포함될 수 있다. rollback 실패 결과도 무시한 채 성공 문구를 붙일 수 있다.
- Impact: 사용자 코드·history의 비가역 손실과 잘못된 복구 신뢰.
- Required Action: 자동 hard reset/auto-commit을 우선 비활성 또는 Ask-only로 격리하고, immutable checkpoint hash + path-scoped transaction + index snapshot/restore로 재설계한다.
- Re-audit Method: concurrent tracked WIP, pre-staged unrelated file, HEAD 변경, merge conflict, failed rollback을 temp repo에서 hash/diff로 검증한다.
- Synthesis Rationale: 정상 selective-staging 테스트 PASS는 pre-staged index와 concurrent change를 다루지 않아 finding을 반증하지 못한다.

### [FIN-F002] MCP child가 parent 환경 secret을 그대로 상속함

- Sources: A03-F015, A05-F013
- Areas: MCP process, environment, secret egress
- Severity: Critical
- Status: Confirmed
- Summary: McpClient::spawn은 env_clear/allowlist 없이 외부 command를 시작한다.
- Verified Evidence: src/infra/mcp_client.rs:41-48과 비교 경로 src/tools/shell.rs:168-180을 메인이 재확인했다.
- Expected Basis: 외부 executable은 provider/API/cloud credential을 자동 상속하지 않아야 한다.
- Actual: parent의 모든 환경변수가 child에 전달되며 이를 차단하는 테스트가 없다.
- Impact: third-party 또는 손상된 MCP server가 API key/cloud token을 읽고 유출할 수 있다.
- Required Action: env_clear 후 최소 non-secret allowlist, per-server explicit opt-in, bounded/redacted stderr, command provenance를 적용한다.
- Re-audit Method: sentinel 환경변수를 둔 env-dump MCP fixture에서 sentinel 부재와 정상 initialize/list/call을 확인한다.
- Synthesis Rationale: MCP가 완전 신뢰 executable인지 명세는 불명확하지만, secret 자동 상속은 hard boundary로 허용할 수 없다.

### [FIN-F003] @file 전처리가 workspace 밖 파일을 읽어 session/provider context에 넣음

- Sources: A03-F026, A05-F011
- Areas: file mention, workspace boundary, provider egress, session logging
- Severity: Critical
- Status: Confirmed with narrowed claim
- Summary: raw @path를 tokio::fs::read_to_string으로 직접 읽어 workspace, symlink, size, binary 검사를 우회한다.
- Verified Evidence: src/app/chat_runtime.rs:270-324와 provider guard 순서 :328-494를 메인이 재확인했다.
- Expected Basis: spec의 @ mention은 workspace picker와 bounded file context를 전제로 한다.
- Actual: absolute/parent/symlink path와 대형 UTF-8 파일이 full user message에 들어간다. 정상 provider policy가 outbound를 허용하면 외부 provider로 전송되고 JSONL에도 남는다.
- Impact: 임의 local file disclosure, local history 잔존, memory/token exhaustion.
- Required Action: ReadFile과 공유하는 canonical workspace capability, symlink containment, binary/byte/per-turn cap을 적용하고 guard 성공 전 session/log에 기록하지 않는다.
- Re-audit Method: inside/outside/parent/symlink/NUL/oversized fixtures와 Deny/ProviderOnly/AllowAll 각각의 session/provider side effect를 검증한다.
- Synthesis Rationale: A05가 NetworkPolicy::Deny 자체의 HTTP 우회는 기각한 것이 맞다. 본 finding은 Deny 우회가 아니라 file boundary 우회와 network-allowed egress를 판정한다.

### [FIN-F004] deterministic sibling temp symlink로 workspace 밖 파일을 덮어쓸 수 있음

- Sources: A03-F001, A03-F027, A05-F012
- Areas: WriteFile, ReplaceFileContent, symlink, atomic write
- Severity: Critical
- Status: Confirmed
- Summary: write_file_commit은 canonical target 옆의 고정 .tmp 경로를 fs::write로 열어 pre-existing symlink를 따라간 뒤 rename한다.
- Verified Evidence: src/tools/file_ops.rs:179-225를 메인이 재확인했다.
- Expected Basis: 모든 temporary/final file operation이 workspace 아래에서 no-follow로 고정되어야 한다.
- Actual: target.tmp → outside/sentinel symlink가 있으면 외부 대상이 먼저 truncate/write되고, 이후 workspace target은 해당 symlink로 치환된다.
- Impact: 악성 repository가 승인된 정상 파일 쓰기를 임의 external overwrite로 전환할 수 있다.
- Required Action: validated parent dirfd 안에서 random unique create-new + no-follow temp를 열고 inode/type 검증, fsync, atomic rename을 수행한다.
- Re-audit Method: pre-existing/dangling/concurrent temp symlink와 외부 sentinel hash/mode 불변을 검증한다.
- Synthesis Rationale: 두 독립 보고서가 같은 code order와 symlink semantics를 확인했다. arbitrary external write이므로 source severity Major보다 통합 severity를 Critical로 상향한다.

### [FIN-F005] custom provider가 key와 request를 OpenRouter/OpenAI/Google로 오배선할 수 있음

- Sources: A02-F004, A03-F019, A03-F028
- Areas: provider registry, credential routing, endpoint trust
- Severity: Critical
- Status: Confirmed
- Summary: persisted Custom provider를 main credential resolver가 인식하지 않고, reload는 custom map을 지우며, missing adapter는 OpenAI로 fallback한다. custom Gemini는 configured base URL을 무시한다.
- Verified Evidence: src/app/chat_runtime.rs:139-240, src/app/wizard_controller.rs:408-529, src/providers/registry.rs:780-870을 메인이 재확인했다.
- Expected Basis: missing/unsupported custom adapter는 fail-closed하고 key는 configured endpoint에만 전달되어야 한다.
- Actual: 정상 model selection 후 custom adapter가 사라질 수 있고, request/key가 다른 vendor endpoint로 전송되거나 custom provider가 작동하지 않는다.
- Impact: credential 및 prompt/session data의 제3자 유출.
- Required Action: typed custom lookup 한 경로로 통합하고 settings-aware atomic reload, missing-adapter error, unsupported dialect rejection, redirect/host policy를 적용한다.
- Re-audit Method: endpoint recorder로 add/select/save/reload/chat을 수행하고 모든 request URL/header를 확인하며 unknown/Gemini custom은 zero-request로 실패시킨다.
- Synthesis Rationale: release cfg에서만 실제 adapter가 사용되고 tests는 MockProvider로 대체되어 기존 PASS가 이 결함을 숨긴다.

### [FIN-F006] MCP 요청·schema·process lifecycle이 bounded/cancellable하지 않음

- Sources: A02-F003, A03-F016, A03-F017, A03-F018, A05-F004
- Areas: MCP lifecycle, cancellation, schema, output cap, approval
- Severity: Major
- Status: Confirmed
- Summary: startup task를 supervisor가 소유하지 않고, routed call은 cancellation token을 무시하며, stdout/stderr/result/schema 크기와 call-time schema 검증이 없다.
- Verified Evidence: src/infra/mcp_client.rs:41-231, :261-327, src/app/tool_runtime.rs:409-476, src/app/mod.rs:163-212 및 :422-428을 재확인했다.
- Impact: cancel/quit 후 mutation 지속, orphan descendants, memory exhaustion, opaque approval.
- Required Action: supervised JoinSet/lease/process group, cancellation-aware request, size/count cap, schema validation과 redacted argument approval을 구현한다.
- Re-audit Method: hanging/EOF/child-spawning/huge-line/malformed-schema fixtures에서 bounded shutdown, empty pending map, zero descendants를 확인한다.
- Synthesis Rationale: 정상 MCP E2E와 isError parsing PASS는 hostile lifecycle을 검증하지 않는다.

### [FIN-F007] FetchURL이 SSRF와 응답 크기·UTF-8 경계를 닫지 못함

- Sources: A03-F002, A03-F003, A05-F007
- Areas: SSRF, redirect/DNS, memory, result integrity
- Severity: Major
- Status: Confirmed
- Summary: AllowAll에서 private/link-local/metadata/redirect/DNS 목적지를 검사하지 않고, chunk를 append한 뒤 cap을 검사하며 truncation metadata도 false다.
- Verified Evidence: src/tools/fetch.rs:46-115를 재확인했다. main 직접 검토에서 byte index 10,000의 String::truncate가 UTF-8 boundary가 아니면 panic 가능한 점도 확인했다.
- Impact: internal service/cloud metadata 노출, memory/CPU pressure, partial 결과의 완전성 오인, Unicode panic.
- Required Action: public-network policy, DNS/redirect hop 재검증, timeout, pre-allocation byte cap, char-safe truncation, typed metadata를 적용한다.
- Re-audit Method: local mock으로 loopback/private/IPv6/redirect/rebinding/large-chunk/CJK boundary를 검증한다.
- Synthesis Rationale: NetworkPolicy enum 테스트는 URL 목적지와 response behavior를 전혀 실행하지 않는다.

### [FIN-F008] shell/sandbox/process 경계가 fail-open이며 process identity도 약함

- Sources: A03-F004, A03-F005, A03-F006, A03-F007, A03-F022, A02-F009, A05-F005
- Areas: ExecShell, bwrap, SafeOnly, timeout, reaper
- Severity: Major
- Status: Confirmed; outer-timeout descendant leak is Probable
- Summary: config 재로드 실패는 sandbox off/network on host shell로 전환되고, extra_binds는 임의 host path를 writable bind하며, SafeOnly는 cat/grep argument의 workspace 범위를 검사하지 않는다. bwrap 설치 여부와 실행 capability도 혼동한다.
- Verified Evidence: src/tools/shell.rs:125-390, src/infra/sandbox.rs:1-78, src/infra/process_reaper.rs:8-54, src/domain/permissions.rs:173-211을 재확인했다.
- Impact: host file/network 접근, background descendant 잔류, unrelated process 오살, sandbox false assurance.
- Required Action: immutable settings snapshot을 execution context로 전달하고 failure 시 Deny, validated read-only binds, argv/path-aware command policy, one deadline/group-kill, capability probe와 lease-based reaper를 적용한다.
- Re-audit Method: malformed/missing config, userns restricted host, background descendants, spoofed SMLCLI_PID, absolute sensitive reads를 격리 fixture로 검증한다.
- Synthesis Rationale: 현재 /etc write 테스트가 sandbox disabled host permission으로도 PASS할 수 있어 sandbox enforcement 증거가 아니다.

### [FIN-F009] approval·write queue의 오류 terminal transition이 정체됨

- Sources: A03-F023, A02-SUP-F001, A02-SUP-F002, A04-F001
- Areas: approval, write serialization, policy revalidation, input safety
- Severity: Major
- Status: Confirmed
- Summary: ToolError는 write running flag와 queue를 해제하지 않고, approval 재검증 Deny는 다음 approval 승격 전에 return한다. 승인 시 최신 PermissionEngine도 다시 실행하지 않는다.
- Verified Evidence: src/app/tool_runtime.rs:489-639, src/app/mod.rs:1037-1356 및 input routing :1915-2205를 메인이 재확인했다.
- Expected Basis: success/error/cancel/deny 모두 한 번의 terminal transition으로 flag, queue, pending count, UI block을 정리해야 한다.
- Actual: 한 번의 Err/Deny 뒤 후속 writes/approvals와 AI turn이 영구 대기할 수 있고, pending 중 강화된 Deny/SafeOnly 정책이 적용되지 않는다. UI는 y/n과 Enter/Esc 계약도 불일치한다.
- Impact: 작업 정체, stale policy 우회, 승인·거부 오인과 앱 종료.
- Required Action: call-id 기반 공통 terminal helper와 approval fingerprint를 도입하고 최신 preflight+PermissionEngine을 승인 직전에 재실행한다. Enter/Esc/y/n 계약을 하나로 고정한다.
- Re-audit Method: write Err, preflight Deny, policy change, queue 2개, Enter/Esc/y/n 각각의 flag/count/block/next dispatch를 검증한다.
- Synthesis Rationale: TTL queue 테스트는 이 두 조기 오류 경로를 다루지 않는다.

### [FIN-F010] WriteFile/ReplaceFileContent의 clobber·cardinality 계약이 안전하게 닫히지 않음

- Sources: A02-SUP-F003, A02-SUP-F004, A02-SUP-F005
- Areas: file mutation semantics, input validation, preview parity
- Severity: Major
- Status: Confirmed actual behavior; Needs Spec Clarification for intended overwrite/multi-match contract
- Summary: overwrite=false/누락을 execute가 무시하고, empty target은 모든 문자열 경계에 replacement를 삽입하며, multiple target은 묵시적으로 전부 치환한다.
- Verified Evidence: src/tools/file_ops.rs:323-537을 메인이 재확인했다.
- Impact: 신규 파일로 표시된 요청이 기존 파일을 덮어쓰고, malformed/ambiguous edit가 파일 전체를 변형할 수 있다.
- Required Action: overwrite semantics를 required schema로 고정하고 no-clobber를 enforce한다. target minLength=1, exact-one 또는 explicit replace-all+match count를 문서·preview·execute에 동일하게 적용한다. 기존 mode 보존도 함께 검증한다.
- Re-audit Method: existing/nonexisting, overwrite true/false/omitted, target 0/1/2/empty, executable-mode fixture에서 byte/mode/diff 일치를 확인한다.
- Synthesis Rationale: 실제 ignore/all-replace 동작은 확정됐고, 제품 owner가 false와 multiple의 최종 의미를 결정해야 한다.

### [FIN-F011] secret/config/wizard persistence가 권한·원자성·rollback을 보장하지 못함

- Sources: A02-F005, A02-F013, A03-F008, A03-F009, A03-F010, A02-SUP-F006, A02-SUP-F007
- Areas: master key, config, async saves, wizard merge
- Severity: Major
- Status: Confirmed; save-order race is Probable
- Summary: existing key/config mode·owner·symlink를 검증하지 않고 malformed nonce가 panic할 수 있다. config load가 active temp를 지울 수 있고 detached saves의 순서가 없다. /setting은 기존 trust/custom/sandbox/git/MCP/keys를 defaults로 교체하고 실패 후 memory를 rollback하지 않는다.
- Verified Evidence: src/infra/secret_store.rs:28-148, src/infra/config_store.rs:24-171, src/app/wizard_controller.rs:182-256, src/app/mod.rs:1366-1391을 재확인했다.
- Impact: API key 노출·DoS, 설정 손실, runtime/disk divergence, security policy 초기화.
- Required Action: owner-only no-follow storage, checked nonce/cipher cap, lock 아래 unique temp+journal+dir fsync, revisioned single writer를 적용한다. Wizard는 existing clone에서 owned fields만 patch하고 success event에서만 commit한다.
- Re-audit Method: loose mode/symlink/malformed nonce/concurrent save/read-only disk/wizard re-entry fixtures에서 old snapshot과 all sentinel fields를 검증한다.
- Synthesis Rationale: file locking은 physical write serialization만 제공하며 logical latest-wins 또는 wizard merge를 보장하지 않는다.

### [FIN-F012] session 로그·index가 개인정보와 손상 복구 경계를 닫지 못함

- Sources: A03-F011, A03-F012, A05-F003, A05-F008
- Areas: session file mode, index atomicity, restore bounds, path containment
- Severity: Major
- Status: Confirmed
- Summary: session directory/log/index mode가 umask에 의존하고, index는 plain read-modify-write와 parse-failure-to-empty를 사용한다. log_filename containment와 line/total size도 제한하지 않는다.
- Verified Evidence: src/infra/session_log.rs:41-183, :218-311, :388-484와 src/app/command_router.rs:993-1010을 재확인했다.
- Impact: prompts/tool output/workspace metadata의 다른 local 사용자 노출, concurrent session loss, corrupted index overwrite, arbitrary log path open, memory DoS.
- Required Action: injectable SessionStoreRoot, directory 0700/files 0600, lock+journal+atomic replace+backup, basename/UUID containment, bounded streaming restore를 구현한다.
- Re-audit Method: umask 000, read-only temp home, concurrent writers, corrupt index, absolute/parent/symlink filename, huge line fixtures를 검증한다.
- Synthesis Rationale: 현재 session test가 실제 home에 접근해 121-test gate 자체를 실패시키고 사용자 cleanup side effect도 만들 수 있다.

### [FIN-F013] tool turn·context compaction transaction이 실패 시 원자적이지 않음

- Sources: A02-F007, A02-F010
- Areas: outstanding tool calls, provider resend, compaction rollback
- Severity: Major
- Status: Confirmed
- Summary: mixed malformed+valid tool calls에서 malformed event가 outstanding count에 포함되지 않아 정상 tool 실행 중 follow-up 요청이 발생할 수 있다. /compact는 provider 성공 전에 원본 messages를 제거하고 실패 시 복구하지 않는다.
- Verified Evidence: src/app/tool_runtime.rs:36-89, src/app/mod.rs:1286-1356, src/domain/session.rs:172-213, src/app/command_router.rs:1099-1114를 재확인했다.
- Impact: 중복 provider 요청·out-of-order tool history와 장기 대화의 비가역 손실.
- Required Action: turn-scoped outstanding ID set과 exactly-once terminal aggregation을 사용하고, compaction은 pending snapshot에서 성공 후 atomic commit/실패 rollback한다.
- Re-audit Method: malformed+valid mixed turn과 timeout/429/invalid summary/cancel fixtures에서 follow-up 1회 및 message byte identity를 검증한다.
- Synthesis Rationale: 정상 tool-only 및 successful compaction tests는 failure transaction을 잠그지 않는다.

### [FIN-F014] provider/shell output streaming·redaction·truncation 계약이 실제 경로와 다름

- Sources: A02-F002, A03-F020, A03-F021, A05-F005, A05-F006, A05-F007
- Areas: SSE, output cap, secret masking, typed metadata
- Severity: Major
- Status: Confirmed; chunk-boundary masker defect is Probable
- Summary: provider chat_stream은 response.text로 EOF까지 버퍼링하고 shell Tool trait은 tx=None 경로를 사용한다. Error body와 Gemini query-key는 중앙 redaction을 우회하며 Grep/Fetch truncation flag도 거짓이다.
- Verified Evidence: src/providers/registry.rs:247-433, src/providers/anthropic.rs:237-449, src/app/chat_runtime.rs:577-638, src/app/mod.rs:916-952 및 :1692-1749, src/tools/shell.rs:38-46을 재확인했다.
- Impact: 실시간 UI 부재, large response OOM, key/error 노출, duplicate/partial live logs, LLM의 incomplete-result 오인.
- Required Action: 공통 incremental framed stream, byte/line/delta cap, cancellation, centralized sink redaction, emit-vs-tail masker API, accurate ToolResult metadata를 구현한다.
- Re-audit Method: slow SSE, large/error body, split-secret 모든 byte position, Unicode, cap±1, shell stdout/stderr fixtures에서 early delta·no leak·no duplicate를 확인한다.
- Synthesis Rationale: MockProvider의 즉시 delta와 wizard 별표 테스트는 production transport/redaction을 검증하지 않는다.

### [FIN-F015] RepoMap·EventLoop·diff cache가 UI responsiveness와 ordering을 보장하지 못함

- Sources: A02-F006, A05-F010, A04-F013
- Areas: RepoMap scheduling, event producers, render cache
- Severity: Major
- Status: Confirmed
- Summary: 첫 request에서 depth-10 AST scan을 event loop에서 동기 실행하고 background refresh와 중복될 수 있다. 실제 multi-producer EventLoop/Quit race는 테스트되지 않으며 diff cache는 theme/width identity를 key로 갖지 않는다.
- Verified Evidence: src/app/chat_runtime.rs:548-558, src/domain/repo_map.rs:102-238, src/app/event_loop.rs:24-90, src/tui/widgets/inspector_tabs.rs:31-46 및 :148-186을 재확인했다.
- Impact: 큰 repo에서 TUI freeze, stale result overwrite, 종료 task 잔류, theme 변경 후 stale style.
- Required Action: revisioned background RepoMap worker, deterministic event source/shutdown, style-independent cache 또는 theme/width key를 도입한다.
- Re-audit Method: large fixture latency/profile, overlapping refresh revisions, Tick/Input/Action/Quit ordering, theme/width cache invalidation을 검증한다.
- Synthesis Rationale: state helper와 text-change cache tests는 실제 scan/event/cache invalidation을 실행하지 않는다.

### [FIN-F016] Workspace Harness·doctor·first-run trust/preset enforcement가 완결되지 않음

- Sources: A01-F007, A02-F008, A03-F029, A04-F009
- Areas: harness drift, session record, first-run trust, network policy, onboarding
- Severity: Major
- Status: Confirmed
- Summary: Phase 54는 baseline-vs-current drift를 비교하지 않고 fallback session은 harness record를 생략한다. Wizard 완료 후 trust gate를 다시 열지 않으며 permission preset 단계도 없다. Doctor는 NetworkPolicy::Deny에서도 OpenRouter probe를 시도한다.
- Verified Evidence: src/infra/workspace_harness.rs:139-243, src/app/state.rs:139-185, src/app/mod.rs:102-122 및 :1366-1377, src/infra/doctor.rs:28-133을 재확인했다.
- Impact: tool이 조용히 Deny되어 onboarding이 막히고, 정책과 다른 network egress 및 재현 불가능한 session audit trail이 발생한다.
- Required Action: generation/baseline snapshot과 drift rules, mandatory harness record failure policy, post-wizard trust/preset/verify 단계, policy-aware offline doctor를 구현한다.
- Re-audit Method: first-run wizard, Trust Once/Remember/Restricted, five drift fields, logger fallback, doctor Deny/ProviderOnly/AllowAll request recorder를 검증한다.
- Synthesis Rationale: 현재 harness field presence tests는 baseline drift·wizard transition·doctor egress를 검증하지 않는다.

### [FIN-F017] 핵심 TUI command·focus·Inspector interaction이 보이는 계약과 다름

- Sources: A04-F002, A04-F003, A04-F004, A04-F015, A04-F016
- Areas: keyboard focus, commands, timeline/session sync, Inspector
- Severity: Major
- Status: Confirmed
- Summary: keyboard로 Timeline focus에 도달할 수 없고 Inspector CTA/Alt shortcuts/mouse actions가 닫히지 않았다. /status는 보이지 않고 /clear는 timeline을 지우지 않으며 command catalogs가 router와 다르다.
- Verified Evidence: src/app/mod.rs:2132-2475 및 :2649-2820, src/app/command_router.rs:295-405, src/tui/layout.rs:409-655, src/app/state.rs:380-489 및 :941-957을 재확인했다.
- Impact: terminal-first primary tasks와 recovery commands가 접근 불가·무반응·불일치 상태다.
- Required Action: shared command registry와 single visible timeline mutation path, explicit pane focus navigation, keyboard/mouse CTA actions를 구현하거나 계획 기능으로 문서에서 내린다.
- Re-audit Method: Composer→Timeline→Inspector key-only flow와 모든 router command의 Help/Palette/Slash parity, /status//clear visibility를 TestBackend로 확인한다.
- Synthesis Rationale: tab-cycle와 mouse tests는 focus acquisition, CTA execution, catalog parity를 검증하지 않는다.

### [FIN-F018] responsive·Unicode·overlay·terminal restore 경계가 panic/오조작을 유발함

- Sources: A04-F005, A04-F006, A04-F007, A04-F010, A04-F011, A04-F012, A04-F013, A04-F014
- Areas: layout, hit-test, Unicode width, overlay priority, terminal RAII
- Severity: Major
- Status: Confirmed
- Summary: compact drawer rect와 hit-test가 다르고, byte slicing truncate_middle은 CJK/emoji에서 panic 가능하다. Overlay render/input priority가 다르며 partial terminal init/restore가 raw/alternate/cursor state를 완전 복구하지 않는다.
- Verified Evidence: src/tui/layout.rs:31-38 및 :52-170, src/app/mod.rs:802-815 및 :1915-2205, src/tui/terminal.rs:16-80, src/tui/widgets/questionnaire.rs:41-174를 재확인했다.
- Impact: 좁은 화면 오포커스, 국제 문자 panic/정렬 오류, 보이는 modal과 다른 input 소비, 종료 후 깨진 terminal.
- Required Action: shared LayoutGeometry/hit-test, UnicodeWidth/char-boundary utilities, one overlay state machine, stepwise RAII rollback과 cursor Show를 적용한다.
- Re-audit Method: 30/80/90/99/100/103/104/120/140 폭, CJK/emoji/ANSI, overlay races, init failure injection을 TestBackend와 물리 TTY에서 검증한다.
- Synthesis Rationale: centered-rect와 한 cache test는 실제 breakpoint/Unicode/terminal cleanup을 다루지 않는다.

### [FIN-F019] 5-locale i18n 완료 주장이 실제 UI 소비·선택 경로보다 강함

- Sources: A01-F003, A04-F008
- Areas: i18n, locale selection, user-facing strings
- Severity: Major
- Status: Confirmed
- Summary: tr 사용은 status/tab/questionnaire 일부에 한정되고 wizard/config/help/palette/error는 English/Korean literals다. 일반 LANG 형태도 normalization하지 못하며 설정 default en이 environment를 덮는다.
- Verified Evidence: src/tui/i18n.rs:19-215, src/app/state.rs:699-708, src/tui/widgets 및 command_router literal/call-site 검색을 재확인했다.
- Impact: 지원 언어에 따라 혼합 UI와 잘못된 onboarding/help를 제공한다.
- Required Action: user-visible catalog와 locale precedence/change/persistence를 단일화하고 5-locale rendered coverage를 구축한다.
- Re-audit Method: 각 locale에서 full draw/doctor/help/wizard/error를 렌더링해 fallback·foreign literal·placeholder를 검사한다.
- Synthesis Rationale: key 대칭 테스트는 call-site coverage나 번역 품질을 증명하지 않는다.

### [FIN-F020] 공개 CLI/provider/module 계약이 구현·책임표와 어긋남

- Sources: A02-F001, A01-F004, A01-F005, A01-F006, A01-F011, A01-F013
- Areas: CLI entry, provider scope, file responsibility, stale phases
- Severity: Major
- Status: Confirmed drift; Ollama scope is Needs Spec Clarification
- Summary: 문서의 smlcli run prompt는 CLI에 없고, Ollama built-in vs Custom 계약이 충돌한다. commands/types 모듈은 비어 있는데 문서가 owner로 지정하며 config path/case와 Phase shortcuts도 stale하다.
- Verified Evidence: src/main.rs:22-84, src/domain/provider.rs:3-42, src/commands/mod.rs, src/types/mod.rs, command_router/README/spec mapping을 재확인했다.
- Impact: 자동화 진입 실패, provider 발견성/구현 오판, 잘못된 파일 수정과 release 기능 과대주장.
- Required Action: shipped/non-goal/deferred matrix를 새 current authority section으로 닫고 runtime owner/call graph에 맞춰 문서를 복구한다. run prompt와 Ollama는 구현 또는 명시 제거 중 하나를 선택한다.
- Re-audit Method: CLI help/completions/headless mock, provider enum/menu/registry/docs, module call-site 및 case-sensitive link check를 실행한다.
- Synthesis Rationale: historical 내용 보존은 가능하지만 current completion claim과 분리해야 한다.

### [FIN-F021] 프로젝트 identity·version·ADR·보안 문서 authority가 충돌함

- Sources: A01-F001, A01-F002, A01-F012, A03-F024, A06-F005
- Areas: project identity, release version, ADR lineage, security claims
- Severity: Major
- Status: Needs Spec Clarification plus Confirmed drift
- Summary: AGENTS는 doomlike 0.5.1, Cargo/README는 smlcli 3.9.0, 완료 Phase는 3.9.1/3.9.2다. ADR ID는 중복/미정의이고 security docs는 opt-in/heuristic을 strict/zero/guarantee로 표현한다.
- Verified Evidence: AGENTS.md:3-30, Cargo.toml:1-4, spec.md:3001-3018, DESIGN_DECISIONS heading/reference set, README security claims를 재확인했다.
- Impact: 감사·릴리스 대상과 지원 보안 경계를 잘못 판정한다.
- Required Action: human owner가 canonical identity/current release/Unreleased Phase를 결정하고 immutable ADR alias/supersedes 표 및 hard-boundary/heuristic matrix를 만든다.
- Re-audit Method: authority table, version/tag/artifact automated sync, ADR reference set, 모든 security claim의 code/test/residual-risk 링크를 확인한다.
- Synthesis Rationale: Cargo/CHANGELOG version-sync PASS는 더 넓은 authority 충돌을 검사하지 않는다.

### [FIN-F022] 테스트 suite가 현재 재현되지 않고 핵심 tests가 production path를 복제함

- Sources: A01-F009, A02-F012, A05-F001, A05-F002, A05-F003, A05-F009, A05-F010
- Areas: test isolation, false positives, coverage wording
- Severity: Major
- Status: Confirmed
- Summary: 121 tests 중 bwrap capability와 real-home session write 두 건이 실패한다. 다수의 E2E/보안 이름 test는 handler/store/renderer를 호출하지 않고 enum/string branch를 복제한다.
- Verified Evidence: current test binary 119/121 재현, src/tests/audit_regression.rs representative tests, cargo metadata의 one bin/doctest false/features empty를 재확인했다.
- Impact: PASS가 actual provider/file/config/UI behavior를 보장하지 않고 user home을 오염·정리할 수 있다.
- Required Action: injectable storage/provider/event seams, capability-gated integration jobs, production-call mutation-sensitive tests와 scope-labeled report를 구축한다.
- Re-audit Method: isolated writable home, userns enabled/restricted jobs, parallel/repeat tests, deliberate implementation mutation, current commit SHA와 command/result를 기록한다.
- Synthesis Rationale: 121은 등록 수이지 현재 성공 수나 whole-project/platform coverage가 아니다.

### [FIN-F023] CI/release가 dependency·version·locked security gate를 집행하지 않음

- Sources: A01-F008, A03-F025, A06-F001, A06-F002
- Areas: CI, dependency advisories, locked release
- Severity: Major
- Status: Confirmed
- Summary: BUILD_GUIDE/spec이 audit/deny/version gate를 요구하지만 CI/release는 unlocked fmt/clippy/test/build만 실행한다. local cargo audit은 2 vulnerabilities와 3 warnings로 실패했다.
- Verified Evidence: .github/workflows/ci.yml:41-60, release.yml:18-73, scripts/check-version-sync.sh 및 main cargo audit --no-fetch 결과를 재확인했다.
- Impact: vulnerable 또는 tag/version-drift artifact가 자동 게시될 수 있다.
- Required Action: all cargo commands --locked, tag version-sync, configured advisory/license/source policy, targeted security tests를 merge/tag gate로 연결한다.
- Re-audit Method: current advisory DB provenance와 feature/target reachability ledger를 만들고 mismatch tag/advisory/test failure가 live workflow를 차단하는지 확인한다.
- Synthesis Rationale: quinn-proto는 lockfile advisory가 확인됐지만 current default host feature reachability는 미확정이다. 이 nuance는 scanner gate 실패를 기각하지 않는다.

### [FIN-F024] cross-platform release target과 artifact provenance가 닫히지 않음

- Sources: A06-F003, A06-F004
- Areas: musl/MSVC/GNU, toolchain, SBOM, signing, permissions
- Severity: Major
- Status: Confirmed control gap; target runtime Unverified
- Summary: release는 musl+MSVC, build.sh는 GNU+MinGW를 사용하고 실제 확인 artifact는 glibc host binary뿐이다. Action/toolchain은 mutable tags이고 checksum/SBOM/signature/attestation/license metadata/rollback이 없다.
- Verified Evidence: build.sh, .cargo/config.toml, release matrix, installed target list, host artifact metadata를 재확인했다.
- Impact: Linux/Windows 지원과 배포 artifact origin/integrity를 재현·검증할 수 없다.
- Required Action: canonical target/toolchain을 pin하고 target-specific prerequisites/tests, job least privilege, checksum/SBOM/license/signature/attestation/rollback을 추가한다.
- Re-audit Method: clean musl/MSVC builds와 smoke, artifact hashes/signatures/provenance, rollback rehearsal를 수행한다.
- Synthesis Rationale: 미검증 target을 build failure로 단정하지 않지만 release readiness는 HOLD다.

### [FIN-F025] local/generated/reference residue와 scratch rewrite 도구의 shipped scope가 불명확함

- Sources: A01-F010, A01-F014, A06-F006, A06-F007
- Areas: repository hygiene, package scope, local metadata
- Severity: Minor
- Status: Confirmed; AGENTS whitespace intent is Needs Clarification
- Summary: pycache, empty .codex, .gemini override, 외부 home symlink, scratch in-place rewrite, reference assets가 추적되고 package include/exclude가 없다.
- Verified Evidence: git ls-files, tracked symlink mode/target, .gitignore:1, scratch.py/scratch2.py를 재확인했다.
- Impact: source archive에 환경 흔적·불필요 산출물이 포함되고 실수로 test source를 광범위 덮어쓸 수 있다.
- Suggested Action: runtime/support/generated/reference 분류와 Cargo package scope를 명시하고 pycache/local symlink/scratch를 제거·격리 또는 owner 문서화한다.
- Re-audit Method: git/package file list, symlink portability, secret scan, scratch dry-run/fixture를 확인한다.
- Synthesis Rationale: reference assets 자체는 결함이 아니며 ownership/shipped scope 부재가 finding이다.

## 10. Critical/Major Direct Re-verification

메인 모델은 모든 Critical/Major canonical finding의 원본 파일·호출 경로 또는 실행 결과를 직접 다시 확인했다.

| Canonical Finding | Directly Checked By Main | Evidence reopened or command rerun | Result | Gate impact |
| --- | --- | --- | --- | --- |
| FIN-F001 | Yes | git_checkpoint.rs, executor.rs, git_engine.rs | Confirmed | Critical, PASS 차단 |
| FIN-F002 | Yes | mcp_client.rs spawn vs shell.rs env_clear | Confirmed | Critical, PASS 차단 |
| FIN-F003 | Yes | chat_runtime.rs @ preprocessing → session → provider guard | Confirmed; Deny HTTP bypass는 없음 | Critical, PASS 차단 |
| FIN-F004 | Yes | file_ops.rs deterministic temp write/rename order | Confirmed | Critical, PASS 차단 |
| FIN-F005 | Yes | chat_runtime custom resolve, registry reload/fallback, wizard save | Confirmed | Critical, PASS 차단 |
| FIN-F006 | Yes | MCP reader/request/shutdown, tool_runtime routed call | Confirmed | Major |
| FIN-F007 | Yes | fetch.rs URL/cap/metadata/UTF-8 truncate | Confirmed | Major |
| FIN-F008 | Yes | shell.rs, sandbox.rs, process_reaper.rs, permissions.rs | Confirmed / descendant race Probable | Major |
| FIN-F009 | Yes | tool_runtime approval paths, mod.rs ToolFinished/ToolError | Confirmed | Major |
| FIN-F010 | Yes | file_ops schemas/preview/execute | Confirmed actual; expected contract unresolved | Major |
| FIN-F011 | Yes | secret_store, config_store, wizard save events | Confirmed / save ordering Probable | Major |
| FIN-F012 | Yes | session_log and resume path | Confirmed | Major |
| FIN-F013 | Yes | mixed tool count and compaction mutation paths | Confirmed | Major |
| FIN-F014 | Yes | provider response.text paths, chat error sinks, shell tx=None | Confirmed / masker split Probable | Major |
| FIN-F015 | Yes | RepoMap sync call, EventLoop producers, diff cache | Confirmed | Major |
| FIN-F016 | Yes | harness baseline absence, first-run gate, doctor request | Confirmed | Major |
| FIN-F017 | Yes | input routing, command catalogs, timeline renderer | Confirmed | Major |
| FIN-F018 | Yes | layout geometry, byte slicing, overlays, TerminalGuard | Confirmed | Major |
| FIN-F019 | Yes | i18n catalog/call sites and locale selection | Confirmed | Major |
| FIN-F020 | Yes | CLI help/runtime, provider enum/UI, empty modules, docs | Confirmed / Ollama unresolved | Major |
| FIN-F021 | Yes | AGENTS/Cargo/spec/ADR/README authority | Confirmed drift / human decision needed | Major |
| FIN-F022 | Yes | 121-test run, representative copied tests, metadata scope | 119 passed, 2 failed | Major |
| FIN-F023 | Yes | cargo audit --no-fetch, CI/release commands | Confirmed | Major |
| FIN-F024 | Yes | target/workflow/toolchain/provenance files | Control gap confirmed; target runtime unverified | Major |

직접 실행 결과:

- main의 current prebuilt test binary: 119 passed, 2 failed
- A02/A05 isolated full suite: 같은 119/121
- main의 fresh isolated build attempt: /tmp quota로 compile 중단, PASS 증거로 사용하지 않음
- cargo audit --no-fetch: exit 1, 2 vulnerabilities, 3 warnings
- cargo fmt --check: exit 0
- version-sync: exit 0이나 Cargo/CHANGELOG 3.9.0만 증명
- manifest verify: 9 reports verified, missing 0

## 11. Cross-Report Conflicts

### XPF-001 — @file의 provider egress 범위

- A03 supplement: normal provider-allowed flow에서 outside file이 provider에 전달되므로 Critical.
- A05 supplement: NetworkPolicy::Deny 자체의 HTTP 우회는 반증됨.
- Resolution: 두 판단은 양립한다. FIN-F003은 Deny bypass를 주장하지 않는다. file read·local persistence는 guard 전에 발생하고, outbound가 허용된 정상 설정에서는 remote egress까지 이어진다.

### XPF-002 — temp symlink severity

- A03/A05 supplements: Major.
- Main: malicious workspace의 pre-existing sibling symlink가 arbitrary external write를 만들며 표준의 Critical 후보인 arbitrary file access/data damage에 해당.
- Resolution: FIN-F004를 Critical로 상향하되 공격 전제와 비실행 fixture를 명시한다.

### XPF-003 — MCP environment inheritance severity

- A03: Critical, A05: Major이며 actual secret이 parent env에 있을 때 Critical로 상승 가능.
- Resolution: 외부 executable이 credential을 읽는 direct secret-egress 경계이므로 FIN-F002를 Critical로 유지한다. MCP를 완전 신뢰 executable로 제한하려면 명시적 policy와 별도 hard boundary가 필요하다.

### XPF-004 — quinn-proto advisory reachability

- cargo audit은 high advisory를 lock graph에서 보고했다.
- current Linux default feature cargo tree에서는 quinn-proto가 활성 runtime graph에 나타나지 않았다.
- Resolution: advisory gate failure는 유지하되 shipped exploitability는 Unverified로 기록한다. crossbeam-epoch 경로는 current graph에 존재한다.

### XPF-005 — historical PASS와 current test failure

- audit_report_8/9는 당시 121/121 PASS를 주장한다.
- current isolated environment는 119/121이며 두 test가 environment-coupled다.
- Resolution: 과거 보고서를 당시 snapshot으로 보존하고 current release evidence로 사용하지 않는다.

### XPF-006 — WriteFile overwrite와 Replace multiple semantics

- actual ignore/all-replace behavior는 Confirmed.
- false/omitted 또는 multiple match의 expected product semantics는 문서에 닫히지 않았다.
- Resolution: FIN-F010은 실제 위험을 유지하고 contract decision을 선행 조건으로 둔다.

## 12. Finding Adjudication Ledger

99개 source finding을 아래 25개 canonical finding으로 병합했다. 모든 source ID는 보존했다.

| Source Findings | Decision | Canonical Finding | Rationale |
| --- | --- | --- | --- |
| A03-F013, A03-F014, A02-F011 | Merged / Accepted | FIN-F001 | 같은 Git user-data ownership root cause |
| A03-F015, A05-F013 | Merged / Accepted | FIN-F002 | MCP parent environment inheritance |
| A03-F026, A05-F011 | Merged / Partially narrowed | FIN-F003 | Deny HTTP bypass는 기각, file boundary와 allowed egress는 유지 |
| A03-F001, A03-F027, A05-F012 | Merged / Accepted | FIN-F004 | canonical target 외 deterministic temp path |
| A02-F004, A03-F019, A03-F028 | Merged / Accepted | FIN-F005 | custom endpoint/auth source-of-truth failure |
| A02-F003, A03-F016, A03-F017, A03-F018, A05-F004 | Merged / Accepted | FIN-F006 | MCP supervisor, bound, schema, approval |
| A03-F002, A03-F003, A05-F007 | Merged / Accepted | FIN-F007 | Fetch network and result-bound root cause |
| A03-F004, A03-F005, A03-F006, A03-F007, A03-F022, A02-F009 | Merged / Accepted | FIN-F008 | shell/sandbox/process enforcement |
| A03-F023, A02-SUP-F001, A02-SUP-F002, A04-F001 | Merged / Accepted | FIN-F009 | terminal transition and approval authority |
| A02-SUP-F003, A02-SUP-F004, A02-SUP-F005 | Merged / Accepted with clarification | FIN-F010 | file edit semantics/cardinality |
| A02-F005, A02-F013, A03-F008, A03-F009, A03-F010, A02-SUP-F006, A02-SUP-F007 | Merged / Accepted | FIN-F011 | config/secret/wizard transaction |
| A03-F011, A03-F012, A05-F003, A05-F008 | Merged / Accepted | FIN-F012 | session privacy/index/storage |
| A02-F007, A02-F010 | Merged / Accepted | FIN-F013 | turn/compaction atomicity |
| A02-F002, A03-F020, A03-F021, A05-F005, A05-F006 | Merged / Accepted | FIN-F014 | streaming, output cap, redaction |
| A02-F006, A05-F010, A04-F013 | Merged / Accepted | FIN-F015 | event-loop work and cache lifecycle |
| A01-F007, A02-F008, A03-F029, A04-F009 | Merged / Accepted | FIN-F016 | harness/onboarding/doctor enforcement |
| A04-F002, A04-F003, A04-F004, A04-F015, A04-F016 | Merged / Accepted | FIN-F017 | TUI interaction/source-of-truth |
| A04-F005, A04-F006, A04-F007, A04-F010, A04-F011, A04-F012, A04-F014 | Merged / Accepted | FIN-F018 | geometry/Unicode/overlay/terminal |
| A01-F003, A04-F008 | Merged / Accepted | FIN-F019 | i18n completion overclaim |
| A02-F001, A01-F004, A01-F005, A01-F006, A01-F011, A01-F013 | Merged / Accepted | FIN-F020 | public contract and file ownership drift |
| A01-F001, A01-F002, A01-F012, A03-F024, A06-F005 | Merged / Accepted | FIN-F021 | authority/version/ADR/security claims |
| A01-F009, A02-F012, A05-F001, A05-F002, A05-F003, A05-F009 | Merged / Accepted | FIN-F022 | non-hermetic/shallow test evidence |
| A01-F008, A03-F025, A06-F001, A06-F002 | Merged / Accepted | FIN-F023 | CI/advisory/locked gate |
| A06-F003, A06-F004 | Merged / Accepted | FIN-F024 | target/provenance/reproducibility |
| A01-F010, A01-F014, A06-F006, A06-F007 | Merged / Accepted with clarification | FIN-F025 | repository/shipped-scope hygiene |

No source finding was discarded solely because only one agent raised it. No repeated finding was accepted solely by majority.

## 13. Required Actions Before Passing

### P0 — 즉시 위험 격리

1. FIN-F001: automatic hard-reset rollback과 index-wide auto-commit을 disable/Ask-only로 격리한다.
2. FIN-F002: MCP child environment를 clear하고 explicit non-secret allowlist만 전달한다.
3. FIN-F003: @file outside/parent/symlink/oversized/binary 입력을 deny하고 guard 전 logging을 중단한다.
4. FIN-F004: deterministic .tmp write를 unique exclusive no-follow temp transaction으로 교체한다.
5. FIN-F005: custom provider fallback을 제거하고 unknown/unsupported/reload 상태에서 zero-request fail-closed 한다.
6. FIN-F007: FetchURL 기본 private/link-local/metadata/redirect 접근을 차단한다.

### P1 — security/data/lifecycle 복구

1. MCP supervisor, cancellation, child tree cleanup, schema validation, output cap과 approval provenance를 구현한다.
2. ExecShell settings snapshot을 단일화하고 config/backend failure를 Deny로 처리한다.
3. approval/write queue의 모든 terminal path를 exactly-once transition으로 통합한다.
4. key/config/session owner mode, no-follow, atomic journal, revisioned writer, bounded restore를 구현한다.
5. Context compaction과 tool turn aggregation을 transactional하게 만든다.
6. Workspace Harness baseline drift, post-wizard trust/preset, policy-aware doctor를 닫는다.

### P2 — runtime/UI/product completeness

1. true incremental streaming, centralized redaction, truncation metadata를 구현한다.
2. RepoMap/background event lifecycle과 large-repo responsiveness를 검증한다.
3. TUI focus/command/Inspector/timeline sync, responsive hit-test, Unicode, overlay, terminal RAII를 수정한다.
4. i18n scope를 실제 UI 전체로 확장하거나 공개 범위를 축소한다.
5. run prompt, Ollama/provider, module ownership, config path의 canonical product contract를 결정한다.

### P3 — authority/test/release closure

1. human owner가 project name/current version/Phase/ADR authority를 확정한다.
2. production call/side-effect 기반 hermetic tests로 121/121 또는 명시적 capability matrix를 확보한다.
3. current lockfile advisories를 update/patch/expiry-bound exception으로 처리한다.
4. CI/release에 locked version/audit/deny/security gates를 연결한다.
5. canonical musl/MSVC or GNU target, pinned toolchain/actions, checksum/SBOM/license/signature/attestation/rollback을 닫는다.
6. shipped/reference/generated/local repository scope를 정리한다.

## 14. Accepted and Remaining Risks

명시적으로 Accepted Risk로 승인된 항목은 없다.

남은 미검증은 다음과 같으며 모두 PASS 근거가 아니다.

- 실제 Windows/MSVC와 Linux musl build/runtime
- 물리 TTY의 terminal restore와 rendering
- live DNS rebinding, redirect, provider/MCP/Fetch network
- local advisory DB 2026-07-17 이후의 advisory
- actual GitHub Actions release upload/rollback
- real user home/config/session mode와 migration (개인정보 보호를 위해 미열람)
- cargo deny license/source/bans result

## 15. Clarifications and Inconclusive Areas

Human/architect 결정이 필요한 항목:

1. canonical project identity는 smlcli 3.9.0인가, AGENTS.md의 doomlike 0.5.1은 잘못 병합된 template인가.
2. Phase 53/54의 v3.9.1/v3.9.2는 Unreleased인가, package version bump 누락인가.
3. Ollama는 built-in provider인가 Custom Provider 예시인가.
4. WriteFile overwrite=false/omitted와 Replace multiple match의 정확한 contract는 무엇인가.
5. NetworkPolicy::AllowAll은 public internet만인가, private/loopback까지 의도하는가.
6. sandbox extra_binds는 trusted-admin escape hatch인가 일반 사용자 기능인가.
7. MCP server는 완전 신뢰 executable만 허용하는가.
8. session local history의 encryption/retention/redaction 기준은 무엇인가.
9. canonical Windows target은 GNU인가 MSVC인가, Linux baseline은 glibc인가 musl인가.
10. .agents/.gemini/.codex/reference assets/scratch/external symlink의 shipped scope는 무엇인가.
11. AGENTS.md trailing double-space는 intentional hard break인가 git diff --check gate 위반인가.

## 16. Re-audit Checklist

- [ ] P0 Critical 5건의 isolated negative fixture가 모두 통과한다.
- [ ] Git concurrent WIP/pre-staged/HEAD-change fixture에서 byte/hash 보존을 확인한다.
- [ ] @file 및 WriteFile symlink fixture에서 workspace 밖 read/write가 0건이다.
- [ ] custom provider recorder에서 OpenAI/OpenRouter/Google 오배선과 secret 전달이 0건이다.
- [ ] MCP env sentinel, cancellation, huge output, malformed schema, child descendant fixture가 통과한다.
- [ ] Fetch private IP/IPv6/redirect/DNS/large/CJK fixtures가 통과한다.
- [ ] config/session isolated store의 concurrent/crash/corruption/mode tests가 통과한다.
- [ ] compaction/tool queue/approval exactly-once state invariants를 검증한다.
- [ ] five-locale TestBackend UI, responsive/Unicode/overlay/terminal failure-injection tests가 통과한다.
- [ ] cargo fmt --check, cargo check --all-targets --locked, cargo clippy --all-targets --all-features --locked -- -D warnings가 통과한다.
- [ ] isolated writable test root에서 cargo test --all-targets --locked --no-fail-fast가 capability matrix에 맞게 통과한다.
- [ ] cargo audit와 configured cargo deny가 exit 0 또는 owner/expiry가 있는 exception ledger만 남긴다.
- [ ] canonical Windows/Linux target을 실제 build/smoke하고 artifact provenance를 검증한다.
- [ ] name/version/Phase/tag/ADR/README/spec/CHANGELOG authority가 하나로 동기화된다.
- [ ] 새 multi-audit turn에서 관련 Pass 1/2/3과 supplements를 독립 재감사한다.

## 17. Final Decision

**HOLD**

판정 근거:

- 5개 Critical이 직접 재확인됐다.
- unresolved Major가 문서, runtime, security, UI, tests, release 전 영역에 존재한다.
- current full test gate는 119/121이다.
- cargo audit gate는 exit 1이다.
- physical/cross-target/live-network 공백도 남아 있다.
- Accepted Risk 또는 owner/expiry가 있는 예외가 없다.

따라서 현재 트리를 안정화 완료, release-ready, security-complete, 또는 PASS 계열로 표현할 수 없다. 구조적 수정이 필요하지만, 이번 감사의 실행 경계에 따라 소스·테스트·설정·제품 문서는 수정하지 않았다.

## 18. Coder Handoff

~~~text
/mnt/Projects_SSD/rust/smlcli/docs/multi_audit/1/final_audit_report_1.md를 먼저 읽고, 각 finding을 프로젝트 문서와 실제 코드에 대조하여 검증한 뒤 우선순위대로 수정하세요. 계약 변경이 필요하면 관련 문서를 먼저 갱신하고, 수정 후 테스트·빌드·재감사 증거를 기록하세요.
~~~
