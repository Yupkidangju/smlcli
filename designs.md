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
- `Ctrl+T`: mode 전환
- `Ctrl+R`: permissions 보기
- `Ctrl+I`: Inspector 토글

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

## 5.5 AI 추론 인디케이터 (v0.1.0-beta.16)

프롬프트 전송 후 AI 응답 수신까지 타임라인 하단에 추론 상태를 표시:

```
✨ AI가 응답을 생성하고 있습니다...
```

- `is_thinking` 플래그로 제어: `dispatch_chat_request()` 시 true, `ChatResponseOk/Err` 수신 시 false.
- 매 틱(tick) UI 리렌더링으로 실시간 반영.

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

**[v0.1.0-beta.16 구현]** 빈 Composer에서 `/` 입력 시 Composer 위에 자동완성 팝업이 활성화됨.
- 11개 내장 명령어 목록이 표시됨: `/config`, `/setting`, `/provider`, `/model`, `/status`, `/mode`, `/tokens`, `/compact`, `/clear`, `/help`, `/quit`
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

## 8.2 Wizard 단계

### Step 1. Provider 선택
- 방향키(`↑`, `↓`)를 이용해 리스트에서 커서로 선택
- 기본 항목: `OpenRouter`, `Google (Gemini)`
- `Enter` 시 즉시 다음 단계(API Key) 전환

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
- `/status`: 현재 적용된 Provider/Model, 잔여 토큰(Budget), 권한 모드 등 요약 출력
- `/mode`: 탐색 중심(PLAN) 모드와 실행 중심(RUN) 모드 즉시 토글
- `/clear`: AI 컨텍스트 윈도우 및 타임라인 채팅 내역 초기화
- `/help`: 전체 슬래시 명령어 및 시스템 단축키 설명서 출력
- `/quit`: 애플리케이션 안전 종료

### 10.2 추천 추가 명령
- `/mode`
- `/recent`
- `/logs`
- `/theme`
- `/doctor`

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
/status       Show current session info
```

---

## 11. 핵심 사용자 시나리오

## 11.1 최초 실행
1. 사용자가 `smlcli` 실행
2. Setup Home 표시
3. provider 연결
4. API key 검증
5. model 선택
6. permission preset 선택
7. 저장
8. 메인 타임라인 진입

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
- `Ctrl+I`: Inspector 토글
- `Ctrl+P`: provider/model quick switch
- `Ctrl+R`: permissions 열기
- `Ctrl+L`: 타임라인 clear
- `Ctrl+T`: mode 전환

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
