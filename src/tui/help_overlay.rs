use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Borders, Clear, Paragraph},
};

use crate::app::state::{FocusedPane, UiState};

pub fn render(
    f: &mut Frame,
    area: Rect,
    state: &UiState,
    use_ascii_borders: bool,
    palette: &crate::tui::palette::Palette,
) {
    if !state.show_help_overlay {
        return;
    }

    let help_text = match state.focused_pane {
        FocusedPane::Composer => vec![
            ("Enter", "Send Message"),
            ("Shift+Enter", "New Line"),
            ("Tab", "Cycle Toolbar Chips"),
            ("Up/Down", "Navigate History"),
            ("/", "Open Slash Commands"),
            ("?", "Toggle Help"),
            ("Esc", "Close Help"),
        ],
        FocusedPane::Timeline => vec![
            ("Up/Down or J/K", "Scroll Timeline"),
            ("Enter", "Expand/Collapse Block"),
            ("y", "Copy Timeline Content"),
            ("Tab", "Focus Next Pane"),
            ("?", "Toggle Help"),
            ("Esc", "Close Help"),
        ],
        FocusedPane::Inspector => vec![
            ("Left/Right", "Switch Tabs (Preview/Diff/Search...)"),
            ("Up/Down", "Scroll Content"),
            ("Tab", "Focus Next Pane"),
            ("?", "Toggle Help"),
            ("Esc", "Close Help"),
        ],
        FocusedPane::Palette => vec![
            ("Up/Down", "Navigate Commands"),
            ("Enter", "Execute Command"),
            ("Esc", "Close Palette/Help"),
        ],
    };

    let popup_width = 40;
    let popup_height = (help_text.len() as u16) + 4; // Borders + title + padding

    // Center layout
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - popup_height) / 2),
            Constraint::Length(popup_height),
            Constraint::Percentage((100 - popup_height) / 2),
        ])
        .split(area);

    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - popup_width) / 2),
            Constraint::Length(popup_width),
            Constraint::Percentage((100 - popup_width) / 2),
        ])
        .split(vertical[1])[1];

    f.render_widget(Clear, popup_area);

    let block = crate::tui::widgets::block_with_borders(Borders::ALL, use_ascii_borders)
        .title(" Keyboard Shortcuts ")
        .title_alignment(Alignment::Center)
        .border_style(
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(palette.bg_panel)); // Or subtle background

    let mut lines = Vec::new();
    lines.push(Line::from("")); // top padding

    for (key, desc) in help_text {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:>15} ", key),
                Style::default()
                    .fg(palette.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("- {}", desc),
                Style::default().fg(palette.text_primary),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);
    f.render_widget(paragraph, popup_area);
}
