# Sub Audit Supplement

## 1. Audit Metadata

- Audit Turn: 1
- Supplement: 1
- Perspective: A02 — 지정 Coverage Gap 보완 (도구 lifecycle·파일 semantics·wizard persistence)
- Audit Basis: Standard-backed
- Standard Path: /mnt/Projects_SSD/rust/smlcli/AI_AUDIT_DOC_STANDARD.md
- Report Contract: /home/eunho1/.codex/skills/multi-audit/references/report-contract.md
- Base Report: /mnt/Projects_SSD/rust/smlcli/docs/multi_audit/1/sub_audit_02_runtime_arch.md
- Base Report Handling: 봉인된 원본은 읽기·수정·덮어쓰기·권한 변경하지 않았고 동료 보고서도 읽지 않았다.

## 2. Assigned Questions

다음 네 질문만 원래 A02 범위에서 실제 호출 경로와 기존 테스트를 대조했다.

1. Action::ToolError에서 is_write_tool_running과 write_tool_queue가 해제·전진하는가.
2. handle_tool_approval의 Harness preflight Deny 조기 return이 queued approvals와 pending count를 정체시키는가.
3. WriteFile의 overwrite=false/누락 semantics가 overwrite를 막는가. ReplaceFileContent가 empty target/multiple match를 안전하게 처리하는가.
4. /setting wizard 저장이 기존 trust/custom/sandbox/git/keys를 보존하고 save 실패 시 memory/disk rollback을 수행하는가.

## 3. Excluded and Uninspected Scope

- 봉인된 base report와 peer reports는 수정·읽기하지 않았다.
- 소스, 테스트, 설정, 제품 문서 외 파일은 변경하지 않았다.
- 외부 provider/MCP/API 호출과 파괴적 명령은 수행하지 않았다.
- 아래 판정은 정적 호출 경로와 기존 테스트 존재 여부에 기반한다. 새 회귀 테스트를 작성하거나 소스를 수정하지 않았다.

## 4. Evidence Examined

- src/app/mod.rs:1238-1356 — ToolFinished/ToolError state transition
- src/app/tool_runtime.rs:93-228, :324-487, :489-639 — permission, execute, write queue, approval preflight
- src/tools/file_ops.rs:323-555 — WriteFile/ReplaceFileContent schema and execute paths
- src/app/wizard_controller.rs:182-256 — wizard settings construction, async save, immediate memory mutation
- src/app/mod.rs:1366-1377 — WizardSaveFinished success/failure handling
- src/domain/settings.rs:28-80 — PersistedSettings fields
- src/tests/audit_regression.rs:123-143, :1648-1698, :207-220, :2640-2699, :2830-2865, :4419-4489
- src/tests/settings_flow.rs — wizard state transition tests

## 5. Findings

### [A02-SUP-F001] Action::ToolError가 write queue와 is_write_tool_running을 해제·전진시키지 않는다

- Area: tool lifecycle, serialized write execution, cancellation/error path
- Severity: Major
- Status: Confirmed
- Summary: write tool 실행 중 executor가 Err를 반환하면 execute_tool_async는 Action::ToolError를 보내지만 ToolError handler에는 write queue 전진 또는 is_write_tool_running 해제가 없다.
- Evidence:
  - src/app/tool_runtime.rs:141-151 및 :551-563에서 write tool은 is_write_tool_running=true가 되고, 이미 실행 중이면 write_tool_queue에 들어간다.
  - src/app/tool_runtime.rs:464-486에서 executor Err는 ToolError로만 전송된다.
  - src/app/mod.rs:1238-1245의 queue pop/reset은 ToolFinished branch에만 있다.
  - src/app/mod.rs:1286-1356의 ToolError branch는 cancel token 제거, outcome/auto-verify/pending count 처리만 하고 is_write_tool_running 또는 write_tool_queue를 참조하지 않는다.
  - ToolError를 직접 호출하는 기존 테스트는 src/tests/audit_regression.rs:2721-2770의 pending/auto-verify만 검증하며 write queue 상태는 검증하지 않는다.
- Expected Basis: write tool은 성공·ToolResult error·executor Err·cancellation 모두 terminal event 뒤 다음 queued write가 진행되거나 queue가 비었으면 running flag가 false여야 한다. 이는 상태 전이·경합 무결성의 DBG-001/TEST-001 기준이다.
- Actual: 첫 write tool이 executor Err로 ToolError를 내면 is_write_tool_running=true가 남고 queue가 pop되지 않는다. 다음 write tool은 계속 queue에 쌓이며, 단일 실패 후에도 flag가 stale 상태로 남는다.
- Impact: WriteFile/ReplaceFileContent/ExecShell의 특정 I/O·cwd·취소 오류 후 후속 write가 영구 정체된다. pending count가 0이어도 UI와 runtime은 실행 중으로 남을 수 있다.
- Suggested Action: write terminal handling을 공통 helper로 추출해 ToolFinished와 ToolError 양쪽에서 동일하게 reset/pop/next dispatch를 수행한다. queue entry별 terminal 처리 중복 방지용 call ID를 함께 둔다.
- Re-audit Method: write_tool_running=true와 queue에 두 번째 WriteFile을 넣고 첫 도구가 executor Err를 반환하도록 fixture를 구성한다. ToolError 처리 후 flag=false 또는 두 번째 dispatch, pending count, timeline status를 모두 단언한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: 모든 file_ops 오류가 현재 ToolResult(is_error=true)로 돌아오는 것은 아니므로, Err branch를 dead code로 간주할 근거가 없다. ExecShell resolve cwd 실패 등 executor Err 경로가 존재한다.

### [A02-SUP-F002] handle_tool_approval의 preflight Deny 조기 return이 queued approval을 고립시킨다

- Area: approval queue, preflight denial, pending tool aggregation
- Severity: Major
- Status: Confirmed
- Summary: 승인된 pending tool의 재검증 preflight가 Deny이면 ToolFinished error를 비동기 전송하고 즉시 return한다. 이 return이 하단의 queued_approvals 승격 로직을 건너뛰므로 queue와 pending count가 정체된다.
- Evidence:
  - src/app/tool_runtime.rs:489-505에서 pending_tool/ID/index와 pending_since를 먼저 take/clear한다.
  - src/app/tool_runtime.rs:507-538에서 HarnessPreflightDecision::Deny 시 error ToolFinished를 spawn한 뒤 return한다.
  - 같은 함수의 src/app/tool_runtime.rs:609-638에 queued_approvals를 다음 pending_tool로 올리는 로직이 있으나 Deny return 뒤에는 도달하지 않는다.
  - src/app/tool_runtime.rs:86-89에서 원래 valid tool call 수만큼 pending_tool_executions를 증가시킨다.
  - src/app/mod.rs:1037-1067 및 :1286-1356의 terminal handling은 event를 하나 줄이지만 approval queue가 비어 있어야만 후속 LLM resend 조건을 만족한다.
  - 기존 src/tests/audit_regression.rs:1648-1698은 TTL 만료 경로의 queue promotion만 검증한다. preflight Deny 후 queue promotion 테스트는 없다.
- Expected Basis: Deny도 approval 요청 하나의 terminal outcome이므로 queue의 다음 항목을 즉시 노출하고, 전체 turn의 pending aggregation을 정합적으로 줄여야 한다.
- Actual: denied pending approval은 UI의 승인 블록이 남은 채 pending_tool은 None이 되고 queued_approvals는 그대로 남는다. pending count가 남아 있으면 flush/resend가 막힌다.
- Impact: workspace trust/snapshot drift 등 정상적인 재검증 Deny 후 모든 후속 승인과 해당 AI turn이 멈출 수 있다.
- Suggested Action: Deny를 공통 terminal transition으로 처리하고 return 전에 next queued approval을 승격하거나 helper를 호출한다. Deny된 approval block도 Error로 갱신하고 pending count/turn outstanding set을 함께 정리한다.
- Re-audit Method: pending tool을 trusted 상태에서 만들고 승인 직전 settings를 Restricted/denied로 바꾼 뒤 y 입력을 보낸다. queued approval이 새 pending card로 표시되고 count가 정확히 감소하며 stale Approval block이 남지 않는지 확인한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: 단순 사용자의 n 거부 경로는 하단 queue promotion에 도달하지만, preflight Deny만 조기 return한다.

### [A02-SUP-F003] WriteFile의 overwrite=false/누락 값이 기존 파일 overwrite를 막지 않는다

- Area: WriteFile contract, destructive file write semantics
- Severity: Major (contract-dependent)
- Status: Needs Clarification
- Summary: schema와 승인 UI에는 overwrite boolean이 있지만 execute 경로가 값을 읽지 않는다. 따라서 false 또는 필드 누락 모두 기존 파일을 그대로 덮어쓴다. 다만 spec은 bool 필드만 선언하고 false의 의미를 명시하지 않아 최종 기대 계약은 Needs Clarification이다.
- Evidence:
  - src/tools/file_ops.rs:335-348은 overwrite를 optional boolean property로 노출한다.
  - src/tools/file_ops.rs:365-372은 승인 표시에서만 overwrite 값을 읽어 생성/덮어쓰기 문구를 선택한다.
  - src/tools/file_ops.rs:387-408의 execute는 path/content만 읽고 overwrite를 읽지 않는다.
  - src/tools/file_ops.rs:179-239의 write_file_commit은 항상 temp write 후 fs::rename(&tmp_path, &canonical)하여 existing target을 교체한다.
  - src/tests/audit_regression.rs:207-220은 overwrite=true에서 AlwaysAsk 권한만 확인하며 실제 overwrite=false/누락 execute test는 없다.
  - spec.md:444-448은 WriteFile { path, content, overwrite } 타입만 선언하고 false/누락 동작을 정의하지 않는다. spec.md:1020-1023은 diff 없는 즉시 덮어쓰기를 금지한다.
- Expected Basis: 제품 owner가 overwrite=false/누락을 “existing target이면 Deny/error”로 할지, 단순 UI hint로 할지 명시해야 한다. 현재 구현을 안전한 no-clobber 계약으로 해석하면 Needs Fix다.
- Actual: existing file은 overwrite=false와 field omission 모두 교체된다. 승인 전 preview 문구와 실제 operation semantics가 다를 수 있다.
- Impact: 모델이 false를 통해 신규 파일 생성만 요청했다고 표시해도 사용자 파일이 덮어써질 수 있다. 기존 파일 보호 계약이 필요하다.
- Suggested Action: contract를 결정한 뒤 no-clobber semantics라면 target existence를 recheck하고 overwrite=false/누락에서 구조화된 error를 반환한다. 덮어쓰기가 허용된 경우에도 field를 required로 만들고 preview·execute·tests를 동일 의미로 고정한다.
- Re-audit Method: temp workspace에서 existing file을 만들고 overwrite=false, true, omission 각각을 실행해 bytes와 ToolResult status를 비교한다. spec/schema/approval preview도 같은 semantics인지 확인한다.
- Owner: Architect / Coder / Human (contract decision)
- Confidence: High (actual), Medium (expected contract)
- Notes: 이 finding은 실제 ignore 동작은 Confirmed이고, false의 제품 의미만 Needs Clarification이다.

### [A02-SUP-F004] ReplaceFileContent의 empty target이 파일 전체에 replacement를 삽입한다

- Area: ReplaceFileContent input validation and data integrity
- Severity: Major
- Status: Confirmed
- Summary: target_content가 빈 문자열이면 contains가 true가 되고 String::replace("", replacement)가 모든 UTF-8 경계에 삽입을 수행한다. 빈 target을 거부하는 검증이 없다.
- Evidence:
  - src/tools/file_ops.rs:424-438 schema는 target_content에 minLength 또는 non-empty validation이 없다.
  - src/tools/file_ops.rs:516-531은 !old_content.contains(&target)만 확인한 뒤 old_content.replace(&target, &replacement)를 호출한다. 빈 문자열은 contains 조건을 통과한다.
  - src/tools/file_ops.rs:459-475의 diff preview도 같은 old_text.replace(target, replacement)를 사용해 승인 preview부터 과도한 변경을 보여줄 수 있다.
  - src/tests/audit_regression.rs에는 empty target 실행/preview regression이 없다.
- Expected Basis: “specific contiguous block”을 교체하는 도구는 empty target을 유효한 target으로 취급하면 안 된다. 파일 변경은 데이터 무결성과 diff preview가 동일해야 한다.
- Actual: 빈 target은 에러가 아니라 replacement를 문자열 경계마다 삽입하는 변경으로 진행된다.
- Impact: 짧은 입력 실수나 모델 malformed args가 파일 전체를 대규모로 변형할 수 있고, 승인 diff가 길이 제한/렌더링에 의해 위험을 숨길 수 있다.
- Suggested Action: target_content.trim 또는 exact empty를 즉시 InvalidArguments로 거부하고 schema minLength=1과 tool-level regression을 추가한다.
- Re-audit Method: 기존 파일에 empty target과 replacement를 넣어 execute/preview 모두 non-mutating error인지, 파일 bytes가 보존되는지 확인한다.
- Owner: Coder
- Confidence: High
- Notes: replacement가 empty인 경우와 target이 empty인 경우를 별도 fixture로 유지해야 한다.

### [A02-SUP-F005] ReplaceFileContent의 multiple match semantics가 all-replace로 고정됐지만 계약과 테스트가 없다

- Area: ReplaceFileContent match cardinality and approval correctness
- Severity: Major (contract-dependent)
- Status: Needs Clarification
- Summary: description은 specific contiguous block 단수로 보이지만 execute와 preview는 target의 모든 occurrence를 replace한다. first-only/exactly-one/replace-all 중 제품 계약이 정해져 있지 않다.
- Evidence:
  - src/tools/file_ops.rs:428-437은 target_content/replacement_content만 받고 match cardinality나 replace_all flag가 없다.
  - src/tools/file_ops.rs:470-475 preview와 :516-537 execute가 모두 String::replace를 사용해 모든 occurrence를 변경한다.
  - target이 여러 번 있어도 :516-529의 contains 검사만 통과하면 success ToolResult가 반환된다.
  - src/tests/audit_regression.rs에는 0/1/multiple occurrence별 결과 assertion이 없다.
- Expected Basis: 승인 가능한 file edit는 preview와 execute가 동일한 match set을 대상으로 해야 하며, 단수 “specific block”을 의미한다면 exactly-one이 불변조건이다. 문서가 replace-all을 의도했다면 이를 명시해야 한다.
- Actual: 다중 match는 사용자에게 “어느 occurrence를 바꾸는지” 별도 선택/경고 없이 모두 바뀐다.
- Impact: 동일 문자열이 여러 위치에 있는 소스에서 의도하지 않은 대량 변경이 발생하고 auto-verify/Git rollback 범위가 커진다.
- Suggested Action: 제품 계약을 결정한다. exactly-one이면 match count를 계산해 0/복수에서 error/approval warning으로 종료하고, replace-all이면 schema에 explicit replace_all과 count를 preview에 표시한다.
- Re-audit Method: 동일 target 0/1/2 occurrence fixture에서 preview diff와 execute bytes가 일치하는지, ambiguous case가 명시적 error/confirmation으로 승격되는지 확인한다.
- Owner: Architect / Coder / Human (contract decision)
- Confidence: High (actual), Medium (expected contract)
- Notes: empty target은 F004에서 별도 확정 finding으로 다룬다.

### [A02-SUP-F006] /setting wizard가 기존 trust/custom/sandbox/git/keys 설정을 보존하지 않고 새 defaults로 교체한다

- Area: wizard settings merge, persisted configuration integrity
- Severity: Major
- Status: Confirmed
- Summary: wizard save가 기존 PersistedSettings를 clone/patch하지 않고 새 struct literal을 생성한다. 기존 workspace trust, custom providers, sandbox, git integration, encrypted keys와 여러 사용자 설정이 사라진다.
- Evidence:
  - src/app/wizard_controller.rs:199-216은 version/provider/model/policies/theme/lmstudio URL/empty encrypted_keys를 새로 만들고 나머지를 Default로 채운다.
  - src/domain/settings.rs:28-80에 trusted_workspaces, denied_roots, extra_workspace_dirs, allowed_env_vars, git_integration, custom_providers, sandbox, mcp_servers, encrypted_keys 등이 존재한다.
  - wizard literal은 trusted_workspaces, denied_roots, extra_workspace_dirs, allowed_env_vars, git_integration, custom_providers, sandbox, mcp_servers를 기존 state에서 복사하지 않는다.
  - src/app/wizard_controller.rs:218-225는 현재 wizard key가 있을 때 새 빈 encrypted_keys에 단일 alias만 삽입한다. LM Studio처럼 key가 비어 있으면 기존 keys가 전부 제거된다.
  - src/tests/audit_regression.rs:4419-4489의 LM Studio wizard test는 Saving 단계 진입까지만 검증하고 실제 settings merge/persistence를 호출하지 않는다. 기존 trust/custom/sandbox/git/key 보존 test는 확인되지 않았다.
- Expected Basis: /setting은 provider/key/model onboarding UI이지 기존 workspace policy와 custom/runtime configuration을 암묵적으로 파괴하는 reset operation으로 문서화되어 있지 않다. 기존 설정 보존은 session/config data integrity 불변조건이다.
- Actual: 이미 configured app에서 /setting을 다시 완료하면 보존 대상이 defaults/empty로 바뀐다. 특히 trust가 Unknown으로 돌아가고 custom provider/LM URL registry가 재구성되어야 하며, 기존 API key가 유실될 수 있다.
- Impact: 사용자가 재설정만 했는데 권한 정책·workspace trust·custom endpoint·Git automation·암호화 credentials를 잃는다. 이후 tool denial, 잘못된 provider fallback, 재인증/데이터 손실로 이어진다.
- Suggested Action: existing settings clone을 base로 하고 wizard-owned fields만 patch한다. provider 변경 시 해당 alias만 교체하고 다른 encrypted keys를 유지하며, reset-all은 별도 명시적 command/confirmation으로 분리한다.
- Re-audit Method: 모든 PersistedSettings 필드를 sentinel 값으로 채운 뒤 /setting save를 수행하고 각 필드·config.toml·provider registry가 유지되는지 비교한다. LM Studio/no-key와 custom provider/key 두 경로를 포함한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: 첫 실행 settings=None이면 Default 생성은 정상이지만, 기존 settings가 있는 재진입 경로와 구분되어야 한다.

### [A02-SUP-F007] wizard save 실패 시 memory settings가 복구되지 않아 disk와 runtime이 분기된다

- Area: async wizard save, rollback, failure propagation
- Severity: Major
- Status: Confirmed
- Summary: save_wizard_settings가 async save task를 시작한 뒤 즉시 state.domain.settings를 새 settings로 교체한다. save 실패 이벤트는 wizard error만 표시하고 이전 memory settings를 복원하지 않는다.
- Evidence:
  - src/app/wizard_controller.rs:231-251은 settings_clone 저장 task를 detached spawn하고 성공/실패 event만 보낸다.
  - src/app/wizard_controller.rs:254-255는 save completion 전에 self.state.domain.settings = Some(settings)로 memory를 변경한다.
  - src/app/mod.rs:1366-1377의 WizardSaveFinished Err는 step/err_msg만 변경하며 이전 settings snapshot이나 disk reload/rollback을 수행하지 않는다.
  - src/tests/audit_regression.rs:123-143의 test_wizard_save_failure_keeps_wizard_open은 실제 App settings snapshot/disk failure를 수행하지 않고 boolean 상태만 시뮬레이션한다.
- Expected Basis: config save가 실패하면 memory와 disk가 같은 이전 snapshot으로 rollback되거나, pending state가 runtime에 사용되지 않아야 한다. async persistence는 transaction boundary를 명시해야 한다.
- Actual: 디스크 rename/write가 실패해도 runtime은 새 provider/model/policy/key를 계속 사용하고 wizard만 Saving error 상태에 남는다. 기존 설정은 disk에 남아 runtime/disk가 divergent하다.
- Impact: 같은 실행 중 provider/권한/credential resolution은 새 값, 재시작 후에는 이전 값이 되어 사용자가 저장 결과를 오판한다. failure 후 재시도도 어느 snapshot을 기준으로 하는지 불명확하다.
- Suggested Action: old settings snapshot과 pending settings를 분리해 저장 성공 event에서만 memory를 commit한다. 실패 시 pending을 버리고 old snapshot을 유지하며, event에는 revision/attempt ID를 포함한다.
- Re-audit Method: read-only config path 또는 injected failing store에서 existing settings를 유지한 채 wizard save를 실패시키고 memory, disk, wizard state, registry 모두 old snapshot인지 확인한다. 성공 path에서는 commit 시점 이후에만 new snapshot이 보이는지 확인한다.
- Owner: Architect / Coder
- Confidence: High
- Notes: 암호화 key 생성이 save 전에 수행되므로 실패 시 새 master key/secret side effect도 별도 cleanup/transaction 검토가 필요하다.

## 6. Uncertainties and Clarifications Needed

- F003과 F005의 expected semantics는 현재 문서가 완전히 닫지 않았다. overwrite=false가 no-clobber인지, ReplaceFileContent multiple match가 replace-all인지 제품 owner가 결정해야 한다.
- actual behavior는 두 경우 모두 소스에서 Confirmed이며, 계약 결정 후 관련 finding을 Needs Fix 또는 Rejected/Documented로 재분류해야 한다.
- F001/F002/F004/F006/F007은 계약 해석과 무관하게 현재 상태 전이 또는 데이터 보존 결함이 Confirmed이다.
- base report의 sealed lineage는 보존해야 하며 이 supplement는 새 증거/coverage만 추가한다.

## 7. Perspective Decision

HOLD (coverage supplement).

네 질문 중 ToolError write queue, approval preflight Deny, Replace empty target, wizard settings preservation, wizard save rollback은 Major 가능성이 높거나 Confirmed Major이며, 기존 회귀 테스트가 이 경계를 직접 잠그지 않는다. WriteFile overwrite flag와 Replace multiple match는 실제 동작은 확인했으나 제품 계약이 불명확하므로 Needs Clarification으로 유지한다. 원본 A02 report는 수정하지 않고 이 supplement를 통합 모델의 보완 source report로 취급해야 한다.

## 8. Re-audit Checklist

- [ ] ToolFinished와 ToolError 양쪽에서 write queue terminal transition을 공통 helper로 검증한다.
- [ ] preflight Deny 후 queued approval promotion, pending count, approval block status를 검증한다.
- [ ] WriteFile overwrite false/true/omission contract와 existing bytes를 fixture로 잠근다.
- [ ] Replace empty target rejection과 0/1/multiple occurrence semantics를 문서·schema·preview·execute에 정렬한다.
- [ ] wizard re-entry에서 trust/custom/sandbox/git/encrypted_keys/mcp settings 보존을 검증한다.
- [ ] wizard save failure에서 memory/disk/registry rollback과 retry revision을 검증한다.

## 9. Coder Handoff

/mnt/Projects_SSD/rust/smlcli/docs/multi_audit/1/sub_audit_02_runtime_arch_supplement_1.md를 base report와 함께 읽고, 위 finding의 실제 계약을 프로젝트 문서와 대조한 뒤 수정하세요. 수정 후 각 Re-audit Method와 관련 회귀 테스트 결과를 기록하고 봉인된 base report는 변경하지 마세요.

