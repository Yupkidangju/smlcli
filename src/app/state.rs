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
}

impl AppState {
    pub fn new() -> Self {
        let mut is_wizard_open = true;
        let mut loaded_settings = None;

        // MVP: 부팅 시 마스터키 확보 및 설정 로딩. 성공하면 셋업 프로세스 생략.
        if let Ok(mk) = crate::infra::secret_store::get_or_create_master_key()
            && let Ok(Some(settings)) = crate::infra::config_store::load_config(&mk)
        {
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
        }
    }
}
