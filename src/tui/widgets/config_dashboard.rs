// [v0.1.0-beta.7] Config 대시보드 위젯.
// [v0.1.0-beta.21] 하드코딩 Color::Yellow를 state.palette() 동적 참조로 전환.

use crate::app::state::{AppState, ConfigPopup};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{Borders, Clear, Paragraph},
};

pub fn draw_config(f: &mut Frame, state: &AppState) {
    // [v0.1.0-beta.21] 동적 팔레트 참조: 테마 전환 즉시 반영
    let p = state.palette();

    let size = f.area();

    // 중앙 정렬된 팝업 영역 생성
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(size);

    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(popup_layout[1])[1];

    f.render_widget(Clear, area); // 배경 클리어

    let block = crate::tui::widgets::block_with_borders(
        Borders::ALL,
        state
            .domain
            .settings
            .as_ref()
            .map(|s| s.use_ascii_borders)
            .unwrap_or(false),
    )
    .title(" Configuration ")
    .style(Style::default().fg(p.warning));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let content = match state.ui.config.active_popup {
        ConfigPopup::Dashboard => {
            let mut s = "Master Settings Dashboard\n\n".to_string();
            let provider = state
                .domain
                .settings
                .as_ref()
                .map(|st| st.default_provider.as_str())
                .unwrap_or("None");
            let model = state
                .domain
                .settings
                .as_ref()
                .map(|st| st.default_model.as_str())
                .unwrap_or("None");
            let shell_policy = state
                .domain
                .settings
                .as_ref()
                .map(|st| format!("{:?}", st.shell_policy))
                .unwrap_or("None".to_string());
            let network_policy = state
                .domain
                .settings
                .as_ref()
                .map(|st| format!("{:?}", st.network_policy))
                .unwrap_or("None".to_string());

            let sandbox_policy = state
                .domain
                .settings
                .as_ref()
                .map(|st| {
                    if st.sandbox.enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    }
                })
                .unwrap_or("Disabled");

            let items = [
                format!("Provider: {}", provider),
                format!("Model: {}", model),
                format!("Shell Policy: {}", shell_policy),
                format!("Network Policy: {}", network_policy),
                format!("Sandbox: {}", sandbox_policy),
            ];

            for (i, item) in items.iter().enumerate() {
                if i == state.ui.config.cursor_index {
                    s.push_str(&format!(" > {}\n", item));
                } else {
                    s.push_str(&format!("   {}\n", item));
                }
            }
            s.push_str("\n(Up/Down to navigate, Enter to change, Esc to close)");

            // [v0.1.0-beta.9] 5차 감사 M-3: err_msg가 존재하면 Dashboard 하단에 표시
            if let Some(err) = &state.ui.config.err_msg {
                s.push_str(&format!("\n\n!! [Error] !!\n{}", err));
            }

            s
        }
        ConfigPopup::ProviderList => {
            let mut s = "Select Provider\n\n".to_string();
            let mut items = vec![
                "OpenAI".to_string(),
                "Anthropic".to_string(),
                "xAI".to_string(),
                "OpenRouter".to_string(),
                "Google (Gemini)".to_string(),
            ];

            if let Some(settings) = &state.domain.settings {
                for cp in &settings.custom_providers {
                    items.push(format!("Custom: {}", cp.id));
                }
            }

            for (i, item) in items.iter().enumerate() {
                if i == state.ui.config.cursor_index {
                    s.push_str(&format!(" > {}\n", item));
                } else {
                    s.push_str(&format!("   {}\n", item));
                }
            }
            s
        }
        ConfigPopup::ModelList => {
            if state.ui.config.is_loading {
                "Loading models...".to_string()
            } else if let Some(e) = &state.ui.config.err_msg {
                format!("Error loading models: {}", e)
            } else {
                let mut s = "Select Model\n\n".to_string();
                let start_idx = state.ui.config.cursor_index.saturating_sub(5);
                let end_idx = (start_idx + 10).min(state.ui.config.available_models.len());
                for (i, m) in state.ui.config.available_models[start_idx..end_idx]
                    .iter()
                    .enumerate()
                {
                    let real_i = start_idx + i;
                    if real_i == state.ui.config.cursor_index {
                        s.push_str(&format!(" > {}\n", m));
                    } else {
                        s.push_str(&format!("   {}\n", m));
                    }
                }
                s
            }
        }
    };

    // [v0.1.0-beta.22] word wrap 적용: 설정 팝업 내 긴 에러 메시지 등이 넘치지 않도록
    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(p.warning))
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(paragraph, inner_area);
}
