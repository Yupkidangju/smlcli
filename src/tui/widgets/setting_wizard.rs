// [v0.1.0-beta.7] Setup Wizard 위젯.
// [v0.1.0-beta.21] 하드코딩 Color::Cyan을 state.palette() 동적 참조로 전환.

use crate::app::state::{AppState, WizardStep};
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Borders, Paragraph},
};

pub fn draw_wizard(f: &mut Frame, state: &AppState, area: Rect) {
    // [v0.1.0-beta.21] 동적 팔레트 참조: 테마 전환 즉시 반영
    let p = state.palette();

    let block = crate::tui::widgets::block_with_borders(
        Borders::ALL,
        state
            .domain
            .settings
            .as_ref()
            .map(|s| s.use_ascii_borders)
            .unwrap_or(false),
    )
    .title("Setup Wizard");

    let content = match state.ui.wizard.step {
        WizardStep::ProviderSelection => {
            let mut list = "[Step 1] Select Provider\n\n".to_string();
            // [v3.7.2] LM Studio 로컬 프로바이더 선택 옵션 추가
            let providers = [
                "OpenAI",
                "Anthropic",
                "xAI",
                "OpenRouter",
                "Google (Gemini)",
                "LM Studio",
            ];
            for (i, prov) in providers.iter().enumerate() {
                if i == state.ui.wizard.cursor_index {
                    list.push_str(&format!(" > {}\n", prov));
                } else {
                    list.push_str(&format!("   {}\n", prov));
                }
            }
            if let Some(err) = &state.ui.wizard.err_msg {
                list.push_str("\n!! [Startup Error] !!\n");
                list.push_str(err);
                list.push('\n');
            }
            list.push_str("\n(Use Up/Down to navigate, Enter to select)");
            list
        }
        // [v3.7.2] LmStudio의 base_url을 입력받기 위한 신규 위저드 단계 UI 렌더링 로직 추가
        WizardStep::BaseUrlInput => {
            let rendered =
                crate::tui::widgets::input_field::InputField::new(&state.ui.wizard.base_url_input)
                    .with_password(false)
                    .render();
            if state.ui.wizard.is_loading_models {
                format!(
                    "[Step 2] Validating Base URL...\n\
                Current URL: {}\n\n\
                Please wait.",
                    rendered
                )
            } else {
                let err_str = state.ui.wizard.err_msg.as_deref().unwrap_or("");
                let err_disp = if err_str.is_empty() {
                    String::new()
                } else {
                    format!("\n\n!! [Validation Error] !!\n{}", err_str)
                };

                format!(
                    "[Step 2] Enter LM Studio Base URL\n\
                Current Base URL: {}\n\n\
                Press Enter to validate URL and fetch available models.{}",
                    rendered, err_disp
                )
            }
        }
        WizardStep::ApiKeyInput => {
            let masked =
                crate::tui::widgets::input_field::InputField::new(&state.ui.wizard.api_key_input)
                    .with_password(true)
                    .render();
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
            if state.ui.wizard.is_custom_model_mode {
                // [v3.7.2] 모델 수동 직접 입력 모드 텍스트 박스 렌더링
                let rendered =
                    crate::tui::widgets::input_field::InputField::new(&state.ui.wizard.custom_model_input)
                        .with_password(false)
                        .render();
                let err_str = state.ui.wizard.err_msg.as_deref().unwrap_or("");
                let err_disp = if err_str.is_empty() {
                    String::new()
                } else {
                    format!("\n\n!! [Input Error] !!\n{}", err_str)
                };
                format!(
                    "[Step 3] Enter Custom Model Name (직접 수동 입력)\n\
                Current Model Name: {}\n\n\
                Press Enter to submit and proceed to save.{}",
                    rendered, err_disp
                )
            } else if state.ui.wizard.is_loading_models {
                "[Step 3] Loading Available Models...\nPlease wait.".to_string()
            } else if let Some(e) = &state.ui.wizard.err_msg {
                format!(
                    "[Error Loading Models]\n{}\nPress Esc to restart or exit.",
                    e
                )
            } else if state.ui.wizard.available_models.is_empty() {
                "[Error Loading Models]\nNo models found. Please check API Key/Base URL and Restart."
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
            if state.ui.wizard.is_loading_models {
                "Saving configuration...\nPlease wait.".to_string()
            } else if let Some(err) = &state.ui.wizard.err_msg {
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
