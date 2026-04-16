// [v0.1.0-beta.7] M-1: Home과 PermissionPreset은 미구현 상태로 enum에만 존재했으므로 제거.
// 삭제된 기능: Home(시작 화면), PermissionPreset(권한 프리셋 선택)
// 삭제 사유: 실제 렌더링과 로직 없이 enum만 있어 코드 혼선 야기
// 삭제 버전: v0.1.0-beta.7
#[derive(Debug, Clone, PartialEq)]
pub enum WizardStep {
    ProviderSelection,
    ApiKeyInput,
    ModelSelection,
    Saving,
}

pub struct WizardState {
    pub step: WizardStep,
    pub selected_provider: Option<crate::domain::provider::ProviderKind>,
    pub api_key_input: String,
    pub selected_model: String,

    // 화면 네비게이션 상태
    pub cursor_index: usize,
    pub available_models: Vec<String>,
    pub is_loading_models: bool,
    pub err_msg: Option<String>,
}

impl WizardState {
    pub fn new() -> Self {
        Self {
            step: WizardStep::ProviderSelection,
            selected_provider: Some(crate::domain::provider::ProviderKind::OpenRouter),
            api_key_input: String::new(),
            selected_model: String::new(),
            cursor_index: 0,
            available_models: Vec::new(),
            is_loading_models: false,
            err_msg: None,
        }
    }
}

pub struct ComposerState {
    pub input_buffer: String,
}

impl ComposerState {
    pub fn new() -> Self {
        Self {
            input_buffer: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigPopup {
    Dashboard,
    ProviderList,
    ModelList,
}

pub struct ConfigState {
    pub is_open: bool,
    pub active_popup: ConfigPopup,
    pub cursor_index: usize,
    pub available_models: Vec<String>,
    pub is_loading: bool,
    pub err_msg: Option<String>,
    // [v0.1.0-beta.10] 6차 감사: provider 전환 원자성 보장을 위한 복구 스냅샷.
    // 비동기 검증 실패 시 이전 provider/model로 in-memory 상태를 되돌림.
    pub rollback_provider: Option<String>,
    pub rollback_model: Option<String>,
}

impl ConfigState {
    pub fn new() -> Self {
        Self {
            is_open: false,
            active_popup: ConfigPopup::Dashboard,
            cursor_index: 0,
            available_models: Vec::new(),
            is_loading: false,
            err_msg: None,
            rollback_provider: None,
            rollback_model: None,
        }
    }
}

pub struct ApprovalState {
    // 승인 대기 중인 도구
    pub pending_tool: Option<crate::domain::tool_result::ToolCall>,
    // WriteFile/ReplaceFileContent 인 경우 생성된 Diff 보전
    pub diff_preview: Option<String>,
}

impl ApprovalState {
    pub fn new() -> Self {
        Self {
            pending_tool: None,
            diff_preview: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorTab {
    Preview,
    Diff,
    Search,
    Logs,
    Recent,
}

pub struct FuzzyFinderState {
    pub is_open: bool,
    pub input: String,
    pub matches: Vec<String>,
    pub cursor: usize,
}

impl FuzzyFinderState {
    pub fn new() -> Self {
        Self {
            is_open: false,
            input: String::new(),
            matches: Vec::new(),
            cursor: 0,
        }
    }
}

/// [v0.1.0-beta.16] 슬래시 커맨드 자동완성 메뉴 상태.
/// Composer에서 `/`를 입력하면 활성화되어 사용 가능한 명령어 목록을 부분 일치 필터링.
pub struct SlashMenuState {
    pub is_open: bool,
    /// `/` 이후 입력된 검색 문자열
    pub filter: String,
    /// 필터에 매칭되는 명령어 목록
    pub matches: Vec<(&'static str, &'static str)>,
    /// 현재 커서 위치
    pub cursor: usize,
}

/// 전체 슬래시 명령어 목록: (명령어, 설명)
pub const SLASH_COMMANDS: &[(&str, &str)] = &[
    ("/config", "설정 대시보드"),
    ("/setting", "셋업 위자드 재실행"),
    ("/provider", "공급자 전환"),
    ("/model", "모델 전환"),
    ("/status", "세션 상태 보기"),
    ("/mode", "PLAN ↔ RUN 전환"),
    ("/tokens", "토큰 사용량 보기"),
    ("/compact", "컨텍스트 압축"),
    ("/clear", "대화 초기화"),
    ("/help", "명령어 도움말"),
    ("/quit", "앱 종료"),
];

impl SlashMenuState {
    pub fn new() -> Self {
        Self {
            is_open: false,
            filter: String::new(),
            matches: Vec::new(),
            cursor: 0,
        }
    }

    /// 필터 문자열로 매칭되는 명령어 갱신
    pub fn update_matches(&mut self) {
        let query = format!("/{}", self.filter.to_lowercase());
        self.matches = SLASH_COMMANDS
            .iter()
            .filter(|(cmd, _)| cmd.starts_with(&query))
            .copied()
            .collect();
        if self.cursor >= self.matches.len() {
            self.cursor = 0;
        }
    }
}

pub struct AppState {
    pub should_quit: bool,
    pub is_wizard_open: bool,
    pub show_inspector: bool,
    pub active_inspector_tab: InspectorTab,
    pub fuzzy: FuzzyFinderState,
    pub wizard: WizardState,
    pub session: crate::domain::session::SessionState,
    pub settings: Option<crate::domain::settings::PersistedSettings>,
    pub config: ConfigState,
    pub composer: ComposerState,
    pub approval: ApprovalState,
    /// [v0.1.0-beta.16] AI 추론 중 여부 (thinking indicator 렌더링용)
    pub is_thinking: bool,
    /// [v0.1.0-beta.16] 슬래시 명령어 자동완성 메뉴
    pub slash_menu: SlashMenuState,
}

impl AppState {
    pub fn new() -> Self {
        let mut is_wizard_open = true;
        let mut loaded_settings = None;

        // [v0.1.0-beta.14] YAML 기반 설정 로드. master_key 불필요.
        if let Ok(Some(settings)) = crate::infra::config_store::load_config() {
            loaded_settings = Some(settings);
            is_wizard_open = false;
        }

        Self {
            should_quit: false,
            is_wizard_open,
            show_inspector: false,
            active_inspector_tab: InspectorTab::Preview,
            fuzzy: FuzzyFinderState::new(),
            settings: loaded_settings,
            wizard: WizardState::new(),
            config: ConfigState::new(),
            session: crate::domain::session::SessionState::new(),
            composer: ComposerState::new(),
            approval: ApprovalState::new(),
            is_thinking: false,
            slash_menu: SlashMenuState::new(),
        }
    }
}
