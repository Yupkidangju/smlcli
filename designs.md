# smlcli designs.md
Version: v0.1 BETA

## 0. 문서 목적

이 문서는 `smlcli`의 TUI 기준 UI/UX 설계를 정의한다.  
목표는 다음 3가지를 동시에 만족하는 것이다.

1. 처음 실행한 사용자가 막히지 않는 설정 흐름
2. 매일 쓰는 CLI 사용자에게 빠르고 예측 가능한 키보드 UX
3. 파일 수정, shell 실행, grep, diff 승인 같은 고위험 동작의 높은 가시성

본 설계는 기존 `smlcli spec`의 구조와 정책을 유지하면서도, 더 사용하기 쉽고 CLI에 어울리는 형태로 정보 구조를 단순화한 최종안이다.

---

## 1. 디자인 원칙

### 1.1 Terminal First
마우스 없이 사용할 수 있어야 한다. 모든 주요 동작은 키보드만으로 3단계 이내에 도달 가능해야 한다.

### 1.2 Conversation Is Primary
사용자의 주 시선은 항상 중앙 타임라인에 머물러야 한다. 상태 정보와 부가 패널은 이를 방해하지 않아야 한다.

### 1.3 Inspect When Needed
파일 프리뷰, grep 결과, diff, shell 로그는 항상 열어두지 않는다. 필요할 때만 Inspector를 열어 문맥을 확장한다.

### 1.4 Approval Must Be Visible
쓰기, shell 실행, 권한 변경 같은 위험 동작은 “무엇이 바뀌는지”가 먼저 보이고 “승인”은 그 다음이어야 한다.

### 1.5 One Mental Model
설정, 검색, 수정, 실행, 승인까지 모두 같은 인터랙션 규칙을 따라야 한다.
- `Enter`: 진행 / 확정
- `Esc`: 뒤로 / 닫기 / 취소
- `Tab`: 작업 영역 전환
- `/`: 명령 진입
- `@`: 파일 문맥 추가
- `!`: 셸 실행

---

## 2. 정보 구조 재설계

기존 스펙의 “왼쪽 상태 패널 + 중앙 대화 패널 + 오른쪽 작업 패널 + 하단 입력창” 4분할 고정 구조는 기능적으로는 풍부하지만, 초반 인지 부하가 크다.

최종안은 아래 4개 블록으로 단순화한다.

1. **상단 상태바**
2. **중앙 타임라인**
3. **우측 Inspector**
4. **하단 Composer**

핵심 차이는 다음과 같다.

- 상태 패널은 고정 좌측 컬럼이 아니라 **1줄 상태바**로 축약한다.
- 작업 패널은 항상 보이지 않고 **필요할 때만 열리는 Inspector**로 통합한다.
- 중앙 대화 패널은 단순 채팅이 아니라 **작업 로그 + 승인 흐름 + 응답**이 합쳐진 타임라인으로 정의한다.
- 입력창은 단순 프롬프트 박스가 아니라 **명령, 파일 참조, 셸 실행, 권한 힌트**를 담는 Composer로 확장한다.

---

## 3. 레이아웃 정의

## 3.1 기본 레이아웃

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ smlcli · OpenAI/gpt-5 · /workspace/app · PLAN · Shell Ask · 61% ctx · ✓    │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Timeline                                                                    │
│  ─────────────────────────────────────────────────────────────────────────   │
│  User: auth flow 설명해줘                                                    │
│  AI  : src/auth, src/session, config/security를 확인했습니다.                │
│        다음 파일을 읽었습니다...                                              │
│                                                                              │
│  Tool Summary                                                                │
│  ReadFile 3 · Grep 1                                                         │
│                                                                              │
│  Proposed Change                                                             │
│  settings.rs  +12 -4                                         [Preview]      │
│                                                                              │
│  Shell Request                                                               │
│  cargo test --lib                                          [Approve] [Deny] │
│                                                                              │
│                                                                              │
├──────────────────────────────────────────────────────────────┬───────────────┤
│ Composer                                                     │ Inspector     │
│ /, @, ! 사용 가능                                            │ Diff / Preview│
│ > @src/auth/mod.rs 로그인 흐름 요약해줘                      │ Search / Logs │
└──────────────────────────────────────────────────────────────┴───────────────┘
```

## 3.2 레이아웃 원칙

### 기본 상태
- Inspector는 닫혀 있어도 된다.
- 사용자가 일반 대화만 할 때는 타임라인과 Composer만 넓게 사용한다.

### 작업 상태
- grep, diff, file preview, shell output이 발생하면 Inspector가 자동으로 열린다.
- Inspector는 사용자의 명시적 닫기 전까지 유지된다.

### 승인 상태
- diff 승인, shell 승인, 손상 복구, `/setting` 저장 전 확인은 모두 modal이 아니라 **타임라인 카드 + Inspector 상세 보기** 조합을 우선 사용한다.
- 정말로 즉시 응답이 필요한 경우에만 modal을 띄운다.

---

## 4. 상단 상태바 설계

## 4.1 표시 정보

상단 상태바는 1줄 고정이며 항상 다음 정보를 담는다.

- 앱 이름: `smlcli`
- provider/model: `openai/gpt-5`
- working directory: `/workspace/my-app`
- current mode: `PLAN` 또는 `RUN`
- shell permission: `Ask`, `SafeOnly`, `Deny`
- context budget: `61%`
- 연결/저장 상태: `✓`, `!`, `Offline`, `ReadOnly`

예시:

```text
smlcli · anthropic/claude-sonnet-4 · ~/project · RUN · Shell Ask · 84% ctx · ✓
```

## 4.2 상태 강조 규칙

- 정상 상태는 저강도 표시
- 주의 상태는 텍스트 강조
- 위험 상태는 점멸 대신 굵은 라벨 사용

예:
- `ctx 85%` 이상: `ctx 87%` 경고
- provider 미설정: `No Provider`
- 설정 미완료: `Setup Required`
- 네트워크 차단: `Network Deny`

## 4.3 클릭이 아닌 키보드 중심 동작

상태바는 클릭 중심 UI가 아니다. 대신 단축 진입점을 제공한다.

- `Ctrl+P`: provider/model 빠른 전환
- `Tab`/`Shift+Tab`: PLAN ↔ RUN 모드 전환
- `Ctrl+R`: permissions 보기
- `F2`: Inspector 토글 (※ `Ctrl+I`는 터미널에서 Tab과 동일한 0x09이므로 사용 불가)

---

## 5. 중앙 타임라인 설계

## 5.1 타임라인이 채팅창과 다른 점

중앙 영역은 단순한 “유저/AI 대화”가 아니다. 아래 요소를 시간 순서대로 함께 보여준다.

- 사용자 메시지
- AI 응답
- tool 실행 요약
- 에러
- 승인 대기 카드
- 완료 결과
- 세션 compact 안내
- 복구/재시도 제안

즉, `smlcli`의 메인 화면은 “채팅 화면”이 아니라 “작업 진행 기록 화면”이다.

## 5.2 카드 타입

### User Message
- 한 줄 요약 + 확장 가능
- 파일 참조가 있으면 chip 형태로 표시

예:
```text
User
@src/main.rs @Cargo.toml 빌드 실패 원인 찾아줘
```

### AI Message
- 서술보다 행동 요약 우선
- 긴 답변은 3~5줄 미리보기 후 접기

### Tool Summary
- 세부 로그 전체를 본문에 펼치지 않는다
- 예:
  - `ReadFile 2`
  - `Grep "AuthError" in src/`
  - `Diff ready for settings.rs`

### Approval Card
- 가장 중요한 카드
- 제목, 영향 범위, 대상 파일/명령, 위험도, 승인 버튼 노출
- 상세 내용은 Inspector에서 확인

### Error Card
- 실패 이유 + 다음 액션 제안
- 예:
  - `API key validation failed`
  - `Retry`
  - `Edit provider`
  - `Open logs`

### Session Notice
- context compact, reconnect, restore session, config corruption recovery 같은 시스템 알림

## 5.3 타임라인 밀도 규칙

- 본문은 80~100 컬럼 기준으로 자연스럽게 읽혀야 한다.
- 도구 로그 원문은 가능한 한 접는다.
- shell stdout/stderr는 본문에 전체 노출하지 않고 “스트리밍 중 / 완료 / 실패” 상태만 먼저 보여준다.

## 5.4 도구 호출 JSON 필터링 (v0.1.0-beta.16)

AI 응답 내 도구 호출 JSON 스키마(```json ... ```)는 사용자에게 가시화하지 않는다.
타임라인 렌더링 시 `filter_tool_json()` 함수로 필터링하여 사용자 친화적 메시지로 대체:

```
⚙️  [ExecShell] 도구 호출 실행 중...
   ↳ $ ls -al
```

- 도구명 표시 필수, 명령어/경로 등 핵심 파라미터만 간략 표시.
- JSON 파싱 실패 시 원문 그대로 표시 (안전 폴백).

## 5.5 AI 추론 인디케이터 및 tick 기반 애니메이션 (v0.1.0-beta.18 개편)

프롬프트 전송 후 AI 응답 수신까지 타임라인 하단에 추론 상태를 표시:

```
◐ AI가 응답을 생성하고 있습니다...
```

- tick 기반 스피너: `◐ ◓ ◑ ◒` (tick_count % 4)
- `is_thinking` 플래그로 제어: `dispatch_chat_request()` 시 `true`, 스트리밍 전체 기간 동안 유지되어 사용자 인터럽트 차단, `ChatResponseOk/Err` 수신 시 비로소 `false`.
- 매 틱(250ms) UI 리렌더링으로 실시간 반영.

추가 애니메이션 요소:

| 상태 | 애니메이션 | 주기 |
|------|------------|------|
| AI 추론 중 | `◐ ◓ ◑ ◒` 스피너 | tick % 4 |
| 도구 실행 중 | `●` / `○` 배지 깜빡임 | tick % 2 |
| diff 승인 대기 | 앞경색 subtle pulse | tick % 6 |
| context compact | `▪▪▪▫▫` 진행 | tick % 5 |

## 5.6 도구 출력 요약 분리 (v0.1.0-beta.18 개편)

ToolFinished 수신 시 raw stdout/stderr를 타임라인에 전체 노출하지 않는다.

- 타임라인: 2~4줄 요약만 표시 (ToolSummaryReady)
- 원문: `logs_buffer`에 push → Inspector Logs 탭에서 전체 확인

예시:
```
✅ ExecShell 완료 (exit 0)
   cargo test ─ 14 passed, 0 failed
   [Logs 탭에서 전체 출력 확인]
```

## 5.7 Tree of Thoughts & Auto Verify (v0.1.0-beta.23 개편)

에이전트 자율 모드가 도입되면서, AI가 단일 턴에 다수의 도구를 연쇄 호출하거나, 에러 발생 시 자가 복구를 시도할 수 있게 되었다. 이를 시각적으로 명확히 전달하기 위해 타임라인 내에 계층화(Depth) 인덴트(`└─`)를 적용한다.

- `TimelineBlock` 구조체에 `depth` 속성을 부여하여 렌더링 시 여백을 자동 생성.
- 도구 호출(`ToolCard`) 및 자동 검증 알림(`SystemNotice`)은 부모 AI 메시지에 종속된 형태로 표시.
- 입력 의도 분류는 참고 신호만 제공하며, 모델이 구조화된 `tool_calls`를 반환하면 런타임은 이를 차단하지 않고 실행 파이프라인으로 전달한다.
- Auto-Verify는 UI 카드용 짧은 요약과 별도로, 모델 재전송에는 더 긴 실패 원문을 보존하여 컴파일 에러 후반부 문맥을 잃지 않도록 한다.

**시각적 예시:**
```text
AI:
[ReplaceFileContent] 도구를 사용하여 src/main.rs를 수정합니다.

  └─ ◻ ReplaceFileContent 권한 검사 중...
  └─ ✅ ReplaceFileContent 완료 (exit 0)

  └─ ℹ  [Auto-Verify: Healing] The previous tool execution failed. Please review...
  └─ ◻ ExecShell 권한 검사 중...
```

---

## 6. Inspector 설계

Inspector는 우측 보조 패널이며, 필요 시 자동 오픈되는 작업 집중 영역이다.

## 6.1 탭 구성

Inspector는 탭 기반으로 구성한다.

- `Preview`
- `Diff`
- `Search`
- `Logs`
- `Recent`

## 6.2 Preview 탭
파일 미리보기 전용
- 읽기 전용
- line number 제공
- 긴 파일은 헤더 + 선택 구간 + 주변 문맥만 노출
- `Enter`: 해당 파일을 현재 작업 대상으로 승격

## 6.3 Diff 탭
쓰기 승인 전 핵심 탭
- unified diff 기본
- 짧은 diff는 inline 하이라이트
- 긴 diff는 hunk 단위 접기
- 파일 단위 승인/거절 가능
- 상태 텍스트 예시:
  - `settings.rs +12 -4`
  - `lib.rs +1 -0`
  - `2 files pending`

## 6.4 Search 탭
grep 결과용
- 파일별 그룹화
- match line과 주변 문맥 표시
- 방향키 이동
- `Enter`: Preview로 전환
- `/`: 검색어 수정

## 6.5 Logs 탭
shell / provider / validation 로그
- shell 출력 스트리밍
- provider 연결 실패 상세 정보
- config 손상 감지 로그
- raw 출력은 색보다는 구조로 구분

## 6.6 Recent 탭
최근 작업 대상을 재진입하기 위한 탭
- 최근 연 파일
- 최근 grep
- 최근 diff
- 최근 명령

## 6.7 Inspector 탭 실체 구현 명세 (v0.1.0-beta.18 개편)

**[v0.1.0-beta.20 갱신]** `widgets/inspector_tabs.rs`에 Logs, Search, Recent 탭이 실제 구현 완료.
Preview와 Diff는 승인 카드 연동으로 동작하며, 추가 콘텐츠는 향후 확장 예정.

구현 요구사항 및 현황:

| 탭 | 데이터 소스 | 렌더링 | 구현 상태 |
|----|-------------|--------|----------|
| Preview | 현재 작업 대상 파일 (최근 ReadFile 결과) | line number + 일부 범위 표시 | 기본 구현 |
| Diff | `approval.diff_preview` + 최근 적용 diff 이력 | +/- 색상 강조 | 승인 카드 연동 완료 |
| Search | `timeline` 전체 텍스트 검색 | Composer 입력 기반 대소문자 무시 필터링, 최대 50건 | ✅ 구현 완료 (beta.20) |
| Logs | `logs_buffer` (shell/provider 원문) | 스크롤 가능한 raw 출력, 최근 100줄 | ✅ 구현 완료 (beta.18) |
| Recent | 최근 실행된 도구 요약 목록 | 상태 아이콘 + 도구명 + 요약, 최근 10건 | ✅ 구현 완료 (beta.18) |

---

## 7. Composer 설계

Composer는 단순 입력 박스가 아니다. `smlcli`의 주된 명령 인터페이스다.

## 7.1 기본 형태

```text
> @src/auth/mod.rs 로그인 로직 설명해줘
```

아래 힌트는 비상시가 아니라 평상시에도 보인다.

```text
/, @, ! 사용 가능 · Tab mode 전환 · Esc 취소
```

## 7.2 입력 해석 규칙

### 일반 텍스트
기본 프롬프트로 처리

### `/`
slash command 자동완성 시작

**[v0.1.0-beta.20 갱신]** 빈 Composer에서 `/` 입력 시 Composer 위에 자동완성 팝업이 활성화됨.
- 12개 내장 명령어 목록이 표시됨: `/config`, `/setting`, `/provider`, `/model`, `/status`, `/mode`, `/tokens`, `/compact`, `/theme`, `/clear`, `/help`, `/quit`
- 키보드 입력으로 부분 일치 필터링
- `↑`/`↓` 방향키로 커서 이동, `Enter`로 선택 즉시 실행
- `Esc` 또는 필터를 모두 지운 후 `Backspace`로 메뉴 닫기

### `@`
파일 fuzzy finder 진입

### `!`
shell command 모드 진입

## 7.3 멀티라인 정책

- 기본은 1줄
- 길어지면 위로 확장
- `Shift+Enter`: 줄바꿈
- `Enter`: 제출

## 7.4 인라인 상태 힌트

Composer 우측 끝에는 현재 실행 맥락이 보인다.

예:
- `PLAN`
- `RUN`
- `FILES 2`
- `SHELL ASK`
- `DIFF 1`

---

## 8. Setup Home 및 `/setting` Wizard 재설계

`/setting`은 단순한 설정 폼이 아니라 “첫 실행을 막힘 없이 통과시키는 onboarding flow”여야 한다.

## 8.1 Setup Wizard (Sequential UX)

설정 미완료 상태에서는 빈 대화 화면 대신 마법사를 자동으로 실행한다.
별도의 Home 단계 없이 즉시 **순차적 입력 단계(Step 1 -> Step 2 -> Step 3)**로 돌입하여, 키보드 화살표만으로 모든 초기 셋업을 완수하도록 디자인을 고도화했다.

예외 상황 발생 시 바로 Esc를 눌러 설정을 재시작할 수 있다.
설정 파일(`config.toml`)이 손상된 경우에도 조용히 빈 상태로 시작하지 않고, Step 1 화면에서 복구/삭제 가이드를 즉시 노출한다.

### 8.1.1 Phase 17: Workspace Trust Gate

구현 전 동결하는 추가 UX다. 설정 마법사와 별도로, **현재 작업 루트(workspace root)** 에 대한 신뢰 여부를 먼저 확인한다.

```text
┌ Workspace Trust ───────────────────────────────────────────────┐
│ 현재 작업 루트: C:\Users\me\Projects\demo                     │
│ Host: cmd.exe   Exec: pwsh                                    │
│                                                               │
│ 이 폴더를 신뢰하시겠습니까?                                   │
│                                                               │
│ > Trust Once                                                  │
│   Trust & Remember                                            │
│   Restricted (read-only)                                      │
│                                                               │
│ Restricted에서는 읽기 전용 도구만 허용됩니다.                │
└───────────────────────────────────────────────────────────────┘
```

**동결 규칙**
- Trust Gate는 앱 시작 직후, 메인 타임라인보다 먼저 노출된다.
- 선택 전까지 Composer 입력은 비활성화된다.
- `Restricted` 선택 시 상태바에 명시적으로 `Restricted` 라벨을 표시한다.
- `Trust & Remember`는 현재 workspace root 경로에만 적용된다.
- 이 화면은 시작 시 1회만 쓰는 임시 모달이 아니라, 이후 `/workspace trust` 및 설정 패널에서도 동일 상태를 관리하는 **정규 설정 surface**의 일부다.

## 8.2 Wizard 단계

### Step 1. Provider 선택
- 방향키(`↑`, `↓`)를 이용해 리스트에서 커서로 선택
- 기본 항목: `OpenRouter`, `Google (Gemini)`
- `Enter` 시 즉시 다음 단계(API Key) 전환
- Workspace Trust Gate가 끝난 뒤에만 진입한다.

### Step 2. 자격 증명 입력 (API Key)
- 타이핑하여 API Key를 인풋 버퍼에 누적(마스킹 지원 예정)
- 복사/붙여넣기 지원
- `Enter` 누르는 즉시 **비동기 모델 리스트 페칭(Loading)** 상태로 전환되어 화면 멈춤(프리징) 없이 통신 상태 표시

### Step 3. Model 선택 (Dynamic Listing)
- API 검증이 성공함과 동시에 동적으로 가져온 수백 개의 모델 리스트 렌더링
- 방향키 커서로 10개 단위 윈도잉 렌더링
- `Enter` 누를 시 최종 저장(Saving 단계 전환)
- API 오류 시 에러 사유를 UI에 알림 카드로 표출 후 `Esc` 대기

### Step 4. Permission Preset
초기 사용자 경험을 위해 세부 항목 직접 선택보다 preset 우선

- `Safe Starter`
  - shell: Ask
  - write: AlwaysAsk
  - network: ProviderOnly

- `Balanced`
  - shell: SafeOnly
  - write: AlwaysAsk
  - network: ProviderOnly

- `Strict`
  - shell: Deny
  - write: AlwaysAsk
  - network: Deny

preset을 고른 뒤 “Advanced”에서 세부 수정 가능

### Step 5. Save & Verify
- config.toml 영속 저장 (API 키 암호화)
- .master_key 파일 생성/검증
- 연결 상태 최종 확인
- 완료 시 메인 화면 진입

## 8.4 Master Settings Dashboard (`/config`)

`/setting`은 최초 1회 실행되는 강제적인 '순차 진행' 온보딩 마법사라면, `/config`는 전체 설정을 자유롭게 열람하고 방향키로 이동/수정하는 **종합 대시보드 창**이다.

- Timeline 화면 중앙에 오버레이 형태로 Modal/Panel이 등장한다.
- 방향키(`↑`, `↓`)로 세팅 카테고리(Provider, Model, Shell Policy, Theme 등)를 선택하고 `Enter`를 누르면 즉시 각 항목의 서브 선택 리스트(항목 변경 뷰)로 진입한다.
- 이 과정에서 Provider나 Model 변경 역시 사용자가 직접 이름을 타이핑할 필요 없이, 서버에서 동적으로 페치해온 리스트를 화살표 키보드로 훑어보고 고를 수 있게 전면 자동화한다.
- 수정 완료 시 자동으로 `~/.smlcli/config.toml`이 갱신된다.

## 8.3 설정 완료 전 차단 규칙

설정 완료 전에는:
- 일반 AI 요청 금지
- shell 실행 금지
- file write 금지

허용되는 것:
- `/help`
- `/setting`
- `/quit`

---

## 9. Mode 설계: PLAN / RUN

기존 스펙에 명시된 권한 정책을 더 직관적으로 이해시키기 위해 `PLAN`과 `RUN` 모드를 명시적으로 둔다.

## 9.1 PLAN
읽기·탐색·설계 중심 모드
- file read: 허용
- grep/diff: 허용
- write: 항상 ask
- shell: ask 또는 deny
- AI 응답 톤: 제안, 분석, 계획

상태바 표기:
```text
PLAN
```

## 9.2 RUN
실행·적용 중심 모드
- file read: 허용
- grep/diff: 허용
- write: 정책에 따름
- shell: 정책에 따름
- AI 응답 톤: 실행, 적용, 결과 요약

## 9.3 전환 UX
- `Tab` 또는 `Ctrl+T`로 전환
- 전환 시 toast:
  - `Switched to PLAN`
  - `Switched to RUN`

---

## 10. Slash Command 체계

최소 지원 명령은 스펙을 유지하되, 사용 빈도 기준으로 정렬된 자동완성을 제공한다.

### 10.1 핵심 명령
- `/config`: 종합 설정 대시보드 모달 오픈 (방향키 기반 전체 세팅 탐색 및 변경)
- `/setting`: 초기 Setup Wizard 강제 재진입 (순차 플로우)
- `/provider`: Provider 즉시 전환 팝업 (방향키 기반 선택)
- `/model`: Model 즉시 전환 팝업 (가용 리스트 페치 후 방향키 기반 선택)
- `/workspace`: 현재 root/trust/추가 workspace/deny roots 관리
- `/status`: 현재 적용된 Provider/Model, 잔여 토큰(Budget), 권한 모드 등 요약 출력
- `/mode`: 탐색 중심(PLAN) 모드와 실행 중심(RUN) 모드 즉시 토글
- `/clear`: AI 컨텍스트 윈도우 및 타임라인 채팅 내역 초기화
- `/help`: 전체 슬래시 명령어 및 시스템 단축키 설명서 출력
- `/quit`: 애플리케이션 안전 종료
- `/status`: Host shell / Exec shell / Workspace trust state도 함께 표시

### 10.2 추천 추가 명령
- `/mode`
- `/recent`
- `/logs`
- `/theme`
- `/doctor`
- `/workspace deny`
- `/workspace add`
- `/workspace remove`

### 10.3 자동완성 규칙
사용자가 `/` 입력 시:
- 상위 5개만 먼저 노출
- 현재 상태에서 쓸 수 없는 명령은 흐리게 표시
- 오른쪽에 짧은 설명 제공

예:
```text
/config       Open Master Settings Dashboard
/setting      Re-run Welcome Setup Wizard
/provider     Switch Provider via interactive list
/model        Switch Model via interactive list
/workspace    Manage trust / roots / extra dirs
/status       Show current session info
```

### 10.5 Planned Workspace Management Commands

Gemini CLI의 `/permissions trust`, `/directory add/show` 계열에서 차용한 관리 구조를 적용한다.

- `/workspace show`
  - 현재 root, trust state, extra workspace dirs, denied roots 출력
- `/workspace trust [path]`
  - 대상 경로의 trust 상태를 바꿈
- `/workspace add <path>`
  - 추가 workspace 디렉터리 등록
- `/workspace remove <path>`
  - 추가 workspace 디렉터리 제거
- `/workspace deny add <path>`
  - 접근 금지 루트 등록
- `/workspace deny remove <path>`
  - 접근 금지 루트 제거
- `/workspace deny list`
  - 현재 금지 루트 목록 표시

### 10.4 Extended Prompt Commands (@ and !)

프롬프트 오버레이에서 표시되는 `Fuzzy Finder`의 UI 렌더링 명세와 제어권 전환(UX) 규칙이다.

**1. 레이아웃 및 시각적 기준 (Visual & Layout Tokens)**
- **위치**: 중앙 타임라인 하단, Composer 패널 상단에 오버레이(`Clear` 후 `Block::bordered()`)로 렌더링한다.
- **제약 (Constraints)**: `Constraint::Length(5)`로 높이를 고정하여, 상하 테두리 2줄을 제외한 최대 **3개의 뷰포트 행**만 표시되도록 설계한다. 화면 침범을 최소화한다.
- **컬러 맵핑**:
  - 선택된 커서 항목: `Palette::bg_base` + `Modifier::BOLD`
  - 에러 피드백: `TimelineEntryKind::SystemNotice` 생성 시 붉은색(`Color::Red`) `⚠` 기호 사용.

**2. `@` 파일 및 특수 멘션 화면 흐름 (Screen Flow & Control)**
- **진입**: Composer에서 `@` 타이핑 시, `FuzzyFinderState.is_open = true` 및 `FuzzyMode::Files` 갱신. UI는 실시간 파일 목록(`workspace`, `terminal` 포함)을 노출한다.
- **탐색**: 방향키(`Up`/`Down`) 입력은 `Composer`가 아니라 `Fuzzy Finder`가 가로채어 `cursor` 값을 갱신한다.
- **제출 정책 (CTA Policy)**:
  - `Enter` 시:
    - 현재 `cursor`가 가리키는 `matches[cursor]` 문자열을 가져온다.
    - Composer의 버퍼를 `format!("@{} ", matches[cursor])` 형태로 치환하며 후속 타이핑을 유도한다.
    - 즉시 `FuzzyFinderState.is_open = false`로 변경하여 패널을 닫는다.

**3. `!` 직접 셸 실행 화면 흐름 (Screen Flow & Control)**
- **진입 활성화 조건**: Composer 버퍼가 `""` (빈 문자열) 일 때만 `!` 단일 입력 시 `FuzzyMode::Macros` 팝업을 노출한다. (중간에 `!`를 타이핑할 때는 반응하지 않음)
- **제출 및 치환 정책 (CTA Policy)**:
  - 노출된 리스트 예시: `build      (cargo build)`
  - `Enter` 시, 괄호 안의 실제 명령어(`cargo build`)를 파싱하여, Composer 버퍼를 `!cargo build`로 완전히 덮어씌운다.
- **히스토리 조작 맵핑 (Action Binding)**:
  - `Fuzzy Finder`가 닫힌 상태(`is_open == false`)이고 입력이 `!`로 시작하거나 버퍼에 셸 명령이 있을 때, 방향키(`Up`/`Down`) 이벤트는 `Fuzzy Finder`가 아닌 `ComposerState.history_idx` 증감으로 직결된다.

### 10.5 Native Structured Tool Call Flow (Phase 12)

기존 마크다운 코드블록 스크래핑 방식에서 Native Tool Call API로 마이그레이션 함에 따라, 사용자에게 보이는 Tool Call의 렌더링 방식 및 상태 전이 로직.

**1. 상태 전이 및 렌더링 타임라인 (Streaming Timeline)**
- **ToolQueued**: SSE 스트림이 완전히 파싱(`[DONE]`)되어, 메모리 버퍼에서 조립된 JSON이 구조체로 정상 매핑된 직후 발동된다. 타임라인에는 `[실행 대기중] 도구이름(파라미터...)` 형태로 옅게(`Color::DarkGray`) 노출된다.
- **ToolStarted**: 사용자의 승인(Ask Policy) 혹은 자동 통과(SafeOnly Policy) 후 즉시 노출. `[실행중] ⚙ 도구이름` 으로 노란색/푸른색 스피너를 상징하는 텍스트로 치환된다.
- **ToolFinished**: 도구 실행이 끝난 후 LLM에게 결과가 재전송(Rolled back into context)되는 시점. `[완료] ✔ 도구이름 (150ms)` 형태로 초록색 텍스트로 치환된다.

**2. 에러 자가 치유 렌더링 (Auto-healing UX)**
- LLM이 규격에 없는 도구 이름("foo_tool")을 호출하거나, 필수 파라미터를 누락했을 경우:
  - 런타임 패닉 없이, 시스템이 내부적으로 `Role::Tool`과 붉은색의 에러 메시지(`TimelineEntryKind::ToolError`)를 렌더링한다.
  - LLM에게 `{"role": "tool", "content": "Error: Invalid argument 'foo'..."}` 가 전달되어 스스로 프롬프트를 교정하도록 유도한다.
  - 사용자는 타임라인에서 AI가 실수하고 바로 수정하는 과정을 투명하게 지켜볼 수 있다.

---

## 11. 핵심 사용자 시나리오

## 11.1 최초 실행
1. 사용자가 `smlcli` 실행
2. Setup Wizard 표시
3. `config.toml` 손상 시 시작 오류 배너와 복구 가이드 표시
4. provider 연결
5. API key 검증
6. model 선택
7. permission preset 선택
8. 저장
9. 메인 타임라인 진입

완료 기준:
- 왜 막혔는지 헷갈리지 않는다.
- 다음 단계가 항상 1개만 보인다.

## 11.2 일반 프롬프트
1. 사용자 입력
2. AI가 필요한 파일/grep 제안 또는 실행
3. Tool Summary 카드 누적
4. 최종 답변 표시
5. 필요 시 Inspector에서 근거 확인

## 11.3 파일 수정
1. AI가 파일 읽기
2. 수정안 생성
3. Diff 탭 오픈
4. 타임라인에 Approval Card 생성
5. 사용자가 `Enter`로 승인 또는 `Esc`로 취소
6. atomic write 수행
7. 완료 메시지 표시

완료 기준:
- 사용자는 “무엇이 바뀌는지”를 승인 전에 본다.

## 11.4 grep 탐색
1. 사용자 또는 AI가 grep 요청
2. Search 탭 자동 오픈
3. 파일별 결과 목록 표시
4. 방향키 이동
5. `Enter`로 Preview 전환
6. 필요 시 해당 파일을 문맥에 추가

## 11.5 shell 실행
1. `!cargo test` 입력 또는 AI 제안
2. 정책 검사
3. 승인 필요 시 Approval Card 생성
4. 승인 후 Logs 탭 스트리밍
5. 완료 후 요약 메시지 생성

## 11.6 오류 복구
1. provider validation 실패
2. Error Card 표시
3. 가능한 액션만 제공
   - `Retry`
   - `Edit Key`
   - `Change Provider`
4. 실패 로그는 Logs 탭에서 확인

## 11.7 컨텍스트 압축 시스템 (Intelligent Compaction)
1. 토큰 예산(`/tokens` 또는 추정 모델) 75% 경고 알림 노출
2. `/compact` 수동 요청 또는 자동 85~90% 도달 시 압축 트리거
3. 삭제될 오래된 메시지들은 단순 폐기되지 않고 백그라운드 비동기로 LLM에 전달되어 한 줄 요약(Summarizing Condenser)으로 교환됨
4. 타임라인에 `[System: [Summary] 12 older messages dropped but goals retained...]` 카드 노출
5. 중요 설계가 담긴 메시지 핀(Pinning) UI 제공 (Pin 아이콘 표시)

---

## 12. 승인 UX 상세

## 12.1 승인 대상
- file write
- shell command
- 손상 복구 후 재설정
- 위험 권한 변경

## 12.2 승인 카드 구조

```text
Pending Approval
Action: Write file
Target: src/settings.rs
Impact: +12 -4
Reason: normalize provider/model on save

[Preview Diff] [Approve] [Deny]
```

## 12.3 승인 기본 포커스 규칙
- 승인 카드 등장 시 기본 포커스는 `Preview Diff`
- 곧바로 `Approve`에 포커스를 두지 않는다.
- 사용자가 실제 변경 내용을 먼저 보게 한다.

---

## 13. 키보드 모델 최종안

### 전역
- `↑ ↓`: 리스트 이동
- `← →`: 탭 이동
- `Enter`: 선택 / 승인 / 제출
- `Esc`: 취소 / 뒤로가기 / 패널 닫기
- `Tab`: PLAN/RUN 또는 포커스 순환
- `Shift+Tab`: 반대 방향 순환
- `Ctrl+C`: 안전 종료
- `PgUp/PgDn`: 타임라인 스크롤

### 입력
- `/`: command
- `@`: 파일 참조
- `!`: shell command
- `Shift+Enter`: 줄바꿈

### 작업
- `F2`: Inspector 토글
- `Ctrl+P`: provider/model quick switch
- `Ctrl+R`: permissions 열기
- `Ctrl+L`: 타임라인 clear
- `Tab`/`Shift+Tab`: PLAN ↔ RUN mode 전환

---

## 14. 좁은 터미널 대응

## 14.1 120컬럼 이상
- Timeline + Inspector 동시 표시

## 14.2 90~119컬럼
- Inspector 기본 닫힘
- 열리면 overlay 또는 비율 축소

## 14.3 70~89컬럼
- 상태바 축약
- provider/model은 provider만 표시 가능
- diff/search/log는 전체 화면 overlay

## 14.4 70컬럼 미만
- TUI 최소 지원
- 경고:
  - `Terminal width too small for full layout`
- Setup, Help, Approval만 단일 컬럼으로 제공

---

## 15. 시각 언어

## 15.1 색상 철학
강한 컬러 사용보다 정보 계층을 우선한다.
- 성공: 저강도 강조
- 경고: 중간 강조
- 위험: 고대비 강조

## 15.2 텍스트 우선
터미널 특성상 선, 배경색, 과한 박스보다 라벨과 정렬이 더 중요하다.

## 15.3 권장 시각 패턴
- 상태바는 촘촘하되 읽기 가능해야 한다.
- 타임라인 카드는 충분한 상하 여백 유지
- 카드 제목과 액션 버튼은 분리
- diff는 색 + 기호 둘 다 사용
  - `+`
  - `-`

---

## 16. 접근성과 신뢰성

## 16.1 색상 비의존
성공/실패/승인은 색만으로 구분하지 않는다.
- `Approved`
- `Denied`
- `Failed`
- `Pending`

## 16.2 종료 복구
panic, Ctrl+C, validation error 후에도:
- raw mode 해제
- alternate screen 해제
- 커서 복구

## 16.3 진행 상태 가시화
오래 걸리는 작업은 spinner만 두지 않는다.
- `Validating provider...`
- `Running shell command...`
- `Generating diff...`

---

## 17. 기존 스펙 대비 변경 요약

### 유지하는 것
- 기본 TUI 진입
- `/setting` wizard
- provider/model 표준화
- diff 기반 write approval
- grep/diff/file read/shell tool 체계
- permission visibility
- Windows/Linux 대응

### 바꾸는 것
- 좌측 상태 패널 삭제
- 우측 작업 패널을 Inspector로 통합
- 항상 3패널을 강제하지 않음
- 빈 채팅 진입 대신 Setup Home 도입
- PLAN/RUN 모드 명시
- permission preset 도입
- 타임라인 중심 작업 UX 강화

---

## 18. 구현 우선순위

### P1
- 상단 상태바
- 타임라인
- Composer
- Setup Home
- 기본 Inspector

### P2
- Diff/Search/Logs/Recent 탭
- 승인 카드 UX
- PLAN/RUN 전환
- responsive terminal width 대응

### P3
- 최근 작업 복원
- toast 체계
- advanced permission editor
- theme 세분화

---

## 19. 디자인 수용 기준

이 설계는 아래 조건을 만족해야 승인된 것으로 본다.

1. 첫 실행 후 30초 안에 provider 연결 경로를 이해할 수 있다.
2. 일반 사용 중 사용자의 시선이 중앙 타임라인에 유지된다.
3. 파일 수정 전 diff를 놓치지 않는다.
4. shell 실행 전 권한 상태를 즉시 이해할 수 있다.
5. 터미널 폭이 줄어도 핵심 흐름이 깨지지 않는다.
6. Linux와 Windows에서 동일한 키보드 모델을 유지한다.

---

## 20. 한 줄 결론

`smlcli`의 최적 UI는 “복잡한 3패널 IDE 흉내”가 아니라,  
**상태는 얇게, 작업은 중앙에, 근거는 Inspector에, 승인은 분명하게** 보이는 terminal-native workflow다.

---

## 21. Semantic Palette 설계 (v0.1.0-beta.18 개편)

모든 색상을 의미 기반(semantic)으로 통일하여 UI 일관성을 확보한다.

### 21.1 전경색 (Foreground)

| 역할 | 색상 | RGB | 용도 |
|------|--------|-----|------|
| `info` | 파랑 | (96, 165, 250) | 시스템 알림, 상태 정보 |
| `success` | 초록 | (74, 222, 128) | 성공 메시지, 완료 표시 |
| `warning` | 앨버 | (251, 191, 36) | 승인 대기, context 경고 |
| `danger` | 빨강 | (248, 113, 113) | 에러, 보안 차단 |
| `muted` | 회색 | (107, 114, 128) | 비활성 텍스트, 힌트 |
| `accent` | 보라 | (167, 139, 250) | 강조 표시, 선택 상태 |

### 21.2 배경색 (Background)

| 역할 | 색상 | RGB | 용도 |
|------|--------|-----|------|
| `bg_base` | 진한 네이비 | (17, 24, 39) | 전체 배경 |
| `bg_panel` | 어두운 네이비 | (31, 41, 55) | 패널 배경 |
| `bg_elevated` | 중간 네이비 | (55, 65, 81) | 카드/팝업 배경 |

### 21.3 고대비 모드 (High Contrast Palette)

접근성 지원을 위해 색상 대비를 극대화한 고대비 팔레트를 추가로 정의한다.

| 역할 | 색상 | 용도 |
|------|--------|------|
| `info` | 밝은 시안 | 시스템 알림 |
| `success` | 순수 초록 | 완료 표시 |
| `warning` | 순수 노랑 | 주의 필요 |
| `danger` | 순수 빨강 | 에러/차단 |
| `accent` | 순수 마젠타 | 선택 강조 |
| `bg_base` | 검정 (#000000) | 배경 |

### 21.4 테마 전환 로직

**[v0.1.0-beta.21 렌더링 연결 완료]**

- `/theme` 명령어를 통해 `Default`와 `HighContrast` 테마를 실시간으로 전환할 수 있다.
- 설정 파일(`config.toml`)에 `theme = "default" | "high_contrast"` 항목으로 저장되어 재시작 시 유지된다.
- 모든 TUI 렌더링 함수는 `state.palette()`를 통해 현재 테마의 `Palette` 참조를 취득하여 색상을 적용한다.

#### 구현 아키텍처

```text
/theme 명령어
    ↓
command_router.rs: settings.theme 토글 ("default" ↔ "high_contrast")
    ↓
tokio::spawn → config_store::save_config() (비동기 TOML 저장)
    ↓
AppState::palette() → get_palette(theme) → &'static Palette 참조 반환
    ↓
draw_top_bar / draw_timeline / draw_inspector / draw_composer
render_logs / render_search / render_recent
draw_config / draw_wizard
 → 각 함수 진입점에서 `let p = state.palette();` 선언
 → 모든 색상을 `p.info`, `p.accent`, `p.warning` 등으로 참조
```

#### 전환된 렌더링 파일 (4개, 50+곳)
- `tui/layout.rs`: 상태바, 타임라인, 인스펙터, 컴포저
- `tui/widgets/inspector_tabs.rs`: Logs, Search, Recent 탭
- `tui/widgets/config_dashboard.rs`: Config 팝업
- `tui/widgets/setting_wizard.rs`: Setup Wizard

#### 관련 파일
- `domain/settings.rs`: `PersistedSettings.theme` 필드 (`#[serde(default = "default_theme")]`)
- `tui/palette.rs`: `Palette` 구조체, `DEFAULT_PALETTE`, `HIGH_CONTRAST_PALETTE`, `get_palette()`
- `app/command_router.rs`: `/theme` 핸들러
- `app/state.rs`: `AppState::palette()` 헬퍼, `SlashMenuState::ALL_COMMANDS`에 `/theme` 등록

---

## 22. Phase 13: Agentic Autonomy UX (진행 예정)

자율적인 에이전트 동작(Auto-Verify, Git Checkpoint)을 지원하기 위해 시각적 정보를 강화하고 타임라인의 복잡도를 낮춥니다.

### 22.1 상단 상태바 Git 추적 (Git Checkpoint Status)
상태바(Top Bar)의 우측 상태 표시 영역을 확장하여 현재 워크스페이스의 Git 상태를 표시합니다.

- `clean`: 작업 트리에 변경 사항이 없을 때 표시 안함.
- `dirty`: 파일이 수정되었으나 커밋되지 않았을 때 `[Git: Dirty]` 노출.
- 체크포인트 생성 직후: 일시적으로 `[Git: Auto-Saved]` 상태를 노출하여 안도감을 줌.

### 22.2 Tree of Thoughts (아코디언 타임라인)
자가 치유(Self-Correction) 루프를 돌 때 에이전트가 호출하는 3~4개의 도구 실행 로그가 타임라인을 뒤덮지 않도록 인덴트(Indent) 트리 구조를 도입합니다.

**렌더링 예시 (접힌 상태 기본):**
```text
AI  : 테스트 실패를 확인했습니다. `lib.rs`의 타입 캐스팅을 수정합니다.
    └─ ⚙️ ExecShell (cargo test) ─ 1 failed
    └─ ⚙️ ReadFile (src/lib.rs)
    └─ ⚙️ WriteFile (src/lib.rs)
    └─ ⚙️ ExecShell (cargo test) ─ 14 passed
```

**동작 원리:**
- `TimelineBlock`에 `depth: u8` 필드를 추가하여 들여쓰기를 조절합니다.
- AI의 메인 답변은 `depth: 0`을 가집니다.
- 내부적으로 실행되는 도구 호출 결과 및 에러 복구 로그는 `depth: 1`을 가지며 타임라인에 `└─ ` 접두사로 렌더링됩니다.
- 현재 구현에서는 depth > 0인 엔트리가 상시 표시됩니다. 향후 아코디언(접힘/펼침) 동작은 별도 로드맵 항목으로 계획 중입니다.
- ※ Planner/Executor 분리는 현재 **프롬프트 수준**(PLAN/RUN 모드 시스템 지시문)에서 구현됩니다. 코드 아키텍처 수준의 분리(별도 Action variant)는 향후 로드맵입니다.
- 승인 대기 카드는 생성 시각을 기록하며, 5분 동안 응답이 없으면 자동으로 붉은색 Notice와 함께 취소됩니다.

---

## 23. Phase 14: TUI UX/UI 고도화 (v0.1.0-beta.24)

### 23.1 멀티라인 텍스트 렌더링

기존에는 `Line::from(msg.as_str())`로 멀티라인 문자열을 단일 `Line`에 밀어 넣어 개행이 무시되었습니다. Phase 14-A에서 `render_multiline_text()` 공용 헬퍼를 도입하여 `\n` 기준으로 분리된 독립 `Line`을 생성합니다.

```text
적용 경로:
  ├─ UserMessage      → render_multiline_text(msg, text_primary)
  ├─ AssistantMessage  → filter_tool_json + render_multiline_text
  ├─ AssistantDelta    → render_multiline_text(buf, text_primary)
  └─ session.messages 폴백 → filter_tool_json + render_multiline_text
```

`/help` 출력은 `command_router.rs`에서 `SystemNotice`로 타임라인에 직접 추가되어 멀티라인 렌더링이 보장됩니다.

### 23.2 스크롤 모델

```text
UiState
  ├─ timeline_scroll: u16       (bottom-up offset, 0 = 최하단/최신)
  ├─ inspector_scroll: u16      (인스펙터 전용, 독립)
  └─ timeline_follow_tail: bool (자동 추적 플래그)

렌더링 변환 (layout.rs):
  bottom_up = follow_tail ? 0 : timeline_scroll
  top_offset = max(0, total_lines - visible_height - bottom_up)
  Paragraph::scroll((top_offset, 0))

입력 매핑:
  PageUp  → timeline_scroll +5, follow_tail = false
  PageDown → timeline_scroll -5, 0이면 follow_tail = true
  Home    → timeline_scroll = MAX, follow_tail = false
  End     → timeline_scroll = 0, follow_tail = true
  Mouse ScrollUp/Down → 메인 영역 안에서 포인터 위치 기반 패널 판정
    └─ Timeline: 3줄 단위 스크롤, follow_tail 연동
    └─ Inspector: 3줄 단위 독립 스크롤
  Mouse Left Click → row/column 기준으로 Timeline / Inspector / Composer 포커스 전환

인스펙터 렌더러:
  inspector_tabs.rs (Logs/Search/Recent) + 승인 화면 → inspector_scroll 사용
```

- 사용자가 최하단에 있는 경우(`timeline_scroll == 0`, `follow_tail == true`) 새 타임라인 콘텐츠는 자동으로 화면에 따라붙어 보여야 한다.
- 사용자가 중간/상단으로 올려본 경우(`follow_tail == false`) 새 콘텐츠가 추가되어도 현재 보는 위치를 유지해야 한다.

### 23.2.1 상태바/툴바 정보 확장 (Planned)

- 상단 상태바와 하단 Toolbar는 다음 정보를 분리하여 노출한다.
  - `Host shell`
  - `Exec shell`
  - `Workspace trust`
- 좁은 폭에서는 아래 우선순위로 축약한다.
  1. `RUN/PLAN`
  2. `ctx`
  3. `Trust`
  4. `Exec`
  5. `Host`

- `terminal.rs`: `EnableMouseCapture` / `DisableMouseCapture` 적용.
- `event_loop.rs`: `CrosstermEvent::Mouse` → `Event::Mouse(MouseEvent)` 전달.

### 23.3 키바인딩 체계

| 키 | 기능 | 비고 |
|----|------|------|
| `Tab` / `Shift+Tab` | PLAN ↔ RUN 모드 전환 | |
| `F2` | 인스펙터 토글 | Ctrl+I는 터미널에서 Tab(0x09)과 동일하므로 사용 불가 |
| `Ctrl+C` | 종료 | |
| `Esc` | 팝업 닫기 / 종료 | 계층적 라우팅 |
| `PageUp/Down` | 타임라인 스크롤 | follow_tail 연동 |
| `Home/End` | 맨 위/아래 이동 | |
| 마우스 휠 | 패널별 독립 스크롤 | 포인터 위치 기반 라우팅 |

### 23.4 반응형 레이아웃

- **상단 바 (적응형)**: 세그먼트별 Span으로 분리하여 폭에 따라 점진적 생략.
  - 항상 표시: `smlcli · provider/model · mode · ctx%`
  - 폭 여유 시: `· cwd`
  - 추가 여유 시: `· Shell policy`
  - `provider`는 `truncate_middle(12)`, `model`은 `truncate_middle(20)`, `cwd`는 `truncate_middle(30)` 적용.
- **인스펙터 폭**: 고정 30% → `(total * 0.30).clamp(32, 48)` 픽셀 클램프. 타임라인 최소 72칼럼 보장.
- **탭 라벨**: 인스펙터 폭 < 40이면 축약형 (Preview→Prev, Search→Srch, Recent→Rcnt).

### 23.5 /help 구조화 렌더링

`/help` 출력은 `TimelineBlockKind::Help` 와 `BlockSection::KeyValueTable(Vec<(String, String)>)`로 구조화.

```text
렌더링 구조:
  ℹ  Available Commands:       ← info 색상 헤더
     /config    설명...          ← cmd: accent 고정 11칸, desc: text_secondary
     /setting   설명...
     ...
```

- 명령어 Span(고정 11칸, accent 색상)과 설명 Span(text_secondary)이 분리된 `Line`으로 렌더링.
- Paragraph wrap 시에도 명령어 부분은 한 줄에 고정되고 설명만 continuation.
- `session.messages`에는 `format!("{:<11}{}", cmd, desc)` 텍스트 형태로 보존 (LLM 컨텍스트용).

---

## 24. Phase 15: 2026 CLI UX 현대화 로드맵 (계획)

### 24.1 핵심 경험 목표

Phase 15의 목표는 `smlcli`를 "대화 로그가 쌓이는 TUI"에서 "작업 블록이 축적되는 작업 콘솔"로 전환하는 것이다.

핵심 경험은 아래 5개로 동결한다.

1. **Block-first Timeline**
   - 한 턴의 입력/AI/도구 결과가 하나의 작업 블록으로 묶여 보여야 한다.
2. **Command Palette First**
   - 긴 도움말보다 빠른 액션 발견이 우선이다.
3. **Composer as Workbench**
   - 입력창은 단순 프롬프트가 아니라 모드/컨텍스트/정책을 보여주는 작업대여야 한다.
4. **Focused Pane UX**
   - 타임라인, 인스펙터, 컴포저, 팔레트는 독립된 포커스 패널로 동작해야 한다.
5. **Restrained ASCII Motion**
   - 애니메이션은 상태 전달용으로만 제한한다.

### 24.2 외부 레퍼런스와 채택 포인트

#### Warp
- Blocks: 입력과 출력, 실행 결과를 블록 단위로 관리하는 UX 채택
- Universal Input: 입력창 주변에 액션/컨텍스트를 붙이는 툴벨트 UX 채택

#### Textual
- Command Palette: fuzzy search 기반 액션 탐색 UX 채택
- 액션/바인딩 분리: 키바인딩과 명령 시스템을 분리하는 구조 채택

#### Ratatui
- 반응형 Layout + 스타일 토큰 기반 렌더링 유지
- 과한 프레임워크 교체 없이 현재 코드베이스를 진화시키는 방향 유지

### 24.3 목표 레이아웃 구조

```text
┌ Top Bar ─ app · provider/model · mode · ctx · cwd · policy ───────────────┐
├ Timeline / Blocks ───────────────────────────────┬ Inspector Workspace ────┤
│ ┌ Block 014 · DONE · Python                      │ [Prev] [Diff] [Srch]    │
│ │ User Prompt                                    │ [Logs] [Rcnt]           │
│ │ AI Summary                                     │                          │
│ │ Tool Result (collapsed)                        │ Selected Block Details   │
│ └────────────────────────────────────────────────│                          │
│ ┌ Block 015 · NEEDS APPROVAL                     │ Recent files / diffs     │
│ │ Approval detail                                │ Logs / search facets     │
│ └────────────────────────────────────────────────│                          │
├ Composer Toolbar ─ [RUN] [@layout.rs] [Shell Ask] [Ctrl+K Actions] ───────┤
│ > prompt buffer...                                                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 24.4 Block Timeline 설계

#### 시각 구조
```text
┌ Block #042 · RUN · 2 tools · 14:22:18
│ 제목: 1부터 100까지 더하는 파이썬 코드 작성
│ 상태: DONE / ERROR / NEEDS APPROVAL / RUNNING
│
│ User
│   1부터 100까지 더하는 파이썬 코드 작성해주세요.
│
│ AI
│   파일을 생성했고 실행 방법도 정리했습니다.
│
│ Tool
│   ✅ WriteFile sum_1_to_100.py
│   ▶ stdout 3 lines hidden
└────────────────────────────────────────────
```

#### 동작 규칙
- 기본 블록은 접히지 않음
- stdout/stderr 12줄 초과 시 본문 일부만 보이고 `… N lines hidden`
- `Space`: 현재 블록 접기/펼치기
- `y`: 블록 고정(pinned)
- `c`: 블록 복사
- `r`: 블록 재실행 후보 액션 열기

### 24.5 Command Palette 설계

#### 호출
- `Ctrl+K`: Quick Actions Palette

#### 분류
- `Navigation`: 포커스 이동, 최근 블록 이동, 맨 위/아래 이동
- `Session`: clear, compact, export-log, quit
- `Settings`: provider/model/theme/mode
- `Context`: 최근 파일, 최근 diff, 최근 검색어
- `Tools`: 최근 셸 명령 재사용, 최근 도구 재실행

#### 렌더링
```text
┌ Quick Actions
│ > theme
│   테마 전환                  /theme
│   고대비 테마 적용
│   Inspector 토글             F2
└─────────────────────────────
```

### 24.6 Composer Toolbar 설계

```text
[RUN] [~/Projects/.../smlcli] [@src/tui/layout.rs] [Shell Ask] [Ctrl+K Actions]
> 사용자 입력...
```

#### 칩 규칙
- `Mode` 칩: 강조 색상
- `Context` 칩: 최대 5개
- `Policy` 칩: muted
- `Hint` 칩: muted/italic
- 칩 길이 18자 초과 시 중략

#### 입력 정책
- `Enter`: 제출
- `Shift+Enter`: 줄바꿈
- `Tab`: PLAN/RUN
- `F2`: Inspector
- `Ctrl+K`: Palette

### 24.7 포커스 및 스크롤 모델

```text
FocusedPane
  ├─ Timeline
  ├─ Inspector
  ├─ Composer
  └─ Palette
```

- 포커스된 pane만 키보드 스크롤 입력을 받는다.
- 마우스 휠은 포인터가 올라간 pane으로 라우팅한다.
- 타임라인은 block selection과 scroll offset을 동시에 가진다.
- 인스펙터는 tab selection과 scroll offset을 동시에 가진다.

### 24.8 모션 규칙

#### 허용되는 ASCII 애니메이션
- Thinking: `⠁⠂⠄⡀⢀⠠⠐⠈`
- Running tool: `●/○` 또는 `▶/▷`
- Approval pending: 테두리 pulse
- Stream settled: 한 번의 강조 후 정지

#### 금지되는 애니메이션
- 전체 화면 깜빡임
- 1초 이상 강한 색상 점멸
- 동일 정보에 2종 이상의 애니메이션 중첩

### 24.9 반응형 규칙

- `<100 cols`
  - 인스펙터는 우측 패널이 아니라 overlay/drawer
  - 상단 바는 `app · provider/model · mode · ctx%`만 보장
- `100..=139 cols`
  - 인스펙터 split view
  - 탭 라벨 축약형 사용 가능
- `>=140 cols`
  - full split view
  - cwd / shell policy / hint 칩 모두 표시

### 24.10 구현 단계별 디자인 체크리스트

#### Step A: Block Timeline
- 블록 헤더/본문/상태 배지 시각 규칙 확정
- stdout/stderr 접힘 표현 확정

#### Step B: Focused Pane
- 포커스 테두리/색/키맵 규칙 확정

#### Step C: Command Palette
- 팔레트 폭, 높이, 검색 결과 수, 카테고리 표현 확정

#### Step D: Composer Toolbar
- 칩 우선순위와 중략 정책 확정

#### Step E: Motion Polish
- 스피너, pulse, settle 효과를 상태별로 1개씩만 배치

## 25. Phase 16: Collapsible Diff UI 디자인 (v0.1.0-beta.26)

### 25.1 ASCII 기반 프로젝트 디자인 구조도

```text
[ Expanded State - 10줄 이하 또는 펼침 모드 ]
┌────────────────────────────────────────────────────────┐
│ ⚙️ ReplaceFileContent (src/main.rs)               [✓]│
│ ────────────────────────────────────────────────────── │
│ - old_function_name()                                  │
│ + new_function_name()                                  │
└────────────────────────────────────────────────────────┘

[ Collapsed State - 10줄 초과 기본 상태 ]
┌────────────────────────────────────────────────────────┐
│ ⚙️ ReplaceFileContent (src/main.rs)               [✓]│
│ ────────────────────────────────────────────────────── │
│ [ +14 lines / -3 lines ] (Enter 키로 펼치기)           │
└────────────────────────────────────────────────────────┘
```

### 25.2 화면별 요소 목록: Collapsed Diff
- **위치**: 타임라인 영역 안, `ReplaceFileContent` 블록의 Body 부분.
- **Collapsed 상태 (10줄 초과 시 기본값)**:
  - 렌더링 규칙: `[ +{add} lines / -{del} lines ] (Enter 키로 펼치기)`
  - 컬러: `palette.muted` (DarkGray 등)
  - `Hover/Focus` 상태일 때는 텍스트에 배경 반전 스타일 추가.
- **Expanded 상태**:
  - 기존의 초록/빨강 Diff 스팬(`Line`) 렌더링 수행.

### 25.3 데이터 연결 기준
- `AppState.ui.timeline[selected_index].display_mode` 값(`BlockDisplayMode::Collapsed/Expanded`)을 읽어 렌더링을 분기한다.
- `app.handle_enter_key()`에서 `FocusedPane == Timeline` && 현재 커서가 `TimelineBlockKind::ToolRun` 위에 있을 때 블록의 `display_mode`를 스왑(Swap)한다.
