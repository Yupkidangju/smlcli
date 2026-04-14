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
