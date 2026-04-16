use crate::app::state::{AppState, TimelineEntryKind, ToolStatus};
use crate::tui::palette as pal;
use crate::tui::widgets::setting_wizard;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
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

    // [v0.1.0-beta.18] context budget 색상을 사용량에 따라 변경
    let ctx_color = if budget >= 85 {
        pal::DANGER
    } else if budget >= 70 {
        pal::WARNING
    } else {
        pal::TEXT_SECONDARY
    };

    let text = format!(
        " smlcli · {}/{} · {} · {} · Shell {} · {}% ctx · ✓ ",
        provider, model, cwd, mode_str, shell_policy_str, budget
    );

    // [v0.1.0-beta.18] Semantic Palette 적용: 상태바 배경에 BG_PANEL 사용
    let paragraph =
        Paragraph::new(text).style(Style::default().bg(pal::BG_PANEL).fg(pal::TEXT_PRIMARY));
    f.render_widget(paragraph, area);
}

fn draw_timeline(f: &mut Frame, state: &AppState, area: Rect) {
    if state.is_wizard_open {
        setting_wizard::draw_wizard(f, state, area);
        return;
    }

    // [v0.1.0-beta.18] Phase 9-A: timeline_entries 기반 렌더링.
    // timeline이 비어있으면 기존 session.messages 폴백 (하위 호환).
    let mut lines: Vec<Line> = Vec::new();

    if !state.timeline.is_empty() {
        // === 타임라인 엔트리 기반 렌더링 ===
        for entry in &state.timeline {
            match &entry.kind {
                TimelineEntryKind::UserMessage(msg) => {
                    lines.push(Line::from(vec![
                        Span::styled("User:\n", Style::default().fg(pal::ACCENT)),
                    ]));
                    lines.push(Line::from(msg.as_str()));
                    lines.push(Line::from(""));
                }
                TimelineEntryKind::AssistantMessage(msg) => {
                    lines.push(Line::from(vec![
                        Span::styled("AI:\n", Style::default().fg(pal::SUCCESS)),
                    ]));
                    // 도구 호출 JSON 필터링 유지
                    let display = filter_tool_json(msg);
                    lines.push(Line::from(display));
                    lines.push(Line::from(""));
                }
                TimelineEntryKind::AssistantDelta(buf) => {
                    // SSE 스트리밍 중간 결과: 실시간 표시
                    if !buf.is_empty() {
                        lines.push(Line::from(vec![
                            Span::styled("AI: ", Style::default().fg(pal::SUCCESS)),
                        ]));
                        lines.push(Line::from(buf.as_str()));
                    }
                }
                TimelineEntryKind::SystemNotice(msg) => {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("ℹ  {}", msg),
                            Style::default().fg(pal::INFO),
                        ),
                    ]));
                    lines.push(Line::from(""));
                }
                TimelineEntryKind::ToolCard { tool_name, status, summary } => {
                    // [v0.1.0-beta.18] A-4: tick 기반 배지 애니메이션
                    let (badge, badge_color) = match status {
                        ToolStatus::Queued => ("◻", pal::MUTED),
                        ToolStatus::Running => {
                            let frame = pal::TOOL_BADGE[(state.tick_count as usize) % 2];
                            let s: String = frame.to_string().chars().collect();
                            // 임시 변수로 lifetime 확보
                            (if (state.tick_count % 2) == 0 { "●" } else { "○" }, pal::WARNING)
                        }
                        ToolStatus::Done => ("✅", pal::SUCCESS),
                        ToolStatus::Error => ("❌", pal::DANGER),
                    };
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("{} {} ", badge, tool_name),
                            Style::default().fg(badge_color),
                        ),
                    ]));
                    if !summary.is_empty() {
                        for sl in summary.lines() {
                            lines.push(Line::from(vec![
                                Span::styled(
                                    format!("   {}", sl),
                                    Style::default().fg(pal::TEXT_SECONDARY),
                                ),
                            ]));
                        }
                    }
                    lines.push(Line::from(""));
                }
                TimelineEntryKind::ApprovalCard { tool_name, detail } => {
                    // [v0.1.0-beta.18] 승인 대기 카드: tick 기반 pulse
                    let pulse_color = if (state.tick_count % 6) < 3 {
                        pal::WARNING
                    } else {
                        pal::TEXT_PRIMARY
                    };
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("⚠  승인 대기: {} ", tool_name),
                            Style::default().fg(pulse_color),
                        ),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("   {}", detail),
                            Style::default().fg(pal::TEXT_SECONDARY),
                        ),
                    ]));
                    lines.push(Line::from(""));
                }
                TimelineEntryKind::CompactSummary(msg) => {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("📋 Context Compacted: {}", &msg[..msg.len().min(80)]),
                            Style::default().fg(pal::MUTED),
                        ),
                    ]));
                    lines.push(Line::from(""));
                }
            }
        }
    } else {
        // === 폴백: 기존 session.messages 기반 ===
        for msg in &state.session.messages {
            if msg.role == crate::providers::types::Role::System && msg.pinned {
                continue;
            }
            let (role_str, role_color) = match msg.role {
                crate::providers::types::Role::User => ("User:\n", pal::ACCENT),
                crate::providers::types::Role::Assistant => ("AI:\n", pal::SUCCESS),
                crate::providers::types::Role::System => ("System:\n", pal::INFO),
                crate::providers::types::Role::Tool => ("Tool:\n", pal::MUTED),
            };
            lines.push(Line::from(vec![
                Span::styled(role_str, Style::default().fg(role_color)),
            ]));
            let display_content = filter_tool_json(&msg.content);
            lines.push(Line::from(display_content));
            lines.push(Line::from(""));
        }
    }

    // [v0.1.0-beta.18] A-4: tick 기반 thinking 스피너
    if state.is_thinking {
        let spinner = pal::SPINNER_FRAMES[(state.tick_count as usize) % 4];
        lines.push(Line::from(vec![
            Span::styled(
                format!("{} AI가 응답을 생성하고 있습니다...", spinner),
                Style::default().fg(pal::INFO),
            ),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from("Welcome to smlcli Timeline."));
        lines.push(Line::from("Press Tab to switch PLAN/RUN. Type in Composer and push Enter."));
    }

    let block = Block::default().title("Timeline").borders(Borders::RIGHT);
    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

/// [v0.1.0-beta.16] AI 응답 텍스트에서 ```json ... ``` 도구 호출 블록을 필터링.
/// 도구 호출 JSON을 사용자 친화적 메시지로 대체.
fn filter_tool_json(content: &str) -> String {
    let mut result = String::new();
    let mut remaining = content;

    while let Some(start) = remaining.find("```json") {
        // JSON 블록 이전 텍스트 추가
        result.push_str(&remaining[..start]);

        let after_marker = &remaining[start + 7..];
        if let Some(end) = after_marker.find("```") {
            // JSON 블록을 사용자 친화적 메시지로 대체
            let json_str = after_marker[..end].trim();
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(tool_name) = parsed.get("tool").and_then(|v| v.as_str()) {
                    result.push_str(&format!("\n⚙️  [{}] 도구 호출 실행 중...", tool_name));
                    // 명령어나 경로 등 핵심 정보만 간략히 표시
                    if let Some(cmd) = parsed.get("command").and_then(|v| v.as_str()) {
                        result.push_str(&format!("\n   ↳ $ {}", cmd));
                    }
                    if let Some(path) = parsed.get("path").and_then(|v| v.as_str()) {
                        result.push_str(&format!("\n   ↳ {}", path));
                    }
                    result.push('\n');
                } else {
                    result.push_str("\n⚙️  도구 호출 실행 중...\n");
                }
            } else {
                // JSON 파싱 실패 시 원문 그대로 표시
                result.push_str(&remaining[start..start + 7 + end + 3]);
            }
            remaining = &after_marker[end + 3..];
        } else {
            // 닫는 ``` 없으면 나머지 그대로
            result.push_str(&remaining[start..]);
            remaining = "";
        }
    }
    result.push_str(remaining);
    result
}

fn draw_inspector(f: &mut Frame, state: &AppState, area: Rect) {
    use crate::app::state::InspectorTab;

    // [v0.1.0-beta.18] Semantic Palette 적용: 활성 탭을 ACCENT 색상으로 강조
    let tab_names = ["Preview", "Diff", "Search", "Logs", "Recent"];
    let active_idx = match state.active_inspector_tab {
        InspectorTab::Preview => 0,
        InspectorTab::Diff => 1,
        InspectorTab::Search => 2,
        InspectorTab::Logs => 3,
        InspectorTab::Recent => 4,
    };
    let tabs_title: String = tab_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            if i == active_idx {
                format!("[*{}*]", name)
            } else {
                format!("[{}]", name)
            }
        })
        .collect::<Vec<_>>()
        .join(" · ");
    let title = format!("Inspector | {}", tabs_title);

    let block = Block::default().title(title).borders(Borders::LEFT);
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // 승인 대기 중이면 Diff 탭 강제 표시
    if let Some(tool) = &state.approval.pending_tool {
        let mut lines = Vec::new();
        lines.push(Line::from(vec![Span::styled(
            "⚠️ APPROVAL REQUIRED ⚠️",
            Style::default().fg(pal::WARNING),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "============================",
            Style::default().fg(pal::WARNING),
        )]));
        lines.push(Line::from(vec![Span::raw(format!("{:?}", tool))]));
        lines.push(Line::from(vec![Span::raw("")]));

        if let Some(diff) = &state.approval.diff_preview {
            lines.push(Line::from(vec![Span::styled(
                "[Diff Preview]",
                Style::default().fg(pal::INFO),
            )]));
            for l in diff.lines() {
                if l.starts_with('+') {
                    lines.push(Line::from(vec![Span::styled(
                        l,
                        Style::default().fg(pal::SUCCESS),
                    )]));
                } else if l.starts_with('-') {
                    lines.push(Line::from(vec![Span::styled(
                        l,
                        Style::default().fg(pal::DANGER),
                    )]));
                } else if l.starts_with('@') {
                    lines.push(Line::from(vec![Span::styled(
                        l,
                        Style::default().fg(pal::INFO),
                    )]));
                } else {
                    lines.push(Line::from(vec![Span::raw(l)]));
                }
            }
            lines.push(Line::from(vec![Span::raw("")]));
        }
        lines.push(Line::from(vec![Span::styled(
            ">> Press 'y' to Approve, 'n' to Reject.",
            Style::default().fg(pal::WARNING),
        )]));

        let paragraph = Paragraph::new(lines);
        f.render_widget(paragraph, inner_area);
        return;
    }

    // [v0.1.0-beta.18] A-5: 탭별 실체 콘텐츠 렌더링
    match state.active_inspector_tab {
        InspectorTab::Logs => {
            // logs_buffer의 최근 항목 표시
            let mut lines = Vec::new();
            if state.logs_buffer.is_empty() {
                lines.push(Line::from(vec![Span::styled(
                    "No logs yet.",
                    Style::default().fg(pal::MUTED),
                )]));
            } else {
                // 마지막 50줄만 표시
                let start = state.logs_buffer.len().saturating_sub(50);
                for log in &state.logs_buffer[start..] {
                    lines.push(Line::from(vec![Span::styled(
                        log.as_str(),
                        Style::default().fg(pal::TEXT_SECONDARY),
                    )]));
                }
            }
            let paragraph = Paragraph::new(lines);
            f.render_widget(paragraph, inner_area);
        }
        InspectorTab::Diff => {
            let text = "No pending diffs.\n\nDiffs will appear here after file write proposals.";
            let paragraph = Paragraph::new(text).style(Style::default().fg(pal::MUTED));
            f.render_widget(paragraph, inner_area);
        }
        InspectorTab::Search => {
            let text = "No search results.\n\nGrep results will appear here.";
            let paragraph = Paragraph::new(text).style(Style::default().fg(pal::MUTED));
            f.render_widget(paragraph, inner_area);
        }
        InspectorTab::Recent => {
            let text = "No recent files.\n\nRecently accessed files will appear here.";
            let paragraph = Paragraph::new(text).style(Style::default().fg(pal::MUTED));
            f.render_widget(paragraph, inner_area);
        }
        _ => {
            // Preview (기본)
            let text = "No active file context.\n\nRelevant files will appear here.";
            let paragraph = Paragraph::new(text).style(Style::default().fg(pal::MUTED));
            f.render_widget(paragraph, inner_area);
        }
    }
}

fn draw_composer(f: &mut Frame, state: &AppState, area: Rect) {
    // [v0.1.0-beta.18] Semantic Palette 적용
    let block = Block::default()
        .title("Composer")
        .borders(Borders::TOP)
        .style(Style::default().fg(pal::TEXT_PRIMARY));
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
            .style(Style::default().bg(pal::BG_ELEVATED).fg(pal::TEXT_PRIMARY));

        let mut lines = Vec::new();
        lines.push(Line::from(vec![Span::raw(
            format!("> {}", state.fuzzy.input),
        )]));

        if state.fuzzy.matches.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("No files found", Style::default().fg(pal::DANGER)),
            ]));
        } else {
            for (i, m) in state.fuzzy.matches.iter().enumerate().take(3) {
                if i == state.fuzzy.cursor {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("▶ {}", m),
                            Style::default().fg(pal::ACCENT),
                        ),
                    ]));
                } else {
                    lines.push(Line::from(vec![Span::raw(
                        format!("  {}", m),
                    )]));
                }
            }
        }
        let f_para = Paragraph::new(lines).block(f_block);
        f.render_widget(ratatui::widgets::Clear, fuzzy_area);
        f.render_widget(f_para, fuzzy_area);
    }

    // [v0.1.0-beta.16] 슬래시 커맨드 자동완성 메뉴: Composer 위에 팝업으로 표시
    if state.slash_menu.is_open {
        let menu_height = (state.slash_menu.matches.len() as u16 + 2).min(13);
        let menu_area = ratatui::layout::Rect {
            x: area.x + 2,
            y: area.y.saturating_sub(menu_height),
            width: 35,
            height: menu_height,
        };
        let menu_block = Block::default()
            .title("Commands (/)")
            .borders(Borders::ALL)
            .style(Style::default().bg(pal::BG_ELEVATED).fg(pal::TEXT_PRIMARY));

        let mut lines = Vec::new();
        for (i, (cmd, desc)) in state.slash_menu.matches.iter().enumerate() {
            let line_text = format!("{:<12} {}", cmd, desc);
            if i == state.slash_menu.cursor {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("▶ {}", line_text),
                        Style::default().fg(pal::ACCENT),
                    ),
                ]));
            } else {
                lines.push(Line::from(vec![Span::raw(
                    format!("  {}", line_text),
                )]));
            }
        }
        let menu_para = Paragraph::new(lines).block(menu_block);
        f.render_widget(ratatui::widgets::Clear, menu_area);
        f.render_widget(menu_para, menu_area);
    }
}
