# D3D Re-Audit Report (audit_report_10.md)

- **감사 대상:** smlcli 3.9.0
- **원 감사:** `docs/multi_audit/1/final_audit_report_1.md`
- **원 finding:** FIN-F001~FIN-F025 (Critical 5, Major 19, Minor 1)
- **감사 기준:** `AI_AUDIT_DOC_STANDARD.md`
- **재감사 일시:** 2026-08-23 (Asia/Seoul)
- **Git 기준:** `main` / `55d1c33` + 미커밋 remediation worktree
- **도구체인:** rustc/cargo 1.94.1, Linux 7.0.0-29-generic x86_64
- **최종 판정:** **HOLD**

> 코드·문서 remediation과 호스트 품질 게이트는 완료되었다. 25개 원 finding 중 24개는 `Verified`이며, FIN-F024는 musl/MSVC hosted runner와 실제 provenance publication이 아직 실행되지 않아 `Hold`다. 이 환경 공백을 PASS 또는 Accepted Risk로 환산하지 않는다.

## 1. Audit Scope

이번 재감사는 원 보고서의 각 finding을 현재 `spec.md` §1.1, ADR-041, 실제 production call path, 회귀 테스트, 빌드·CI·패키지 산출물에 다시 대조했다.

포함 범위:

- Git/file/path/MCP/provider/network/shell/secret/config/session 보안 경계
- tool turn, approval queue, streaming, RepoMap, EventLoop, Workspace Harness
- TUI command/focus/layout/Unicode/overlay/i18n 및 공개 CLI 계약
- identity/version/ADR authority, hermetic test suite, dependency policy
- CI/release target, action provenance, SBOM/checksum/attestation, package scope
- 원 감사 baseline과 remediation 후 명령 결과 비교

최종 read-only pass는 마지막 README 계약 보정 후 수행했다. Git commit, push, tag, release 또는 운영 데이터 변경은 수행하지 않았다.

## 2. Excluded Scope

- 실제 유료/외부 LLM credential 전송과 public DNS/redirect 실서비스 호출
- GitHub-hosted `ubuntu-24.04` 및 `windows-2025` workflow 실행
- release tag/upload와 공개 attestation 조회
- 물리 TTY 및 여러 terminal emulator의 수동 시각 검증

외부 provider는 deterministic transport/parser/recorder-equivalent 테스트로 대체했다. 반면 canonical release target은 실제 runner 결과가 지원 계약의 일부이므로 FIN-F024에 대해 제외 범위를 PASS로 간주하지 않았다.

## 3. Evidence Summary

| Evidence | Baseline | Re-audit result |
| --- | --- | --- |
| `cargo fmt --all -- --check` | PASS | PASS |
| `cargo check --all-targets --locked` | PASS | PASS |
| `cargo clippy --all-targets --all-features --locked -- -D warnings` | PASS | PASS, warnings 0 |
| `cargo test --all-targets --locked --no-fail-fast` | 119 passed / 2 failed | **183 passed / 0 failed** |
| `cargo build --release --locked` | glibc host PASS | glibc host PASS |
| `bash scripts/check-version-sync.sh` | narrow PASS | PASS, Cargo/CHANGELOG/spec/AGENTS 3.9.0 |
| `python3 scripts/check-workflows.py` | 없음 | PASS |
| `cargo audit --no-fetch` | FAIL, vulnerabilities 2 + warnings 3 | PASS, 428 lock dependencies / vulnerability 0 |
| `cargo deny check --disable-fetch` | Not Covered | PASS, advisories/bans/licenses/sources |
| `cargo package --allow-dirty --locked --offline` | Not Covered | PASS, 79 files / 1.6 MiB |
| SPDX 2.3 + SHA-256 verification | Not Covered | PASS |
| local musl build | Not Covered | target 설치 후 `x86_64-linux-musl-gcc` 부재로 환경 차단 |
| local MSVC check | Not Covered | target 설치 후 Windows SDK `lib.exe` 부재로 환경 차단 |

`cargo deny`의 transitive duplicate 경고는 `getrandom`, `hashbrown`, `windows-sys` 세 계열이며 advisory/license/source 위반은 아니다. 후속 dependency convergence 후보로만 남긴다.

## 4. Pass 1: Implementation Compliance Re-audit

### [FIN-F010] Re-audit #1 — WriteFile/ReplaceFileContent 계약

- **Original Severity / Status:** Major / Confirmed + Needs Spec Clarification
- **Status:** **Verified**
- **Changed authority/files:** `spec.md` §1.1, `src/tools/file_ops.rs`
- **Evidence:** `overwrite`가 required이며 `false`는 exclusive create-only다. Replace는 non-empty target이 정확히 1회 존재할 때만 preview와 execute가 같은 helper를 사용한다. 기존 mode와 atomic publish를 보존한다.
- **Tests:** `fin_f010_write_file_false_is_create_only`, `fin_f010_write_file_schema_requires_overwrite`, `fin_f010_replace_rejects_empty_or_ambiguous_target_without_mutation`, `fin_f010_atomic_replace_preserves_existing_mode`.
- **Residual risk:** 없음. 이전 명세 공백은 보수적 계약으로 닫혔다.

### [FIN-F017] Re-audit #1 — TUI command/focus/Inspector 계약

- **Original Severity / Status:** Major / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/commands/mod.rs`, `src/app/command_router.rs`, `src/app/mod.rs`, `src/app/state.rs`, `src/tui/help_overlay.rs`
- **Evidence:** Slash/Palette/Help가 공통 registry를 소비하고 `/status`와 `/clear`가 보이는 timeline을 변경한다. Ctrl+Left/Right pane focus, Alt+1~6 및 mouse Inspector tab 경로가 연결된다.
- **Tests:** `fin_f017_command_registry_drives_slash_palette_and_help_surfaces`, `fin_f017_status_and_clear_mutate_visible_timeline`, `fin_f017_key_only_focus_and_inspector_shortcuts_are_reachable`.
- **Residual risk:** 물리 terminal별 Alt key 전달 차이는 manual TTY 범위다.

### [FIN-F019] Re-audit #1 — 5-locale i18n 계약

- **Original Severity / Status:** Major / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/tui/i18n.rs`, config/wizard/help/empty-state widgets, `README.md`
- **Evidence:** `LANG` suffix/encoding을 normalization하고 명시 설정→환경→English 순서를 사용한다. 공통 catalog를 ko/en/ja/zh-TW/zh-CN이 모두 소유하고 주요 화면이 catalog를 소비한다.
- **Tests:** `fin_f019_locale_normalization_supports_real_lang_formats`, `fin_f019_all_supported_locales_own_common_surface_keys`, `fin_f019_wizard_and_config_render_each_locale_catalog`.
- **Residual risk:** 번역의 언어학적 품질은 별도 native-speaker review 대상이다.

### [FIN-F020] Re-audit #1 — 공개 CLI/provider/module 계약

- **Original Severity / Status:** Major / Confirmed + Needs Spec Clarification
- **Status:** **Verified**
- **Changed authority/files:** `spec.md`, `README.md`, `designs.md`, `src/main.rs`, `src/commands/mod.rs`; 빈 `src/types/mod.rs` 제거
- **Evidence:** `smlcli run`은 prompt argument 없는 interactive TUI다. built-in provider 목록과 Custom-only Ollama 방침, config 형식, module owner가 runtime과 일치한다.
- **Tests:** `fin_f020_cli_run_contract_is_interactive_without_prompt_argument`, `fin_f020_provider_and_command_public_contract_matches_runtime`.
- **Residual risk:** 없음.

### [FIN-F021] Re-audit #1 — identity/version/ADR/security authority

- **Original Severity / Status:** Major / Needs Spec Clarification + Confirmed drift
- **Status:** **Verified**
- **Changed authority/files:** `AGENTS.md`, `spec.md` §1.1, `DESIGN_DECISIONS.md` Authority Index/ADR-041, `README.md`, `scripts/check-version-sync.sh`
- **Evidence:** canonical identity는 smlcli 3.9.0, Phase 53/54는 Unreleased다. ADR ID가 유일하며 과거 alias가 색인화되었다. README 5개 언어는 redaction/sandbox/platform을 보장 문구가 아닌 실제 경계와 runner 조건으로 설명한다.
- **Tests/commands:** `fin_f021_identity_version_and_adr_authority_are_unique`; version-sync PASS; README overclaim scan 재확인.
- **Residual risk:** 없음.

### [FIN-F025] Re-audit #1 — repository/package scope

- **Original Severity / Status:** Minor / Confirmed
- **Status:** **Verified**
- **Changed files:** `Cargo.toml`, `.gitignore`, `stitch_modern_tui_redesign/README.md`; local residue 및 scratch 도구 제거
- **Evidence:** package include가 runtime source, 필수 MCP fixture, license와 사용자/build 문서로 제한된다. `.agents` support와 Stitch reference corpus는 runtime/package가 아님을 명시했다.
- **Tests/commands:** `fin_f025_package_scope_excludes_local_and_reference_residue`; `cargo package --list --allow-dirty --locked`; offline package/verify PASS.
- **Residual risk:** 제거된 local residue는 Git 및 `/tmp/smlcli-audit-residue-backup-20260823/`에서 복구 가능하다.

## 5. Pass 2: Debug / Engineering Quality Re-audit

### [FIN-F001] Re-audit #1 — Git WIP/index 무결성

- **Original Severity / Status:** Critical / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/tools/executor.rs`, `src/tools/git_checkpoint.rs`, `src/infra/git_engine.rs`
- **Evidence:** application execution path는 automatic hard reset과 auto-commit을 수행하지 않는다. test-only explicit commit은 `git commit --only -- <paths>`를 사용해 기존 staged index를 보존하고 rollback helper는 fail-closed다.
- **Tests:** `fin_f001_auto_commit_preserves_pre_staged_user_index`, `fin_f001_hard_reset_rollback_is_disabled_and_preserves_wip` 및 temp Git repo E2E.
- **Residual risk:** 사용자가 직접 실행한 외부 Git 명령은 제품 rollback 계약 밖이다.

### [FIN-F006] Re-audit #1 — MCP bounded/cancellable lifecycle

- **Original Severity / Status:** Major / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/infra/mcp_client.rs`, `src/app/mod.rs`, `src/app/tool_runtime.rs`, `scripts/mock_mcp_server.py`
- **Evidence:** client가 child/tasks/pending/schema를 소유한다. request/result/line/tool/schema cap, call-time required/type 검증, cancellation cleanup, EOF propagation, process-group shutdown을 적용했다.
- **Tests:** `fin_f006_mcp_call_is_cancellable_and_clears_pending_request`, `fin_f006_mcp_oversized_response_is_bounded`, `fin_f006_mcp_schema_requires_object_shape_and_bounded_size`, `fin_f006_mcp_shutdown_terminates_descendant_process_group`.
- **Residual risk:** hostile non-Unix executable의 descendant semantics는 Windows runner smoke가 필요하다.

### [FIN-F009] Re-audit #1 — approval/write terminal transition

- **Original Severity / Status:** Major / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/app/state.rs`, `src/app/mod.rs`, `src/app/tool_runtime.rs`
- **Evidence:** execution key별 exactly-once terminal helper가 success/error/cancel/deny에서 token, write owner, pending count, next queue를 정리한다. 승인 직전에 현재 preflight와 PermissionEngine을 다시 실행하고 다음 approval을 승격한다.
- **Tests:** `fin_f009_terminal_transition_is_exactly_once_and_releases_write_queue`, `fin_f009_approval_revalidates_policy_and_promotes_next_request`, `fin_f009_approval_keyboard_contract_accepts_enter_and_rejects_escape`.
- **Residual risk:** 없음.

### [FIN-F011] Re-audit #1 — secret/config/wizard persistence

- **Original Severity / Status:** Major / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/infra/secure_fs.rs`, `config_store.rs`, `secret_store.rs`, `src/app/wizard_controller.rs`, `src/app/mod.rs`
- **Evidence:** directory/file 0700/0600, no-follow, exclusive create, unique atomic write와 revisioned writer를 사용한다. nonce/cipher 크기를 복호화 전에 검사하고 wizard는 existing settings를 patch한 뒤 disk success에서만 memory를 commit한다.
- **Tests:** `fin_f011_malformed_encrypted_nonce_returns_error_without_panic`, `fin_f011_config_store_is_private_atomic_and_no_follow`, `fin_f011_master_key_is_private_and_exclusive`, `fin_f011_wizard_patch_preserves_unowned_settings_fields`.
- **Residual risk:** POSIX mode 검사는 Unix에서 강제하며 Windows ACL parity는 runner 검증 대상이다.

### [FIN-F012] Re-audit #1 — session privacy/corruption recovery

- **Original Severity / Status:** Major / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/infra/secure_fs.rs`, `src/infra/session_log.rs`, session command paths
- **Evidence:** injectable private root, contained UUID basename, bounded 32 MiB file/1 MiB line restore, locked backup+atomic index를 사용하며 corrupt index를 empty로 덮지 않는다.
- **Tests:** `fin_f012_session_store_is_private_and_contained`, `fin_f012_restore_rejects_oversized_line_without_unbounded_read`, `fin_f012_corrupt_index_is_not_silently_overwritten`, `fin_f012_concurrent_index_writers_preserve_all_entries`.
- **Residual risk:** 없음.

### [FIN-F013] Re-audit #1 — tool turn/compaction transaction

- **Original Severity / Status:** Major / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/domain/session.rs`, `src/app/tool_runtime.rs`, `src/app/mod.rs`, `src/app/command_router.rs`
- **Evidence:** compaction은 immutable pending snapshot을 두고 non-empty success와 unchanged revision에서만 commit한다. malformed/valid tool call을 같은 turn count에 포함하고 unique execution key와 single follow-up flag를 사용한다.
- **Tests:** `fin_f013_compaction_failure_or_drift_preserves_original_messages`, `fin_f013_compaction_commits_only_after_nonempty_success`, `fin_f013_malformed_and_valid_tool_calls_are_counted_in_one_turn`.
- **Residual risk:** 없음.

### [FIN-F014] Re-audit #1 — streaming/redaction/truncation

- **Original Severity / Status:** Major / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/providers/streaming.rs`, provider adapters, `src/infra/redaction.rs`, `src/tools/shell.rs`, `grep.rs`, `fetch.rs`
- **Evidence:** incremental SSE framing과 bounded sync/error body를 사용한다. 중앙 redactor와 tail-holding streaming masker가 chunk 경계 secret을 emit하지 않으며 Shell/Grep/Fetch cap metadata가 실제 결과와 일치한다.
- **Tests:** `fin_f014_sse_decoder_handles_chunk_boundaries_incrementally`, `fin_f014_central_redaction_masks_exact_header_query_and_json_secrets`, `fin_f014_streaming_masker_never_emits_split_secret_prefix`, `fin_f014_exec_shell_tool_path_emits_live_output_events`, `fin_f014_grep_truncation_metadata_matches_actual_limit`.
- **Residual risk:** redaction은 알려진 credential 값/형식에 대한 방어층이며 임의 비밀의 자동 발견을 보장하지 않는다.

### [FIN-F015] Re-audit #1 — RepoMap/EventLoop/cache ordering

- **Original Severity / Status:** Major / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/domain/repo_map.rs`, `src/app/mod.rs`, `src/app/event_loop.rs`, `src/tui/widgets/inspector_tabs.rs`
- **Evidence:** 첫 request의 동기 scan을 제거하고 revisioned worker가 stale result를 폐기한다. EventLoop가 stop flag와 producer handles를 소유하며 shutdown한다. diff cache key에 text/palette/width가 포함된다.
- **Tests:** `fin_f015_repo_map_discards_stale_worker_revision`, `fin_f015_event_loop_shutdown_closes_all_producers` 및 기존 cache invalidation tests.
- **Residual risk:** 초대형 실제 repository profiler는 성능 벤치마크 범위로 남는다.

### [FIN-F016] Re-audit #1 — Workspace Harness/onboarding/doctor

- **Original Severity / Status:** Major / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/infra/workspace_harness.rs`, `doctor.rs`, `src/app/state.rs`, `src/app/mod.rs`, wizard flow
- **Evidence:** session baseline과 5-field drift를 검사하고 fallback 없이 harness record를 생성한다. wizard 성공은 safe preset으로 trust gate를 다시 열며 Doctor Deny는 offline이다.
- **Tests:** `fin_f016_harness_baseline_detects_all_enforced_drift_fields`, `fin_f016_doctor_network_probe_respects_network_policy`, `fin_f016_wizard_success_reopens_trust_gate_with_safe_preset`.
- **Residual risk:** 실제 provider health endpoint availability는 외부 서비스 상태다.

### [FIN-F018] Re-audit #1 — responsive/Unicode/overlay/terminal cleanup

- **Original Severity / Status:** Major / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/tui/layout.rs`, `src/app/mod.rs`, `src/tui/terminal.rs`
- **Evidence:** render와 hit-test가 공통 `LayoutGeometry`를 사용하고 truncation은 display width/char boundary 안전하다. overlay render/input priority가 일치하며 `TerminalGuard`는 단계별 init rollback과 cursor Show를 수행한다.
- **Tests:** `fin_f018_unicode_truncation_is_width_bounded_and_boundary_safe`, `fin_f018_layout_geometry_and_hit_test_share_all_breakpoints`, `fin_f018_compact_inspector_mouse_tabs_map_to_six_actions`, `fin_f018_overlay_render_and_input_priority_match`.
- **Residual risk:** 물리 TTY의 signal/emulator별 복구는 수동 확인 범위다.

### [FIN-F022] Re-audit #1 — hermetic production-path tests

- **Original Severity / Status:** Major / Confirmed
- **Status:** **Verified**
- **Changed files:** injectable config/session roots, production helpers, `src/tests/multi_audit_remediation.rs`, MCP fixture
- **Evidence:** real home을 사용하지 않고 sandbox capability는 실제 probe 결과를 기준으로 fail-closed 검증한다. 테스트가 duplicated branch가 아니라 production helpers/handlers를 호출한다.
- **Command:** `cargo test --all-targets --locked --no-fail-fast` = 183 passed / 0 failed, 반복 실행 동일 결과.
- **Residual risk:** userns-enabled와 restricted 두 Linux runner의 integration matrix는 CI 확장 후보지만 host-dependent 실패를 PASS로 위장하지 않는다.

### [FIN-F023] Re-audit #1 — dependency/locked security gates

- **Original Severity / Status:** Major / Confirmed
- **Status:** **Verified**
- **Changed files:** `Cargo.toml`, `Cargo.lock`, `deny.toml`, CI/release workflows, `scripts/check-workflows.py`
- **Evidence:** affected `crossbeam-epoch 0.9.20`, `quinn-proto 0.11.15`, `anyhow 1.0.103`으로 갱신했다. 불필요한 shadow-rs/git2와 GPL runtime dependency를 제거했다. 모든 cargo gate는 `--locked`이고 audit/deny/version sync가 CI와 release에 있다.
- **Tests/commands:** `fin_f023_lockfile_contains_patched_advisory_versions`; `cargo audit --no-fetch` PASS; `cargo deny check --disable-fetch` PASS.
- **Residual risk:** advisory DB freshness는 workflow 실행 시 fetch되는 시점에 의존한다.

### [FIN-F024] Re-audit #1 — cross-target/provenance release gate

- **Original Severity / Status:** Major / Confirmed control gap; runtime Unverified
- **Status:** **Hold**
- **Changed files:** `build.sh`, `BUILD_GUIDE.md`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, release scripts
- **Verified control evidence:** canonical Linux musl/Windows MSVC, Rust 1.94.1, full-SHA official actions, least job permissions, locked gates, target smoke, portable checksum, SPDX 2.3, Sigstore attestation 및 rollback 절차가 코드/문서에 일치한다. `python3 scripts/check-workflows.py`, local SPDX/checksum pipeline은 PASS다.
- **Unverified execution evidence:** Rust targets는 설치했으나 local musl은 `x86_64-linux-musl-gcc`가 없고 `sudo apt-get install musl-tools`는 terminal authentication에서 차단되었다. local MSVC는 Windows SDK/Visual Studio `lib.exe`가 없어 실패했다. GitHub-hosted workflow는 미커밋 tree에서 실행할 수 없다.
- **Required before PASS:** 현재 변경을 review/commit한 뒤 CI와 tag/release workflow에서 두 target build+smoke, checksum/SBOM, attestation publication을 성공시키고 run URL과 attestation을 후속 재감사에 기록한다.
- **Residual risk:** canonical 지원 artifact의 실제 생성·실행 증거가 없으므로 release readiness는 HOLD다.

### [FIN-F024] Re-audit #2 — hosted cross-target/provenance execution

- **Re-audit date:** 2026-08-24 (Asia/Seoul)
- **Status:** **In Progress**
- **Hosted attempt:** commit `a518a99e65eee20c722b5c05240ea46d4b83d5ea`, [Release run 32648622100](https://github.com/Yupkidangju/smlcli/actions/runs/32648622100), [CI run 32648622085](https://github.com/Yupkidangju/smlcli/actions/runs/32648622085).
- **Verified partial evidence:** CI와 pre-release quality gate는 성공했다. [Linux musl job 97217553725](https://github.com/Yupkidangju/smlcli/actions/runs/32648622100/job/97217553725)는 build, `--version`/`--help` smoke, checksum/SPDX 2.3, Sigstore provenance/SBOM 생성·검증 및 artifact upload를 모두 통과했다.
- **Fail-closed evidence:** [Windows MSVC job 97217553777](https://github.com/Yupkidangju/smlcli/actions/runs/32648622100/job/97217553777)는 release build에서 실패해 이후 smoke/checksum/SBOM/attestation이 모두 skipped 되었다. `RUSTFLAGS='-D warnings' cargo build --release --locked --target x86_64-pc-windows-gnu`로 동일 Windows cfg 경고 4건을 재현했다.
- **Root cause / corrective plan:** `src/infra/secure_fs.rs`의 Unix 전용 parameter 사용과 `OpenOptions` mutation이 non-Unix compile에서 unused/unused-mut 경고가 되고 workflow의 `-D warnings`로 오류 승격된다. cfg별 구조를 분리해 경고를 제거하고 같은 local Windows cross-build와 hosted MSVC 전체 gate를 재실행한다.
- **Interim verdict:** Windows 실행 증거가 완성될 때까지 FIN-F024와 전체 판정은 `Hold`를 유지한다.

## 6. Pass 3: Security Re-audit

### [FIN-F002] Re-audit #1 — MCP environment secret isolation

- **Original Severity / Status:** Critical / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/infra/mcp_client.rs`, `src/domain/settings.rs`, MCP load path
- **Evidence:** child command에 `env_clear()` 후 안전 base allowlist와 per-server `allowed_env_vars`만 전달한다. secret-like opt-in 이름은 거부하며 stderr를 bounded/redacted sink로 소비한다.
- **Test:** `fin_f002_mcp_child_does_not_inherit_parent_pwd` 및 sentinel env fixture.
- **Residual risk:** 사용자가 명시적으로 허용한 non-secret 변수의 값은 MCP trust 결정에 포함된다.

### [FIN-F003] Re-audit #1 — @file containment/egress

- **Original Severity / Status:** Critical / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/app/chat_runtime.rs`, `src/tools/file_ops.rs`
- **Evidence:** explicit canonical root 아래 regular file만 허용하고 parent/absolute-outside/symlink-outside/NUL/invalid UTF-8/oversize를 거부한다. 256 KiB/file, 512 KiB/turn, 16 files 제한을 guard와 session/provider side effect 전에 적용한다.
- **Tests:** `fin_f003_file_context_rejects_outside_symlink_binary_and_oversize`, `fin_f003_chat_mentions_enforce_per_turn_count`.
- **Residual risk:** 사용자가 workspace 안에 직접 둔 민감 파일은 전송 전 승인/정책 판단 대상이다.

### [FIN-F004] Re-audit #1 — no-follow atomic file mutation

- **Original Severity / Status:** Critical / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/tools/file_ops.rs`
- **Evidence:** random UUID temp를 exclusive create + no-follow로 열고 fsync/mode preservation/atomic rename을 수행한다. create-only publish는 hard-link no-clobber로 race를 닫는다.
- **Tests:** `fin_f004_preexisting_deterministic_temp_symlink_cannot_modify_target`, `fin_f010_atomic_replace_preserves_existing_mode`.
- **Residual risk:** non-Unix permission semantics는 target runner에서 별도 확인한다.

### [FIN-F005] Re-audit #1 — custom provider credential routing

- **Original Severity / Status:** Critical / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/app/chat_runtime.rs`, `src/providers/registry.rs`, provider adapters, wizard/config flow
- **Evidence:** persisted `Custom: <id>`를 strict resolver로 해석하고 settings-aware registry를 atomically reload한다. missing/invalid/unsupported Gemini custom은 request 전 오류이며 credential-bearing client는 redirect를 따르지 않는다.
- **Tests:** `fin_f005_missing_or_unsupported_custom_provider_fails_closed`, `fin_f005_persisted_provider_name_is_strictly_resolved`.
- **Residual risk:** custom endpoint 자체의 신뢰성은 사용자의 explicit configuration 책임이다.

### [FIN-F007] Re-audit #1 — FetchURL SSRF/size/Unicode

- **Original Severity / Status:** Major / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/tools/fetch.rs`
- **Evidence:** syntax/IP 검사와 custom resolver가 모든 resolved address의 public 여부를 확인한다. environment proxy와 automatic redirect를 끄고 최대 5 hop을 재검증한다. 5 MiB wire cap과 UTF-8-safe 10,000-byte render cap 및 truthful metadata를 사용한다.
- **Tests:** `fin_f007_fetch_permission_denies_non_public_destinations`, `fin_f007_utf8_truncation_is_char_boundary_safe_and_truthful`.
- **Residual risk:** DNS rebinding은 connect-time `remote_addr` public check로 완화하지만 실제 다양한 resolver 동작은 live integration 범위다.

### [FIN-F008] Re-audit #1 — shell/sandbox/process fail-closed

- **Original Severity / Status:** Major / Confirmed
- **Status:** **Verified**
- **Changed files:** `src/tools/registry.rs`, `src/tools/executor.rs`, `src/tools/shell.rs`, `src/infra/sandbox.rs`, `process_reaper.rs`, permissions
- **Evidence:** immutable settings snapshot을 execution context에서 사용한다. SafeOnly path는 workspace 밖 읽기를 거부하고 bwrap capability failure는 Deny다. extra bind는 trusted roots만 `/workspace-extra/*`에 read-only로 연결하며 owned process group만 deadline/cancel/quit에서 정리한다.
- **Tests:** `fin_f008_safe_only_denies_read_paths_outside_workspace`, `fin_f008_extra_binds_are_trusted_read_only_workspace_extra_mounts`, `fin_f008_global_pid_environment_reaper_is_disabled` 및 shell process-group tests.
- **Residual risk:** 이 host는 user namespace capability가 제한되어 실제 bwrap success path 대신 fail-closed path가 검증되었다.

## 7. Cross-Pass Conflicts

### [XPF-F001] FIN-F024 정적 release control과 실제 artifact evidence

- **Related finding:** FIN-F024
- **Pass 1/3 evidence:** 문서, workflow, permissions, action SHA, checksum/SBOM/attestation control은 일치하고 정적 검사를 통과한다.
- **Pass 2 evidence:** canonical musl/MSVC artifact가 실제 hosted runner에서 생성·smoke·attest된 증거는 없다.
- **Resolution:** 낮은 판정을 우선해 FIN-F024와 전체 결론을 `Hold`로 유지한다.

## 8. Required Fixes Before PASS

코드상 새 Critical/Major 수정 요구는 발견되지 않았다. PASS 전 필요한 실행 게이트는 다음 하나다.

1. 현재 tree를 검토·커밋한 뒤 GitHub-hosted CI/release workflow에서 Linux musl 및 Windows MSVC build+smoke를 성공시킨다.
2. 생성된 checksum, SPDX SBOM, Sigstore attestation을 검증하고 workflow run URL/commit SHA를 `Re-audit #2`에 기록한다.

## 9. Accepted Risks

- 없음. FIN-F024를 risk acceptance로 면제하지 않았다.

## 10. Needs Spec Clarification

- 없음. FIN-F010, FIN-F020, FIN-F021의 이전 계약 공백은 `spec.md` §1.1과 ADR-041에서 해소되었다.

## 11. New Findings

- 없음.

## 12. Re-audit Checklist

| Finding set | Verdict |
| --- | --- |
| FIN-F001~FIN-F005 | 5 Verified |
| FIN-F006~FIN-F010 | 5 Verified |
| FIN-F011~FIN-F015 | 5 Verified |
| FIN-F016~FIN-F020 | 5 Verified |
| FIN-F021~FIN-F023 | 3 Verified |
| FIN-F024 | 1 Hold |
| FIN-F025 | 1 Verified |
| New findings | 0 |

## 13. Final Decision

**HOLD**

원 감사의 5개 Critical은 모두 해소되었고, 총 24개 finding이 문서·코드·회귀·호스트 실행 증거에서 `Verified`다. 그러나 원 Major FIN-F024의 canonical release artifact 실행 증거가 남아 있다. 표준상 검증되지 않은 build/release gate를 PASS로 해석할 수 없으므로 현재 제품 remediation은 완료 상태이지만 릴리스 판정은 HOLD다.

## 14. Coder / Release Handoff

후속 작업은 추가 코드 수정이 아니라 현재 변경 검토 및 hosted release gate 실행이다. 기준 보고서는 이 파일이며, 성공한 workflow의 commit SHA, run URL, 두 target smoke 결과와 attestation URL을 첨부해 FIN-F024 Re-audit #2만 수행하면 된다.
