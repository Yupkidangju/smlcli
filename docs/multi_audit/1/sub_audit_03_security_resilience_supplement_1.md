# Sub Audit Supplement

## 1. Audit Metadata

- Audit Turn: 1
- Supplement: 1
- Perspective: A03 — 보안·신뢰 경계·오류 복원력
- Audit Basis: Standard-backed
- Standard Path: `/mnt/Projects_SSD/rust/smlcli/AI_AUDIT_DOC_STANDARD.md`
- Report Contract: `/home/eunho1/.codex/skills/multi-audit/references/report-contract.md`
- Supplement Of: `/mnt/Projects_SSD/rust/smlcli/docs/multi_audit/1/sub_audit_03_security_resilience.md`
- Immutability: 원본 보고서는 `0444` 상태로 확인했으며 수정·덮어쓰기·권한 변경을 하지 않았다.
- Scope rule: 동료 보고서는 읽지 않았고, 이 supplement 파일만 생성했다.

## 2. Coverage Gap Questions

이번 보완은 다음 네 질문만 독립적으로 확인했다.

1. `src/app/chat_runtime.rs`의 `@file` inline 경로가 workspace/sandbox/size/binary 경계를 우회하는가.
2. `write_file_commit`의 `<canonical>.tmp`가 pre-existing symlink일 때 외부 target write 또는 target symlink 치환이 가능한가.
3. custom provider 선택 후 credential resolution 및 registry reload/missing-adapter fallback이 잘못된 endpoint로 key/request를 보낼 수 있는가.
4. doctor network probe가 `NetworkPolicy::Deny`를 존중하는가.

## 3. Evidence Examined

- `src/app/chat_runtime.rs:249-325`, `src/app/mod.rs:336-343`, `:2739-2777`, `:2865-2905`
- `src/tools/file_ops.rs:10-53`, `:174-225`, `:283-320`, `:347-407`, `:443-499`
- `src/app/wizard_controller.rs:408-529`, `src/app/chat_runtime.rs:131-176`, `:178-247`
- `src/providers/registry.rs:803-870`, `src/app/state.rs:139-163`
- `src/infra/doctor.rs:19-49`, `:72-133`, `src/main.rs:71-105`, `src/app/mod.rs:108-122`
- Expected contracts: `spec.md:807-815`, `spec.md:465-468`, `spec.md:1938-1948`, `spec.md:2326-2349`, `designs.md:736-743`, `designs.md:524-528`
- Test inventory command:
  - `rg -n "dispatch_chat_request|write_file_commit|check_api|reload_providers|missing.*adapter|fallback" src/tests/audit_regression.rs src/tests/*.rs || true`
  - Result: only comments around `dispatch_chat_request` were found; no targeted regression test for the four queried boundaries.
- No actual user config/secret/session, external URL, provider API, or source test was modified/executed.

## 4. Findings

### [A03-F026] `@file` inline injection directly reads arbitrary paths and sends unbounded content to the provider

- Related Original Finding: A03-F001 (workspace path boundary), A03-F020 (provider/session secret boundary)
- Area: `@file` context injection, workspace trust, provider egress
- Pattern: `SEC-004`, `SEC-001`
- Severity: Critical
- Status: Confirmed
- Standard Disposition: Needs Fix / Hold
- Summary: `dispatch_chat_request()` handles any token beginning with `@` by calling `tokio::fs::read_to_string(path)` directly. It does not call `validate_sandbox()`, `HarnessPreflight`, a size check, binary detector, or NetworkPolicy-aware redaction before inserting content into the user message that is later sent to the provider.
- Evidence:
  - `src/app/chat_runtime.rs:270-276` splits all whitespace tokens and treats `@/absolute/path`, `@../path`, and `@symlink` as file paths.
  - `src/app/chat_runtime.rs:292-315` calls `tokio::fs::read_to_string(path)` directly and replaces the token with the full file content.
  - The resulting text is sent as `Action::SubmitChatRequest` at `src/app/chat_runtime.rs:320-324`, then `submit_chat_request()` stores it and proceeds to provider resolution (`:328-358`, `:468-494`).
  - `src/app/mod.rs:336-343` only normalizes the process cwd to a detected root; it does not make a path capability or enforce a workspace boundary.
  - The UI fuzzy list is workspace-oriented (`src/app/mod.rs:2880-2905`), but manual Composer text bypasses that list; selection merely appends `@{selected}` (`:2754-2760`).
  - `spec.md:807-815` defines `@` as workspace file search, caps matches at 100, and requires unreadable/binary errors to become notices. It does not authorize arbitrary absolute paths.
- Expected Basis: `spec.md:807-815`, `designs.md:736-743`, and `SEC-004` require the file mention surface to stay within the workspace and honor file-size/binary/error boundaries. Provider egress must not receive a file merely because a token was typed.
- Actual: A caller can submit `@/etc/passwd`, `@../sibling/secret`, or a workspace symlink path and the process reads it outside the workspace. There is no byte/line cap; a large valid UTF-8 file is copied into memory, the session, request body, and provider context. `read_to_string` rejects invalid UTF-8 but is not a binary detector, so binary data that is valid UTF-8 (including NUL-containing data) can pass.
- Attack Preconditions: Any caller able to place an `@path` token in Composer input (including pasted or prompt-injected user text) and a configured provider/network path. For external disclosure, provider resolution must be allowed; under `NetworkPolicy::Deny`, the local read and session insertion still occur before the provider guard.
- Impact: Arbitrary local file disclosure to the configured provider, session-log persistence of the file, unbounded memory/context growth, and workspace trust bypass. This is a direct exfiltration path, not a symlink race.
- Suggested Action: Resolve mentions through one workspace-scoped helper that canonicalizes and validates the path immediately before reading, rejects absolute/parent/symlink escapes, applies a byte cap, binary heuristic, and per-turn aggregate cap, and returns a notice on denial. Keep `@workspace`/`@terminal` special cases separate. Do not append or send content after a denied/read-error path.
- Re-audit Method: Add isolated temp-workspace fixtures for `@inside`, `@/outside`, `@../outside`, symlink-to-outside, >1MB UTF-8, NUL-containing UTF-8, invalid UTF-8, and `NetworkPolicy::Deny`. Assert no external read/content insertion/provider request and that only bounded workspace content reaches the request.
- Confidence: High
- Notes: No targeted `@file` regression test was found by the inventory command above. Existing workspace/file tool tests do not cover this separate natural-language preprocessing path.

### [A03-F027] pre-existing `<canonical>.tmp` symlink is followed and can write outside the workspace

- Related Original Finding: A03-F001 (TOCTOU/symlink), A03-F013 (data integrity)
- Area: WriteFile atomic temp path, symlink handling
- Pattern: `SEC-004`
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: `write_file_commit()` computes a deterministic sibling temp path and uses `std::fs::write()` on it. If `<canonical>.tmp` already exists as a symlink to an external file, the open/truncate operation follows that symlink. The subsequent rename moves the symlink itself into the canonical target path.
- Evidence:
  - `src/tools/file_ops.rs:179-195` validates only the final requested path and obtains `canonical`.
  - `src/tools/file_ops.rs:197-200` derives `tmp_path = format!("{}.tmp", path_str)` and calls `fs::write(&tmp_path, new_content)` with no `create_new`, `O_NOFOLLOW`, lstat, or temp-file identity check.
  - `src/tools/file_ops.rs:200-225` then calls `fs::rename(&tmp_path, &canonical)`. Rename operates on the symlink directory entry; it does not retroactively prove that the earlier write targeted the workspace.
  - `WriteFileTool::check_permission()` validates the requested path only (`src/tools/file_ops.rs:347-365`); no `.tmp` sibling is checked.
  - The existing `test_write_file_sandbox_blocks_absolute_path_outside_workspace` and `test_all_write_tools_deny_outside_workspace_paths` cover requested target paths, not a pre-existing canonical temp symlink.
- Expected Basis: `spec.md:1938-1948` and the claimed atomic write/symlink boundary require both the temporary write and final replacement to stay inside the workspace and to fail closed on symlinks.
- Actual: With `/workspace/inside.txt.tmp -> /outside/target`, `fs::write` follows the link and truncates/writes `/outside/target`. `fs::rename` then moves the link to `/workspace/inside.txt`, leaving a workspace symlink to the external target. The next validation may detect that link, but the external write has already occurred.
- Attack Preconditions: An attacker or untrusted repository process can pre-create the sibling `.tmp` symlink before an approved `WriteFile`/`ReplaceFileContent` call. No race is needed after the symlink is planted.
- Impact: Arbitrary external file overwrite/truncation and workspace target symlink replacement. A malicious repository can redirect a legitimate AI write into a user file outside the workspace.
- Suggested Action: Create a unique temp file with `O_CREAT|O_EXCL|O_NOFOLLOW` in the validated parent directory, verify its inode/owner, write+fsync it, and rename only that open-created entry. Refuse any pre-existing temp entry and revalidate the parent/target with dirfd/openat semantics.
- Re-audit Method: In an isolated temp workspace, pre-create `<canonical>.tmp` symlinks to a sentinel outside file and run WriteFile/ReplaceFileContent. Assert sentinel bytes/mode and final target type remain unchanged; repeat with a concurrent temp replacement fixture.
- Confidence: High
- Notes: This supplement treats the operation as a direct pre-existing symlink case; it is stronger evidence than the race-only concern in A03-F001.

### [A03-F028] custom provider selection can fail open to OpenRouter/OpenAI/Google endpoints after reload or dialect mapping

- Area: custom provider credential resolution, registry reload, missing adapter fallback
- Pattern: `SEC-001`, `SEC-003`
- Severity: Critical
- Status: Confirmed
- Standard Disposition: Needs Fix / Hold
- Summary: Custom provider routing has three independent fail-open paths: `resolve_credentials()` does not recognize a persisted `Custom: id` default and maps it to OpenRouter; provider-list model selection calls `reload_providers()` which clears custom adapters without re-registering them; and `get_adapter(Custom(id))` silently falls back to the OpenAI adapter. A custom Gemini dialect also ignores its configured base URL and constructs the fixed Google adapter.
- Evidence:
  - `src/app/chat_runtime.rs:139-146` matches only built-in provider names; persisted `default_provider == "Custom: id"` falls through to `ProviderKind::OpenRouter`.
  - `src/app/chat_runtime.rs:153-160` derives the key alias from the raw string (`custom: id_key`), not the custom provider id alias used by `resolve_credentials_for_provider()` (`src/app/chat_runtime.rs:196-207`). If such an alias exists, the wrong OpenRouter adapter receives it; otherwise the path fails with an authentication error rather than using the configured custom endpoint.
  - `src/app/wizard_controller.rs:408-437` selects `Custom: id` and initially resolves the custom key; after model selection, `:511-524` saves settings and invokes `ProviderRegistry::reload_providers()`.
  - `src/providers/registry.rs:781-800` constructs a registry with an empty `custom_adapters` map; `:850-868` maps a missing `ProviderKind::Custom(id)` adapter to `self.openai.clone()` instead of returning an error.
  - `src/providers/registry.rs:822-831` maps a custom `ToolDialect::Gemini` to `GeminiAdapter::new()` and discards `config.base_url`/auth strategy, so the request goes to the fixed Google endpoint.
  - `src/app/state.rs:158-162` registers custom providers at initial load, but no equivalent re-registration follows the provider-selection `reload_providers()` path.
  - `src/providers/registry.rs:844-848` makes all `get_adapter()` calls return `MockProvider` under tests, masking this release-only fallback. The test inventory found no custom reload/fallback regression test.
- Expected Basis: `spec.md:2294-2349` requires custom `base_url + api_key` routing and graceful invalid-base-url failure. A missing adapter or unsupported dialect must fail closed, never select a different vendor endpoint.
- Actual: After a normal custom selection and model save, custom adapters can be absent. Subsequent chat may fail as OpenRouter, or if a matching malformed alias exists, send the custom secret to OpenRouter. A direct Custom adapter lookup sends the same secret to OpenAI when missing. Custom Gemini configurations send requests to Google rather than their configured base URL.
- Attack Preconditions: User adds/selects a custom provider, completes model selection (triggering reload), and has a key or no-auth configuration that reaches provider request code. A malicious/invalid config can also create a missing adapter or Gemini dialect.
- Impact: API key disclosure to an unintended third-party vendor, request/session data sent to the wrong endpoint, policy/audit provenance mismatch, and inability to trust custom provider isolation. This is a Critical secret-egress boundary failure.
- Suggested Action: Parse and route `Custom: id` in `resolve_credentials()` through a single provider-config lookup. Make `reload_providers()` accept/re-register the current custom configs atomically. Change missing adapter fallback to a typed error/Deny. Reject dialects whose adapter cannot honor `base_url`/auth instead of mapping to a fixed vendor. Add endpoint and auth provenance to the request context without logging keys.
- Re-audit Method: With a mock custom adapter and endpoint recorder, select a custom provider, save a model, reload, and send chat/model requests. Assert every request uses the configured custom URL/auth. Delete the adapter, use unknown id, and choose custom Gemini; each case must fail closed with no request to OpenAI/OpenRouter/Google.
- Confidence: High

### [A03-F029] doctor network probe ignores `NetworkPolicy::Deny` and contacts OpenRouter

- Area: doctor/silent health check network egress
- Pattern: `SEC-003`
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: `DoctorReport::check_api()` performs an unconditional HTTPS request to OpenRouter whenever config is valid and any encrypted key exists. It never reads or checks `settings.network_policy`, so `NetworkPolicy::Deny` does not prevent doctor or the startup silent health check from making network egress.
- Evidence:
  - `src/infra/doctor.rs:28-39` runs `check_api()` before collecting the settings snapshot; no policy argument is passed.
  - `src/infra/doctor.rs:88-105` loads settings only to inspect `encrypted_keys` and then calls `client.get("https://openrouter.ai/api/v1/auth/key").send()`; there is no `NetworkPolicy` branch.
  - `src/infra/doctor.rs:107-125` reports network reachability based on that request even when the configured provider is custom/local.
  - `src/main.rs:58-66`, `:93-105` runs doctor at startup or via the explicit doctor command; `src/app/mod.rs:108-122` also spawns `DoctorReport::run_diagnostics()` in normal app startup.
  - `src/domain/permissions.rs:19-22`, `spec.md:465-469`, and `src/app/chat_runtime.rs:131-137` define `NetworkPolicy::Deny` as blocking all network communication, but doctor has no corresponding guard.
  - Existing `test_network_policy_deny_blocks_chat` only exercises chat/settings behavior (`src/tests/audit_regression.rs:57-81`); the test inventory found no doctor policy test.
- Expected Basis: `NetworkPolicy::Deny` is a global egress policy. Diagnostics must either be offline or explicitly honor the same policy.
- Actual: With a valid config containing encrypted keys and `Deny`, startup and `smlcli doctor` still attempt a request to OpenRouter. No API key is attached to this probe, but the network contact, DNS lookup, TLS handshake, and user-IP disclosure still occur.
- Attack Preconditions: Valid config with at least one encrypted key and an execution of normal app startup or doctor. Network availability is not required for the policy violation; the request is attempted even if it later fails.
- Impact: Offline/air-gapped policy violation, unexpected network metadata egress, misleading diagnostic status, and a bypass of the single NetworkPolicy choke point.
- Suggested Action: Pass the loaded settings/policy into `check_api()` and return an explicit `Skipped (NetworkPolicy::Deny)` before constructing a client. Under ProviderOnly, probe only an explicitly configured provider or skip network entirely. Remove duplicate config reloads and ensure silent health checks use the same policy snapshot.
- Re-audit Method: Mock the HTTP client or inject a request recorder with `Deny`, `ProviderOnly`, `AllowAll`, no-key, custom-provider, and valid-key settings. Assert zero request/DNS attempt for Deny and documented behavior for the other modes; verify startup silent health check follows the same result.
- Confidence: High

## 5. Coverage Decision

| Question | Decision | Gate impact |
| --- | --- | --- |
| `@file` workspace/size/binary boundary | Confirmed | Critical; blocks PASS |
| pre-existing `.tmp` symlink | Confirmed | Major; blocks PASS until atomic temp path is fixed |
| custom provider resolve/reload/fallback | Confirmed | Critical; blocks PASS |
| doctor + `NetworkPolicy::Deny` | Confirmed | Major; blocks PASS |

All four coverage gaps are confirmed findings. No item is `Rejected` or merely `Needs Clarification`; the product policy around custom/private endpoints may still need clarification, but the observed fail-open behavior is independently confirmed.

## 6. Supplement Re-audit Requirements

1. Add isolated, no-network regression fixtures for all four findings before changing the original report status.
2. Re-run the provider path in a non-test/release configuration; `MockProvider` must not mask missing-adapter behavior.
3. Re-run the complete related Pass 3 surfaces after fixes: workspace/file boundary, provider egress/redaction, config/session data integrity, and doctor startup lifecycle.
4. Preserve the original report as immutable and seal this supplement separately before integration.

### Coder Handoff

`/mnt/Projects_SSD/rust/smlcli/docs/multi_audit/1/sub_audit_03_security_resilience_supplement_1.md`를 먼저 읽고, 네 가지 Confirmed finding을 현재 코드·통제 문서와 대조하여 수정하세요. 원본 A03 보고서는 수정하지 말고, 수정 후 격리 fixture·release provider path·doctor policy 검증과 supplement 재감사 증거를 기록하세요.
