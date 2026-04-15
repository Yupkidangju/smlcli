use crate::app::state::AppState;
use crate::tui::widgets::setting_wizard;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

pub fn draw(f: &mut Frame, state: &AppState) {
    let size = f.area();

    // 메인 레이아웃 분할: 상단바, 본문 영역(타임라인+인스펙터), 컴포저
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1), // 상태바(Top Bar)
                Constraint::Min(0),    // 타임라인 + 인스펙터
                Constraint::Length(3), // 하단 Composer
            ]
            .as_ref(),
        )
        .split(size);

    // 본문 영역을 타임라인과 인스펙터로 나눔 (만약 인스펙터 활성화 시 30% 영역 할당)
    draw_top_bar(f, state, chunks[0]);

    // 만약 승인 대기가 있거나, 사용자가 수동으로 인스펙터를 켰다면 분할 렌더링
    if state.show_inspector || state.approval.pending_tool.is_some() {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(70), // 타임라인 영역
                    Constraint::Percentage(30), // 인스펙터 패널 영역
                ]
                .as_ref(),
            )
            .split(chunks[1]);

        draw_timeline(f, state, main_chunks[0]);
        draw_inspector(f, state, main_chunks[1]);
    } else {
        // 인스펙터가 꺼져 있다면 Timeline 100% 사용
        draw_timeline(f, state, chunks[1]);
    }

    draw_composer(f, state, chunks[2]);

    if state.config.is_open {
        crate::tui::widgets::config_dashboard::draw_config(f, state);
    }
}

fn draw_top_bar(f: &mut Frame, state: &AppState, area: Rect) {
    let mode_str = match state.session.mode {
        crate::domain::session::AppMode::Plan => "PLAN",
        crate::domain::session::AppMode::Run => "RUN",
    };

    // [v0.1.0-beta.7] H-6: 하드코딩된 CWD와 정책을 실제 환경에서 동적 취득
    let budget = state.session.get_context_load_percentage();
    let provider = state
        .settings
        .as_ref()
        .map(|s| s.default_provider.clone())
        .unwrap_or_else(|| "None".to_string());
    let model = state
        .settings
        .as_ref()
        .map(|s| s.default_model.clone())
        .unwrap_or_else(|| "None".to_string());
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "?".to_string());
    let shell_policy_str = state
        .settings
        .as_ref()
        .map(|s| format!("{:?}", s.shell_policy))
        .unwrap_or_else(|| "None".to_string());

    let text = format!(
        " smlcli · {}/{} · {} · {} · Shell {} · {}% ctx · ✓ ",
        provider, model, cwd, mode_str, shell_policy_str, budget
    );
    let paragraph =
        Paragraph::new(text).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(paragraph, area);
}

fn draw_timeline(f: &mut Frame, state: &AppState, area: Rect) {
    if state.is_wizard_open {
        setting_wizard::draw_wizard(f, state, area);
        return;
    }

    let mut chat_history = String::new();
    for msg in &state.session.messages {
        // [v0.1.0-beta.7] 내부 시스템 프롬프트(pinned System)는 타임라인에서 숨김.
        // 사용자에게 내부 도구 프로토콜 지시문이 노출되는 것을 방지함.
        if msg.role == crate::providers::types::Role::System && msg.pinned {
            continue;
        }
        let role_str = match msg.role {
            crate::providers::types::Role::User => "User:\n",
            crate::providers::types::Role::Assistant => "AI:\n",
            crate::providers::types::Role::System => "System:\n",
            crate::providers::types::Role::Tool => "Tool:\n",
        };
        chat_history.push_str(role_str);
        chat_history.push_str(&msg.content);
        chat_history.push_str("\n\n");
    }

    if chat_history.is_empty() {
        chat_history = "Welcome to smlcli Timeline.\nPress Tab to switch PLAN/RUN. Type in Composer and push Enter.".to_string();
    }

    let block = Block::default().title("Timeline").borders(Borders::RIGHT);
    let paragraph = Paragraph::new(chat_history).block(block);
    f.render_widget(paragraph, area);
}

fn draw_inspector(f: &mut Frame, state: &AppState, area: Rect) {
    use crate::app::state::InspectorTab;
    let tabs_title = match state.active_inspector_tab {
        InspectorTab::Preview => "Inspector | [*Preview*] • [Diff] • [Search] • [Logs] • [Recent]",
        InspectorTab::Diff => "Inspector | [Preview] • [*Diff*] • [Search] • [Logs] • [Recent]",
        InspectorTab::Search => "Inspector | [Preview] • [Diff] • [*Search*] • [Logs] • [Recent]",
        InspectorTab::Logs => "Inspector | [Preview] • [Diff] • [Search] • [*Logs*] • [Recent]",
        InspectorTab::Recent => "Inspector | [Preview] • [Diff] • [Search] • [Logs] • [*Recent*]",
    };
    let block = Block::default().title(tabs_title).borders(Borders::LEFT);
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    use ratatui::text::{Line, Span};

    if let Some(tool) = &state.approval.pending_tool {
        let mut lines = Vec::new();
        lines.push(Line::from(vec![Span::styled(
            "⚠️ APPROVAL REQUIRED ⚠️",
            Style::default().fg(Color::Yellow),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "============================",
            Style::default().fg(Color::Yellow),
        )]));
        lines.push(Line::from(vec![Span::raw(format!("{:?}", tool))]));
        lines.push(Line::from(vec![Span::raw("")]));

        if let Some(diff) = &state.approval.diff_preview {
            lines.push(Line::from(vec![Span::styled(
                "[Diff Preview]",
                Style::default().fg(Color::Cyan),
            )]));
            for l in diff.lines() {
                if l.starts_with('+') {
                    lines.push(Line::from(vec![Span::styled(
                        l,
                        Style::default().fg(Color::Green),
                    )]));
                } else if l.starts_with('-') {
                    lines.push(Line::from(vec![Span::styled(
                        l,
                        Style::default().fg(Color::Red),
                    )]));
                } else if l.starts_with('@') {
                    lines.push(Line::from(vec![Span::styled(
                        l,
                        Style::default().fg(Color::Cyan),
                    )]));
                } else {
                    lines.push(Line::from(vec![Span::raw(l)]));
                }
            }
            lines.push(Line::from(vec![Span::raw("")]));
        }
        lines.push(Line::from(vec![Span::styled(
            ">> Press 'y' to Approve, 'n' to Reject.",
            Style::default().fg(Color::Yellow),
        )]));

        let paragraph = Paragraph::new(lines);
        f.render_widget(paragraph, inner_area);
    } else {
        let text = "No active tool context.\n\nRelevant files will appear here.";
        let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Yellow));
        f.render_widget(paragraph, inner_area);
    }
}

fn draw_composer(f: &mut Frame, state: &AppState, area: Rect) {
    let block = Block::default().title("Composer").borders(Borders::TOP);
    let content = if state.composer.input_buffer.is_empty() {
        "> (/, @, ! 사용 가능) Type your prompt here...".to_string()
    } else {
        format!("> {}", state.composer.input_buffer)
    };
    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);

    if state.fuzzy.is_open {
        let fuzzy_area = ratatui::layout::Rect {
            x: area.x + 2,
            y: area.y.saturating_sub(6),
            width: 40,
            height: 6,
        };
        let f_block = Block::default()
            .title("File Select (@)")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));

        let mut lines = Vec::new();
        lines.push(ratatui::text::Line::from(vec![ratatui::text::Span::raw(
            format!("> {}", state.fuzzy.input),
        )]));

        if state.fuzzy.matches.is_empty() {
            lines.push(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled("No files found", Style::default().fg(Color::Red)),
            ]));
        } else {
            for (i, m) in state.fuzzy.matches.iter().enumerate().take(3) {
                if i == state.fuzzy.cursor {
                    lines.push(ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled(
                            format!("▶ {}", m),
                            Style::default().fg(Color::Cyan),
                        ),
                    ]));
                } else {
                    lines.push(ratatui::text::Line::from(vec![ratatui::text::Span::raw(
                        format!("  {}", m),
                    )]));
                }
            }
        }
        let f_para = Paragraph::new(lines).block(f_block);
        f.render_widget(ratatui::widgets::Clear, fuzzy_area);
        f.render_widget(f_para, fuzzy_area);
    }
}
