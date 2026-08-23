# Sub Audit Report Supplement

## 1. Audit Metadata

- Audit Turn: 1
- Supplement: 1
- Supplement Of: `/mnt/Projects_SSD/rust/smlcli/docs/multi_audit/1/sub_audit_05_tests_concurrency.md`
- Perspective: A05 — 테스트 품질·동시성·실패 복원·데이터 무결성
- User Goal: `$multi-audit` 프로젝트의 모든 문제점을 파악하여 안정화하고 완성도를 높이는 전체감사 개시. 프로젝트의 모든 문서 및 구현내용을 먼저 파악후 작업을 분해하여 상세 감사 후 근본적인 문제를 해결할 수 있도록 합니다.
- Audit Basis: Standard-backed
- Standard Path: `/mnt/Projects_SSD/rust/smlcli/AI_AUDIT_DOC_STANDARD.md`
- Report Contract: `/home/eunho1/.codex/skills/multi-audit/references/report-contract.md`

## 2. Assigned Coverage Gap

원래 A05 범위에서 다음 세 가지 고위험 경계를 별도로 재검증했다.

1. `chat_runtime`의 `@file` preprocessing: workspace, size, binary, provider-egress
2. `file_ops::write_file_commit`: 고정 `.tmp` 경로와 선행 symlink
3. `McpClient::spawn`: parent environment secret 상속 및 차단 테스트

봉인된 원본과 peer 보고서는 읽거나 수정하지 않았다. 이 supplement는 원본에 추가 증거를 제공하며 원본 파일의 권한을 변경하지 않는다.

## 3. Excluded and Uninspected Scope

- 외부 LLM/API 호출 및 실제 provider 전송은 수행하지 않았다.
- 실제 secret 값은 읽거나 출력하지 않았다. 환경 상속 판정은 `Command` 구성과 테스트 부재를 소스 기준으로 확인했다.
- 소스·테스트·설정·기존 보고서는 수정하지 않았다. 지정된 supplement 파일만 생성했다.
- peer 보고서, `.git/`, 프로젝트 `target/`, 캐시는 조사하지 않았다.

## 4. Evidence Examined

- `src/app/chat_runtime.rs:249-325,328-495`
- `src/tools/file_ops.rs:10-53,179-239,252-410`
- `src/infra/mcp_client.rs:21-112,135-165`
- 기존 테스트 `src/tests/audit_regression.rs:2632-2700,2800-2870,3213-4150`
- `spec.md:807-815,2427-2531`
- `designs.md:736-743`
- `IMPLEMENTATION_SUMMARY.md:487-492,983-1025`
- 비교용 shell 환경 격리 `src/tools/shell.rs:125-183`
- 기존 test/source 검색 결과: `dispatch_chat_request`, `write_file_commit`, `symlink`, `env_clear` 관련 직접 회귀 테스트 부재

## 5. Findings

### [A05-F011] `@file` preprocessing이 workspace·size·binary 경계를 우회함; 직접 provider egress 우회는 반증되나 local persistence 경계는 불명확함

- Pass: Debug / Engineering Quality; Security 보조
- Pattern: `TEST-001`, `SEC-004`, `SEC-001` 보조
- Area: 파일 멘션 입력·workspace trust·provider data boundary
- Severity: Major
- Status: Confirmed
- Summary: 사용자가 입력한 `@path`는 UI fuzzy 목록을 거치지 않고 `dispatch_chat_request`에서 바로 읽힌다. `validate_sandbox`, canonical workspace 확인, 파일 크기 제한, binary 검사가 없다. 반면 `NetworkPolicy::Deny`의 provider 직접 전송은 `submit_chat_request` 후반 guard 때문에 이 경로만으로는 우회되지 않는다. 다만 guard 전에 파일 내용을 세션과 JSONL에 기록하므로 local persistence/egress 의미론은 명세 확인이 필요하다.
- Boundary Decision:
  - Workspace/traversal/symlink: `Confirmed` — `tokio::fs::read_to_string(path)`에 sandbox/trust 검사가 없다.
  - Size: `Confirmed` — 읽기 전후 byte/line cap이 없다.
  - Binary: `Confirmed` — UTF-8 `read_to_string` 실패 외에 NUL/non-printable binary 검사가 없다. `ReadFile`의 별도 binary 검사를 우회한다.
  - Direct provider egress under `NetworkPolicy::Deny`: `Rejected as a direct bypass` — `submit_chat_request`가 session/log 기록 후 `resolve_credentials()`를 호출하고 `Deny`에서 provider spawn을 중단한다.
  - Local persistence before provider guard: `Needs Clarification` — 현재 코드가 파일 내용을 session/logger에 남기는 것이 의도된 data policy인지 문서에 닫혀 있지 않다.
- Evidence:
  - `src/app/chat_runtime.rs:249-272`는 입력을 async task로 넘기고, `:272-318`에서 `@` 토큰의 path를 그대로 사용한다.
  - `src/app/chat_runtime.rs:295-304`는 `tokio::fs::read_to_string(path)`로 직접 읽고, 성공하면 전체 content를 `final_text`에 삽입한다. `validate_sandbox`, `canonicalize`, `metadata`/size cap 호출이 없다.
  - `src/app/mod.rs:2866-2905`의 fuzzy picker는 workspace `WalkBuilder`를 사용하지만, picker에서 선택된 path라는 사실을 `dispatch_chat_request`가 재검증하지 않는다. 사용자는 `@/absolute/path`, `@../outside`, symlink path를 직접 입력할 수 있다.
  - 비교로 `src/tools/file_ops.rs:10-53`은 workspace root canonicalization을 수행하고, `src/tools/file_ops.rs:79-119`의 `read_file`은 NUL/non-printable binary를 거부한다. `@file`은 이 경로를 사용하지 않는다.
  - `src/app/chat_runtime.rs:328-358`은 `submit_chat_request` 초기에 사용자 메시지를 session에 추가하고 logger에 append한다. `:467-494`에서야 `resolve_credentials()`를 호출하며, `:131-137`의 `NetworkPolicy::Deny`가 provider 호출을 차단한다.
  - 기존 `src/tests/audit_regression.rs`에는 `dispatch_chat_request`, `@file`, outside-workspace file mention, oversized mention, binary mention을 호출하는 테스트가 없다. 관련 테스트는 `ReadFile`/write tool permission 경계만 다룬다(`:2632-2700`, `:2800-2870`).
  - `spec.md:807-815`는 workspace `WalkBuilder`, 100건 UI 제한, 읽을 수 없는 파일/binary 오류 notice를 요구한다. `designs.md:736-743`도 fuzzy picker가 제공한 file match를 CTA로 설명하지만 raw dispatch 입력에 대한 재검증은 정의하지 않는다.
- Expected Basis: `spec.md` `@` mention contract, 공통 workspace/file security boundary, 사용자 지정 provider-egress·데이터 무결성 질문. 요구사항이 불명확한 local persistence 의미론은 창작하지 않고 `Needs Clarification`으로 분리한다.
- Actual: raw path와 전체 file content가 제한 없이 session 전송 준비 경로에 들어간다. provider `Deny`는 직접 HTTP를 막지만 파일 read와 local logging은 먼저 일어난다.
- Attack/Failure Preconditions: 사용자가 fuzzy picker를 우회해 `@/absolute`, `@../outside` 또는 workspace symlink path를 입력하고 프로세스가 해당 파일을 읽을 수 있어야 한다. 대용량/UTF-8 binary fixture는 별도 파일 권한 없이 재현 가능하다.
- Impact: workspace 밖 readable file 또는 symlink target이 provider prompt와 local session log에 포함될 수 있고, 대용량 파일은 memory/token budget를 소진할 수 있다. binary data는 spec의 명시적 오류 notice 없이 UTF-8 가능 영역이 그대로 들어갈 수 있다.
- Suggested Action: `@file` preprocessing을 shared canonical workspace validator와 연결하고, picker/직접 입력 모두에 동일한 root/trust/symlink check를 적용한다. read-before-allocate size cap과 binary probe를 둔다. `NetworkPolicy::Deny` 시 local persistence도 허용할지 명세를 결정하고, 금지라면 provider guard보다 앞선 session/logger 기록을 차단하거나 redaction한다.
- Re-audit Method: test-only provider/session/logger seam으로 외부 absolute path, `../`, workspace symlink, >limit UTF-8, NUL/non-printable fixture를 각각 입력하고, provider action·session message·JSONL side effect가 모두 예상대로 차단/표시되는지 확인한다. `NetworkPolicy::Deny`에서는 provider 호출과 local persistence를 별도로 assert한다.
- Owner: Coder / Architect / Security reviewer
- Confidence: High
- Notes: “provider-egress 직접 우회”는 현재 guard 때문에 rejected지만, 이것이 파일 read/local persistence까지 안전하다는 뜻은 아니다.

### [A05-F012] 선행 `.tmp` symlink가 외부 파일을 가리키면 `write_file_commit`이 외부 대상을 먼저 덮어씀

- Pass: Security / Debug 보조
- Pattern: `SEC-004`, `TEST-001`
- Area: atomic file write·symlink TOCTOU·workspace escape
- Severity: Major
- Status: Confirmed
- Summary: 대상 본체는 `validate_sandbox`가 canonicalize하지만, atomic temp path는 `"{}.tmp"`로 결정되고 `fs::write`가 기존 symlink를 따라간다. workspace 내부의 target 파일에 대해 공격자가 사전에 `target.tmp -> /external/file` symlink를 만들어 두면, `fs::write`가 외부 파일을 수정한 뒤 symlink를 rename으로 제거한다.
- Evidence:
  - `src/tools/file_ops.rs:179-198`은 입력 path만 `validate_sandbox`한 뒤 canonical target의 문자열에 고정 접미사 `.tmp`를 붙인다.
  - `src/tools/file_ops.rs:198-200`은 `fs::write(&tmp_path, new_content)`를 먼저 수행한다. `tmp_path`의 존재·regular-file 여부·symlink 여부를 `symlink_metadata`/`O_NOFOLLOW`/`create_new`로 확인하지 않는다.
  - `src/tools/file_ops.rs:200-225`는 성공 후 `fs::rename(&tmp_path, &canonical)`을 수행한다. rename은 workspace 내부 이름을 교체할 뿐, 이미 `fs::write`로 외부 symlink target에 기록된 bytes를 되돌리지 않는다.
  - `src/tools/file_ops.rs:10-53`의 sandbox 검사는 원래 target path를 확인할 뿐 `.tmp` path를 검사하지 않는다. 원래 target이 symlink인 경우와 달리, pre-existing `.tmp` symlink는 검증 대상에 포함되지 않는다.
  - 기존 tests `src/tests/audit_regression.rs:2632-2700,2800-2870`은 `/etc/passwd`/`../` 경로의 permission 및 registry guard만 확인한다. `write_file_commit` 실행 자체, `.tmp` pre-existing symlink, 외부 sentinel 보존을 검증하는 테스트는 없다.
  - `CHANGELOG.md:313`과 `:782`는 canonicalize와 temp+rename으로 symlink/atomic write가 원천 차단된다고 서술하지만, 해당 `.tmp` side path는 현재 코드와 테스트에서 닫히지 않았다.
- Expected Basis: `AI_AUDIT_DOC_STANDARD.md` `SEC-004`의 path/workspace/file-mode 제어군 분리, `spec.md:1037-1039,1169-1174`의 temp+atomic write 계약, 사용자 지정 “pre-existing symlink external write” 질문.
- Actual: 대상 path의 canonicalization만으로 temp path의 symlink follow를 막지 못한다. 공격자가 workspace를 미리 조작할 수 있고 외부 파일에 쓰기 권한이 있으면 외부 overwrite가 발생한다.
- Attack/Failure Preconditions: 공격자 또는 사전 오염된 workspace가 정상 target 옆에 `target.tmp` symlink를 만들고, symlink가 가리키는 외부 파일에 프로세스가 쓰기 권한을 가져야 한다. Unix의 기본 `fs::write` symlink-follow semantics에서 재현된다.
- Impact: atomic write를 신뢰하는 호출자가 workspace 밖 파일을 변경할 수 있다. secret/config/source 등 외부 sentinel이 덮어써질 수 있어 데이터 손상 및 path security boundary 위반이다.
- Suggested Action: temp 파일은 target directory에서 `create_new`/exclusive create로 무작위 이름을 사용하고 symlink를 절대 따라가지 않게 한다. open file descriptor 기반 write+fsync 후 rename을 사용하고, 기존 deterministic `.tmp`가 존재하면 안전하게 실패한다. rename 직전에도 temp inode/type을 확인한다.
- Re-audit Method: writable temp workspace와 외부 sentinel 파일을 준비하고 `target.tmp`를 외부 sentinel symlink로 pre-create한 뒤 실제 `WriteFileTool::execute`/`write_file_commit`을 호출한다. 기대 결과는 명시적 failure, 외부 sentinel byte/hash 불변, target의 unintended write 없음이다. 정상 경로와 dangling symlink도 함께 확인한다.
- Owner: Coder / Security reviewer
- Confidence: High
- Notes: 테스트 실행으로 실제 외부 sentinel을 변경하지 않고 source semantics로 판정했다. 이는 `fs::write`의 기본 symlink-follow 동작과 코드 순서에 근거한 confirmed finding이다.

### [A05-F013] `McpClient::spawn`이 parent environment를 그대로 상속하며 secret 차단 테스트가 없음

- Pass: Security / Debug 보조
- Pattern: `SEC-001`, `SEC-005`, `TEST-001`
- Area: MCP child process trust boundary·environment secret exposure
- Severity: Major
- Status: Confirmed
- Summary: `McpClient::spawn`은 `Command::new(cmd)`와 `args`만 설정하고 `env_clear` 또는 allowlist를 적용하지 않는다. Rust child process는 기본적으로 parent environment를 상속하므로, configured MCP command가 `OPENAI_API_KEY`, cloud credentials 또는 기타 secret 환경변수를 읽을 수 있다. 기존 MCP 테스트는 정상 JSON-RPC 왕복만 확인하고 environment 차단을 확인하지 않는다.
- Evidence:
  - `src/infra/mcp_client.rs:40-48`은 `Command::new(cmd)`, `command.args(args)`, stdio만 설정한 뒤 `command.spawn()`한다. `env_clear()`, secret denylist, explicit allowlist가 없다.
  - 같은 파일 `:52-72`의 stderr task도 child 환경을 제한하지 않고, `:75-112`에서 그대로 초기화한다.
  - 비교로 shell 경로는 `src/tools/shell.rs:168-180`에서 `command.env_clear()` 후 whitelist와 PATH/SMLCLI_PID만 주입한다. MCP에는 같은 경계가 없다.
  - `spec.md:2427-2433,2437-2445`는 MCP를 외부 MCP server command를 실행하는 stdio 통합으로 정의하지만 parent environment trust/allowlist 정책을 명시하지 않는다. 이는 제품 trust model의 clarification gap이지만, 현재 상속 사실은 코드로 confirmed다.
  - `src/tests/audit_regression.rs:4021-4150`의 MCP E2E는 initialize/list/call 성공과 response content만 확인한다. mock child가 sentinel environment를 보지 못하는지, `env_clear`가 적용됐는지 assert하는 테스트는 없다.
  - `scripts/mock_mcp_server.py`는 요청 JSON만 처리하며 `os.environ`을 검사하지 않아 현재 정상 E2E가 environment boundary를 소비하지 않는다.
- Expected Basis: `AI_AUDIT_DOC_STANDARD.md` `SEC-001` secret storage/exposure 및 `SEC-005` 실제 보호 경계 문서화, 사용자 지정 “parent environment secret inheritance” 질문. 외부 MCP command는 별도 child trust boundary로 취급해야 한다.
- Actual: parent env의 모든 항목이 MCP child에 노출될 수 있다. MCP command가 악성/손상되었거나 third-party package인 경우 secret을 읽고 외부로 전송할 수 있으며, 앱은 이를 감지·차단하지 않는다.
- Attack/Failure Preconditions: 사용자가 등록한 MCP command가 parent env를 읽을 수 있고, parent process 환경에 API key/cloud credential/센티넬이 존재해야 한다. MCP command가 trusted-only인지 third-party/untrusted도 가능한지는 현재 명세 미확정이다.
- Impact: configured MCP process 하나가 parent process 환경에 있던 API key/cloud token/CI credential을 읽을 수 있다. secret이 실제 환경에 존재하는 배포에서 Critical impact로 상승할 수 있다.
- Suggested Action: MCP child에 `env_clear` 후 명시적으로 필요한 비밀 없는 환경만 전달한다. command resolution과 PATH도 allowlist 정책에 맞추고, 필요 환경변수는 서버별 opt-in 설정으로 제한한다. trusted/untrusted MCP model과 secret inheritance 정책을 문서에 hard boundary로 기록한다.
- Re-audit Method: 실제 secret 대신 `SMLCLI_A05_SENTINEL=sentinel-value`를 parent에 설정하고 env-dump mock MCP command를 spawn한다. child stdout/JSON-RPC fixture에 sentinel이 보이지 않는지, 허용된 non-secret 변수만 남는지, 정상 initialize/list/call과 shutdown이 유지되는지 확인한다. 실제 secret 값은 사용하지 않는다.
- Owner: Coder / Architect / Security reviewer
- Confidence: High
- Notes: configured MCP server가 완전히 trusted인지 여부는 `Needs Clarification`이다. 그러나 trust decision이 문서화되지 않은 상태에서 parent env를 전부 상속하는 구현과 차단 테스트 부재는 confirmed이다.

## 6. Uncertainties and Clarifications Needed

- `@file`에서 `NetworkPolicy::Deny`가 provider HTTP만 막고 local session/logger 기록은 허용하는 것이 의도인지 결정해야 한다. 현재 코드는 guard 전에 기록한다.
- MCP 서버 command가 항상 사용자가 완전히 신뢰하는 local process인지, third-party/untrusted process도 허용하는지 명세가 없다. 후자라면 environment inheritance는 Critical secret boundary로 승격될 수 있다.
- `@file` 직접 입력이 fuzzy picker에서 나온 workspace-relative path만 허용되는지, 절대/상위 경로를 제품 기능으로 허용하는지 명세에 명시가 없다. 현재 `spec.md`와 designs는 picker 흐름만 설명한다.
- `write_file_commit`의 deterministic `.tmp` 이름이 외부 호환성 요구인지 확인이 필요하다. 그렇지 않다면 random exclusive temp path로 바꾸는 것이 안전하다.

## 7. Perspective Decision

- Decision: `HOLD` (coverage gap supplement)
- Rationale: `@file`은 세 가지 입력 경계 중 workspace·size·binary를 직접 우회하고, write temp symlink는 외부 파일 쓰기를 confirmed하며, MCP child는 parent environment secret을 confirmed 상속한다. 세 항목 모두 관련 negative/side-effect 테스트가 없다.
- Provider-egress nuance: `NetworkPolicy::Deny`의 직접 HTTP 우회는 현재 guard 때문에 rejected지만, guard 전 local persistence는 별도 명세 결정 없이는 안전하다고 판정할 수 없다.
- Required re-audit gate:
  1. `@file` outside/symlink/oversized/binary fixtures와 provider deny local-persistence assertion을 추가한다.
  2. pre-existing `.tmp` symlink external sentinel 불변 테스트를 통과시킨다.
  3. env-dump MCP mock에서 parent sentinel이 child에 전달되지 않음을 검증한다.
