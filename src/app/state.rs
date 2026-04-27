// [v0.1.0-beta.18] Phase 9-A: AppState 모듈.
// [v0.1.0-beta.19] AppState를 Domain, Ui, Runtime으로 분리하고 비동기 초기화 지원.

// [v3.7.0] 인스펙터 탭 variant는 TUI 인스펙터 고도화 시 활성화 예정.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum InspectorTab {
    Preview,
    Diff,
    Search,
    Logs,
    Recent,
    Git,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimelineBlockKind {
    Conversation,
    ToolRun,
    Approval,
    Help,
    Notice,
    GitCommit,
}

// [v3.7.0] CodeFence variant는 코드 블록 렌더링 전환 시 활성화 예정.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum BlockSection {
    Markdown(String),
    CodeFence {
        language: Option<String>,
        content: String,
    },
    KeyValueTable(Vec<(String, String)>),
    ToolSummary {
        tool_name: String,
        summary: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockStatus {
    Idle,
    Running,
    Done,
    Error,
    NeedsApproval,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockDisplayMode {
    Collapsed,
    Expanded,
}

// [v3.7.0] TimelineBlock의 id, subtitle, pinned, created_at_ms 필드는
// 블록 북마크/타임스탬프 표시 기능 구현 시 활성화 예정.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TimelineBlock {
    pub id: String,
    pub kind: TimelineBlockKind,
    /// [v0.1.0-beta.25] Tree of Thoughts 렌더링용 계층 깊이.
    /// 0은 루트 타임라인, 1 이상은 부모 응답에 종속된 도구/복구 카드다.
    pub depth: u8,
    pub title: String,
    pub subtitle: Option<String>,
    pub body: Vec<BlockSection>,
    pub status: BlockStatus,
    pub display_mode: BlockDisplayMode,
    pub role: Option<crate::providers::types::Role>,
    pub diff_summary: Option<(usize, usize)>,
    pub tool_call_id: Option<String>,
    pub pinned: bool,
    pub created_at_ms: u64,
}

impl TimelineBlock {
    pub fn new(kind: TimelineBlockKind, title: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            kind,
            depth: 0,
            title: title.into(),
            subtitle: None,
            body: Vec::new(),
            status: BlockStatus::Idle,
            display_mode: BlockDisplayMode::Expanded,
            role: None,
            diff_summary: None,
            tool_call_id: None,
            pinned: false,
            created_at_ms: std::time::UNIX_EPOCH.elapsed().unwrap().as_millis() as u64,
        }
    }

    /// [v0.1.0-beta.25] 문서 스펙의 트리형 타임라인을 맞추기 위한 깊이 지정 헬퍼.
    pub fn with_depth(mut self, depth: u8) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_role(mut self, role: crate::providers::types::Role) -> Self {
        self.role = Some(role);
        self
    }

    pub fn with_tool_call_id(mut self, id: Option<String>) -> Self {
        self.tool_call_id = id;
        self
    }

    /// [v0.1.0-beta.26] 접기/펼치기 토글 헬퍼.
    pub fn toggle_collapse(&mut self) {
        self.display_mode = match self.display_mode {
            BlockDisplayMode::Collapsed => BlockDisplayMode::Expanded,
            BlockDisplayMode::Expanded => BlockDisplayMode::Collapsed,
        };
    }
}

pub struct DomainState {
    pub session: crate::domain::session::SessionState,
    pub settings: Option<crate::domain::settings::PersistedSettings>,
    pub session_logger: Option<crate::infra::session_log::SessionLogger>,
    pub config_load_error: Option<String>,
    // [v3.6.0] Phase 46 Task S-1: 현재 활성 세션의 메타데이터.
    // 세션 전환(/resume, /new) 시 이 필드가 교체됨.
    pub current_session_metadata: Option<crate::domain::session::SessionMetadata>,
}

impl DomainState {
    /// 비동기 초기화: config.toml 로드 및 세션 로거 생성
    pub async fn new_async() -> Self {
        let (loaded_settings, config_load_error) = match crate::infra::config_store::load_config()
            .await
        {
            Ok(settings) => (settings, None),
            Err(err) => {
                let path = crate::infra::config_store::config_path();
                (
                    None,
                    Some(format!(
                        "설정 파일을 읽지 못했습니다: {}.\n문제가 지속되면 {} 파일을 복구하거나 삭제한 뒤 설정 마법사를 다시 진행하세요.",
                        err,
                        path.display()
                    )),
                )
            }
        };

        if let Some(settings) = &loaded_settings {
            crate::providers::registry::update_custom_providers(&settings.custom_providers);
        }

        // [v3.6.0] Phase 46: 워크스페이스 기반 세션 생성
        let workspace_root = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string());
        let (session_logger, session_metadata) =
            match crate::infra::session_log::SessionLogger::new_workspace_session(&workspace_root) {
                Ok((logger, meta)) => (Some(logger), Some(meta)),
                Err(_) => {
                    // 폴백: 기존 방식으로 로거만 생성
                    let logger = crate::infra::session_log::SessionLogger::new_session().ok();
                    (logger, None)
                }
            };

        Self {
            session: crate::domain::session::SessionState::new(),
            settings: loaded_settings,
            session_logger,
            config_load_error,
            current_session_metadata: session_metadata,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FocusedPane {
    Timeline,
    Inspector,
    Composer,
    Palette,
}

/// [v2.3.0] Phase 31: 클립보드 등 UI 알림을 위한 상태 구조체
#[derive(Debug, Clone)]
pub struct ToastNotification {
    pub message: String,
    pub expires_at: std::time::Instant,
    pub is_error: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputChipKind {
    Mode,
    Path,
    Context,
    Policy,
    Hint,
}

// [v3.7.0] emphasized 필드는 Composer Toolbar 강조 칩 렌더링 시 활성화 예정.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InputChip {
    pub kind: InputChipKind,
    pub label: String,
    pub emphasized: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ComposerToolbarState {
    pub chips: Vec<InputChip>,
    pub multiline: bool,
}

// [v3.7.0] tick_ms 필드는 모션 프레임 제어 최적화 시 활성화 예정.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MotionProfile {
    pub tick_ms: u64,
    pub spinner_frames: &'static [&'static str],
    pub pulse_period_ticks: u8,
}

impl Default for MotionProfile {
    fn default() -> Self {
        Self {
            tick_ms: 120,
            spinner_frames: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            pulse_period_ticks: 6,
        }
    }
}

pub struct UiState {
    /// [v2.3.0] Phase 31: 클립보드 피드백용 토스트 알림
    pub toast: Option<ToastNotification>,
    pub is_wizard_open: bool,
    pub trust_gate: TrustGateState,
    pub show_inspector: bool,
    pub active_inspector_tab: InspectorTab,
    pub fuzzy: FuzzyFinderState,
    pub wizard: WizardState,
    pub config: ConfigState,
    pub composer: ComposerState,
    pub toolbar: ComposerToolbarState,
    pub slash_menu: SlashMenuState,
    pub timeline: Vec<TimelineBlock>,
    pub tick_count: u64,
    pub motion: MotionProfile,
    #[allow(dead_code)] // [v3.7.0] 블록 기반 스크롤 전환 시 활성화 예정
    pub timeline_scroll_offset: usize,
    /// [Phase 15-E] 타임라인 내 선택된 블록 커서.
    pub timeline_cursor: usize,
    /// [v0.1.0-beta.24] 타임라인 세로 스크롤 오프셋 (bottom-up: 0 = 최하단/최신, N = 바닥에서 N줄 위).
    /// 렌더링 시 layout.rs에서 top-based offset으로 변환됨.
    pub timeline_scroll: u16,
    /// [v0.1.0-beta.24] Phase 14-B: 인스펙터 전용 스크롤 오프셋
    pub inspector_scroll: std::cell::Cell<u16>,
    /// [v2.1.0] Phase 29: 스크롤 앵커 상태 관리
    pub inspector_anchor: std::cell::Cell<crate::tui::widgets::inspector_tabs::ScrollAnchor>,
    pub last_inspector_height: std::cell::Cell<usize>,
    /// [v0.1.0-beta.24] Phase 14-B: 타임라인 자동 추적 플래그.
    /// true이면 새 콘텐츠 추가 시 스크롤을 맨 아래로 이동.
    /// 사용자가 위로 스크롤하면 false, End 키 또는 맨 아래 도달 시 다시 true.
    pub timeline_follow_tail: bool,
    pub focused_pane: FocusedPane,
    pub palette: CommandPaletteState,
    pub force_clear: bool,
    // [v2.4.0] Phase 32: Help Overlay
    pub show_help_overlay: bool,
    // [v3.7.0] Phase 47: Interactive Planning Questionnaire UI 상태.
    // Some이면 Questionnaire 모달이 활성화된 상태.
    pub questionnaire: Option<crate::domain::questionnaire::QuestionnaireState>,
}

impl UiState {
    pub fn new(is_wizard_open: bool) -> Self {
        Self {
            toast: None,
            is_wizard_open,
            trust_gate: TrustGateState::new(),
            show_inspector: false,
            active_inspector_tab: InspectorTab::Preview,
            fuzzy: FuzzyFinderState::new(),
            wizard: WizardState::new(),
            config: ConfigState::new(),
            composer: ComposerState::new(),
            toolbar: ComposerToolbarState::default(),
            slash_menu: SlashMenuState::new(),
            timeline: Vec::new(),
            tick_count: 0,
            motion: MotionProfile::default(),
            timeline_scroll_offset: 0,
            timeline_cursor: 0,
            timeline_scroll: 0,
            inspector_scroll: std::cell::Cell::new(0),
            inspector_anchor: std::cell::Cell::new(
                crate::tui::widgets::inspector_tabs::ScrollAnchor::default(),
            ),
            last_inspector_height: std::cell::Cell::new(0),
            timeline_follow_tail: true,
            focused_pane: FocusedPane::Composer,
            palette: CommandPaletteState::new(),
            force_clear: false,
            show_help_overlay: false,
            questionnaire: None,
        }
    }
}

// [v3.7.0] Tools, Settings, Context variant는 Command Palette 카테고리 확장 시 활성화 예정.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PaletteCategory {
    Navigation,
    Session,
    Tools,
    Settings,
    Context,
}

impl std::fmt::Display for PaletteCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Navigation => write!(f, "Navigation"),
            Self::Session => write!(f, "Session"),
            Self::Tools => write!(f, "Tools"),
            Self::Settings => write!(f, "Settings"),
            Self::Context => write!(f, "Context"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PaletteCommand {
    pub id: &'static str,
    pub title: &'static str,
    pub category: PaletteCategory,
    pub shortcut_hint: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct CommandPaletteState {
    pub is_open: bool,
    pub query: String,
    pub cursor: usize,
    pub results: Vec<PaletteCommand>,
    // 전체 커맨드 풀을 내부에 가지고 있다가 query에 맞게 results를 필터링
    pub all_commands: Vec<PaletteCommand>,
}

impl CommandPaletteState {
    pub fn new() -> Self {
        let all_commands = vec![
            PaletteCommand {
                id: "/help",
                title: "Help",
                category: PaletteCategory::Navigation,
                shortcut_hint: None,
            },
            PaletteCommand {
                id: "/config",
                title: "Settings Dashboard",
                category: PaletteCategory::Navigation,
                shortcut_hint: None,
            },
            PaletteCommand {
                id: "/setting",
                title: "Setup Wizard",
                category: PaletteCategory::Navigation,
                shortcut_hint: None,
            },
            PaletteCommand {
                id: "/provider",
                title: "Switch Provider",
                category: PaletteCategory::Navigation,
                shortcut_hint: None,
            },
            PaletteCommand {
                id: "/model",
                title: "Switch Model",
                category: PaletteCategory::Navigation,
                shortcut_hint: None,
            },
            PaletteCommand {
                id: "/status",
                title: "Session Info",
                category: PaletteCategory::Session,
                shortcut_hint: None,
            },
            PaletteCommand {
                id: "/mode",
                title: "PLAN ↔ RUN Toggle",
                category: PaletteCategory::Session,
                shortcut_hint: None,
            },
            PaletteCommand {
                id: "/tokens",
                title: "Token Usage",
                category: PaletteCategory::Session,
                shortcut_hint: None,
            },
            PaletteCommand {
                id: "/compact",
                title: "Compact Context",
                category: PaletteCategory::Session,
                shortcut_hint: None,
            },
            PaletteCommand {
                id: "/theme",
                title: "Toggle Theme",
                category: PaletteCategory::Navigation,
                shortcut_hint: None,
            },
            PaletteCommand {
                id: "/clear",
                title: "Clear Session",
                category: PaletteCategory::Session,
                shortcut_hint: None,
            },
            PaletteCommand {
                id: "/workspace trust",
                title: "Workspace: Trust & Remember",
                category: PaletteCategory::Navigation,
                shortcut_hint: None,
            },
            PaletteCommand {
                id: "/workspace deny",
                title: "Workspace: Restrict (Read-only)",
                category: PaletteCategory::Navigation,
                shortcut_hint: None,
            },
            PaletteCommand {
                id: "/workspace clear",
                title: "Workspace: Clear Trust State",
                category: PaletteCategory::Navigation,
                shortcut_hint: None,
            },
            PaletteCommand {
                id: "toggle_inspector",
                title: "Toggle Inspector",
                category: PaletteCategory::Navigation,
                shortcut_hint: Some("F2"),
            },
            // [v3.6.0] Phase 46: 세션 관리 명령어
            PaletteCommand {
                id: "/new",
                title: "New Session",
                category: PaletteCategory::Session,
                shortcut_hint: None,
            },
            PaletteCommand {
                id: "/resume",
                title: "Resume Session",
                category: PaletteCategory::Session,
                shortcut_hint: None,
            },
            PaletteCommand {
                id: "/session",
                title: "Session List",
                category: PaletteCategory::Session,
                shortcut_hint: None,
            },
        ];

        Self {
            is_open: false,
            query: String::new(),
            cursor: 0,
            results: all_commands.clone(),
            all_commands,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AutoVerifyState {
    Idle,
    Healing { retries: usize },
    // [v2.5.0] 3회 초과 실패 시 Aborted 상태로 전환.
    // 이 상태에서는 pending_tool_executions == 0 시점에서도 LLM 재전송을 차단한다.
    // 병렬 도구 간 abort 결정이 일관되게 유지됨.
    Aborted,
}

// [v3.7.0] root_path, trust_state 등은 Workspace Trust Gate UI 연동 시 활성화 예정.
#[allow(dead_code)]
pub struct RuntimeWorkspaceState {
    pub root_path: String,
    pub host_shell: String,
    pub exec_shell: String,
    pub trust_state: crate::domain::settings::WorkspaceTrustState,
    pub trust_prompt_visible: bool,
    pub extra_workspace_dirs: Vec<String>,
}

impl RuntimeWorkspaceState {
    pub fn new() -> Self {
        let host_shell = if cfg!(target_os = "windows") {
            std::env::var("ComSpec").unwrap_or_else(|_| "cmd.exe".to_string())
        } else {
            std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string())
        };

        let exec_shell = if cfg!(target_os = "windows") {
            if crate::tools::shell::command_in_path("pwsh.exe").is_some()
                || crate::tools::shell::command_in_path("pwsh").is_some()
            {
                "pwsh".to_string()
            } else if crate::tools::shell::command_in_path("powershell.exe").is_some() {
                "powershell.exe".to_string()
            } else {
                "Not Found".to_string()
            }
        } else {
            "sh (bwrap)".to_string()
        };

        Self {
            root_path: String::new(),
            host_shell,
            exec_shell,
            trust_state: crate::domain::settings::WorkspaceTrustState::Unknown,
            trust_prompt_visible: false,
            extra_workspace_dirs: Vec::new(),
        }
    }
}

pub struct RuntimeState {
    pub is_thinking: bool,
    pub approval: ApprovalState,
    pub logs_buffer: Vec<String>,
    pub repo_map: crate::domain::repo_map::RepoMapState,
    // [v0.1.0-beta.22] assistant_turn_count 삭제됨.
    // 사유: 첫 턴 차단 로직이 제거되어 카운터만 증가하는 데드 코드였음.
    // 삭제 버전: v0.1.0-beta.22 (재감사 6차)
    /// [v0.1.0-beta.25] 사용자의 마지막 입력을 작업성 힌트로 분류한 값.
    /// LLM의 구조화된 도구 판단을 강제 차단하지 않고, 로깅/설명 보조용으로만 사용한다.
    pub user_intent_actionable: bool,
    pub auto_verify: AutoVerifyState,
    pub workspace: RuntimeWorkspaceState,
    pub active_chat_block_idx: Option<usize>,
    // [v2.5.0] Phase 35: 병렬 도구 실행을 위한 per-tool CancellationToken 맵.
    // 키: tool_call_id (없으면 인덱스 기반 문자열), 값: CancellationToken.
    pub active_tool_cancel_tokens:
        std::collections::HashMap<String, tokio_util::sync::CancellationToken>,
    pub stream_accumulator: String,
    // [v1.6.0] RepoMap의 동적 갱신을 위한 Dirty Flag
    pub repo_map_dirty: bool,
    // [v1.6.0] API 키 노출 방지 마스킹용 정규식 캐시
    pub secret_mask_regex: Option<regex::Regex>,
    // [v1.8.0] Phase 26: 스트리밍 마스킹 윈도우
    pub streaming_masker: StreamingMasker,

    // [v2.4.0] Phase 32: Parallel Tool Execution
    pub pending_tool_executions: usize,
    pub write_tool_queue:
        std::collections::VecDeque<(crate::domain::tool_result::ToolCall, Option<String>, usize)>,
    pub is_write_tool_running: bool,
    #[allow(dead_code)] // [v3.7.0] 병렬 도구 순서 보장 최적화 시 활성화 예정
    pub pending_tool_outcomes: Vec<(usize, ToolOutcome)>,

    // [v3.3.0] Phase 43: MCP 클라이언트
    // [v3.3.2] 감사 HIGH-3 수정: mcp_clients key를 정규화된 서버명으로 저장.
    // 스키마 노출 시 정규화된 이름을 사용하므로, 런타임 라우팅도 정규화명으로 매칭해야 함.
    pub mcp_clients: std::collections::HashMap<String, crate::infra::mcp_client::McpClient>,
    pub mcp_tools_cache: Vec<serde_json::Value>,
    // [v3.3.2] 감사 HIGH-3 수정: 정규화 도구명 → MCP 원본 도구명 역매핑 테이블.
    // key: "mcp_{sanitized_server}_{sanitized_tool}", value: (sanitized_server, original_tool_name)
    // tool_runtime에서 strip_prefix 후 원본 도구명으로 MCP call_tool() 호출.
    pub mcp_tool_name_map: std::collections::HashMap<String, (String, String)>,
}

pub enum ToolOutcome {
    Success(Box<crate::domain::tool_result::ToolResult>),
    Error(crate::domain::error::ToolError, Option<String>),
}

// 스트리밍 마스킹용 윈도우 상태
#[derive(Debug, Clone, Default)]
pub struct StreamingMasker {
    // 이전 청크의 마지막 N 바이트를 보관하여 스트림 경계 단어 보존
    pub trailing_buffer: String,
    pub max_match_len: usize,
}

impl RuntimeState {
    pub fn new() -> Self {
        Self {
            is_thinking: false,
            approval: ApprovalState::new(),
            logs_buffer: Vec::new(),
            repo_map: crate::domain::repo_map::RepoMapState::new(),
            user_intent_actionable: true,
            auto_verify: AutoVerifyState::Idle,
            workspace: RuntimeWorkspaceState::new(),
            active_chat_block_idx: None,
            active_tool_cancel_tokens: std::collections::HashMap::new(),
            stream_accumulator: String::new(),
            repo_map_dirty: true,
            secret_mask_regex: None,
            streaming_masker: StreamingMasker::default(),
            pending_tool_executions: 0,
            write_tool_queue: std::collections::VecDeque::new(),
            is_write_tool_running: false,
            pending_tool_outcomes: Vec::new(),
            mcp_clients: std::collections::HashMap::new(),
            mcp_tools_cache: Vec::new(),
            mcp_tool_name_map: std::collections::HashMap::new(),
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

        let mut state = Self {
            should_quit: false,
            domain,
            ui,
            runtime,
        };
        if let Some(err) = state.domain.config_load_error.clone() {
            state.apply_startup_config_error(err);
        }

        state
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
            config_load_error: None,
            current_session_metadata: None,
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

    /// [v0.1.0-beta.26] 시작 시점 설정 로드 오류를 사용자에게 명시적으로 노출한다.
    /// 기존에는 파싱 실패가 "설정 없음"처럼 보였기 때문에 복구 가이드를 위자드와 로그에 함께 남긴다.
    pub(crate) fn apply_startup_config_error(&mut self, message: String) {
        self.ui.is_wizard_open = true;
        self.ui.wizard.err_msg = Some(message.clone());
        self.runtime
            .logs_buffer
            .push(format!("[Startup Config Error] {}", message));
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

#[derive(Debug, PartialEq, Clone)]
pub enum TrustGatePopup {
    Closed,
    Open { root: String },
}

pub struct TrustGateState {
    pub popup: TrustGatePopup,
    pub cursor_index: usize, // 0: Trust Once, 1: Trust & Remember, 2: Restricted
}

impl TrustGateState {
    pub fn new() -> Self {
        Self {
            popup: TrustGatePopup::Closed,
            cursor_index: 0,
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

#[derive(Debug, Clone, Default)]
pub struct ApprovalState {
    pub pending_tool: Option<crate::domain::tool_result::ToolCall>,
    pub pending_tool_call_id: Option<String>,
    pub pending_tool_index: Option<usize>,
    pub pending_since_ms: Option<u64>,
    pub diff_preview: Option<String>,
    pub queued_approvals:
        std::collections::VecDeque<(crate::domain::tool_result::ToolCall, Option<String>, usize)>,
    #[allow(dead_code)] // [v3.7.0] 병렬 도구 결과 순서 보장 시 활성화 예정
    pub pending_tool_outcomes:
        std::collections::HashMap<usize, crate::domain::tool_result::ToolResult>,
}

impl ApprovalState {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct SlashMenuState {
    pub is_open: bool,
    pub filter: String,
    pub matches: Vec<(&'static str, &'static str)>,
    pub cursor: usize,
}

impl SlashMenuState {
    const ALL_COMMANDS: [(&'static str, &'static str); 16] = [
        ("/config", "Settings Dashboard"),
        ("/setting", "Setup Wizard"),
        ("/provider", "Switch Provider"),
        ("/model", "Switch Model"),
        ("/status", "Session Info"),
        ("/mode", "PLAN ↔ RUN Toggle"),
        ("/tokens", "Token Usage"),
        ("/compact", "Compress Context"),
        ("/theme", "Toggle Theme"),
        ("/workspace", "Manage Workspace Trust"),
        ("/new", "New Session"),
        ("/resume", "Resume Session"),
        ("/session", "Session List"),
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
