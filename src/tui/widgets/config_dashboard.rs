use crate::app::state::{AppState, ConfigPopup};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn draw_config(f: &mut Frame, state: &AppState) {
    let size = f.area();

    // Create a centered popup area
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

    f.render_widget(Clear, area); // Clear background

    let block = Block::default()
        .title(" Configuration ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let content = match state.config.active_popup {
        ConfigPopup::Dashboard => {
            let mut s = "Master Settings Dashboard\n\n".to_string();
            let provider = state
                .settings
                .as_ref()
                .map(|st| st.default_provider.as_str())
                .unwrap_or("None");
            let model = state
                .settings
                .as_ref()
                .map(|st| st.default_model.as_str())
                .unwrap_or("None");
            let shell_policy = state
                .settings
                .as_ref()
                .map(|st| format!("{:?}", st.shell_policy))
                .unwrap_or("None".to_string());

            let items = [
                format!("Provider: {}", provider),
                format!("Model: {}", model),
                format!("Shell Policy: {}", shell_policy),
            ];

            for (i, item) in items.iter().enumerate() {
                if i == state.config.cursor_index {
                    s.push_str(&format!(" > {}\n", item));
                } else {
                    s.push_str(&format!("   {}\n", item));
                }
            }
            s.push_str("\n(Up/Down to navigate, Enter to change, Esc to close)");
            s
        }
        ConfigPopup::ProviderList => {
            let mut s = "Select Provider\n\n".to_string();
            let items = ["OpenRouter", "Google (Gemini)"];
            for (i, item) in items.iter().enumerate() {
                if i == state.config.cursor_index {
                    s.push_str(&format!(" > {}\n", item));
                } else {
                    s.push_str(&format!("   {}\n", item));
                }
            }
            s
        }
        ConfigPopup::ModelList => {
            if state.config.is_loading {
                "Loading models...".to_string()
            } else if let Some(e) = &state.config.err_msg {
                format!("Error loading models: {}", e)
            } else {
                let mut s = "Select Model\n\n".to_string();
                let start_idx = state.config.cursor_index.saturating_sub(5);
                let end_idx = (start_idx + 10).min(state.config.available_models.len());
                for (i, m) in state.config.available_models[start_idx..end_idx]
                    .iter()
                    .enumerate()
                {
                    let real_i = start_idx + i;
                    if real_i == state.config.cursor_index {
                        s.push_str(&format!(" > {}\n", m));
                    } else {
                        s.push_str(&format!("   {}\n", m));
                    }
                }
                s
            }
        }
    };

    let paragraph = Paragraph::new(content).style(Style::default().fg(Color::Yellow));
    f.render_widget(paragraph, inner_area);
}
