// [v0.1.0-beta.18] Phase 9-A: 메인 TUI 레이아웃 모듈.
// [v0.1.0-beta.21] 모든 색상 참조를 정적 pal:: 상수에서 state.palette() 동적 참조로 전환.
//   /theme 명령어로 전환된 테마가 화면에 즉시 반영됨.
//   designs.md §21.4 구현 아키텍처 참조.

use crate::app::state::{AppState, TimelineEntryKind, ToolStatus};
use crate::tui::palette as pal;
use crate::tui::widgets::setting_wizard;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, state: &AppState) {
    let size = f.area();

    // 메인 레이아웃 분할: 상단바, 본문 영역(타임라인+인스펙터), 커맨드 상태바, 컴포저
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1), // 상태바(Top Bar)
                Constraint::Min(0),    // 타임라인 + 인스펙터
                Constraint::Length(1), // 상태/명령 바 (Command Status Bar)
                Constraint::Length(3), // 하단 Composer
            ]
            .as_ref(),
        )
        .split(size);

    // 본문 영역을 타임라인과 인스펙터로 나눔 (만약 인스펙터 활성화 시 30% 영역 할당)
    draw_top_bar(f, state, chunks[0]);

    // 만약 승인 대기가 있거나, 사용자가 수동으로 인스펙터를 켰다면 분할 렌더링
    if state.ui.show_inspector || state.runtime.approval.pending_tool.is_some() {
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

    draw_command_status_bar(f, state, chunks[2]);
    draw_composer(f, state, chunks[3]);

    if state.ui.config.is_open {
        crate::tui::widgets::config_dashboard::draw_config(f, state);
    }
}

fn draw_top_bar(f: &mut Frame, state: &AppState, area: Rect) {
    // [v0.1.0-beta.21] 동적 팔레트 참조: 테마 전환 즉시 반영
    let p = state.palette();

    let mode_str = match state.domain.session.mode {
        crate::domain::session::AppMode::Plan => "PLAN",
        crate::domain::session::AppMode::Run => "RUN",
    };

    // [v0.1.0-beta.7] H-6: 하드코딩된 CWD와 정책을 실제 환경에서 동적 취득
    let budget = state.domain.session.get_context_load_percentage();
    let provider = state
        .domain
        .settings
        .as_ref()
        .map(|s| s.default_provider.clone())
        .unwrap_or_else(|| "None".to_string());
    let model = state
        .domain
        .settings
        .as_ref()
        .map(|s| s.default_model.clone())
        .unwrap_or_else(|| "None".to_string());
    let cwd = std::env::current_dir()
        .map(|pp| pp.display().to_string())
        .unwrap_or_else(|_| "?".to_string());
    let shell_policy_str = state
        .domain
        .settings
        .as_ref()
        .map(|s| format!("{:?}", s.shell_policy))
        .unwrap_or_else(|| "None".to_string());

    // [v0.1.0-beta.18] context budget 색상을 사용량에 따라 변경
    let ctx_color = if budget >= 85 {
        p.danger
    } else if budget >= 70 {
        p.warning
    } else {
        p.text_secondary
    };

    // [v0.1.0-beta.18] Semantic Palette 적용: 상태바 배경에 bg_panel,
    // 컨텍스트 사용량 비율에 따라 ctx% 텍스트 색상 차등 적용.
    let prefix = format!(
        " smlcli · {}/{} · {} · {} · Shell {} · ",
        provider, model, cwd, mode_str, shell_policy_str
    );
    let ctx_span = format!("{}% ctx", budget);
    let suffix = " · ✓ ";
    let status_line = Line::from(vec![
        Span::styled(prefix, Style::default().fg(p.text_primary)),
        Span::styled(ctx_span, Style::default().fg(ctx_color)),
        Span::styled(suffix, Style::default().fg(p.text_primary)),
    ]);
    let paragraph = Paragraph::new(status_line).style(Style::default().bg(p.bg_panel));
    f.render_widget(paragraph, area);
}

fn draw_timeline(f: &mut Frame, state: &AppState, area: Rect) {
    if state.ui.is_wizard_open {
        setting_wizard::draw_wizard(f, state, area);
        return;
    }

    // [v0.1.0-beta.21] 동적 팔레트 참조
    let p = state.palette();

    // [v0.1.0-beta.18] Phase 9-A: timeline_entries 기반 렌더링.
    // timeline이 비어있으면 기존 session.messages 폴백 (하위 호환).
    let mut lines: Vec<Line> = Vec::new();

    if !state.ui.timeline.is_empty() {
        // === 타임라인 엔트리 기반 렌더링 ===
        for entry in &state.ui.timeline {
            match &entry.kind {
                TimelineEntryKind::UserMessage(msg) => {
                    lines.push(Line::from(vec![Span::styled(
                        "User:\n",
                        Style::default().fg(p.accent),
                    )]));
                    lines.push(Line::from(msg.as_str()));
                    lines.push(Line::from(""));
                }
                TimelineEntryKind::AssistantMessage(msg) => {
                    lines.push(Line::from(vec![Span::styled(
                        "AI:\n",
                        Style::default().fg(p.success),
                    )]));
                    // 도구 호출 JSON 필터링 유지
                    let display = filter_tool_json(msg);
                    lines.push(Line::from(display));
                    lines.push(Line::from(""));
                }
                TimelineEntryKind::AssistantDelta(buf) => {
                    // SSE 스트리밍 중간 결과: 실시간 표시
                    if !buf.is_empty() {
                        lines.push(Line::from(vec![Span::styled(
                            "AI: ",
                            Style::default().fg(p.success),
                        )]));
                        lines.push(Line::from(buf.as_str()));
                    }
                }
                TimelineEntryKind::SystemNotice(msg) => {
                    for (i, line) in msg.lines().enumerate() {
                        let prefix = if i == 0 { "ℹ  " } else { "   " };
                        lines.push(Line::from(vec![Span::styled(
                            format!("{}{}", prefix, line),
                            Style::default().fg(p.info),
                        )]));
                    }
                    lines.push(Line::from(""));
                }
                TimelineEntryKind::ToolCard {
                    tool_name,
                    status,
                    summary,
                } => {
                    // [v0.1.0-beta.18] A-4: tick 기반 배지 애니메이션
                    let (badge, badge_color) = match status {
                        ToolStatus::Queued => ("◻", p.muted),
                        ToolStatus::Running => {
                            // [v0.1.0-beta.18] tick 기반 배지 깜빡임
                            (
                                if state.ui.tick_count.is_multiple_of(2) {
                                    "●"
                                } else {
                                    "○"
                                },
                                p.warning,
                            )
                        }
                        ToolStatus::Done => ("✅", p.success),
                        ToolStatus::Error => ("❌", p.danger),
                    };
                    lines.push(Line::from(vec![Span::styled(
                        format!("{} {} ", badge, tool_name),
                        Style::default().fg(badge_color),
                    )]));
                    if !summary.is_empty() {
                        for sl in summary.lines() {
                            lines.push(Line::from(vec![Span::styled(
                                format!("   {}", sl),
                                Style::default().fg(p.text_secondary),
                            )]));
                        }
                    }
                    lines.push(Line::from(""));
                }
                TimelineEntryKind::ApprovalCard { tool_name, detail } => {
                    // [v0.1.0-beta.18] 승인 대기 카드: tick 기반 pulse
                    let pulse_color = if (state.ui.tick_count % 6) < 3 {
                        p.warning
                    } else {
                        p.text_primary
                    };
                    lines.push(Line::from(vec![Span::styled(
                        format!("⚠  승인 대기: {} ", tool_name),
                        Style::default().fg(pulse_color),
                    )]));
                    lines.push(Line::from(vec![Span::styled(
                        format!("   {}", detail),
                        Style::default().fg(p.text_secondary),
                    )]));
                    lines.push(Line::from(""));
                }
                TimelineEntryKind::CompactSummary(msg) => {
                    lines.push(Line::from(vec![Span::styled(
                        format!("📋 Context Compacted: {}", &msg[..msg.len().min(80)]),
                        Style::default().fg(p.muted),
                    )]));
                    lines.push(Line::from(""));
                }
            }
        }
    } else {
        // === 폴백: 기존 session.messages 기반 ===
        for msg in &state.domain.session.messages {
            if msg.role == crate::providers::types::Role::System && msg.pinned {
                continue;
            }
            let (role_str, role_color) = match msg.role {
                crate::providers::types::Role::User => ("User:\n", p.accent),
                crate::providers::types::Role::Assistant => ("AI:\n", p.success),
                crate::providers::types::Role::System => ("System:\n", p.info),
                crate::providers::types::Role::Tool => ("Tool:\n", p.muted),
            };
            lines.push(Line::from(vec![Span::styled(
                role_str,
                Style::default().fg(role_color),
            )]));
            let content_str = msg.content.as_deref().unwrap_or_default();
            let display_content = filter_tool_json(content_str);
            lines.push(Line::from(display_content));
            lines.push(Line::from(""));
        }
    }

    // [v0.1.0-beta.18] A-4: tick 기반 thinking 스피너
    if state.runtime.is_thinking {
        let spinner = pal::SPINNER_FRAMES[(state.ui.tick_count as usize) % 4];
        lines.push(Line::from(vec![Span::styled(
            format!("{} AI가 응답을 생성하고 있습니다...", spinner),
            Style::default().fg(p.info),
        )]));
    }

    if lines.is_empty() {
        lines.push(Line::from("Welcome to smlcli Timeline."));
        lines.push(Line::from(
            "Press Tab to switch PLAN/RUN. Type in Composer and push Enter.",
        ));
    }

    let block = Block::default().title("Timeline").borders(Borders::RIGHT);
    // [v0.1.0-beta.22] word wrap + 스크롤 오프셋 적용.
    // 긴 응답이 가로로 넘치지 않고, 세로 스크롤로 탐색 가능.
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((state.ui.timeline_scroll, 0));
    f.render_widget(paragraph, area);
}

/// [v0.1.0-beta.22] AI 응답 텍스트에서 도구 호출 JSON을 필터링.
/// - fenced ```json 블록: 사용자 친화적 요약으로 대체
/// - bare JSON (fenced가 아닌 raw JSON 객체): "tool" 키가 있으면 요약으로 대체,
///   없으면 원문 유지. 도구 스키마가 사용자에게 노출되지 않도록 함.
pub(crate) fn filter_tool_json(content: &str) -> String {
    let mut result = String::new();
    let mut remaining = content;

    // [v0.1.0-beta.22] 1단계: bare JSON 필터링 (mixed 패턴 포함)
    // 응답 내에 fenced가 아닌 raw JSON 객체({"tool":...})가 포함된 경우,
    // 해당 JSON 부분만 사용자 친화적 요약으로 대체한다.
    // 패턴: "설명 텍스트 + {\"tool\":...}" 또는 전체가 JSON인 경우 모두 커버.
    if !remaining.contains("```json") {
        // fenced가 전혀 없을 때만 bare JSON 스캔 수행
        let mut scan_pos = 0;
        let bytes = remaining.as_bytes();
        while scan_pos < bytes.len() {
            if bytes[scan_pos] == b'{' {
                // '{' 발견 → JSON 파싱 시도
                let candidate = &remaining[scan_pos..];
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(candidate)
                    && let Some(tool_name) = parsed.get("tool").and_then(|v| v.as_str())
                {
                    // '{' 이전 텍스트를 먼저 추가
                    result.push_str(&remaining[..scan_pos]);
                    // bare 도구 JSON → 요약으로 대체
                    result.push_str(&format!("\n⚙️  [{}] 도구 호출 감지됨", tool_name));
                    if let Some(cmd) = parsed.get("command").and_then(|v| v.as_str()) {
                        result.push_str(&format!("\n   ↳ $ {}", cmd));
                    }
                    if let Some(path) = parsed.get("path").and_then(|v| v.as_str()) {
                        result.push_str(&format!("\n   ↳ {}", path));
                    }
                    result.push('\n');
                    // JSON 원문 길이를 정확히 찾기 위해 매칭 braces 카운트
                    let actual_end = find_json_end(candidate).unwrap_or(candidate.len());
                    let after = &remaining[scan_pos + actual_end..];
                    result.push_str(after);
                    return result;
                }
            }
            scan_pos += 1;
        }
    }

    // 2단계: fenced ```json 블록 필터링 (기존 로직)
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

/// [v0.1.0-beta.22] JSON 객체의 원문 종료 위치를 brace 매칭으로 찾는다.
/// 문자열 내부의 escaped braces도 처리한다.
pub(crate) fn find_json_end(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, ch) in s.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }
        match ch {
            '\\' if in_string => {
                escape_next = true;
            }
            '"' => {
                in_string = !in_string;
            }
            '{' if !in_string => {
                depth += 1;
            }
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
    }
    None
}

fn draw_inspector(f: &mut Frame, state: &AppState, area: Rect) {
    use crate::app::state::InspectorTab;

    // [v0.1.0-beta.21] 동적 팔레트 참조
    let p = state.palette();

    // [v0.1.0-beta.18] Semantic Palette 적용: 활성 탭을 ACCENT 색상으로 강조
    let tab_names = ["Preview", "Diff", "Search", "Logs", "Recent"];
    let active_idx = match state.ui.active_inspector_tab {
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
    if let Some(tool) = &state.runtime.approval.pending_tool {
        let mut lines = Vec::new();
        lines.push(Line::from(vec![Span::styled(
            "⚠️ APPROVAL REQUIRED ⚠️",
            Style::default().fg(p.warning),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "============================",
            Style::default().fg(p.warning),
        )]));

        // [v0.1.0-beta.22] format_tool_name/detail을 사용하여 도구 정보를
        // 사용자가 읽을 수 있는 형태로 표시. 이전: format!("{:?}", tool)
        let tool_name = crate::app::App::format_tool_name(tool);
        let tool_detail = crate::app::App::format_tool_detail(tool);
        lines.push(Line::from(vec![Span::styled(
            format!("🔧 {}", tool_name),
            Style::default().fg(p.accent),
        )]));
        if !tool_detail.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                format!("   {}", tool_detail),
                Style::default().fg(p.text_primary),
            )]));
        }
        lines.push(Line::from(vec![Span::raw("")]));

        if let Some(diff) = &state.runtime.approval.diff_preview {
            lines.push(Line::from(vec![Span::styled(
                "[Diff Preview]",
                Style::default().fg(p.info),
            )]));
            for l in diff.lines() {
                if l.starts_with('+') {
                    lines.push(Line::from(vec![Span::styled(
                        l,
                        Style::default().fg(p.success),
                    )]));
                } else if l.starts_with('-') {
                    lines.push(Line::from(vec![Span::styled(
                        l,
                        Style::default().fg(p.danger),
                    )]));
                } else if l.starts_with('@') {
                    lines.push(Line::from(vec![Span::styled(
                        l,
                        Style::default().fg(p.info),
                    )]));
                } else {
                    lines.push(Line::from(vec![Span::raw(l)]));
                }
            }
            lines.push(Line::from(vec![Span::raw("")]));
        }
        lines.push(Line::from(vec![Span::styled(
            ">> Press 'y' to Approve, 'n' to Reject.",
            Style::default().fg(p.warning),
        )]));

        // [v0.1.0-beta.22] Wrap + scroll 적용 — 긴 승인 내용을 탐색할 수 있도록
        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((state.ui.timeline_scroll, 0));
        f.render_widget(paragraph, inner_area);
        return;
    }

    // [v0.1.0-beta.18] A-5: 탭별 실체 콘텐츠 렌더링
    // [v0.1.0-beta.19] inspector_tabs 위젯 연동
    use crate::tui::widgets::inspector_tabs;
    match state.ui.active_inspector_tab {
        InspectorTab::Logs => {
            inspector_tabs::render_logs(f, state, inner_area);
        }
        InspectorTab::Search => {
            inspector_tabs::render_search(f, state, inner_area);
        }
        InspectorTab::Recent => {
            inspector_tabs::render_recent(f, state, inner_area);
        }
        InspectorTab::Diff => {
            let text = "No pending diffs.\n\nDiffs will appear here after file write proposals.";
            let paragraph = Paragraph::new(text).style(Style::default().fg(p.muted));
            f.render_widget(paragraph, inner_area);
        }
        _ => {
            // Preview (기본)
            let text = "No active file context.\n\nRelevant files will appear here.";
            let paragraph = Paragraph::new(text).style(Style::default().fg(p.muted));
            f.render_widget(paragraph, inner_area);
        }
    }
}

fn draw_command_status_bar(f: &mut Frame, state: &AppState, area: Rect) {
    let p = state.palette();
    use crate::domain::session::AppMode;

    let (mode_str, mode_color) = match state.domain.session.mode {
        AppMode::Plan => (" [PLAN] ", p.info),
        AppMode::Run => (" [RUN] ", p.success),
    };

    let msg = vec![
        Span::styled(mode_str, Style::default().fg(p.bg_base).bg(mode_color)),
        Span::styled(
            " (Tab/Shift+Tab) 모드 전환 | (Ctrl+I) 인스펙터 토글 | (/help) 전체 커맨드",
            Style::default().fg(p.muted),
        ),
    ];

    let pgh = Paragraph::new(Line::from(msg)).style(Style::default().bg(p.bg_base));

    f.render_widget(pgh, area);
}

fn draw_composer(f: &mut Frame, state: &AppState, area: Rect) {
    // [v0.1.0-beta.21] 동적 팔레트 참조
    let p = state.palette();

    let block = Block::default()
        .title("Composer")
        .borders(Borders::TOP)
        .style(Style::default().fg(p.text_primary));
    let content = if state.ui.composer.input_buffer.is_empty() {
        "> (/, @, ! 사용 가능) Type your prompt here...".to_string()
    } else {
        format!("> {}", state.ui.composer.input_buffer)
    };
    // [v0.1.0-beta.22] word wrap 적용: 긴 프롬프트가 가로로 넘치지 않도록
    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);

    if state.ui.fuzzy.is_open {
        let fuzzy_area = ratatui::layout::Rect {
            x: area.x + 2,
            y: area.y.saturating_sub(6),
            width: 40,
            height: 6,
        };
        let f_block = Block::default()
            .title("File Select (@)")
            .borders(Borders::ALL)
            .style(Style::default().bg(p.bg_elevated).fg(p.text_primary));

        let mut lines = Vec::new();
        lines.push(Line::from(vec![Span::raw(format!(
            "> {}",
            state.ui.fuzzy.input
        ))]));

        if state.ui.fuzzy.matches.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "No files found",
                Style::default().fg(p.danger),
            )]));
        } else {
            for (i, m) in state.ui.fuzzy.matches.iter().enumerate().take(3) {
                if i == state.ui.fuzzy.cursor {
                    lines.push(Line::from(vec![Span::styled(
                        format!("▶ {}", m),
                        Style::default().fg(p.accent),
                    )]));
                } else {
                    lines.push(Line::from(vec![Span::raw(format!("  {}", m))]));
                }
            }
        }
        let f_para = Paragraph::new(lines).block(f_block);
        f.render_widget(ratatui::widgets::Clear, fuzzy_area);
        f.render_widget(f_para, fuzzy_area);
    }

    // [v0.1.0-beta.16] 슬래시 커맨드 자동완성 메뉴: Composer 위에 팝업으로 표시
    if state.ui.slash_menu.is_open {
        let menu_height = (state.ui.slash_menu.matches.len() as u16 + 2).min(13);
        let menu_area = ratatui::layout::Rect {
            x: area.x + 2,
            y: area.y.saturating_sub(menu_height),
            width: 35,
            height: menu_height,
        };
        let menu_block = Block::default()
            .title("Commands (/)")
            .borders(Borders::ALL)
            .style(Style::default().bg(p.bg_elevated).fg(p.text_primary));

        let mut lines = Vec::new();
        for (i, (cmd, desc)) in state.ui.slash_menu.matches.iter().enumerate() {
            let line_text = format!("{:<12} {}", cmd, desc);
            if i == state.ui.slash_menu.cursor {
                lines.push(Line::from(vec![Span::styled(
                    format!("▶ {}", line_text),
                    Style::default().fg(p.accent),
                )]));
            } else {
                lines.push(Line::from(vec![Span::raw(format!("  {}", line_text))]));
            }
        }
        let menu_para = Paragraph::new(lines).block(menu_block);
        f.render_widget(ratatui::widgets::Clear, menu_area);
        f.render_widget(menu_para, menu_area);
    }
}
