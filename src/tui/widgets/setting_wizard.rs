// [v0.1.0-beta.7] Setup Wizard 위젯.
// [v0.1.0-beta.21] 하드코딩 Color::Cyan을 state.palette() 동적 참조로 전환.

use crate::app::state::{AppState, WizardStep};
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph},
};

pub fn draw_wizard(f: &mut Frame, state: &AppState, area: Rect) {
    // [v0.1.0-beta.21] 동적 팔레트 참조: 테마 전환 즉시 반영
    let p = state.palette();

    let block = Block::default().title("Setup Wizard").borders(Borders::ALL);

    let content = match state.ui.wizard.step {
        WizardStep::ProviderSelection => {
            let mut list = "[Step 1] Select Provider\n\n".to_string();
            let providers = ["OpenRouter", "Google (Gemini)"];
            for (i, prov) in providers.iter().enumerate() {
                if i == state.ui.wizard.cursor_index {
                    list.push_str(&format!(" > {}\n", prov));
                } else {
                    list.push_str(&format!("   {}\n", prov));
                }
            }
            list.push_str("\n(Use Up/Down to navigate, Enter to select)");
            list
        }
        WizardStep::ApiKeyInput => {
            let masked = "*".repeat(state.ui.wizard.api_key_input.len());
            if state.ui.wizard.is_loading_models {
                format!(
                    "[Step 2] Validating API Key...\n\
                Current buffer: {}\n\n\
                Please wait.",
                    masked
                )
            } else {
                let err_str = state.ui.wizard.err_msg.as_deref().unwrap_or("");
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
            if state.ui.wizard.is_loading_models {
                "[Step 3] Loading Available Models...\nPlease wait.".to_string()
            } else if let Some(e) = &state.ui.wizard.err_msg {
                format!(
                    "[Error Loading Models]\n{}\nPress Esc to restart or exit.",
                    e
                )
            } else if state.ui.wizard.available_models.is_empty() {
                "[Error Loading Models]\nNo models found. Please check API Key and Restart."
                    .to_string()
            } else {
                let mut list = "[Step 3] Select Model\n\n".to_string();
                let start_idx = state.ui.wizard.cursor_index.saturating_sub(5); // Show items in window
                let end_idx = (start_idx + 10).min(state.ui.wizard.available_models.len());
                for (i, m) in state.ui.wizard.available_models[start_idx..end_idx]
                    .iter()
                    .enumerate()
                {
                    let real_i = start_idx + i;
                    if real_i == state.ui.wizard.cursor_index {
                        list.push_str(&format!(" > {}\n", m));
                    } else {
                        list.push_str(&format!("   {}\n", m));
                    }
                }
                list.push_str(
                    format!(
                        "\n({}/{} - Use Up/Down and Enter to save completely)",
                        state.ui.wizard.cursor_index + 1,
                        state.ui.wizard.available_models.len()
                    )
                    .as_str(),
                );
                list
            }
        }
        WizardStep::Saving => {
            // [v0.1.0-beta.9] 5차 감사 Low: 문구가 실제 동작과 일치하도록 수정.
            // Enter를 눌러야 저장이 실행되므로, "saved" 대신 "Press Enter to save" 표현.
            if let Some(err) = &state.ui.wizard.err_msg {
                format!("[Save Error]\n{}\n\nPress Esc to go back and retry.", err)
            } else {
                "Ready to save configuration.\nPress Enter to save and start smlcli.".to_string()
            }
        }
    };

    // [v0.1.0-beta.22] word wrap 적용: 위자드 에러 메시지 등이 넘치지 않도록
    let paragraph = Paragraph::new(content)
        .block(block)
        .style(Style::default().fg(p.info))
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(paragraph, area);
}
