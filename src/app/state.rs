// [v0.1.0-beta.18] Phase 9-A: AppState 모듈.
// [v0.1.0-beta.19] AppState를 Domain, Ui, Runtime으로 분리하고 비동기 초기화 지원.

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorTab {
    Preview,
    Diff,
    Search,
    Logs,
    Recent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolStatus {
    Queued,
    Running,
    Done,
    Error,
}

#[derive(Debug, Clone)]
pub enum TimelineEntryKind {
    UserMessage(String),
    AssistantMessage(String),
    AssistantDelta(String),
    SystemNotice(String),
    ToolCard {
        tool_name: String,
        status: ToolStatus,
        summary: String,
    },
    ApprovalCard {
        tool_name: String,
        detail: String,
    },
    CompactSummary(String),
}

#[derive(Debug, Clone)]
pub struct TimelineEntry {
    pub kind: TimelineEntryKind,
    pub timestamp: std::time::Instant,
}

impl TimelineEntry {
    pub fn now(kind: TimelineEntryKind) -> Self {
        Self {
            kind,
            timestamp: std::time::Instant::now(),
        }
    }
}

pub struct DomainState {
    pub session: crate::domain::session::SessionState,
    pub settings: Option<crate::domain::settings::PersistedSettings>,
    pub session_logger: Option<crate::infra::session_log::SessionLogger>,
}

impl DomainState {
    /// 비동기 초기화: config.toml 로드 및 세션 로거 생성
    pub async fn new_async() -> Self {
        let loaded_settings = crate::infra::config_store::load_config()
            .await
            .ok()
            .flatten();
        let session_logger = crate::infra::session_log::SessionLogger::new_session().ok();

        Self {
            session: crate::domain::session::SessionState::new(),
            settings: loaded_settings,
            session_logger,
        }
    }
}

pub struct UiState {
    pub is_wizard_open: bool,
    pub show_inspector: bool,
    pub active_inspector_tab: InspectorTab,
    pub fuzzy: FuzzyFinderState,
    pub wizard: WizardState,
    pub config: ConfigState,
    pub composer: ComposerState,
    pub slash_menu: SlashMenuState,
    pub timeline: Vec<TimelineEntry>,
    pub tick_count: u64,
    /// [v0.1.0-beta.22] 타임라인 세로 스크롤 오프셋 (0 = 최하단/최신)
    pub timeline_scroll: u16,
}

impl UiState {
    pub fn new(is_wizard_open: bool) -> Self {
        Self {
            is_wizard_open,
            show_inspector: false,
            active_inspector_tab: InspectorTab::Preview,
            fuzzy: FuzzyFinderState::new(),
            wizard: WizardState::new(),
            config: ConfigState::new(),
            composer: ComposerState::new(),
            slash_menu: SlashMenuState::new(),
            timeline: Vec::new(),
            tick_count: 0,
            timeline_scroll: 0,
        }
    }
}

pub struct RuntimeState {
    pub is_thinking: bool,
    pub approval: ApprovalState,
    pub logs_buffer: Vec<String>,
    // [v0.1.0-beta.22] assistant_turn_count 삭제됨.
    // 사유: 첫 턴 차단 로직이 제거되어 카운터만 증가하는 데드 코드였음.
    // 삭제 버전: v0.1.0-beta.22 (재감사 6차)
    /// [v0.1.0-beta.22] 사용자의 마지막 입력이 작업 요청인지 여부.
    /// false이면 인삿말/잡담으로 판단하여 도구 디스패치를 런타임에서 억제한다.
    pub user_intent_actionable: bool,
}

impl RuntimeState {
    pub fn new() -> Self {
        Self {
            is_thinking: false,
            approval: ApprovalState::new(),
            logs_buffer: Vec::new(),
            user_intent_actionable: true,
        }
    }
}

pub struct AppState {
    pub should_quit: bool,
    pub domain: DomainState,
    pub ui: UiState,
    pub runtime: RuntimeState,
}

impl AppState {
    pub async fn new_async() -> Self {
        let domain = DomainState::new_async().await;
        let is_wizard_open = domain.settings.is_none();
        let ui = UiState::new(is_wizard_open);
        let runtime = RuntimeState::new();

        Self {
            should_quit: false,
            domain,
            ui,
            runtime,
        }
    }

    /// [v0.1.0-beta.22] 테스트 전용 동기 생성자.
    /// async 없이 App/AppState를 구성하여 process_tool_calls_from_response 등
    /// 핵심 경로를 통합 테스트할 수 있도록 한다.
    #[cfg(test)]
    pub fn new_for_test() -> Self {
        let domain = DomainState {
            session: crate::domain::session::SessionState::new(),
            settings: None,
            session_logger: None,
        };
        let ui = UiState::new(false);
        let runtime = RuntimeState::new();
        Self {
            should_quit: false,
            domain,
            ui,
            runtime,
        }
    }

    /// [v0.1.0-beta.21] 현재 설정된 테마에 따른 Palette 참조를 반환.
    /// 설정이 없거나 theme 값이 인식되지 않으면 기본(Default) 팔레트를 반환한다.
    /// 모든 TUI 렌더링 코드는 이 메서드를 통해 색상을 참조해야 함.
    pub fn palette(&self) -> &'static crate::tui::palette::Palette {
        let theme = self
            .domain
            .settings
            .as_ref()
            .map(|s| s.theme.as_str())
            .unwrap_or("default");
        crate::tui::palette::get_palette(theme)
    }
}

// === 서브 상태 구조체들 ===

#[derive(Debug, PartialEq, Clone)]
pub enum FuzzyMode {
    Files,
    Macros,
}

pub struct FuzzyFinderState {
    pub is_open: bool,
    pub mode: FuzzyMode,
    pub input: String,
    pub matches: Vec<String>,
    pub cursor: usize,
}

impl FuzzyFinderState {
    pub fn new() -> Self {
        Self {
            is_open: false,
            mode: FuzzyMode::Files,
            input: String::new(),
            matches: Vec::new(),
            cursor: 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum WizardStep {
    ProviderSelection,
    ApiKeyInput,
    ModelSelection,
    Saving,
}

pub struct WizardState {
    pub step: WizardStep,
    pub cursor_index: usize,
    pub selected_provider: Option<crate::domain::provider::ProviderKind>,
    pub api_key_input: String,
    pub available_models: Vec<String>,
    pub selected_model: String,
    pub is_loading_models: bool,
    pub err_msg: Option<String>,
}

impl WizardState {
    pub fn new() -> Self {
        Self {
            step: WizardStep::ProviderSelection,
            cursor_index: 0,
            selected_provider: None,
            api_key_input: String::new(),
            available_models: Vec::new(),
            selected_model: String::new(),
            is_loading_models: false,
            err_msg: None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ConfigPopup {
    Dashboard,
    ProviderList,
    ModelList,
}

pub struct ConfigState {
    pub is_open: bool,
    pub active_popup: ConfigPopup,
    pub cursor_index: usize,
    pub is_loading: bool,
    pub available_models: Vec<String>,
    pub err_msg: Option<String>,
    pub rollback_provider: Option<String>,
    pub rollback_model: Option<String>,
}

impl ConfigState {
    pub fn new() -> Self {
        Self {
            is_open: false,
            active_popup: ConfigPopup::Dashboard,
            cursor_index: 0,
            is_loading: false,
            available_models: Vec::new(),
            err_msg: None,
            rollback_provider: None,
            rollback_model: None,
        }
    }
}

pub struct ComposerState {
    pub input_buffer: String,
    pub history: Vec<String>,
    pub history_idx: Option<usize>,
}

impl ComposerState {
    pub fn new() -> Self {
        Self {
            input_buffer: String::new(),
            history: Vec::new(),
            history_idx: None,
        }
    }
}

pub struct ApprovalState {
    pub pending_tool: Option<crate::domain::tool_result::ToolCall>,
    pub pending_tool_call_id: Option<String>,
    pub diff_preview: Option<String>,
}

impl ApprovalState {
    pub fn new() -> Self {
        Self {
            pending_tool: None,
            pending_tool_call_id: None,
            diff_preview: None,
        }
    }
}

pub struct SlashMenuState {
    pub is_open: bool,
    pub filter: String,
    pub matches: Vec<(&'static str, &'static str)>,
    pub cursor: usize,
}

impl SlashMenuState {
    const ALL_COMMANDS: [(&'static str, &'static str); 12] = [
        ("/config", "Settings Dashboard"),
        ("/setting", "Setup Wizard"),
        ("/provider", "Switch Provider"),
        ("/model", "Switch Model"),
        ("/status", "Session Info"),
        ("/mode", "PLAN ↔ RUN Toggle"),
        ("/tokens", "Token Usage"),
        ("/compact", "Compress Context"),
        ("/theme", "Toggle Theme"),
        ("/clear", "Clear Chat"),
        ("/help", "Show Help"),
        ("/quit", "Exit"),
    ];

    pub fn new() -> Self {
        Self {
            is_open: false,
            filter: String::new(),
            matches: Self::ALL_COMMANDS.to_vec(),
            cursor: 0,
        }
    }

    pub fn update_matches(&mut self) {
        self.matches = Self::ALL_COMMANDS
            .iter()
            .filter(|(cmd, _)| cmd.starts_with(&self.filter) || cmd[1..].starts_with(&self.filter))
            .cloned()
            .collect();
        if self.cursor >= self.matches.len() {
            self.cursor = self.matches.len().saturating_sub(1);
        }
    }
}
