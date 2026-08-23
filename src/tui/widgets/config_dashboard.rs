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
    let tr = &state.i18n;

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
    .title(format!(" {} ", tr.tr("config_title")))
    .style(Style::default().fg(p.warning));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let content = match state.ui.config.active_popup {
        ConfigPopup::Dashboard => {
            let mut s = format!("{}\n\n", tr.tr("config_dashboard"));
            let provider = state
                .domain
                .settings
                .as_ref()
                .map(|st| st.default_provider.as_str())
                .unwrap_or_else(|| tr.tr("none"));
            let model = state
                .domain
                .settings
                .as_ref()
                .map(|st| st.default_model.as_str())
                .unwrap_or_else(|| tr.tr("none"));
            let shell_policy = state
                .domain
                .settings
                .as_ref()
                .map(|st| format!("{:?}", st.shell_policy))
                .unwrap_or_else(|| tr.tr("none").to_string());
            let network_policy = state
                .domain
                .settings
                .as_ref()
                .map(|st| format!("{:?}", st.network_policy))
                .unwrap_or_else(|| tr.tr("none").to_string());

            let sandbox_policy = state
                .domain
                .settings
                .as_ref()
                .map(|st| {
                    if st.sandbox.enabled {
                        tr.tr("enabled")
                    } else {
                        tr.tr("disabled")
                    }
                })
                .unwrap_or_else(|| tr.tr("disabled"));

            let items = [
                format!("{}: {}", tr.tr("label_provider"), provider),
                format!("{}: {}", tr.tr("label_model"), model),
                format!("{}: {}", tr.tr("label_shell_policy"), shell_policy),
                format!("{}: {}", tr.tr("label_network_policy"), network_policy),
                format!("{}: {}", tr.tr("label_sandbox"), sandbox_policy),
            ];

            for (i, item) in items.iter().enumerate() {
                if i == state.ui.config.cursor_index {
                    s.push_str(&format!(" > {}\n", item));
                } else {
                    s.push_str(&format!("   {}\n", item));
                }
            }
            s.push_str(&format!("\n({})", tr.tr("navigate_hint")));

            // [v0.1.0-beta.9] 5차 감사 M-3: err_msg가 존재하면 Dashboard 하단에 표시
            if let Some(err) = &state.ui.config.err_msg {
                s.push_str(&format!("\n\n!! [Error] !!\n{}", err));
            }

            s
        }
        ConfigPopup::ProviderList => {
            let mut s = format!("{}\n\n", tr.tr("select_provider"));
            let mut items = vec![
                "OpenAI".to_string(),
                "Anthropic".to_string(),
                "xAI".to_string(),
                "OpenRouter".to_string(),
                "Google (Gemini)".to_string(),
                "LM Studio".to_string(),
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
                tr.tr("loading_models").to_string()
            } else if let Some(e) = &state.ui.config.err_msg {
                format!("{}: {}", tr.tr("error_loading_models"), e)
            } else {
                let mut s = format!("{}\n\n", tr.tr("select_model"));
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
