# Lessons Learned

`smlcli` 개발 및 유지보수 과정에서 얻은 주요 교훈과 경험을 기록합니다. AI의 자체 학습용 DNA 스토리지로 사용되며, 향후 아키텍처나 구조 변화 시 같은 실수를 반복하지 않기 위해 참고합니다.

## 기록 규칙
- 문제 상황과 근본 원인을 명확하게 정의합니다.
- 문제를 어떻게 해결했는지, 어떤 아키텍처적 Trade-off가 있었는지 기록합니다.
- AI 에이전트가 향후 유사한 코드를 짤 때 참고할 수 있도록 짧고 단호하게 기술합니다.

---

### [2026-04-15] AppState 동기화 및 보안 정책 강제(Enforcement) 교훈
- **상황:** 설정 마법사(Setup Wizard) 완료 후 저장 로직만 수행하고 `AppState` 내 메모리 값을 즉시 갱신하지 않아, 앱 재부팅 전까지는 설정을 인식하지 못하고 기본 "dummy_key"가 사용되던 치명적 기능 결함 발견.
- **해결/결정:** `Saving` 단계 완료 시 `save_config` 이후 즉시 `self.state.settings = Some(settings)`를 호출하여 런타임 상태를 핫-로드(Hot-load)하도록 수정. 또한, `PermissionEngine`을 통해 도구 실행 전 정책(ShellPolicy, FileWritePolicy 등)을 강제로 확인하는 단일 진입점(Checkgate)을 구축.
- **교훈:** 영구 저장소(Keyring, File)와 메모리 상태(State) 사이의 동기화 누락은 보안 및 기능상의 치명적 위험을 초래합니다. "Save-and-Apply" 패턴을 항상 명시적으로 구현하고, 도구 실행 로직은 반드시 중앙 집중형 정책 엔진(`PermissionEngine`)을 거치도록 설계하십시오.

---

### [2026-04-16] keyring 의존성과 크로스플랫폼 호환성 교훈
- **상황:** `keyring` 크레이트의 `sync-secret-service` feature 미지정으로 mock store가 사용되어 API 키가 영속화되지 않는 치명적 버그(beta.13)가 발생. 수정 후에도 gnome-keyring 미설치 환경(WSL, Docker)에서 실행 불가.
- **해결/결정:** keyring 크레이트를 완전 제거하고, `~/.smlcli/` 디렉토리에 마스터키 파일(.master_key) + TOML 설정(config.toml)의 파일 기반 암호화로 전환. ChaCha20Poly1305 필드별 암호화 유지, chmod 600 적용.
- **교훈:** OS 의존적 보안 저장소(keyring, Credential Manager)는 개발 환경에서는 작동하더라도 배포 환경(CI, headless, 컨테이너)에서 반드시 실패합니다. 크로스플랫폼 CLI 도구는 파일 기반 솔루션 + 적절한 파일 권한이 훨씬 안정적입니다. 또한, `cargo audit` 경고가 있는 크레이트(serde_yml)를 도입할 때는 반드시 RUSTSEC 데이터베이스를 먼저 확인해야 합니다.

---

### [2026-04-16] AI 응답 렌더링과 UX 피드백 교훈
- **상황:** AI 도구 호출 시 원시 JSON 스키마가 타임라인에 그대로 노출되어 사용자 혼란 야기. AI 추론 시 아무런 시각적 피드백 없음. 슬래시 명령어 자동완성 없음.
- **해결/결정:** (1) 타임라인 렌더링에 `filter_tool_json()` 필터 추가, (2) `is_thinking` 상태 플래그로 추론 인디케이터, (3) `SlashMenuState`로 자동완성 팝업, (4) 시스템 프롬프트에 ~1K 토큰 페르소나 정의.
- **교훈:** LLM 기반 CLI 에이전트에서 "AI가 출력하는 모든 것이 사용자에게 보인다"는 사실을 항상 고려해야 합니다. 도구 호출 JSON, 디버그 텍스트 등 내부 프로토콜이 사용자 타임라인에 노출되지 않도록 렌더링 레이어에서 필터링해야 하며, 비동기 작업에는 반드시 로딩 인디케이터를 동반해야 합니다.

### 3. 이벤트 아키텍처는 처음부터 세분화해야 한다 (v0.1.0-beta.18 분석)
- **문제:** 초기 설계에서 `Action` enum을 `ToolFinished`/`ChatResponseOk` 등 완료 이벤트 중심으로 7종만 정의했습니다. 이 구조는 MVP에서는 충분했으나, 진행 표시(스피너, 스트리밍, 작업 카드)를 구현하려 하자 시작·진행·완료·에러를 구분할 수 없어 구조적 한계에 부딪혔습니다.
- **해결/결정:** Action을 14종으로 확장하고 (ChatStarted, ChatDelta, ToolQueued, ToolStarted, ToolOutputChunk, ToolSummaryReady 추가), `session.messages`(LLM 컨텍스트)와 `timeline_entries`(UI 카드)를 분리하는 이중 데이터 모델을 도입합니다.
- **교훈:** 비동기 에이전트 시스템에서 이벤트 enum은 "무엇이 끝났는지"만이 아니라 "무엇이 시작되었고 어디까지 진행되었는지"까지 표현할 수 있어야 합니다. MVP 단계에서도 라이프사이클 전체(Queued → Started → Progress → Done/Error)를 설계해두면 이후 UI 확장 시 재작업을 피할 수 있습니다. 또한 LLM 컨텍스트와 사용자 화면 표시는 목적이 다르므로 반드시 분리해야 합니다.
