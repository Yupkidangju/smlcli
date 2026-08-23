# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 1
- Perspective: A03 — 보안·신뢰 경계·오류 복원력
- User Goal: `$multi-audit` 프로젝트의 모든 문제점을 파악하여 안정화하고 완성도를 높이는 전체감사 개시. 프로젝트의 문서와 구현을 파악한 뒤 근본 문제를 해결할 수 있도록 상세 감사한다.
- Audit Basis: Standard-backed
- Standard Path: `/mnt/Projects_SSD/rust/smlcli/AI_AUDIT_DOC_STANDARD.md`
- Report Contract: `/home/eunho1/.codex/skills/multi-audit/references/report-contract.md`
- Audit rule: 소스, 테스트, 설정, 제품 문서와 CI를 읽기 감사했으며, 수정은 이 보고서만 수행했다.

## 2. Assigned Scope

다음 신뢰 경계를 공격 관점에서 추적했다.

- `PermissionEngine`, Workspace Trust/Harness/preflight, 경로 canonicalization·symlink·TOCTOU, 셸 `cwd`·환경변수·`bwrap`·프로세스 lifecycle
- `FetchURL`의 SSRF, redirect/DNS, 응답 크기와 NetworkPolicy
- provider base URL/auth 및 스트리밍·오류 redaction
- `secret_store`/`config_store`의 키 파일 권한, nonce 검증, 평문·백업·원자성
- JSONL session log의 개인정보 권한, 손상 복구, 크기 상한과 SessionIndex 무결성
- Git checkpoint/rollback/auto-commit의 사용자 데이터 보존
- MCP stdio JSON-RPC 입력·schema·도구명 충돌·프로세스 종료·cancellation·timeout·output cap
- process reaper의 타 프로세스 오살 가능성
- CI/release와 문서가 heuristic을 hard boundary처럼 주장하는지 여부

## 3. Excluded and Uninspected Scope

- `.git`, `target`, `/tmp` 빌드 산출물, 캐시, 생성된 UI/reference corpus는 읽지 않았다.
- 실제 사용자 `~/.smlcli/config.toml`, `.master_key`, `sessions/` 파일의 내용·권한은 개인정보와 사용자 데이터 보호를 위해 열지 않았다.
- 실제 외부 URL, provider API, DNS rebinding, redirect, 유료 API 호출은 수행하지 않았다.
- Windows/macOS 실환경, CI runner 실행, release 업로드·서명·artifact 공급망 검증은 실행하지 않았다.
- `build.sh`, `cargo fmt` 수정 모드, 위험 셸 명령은 실행하지 않았다.
- `cargo audit` advisory DB와 실제 dependency 취약점 판정은 네트워크/외부 advisory 범위 밖이므로 `Not Covered`다.
- 동료 `docs/multi_audit/1/sub_audit_*.md` 보고서는 읽지 않았다.
- 전체 테스트는 실행하지 않았다. 허용된 필터 테스트 중 추가 세션/MCP 필터는 `/tmp` quota 고갈로 compile 단계에서 중단되어 `Not Covered`다.

## 4. Evidence Examined

### Project documents and controls

- `AI_AUDIT_DOC_STANDARD.md` 전체, `report-contract.md` 전체
- `README.md`, `spec.md`, `designs.md`, `DESIGN_DECISIONS.md`, `IMPLEMENTATION_SUMMARY.md`, `BUILD_GUIDE.md`, `audit_roadmap.md`, `CHANGELOG.md`, `LESSONS_LEARNED.md`
- `Cargo.toml`, `Cargo.lock`, `.cargo/config.toml`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `scripts/check-version-sync.sh`

### Implementation surfaces

- `src/domain/permissions.rs`
- `src/infra/workspace_harness.rs`, `workspace_utils.rs`, `sandbox.rs`, `config_store.rs`, `secret_store.rs`, `session_log.rs`, `process_reaper.rs`, `mcp_client.rs`, `git_engine.rs`
- `src/tools/file_ops.rs`, `sys_ops.rs`, `shell.rs`, `fetch.rs`, `executor.rs`, `git_checkpoint.rs`, `registry.rs`
- `src/app/tool_runtime.rs`, `chat_runtime.rs`, `mod.rs`, `command_router.rs`, `wizard_controller.rs`, `state.rs`
- `src/providers/registry.rs`, `anthropic.rs`, `types.rs`
- `src/tests/audit_regression.rs`와 관련 테스트 모듈

### Commands and results

- `CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo test --locked test_fetch_url_network_policy` — **PASS**, 1 passed, 120 filtered.
- `... cargo test --locked test_harness_preflight_denies_cwd_outside_workspace` — **PASS**, 1 passed, 120 filtered.
- `... cargo test --locked test_exec_shell_cwd_absolute_path_outside_workspace` — **PASS**, 1 passed, 120 filtered.
- `... cargo test --locked test_git_auto_commit_selective_staging` — **PASS**, 1 passed, 120 filtered.
- `... cargo test --locked test_git_auto_commit_wip_protection` — **PASS**, 1 passed, 120 filtered.
- `... cargo test --locked test_mcp_permission_engine_always_ask` — **PASS**, 1 passed, 120 filtered.
- `... cargo test --locked test_mcp_call_tool_result_success_path` — **PASS**, 1 passed, 120 filtered.
- `... cargo test --locked test_mcp_call_tool_result_is_error_with_content` — **PASS**, 1 passed, 120 filtered.
- `... cargo test --locked test_session_logger_append_and_restore` 등 추가 필터 — **NOT COVERED**, `build.rs` compile 단계에서 `Disk quota exceeded (os error 122)`; 이후 bwrap synthetic mount도 `Quota exceeded`로 중단.
- 실행 후 `git status --short`에서 감사자가 변경한 소스·테스트·설정은 없었다. 감사자가 만든 `/tmp/smlcli-multi-audit-1-target`만 제거했다.

## 5. Findings

### [A03-F001] canonicalize 후 실제 파일 사용 사이의 TOCTOU가 symlink 경계를 보장하지 않음

- Pattern: `SEC-004` — 경로, workspace, 셸 실행 경계를 독립 제어군으로 감사
- Area: file path canonicalization, symlink, TOCTOU
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: `validate_sandbox()`는 경로를 검사하고 `PathBuf`를 반환하지만, 파일 descriptor를 고정하거나 `O_NOFOLLOW`/dirfd 계층을 사용하지 않는다. `ListDir`/`Stat`은 검사 결과를 버리고 원래 문자열을 다시 사용한다.
- Evidence:
  - `src/tools/file_ops.rs:10-53` (`validate_sandbox`)는 `exists`/`canonicalize` 후 별도 경로를 반환한다.
  - `src/tools/file_ops.rs:60-77`은 canonical path를 `File::open`하기 전에 다른 프로세스가 대상을 교체할 수 있다.
  - `src/tools/file_ops.rs:180-200`은 canonical path에서 `.tmp`를 만들지만 missing parent를 검사 시점에만 확인한다.
  - `src/tools/sys_ops.rs:180-198` (`ListDirTool::execute`)와 `:266-281` (`StatTool::execute`)은 `check_permission()`의 검증 후 raw `path`를 다시 `read_dir`/`metadata`에 전달한다.
  - `src/infra/workspace_harness.rs:329-345` (`path_outside_workspace`)도 `canonicalize().unwrap_or(candidate)`인 검사 전용 로직이다.
  - `spec.md:1938-1948`와 `IMPLEMENTATION_SUMMARY.md:118-121`은 canonicalize로 symlink/path traversal를 차단한다고 hard boundary처럼 적는다.
- Expected Basis: `SEC-004`, `spec.md:1940`의 “최종 경로가 workspace 밖이면 무조건 차단” 계약. 파일 검사와 사용 사이에도 동일한 경계가 유지되어야 한다.
- Actual: 존재하는 symlink를 검사 시점에 통과시키지는 않지만, 검사 후 symlink/parent를 바꿀 수 있다. 특히 ListDir/Stat은 검증 결과를 사용하지 않아 검증과 실제 대상이 즉시 분리된다.
- Attack Preconditions: 같은 사용자 또는 같은 workspace에 쓸 수 있는 동시 프로세스/작업이 검사와 사용 사이에 대상 파일 또는 missing parent symlink를 교체할 수 있어야 한다.
- Impact: `ReadFile`의 외부 파일 읽기, `ListDir`/`Stat`의 workspace 외부 정보 노출, missing parent를 통한 외부 temp 파일 쓰기 가능성. “strict symlink sandbox” 주장을 반증한다.
- Suggested Action: Linux에서는 `openat2(RESOLVE_BENEATH|RESOLVE_NO_SYMLINKS)` 또는 동등한 dirfd/open-handle 기반 API를 사용하고, 각 도구가 검증된 canonical 결과/descriptor만 사용하게 한다. 생성 경로는 모든 부모를 `mkdir/openat`로 다시 검증하고, `ListDir`/`Stat`은 raw path 재사용을 제거한다.
- Re-audit Method: race fixture를 소스 테스트에 추가한 뒤(현재 감사에서는 추가 금지), symlink 교체 중 Read/List/Stat/Write/Delete가 외부 경로를 열거나 쓰지 않는지 확인한다. `test_*_sandbox_*` 계열에 missing-parent와 concurrent swap 케이스를 포함한다.
- Confidence: High

### [A03-F024] 보안 문서가 heuristic/opt-in을 strict hard boundary와 zero-leak guarantee로 과대주장함

- Pattern: `SEC-005` — security documentation must match enforcement strength
- Area: README/spec/implementation summary claims
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix / Hold
- Summary: 문서는 strict symlink sandbox, real bwrap isolation, process group extermination, environment isolation, zero API-key leakage, safe self-healing을 완료 보장처럼 쓴다. 실제로는 sandbox가 opt-in/default-off이고, TOCTOU, config fail-open, reaper 오살, provider/MCP redaction/lifecycle, rollback data loss가 남아 있다.
- Evidence:
  - `README.md:17-18`, `:28`, English `:69-80`에 “strict”, “real”, “zero”, “guarantees”가 있다.
  - `src/domain/settings.rs:127-133` sandbox default는 disabled.
  - `src/tools/shell.rs:195-385` outer timeout descendant gap, `src/infra/process_reaper.rs:22-50` spoofable reaper, `src/app/mod.rs:916-952` unmasked provider errors가 구현 증거다.
  - `IMPLEMENTATION_SUMMARY.md:120`, `:151`, `:808-811`, `:877`은 canonicalize/실제 sandbox/kill/masking을 완료로 표시한다.
  - `spec.md:1940`, `:1994-2018`은 path, process, output 경계를 hard success criteria로 쓴다.
- Expected Basis: `SEC-005`와 표준의 hard boundary/heuristic 구분. 문서가 실제 enforcement와 동일한 강도로 말해야 한다.
- Actual: 권한 정책, preflight, canonicalize, bwrap 옵션은 유용한 defense-in-depth이지만 모든 경로에서 kernel/hard guarantee가 아니다. 현재 문구는 운영자에게 false assurance를 준다.
- Attack Preconditions: 사용자가 README/spec/summary를 보안 모델로 신뢰하고 sandbox/secret/session/rollback 위험을 별도 확인하지 않아야 한다.
- Impact: 잘못된 배포·운영 판단, 보안 사고 시 탐지·대응 지연, Phase PASS 오판.
- Suggested Action: 문서에서 `hard boundary`, `heuristic`, `opt-in`, `Not Covered`를 각 기능별로 분리하고 대응 code/test 위치와 residual risk를 연결한다. Critical/Major finding이 해결되기 전 “strict/guarantee/zero” 표현을 제거한다.
- Re-audit Method: README/spec/implementation summary의 보안 문장을 표로 추출해 각 주장마다 enforcement code, failure mode, test, release mode를 연결하고 unresolved claim은 Hold로 판정한다.
- Confidence: High

### [A03-F025] CI/release가 문서의 security/supply-chain gate를 실제로 시행하지 않음

- Pattern: `SEC-006` — scanner/supply-chain and shipped scope provenance
- Area: CI, release, dependency lock, artifact trust
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: BUILD_GUIDE는 `cargo audit`를 merge 전 필수로 선언하지만 CI/release workflow에는 audit, `--locked`, artifact smoke/checksum/signing, secret scanning 또는 security boundary tests가 없다. release는 빌드한 unsigned binary를 바로 GitHub Release에 업로드한다.
- Evidence:
  - `BUILD_GUIDE.md:31-38`은 fmt/clippy/test/cargo audit를 필수라고 한다.
  - `.github/workflows/ci.yml:41-48`은 fmt/clippy/`cargo test --all-targets`만 실행하고 `cargo audit`/`--locked`가 없다.
  - `.github/workflows/release.yml:18-29`의 quality gate도 audit/locked/security tests가 없다.
  - `.github/workflows/release.yml:72-88`은 `cargo build --release` 후 checksum/signature/provenance 없이 `softprops/action-gh-release`로 업로드한다.
  - CI 주석 `ci.yml:1-3`과 release 주석 `release.yml:1-3`은 이 경로를 품질/배포 gate로 표현한다.
- Expected Basis: `SEC-006`, `BUILD_GUIDE`와 release 문서의 gate 정합성. shipped artifact의 provenance와 dependency lock이 검증되어야 한다.
- Actual: dependency advisory drift, lockfile update, release binary tampering/mismatch, security regression이 PR/tag gate에서 탐지되지 않는다. 현재 저장소에서 `cargo audit` 자체는 외부 advisory 범위로 실행하지 않았으므로 dependency status는 별도 Not Covered다.
- Attack Preconditions: 취약 dependency/lock drift/security regression 또는 release artifact supply-chain 이벤트가 발생해야 한다.
- Impact: 취약 artifact가 merge/release되고, 문서상 security gate와 실제 pipeline 사이에 false assurance가 생긴다.
- Suggested Action: CI/release에 `cargo test --locked`, `cargo audit`/advisory provenance, targeted security regression, secret scan, artifact hash/signature/SBOM, release smoke를 추가한다. release permission은 job scope로 최소화한다.
- Re-audit Method: clean runner에서 PR/tag workflow를 실행해 lock drift/audit failure/security regression이 차단되는지, artifact SHA/signature가 published asset과 일치하는지 확인한다.
- Confidence: High

### [A03-F019] custom provider base URL/auth에는 redirect·host·IP 경계가 없어 key 전달 범위를 보장하지 못함

- Pattern: `SEC-003`, `SEC-001` — provider network/auth boundary
- Area: custom base URL, auth header, redirect policy
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix / Needs Spec Clarification
- Summary: custom provider의 `base_url`, `auth_type`, `auth_header_name`이 문자열로 저장되고 URL/host/IP/redirect policy가 없다. Bearer 또는 custom header API key를 해당 URL에 보낸다.
- Evidence:
  - `src/domain/provider.rs:18-25` `base_url`, `auth_type`, `auth_header_name`에는 제한 계약이 없다.
  - `src/providers/registry.rs:61-96` (`OpenAICompatAdapter`, `apply_auth`)는 base URL과 custom header를 그대로 사용한다.
  - `src/providers/registry.rs:186-195`, `:308-317`은 `format!("{}/chat/completions", self.base_url)` 후 reqwest default client로 전송한다.
  - `src/providers/registry.rs:803-830`은 config의 auth type을 adapter에 주입하지만 URL/redirect allowlist는 하지 않는다.
  - `src/app/chat_runtime.rs:196-226`은 custom provider가 `NetworkPolicy::ProviderOnly`에서도 선택될 수 있게 한다. ProviderOnly는 FetchURL만 Deny한다.
- Expected Basis: provider base URL은 사용자 기능이지만 API key가 전달되는 network destination은 명시적인 trusted endpoint/redirect 정책이어야 한다. `SEC-003`과 `SEC-001`의 secret egress boundary가 기준이다.
- Actual: ProviderOnly가 arbitrary custom/private endpoint를 허용하고, redirect/DNS 목적지 변화에 대한 앱 수준 검증이 없다. CustomHeader는 cross-origin redirect 시에도 어떤 헤더가 전달되는지 contract가 고정되어 있지 않다.
- Attack Preconditions: 사용자가 악성/오타 config를 저장하거나 custom endpoint가 redirect/DNS를 제어해야 한다.
- Impact: API key가 의도하지 않은 host로 전송되거나 내부망 endpoint에 request가 도달할 위험, provider-only network policy 의미론 약화.
- Suggested Action: URL parse/scheme/host/IP policy를 명세화하고, loopback local provider와 public provider를 별도 allowlist/opt-in으로 분리한다. redirect를 끄거나 same-origin 검증 후 허용하고 cross-origin에서는 auth header를 제거한다. Request URL에 credential query가 들어가지 않도록 한다.
- Re-audit Method: mock provider가 same-origin/cross-origin redirect, private/loopback DNS, invalid scheme, custom header를 반환하는 fixture를 사용해 request destination과 headers를 캡처한다.
- Confidence: Medium
- Notes: local Ollama/LM Studio 지원 때문에 허용 대상의 제품 정책은 `Needs Spec Clarification`이 함께 필요하다.

### [A03-F020] Gemini key와 provider 오류 본문이 redaction 경로를 우회해 session/UI에 남을 수 있음

- Pattern: `SEC-001`, `SEC-005` — secret redaction and documentation boundary
- Area: provider auth URL, error redaction, session persistence
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: Gemini credential validation/model fetch는 API key를 URL query에 넣는다. Provider error를 처리하는 `ChatResponseErr`는 `mask_secrets()` 없이 UI와 session에 그대로 추가한다. `mask_secrets()`는 tool output/stream chunk 경로에만 호출된다.
- Evidence:
  - `src/providers/registry.rs:467-476`, `:715-724`는 `...?key={api_key}` URL을 만든다.
  - `src/providers/registry.rs:197-203`, `:319-325`, `src/providers/anthropic.rs:366-372`는 response body를 `ProviderError::ApiResponse` message로 보존한다.
  - `src/app/mod.rs:916-952` (`ChatResponseErr`)는 `e.to_actionable()`를 timeline과 `ChatMessage::System`에 직접 저장한다.
  - `src/app/mod.rs:1692-1728`의 `mask_secrets()` 호출은 ToolFinished/ToolOutputChunk 등에서만 보이며 ChatResponseErr에는 없다.
  - `README.md:17`, `:69`는 “API key leakage zero/streaming masking”을 강하게 주장하고, `audit_roadmap.md:328-333`은 화면·파일 로그 redaction을 합격 기준으로 둔다.
- Expected Basis: `SEC-001`, `SEC-005`, `spec.md:1954-1987`의 모든 표시/로그 경계 API key 100% masking.
- Actual: proxy/transport error가 URL을 포함하거나 provider/custom server가 error body에 credential을 echo하면 key가 session/UI/log로 전파될 수 있다. Gemini key는 URL query라 intermediary/proxy/access log에도 노출될 수 있다.
- Attack Preconditions: Gemini validation/fetch 또는 custom/provider error가 발생하고 error string/body/URL에 key가 포함되어야 한다.
- Impact: API credential disclosure to session file, TUI, copied logs, proxy/access logs; key rotation 필요.
- Suggested Action: Gemini는 supported header auth를 우선 사용하고 URL query를 제거한다. ProviderError는 URL/header/body를 typed-redacted form으로 만들고 모든 UI/session/log sink를 통과하는 중앙 redaction boundary를 둔다. Error body cap도 함께 둔다.
- Re-audit Method: mock adapter가 URL/key를 포함한 network error와 response body를 반환하게 하여 UI/timeline/session/log 모든 sink에 `[REDACTED]`만 남는지 확인한다.
- Confidence: High

### [A03-F021] provider SSE/error 응답을 `response.text()`로 통째로 버퍼링해 stream/output cap이 없음

- Pattern: `SEC-004` — network response resilience and output cap
- Area: provider streaming, request timeout, memory
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: OpenAI-compatible와 Anthropic streaming adapter가 실제 byte stream을 읽지 않고 `response.text().await`로 전체 body를 메모리에 쌓은 뒤 파싱한다. error body도 cap이 없다.
- Evidence:
  - `src/providers/registry.rs:319-337` OpenAI-compatible stream error/text full-body read.
  - `src/providers/anthropic.rs:366-385` Anthropic stream error/text full-body read.
  - `src/providers/registry.rs:197-203`, `src/providers/anthropic.rs:175-181` non-2xx body도 unbounded `response.text()`다.
  - `src/app/chat_runtime.rs:587-595`는 adapter 전체 future에 60초 timeout을 둘 뿐 response byte cap을 두지 않는다.
- Expected Basis: `README.md:26`, `:78`의 token streaming과 `spec.md:1994-2018`의 수십 MB output memory safety. Network timeout은 size cap을 대체하지 않는다.
- Actual: provider/custom endpoint가 60초 안에 매우 큰 SSE/error body를 보내면 memory spike/OOM이 발생한다. UI delta는 body 전체 수신 후에야 전달되어 true streaming/early cancellation도 아니다.
- Attack Preconditions: provider endpoint/redirect/custom backend가 large or endless response를 보낼 수 있어야 한다.
- Impact: process memory exhaustion, TUI stall, delayed cancellation, partial response integrity ambiguity.
- Suggested Action: `bytes_stream()`/line framing으로 incremental parse하고 total bytes, line bytes, tool argument bytes, delta count를 cap한다. overflow 시 response를 abort하고 `is_truncated`/typed error를 반환한다.
- Re-audit Method: mock provider가 slow/chunked/large SSE와 large error body를 보내도록 하여 peak memory, first-delta latency, 60s cancel, cap metadata를 측정한다.
- Confidence: High

### [A03-F022] SafeOnly/whitelist가 command argument의 workspace 범위를 검사하지 않아 host read를 자동 허용함

- Pattern: `SEC-004` — path, workspace, shell controls must be independent
- Area: PermissionEngine SafeOnly, command argument paths
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: SafeOnly는 첫 token과 `safe_to_auto_run`만 검사한다. `cat`, `grep`, `find`, `ls` 등 허용 binary의 arguments는 workspace 밖인지 검사하지 않는다. Sandbox 기본값은 disabled라 host shell이 선택된다.
- Evidence:
  - `src/tools/shell.rs:473-505`는 `parts[0]`만 `safe_commands`/builtin 목록과 비교한다.
  - `src/domain/permissions.rs:177-209`의 whitelist도 first word만 검사하며 `cat/grep/find/ls`를 허용한다. 민감 경로 guard는 `sudo`/`rm`에만 적용된다(`:187-198`).
  - `src/tools/shell.rs:82-87`, `:151-160`은 sandbox false이면 host `sh -c`를 실행한다.
  - `src/domain/settings.rs:127-133` SandboxConfig 기본값은 `enabled=false`다.
  - `spec.md:516-520`, `:1940`은 workspace boundary/경로 차단을 안전 기준으로 둔다.
- Expected Basis: SafeOnly는 안전한 자동 실행이어야 하고 `SEC-004`는 shell 허용과 path/workspace 검사를 분리해 강제하라고 한다.
- Actual: trusted workspace + SafeOnly + `safe_to_auto_run=true`에서 `cat /etc/passwd`나 `grep secret /home/...`가 Allow가 될 수 있다. 사용자가 sandbox를 켜지 않은 일반 wizard 상태에서 host read boundary가 없다.
- Attack Preconditions: 사용자가 SafeOnly/Balanced를 선택하고 모델/문서 prompt injection이 허용 binary에 외부 경로 argument를 넣어야 한다.
- Impact: workspace 밖 파일/credential read와 LLM context exfiltration, “workspace-scoped shell” 기대 위반.
- Suggested Action: command parser로 executable/argument path를 분리해 각 path를 canonical workspace/allowlist로 검증한다. SafeOnly 자동 실행은 read-only capability와 path scope를 함께 선언하고, sandbox disabled 상태에서는 외부 path command를 Ask/Deny한다.
- Re-audit Method: SafeOnly fixture에서 `cat /etc/passwd`, `grep /home/...`, `ls /tmp`, `find -path`와 workspace 내부 정상 명령을 각각 검사하고 decision/실행 path를 확인한다.
- Confidence: High

### [A03-F023] 승인 대기 후 PermissionEngine을 재실행하지 않아 최신 정책을 우회할 수 있음

- Pattern: `SEC-004` — approval and policy revalidation
- Area: stale approval, policy change, tool execution
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: 최초 dispatch에서 `PermissionEngine::check()`가 Ask를 반환한 뒤, 사용자가 정책을 바꿔도 승인 처리에서는 Harness preflight만 재검사하고 PermissionEngine을 재검사하지 않는다.
- Evidence:
  - 최초 path `src/app/tool_runtime.rs:102-127`은 preflight 후 `PermissionEngine::check`를 수행한다.
  - 승인 path `src/app/tool_runtime.rs:507-539`는 현재 settings로 preflight를 확인하지만 `PermissionEngine::check(&tool, &settings)` 호출이 없다.
  - 이후 `src/app/tool_runtime.rs:551-564`에서 바로 `execute_tool_async`한다.
  - `src/app/tool_runtime.rs:647-663` direct shell은 별도 preflight 없이 처음 PermissionEngine만 호출하는 인접 bypass surface다.
- Expected Basis: `spec.md:54.1`의 “tool 실행 직전” policy/preflight와 `SEC-004`의 approval hard boundary. 사용자가 승인할 때의 최신 정책이 최종 권위여야 한다.
- Actual: ShellPolicy Ask로 pending 된 `cat /etc/passwd`가 승인 대기 중 SafeOnly/Deny로 바뀌어도 실행 경로에서 최신 shell policy를 적용하지 않는다.
- Attack Preconditions: tool이 Ask 상태로 pending된 동안 사용자가 policy를 변경하거나 config/runtime state가 drift해야 한다.
- Impact: 사용자가 현재 선택한 Deny/SafeOnly 정책을 stale approval이 우회한다. 모델이 기다리는 동안 정책을 강화해도 효과가 없다.
- Suggested Action: 승인 직전 preflight와 PermissionEngine을 모두 재실행하고, decision fingerprint(툴 args, policy, root, server/schema)를 approval record에 저장해 drift 시 새 Ask로 전환한다. direct shell도 동일 preflight pipeline으로 통합한다.
- Re-audit Method: Ask pending → policy Ask→Deny/SafeOnly 변경 → approve fixture에서 실행이 차단되고 새 approval이 요구되는지 확인한다.
- Confidence: High

### [A03-F011] session log와 sessions directory가 개인정보를 0600으로 고정하지 않음

- Pattern: `SEC-001` — secret/privacy storage boundary
- Area: session log file mode, directory mode, personal data
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: 세션에는 사용자 prompt, provider/tool 결과, workspace path와 harness snapshot이 저장되지만 directory/file creation에 Unix mode가 없다. 실제 mode는 process umask에 의존한다.
- Evidence:
  - `src/infra/session_log.rs:43-59` (`new_session`)은 `create_dir_all`과 `OpenOptions::create+append`만 호출한다.
  - `src/infra/session_log.rs:328-348` (`new_workspace_session`)도 같은 방식이며 mode 0700/0600이 없다.
  - `src/infra/session_log.rs:140-164`는 직렬화된 `ChatMessage`를 JSONL 평문으로 저장한다.
  - `src/infra/session_log.rs:167-183`는 harness record에 canonical workspace/root/shell/sandbox를 저장한다.
  - `README.md:27`, `:79`는 `~/.smlcli/sessions/` 자동 기록을 공개 기능으로 설명하지만 privacy/mode caveat가 없다.
- Expected Basis: 세션은 local-only라도 개인정보와 tool output을 포함하므로 owner-only directory/file boundary가 필요하다(`SEC-001`).
- Actual: mode가 OS umask에 맡겨져 0644/0755가 될 수 있으며, 세션 로그는 암호화되지 않는다.
- Attack Preconditions: 같은 host의 다른 계정 또는 shared backup/파일 수집기가 sessions directory를 읽을 수 있어야 한다.
- Impact: prompt, 코드 경로, 내부 tool 출력, harness 환경 정보와 provider error가 다른 사용자에게 노출된다.
- Suggested Action: directory를 0700, log/index/rotation backup을 0600으로 `create_new`하고 기존 파일 mode/owner를 확인·교정한다. privacy 요구에 맞게 log encryption/redaction/retention을 명세화한다.
- Re-audit Method: umask 000/022와 pre-existing loose file 환경에서 생성·rotation·resume 후 `stat` mode/owner를 확인하고, log contents가 정책대로 redacted/encrypted인지 검증한다.
- Confidence: High

### [A03-F012] session restore/index는 손상 시 조용히 초기화되고 크기·경로 무결성이 없다

- Pattern: `SEC-001` — user data integrity and recovery
- Area: JSONL corruption, SessionIndex atomicity, unbounded restore
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: JSONL 복원은 한 줄 길이/전체 메시지 수를 제한하지 않고, SessionIndex parse failure를 빈 목록으로 바꾼다. index의 `log_filename`은 sessions directory 내부인지 검증하지 않고 `/resume`에서 join한다.
- Evidence:
  - `src/infra/session_log.rs:218-262` (`restore_messages`)는 `BufRead::lines()`와 `serde_json::from_str`를 무제한으로 수행하며 오류를 세지만 호출자가 복구 결정을 강제받지 않는다.
  - `src/infra/session_log.rs:296-311` (`list_sessions`)는 각 JSONL 전체를 `read_to_string`으로 읽어 line count를 계산한다.
  - `src/infra/session_log.rs:399-409` (`SessionIndex::load_all`)은 JSON parse 실패 시 `unwrap_or_default()`로 빈 index를 반환한다.
  - `src/infra/session_log.rs:470-484` (`save_all`)은 lock/unique temp/atomic rename 없이 직접 `std::fs::write`한다.
  - `src/app/command_router.rs:993-1010`은 index의 `target.log_filename`을 `log_dir.join(...)`해 `from_file`에 넘긴다. filename containment 검사가 없다.
  - `src/app/command_router.rs:900-906`은 raw `current_dir()`로 목록을 조회하는 반면 `/new`는 canonical root를 저장한다(`:852-854`).
- Expected Basis: `spec.md` Phase 46와 `report-contract`의 data integrity 불변조건; 손상은 보존·진단되어야 하고 session path는 sessions root 안이어야 한다.
- Actual: 한 줄이 매우 크면 restore/list가 memory를 소모한다. index corruption은 빈 목록으로 위장되어 사용자 session metadata를 잃고, tampered absolute/path-containing filename은 arbitrary local path를 열 수 있다. canonical/raw root drift로 resume list가 누락될 수 있다.
- Attack Preconditions: 손상/동시 write/대형 tool output 또는 동일 사용자 프로세스가 `sessions_index.json`을 수정해야 한다.
- Impact: 세션 목록·복원 불능, memory DoS, 의도하지 않은 파일 read/append, workspace 세션 격리 실패.
- Suggested Action: index에 lock+journal/atomic replace+backup recovery를 적용하고 parse failure를 명시적 corruption state로 노출한다. filename은 UUID basename만 허용하고 canonical containment를 재검증한다. JSONL line/message/total size cap과 streaming parser를 둔다. 목록/저장 root는 canonical root 한 source로 통일한다.
- Re-audit Method: truncated JSON, invalid UTF-8, multi-GB line fixture, concurrent index writers, absolute/`../` log filename, symlinked log path를 재생하고 원본 보존·격리·bounded memory를 확인한다.
- Confidence: High

### [A03-F013] rollback_checkpoint가 checkpoint ref를 무시하고 현재 HEAD 전체를 hard reset 함

- Pattern: `SEC-004` — data integrity and destructive rollback
- Area: Git checkpoint/rollback
- Severity: Critical
- Status: Confirmed
- Standard Disposition: Needs Fix / Hold
- Summary: `create_checkpoint()`는 clean 시점의 ref를 기록하지만 `rollback_checkpoint()`는 그 ref를 사용하지 않고 현재 `HEAD`로 전체 tracked tree를 reset한다. checkpoint 생성 후 사용자/동시 프로세스가 만든 tracked 변경도 삭제될 수 있다.
- Evidence:
  - `src/tools/git_checkpoint.rs:208-233`은 clean 여부를 한 번 확인한 뒤 `refs/smlcli/checkpoints/<timestamp>_<tool>`에 HEAD를 저장한다.
  - `src/tools/git_checkpoint.rs:239-266`은 저장된 ref/hash를 조회하지 않고 `git reset --hard HEAD`를 실행한다.
  - `src/tools/executor.rs:14-51`은 destructive tool error/cancellation이면 이 rollback을 호출한다.
  - `spec.md:2210-2213`, `:953-964`, `IMPLEMENTATION_SUMMARY.md:532-539`는 자동 rollback을 안전망으로 주장한다.
- Expected Basis: rollback은 작업 시작 snapshot으로만 되돌리고 사용자 변경을 보존해야 한다. `git clean -fd`를 제거했다는 사실만으로 tracked data loss가 해결되지 않는다.
- Actual: clean check와 tool execution 사이 또는 tool execution 중 tracked file을 변경한 사용자의 WIP가 있으면 hard reset이 모두 사라진다. tool이 HEAD를 바꾼 경우 현재 HEAD를 reset해 pre-tool state를 복구하지 못한다.
- Attack Preconditions: 승인된 destructive tool이 실패/취소되고, checkpoint 이후 concurrent/user tracked change 또는 tool의 git HEAD mutation이 있어야 한다.
- Impact: 사용자 코드의 비가역적 손실, 자동 복구가 오히려 손상을 일으키는 Critical gate.
- Suggested Action: checkpoint ref를 immutable snapshot으로 저장하고 rollback은 해당 hash에 대해 path-scoped restore 또는 index/worktree state transaction을 사용한다. checkpoint 이후 변경이 있으면 conflict/hold로 중단하고 reset하지 않는다. tool이 HEAD/index를 바꾸지 못하게 별도 guard를 둔다.
- Re-audit Method: clean repo에서 checkpoint 후 tracked WIP를 삽입하고 failing destructive tool을 실행해 WIP가 보존되는지 확인한다. HEAD-changing command, merge/rebase, untracked creation 각각의 rollback 결과를 hash/diff로 검증한다.
- Confidence: High

### [A03-F014] auto_commit이 기존 staged 사용자 변경까지 함께 커밋할 수 있음

- Pattern: `SEC-004` — Git data ownership boundary
- Area: selective staging, auto-commit
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: `GitEngine::auto_commit()`은 지정 파일을 `git add`한 뒤 전체 staged diff가 비어있는지만 검사하고 commit한다. 호출 전 index에 사용자가 stage해 둔 다른 파일이 있으면 함께 커밋된다.
- Evidence:
  - `src/infra/git_engine.rs:31-52`는 파일을 add하지만 pre-existing index snapshot을 검사하거나 보존하지 않는다.
  - `src/infra/git_engine.rs:55-81`은 `git diff --cached --quiet` 후 전체 index를 `git commit`한다.
  - `src/tests/audit_regression.rs:3019-3048`의 `test_git_auto_commit_selective_staging`은 clean index에서 대상 파일 하나만 검증한다.
  - `src/tests/audit_regression.rs:3050-3080`의 WIP 테스트는 unrelated file을 unstaged로 두므로 pre-staged WIP를 검증하지 않는다. 해당 필터 명령은 PASS했다.
- Expected Basis: `spec.md:2262-2268`, `:2285-2289`의 affected_paths만 stage하여 사용자 WIP를 보호한다는 계약.
- Actual: pre-staged WIP/secret file이 있으면 대상 파일 add 후 전체 staged set이 자동 커밋된다.
- Attack Preconditions: 사용자가 index에 별도 변경을 stage한 상태에서 auto_commit opt-in과 affected path가 있는 성공 tool이 실행되어야 한다.
- Impact: 사용자 승인 없는 코드/secret commit, auto-commit provenance 혼합, 이후 `/undo` 시 unrelated 변경까지 revert될 수 있음.
- Suggested Action: pre-commit index/tree snapshot을 비교하고, dedicated temporary index 또는 `git commit <paths>`/pathspec 기반으로 대상만 커밋한다. pre-existing staged changes가 있으면 auto-commit을 skip/Ask한다.
- Re-audit Method: target과 unrelated file을 모두 staged/unstaged 조합으로 만들어 commit tree와 `git diff --cached`를 비교한다. commit 후 user index/worktree가 byte-identical인지 확인한다.
- Confidence: High

### [A03-F015] MCP child가 parent의 전체 환경변수를 상속하여 secret boundary를 우회함

- Pattern: `SEC-001` — secret storage and external process boundary
- Area: MCP stdio child environment
- Severity: Critical
- Status: Confirmed
- Standard Disposition: Needs Fix / Hold
- Summary: shell 도구에는 `env_clear()`가 있지만 MCP `Command::new(cmd)`에는 env clearing/allowlist가 없다. 설정된 MCP 서버는 smlcli parent의 모든 환경변수(환경 기반 API key 포함)를 읽을 수 있다.
- Evidence:
  - `src/infra/mcp_client.rs:41-48`은 command/args/stdin/stdout/stderr만 설정하고 spawn한다.
  - 같은 파일에 `env_clear`, `env_remove`, allowlist 설정이 없다.
  - 비교로 `src/tools/shell.rs:168-180`은 shell에 `env_clear()` 후 whitelist를 주입한다.
  - `src/app/mod.rs:173-179`은 config에 등록된 MCP command를 앱 시작 시 자동 spawn한다.
- Expected Basis: `SEC-001`, `spec.md:1994-2010`의 외부 process 환경 격리 계약. MCP는 arbitrary executable이므로 shell보다 약한 경계가 될 수 없다.
- Actual: `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, cloud credentials 등 parent environment가 그대로 MCP server에 노출된다. debug build의 stderr도 `eprintln!`로 터미널에 출력된다(`mcp_client.rs:55-72`).
- Attack Preconditions: 사용자가 환경변수에 secret을 둔 상태에서 신뢰되지 않거나 탈취된 MCP server command가 config에 등록되어야 한다.
- Impact: MCP server가 API keys/cloud tokens를 읽어 외부로 전송할 수 있으며, release 문서의 환경 격리·secret 보호가 무력화된다.
- Suggested Action: MCP child도 `env_clear()` 후 protocol에 필요한 최소 env만 전달하고, inherit를 명시적 per-server opt-in으로 제한한다. stderr는 bounded/redacted diagnostic channel로 보낸다. 기존 config의 command path/owner도 검증한다.
- Re-audit Method: fixture MCP server가 `/proc/self/environ`/환경 dump를 반환하도록 하되 실제 secret은 사용하지 않고, 허용된 변수만 존재하는지 확인한다. debug/release stderr redaction과 child startup failure를 검증한다.
- Confidence: High

### [A03-F016] MCP stdio reader와 tools/call 결과에 line/output/schema 크기 경계가 없음

- Pattern: `SEC-004` — untrusted input/process output boundary
- Area: JSON-RPC input, output cap, schema parsing
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: MCP stdout/stderr는 `read_line`으로 무제한 누적되고, `parse_call_tool_result()`는 모든 content text를 무제한 `String`에 붙인다. `ToolResult`에는 truncation metadata도 채우지 않는다.
- Evidence:
  - `src/infra/mcp_client.rs:55-71` stderr reader가 line size cap 없이 `read_line`한다.
  - `src/infra/mcp_client.rs:170-195` stdout reader가 line size/JSON size/response count cap 없이 `read_line`과 `serde_json::from_str`를 수행한다.
  - `src/infra/mcp_client.rs:281-287`은 caller arguments를 바로 JSON-RPC `tools/call`에 넣는다.
  - `src/infra/mcp_client.rs:295-327`은 content text를 무제한 append/trim하고 raw response도 그대로 반환한다.
  - `src/app/tool_runtime.rs:435-444`는 MCP output을 `ToolResult.stdout`에 그대로 넣고 `is_truncated=false`, `original_size_bytes=None`으로 만든다.
- Expected Basis: `spec.md:1994-2018`, `report-contract`의 untrusted input/output와 cancellation/size cap 불변조건.
- Actual: MCP server가 한 줄에 매우 큰 JSON 또는 많은 content를 보내면 memory/CPU를 소모한다. malformed schema/tool entry는 조용히 drop된다(`mcp_client.rs:261-273`)며 사용자에게 provenance가 남지 않는다.
- Attack Preconditions: config에 등록된 MCP server가 악성/오작동하거나, compromised local binary가 10초 안에 큰 line/result를 보낼 수 있어야 한다.
- Impact: TUI/LLM context OOM, event loop 지연, truncated output을 성공 결과로 오인, malformed tool schema로 기능 목록과 실행 경로 drift.
- Suggested Action: framed JSON-RPC line/byte cap, max pending/server/tool/schema count, response content cap을 두고 초과 시 child shutdown/typed error로 종료한다. malformed schema는 경고와 함께 해당 server를 격리하고 result metadata를 정확히 표시한다.
- Re-audit Method: fixture server가 1MB/10MB single-line, endless stderr, huge `content`, malformed JSON/schema를 보내도록 하고 bounded memory, timeout, user-visible error, child termination을 검증한다.
- Confidence: High

### [A03-F017] MCP route는 cancellation을 무시하고 startup/routed child lifecycle을 추적하지 않음

- Pattern: `SEC-004` — process lifecycle and cancellation
- Area: MCP tool cancellation, shutdown, startup race
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: 일반 tool은 `executor::execute_tool()`에서 `CancellationToken`을 select하지만 MCP route는 `client.call_tool()`을 직접 await한다. `McpClient::shutdown()`은 direct child 한 개만 kill하며, 앱 종료 시 runtime map에 저장된 client만 순회한다.
- Evidence:
  - `src/app/tool_runtime.rs:426-463` MCP route는 `call_tool`만 await하고 `cancel_token`을 사용하지 않는다.
  - 같은 파일 `:464-476`의 non-MCP route만 `execute_tool(..., cancel_token)`을 호출한다.
  - `src/infra/mcp_client.rs:135-142` shutdown은 `child.kill()`만 수행하고 process group/descendant wait가 없다.
  - `src/app/mod.rs:173-180` startup spawn task는 client를 이벤트로 보내기 전 별도 registry가 없다.
  - `src/app/mod.rs:422-428` 종료 시 `mcp_clients.values()`만 shutdown한다. queue 중인 `McpToolsLoaded`/failed task와 send 실패 client는 추적되지 않는다.
- Expected Basis: 사용자 목표의 MCP process lifecycle/cancellation/timeouts, `spec.md:1991-2018`의 orphan 차단.
- Actual: cancel 키가 pending MCP call을 중지하지 않으며, 앱 종료 race에서 event에 도달하지 않은 client나 child descendants가 남을 수 있다. 10초 request timeout은 call만 끝내고 server process를 종료하지 않는다.
- Attack Preconditions: MCP tool이 pending 상태일 때 사용자가 cancel/quit하거나 MCP server가 child process를 만든다.
- Impact: 실행 취소 실패, 외부 mutation 지속, zombie/orphan process와 resource leak.
- Suggested Action: MCP call에 cancellation-aware request/child shutdown을 연결하고, server마다 process group/kill-on-drop/lease registry를 둔다. startup task와 loaded client를 공통 supervisor가 추적하고 app shutdown을 supervisor join으로 닫는다.
- Re-audit Method: hanging mock MCP server, child-spawning server, cancel/quit 중 startup race fixture에서 request, direct child, descendants, pending map이 모두 정리되는지 확인한다.
- Confidence: High

### [A03-F018] MCP tools/call 인자가 schema로 검증되지 않고 승인 화면도 실제 payload를 설명하지 않음

- Pattern: `SEC-004` — approval, external tool input and schema boundary
- Area: MCP schema/input validation, user consent
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix / Needs Spec Clarification
- Summary: `tools/list`의 `inputSchema`는 provider schema로 복사될 뿐 call-time validation에 사용되지 않는다. 승인 카드에서 MCP는 GLOBAL_REGISTRY에 없는 도구로 표시되어 server, 원본 schema, arguments가 사용자에게 보이지 않는다.
- Evidence:
  - `src/infra/mcp_client.rs:261-273`은 `McpToolInfo`를 역직렬화하고 invalid entry를 skip할 뿐 schema를 검증/compile하지 않는다.
  - `src/infra/mcp_client.rs:281-287`은 `arguments`를 schema validation 없이 `tools/call`에 전달한다.
  - `src/app/tool_runtime.rs:166-184` approval detail은 registry tool이 있는 경우에만 detail/diff를 생성한다.
  - `src/app/tool_runtime.rs:316-320` registry에 없는 `mcp_` tool은 `알 수 없는 도구`만 표시한다.
  - `src/domain/permissions.rs:169-170`은 `mcp_` prefix만 보고 무조건 Ask한다. 실제 server/name/input binding은 별도 검증하지 않는다.
- Expected Basis: MCP 도구는 외부 mutation surface이며, 사용자 승인에는 대상·인자·schema provenance가 있어야 한다. `spec.md:2427-2478`의 MCP 위임 계약을 안전하게 닫아야 한다.
- Actual: 모델이 schema 밖의 path/command/secret-bearing arguments를 만들 수 있고 사용자는 opaque approval만 보게 된다. MCP server의 자체 검증을 신뢰하는 구조지만 client hard boundary가 없다.
- Attack Preconditions: 악성/오작동 모델이 MCP tool call을 생성하거나 MCP server가 permissive schema를 제공해야 한다.
- Impact: 사용자가 의도하지 않은 external tool mutation/data read를 승인할 가능성, audit trail의 대상·인자 불명확.
- Suggested Action: schema를 JSON Schema subset으로 compile하여 call 전에 validate하고, server/name/schema hash와 redacted arguments를 approval card에 표시한다. validation 불능 schema는 tool을 노출하지 말고 Ask/deny 정책을 명세화한다.
- Re-audit Method: required/type/additionalProperties/path-like argument fixture와 malformed schema를 사용해 validation/approval text/route가 일치하는지 확인한다.
- Confidence: High

### [A03-F005] bwrap extra_binds가 arbitrary host source/target을 그대로 sandbox에 추가함

- Pattern: `SEC-004` — path/workspace/shell/file-mode 제어군 분리
- Area: sandbox mount policy, extra bind trust
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix / Needs Spec Clarification
- Summary: `extra_binds`는 사용자 설정 문자열을 `:` 기준으로만 나누어 `bwrap --bind`에 전달한다. source/target canonicalization, workspace containment, read-only 여부, target allowlist가 없다.
- Evidence:
  - `src/infra/sandbox.rs:60-68`은 `parts[0]`, `parts[1]`을 검증 없이 `--bind` 인자로 넘긴다.
  - `src/tools/shell.rs:139-156`은 config의 `settings.sandbox.extra_binds`를 그대로 wrapper에 전달한다.
  - `src/infra/workspace_harness.rs:49-61`, `:109-116`은 extra bind를 snapshot/prompt에 표시하지만 안전한 mount contract를 강제하지 않는다.
  - `README.md:17`, `:69`은 이를 “strict ... bwrap sandbox”로 요약한다.
- Expected Basis: sandbox가 workspace 외부 접근을 차단한다는 `spec.md:2353-2395` 계약. Extra bind가 escape hatch라면 명시적으로 trusted host configuration으로 분류되어야 한다.
- Actual: config에 `/home/user/.ssh:/mnt/secret` 또는 `/etc:/workspace/etc` 같은 bind를 넣으면 sandbox가 그 host path를 노출/쓰기 가능하게 만들 수 있다. colon-containing path도 조용히 잘못 해석된다.
- Attack Preconditions: 공격자가 config를 변경하거나 사용자가 검증되지 않은 extra bind를 입력해야 한다.
- Impact: sandbox의 filesystem boundary가 설정 한 줄로 임의 확장되고, read/write secret exposure가 발생한다.
- Suggested Action: source는 canonical existing path이며 허용된 host roots 아래인지 검사하고, target은 고정 guest allowlist로 제한한다. 기본은 `--ro-bind`, write bind는 별도 explicit confirmation으로 분리하며 malformed entry는 Deny한다.
- Re-audit Method: `/etc`, home secret, workspace 밖 directory, symlink source/target, colon path fixtures를 wrapper argv와 실제 bwrap mount에서 검증한다.
- Confidence: High
- Notes: extra bind의 제품 의도(임의 host bind 허용 여부)가 명세에서 충분히 구분되지 않아 명세 보완도 필요하다.

### [A03-F006] shell outer timeout이 process group을 죽이지 않아 descendant가 남을 수 있음

- Pattern: `SEC-004` — process lifecycle/cancellation
- Area: timeout, child process group, orphan processes
- Severity: Major
- Status: Probable
- Standard Disposition: Needs Fix
- Summary: 전체 실행을 30초 outer `timeout`으로 감싸면서 내부 `child.wait()`에도 같은 30초 timeout을 둔다. outer timeout이 먼저 만료되면 내부 `kill_process_group()`가 실행되지 않고 future drop과 `kill_on_drop`만 direct child에 의존한다.
- Evidence:
  - `src/tools/shell.rs:195-197` outer `tokio::time::timeout(30s, async { ... })`.
  - `src/tools/shell.rs:328-341` inner wait timeout/cancel 경로에서만 `kill_process_group()`를 호출한다.
  - `src/tools/shell.rs:192-193`의 `kill_on_drop(true)`는 `Child` 자체에 대한 보호이며 descendant group 종료를 보장하지 않는다.
  - `src/infra/sandbox.rs:21-76` bwrap에도 `--die-with-parent` 또는 pid namespace/process-group lifecycle 옵션이 없다.
  - `spec.md:1991-2018`은 zombie/orphan 완전 차단을 성공 기준으로 주장한다.
- Expected Basis: 취소/timeout 시 shell process tree가 모두 종료되어야 한다.
- Actual: output reader task가 아직 완료되지 않은 상태에서 outer deadline이 이기면 direct child만 drop/kill되고, child가 만든 background process는 계속 실행될 수 있다.
- Attack Preconditions: 명령이 background descendant를 만들고 parent/pipe lifecycle이 outer deadline과 경합해야 한다.
- Impact: 지속적인 CPU/network/file mutation, secret-bearing child 잔류, 이후 process reaper와의 충돌.
- Suggested Action: 하나의 deadline source만 사용하고 timeout/cancel 모두 같은 group-kill 경로를 호출한다. Linux에는 `--die-with-parent`/pid namespace를 적용하고 kill 후 group wait/verification을 수행한다.
- Re-audit Method: `sh -c 'sleep ... & ...'` fixture에서 timeout/cancel 후 PGID와 descendant 생존 여부를 확인하고, outer/inner race를 반복한다.
- Confidence: Medium

### [A03-F007] process_reaper가 SMLCLI_PID 환경변수만 믿어 타 프로세스를 오살할 수 있음

- Pattern: `SEC-004` — process identity/lifecycle boundary
- Area: orphan reaping, PID reuse, process identity
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: reaper는 실제 parent-child 관계나 executable identity를 확인하지 않고, 임의 프로세스의 `SMLCLI_PID` 문자열과 PID 생존 여부만으로 kill 목록을 만든다.
- Evidence:
  - `src/infra/process_reaper.rs:17-26`은 모든 process environment에서 `SMLCLI_PID=`를 찾는다.
  - `src/infra/process_reaper.rs:34-43`은 해당 PID가 없으면 고아로 확정한다.
  - `src/infra/process_reaper.rs:47-50`은 PID 재검증 없이 `p.kill()`을 호출한다.
  - `src/tools/shell.rs:178-180`은 모든 child command에 현재 smlcli PID를 주입한다.
  - `src/main.rs:58-66`, `:93-100`은 일반 TUI 시작 때도 reaper를 자동 실행한다.
- Expected Basis: `spec.md:2171-2194`와 코드 주석은 “SMLCLI 자식만” 정리하고 다른 프로세스는 절대 건드리지 않는다고 주장한다.
- Actual: 사용자가 `SMLCLI_PID=0` 또는 종료된 임의 PID를 환경에 넣은 unrelated process는 다음 smlcli 시작 시 kill 대상이 된다. PID reuse도 parent identity를 보장하지 않는다.
- Attack Preconditions: 같은 사용자 권한으로 환경변수를 설정한 프로세스가 존재하거나, 오래된 PID를 가진 정상 process가 있어야 한다.
- Impact: 사용자의 장기 실행 build/editor/server가 강제 종료되고 데이터 손상/작업 중단이 발생한다.
- Suggested Action: Unix parent PID/PGID와 executable path, start time/token을 함께 확인하고 smlcli가 생성한 opaque lease를 검증한다. 안전한 descendant tree가 아니면 kill하지 말고 진단만 한다. 기본 startup 자동 reaping 대신 명시적 opt-in을 검토한다.
- Re-audit Method: fixture process가 spoofed/stale `SMLCLI_PID`를 갖는 경우 reaper가 kill하지 않는지, 실제 smlcli child만 clean 되는지 process start time/parent tree 기반 테스트로 확인한다.
- Confidence: High

### [A03-F008] master key/config 파일의 권한·symlink·생성 원자성이 강제되지 않음

- Pattern: `SEC-001` — secret storage/config persistence boundary
- Area: key permissions, config lock, symlink and file creation
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: 새 master key에는 Unix mode 0600을 지정하지만 기존 파일 권한을 검사/교정하지 않고, config directory/lock path에는 명시 mode가 없다. `path.exists()` 뒤에 일반 `create/truncate`를 사용하여 symlink와 생성 race도 허용한다.
- Evidence:
  - `src/infra/secret_store.rs:29-36`은 `create_dir_all` 후 `path.exists()`를 검사한다.
  - `src/infra/secret_store.rs:53-66`은 새 파일에만 `mode(0o600)`을 적용하고 existing key의 mode를 검증하지 않으며 `sync_all`/atomic rename도 없다.
  - `src/infra/config_store.rs:37-54`의 directory/lock file 생성은 mode를 지정하지 않는다. lock file은 `truncate`이며 symlink no-follow가 없다.
  - `src/infra/config_store.rs:72-84`에서 temp config만 0600으로 설정한다.
  - `src/infra/secret_store.rs:15-19`, `config_store.rs:12-17`은 home을 찾지 못하면 현재 디렉터리 `.`로 fallback한다.
- Expected Basis: `SEC-001`, `spec.md:1109-1129`, `IMPLEMENTATION_SUMMARY.md:825-826`의 secret/config protected storage 계약.
- Actual: 이전/수동 생성된 `.master_key`가 loose mode여도 계속 사용한다. lock/config 디렉터리와 symlink target은 안전한 owner/mode/no-follow 경계가 아니다. home 미검출 환경에서는 프로젝트 cwd에 secret이 생길 수 있다.
- Attack Preconditions: 동일 호스트의 다른 계정/프로세스가 loose mode 파일을 읽거나, config directory 안에 symlink를 심을 수 있어야 한다.
- Impact: master key 획득 후 encrypted API key 복호화, 임의 파일 truncate/교체, secret이 workspace/현재 경로에 생성될 위험.
- Suggested Action: directory 0700, key/config/session 0600을 생성·검사·교정하고 owner/group을 확인한다. `create_new + O_NOFOLLOW`와 atomic temp-create/rename을 사용하며 home 미검출은 fail closed한다.
- Re-audit Method: loose mode/existing symlink/권한 경합 fixture와 crash injection을 실행해 key/config가 외부 target을 읽거나 truncate하지 않는지 확인한다.
- Confidence: High

### [A03-F009] malformed nonce가 `XNonce::from_slice` panic으로 설정 손상을 프로세스 DoS로 바꿈

- Pattern: `SEC-001` — encrypted secret parsing/error recovery
- Area: nonce length validation, malformed config
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: decrypt path는 hex decoding만 하고 nonce가 정확히 24 bytes인지 검사하지 않은 채 `XNonce::from_slice`를 호출한다. 잘린/악의적인 config encrypted value가 정상적인 `Result` 오류가 아니라 panic을 만들 수 있다.
- Evidence:
  - `src/infra/secret_store.rs:101-109`은 nonce/ciphertext를 unbounded hex decode한다.
  - `src/infra/secret_store.rs:111-117`은 `nonce_bytes.len()==24` 검증 없이 `XNonce::from_slice(&nonce_bytes)`를 호출한다.
  - `src/infra/secret_store.rs:137-148`의 `get_api_key`가 설정 로드 이후 이 경로를 호출한다.
  - `IMPLEMENTATION_SUMMARY.md:148-149`는 손상 TOML/설정 오류 graceful recovery를 완료했다고 주장한다.
- Expected Basis: 손상된 secret/config는 사용자에게 복구/재설정 오류를 제공해야 하며 process panic이 없어야 한다(`spec.md:1122-1129`).
- Actual: nonce length가 0/짧거나 과대하면 crypto crate의 fixed-size conversion precondition을 위반한다. ciphertext도 길이 상한이 없어 decode allocation이 제한되지 않는다.
- Attack Preconditions: 사용자가 config를 손상시키거나 동일 사용자 프로세스가 encrypted key 값을 바꿀 수 있어야 한다.
- Impact: 앱 시작/Provider 호출/secret masking 시 crash 및 서비스 거부, 대형 ciphertext에 의한 memory pressure.
- Suggested Action: nonce는 decode 직후 정확히 24 bytes인지 확인하고, ciphertext/plaintext 크기를 bounded parse한다. crypto conversion panic 가능 API 대신 length-checked constructor를 사용하고 malformed entry는 alias 단위로 격리한다.
- Re-audit Method: empty/23/24/25-byte nonce, invalid hex, huge ciphertext config fixture를 로드하여 모두 typed error와 UI recovery로 끝나는지 확인한다.
- Confidence: High

### [A03-F010] config temp cleanup/backup/rename은 동시성·crash 복구를 완전히 닫지 못함

- Pattern: `SEC-001` — config atomicity and data integrity
- Area: temp file cleanup, migration backup, fsync/rollback
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: `load_config()`가 writer lock 없이 모든 `.tmp`를 삭제하고, `save_config()`는 고정된 단일 temp path를 사용한다. migration backup은 성공 후 정리되지 않으며 directory fsync와 backup integrity 검증이 없다.
- Evidence:
  - `src/infra/config_store.rs:125-139` (`cleanup_tmp_files`)는 다른 writer가 쓰는 `.tmp`도 제거할 수 있다.
  - `src/infra/config_store.rs:47-70`은 lock을 잡은 뒤에도 고정 `config.toml.tmp`를 사용한다. load cleanup은 lock보다 먼저 실행된다(`:141-147`).
  - `src/infra/config_store.rs:100-120`은 temp file fsync 후 rename하지만 parent directory fsync는 없다.
  - `src/infra/config_store.rs:149-169`은 migration 전에 `.bak`을 만들고 실패 시 rename하지만 성공 시 backup을 남긴다. backup/restore return value도 모두 검증하지 않는다.
- Expected Basis: `BUILD_GUIDE.md:31-38`, `spec.md:2136-2159`의 config migration/backup/atomic 저장 및 기존 설정 보존 계약.
- Actual: concurrent load가 active temp를 삭제해 save가 실패할 수 있고, crash 전원 장애에서 rename durability가 보장되지 않는다. stale `.bak`가 계속 남아 설정 metadata와 encrypted values의 보존 범위를 넓힌다.
- Attack Preconditions: 두 smlcli 인스턴스/동시 config save 또는 migration 중 process kill/전원 장애가 있어야 한다.
- Impact: 사용자 설정 손실/rollback 실패, stale backup에 의한 민감 metadata 잔존, 설정 복구 불능.
- Suggested Action: lock을 cleanup/load에도 공통 적용하고 per-write unique temp를 lock 아래 생성한다. fsync parent directory, backup mode/retention/cleanup, rename/restore 실패 전파와 startup recovery journal을 구현한다.
- Re-audit Method: 두 concurrent writer와 load, temp 삭제 경합 및 rename 직전 crash injection을 반복하고 원본/backup/temp 중 하나로 deterministic recovery 되는지 확인한다.
- Confidence: High
- Notes: 현재 허용 테스트 `test_harness_preflight_denies_cwd_outside_workspace`, `test_exec_shell_cwd_absolute_path_outside_workspace`는 정적 정상 경로 판정만 검증하며 TOCTOU를 검증하지 않는다.

### [A03-F002] AllowAll FetchURL은 내부망·metadata endpoint와 redirect/DNS rebinding을 차단하지 않음

- Pattern: `SEC-003` — 네트워크 노출은 인증·허용목록 없이 통과 불가
- Area: FetchURL SSRF, redirect, DNS/IP policy
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: NetworkPolicy가 `AllowAll`이면 URL 문자열의 scheme만 확인하고 reqwest 기본 client로 요청한다. loopback, RFC1918, link-local/cloud metadata, DNS 결과, redirect 목적지에 대한 allowlist가 없다.
- Evidence:
  - `src/tools/fetch.rs:46-57`은 `AllowAll`을 무조건 `PermissionResult::Allow`로 반환하고, `ProviderOnly`에서만 FetchURL을 Deny한다.
  - `src/tools/fetch.rs:73-81`은 `http://`/`https://` prefix만 확인한 뒤 `reqwest::get(&url)`를 호출한다.
  - redirect 정책과 IP 재검증 코드가 저장소에 없다(`rg -n "redirect|resolve|lookup|loopback|169\\.254|127\\.0\\.0\\.1"` 결과 관련 enforcement 없음).
  - 신규 wizard는 `src/app/wizard_controller.rs:203-206`에서 `NetworkPolicy::AllowAll`을 저장하고, `designs.md:524-528`은 이를 Safe Starter 기본값으로 설명한다.
  - `spec.md:465-468`은 ProviderOnly의 SSRF 방지를 명시하지만 AllowAll의 내부망 경계는 정의하지 않는다.
- Expected Basis: `SEC-003`/네트워크 입력 경계. “외부 문서” 기능이어도 에이전트가 접근 가능한 네트워크 범위와 redirect 최종 목적지는 명시적 allowlist로 고정되어야 한다.
- Actual: 모델/페이지 prompt injection이 `http://127.0.0.1`, `http://169.254.169.254`, 사설 DNS, 또는 외부 URL의 redirect를 유도하면 로컬 서비스/metadata를 읽을 수 있다. AllowAll이 기본 wizard 경로여서 사용자가 별도 선택하지 않을 수 있다.
- Attack Preconditions: 사용자가 AllowAll/Safe Starter를 사용하고, 공격자가 모델 입력·웹 문서·workspace 데이터로 FetchURL URL을 제어하거나 redirect/DNS를 제공해야 한다.
- Impact: localhost 관리 API, 클라우드 credential metadata, 사설망 정보 노출과 prompt context로의 유입.
- Suggested Action: URL을 구조적으로 파싱하고 기본적으로 loopback/사설/link-local/Unix-local 목적지를 차단한다. DNS resolve 후 연결 IP와 redirect 각 hop을 재검증하고, redirect를 끄거나 동일 allowlist 내에서만 허용한다. AllowAll은 “공용 URL만”인지 “임의 URL”인지 명세를 먼저 확정한다.
- Re-audit Method: mock HTTP server와 redirect/DNS fixture로 public→loopback, public→metadata, IPv4/IPv6/decimal/hostname rebinding 케이스를 실행하고 모든 hop이 Deny 되는지 확인한다. 실제 외부 URL은 사용하지 않는다.
- Confidence: High
- Notes: `test_fetch_url_network_policy`는 AllowAll/ProviderOnly/Deny enum 분기만 PASS했으며 실제 URL, redirect, IP 검증은 다루지 않는다.

### [A03-F003] FetchURL 500KB cap은 read 후 검사이며 truncation 상태도 표시하지 않음

- Pattern: `SEC-003` — 네트워크 입력/응답 크기 경계
- Area: response size, memory, result integrity
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: 응답 chunk를 무조건 `Vec`에 append한 뒤 `len() > 500_000`을 검사한다. chunk 하나가 cap을 초과할 수 있고, `html2md` 전에 완전한 body를 보관한다. 결과를 잘라도 `ToolResult.is_truncated`는 false다.
- Evidence:
  - `src/tools/fetch.rs:83-93` (`body_bytes.extend_from_slice(&chunk)` 후 cap 검사).
  - `src/tools/fetch.rs:95-103`은 500KB input을 HTML→Markdown으로 처리한 뒤 문자열 10,000자를 자른다.
  - `src/tools/fetch.rs:105-114`는 `is_truncated: false`, `original_size_bytes: None`을 반환한다.
  - `spec.md:730-732`, `spec.md:2011`, README의 “memory capping”은 hard cap과 잘림 메타데이터를 기대한다.
- Expected Basis: 네트워크/출력은 bounded memory와 truncation provenance를 가져야 한다.
- Actual: 서버가 큰 HTTP chunk를 보내면 cap 검사 전에 그 chunk 전체가 메모리에 들어간다. partial body를 성공으로 반환하면서 downstream은 결과가 잘렸는지 알 수 없다.
- Attack Preconditions: AllowAll 상태에서 공격자가 FetchURL 응답을 제어하거나 응답을 매우 크게 만들어야 한다.
- Impact: 메모리/CPU 압박, HTML parser 비용 증가, 모델이 불완전한 문서를 완전한 사실로 판단하는 integrity drift.
- Suggested Action: streaming reader에서 남은 허용 바이트만 읽고 초과 시 즉시 body를 폐기/중단한다. `Content-Length`와 chunk 모두 상한을 적용하며, `is_truncated=true`, 원래 관측 크기와 안전한 표시를 반환한다.
- Re-audit Method: mock server가 1KB chunk, 500KB+1 chunk, 단일 수십 MB chunk, chunked no-length 응답을 반환하도록 하고 peak allocation/결과 metadata를 확인한다.
- Confidence: High
- Notes: 외부 네트워크는 실행하지 않았다.

### [A03-F004] 설정 로드 실패 시 ExecShell이 sandbox를 끄고 host shell로 fail-open 됨

- Pattern: `SEC-002`, `SEC-004` — 개발/런타임 우회와 셸 경계
- Area: sandbox configuration, fail-open behavior
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: tool permission은 App state의 settings로 판정하지만 실제 shell 실행은 config 파일을 다시 읽는다. 재로드 실패/파일 부재 시 `sandbox_enabled=false`, `allow_network=true` 기본값으로 host shell을 실행한다.
- Evidence:
  - `src/tools/shell.rs:125-143`은 `sandbox_enabled=false`, `allow_network=true`로 시작한 뒤 `if let Ok(Some(settings))`일 때만 설정을 적용한다.
  - `src/tools/shell.rs:151-160`은 재로드 결과가 없으면 `build_shell_command(..., false)`로 host shell을 선택한다.
  - `src/domain/settings.rs:127-133`의 기본 SandboxConfig도 `enabled=false`, `allow_network=true`다.
  - `src/app/tool_runtime.rs:102-113`은 이미 메모리 settings로 preflight/PermissionEngine을 수행하므로 두 설정 source가 분리된다.
- Expected Basis: `SEC-002`의 release-mode fence와 `spec.md:2353-2412`의 sandbox hard boundary. 사용자가 sandbox를 켠 상태에서는 설정 오류가 실행 완화로 이어지면 안 된다.
- Actual: config가 삭제·권한 오류·일시적 read failure가 되면 trusted shell call은 sandbox 없이 실행된다. 같은 요청의 permission decision과 execution mode가 달라진다.
- Attack Preconditions: 사용자가 sandbox를 켜고, 실행 직전 config 파일이 없거나 읽기 실패하도록 만들 수 있어야 한다(동시 설정 저장/권한 변경/파일 손상 포함).
- Impact: 의도한 kernel-level filesystem/network isolation이 사라져 shell이 host filesystem과 network에 접근한다.
- Suggested Action: 실행 context에 이미 검증된 immutable settings snapshot을 전달하고 재로드하지 않는다. sandbox enabled인데 backend/config 확인이 실패하면 실행을 Deny한다. `allow_network`도 fail-open하지 않는다.
- Re-audit Method: in-memory settings는 sandbox enabled로 두고 실행 직전에 config read를 `NotFound`, `PermissionDenied`, malformed로 만드는 fixture를 실행한다. 모든 케이스가 host shell로 fallback하지 않고 Deny/명시적 오류인지 확인한다.
- Confidence: High

## 6. Uncertainties and Clarifications Needed

- `NetworkPolicy::AllowAll`이 public internet만 의미하는지, loopback/private/link-local까지 의도적으로 허용하는지 명세를 확정해야 한다. 현재 wizard/design은 AllowAll을 기본값으로 두지만 SSRF boundary는 없다.
- `SandboxConfig.extra_binds`가 trusted administrator escape hatch인지 일반 사용자 기능인지 결정해야 한다. 전자라면 config owner/mode와 표시를 강화하고, 후자라면 source/target allowlist가 필수다.
- custom provider가 arbitrary LAN/loopback endpoint를 허용해야 하는지, API key를 redirect destination에 전달하지 않는 정책을 고정해야 한다.
- session log가 평문 local history인지, owner-only 접근이면 충분한지, 암호화/retention/redaction을 요구하는지 결정해야 한다.
- MCP server command가 사용자가 명시적으로 신뢰한 executable인지, startup prompt/allowlist/absolute path pinning이 필요한지 결정해야 한다.
- SafeOnly가 “read-only binary”만 의미하는지, read target도 workspace로 제한하는지 명세에 적어야 한다.
- 현재 테스트 인프라 quota로 session/MCP 추가 필터를 실행하지 못했다. 해당 경로의 runtime evidence는 `Not Covered`로 유지한다.

## 7. Perspective Decision

- Coverage: **Partially Covered**. 코드·문서·설정·CI 정적 증거와 8개 선택 테스트를 확인했으나 실제 user state, network, non-Linux, full suite와 advisory scan은 제외했다.
- Critical findings: A03-F013(rollback data loss), A03-F015(MCP environment secret exposure).
- Major findings: A03-F001–F012, A03-F014, A03-F016–F025 중 다수. 특히 A03-F002/F004/F007/F011/F012/F019/F020/F021/F023/F024는 security/recovery gate를 직접 막는다.
- Verified narrow passes: FetchURL policy enum, workspace outside-cwd preflight, MCP Ask/result parsing, selective-staging 정상 fixture가 PASS했지만, 이는 전체 보안 경계 PASS가 아니다.
- Final decision: **HOLD**. Critical/Major 데이터 손상·secret·network·process 경계가 남아 있고, 문서의 hard-boundary 주장이 실제 enforcement보다 강하다. 수정 후 관련 Pass 3와 변경된 build/test/config 경로를 재감사해야 한다.

### Required re-audit gates before any PASS claim

1. A03-F001/F004/F005/F021/F023: file/shell/harness path, config drift, SafeOnly argument scope와 approval revalidation을 실제 실행 fixture로 검증.
2. A03-F002/F003/F019/F020/F021: mock HTTP/provider tests로 private-IP/redirect/DNS/size/error-redaction/stream cap 검증.
3. A03-F008–F012: isolated HOME fixture에서 mode/owner, malformed nonce, concurrent config/index recovery, session size/path containment 검증.
4. A03-F013/F014: concurrent WIP/pre-staged index/HEAD mutation/rollback fixture로 user data preservation을 hash/diff 검증.
5. A03-F015–F018: hostile MCP fixture로 env allowlist, schema validation, output cap, cancel/shutdown/descendant cleanup 검증.
6. A03-F007: real parent/start-time/PGID 기반 reaper fixture로 unrelated process non-kill을 검증.
7. A03-F024/F025: 문서 claim을 enforcement 증거와 동기화하고 CI/release에서 locked audit/security/artifact gates를 실제 실행.

### Coder Handoff

`/mnt/Projects_SSD/rust/smlcli/docs/multi_audit/1/sub_audit_03_security_resilience.md`를 먼저 읽고, 각 finding을 프로젝트 문서와 실제 코드에 대조하여 우선순위대로 수정하세요. 계약 변경이 필요하면 관련 문서를 먼저 갱신하고, 수정 후 지정 테스트·보안 fixture·CI/release 검증과 A03 재감사 증거를 기록하세요.
