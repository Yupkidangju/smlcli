# Sub Audit Report

## 1. Audit Metadata

- Audit Turn: 1
- Perspective: A04 — TUI·사용성·접근성·다국어·렌더링 완성도
- User Goal: `$multi-audit` 프로젝트의 모든 문제점을 파악하여 안정화하고 완성도를 높이는 전체감사
- Audit Basis: Standard-backed
- Standard Path: `AI_AUDIT_DOC_STANDARD.md`; `/home/eunho1/.codex/skills/multi-audit/references/report-contract.md`
- Source-report policy: `docs/multi_audit/1`의 동료 보고서는 읽지 않았고, 현재 코드·프로젝트 문서·지정 테스트만 사용했다.

## 2. Assigned Scope

다음 범위를 A04 관점에서 대조했다.

- `designs.md`, `redesign_plan.md`, `stitch_modern_tui_redesign/terminal_precision/DESIGN.md`
- 5개 README 로케일의 TUI/단축키/반응형/접근성 관련 주장
- `src/tui/**`
- `src/app/state.rs`, `src/app/mod.rs`, `src/app/wizard_controller.rs`, `src/app/command_router.rs`, `src/app/tool_runtime.rs`, `src/app/chat_runtime.rs`
- 관련 회귀 테스트 및 `spec.md`, `audit_roadmap.md`, `CHANGELOG.md`의 구현·검증 주장

중점 추적 경로는 Timeline, Composer, Inspector, Command Palette, Slash Menu, Setup Wizard, Config Dashboard, Approval, Workspace Trust, Questionnaire, overlay stack, resize/mouse routing, locale selection/fallback, Unicode width/ANSI/ASCII fallback, terminal cleanup이다.

## 3. Excluded and Uninspected Scope

- 소스·테스트·설정·제품 문서는 수정하지 않았다. 이 보고서 파일만 생성했다.
- `.git`, `target`, 캐시, 네트워크·외부 API, `build.sh`, 유료 검사 및 파괴적 검사는 제외했다.
- 실제 물리 TTY에서의 육안 검증, 실제 마우스 장치/터미널 에뮬레이터별 렌더링, RTL 폰트 조합 검증은 실행하지 못했다. 따라서 해당 항목은 `Uncertainties and Clarifications Needed`에 남긴다.
- 동료 `docs/multi_audit/1/sub_audit_*.md`는 읽지 않았다.

## 4. Evidence Examined

### Documents

- `designs.md`: Terminal-first 키 모델, approval UX, Inspector 계약, 좁은 터미널, 접근성, Questionnaire UX
- `redesign_plan.md`: Terminal Precision 색상/레이아웃 수치, i18n, `TuiAction`, Diff cache, Wizard/Questionnaire micro-task
- `stitch_modern_tui_redesign/terminal_precision/DESIGN.md`: adaptive header, rounded/ASCII border fallback, fixed-grid/contrast 규칙
- `spec.md`: Phase 15 typed contracts, breakpoints, toolbar, snapshot 검증, Wizard/Questionnaire 성공 기준
- `README.md`: 5개 로케일의 기능·키보드·반응형·TUI 완성 주장
- `audit_roadmap.md`, `CHANGELOG.md`: Phase 35/47/52 완료·검증 주장

### Source and tests

- `src/tui/layout.rs`, `src/tui/i18n.rs`, `src/tui/palette.rs`, `src/tui/terminal.rs`, `src/tui/help_overlay.rs`
- `src/tui/widgets/{mod,config_dashboard,input_field,inspector_tabs,questionnaire,setting_wizard}.rs`
- `src/app/state.rs`, `src/app/mod.rs`, `src/app/event_loop.rs`, `src/app/action.rs`
- `src/app/{wizard_controller,command_router,tool_runtime,chat_runtime}.rs`
- `src/tests/audit_regression.rs`

### Commands and results

- `CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo test --locked i18n -- --nocapture` — 1 passed
- `CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo test --locked mouse_wheel_routing -- --nocapture` — 1 passed
- `CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo test --locked centered_rect_formula -- --nocapture` — 1 passed
- `CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo test --locked inspector_tab_cycles -- --nocapture` — 1 passed
- `CARGO_TARGET_DIR=/tmp/smlcli-multi-audit-1-target cargo test --locked visible_window_start -- --nocapture` — 1 passed

위 테스트들은 상태 전이·수학식·일부 좌표만 검증하며 실제 `ratatui` 화면 버퍼, locale별 문자열, 80/100/120/140 폭, overlay stack을 검증하지 않는다.

## 5. Findings

### [A04-F001] Approval의 표시 키와 실제 입력 라우팅이 서로 다르며 Enter/Esc가 안전한 CTA가 아니다

- Pass: Implementation Compliance / Debug / Engineering Quality
- Pattern: `IMP-001`, `TEST-001`
- Area: Approval, input routing, high-risk action visibility
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: 설계의 기본 approval 계약은 `Enter` 승인·`Esc` 취소/거부인데, 실제 Inspector는 `y/n`만 표시하고 해당 두 키만 처리한다. pending approval이 생겨도 Inspector 포커스를 설정하지 않아 기본 포커스가 Composer에 남는다.
- Evidence:
  - `designs.md:66-90`은 Approval 카드에 `[ Approve (Enter) ] [ Reject (Esc) ]`를 표시한다.
  - `designs.md:866-875`는 `Enter: 선택 / 승인 / 제출`, `Esc: 취소 / 뒤로가기 / 패널 닫기`로 고정한다. 같은 문서의 `designs.md:1513-1532`에는 MCP 전용으로 `y/n` 표기가 있어 문서 자체도 계약이 충돌한다.
  - `src/tui/layout.rs:934-998`은 `Press 'y' to Approve, 'n' to Reject.`만 출력한다.
  - `src/app/mod.rs:2184-2194`는 pending tool일 때 문자 `y/n`만 `handle_tool_approval()`로 보낸다. `src/app/mod.rs:1993-2050`, `src/app/mod.rs:2668-2820`에는 pending approval을 위한 Enter/Esc 분기가 없다. 그 결과 Esc는 다른 overlay가 없으면 종료 경로(`should_quit = true`)로 흐르고, Enter는 pending tool을 승인하지 않는다.
  - `src/app/tool_runtime.rs:156-178`, `:665-684`는 pending approval 및 `show_inspector = true`만 설정하고 `focused_pane = Inspector`를 설정하지 않는다.
  - 회귀 테스트에는 approval의 Enter/Esc/y/n 실제 키 라우팅 테스트가 없다. 실행한 `test_mouse_wheel_routing` 등은 이 경계를 검증하지 않는다.
- Expected Basis: `designs.md`의 Approval 카드/키 모델, `spec.md:1375-1399`의 diff/write approval 계약.
- Actual: 표시·문서·입력 구현이 서로 다른 키 계약을 가지며, 사용자가 가장 중요한 승인 화면에서 Enter로 진행할 수 없고 Esc가 앱 종료로 해석될 수 있다.
- Impact: 변경 승인 실수, 승인 불능, 앱 종료를 거부 동작으로 오인할 위험이 있다. 위험 동작의 가시성·취소 가능성·키보드 접근성이 동시에 깨진다.
- Suggested Action: approval 키 계약을 하나로 확정하고 문서/렌더러/라우터를 동기화한다. pending approval을 최우선 상태로 두어 Enter=승인, Esc=거부/취소를 결정적으로 처리하고 Inspector/Approval 포커스를 설정한다. y/n을 유지하려면 모든 문서와 hint를 y/n으로 바꾸고 Esc 종료와 충돌하지 않게 별도 취소키를 둔다. 두 경로 모두 회귀 테스트를 추가한다.
- Re-audit Method: pending WriteFile/ExecShell 상태를 fixture로 만든 뒤 Enter, Esc, y, n 각각에 대해 pending 상태·tool 결과·`should_quit`·focus·다음 approval queue를 확인하고 화면 hint와 일치하는지 버퍼 테스트로 검증한다.
- Confidence: High
- Notes: MCP 전용 y/n 예시가 전체 approval 계약의 권위인지 일반 approval 계약의 권위인지 `Needs Spec Clarification`이 필요하다.

### [A04-F002] Timeline 포커스가 키보드로 도달 불가능하여 Terminal-first 약속을 위반한다

- Pass: Implementation Compliance / Debug / Engineering Quality
- Pattern: `IMP-001`, `IMP-003`
- Area: Focus model, keyboard accessibility, timeline selection/scroll
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: `FocusedPane::Timeline`을 설정하는 일반 키보드 경로가 없다. Timeline 선택·블록 접기·Timeline 내용 복사는 mouse click으로만 도달하며, `Tab`은 pane 순환이 아니라 PLAN/RUN 전환으로 사용된다.
- Evidence:
  - `designs.md:17-38`은 마우스 없이 주요 동작을 키보드로 3단계 이내에 도달해야 한다고 한다. `designs.md:1370-1383`, `spec.md:1572-1575`는 pane focus/키보드 라우팅을 계약으로 둔다.
  - `src/app/mod.rs:2132-2144`에서 Composer 상태의 Tab/Shift+Tab은 PLAN/RUN만 토글한다. Timeline으로 focus를 옮기는 키 분기가 없다.
  - `src/app/mod.rs:2339-2347`, `:2426-2434`, `:2784-2791`의 선택·스크롤·Enter 접기 로직은 `FocusedPane::Timeline`일 때만 동작한다.
  - 실제 Timeline focus 대입은 `src/app/mod.rs:2649-2663`의 mouse pane routing에만 존재한다. `y` 복사도 `src/app/mod.rs:2250-2279`에서 non-Composer focus를 전제로 한다.
  - `test_mouse_wheel_routing`은 mouse click/scroll만 확인하며 키보드로 Timeline focus를 획득하는 테스트는 없다.
- Expected Basis: `designs.md:19-20`, `spec.md:1572-1575`의 keyboard-first/focused-pane 계약.
- Actual: 기본 Composer에서 키보드만으로 Timeline selection/focus를 얻을 수 없다. `?` 컨텍스트 도움말의 Timeline 경로도 mouse로만 도달한다.
- Impact: 키보드 사용자는 block 접기, 선택, copy, Timeline 전용 탐색을 사용할 수 없고 primary timeline UX에서 이탈한다. 저시력/마우스 없는 환경 접근성이 낮아진다.
- Suggested Action: Tab을 mode 전환과 focus 전환 중 하나로 명확히 분리하거나 명시적 pane-focus 단축키를 추가하고, 모든 focus 상태의 도움말을 동기화한다. Timeline/Inspector/Composer에서 focus 순환과 selection/scroll을 headless key fixture로 잠근다.
- Re-audit Method: 실제 UI를 조작하지 않고 `handle_input()` fixture에서 기본 Composer → Timeline → Inspector → Composer 순환과 Up/Down/Enter/y 결과를 확인한다. 해당 focus가 각 renderer의 강조 상태와 일치하는지 TestBackend snapshot으로 검증한다.
- Confidence: High

### [A04-F003] Inspector의 계획된 CTA·Alt 단축키·마우스 동작이 구현되지 않았다

- Pass: Implementation Compliance
- Pattern: `IMP-001`, `IMP-003`
- Area: Inspector Preview/Diff/Search/Recent/Git interaction
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: Inspector 탭은 렌더링되지만 계획된 Preview/검색/재진입/승인 CTA와 Alt 단축키가 실제 라우팅되지 않는다. 마우스는 pane focus만 바꾸고 탭/버튼을 선택하지 않는다.
- Evidence:
  - `designs.md:304-328`은 Preview의 Enter 승격, Search의 `/` 검색어 수정·Enter Preview 전환을 요구한다. `designs.md:337-343`은 Recent에서 최근 파일/grep/diff/명령 재진입을 요구한다.
  - `redesign_plan.md:108-137`은 5개 탭과 `TuiAction::SwitchTab`, `SetBlockApproval`, `ResizeTerminal` 계약 및 Alt+1~5를 명시한다. `audit_roadmap.md:588-598`은 Alt+1~6 연동을 완료했다고 주장한다.
  - `src/app/mod.rs:2451-2474`에는 Tab/BackTab 순환만 있고 Alt+1~6 처리 분기가 없다. `src/app/action.rs:25-121`에도 `TuiAction` 계약이 없다.
  - `src/app/mod.rs:2732-2820`의 Enter 라우팅에는 Palette/Slash/Fuzzy/Config/Wizard/Timeline만 있고 Inspector case가 없다. 따라서 Search에서 Enter로 Preview 전환, Preview에서 파일 승격, Recent 재진입, Inspector approval CTA는 실행되지 않는다.
  - `src/app/mod.rs:2649-2663`의 mouse handler는 Timeline/Inspector/Composer focus만 바꾼다. 탭 헤더나 CTA 좌표는 처리하지 않는다.
  - `src/tui/widgets/inspector_tabs.rs:334-451`의 Search는 Composer 버퍼를 표시하지만 Search 전용 입력/Preview 전환 상태가 없다. `:474-543`의 Recent는 최근 도구만 표시한다.
- Expected Basis: `designs.md` Inspector 계약, `redesign_plan.md:342-354`, `spec.md:1773-1778`의 Inspector/shortcut/snapshot 계획.
- Actual: 탭을 보는 것은 가능하지만 보이는 탭과 CTA가 실행 가능한 interaction surface로 닫히지 않았다.
- Impact: Inspector가 “preview/diff/search/recent workspace”가 아니라 사실상 읽기 전용 탭으로 축소되고, 사용자가 표시된 버튼/탭을 눌러도 변화가 없다.
- Suggested Action: 탭 수와 단축키 권위를 먼저 확정하고(`Recent`/`Git` 포함 여부), keyboard/mouse CTA를 동일한 action으로 라우팅한다. Preview line selection, Search query mode, Recent reentry, approval CTA를 상태 계약과 함께 구현하거나 문서에서 계획 기능으로 명시적으로 이관한다.
- Re-audit Method: 각 탭에서 Tab/BackTab/Alt+1..6/마우스 클릭/Enter/`/` 입력 fixture를 실행하여 active tab, query, preview target, approval result의 상태 변화와 화면을 확인한다.
- Confidence: High
- Notes: 문서 간 탭 계약이 5개(`designs.md:294-302`, `redesign_plan.md:108-116`)와 6개(`designs.md:1370-1378`, 실제 enum)로 충돌하므로 canonical spec clarification이 필요하다.

### [A04-F004] `session.messages`와 `ui.timeline` 분리로 `/status`가 사라지고 `/clear`가 화면을 지우지 않는다

- Pass: Implementation Compliance
- Pattern: `IMP-001`, `IMP-002`
- Area: Timeline state consumption, command output visibility
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: Timeline이 한 번이라도 채워지면 renderer는 `session.messages`를 더 이상 읽지 않는다. 그러나 `/status`는 session message에만 기록하고 `/clear`는 session message만 삭제하므로, 정상적인 사용 흐름에서 상태 출력은 보이지 않고 clear 후 오래된 Timeline이 남는다.
- Evidence:
  - `src/tui/layout.rs:409-410`은 `ui.timeline`이 비어 있지 않으면 TimelineBlock만 렌더링한다. `:626-655`의 `session.messages` fallback은 timeline이 빈 경우에만 실행된다.
  - `src/app/chat_runtime.rs:390-404`는 첫 일반 프롬프트부터 `ui.timeline`에 User block을 추가한다.
  - `src/app/command_router.rs:295-317`의 `/status`는 `[Status]`를 `domain.session.messages`에만 추가한다.
  - `src/app/command_router.rs:326-329`의 `/clear`는 pinned session messages만 남기고 `ui.timeline`, `timeline_cursor`, scroll을 초기화하지 않는다.
  - `/tokens`만 `src/app/command_router.rs:333-348`에서 별도로 Timeline block을 추가하며, `test_tokens_command_pushes_visible_timeline_notice`가 이 경로만 검증한다.
- Expected Basis: `spec.md:1560-1563`, `designs.md:155-168`, `designs.md:608-619`의 block-first timeline과 사용자-visible system notice 계약.
- Actual: `/status` 실행 결과가 화면에 나타나지 않으며 `/clear` 후 화면에는 과거 block이 남는다. 세션 데이터와 화면 데이터가 동일한 사용자 mental model을 제공하지 않는다.
- Impact: 명령이 무시된 것처럼 보이고, 사용자가 clear를 했다고 믿은 뒤 이전 대화/도구 결과를 계속 보게 된다. 상태 확인·복구·재현성이 저하된다.
- Suggested Action: command/system output을 단일 `push_timeline_notice()` 경로로 소비시키고 `/clear`에서 session과 timeline 및 모든 pane anchor를 함께 초기화한다. `session.messages ↔ timeline` 동기화 정책을 문서에 고정한다.
- Re-audit Method: 일반 User/AI block을 만든 뒤 `/status`, `/clear`, `/help`, `/tokens`를 실행하여 각 결과가 한 번만 visible하고 clear 후 모든 화면 상태가 빈지 확인한다.
- Confidence: High

### [A04-F005] Responsive breakpoints와 Drawer/Inspector 폭이 동결 수치와 다르고 작은 화면에서 toolbar 우선순위가 없다

- Pass: Implementation Compliance / Debug / Engineering Quality
- Pattern: `IMP-001`, `BUILD-001`
- Area: Responsive layout, resize, small-screen usability
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: `<100` drawer가 계획된 35열이 아니라 32열이고, `100..103`열에서는 Inspector 실제 폭이 28~31열로 최소 32열보다 작다. Composer toolbar는 CWD/context/policy/hint를 폭에 따라 숨기거나 재배치하지 않으며, `<80`에서는 설계된 축약 화면 대신 전체 UI를 경고 화면으로 대체한다.
- Evidence:
  - `redesign_plan.md:59-69`은 `>=100` Inspector 30%(32~48열), `<100` F2 Drawer 35열, Timeline 최소 45열, 80열 미만 가이드라인을 명시한다.
  - `spec.md:1674-1700`은 compact/standard/wide breakpoint와 Inspector 32~48열, chip 최대 길이 18을 고정한다.
  - `src/tui/layout.rs:52-91`은 폭 80 또는 높이 24 미만이면 전체를 `터미널 크기가 너무 작습니다` 경고로 중단한다.
  - `src/tui/layout.rs:112-145`는 compact Drawer를 32열로 만들고, standard에서 `timeline_width = ...max(72)`를 적용한다. 총 폭 100이면 inspector 32를 계산한 뒤 timeline 72를 강제하여 `actual_inspector=28`이 된다(101→29, 102→30, 103→31).
  - `src/app/mod.rs:2932-3005`는 매번 Path, 최대 5 Context, Policy, 두 Hint를 생성하며, `src/tui/layout.rs:1028-1073`에는 폭 기반 숨김·축약·wrap 정책이 없다. Path/Context 중략도 24열로 spec의 18열과 다르다.
  - `stitch_modern_tui_redesign/terminal_precision/DESIGN.md:115-121`은 폭 80 미만에서 secondary metadata를 제거하는 adaptive header를 요구한다.
- Expected Basis: `spec.md` Phase 15 success criteria와 concrete numbers, `redesign_plan.md` 2.2, Terminal Precision adaptive header.
- Actual: 100열 경계에서 Inspector min width를 보장하지 못하고, 좁은 화면에서 toolbar가 잘리거나 우선순위 없이 뒤쪽 칩을 버퍼 클리핑에 맡긴다. 80열 미만에는 Setup/Help/Approval 단일 컬럼 UX가 아니라 경고만 나온다.
- Impact: 경계 폭에서 탭·콘텐츠가 겹치거나 보이지 않으며, CWD/policy/hint가 핵심 입력을 가린다. 리사이즈 시 사용 가능한 UI가 갑자기 사라진다.
- Suggested Action: breakpoints와 width budget을 하나의 함수로 만들고 35열 Drawer/32~48열 split/Timeline 최소폭을 수식으로 검증한다. toolbar chip priority와 header metadata elision을 구현하고 80/90/99/100/103/104/120/140 폭 snapshot을 추가한다.
- Re-audit Method: TestBackend에서 지정 폭·높이를 순회해 Rect 합계, visible text, drawer/split 모드, chip elision을 확인한다. 실제 TTY 육안 확인은 후속 수동 검증으로 남긴다.
- Confidence: High

### [A04-F006] Compact Drawer의 마우스 좌표가 실제 렌더 위치와 달라 좁은 화면 Inspector scroll/focus가 Timeline으로 라우팅된다

- Pass: Debug / Engineering Quality
- Pattern: `DBG-002`, `TEST-001`
- Area: Mouse routing, overlay/drawer geometry
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: 렌더러는 `<100`에서 우측 32열 Drawer를 `x = total_width - 32`에 그리지만 mouse target 계산은 항상 split layout의 `timeline_width=max(total-30%,72)`를 사용한다. 80열에서 실제 Drawer 시작은 column 48인데 mouse handler는 column 72부터 Inspector로 분류한다.
- Evidence:
  - 실제 Drawer: `src/tui/layout.rs:114-127` (`drawer_width=32`, `x=...total_width-drawer_width`).
  - mouse 계산: `src/app/mod.rs:802-815`는 compact 여부를 분기하지 않고 `inspector_width=(term_cols*0.30).clamp(32,48)`, `timeline_width=...max(72)`를 적용한다.
  - 따라서 `term_cols=80`이면 실제 Drawer는 48~79열인데 `mouse.column=50..71`을 Timeline으로 반환한다. 클릭 focus/scroll 모두 잘못된 pane으로 간다.
  - `test_mouse_wheel_routing`은 `cfg!(test)`로 `(100,30)`을 강제하고 column 80을 Inspector로 검사한다(`src/app/mod.rs:2615-2624`, `src/tests/audit_regression.rs:2277-2357`). compact Drawer 좌표는 검증하지 않는다.
- Expected Basis: `redesign_plan.md:66-69`, `designs.md:1370-1383`의 pointer-over-pane routing.
- Actual: 80~99열 Drawer에서 보이는 Inspector를 스크롤하거나 클릭해도 Timeline이 움직일 수 있다.
- Impact: 좁은 화면에서 가장 중요한 fallback interaction surface가 오작동하며, 사용자에게 포커스가 이동했다는 시각 신호와 실제 상태가 어긋난다.
- Suggested Action: 렌더러와 동일한 `LayoutGeometry`/hit-test source of truth를 공유하고 compact drawer x/width를 mouse target이 그대로 사용하게 한다. 80/90/99 폭의 scroll·click 회귀 테스트를 추가한다.
- Re-audit Method: 80, 90, 99열에서 실제 Rect와 hit-test 결과를 같은 fixture로 비교하고 Drawer 내부/외부 각 column의 focus 및 scroll delta를 검증한다.
- Confidence: High

### [A04-F007] Unicode/ANSI/locale width 처리가 부분적으로만 적용되어 CJK·emoji 경로가 panic 또는 정렬 오류를 낼 수 있다

- Pass: Debug / Engineering Quality
- Pattern: `DBG-002`, `TEST-001`
- Area: Unicode width, long path/model, ANSI, accessibility rendering
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: Timeline 본문에는 `unicode-width` 래핑이 있지만 top bar/chip/toast의 중략·폭 계산은 byte/char count를 사용한다. 특히 `truncate_middle()`는 UTF-8 byte index를 직접 slice하여 CJK/emoji custom provider/model/path에서 panic할 수 있다. Timeline ANSI escape는 제거되지 않는다.
- Evidence:
  - `src/tui/layout.rs:31-38`은 `s.len()`과 `&s[..half]`, `&s[s.len()-half..]`를 사용한다. `src/tui/layout.rs:248-284`와 `:1055-1059`에서 cwd/provider/model/chip label에 호출된다. `half`가 UTF-8 code point 경계를 가르지 않으면 panic한다.
  - `src/tui/layout.rs:308-311`은 top-bar 폭을 `chars().count()`로 계산하고, `src/tui/layout.rs:164-171`의 toast 폭도 char count다. CJK/emoji 표시 셀 폭과 다르다.
  - `src/tui/layout.rs:366-393`의 width-aware helper는 Timeline 일부에만 쓰인다. `src/tui/layout.rs:451-467`, `:472-479`는 assistant/notice text의 ANSI를 제거하지 않은 채 `wrap_text_with_width`로 보낸다.
  - `src/tui/widgets/inspector_tabs.rs:299-317`만 로그에 제한적인 ANSI regex를 적용한다. OSC/hyperlink/다른 escape sequence는 대상이 아니다.
  - `src/tui/widgets/mod.rs:13-17`은 locale 문자열에 정확히 `UTF-8`이 포함되는지만 검사한다. 일반적인 `LANG=en_US.utf8`/`ko_KR.utf8`는 uppercase 후 `UTF8`이 되어 불필요하게 ASCII fallback으로 분류된다. 또한 `src/tui/widgets/mod.rs:30-32`의 기본 `PLAIN` border는 Terminal Precision이 요구한 rounded corner(`╭╮╰╯`)와 다르다(`stitch_modern_tui_redesign/terminal_precision/DESIGN.md:123-138`).
  - `CHANGELOG.md:311-316`, `spec.md:1938-1947`은 멀티바이트 slice panic을 이미 수정했다고 주장하지만 현재 top-bar 경로는 byte slice를 남겨 두고 있다.
  - 실행한 회귀 테스트에는 CJK/emoji long path, combining mark, ANSI/OSC buffer 테스트가 없다.
- Expected Basis: `spec.md:1938-1947`, `stitch_modern_tui_redesign/terminal_precision/DESIGN.md:106-113`의 strict character-cell grid 및 README의 `unicode-width` 안전성 주장.
- Actual: 일부 본문만 안전하고, 사용자 설정/경로가 들어가는 상단바·toolbar·toast에서 panic/clip/misalignment 가능성이 남아 있다.
- Impact: 비영어 locale과 긴 경로에서 앱이 panic하거나 핵심 상태가 잘려 보일 수 있다. 고대비/색상 외에도 문자 폭 자체가 접근성을 훼손한다.
- Suggested Action: `truncate_middle`와 모든 width budget을 `char_indices`/`UnicodeWidthStr` 기반으로 통일하고, 각 Span의 ANSI를 render 전에 정규화한다. escape sequence 정책을 명시하고 CJK/emoji/combining/ANSI fixture를 추가한다.
- Re-audit Method: 실제 문자열을 변경하지 않는 headless TestBackend fixture로 CJK/emoji/긴 경로/ANSI를 top bar, toolbar, toast, Timeline, tabs에 렌더링해 panic 없음·셀 폭·escape 미노출을 확인한다.
- Confidence: High

### [A04-F008] 5개 언어 i18n은 Questionnaire와 일부 탭에만 연결되고 locale 선택/영속화도 닫히지 않았다

- Pass: Implementation Compliance
- Pattern: `IMP-001`, `DOC-BACKFILL-001`
- Area: i18n coverage, locale selection/fallback, user-visible strings
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: `I18nManager` 사전은 존재하지만 실제 `tr()` 사용은 Timeline status, Inspector tab, Questionnaire로 제한된다. Wizard/Config/Help/Palette/Composer/Trust/Approval/오류 문구 대부분은 하드코딩되어 locale과 무관하게 영어·한국어가 섞여 나온다. 설정이 존재하면 `lang`이 기본 `en`으로 환경변수를 덮고, UI에서 언어를 바꾸거나 저장하는 경로도 없다.
- Evidence:
  - `redesign_plan.md:301-313`은 5개 언어의 정적 label 100% 매핑과 설정/`LANG` 동적 연동을 요구한다.
  - 실제 `tr()` 사용은 `src/tui/layout.rs:426-430`, `:845-929`, `src/tui/widgets/questionnaire.rs:66-171`에 집중되어 있다. 대표적인 미연결 문자열은 `src/tui/widgets/config_dashboard.rs:48-150`, `src/tui/widgets/setting_wizard.rs:25-176`, `src/tui/help_overlay.rs:24-121`, `src/tui/layout.rs:934-1110`, `:1340-1438`에 하드코딩되어 있다.
  - `src/app/state.rs:699-708`은 settings가 있으면 `settings.lang`만 사용하고 `LANG/LC_ALL`을 읽지 않는다. `src/domain/settings.rs:49-52`, `:142-145`의 serde 기본값은 `en`이다. `save_wizard_settings()`는 `src/app/wizard_controller.rs:199-216`에서 `..Default::default()`를 사용하며 locale 선택 UI가 없다.
  - 일본어 사전에도 `src/tui/i18n.rs:93-100`의 `badge_pending = "待機중"`처럼 한국어가 섞인 값이 있다.
  - `test_v3_9_0_i18n_key_completeness`는 정해진 19개 key가 다섯 map에 존재하는지만 검사하며 실제 call-site coverage, locale selection, rendered text를 검사하지 않는다(`src/tests/audit_regression.rs:4495-4535`).
- Expected Basis: `redesign_plan.md:301-313`, `spec.md:1788-1797`, README 5개 locale 및 TUI 기능 주장.
- Actual: locale을 선택해도 핵심 화면은 영어/한국어 문자열을 계속 사용하고, 기존 config가 있으면 환경 locale이 무시된다. 사전 완전성 테스트는 실제 UI 다국어 완성도를 보장하지 않는다.
- Impact: 5개 locale 지원 주장이 사용자-visible UI에서 성립하지 않으며, 접근성·발견성·키 hint 이해도가 locale마다 달라진다.
- Suggested Action: 사용자 노출 문자열 catalog를 단일 source로 만들고 모든 pane/modal/status/error/action title을 key로 전환한다. config/env 우선순위와 runtime locale 변경/영속화 경로를 명시하고, 누락 key·placeholder·각 renderer의 실제 출력 fixture를 검증한다.
- Re-audit Method: ko/en/ja/zh_TW/zh_CN 각각으로 전체 draw 경로를 렌더링해 원문 key·다른 언어 혼입·영어 fallback을 검색하고, config 존재/부재 및 `LANG` 조합에서 선택·저장 결과를 확인한다.
- Confidence: High

### [A04-F009] Setup Wizard가 문서상 Permission Preset/최종 검증 단계를 건너뛰고 정책을 고정 저장한다

- Pass: Implementation Compliance
- Pattern: `IMP-001`, `IMP-002`
- Area: Onboarding wizard, permission visibility, first-run accessibility
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: 문서는 Step 4 Permission Preset과 Step 5 Save & Verify를 요구하고 README는 wizard에서 permission policies를 설정한다고 설명하지만 실제 Wizard에는 Provider/Base URL/API Key/Model/Saving만 있다. 저장 시 shell/write/network가 고정값으로 채워져 사용자가 선택할 수 없다.
- Evidence:
  - `designs.md:494-546`은 Step 4 `Safe Starter/Balanced/Strict`와 Step 5 Save & Verify를 명시한다. `README.md:50-52`도 설정 마법사에서 권한 정책을 정의한다고 주장한다.
  - `src/app/state.rs:800-807`의 `WizardStep`에는 PermissionPreset/Verify 단계가 없다.
  - `src/tui/widgets/setting_wizard.rs:27-176`은 Step 1 provider, Step 2 URL/API key, Step 3 model, Saving만 렌더링한다.
  - `src/app/wizard_controller.rs:32-179`의 상태 전이에도 preset/verify가 없고, `:199-216`은 `ShellPolicy::Ask`, `FileWritePolicy::AlwaysAsk`, `NetworkPolicy::AllowAll`을 직접 저장한다.
  - `test_lm_studio_wizard_flow`는 provider/base URL/model fallback만 확인하고 정책 선택/최종 verify를 확인하지 않는다(`src/tests/audit_regression.rs:4419-4489`).
- Expected Basis: `designs.md:521-546`, `spec.md:291-325`의 sequential onboarding 및 README 설정 주장.
- Actual: 첫 실행 사용자는 보안 preset을 인지·선택하지 못하고, 문서와 다른 고정 정책으로 저장된다. `Safe Starter`라는 주석은 있지만 UI 계약으로 닫혀 있지 않다.
- Impact: 신규 사용자의 권한 mental model과 실제 설정이 어긋나며, 첫 실행의 안전성·접근성이 떨어진다. README의 onboarding 완료 주장을 재현할 수 없다.
- Suggested Action: preset/advanced/verify 단계를 실제 state·renderer·controller로 구현하거나 문서/README에서 현재 범위로 축소한다. 정책별 설명·현재값·다음 단계 hint를 locale catalog에 연결하고 저장 전 실제 config preview/verify를 제공한다.
- Re-audit Method: 신규 설정 fixture에서 각 preset을 선택하고 저장한 뒤 shell/write/network 값과 최종 화면/상태바를 비교한다. 저장 실패·취소 시 이전 설정 복구도 확인한다.
- Confidence: High

### [A04-F010] Questionnaire의 계획된 크기 계약과 옵션 cursor 동작이 다르고 긴 질문을 scroll하지 않는다

- Pass: Implementation Compliance / Debug / Engineering Quality
- Pattern: `IMP-001`, `TEST-001`
- Area: Questionnaire modal, keyboard navigation, small screens
- Severity: Minor
- Status: Confirmed
- Standard Disposition: Needs Fix / Needs Spec Clarification
- Summary: `redesign_plan`의 Questionnaire min/max 수치와 `designs.md`의 40/45% 문서가 서로 다르고, 실제 code는 min/max 일부만 적용한다. `↑/↓` cursor는 wrap-around하지 않고 clamp되며, 옵션/질문 제목이 modal 높이를 넘을 때 scroll 경로가 없다.
- Evidence:
  - `redesign_plan.md:364-375`는 width 60%, height 45%, width 50~90, height 15~25를 요구한다. `designs.md:1636-1648`은 height 40%를 적는다.
  - `src/tui/widgets/questionnaire.rs:41-51`은 60/45%와 min 40/10만 적용하고 max 90/25를 적용하지 않는다.
  - `src/app/mod.rs:3048-3061`은 cursor를 0/max에서 clamp하며 wrap-around하지 않는다. 설계의 wrap-around 요구는 `designs.md:1650-1659`에 있다.
  - `src/tui/widgets/questionnaire.rs:129-174`는 모든 option line을 한 번에 렌더링하고 scroll offset이 없다.
  - `test_v3_9_0_centered_rect_formula`는 120x40 및 30x8 Rect 계산만 확인하며 max clamp/긴 질문/키 wrap/render overflow는 확인하지 않는다(`src/tests/audit_regression.rs:4537-4570`).
- Expected Basis: `designs.md:1636-1668`, `redesign_plan.md:364-375`.
- Actual: 문서 수치의 권위가 불명확하고, 긴 option list는 아래가 잘릴 수 있으며, 마지막 option에서 Down을 눌러 첫 option으로 이동하지 않는다.
- Impact: Questionnaire가 긴/다국어 질문에서 답변을 잃게 하거나 선택 가능한 항목을 숨길 수 있다.
- Suggested Action: modal sizing authority를 하나로 정하고 max/min/available area를 함께 계산한다. 질문/옵션 content viewport와 scroll/visible cursor를 도입하고 wrap-around 여부를 문서에 맞춘다. width-aware rendered buffer test를 추가한다.
- Re-audit Method: 30x8, 80x24, 120x40 및 6개 이상 긴 CJK/emoji option fixture에서 title/option/hint가 모두 접근 가능하고 cursor가 보이는지 확인한다.
- Confidence: High for clamp/no-wrap; Medium for intended dimension authority.

### [A04-F011] Overlay render 순서와 input 우선순위가 공유되지 않아 Questionnaire가 보이는데 Help가 키를 소비할 수 있다

- Pass: Implementation Compliance / Debug / Engineering Quality
- Pattern: `IMP-001`, `DBG-002`
- Area: Modal/overlay precedence, questionnaire/help/palette/config/trust
- Severity: Minor
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: draw는 Questionnaire를 마지막에 그리지만 input은 Help를 Questionnaire보다 먼저 처리한다. 비동기 `ShowQuestionnaire`가 Help가 열린 상태에서 도착하면 화면상 질문 modal이 Help 위에 보이는데 Up/Down/문자 입력은 Help에 소비된다. 다른 overlay도 중앙 stack/상호 배제가 없다.
- Evidence:
  - `src/tui/layout.rs:154-221`은 Config → Trust → Palette → Toast → Help → Questionnaire 순으로 그린다.
  - `src/app/mod.rs:1929-1945`는 Help를 Questionnaire보다 먼저 intercept하고 return한다. `src/app/mod.rs:1940-1945`의 Questionnaire handler는 Help가 닫힌 뒤에야 실행된다.
  - `src/app/mod.rs:1558-1579`의 `ShowQuestionnaire`는 `questionnaire`만 설정하고 `show_help_overlay`, palette/config, `focused_pane`를 정리하지 않는다.
  - `src/app/mod.rs:1947-1952`는 Palette/Inspector 등 overlay 상태와 관계없이 F1/`?`로 Help를 추가로 열 수 있다.
- Expected Basis: `designs.md:103-105`, `designs.md:1638-1668`의 overlay 우선순위와 Questionnaire 중 입력 차단 규칙.
- Actual: 보이는 최상단 modal과 실제 키 입력 consumer가 달라질 수 있다. 사용자는 질문에 답하려고 눌렀는데 Help가 닫히거나 아무 변화가 없다.
- Impact: 비동기 상태 전환에서 modal dead-end/혼란이 발생하고, approval/questionnaire 입력의 신뢰성이 낮아진다.
- Suggested Action: overlay priority를 enum/state machine으로 단일화하고 render와 input이 같은 topmost overlay를 사용하게 한다. Questionnaire/Trust/Approval은 생성 시 하위 overlay를 닫거나 명시적으로 stack한다.
- Re-audit Method: Help/Palette/Config가 열린 상태에서 `ShowQuestionnaire`/Trust/Approval action을 주입하고, 화면상 topmost overlay와 실제 key dispatch가 일치하는지 fixture로 확인한다.
- Confidence: High

### [A04-F012] TerminalGuard가 부분 초기화 실패와 panic에서 커서/alternate screen 복원을 완전히 보장하지 않는다

- Pass: Debug / Engineering Quality
- Pattern: `DBG-001`, `TEST-001`
- Area: terminal restore/panic safety
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: `TerminalGuard::init()`이 raw mode를 켠 뒤 Guard가 생성되기 전에 alternate screen/mouse capture/Terminal 생성이 실패하면 raw mode를 복원할 소유 객체가 없다. panic hook와 restore 함수도 cursor Show 및 단계별 실패를 보장하지 않는다.
- Evidence:
  - `src/tui/terminal.rs:20-27`은 `enable_raw_mode()` 후 `execute!`와 `Terminal::new()`를 호출하고, 성공한 뒤에야 `TerminalGuard`를 반환한다. 중간 `Err`에는 Drop이 실행되지 않는다.
  - `src/tui/terminal.rs:62-67`의 `restore_terminal()`은 `disable_raw_mode()?`가 실패하면 `LeaveAlternateScreen`/`DisableMouseCapture`를 실행하지 않는다.
  - `src/tui/terminal.rs:69-80` panic hook는 raw mode/alternate/mouse만 복구하고 `cursor::Show`를 보내지 않는다.
  - `designs.md:945-950`, `spec.md:1063-1070`은 panic/Ctrl+C/validation error 후 raw mode, alternate screen, cursor 복구를 요구한다.
  - 현재 검사 범위에서는 물리 TTY를 사용할 수 없고 terminal failure injection 테스트도 없다.
- Expected Basis: `designs.md:936-950`, `spec.md:340-344`의 terminal cleanup 계약.
- Actual: 초기화/복구 중 I/O 오류 또는 panic 시 사용자의 터미널이 raw/alternate 상태나 hidden cursor로 남을 수 있다.
- Impact: 앱 종료 후 터미널을 수동 `reset`해야 하거나 입력이 보이지 않는 복구 불능 UX가 된다.
- Suggested Action: 단계별 RAII guard/rollback을 도입해 각 성공 단계의 inverse를 항상 실행하고, panic/Drop/normal restore에 cursor Show를 포함한다. 복구 명령의 오류를 독립적으로 시도하고 TTY-independent failure-injection 테스트를 추가한다.
- Re-audit Method: enable/execute/Terminal::new 각각의 실패를 mock/failure injection으로 만들고 raw/alt/mouse/cursor inverse가 모두 호출되는지 확인한다. 실제 터미널 수동 검증은 별도 게이트로 남긴다.
- Confidence: High for partial-init path; Medium for terminal-specific observable outcome because physical TTY was excluded.

### [A04-F013] Diff cache가 설계된 Path/timestamp cache가 아니며 theme 전환 후 stale style을 재사용한다

- Pass: Implementation Compliance / Debug / Engineering Quality
- Pattern: `IMP-003`, `TEST-001`
- Area: Diff rendering performance and high-contrast theme
- Severity: Minor
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: 계획은 PathBuf+modified timestamp 기반 `DiffLineCache`와 5000-line hit path를 요구하지만 구현은 thread-local 단일 `Option<DiffCache>`를 원문 문자열 하나로 비교한다. cache hit 시 이전 palette로 스타일된 Line을 clone하므로 `/theme` 전환 후 Diff가 이전 색상을 유지한다.
- Evidence:
  - `redesign_plan.md:140-147`, `:349-354`는 파일 경로·timestamp 기반 HashMap cache를 명시한다.
  - `src/tui/widgets/inspector_tabs.rs:31-46`은 `last_diff_text: String`과 `cached_lines` 하나만 thread-local에 보관한다. `:151-186`은 diff text equality만 검사하고 hit 시 `cached_lines.clone()`을 반환한다.
  - `render_diff`의 palette는 `src/tui/widgets/inspector_tabs.rs:148-176` cache miss에서만 색상에 사용된다. `/theme`은 `src/app/command_router.rs:473-490`에서 settings를 바꾸지만 cache를 무효화하지 않는다.
  - Timeline diff 줄도 `src/tui/layout.rs:606-610`에서 `Color::Green`/`Color::Red`를 직접 사용한다. `audit_roadmap.md:588-599`가 주장하는 하드코딩 색상 제거와 semantic/high-contrast palette 적용이 이 경로에서는 닫히지 않는다.
  - `test_diff_render_cache_invalidation`은 diff text 변경만 시뮬레이션한다(`src/tui/widgets/inspector_tabs.rs:636-718`). theme/width/large diff frame 비용은 검증하지 않는다.
- Expected Basis: `redesign_plan.md` Task 3.2, `designs.md:1073-1105`의 실시간 high-contrast 전환.
- Actual: cache 구조·무효화 계약이 문서와 다르고, 같은 diff를 high-contrast 전환 뒤 보면 이전 palette style이 남을 수 있다.
- Impact: 접근성 테마 전환이 Diff pane에서 즉시 반영되지 않으며, 5000-line 성능 성공 기준도 직접 증명되지 않는다.
- Suggested Action: cache key에 canonical diff identity/version과 palette/theme 또는 style-independent parsed representation을 포함하고, clone 비용·width/viewport 정책을 명시한다. theme switch/large diff benchmark 또는 deterministic render test를 추가한다.
- Re-audit Method: 동일 diff를 default→high_contrast로 전환해 cell style을 비교하고, 서로 다른 파일/동일 text 및 수정 timestamp fixture에서 cache hit/invalidation을 확인한다.
- Confidence: High

### [A04-F014] 상태바의 성공 상태와 좁은 화면 metadata가 실제 상태를 반영하지 않는다

- Pass: Implementation Compliance
- Pattern: `IMP-001`
- Area: Status bar, accessibility/status semantics, toolbar
- Severity: Minor
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: 상태바 마지막 상태 glyph가 설정·연결·저장·오프라인 여부와 무관하게 항상 `✓`이며, `Setup Required`, `Offline`, `ReadOnly`, `Network Deny` 같은 설계 상태가 없다. toolbar도 policy/CWD/hint를 항상 생성해 폭 clipping에 맡긴다.
- Evidence:
  - `designs.md:109-139`는 상태바에 연결/저장 상태와 `No Provider`, `Setup Required`, `Network Deny`를 표시하도록 한다.
  - `src/tui/layout.rs:304-306`은 조건 없이 `" · ✓ "`를 추가한다. `:229-258`은 설정이 없을 때 provider/model/policy를 `None`으로 만들지만 success glyph는 바뀌지 않는다.
  - `src/app/state.rs:749-759`는 config load error를 wizard/log에 기록하지만 상태바 상태에는 연결하지 않는다.
  - `src/app/mod.rs:2932-3005`는 compact width와 무관하게 Path/Policy/Hint 칩을 만들고, `src/tui/layout.rs:1068-1073`은 한 줄 Paragraph에 그대로 렌더링한다.
- Expected Basis: `designs.md` status bar/toolbar 규칙, `stitch_modern_tui_redesign/terminal_precision/DESIGN.md:115-121` adaptive header.
- Actual: 미설정/오류 상태에서도 정상 저장·연결처럼 보이며, 폭이 좁으면 상태 우선순위가 없는 clipping으로 핵심 hint가 사라진다.
- Impact: 사용자 판단과 실제 trust/config/runtime 상태가 어긋나고, 색상에 의존하지 않는 상태 전달 원칙을 약화한다.
- Suggested Action: runtime status enum을 상태바 source of truth로 만들고 `✓/!/Offline/ReadOnly/Setup Required`를 명시적으로 계산한다. toolbar chip priority/visible width를 계산해 compact에서는 secondary chip을 제거한다.
- Re-audit Method: no settings, config error, offline, denied/restricted, high context fixture를 80/100/140열에 렌더링해 텍스트·색·숨김 우선순위를 확인한다.
- Confidence: High

### [A04-F015] Command Palette/Slash Menu의 command catalog가 실제 router와 동기화되지 않는다

- Pass: Implementation Compliance
- Pattern: `IMP-003`, `IMP-004`
- Area: Command discovery, palette/Slash Menu parity
- Severity: Minor
- Status: Confirmed
- Standard Disposition: Needs Documentation Recovery / Needs Fix
- Summary: 실제 router가 지원하는 `/mcp`와 `/undo`가 Palette와 Slash Menu에 없고, `/help`만 이 명령을 설명한다. Palette fuzzy matching도 command id를 검색 대상에 포함하지 않는다.
- Evidence:
  - `src/app/command_router.rs:638-770`에는 `/mcp`와 `/undo`가 실제로 라우팅된다.
  - `src/app/state.rs:380-489`의 `CommandPaletteState::new()`에는 `/mcp`, `/undo`가 없다. `src/app/state.rs:941-957`의 Slash Menu 16개에도 없다.
  - `src/app/command_router.rs:387-393`의 `/help`에는 MCP/Undo 설명이 있어 catalog source가 분리되어 있다.
  - `src/app/mod.rs:2822-2861`은 Palette match target을 title/category로만 만들며 `cmd.id`는 검색하지 않는다. `/mcp`·`/undo`를 직접 검색해도 id 자체로는 찾을 수 없다.
  - `test_ctrl_k_command_palette`는 열기/닫기만 검증하고 catalog parity/fuzzy id 검색은 검증하지 않는다(`src/tests/audit_regression.rs:2095-2131`).
- Expected Basis: `spec.md:1564-1566`, `:1786-1787`의 Command Palette primary entry와 `/help` synchronization, `designs.md:631-637` 자동완성 규칙.
- Actual: 사용자에게 보이는 primary discovery surface에서 실제 명령 일부가 누락되고, `/help`와 Palette/Slash Menu가 서로 다른 목록을 가진다.
- Impact: 안전한 `/undo`, MCP 관리 및 기타 기능을 사용자가 발견하기 어렵고, palette가 primary command catalog라는 설계가 깨진다.
- Suggested Action: router/help/palette/slash menu가 공유하는 command registry를 만들고 mode/permission별 availability를 계산한다. id/title/description 모두 fuzzy 대상에 넣고 catalog parity 테스트를 추가한다.
- Re-audit Method: router의 모든 user-facing command를 수집해 Palette/Slash Menu/Help에 존재하는지 비교하고, command id·title·한글 description query 각각의 match/execute 결과를 검증한다.
- Confidence: High

### [A04-F016] Inspector 콘텐츠가 Preview/Search/Recent 설계 계약보다 얕아 핵심 데이터 소비가 끊긴다

- Pass: Implementation Compliance
- Pattern: `IMP-001`, `IMP-002`
- Area: Inspector data consumption and rendered semantics
- Severity: Major
- Status: Confirmed
- Standard Disposition: Needs Fix
- Summary: Preview에는 line number/파일 범위/Enter 승격이 없고, Search는 grep 파일 그룹·match line·context 대신 Timeline block text를 단순 검색하며, Recent는 최근 파일/grep/diff/명령이 아닌 최근 ToolRun만 보여준다. Diff도 pending approval만 소비한다.
- Evidence:
  - `designs.md:304-343`은 Preview line number/선택 구간, Search 파일 그룹·match/context·Enter Preview, Recent 파일/grep/diff/명령 재진입을 요구한다.
  - `src/tui/widgets/inspector_tabs.rs:53-141`의 Preview는 block body를 그대로 출력하고 line number/target action이 없다.
  - `src/tui/widgets/inspector_tabs.rs:334-451`의 Search는 `composer.input_buffer`와 `ui.timeline` 텍스트만 사용하고, 결과도 block 번호/label/60열 preview다.
  - `src/tui/widgets/inspector_tabs.rs:474-543`의 Recent는 `TimelineBlockKind::ToolRun`만 수집한다.
  - `src/tui/widgets/inspector_tabs.rs:148-208`의 Diff source는 `runtime.approval.diff_preview` 하나이며 최근 적용 diff 이력 source가 없다.
- Expected Basis: `designs.md:304-357`, `spec.md:1773-1797`의 Inspector workspace data contract.
- Actual: 탭 이름은 있지만 설계가 요구한 파일/grep/diff/session 데이터와 후속 행동을 소비하지 않는다.
- Impact: 사용자가 승인 전 근거를 검토하거나 검색 결과에서 파일 문맥으로 이동하는 핵심 workflow가 닫히지 않는다.
- Suggested Action: 각 탭의 canonical data source와 action contract를 명시하고 Preview/Search/Recent/Diff에 실제 selection/reentry/history를 연결한다. 구현하지 않는 계획은 phase/deferred로 문서화한다.
- Re-audit Method: ReadFile/Grep/Diff/Tool/session fixture를 주입해 각 탭의 목록·group/context·selection·Enter 후 전환·scroll 결과를 확인한다.
- Confidence: High

## 6. Uncertainties and Clarifications Needed

- Approval의 일반 키 계약이 `Enter/Esc`인지 MCP 예시의 `y/n`인지 문서 권위를 먼저 확정해야 한다. 현재 실제 구현은 y/n만 처리한다.
- Inspector 탭이 5개(Preview/Diff/Logs/Search/Git 또는 Recent 포함)인지 6개(Recent+Git)인지 `designs.md`, `redesign_plan.md`, `spec.md`, 실제 enum이 서로 다르다.
- Questionnaire 높이 규격은 `designs.md` 40%와 `redesign_plan.md` 45%가 충돌하며, min/max 수치의 authority도 확정해야 한다.
- RTL 지원은 요구 문서가 확인되지 않아 기능 부재를 finding으로 창작하지 않았다. 다만 실제 렌더링 검증은 하지 못했다.
- 물리 터미널별 rounded border, combining mark, OSC/hyperlink 처리 결과는 headless static evidence만으로 최종 PASS할 수 없다.

## 7. Perspective Decision

`HOLD` — A04 관점에서 Major finding이 여러 개 남아 있고, approval safety CTA, keyboard-only Timeline 접근, responsive geometry/drawer hit-test, locale coverage, first-run permission UX, terminal cleanup이 구현·문서·테스트로 닫히지 않았다. 기존 targeted state tests가 통과했지만 실제 화면 버퍼/폭/locale/overlay 증거가 없어 TUI 완성도 PASS로 해석할 수 없다.

### Required Fixes Before A04 PASS

1. Approval Enter/Esc/y/n 계약 확정 및 실제 routing/포커스/회귀 테스트 정렬
2. keyboard focus graph와 Inspector CTA/Alt/mouse action 구현 또는 문서상 phase 경계 명시
3. Timeline 단일 소비 경로와 `/status`/`/clear` visible state 정합화
4. responsive geometry/hit-test source of truth 및 80/100/120/140 buffer snapshots
5. UTF-8/Unicode width/ANSI/ASCII fallback 경로의 공통 helper와 CJK/emoji tests
6. 전체 UI i18n catalog 및 locale 선택·영속화, Wizard permission preset/verify 구현
7. overlay stack/terminal restore failure path와 Questionnaire viewport/cursor 계약 정렬

### Coder Handoff

`/mnt/Projects_SSD/rust/smlcli/docs/multi_audit/1/sub_audit_04_tui_ux_i18n.md`를 먼저 읽고, 각 finding을 현재 프로젝트 문서와 실제 코드에 대조하여 우선순위대로 수정하세요. 계약 변경이 필요하면 관련 문서를 먼저 갱신하고, 수정 후 키 상태 전이·TestBackend 화면·locale·responsive·terminal cleanup 검증과 A04 재감사 증거를 기록하세요.
