use crate::app::state::{AppState, WizardStep};
use crate::domain::provider::ProviderKind;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

pub fn draw_wizard(f: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default().title("Setup Wizard").borders(Borders::ALL);

    let content = match state.wizard.step {
        WizardStep::ProviderSelection => {
            let mut list = "[Step 1] Select Provider\n\n".to_string();
            let providers = ["OpenRouter", "Google (Gemini)"];
            for (i, p) in providers.iter().enumerate() {
                if i == state.wizard.cursor_index {
                    list.push_str(&format!(" > {}\n", p));
                } else {
                    list.push_str(&format!("   {}\n", p));
                }
            }
            list.push_str("\n(Use Up/Down to navigate, Enter to select)");
            list
        }
        WizardStep::ApiKeyInput => {
            let masked = "*".repeat(state.wizard.api_key_input.len());
            if state.wizard.is_loading_models {
                format!(
                    "[Step 2] Validating API Key...\n\
                Current buffer: {}\n\n\
                Please wait.",
                    masked
                )
            } else {
                let err_str = state.wizard.err_msg.as_deref().unwrap_or("");
                let err_disp = if err_str.is_empty() {
                    String::new()
                } else {
                    format!("\n\n!! [Validation Error] !!\n{}", err_str)
                };

                format!(
                    "[Step 2] Enter API Key\n\
                Current buffer: {}\n\n\
                Press Enter to submit and fetch available models.{}",
                    masked, err_disp
                )
            }
        }
        WizardStep::ModelSelection => {
            if state.wizard.is_loading_models {
                "[Step 3] Loading Available Models...\nPlease wait.".to_string()
            } else if let Some(e) = &state.wizard.err_msg {
                format!(
                    "[Error Loading Models]\n{}\nPress Esc to restart or exit.",
                    e
                )
            } else if state.wizard.available_models.is_empty() {
                "[Error Loading Models]\nNo models found. Please check API Key and Restart."
                    .to_string()
            } else {
                let mut list = "[Step 3] Select Model\n\n".to_string();
                let start_idx = state.wizard.cursor_index.saturating_sub(5); // Show items in window
                let end_idx = (start_idx + 10).min(state.wizard.available_models.len());
                for (i, m) in state.wizard.available_models[start_idx..end_idx]
                    .iter()
                    .enumerate()
                {
                    let real_i = start_idx + i;
                    if real_i == state.wizard.cursor_index {
                        list.push_str(&format!(" > {}\n", m));
                    } else {
                        list.push_str(&format!("   {}\n", m));
                    }
                }
                list.push_str(
                    format!(
                        "\n({}/{} - Use Up/Down and Enter to save completely)",
                        state.wizard.cursor_index + 1,
                        state.wizard.available_models.len()
                    )
                    .as_str(),
                );
                list
            }
        }
        WizardStep::Saving => {
            "Configuration saved successfully! Press Enter one more time to start smlcli."
                .to_string()
        }
    };

    // TODO: 현재 입력/선택 상태를 시각적으로 더 상세히 표현할 것 (목록 포커스 효과 등)
    let paragraph = Paragraph::new(content)
        .block(block)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(paragraph, area);
}
