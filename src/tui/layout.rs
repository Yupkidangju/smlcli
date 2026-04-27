// [v0.1.0-beta.18] Phase 9-A: 메인 TUI 레이아웃 모듈.
// [v0.1.0-beta.21] 모든 색상 참조를 정적 pal:: 상수에서 state.palette() 동적 참조로 전환.
//   /theme 명령어로 전환된 테마가 화면에 즉시 반영됨.
//   designs.md §21.4 구현 아키텍처 참조.

use crate::app::state::AppState;
use crate::tui::palette as pal;
use crate::tui::widgets::setting_wizard;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// [v0.1.0-beta.24] Phase 14-A: 멀티라인 텍스트 렌더링 헬퍼.
/// `\n` 기준으로 분리하여 각 줄을 독립 `Line`으로 변환.
/// 기존의 `Line::from(msg.as_str())`는 개행을 무시하므로,
/// 모든 텍스트 렌더링 경로에서 이 헬퍼를 사용해야 함.
/// 라이프타임 이슈를 방지하기 위해 각 줄을 String으로 복사한다.
fn render_multiline_text(text: &str, style: Style) -> Vec<Line<'static>> {
    text.lines()
        .map(|line| Line::from(vec![Span::styled(line.to_string(), style)]))
        .collect()
}

/// [v0.1.0-beta.24] Phase 14-D: 긴 문자열의 중간을 생략하는 헬퍼.
/// 예: "/home/user/very/long/path" → "/home/u…long/path" (max_len=20)
fn truncate_middle(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    let half = max_len.saturating_sub(1) / 2;
    let start = &s[..half];
    let end = &s[s.len().saturating_sub(half)..];
    format!("{}…{}", start, end)
}

pub fn draw(f: &mut Frame, state: &AppState) {
    let size = f.area();

    // [v1.5.0] 터미널 최소 크기 경고 및 리사이즈 붕괴 방지
    if size.width < 80 || size.height < 24 {
        let p = state.palette();
        let warning = Paragraph::new(vec![
            Line::from(Span::styled(
                "⚠️ 터미널 크기가 너무 작습니다.",
                Style::default().fg(p.danger),
            )),
            Line::from(Span::styled(
                format!("현재: {}x{} / 권장: 80x24", size.width, size.height),
                Style::default().fg(p.text_secondary),
            )),
            Line::from("터미널 창을 늘려주세요."),
        ])
        .block(
            crate::tui::widgets::block_with_borders(
                Borders::ALL,
                state
                    .domain
                    .settings
                    .as_ref()
                    .map(|s| s.use_ascii_borders)
                    .unwrap_or(false),
            )
            .border_style(Style::default().fg(p.danger)),
        )
        .alignment(ratatui::layout::Alignment::Center);

        let warning_area = Rect {
            x: size.width.saturating_sub(40) / 2,
            y: size.height.saturating_sub(5) / 2,
            width: 40.min(size.width),
            height: 5.min(size.height),
        };

        f.render_widget(ratatui::widgets::Clear, size);
        f.render_widget(warning, warning_area);
        return;
    }

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

    // [v0.1.0-beta.24] Phase 14-D: 반응형 인스펙터 폭.
    // 인스펙터: 32~48칼럼 범위 클램프. 타임라인 최소 72칼럼 보장.
    if state.ui.show_inspector || state.runtime.approval.pending_tool.is_some() {
        let total_width = chunks[1].width;
        if total_width < 100 {
            // [v0.1.0-beta.26] 100칼럼 미만일 경우 오버레이/드로어(Drawer) 모드 적용
            draw_timeline(f, state, chunks[1]);
            let drawer_width = 32;
            let drawer_area = Rect {
                x: chunks[1]
                    .x
                    .saturating_add(total_width.saturating_sub(drawer_width)),
                y: chunks[1].y,
                width: drawer_width.min(total_width),
                height: chunks[1].height,
            };
            f.render_widget(ratatui::widgets::Clear, drawer_area);
            draw_inspector(f, state, drawer_area);
        } else {
            let inspector_width = (total_width as f32 * 0.30).clamp(32.0, 48.0) as u16;
            let timeline_width = total_width.saturating_sub(inspector_width).max(72);
            let actual_inspector = total_width.saturating_sub(timeline_width);

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Length(timeline_width),
                        Constraint::Length(actual_inspector),
                    ]
                    .as_ref(),
                )
                .split(chunks[1]);

            draw_timeline(f, state, main_chunks[0]);
            draw_inspector(f, state, main_chunks[1]);
        }
    } else {
        draw_timeline(f, state, chunks[1]);
    }

    draw_composer_toolbar(f, state, chunks[2]);
    draw_composer(f, state, chunks[3]);

    if state.ui.config.is_open {
        crate::tui::widgets::config_dashboard::draw_config(f, state);
    }
    if let crate::app::state::TrustGatePopup::Open { .. } = state.ui.trust_gate.popup {
        draw_trust_gate(f, state);
    }
    if state.ui.palette.is_open {
        draw_command_palette(f, state);
    }
    // [v2.3.0] Phase 31: 토스트 알림 렌더링
    if let Some(toast) = &state.ui.toast {
        let msg_len = toast.message.chars().count() as u16;
        let toast_area = Rect {
            x: size.width.saturating_sub(msg_len + 4),
            y: size.height.saturating_sub(5), // 컴포저 바로 위
            width: msg_len + 4,
            height: 3,
        };
        let p = state.palette();
        let color = if toast.is_error { p.danger } else { p.success };
        let block = crate::tui::widgets::block_with_borders(
            Borders::ALL,
            state
                .domain
                .settings
                .as_ref()
                .map(|s| s.use_ascii_borders)
                .unwrap_or(false),
        )
        .border_style(Style::default().fg(color))
        .style(Style::default().bg(p.bg_elevated));
        let p = Paragraph::new(toast.message.as_str())
            .block(block)
            .style(Style::default().fg(color))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(ratatui::widgets::Clear, toast_area);
        f.render_widget(p, toast_area);
    }

    // [v2.4.0] Phase 32: 헬프 오버레이
    crate::tui::help_overlay::render(
        f,
        size,
        &state.ui,
        state
            .domain
            .settings
            .as_ref()
            .map(|s| s.use_ascii_borders)
            .unwrap_or(false),
        state.palette(),
    );

    // [v3.7.0] Phase 47: Questionnaire 오버레이 (AskClarification TUI 모달)
    if let Some(ref qs) = state.ui.questionnaire {
        let use_ascii = state
            .domain
            .settings
            .as_ref()
            .map(|s| s.use_ascii_borders)
            .unwrap_or(false);
        let widget = crate::tui::widgets::questionnaire::QuestionnaireWidget::new(
            qs,
            use_ascii,
            state.palette().clone(),
        );
        f.render_widget(widget, size);
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
    let cwd_raw = std::env::current_dir()
        .map(|pp| pp.display().to_string())
        .unwrap_or_else(|_| "?".to_string());
    // [v0.1.0-beta.24] Phase 14-D: cwd가 너무 길면 중간을 생략
    let cwd = truncate_middle(&cwd_raw, 30);
    let shell_policy_str = state
        .domain
        .settings
        .as_ref()
        .map(|s| format!("{:?}", s.shell_policy))
        .unwrap_or_else(|| "None".to_string());
    let trust_str = state
        .domain
        .settings
        .as_ref()
        .map(|s| match s.get_workspace_trust(&cwd_raw) {
            crate::domain::settings::WorkspaceTrustState::Trusted => "Trust: ✓",
            crate::domain::settings::WorkspaceTrustState::Restricted => "Trust: ✕",
            crate::domain::settings::WorkspaceTrustState::Unknown => "Trust: ?",
        })
        .unwrap_or("Trust: ?");

    // [v0.1.0-beta.18] context budget 색상을 사용량에 따라 변경
    let ctx_color = if budget >= 85 {
        p.danger
    } else if budget >= 70 {
        p.warning
    } else {
        p.text_secondary
    };

    // [v0.1.0-beta.24] Phase 14 감사 보정 2차: 상단 바 좌우 구조적 분리
    // 긴 경로/모델명 등으로 인해 핵심 UI가 밀려나지 않도록 Layout을 좌/우로 분할.
    let bar_width = area.width;

    let provider_display = truncate_middle(&provider, 15);
    let model_display = truncate_middle(&model, 25);
    let ctx_span_text = format!("{}% ctx", budget);

    // 우측 영역 (우선순위 높음: mode, ctx, 상태확인)
    let mut right_spans = vec![
        Span::styled(mode_str.to_string(), Style::default().fg(p.accent)),
        Span::styled(" · ".to_string(), Style::default().fg(p.text_primary)),
        Span::styled(ctx_span_text, Style::default().fg(ctx_color)),
    ];
    if bar_width > 90 {
        let host_shell = &state.runtime.workspace.host_shell;
        let exec_shell = &state.runtime.workspace.exec_shell;
        right_spans.push(Span::styled(
            format!(
                " · Host: {} | Exec: {} | {} · {}",
                host_shell, exec_shell, shell_policy_str, trust_str
            ),
            Style::default().fg(p.text_secondary),
        ));
    }
    right_spans.push(Span::styled(
        " · ✓ ".to_string(),
        Style::default().fg(p.text_primary),
    ));
    let right_width: u16 = right_spans
        .iter()
        .map(|s| s.content.chars().count() as u16)
        .sum();
    let right_line = Line::from(right_spans).alignment(ratatui::layout::Alignment::Right);

    // 좌측 영역 (상대적 여유 시 cwd 추가)
    let mut left_spans = vec![Span::styled(
        format!(" smlcli · {}/{}", provider_display, model_display),
        Style::default().fg(p.text_primary),
    )];
    if bar_width > right_width + 45 {
        left_spans.push(Span::styled(
            format!(" · {}", cwd),
            Style::default().fg(p.text_secondary),
        ));
    }
    let left_line = Line::from(left_spans);

    // 배경색을 채우기 위해 전체 영역에 Block 렌더링
    f.render_widget(
        Block::default().style(Style::default().bg(p.bg_panel)),
        area,
    );

    // 좌/우 분할
    let chunks = ratatui::layout::Layout::horizontal([
        ratatui::layout::Constraint::Min(0),
        ratatui::layout::Constraint::Length(right_width),
    ])
    .split(area);

    f.render_widget(
        Paragraph::new(left_line).style(Style::default().bg(p.bg_panel)),
        chunks[0],
    );
    f.render_widget(
        Paragraph::new(right_line).style(Style::default().bg(p.bg_panel)),
        chunks[1],
    );
}

/// [v0.1.0-beta.25] 타임라인 블록의 깊이에 맞는 들여쓰기 접두사.
/// depth=0이면 평면, depth>=1이면 첫 줄에 `└─`, 후속 줄에 공백 인덴트를 준다.
fn timeline_prefix(depth: u8, first_line: bool) -> String {
    if depth == 0 {
        return String::new();
    }

    let base_indent = "  ".repeat(depth.saturating_sub(1) as usize);
    if first_line {
        format!("{}└─ ", base_indent)
    } else {
        format!("{}   ", base_indent)
    }
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
        // === 타임라인 블록 기반 렌더링 ===
        for (idx, block) in state.ui.timeline.iter().enumerate() {
            let is_selected = matches!(
                state.ui.focused_pane,
                crate::app::state::FocusedPane::Timeline
            ) && state.ui.timeline_cursor == idx;
            let start_len = lines.len();
            match block.kind {
                crate::app::state::TimelineBlockKind::Conversation => {
                    let is_user = block.role == Some(crate::providers::types::Role::User);
                    let label = if is_user { "User:" } else { "AI:" };
                    let label_color = if is_user { p.accent } else { p.success };
                    lines.push(Line::from(vec![Span::styled(
                        label,
                        Style::default().fg(label_color),
                    )]));
                    for section in &block.body {
                        if let crate::app::state::BlockSection::Markdown(msg) = section {
                            if !is_user {
                                let display = filter_tool_json(msg);
                                lines.extend(render_multiline_text(
                                    &display,
                                    Style::default().fg(p.text_primary),
                                ));
                            } else {
                                lines.extend(render_multiline_text(
                                    msg,
                                    Style::default().fg(p.text_primary),
                                ));
                            }
                        }
                    }
                    lines.push(Line::from(""));
                }
                crate::app::state::TimelineBlockKind::Notice => {
                    for section in &block.body {
                        if let crate::app::state::BlockSection::Markdown(msg) = section {
                            for (i, line) in msg.lines().enumerate() {
                                let prefix = if i == 0 {
                                    format!("{}ℹ  ", timeline_prefix(block.depth, true))
                                } else {
                                    format!("{}   ", timeline_prefix(block.depth, false))
                                };
                                lines.push(Line::from(vec![Span::styled(
                                    format!("{}{}", prefix, line),
                                    Style::default().fg(
                                        if block.status == crate::app::state::BlockStatus::Error {
                                            p.danger
                                        } else {
                                            p.info
                                        },
                                    ),
                                )]));
                            }
                        }
                    }
                    lines.push(Line::from(""));
                }
                crate::app::state::TimelineBlockKind::ToolRun => {
                    let (badge, badge_color) = match block.status {
                        crate::app::state::BlockStatus::Running => {
                            let frames = state.ui.motion.spinner_frames;
                            let idx = (state.ui.tick_count as usize) % frames.len();
                            (frames[idx], p.warning)
                        }
                        crate::app::state::BlockStatus::Done => ("✅", p.success),
                        crate::app::state::BlockStatus::Error => ("❌", p.danger),
                        crate::app::state::BlockStatus::NeedsApproval => {
                            let pulse = state.ui.motion.pulse_period_ticks as u64;
                            if (state.ui.tick_count % pulse) < (pulse / 2) {
                                ("⏸", p.warning)
                            } else {
                                (" ", p.warning)
                            }
                        }
                        _ => ("◻", p.muted),
                    };
                    lines.push(Line::from(vec![Span::styled(
                        format!(
                            "{}{} {} ",
                            timeline_prefix(block.depth, true),
                            badge,
                            block.title
                        ),
                        Style::default().fg(badge_color),
                    )]));
                    for section in &block.body {
                        match section {
                            crate::app::state::BlockSection::ToolSummary { summary, .. } => {
                                if block.display_mode
                                    == crate::app::state::BlockDisplayMode::Collapsed
                                {
                                    if let Some((add, del)) = block.diff_summary {
                                        let text = format!(
                                            "[ +{} lines / -{} lines ] (Enter 키로 펼치기)",
                                            add, del
                                        );
                                        let style = if is_selected {
                                            Style::default().fg(p.bg_base).bg(p.accent)
                                        } else {
                                            Style::default().fg(p.muted)
                                        };
                                        lines.push(Line::from(vec![Span::styled(
                                            format!(
                                                "{}{}",
                                                timeline_prefix(block.depth, false),
                                                text
                                            ),
                                            style,
                                        )]));
                                    }
                                } else {
                                    for sl in summary.lines() {
                                        let style = if sl.starts_with('+') && !sl.starts_with("+++")
                                        {
                                            Style::default().fg(ratatui::style::Color::Green)
                                        } else if sl.starts_with('-') && !sl.starts_with("---") {
                                            Style::default().fg(ratatui::style::Color::Red)
                                        } else {
                                            Style::default().fg(p.text_secondary)
                                        };
                                        lines.push(Line::from(vec![Span::styled(
                                            format!(
                                                "{}{}",
                                                timeline_prefix(block.depth, false),
                                                sl
                                            ),
                                            style,
                                        )]));
                                    }
                                }
                            }
                            crate::app::state::BlockSection::Markdown(msg) => {
                                for sl in msg.lines() {
                                    lines.push(Line::from(vec![Span::styled(
                                        format!("{}{}", timeline_prefix(block.depth, false), sl),
                                        Style::default().fg(p.text_secondary),
                                    )]));
                                }
                            }
                            _ => {}
                        }
                    }
                    lines.push(Line::from(""));
                }
                crate::app::state::TimelineBlockKind::Approval => {
                    let pulse_color = if (state.ui.tick_count % 6) < 3 {
                        p.warning
                    } else {
                        p.text_primary
                    };
                    lines.push(Line::from(vec![Span::styled(
                        format!(
                            "{}⚠  승인 대기: {} ",
                            timeline_prefix(block.depth, true),
                            block.title
                        ),
                        Style::default().fg(pulse_color),
                    )]));
                    for section in &block.body {
                        if let crate::app::state::BlockSection::Markdown(msg) = section {
                            lines.push(Line::from(vec![Span::styled(
                                format!("{}{}", timeline_prefix(block.depth, false), msg),
                                Style::default().fg(p.text_secondary),
                            )]));
                        }
                    }
                    lines.push(Line::from(""));
                }
                crate::app::state::TimelineBlockKind::Help => {
                    lines.push(Line::from(vec![Span::styled(
                        "ℹ  Available Commands:",
                        Style::default().fg(p.info),
                    )]));
                    let inner_width = area.width.saturating_sub(1);
                    let cmd_col_width = 14;
                    let max_desc_width =
                        (inner_width as usize).saturating_sub(cmd_col_width).max(10);
                    for section in &block.body {
                        if let crate::app::state::BlockSection::KeyValueTable(entries) = section {
                            for (cmd, desc) in entries {
                                let mut current_line = String::new();
                                let mut desc_lines = Vec::new();
                                for word in desc.split_whitespace() {
                                    let word_width = word.chars().count();
                                    let current_width = current_line.chars().count();
                                    if current_width > 0
                                        && current_width + 1 + word_width > max_desc_width
                                    {
                                        desc_lines.push(current_line.clone());
                                        current_line.clear();
                                    }
                                    if !current_line.is_empty() {
                                        current_line.push(' ');
                                    }
                                    current_line.push_str(word);
                                }
                                if !current_line.is_empty() {
                                    desc_lines.push(current_line);
                                }
                                if desc_lines.is_empty() {
                                    desc_lines.push(String::new());
                                }
                                for (i, dline) in desc_lines.iter().enumerate() {
                                    let cmd_str = if i == 0 {
                                        format!("   {:<11}", cmd)
                                    } else {
                                        " ".repeat(cmd_col_width)
                                    };
                                    lines.push(Line::from(vec![
                                        Span::styled(cmd_str, Style::default().fg(p.accent)),
                                        Span::styled(
                                            dline.to_string(),
                                            Style::default().fg(p.text_secondary),
                                        ),
                                    ]));
                                }
                            }
                        }
                    }
                    lines.push(Line::from(""));
                }
                crate::app::state::TimelineBlockKind::GitCommit => {
                    lines.push(Line::from(vec![Span::styled(
                        format!(
                            "{}🌿 Git Commit: {}",
                            timeline_prefix(block.depth, true),
                            block.title
                        ),
                        Style::default().fg(p.success),
                    )]));
                    for section in &block.body {
                        if let crate::app::state::BlockSection::Markdown(msg) = section {
                            lines.push(Line::from(vec![Span::styled(
                                format!("{}{}", timeline_prefix(block.depth, false), msg),
                                Style::default().fg(p.text_secondary),
                            )]));
                        }
                    }
                    lines.push(Line::from(""));
                }
            }
            let is_selected = state.ui.focused_pane == crate::app::state::FocusedPane::Timeline
                && state.ui.timeline_cursor == idx;
            if is_selected && let Some(first_line) = lines.get_mut(start_len) {
                *first_line = first_line
                    .clone()
                    .patch_style(Style::default().bg(p.bg_elevated));
            }
        }
    } else {
        // === 폴백: 기존 session.messages 기반 ===
        for msg in &state.domain.session.messages {
            if msg.role == crate::providers::types::Role::System && msg.pinned {
                continue;
            }
            let (role_str, role_color) = match msg.role {
                crate::providers::types::Role::User => ("User:", p.accent),
                crate::providers::types::Role::Assistant => ("AI:", p.success),
                crate::providers::types::Role::System => ("System:", p.info),
                crate::providers::types::Role::Tool => ("Tool:", p.muted),
            };
            lines.push(Line::from(vec![Span::styled(
                role_str,
                Style::default().fg(role_color),
            )]));
            let content_str = msg.content.as_deref().unwrap_or_default();
            let display_content = filter_tool_json(content_str);
            // [v0.1.0-beta.24] Phase 14-A: 폴백 경로에도 멀티라인 렌더링 적용
            lines.extend(render_multiline_text(
                &display_content,
                Style::default().fg(p.text_primary),
            ));
            lines.push(Line::from(""));
        }
    }

    // [v0.1.0-beta.18] A-4: tick 기반 thinking 스피너
    if state.runtime.is_thinking {
        let spinner = pal::SPINNER_FRAMES[(state.ui.tick_count as usize) % 8];
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

    let mut block = crate::tui::widgets::block_with_borders(
        Borders::RIGHT,
        state
            .domain
            .settings
            .as_ref()
            .map(|s| s.use_ascii_borders)
            .unwrap_or(false),
    )
    .title("Timeline");
    if state.ui.focused_pane == crate::app::state::FocusedPane::Timeline {
        block = block.border_style(Style::default().fg(p.accent));
    }
    // [v0.1.0-beta.24] Phase 14-B 감사 보정: follow_tail을 실제 렌더링에 연결.
    // timeline_scroll은 "바닥으로부터의 오프셋" (0 = 최하단/최신).
    // follow_tail=true이면 강제로 0 (맨 아래), false이면 사용자가 설정한 오프셋 사용.
    // ratatui의 scroll()은 top-based이므로 변환이 필요.
    let visible_height = area.height.saturating_sub(2) as usize; // border 제외
    let total_lines = lines.len();

    let bottom_up_offset = if state.ui.timeline_follow_tail {
        0usize
    } else {
        state.ui.timeline_scroll as usize
    };

    // bottom-up offset → top-based offset 변환
    // total_lines가 visible_height보다 작으면 스크롤 불필요
    let top_offset = if total_lines > visible_height {
        (total_lines - visible_height).saturating_sub(bottom_up_offset)
    } else {
        0
    };

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((top_offset as u16, 0));
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

    // [v0.1.0-beta.24] Phase 14-D: 인스펙터 폭이 좁으면 탭 라벨 축약
    let use_short = area.width < 40;
    let tab_names_full = ["Preview", "Diff", "Search", "Logs", "Recent", "Git"];
    let tab_names_short = ["Prev", "Diff", "Srch", "Logs", "Rcnt", "Git"];
    let tab_names = if use_short {
        &tab_names_short
    } else {
        &tab_names_full
    };
    let active_idx = match state.ui.active_inspector_tab {
        InspectorTab::Preview => 0,
        InspectorTab::Diff => 1,
        InspectorTab::Search => 2,
        InspectorTab::Logs => 3,
        InspectorTab::Recent => 4,
        InspectorTab::Git => 5,
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
        .join(if use_short { "·" } else { " · " });
    let mut block = crate::tui::widgets::block_with_borders(
        Borders::LEFT,
        state
            .domain
            .settings
            .as_ref()
            .map(|s| s.use_ascii_borders)
            .unwrap_or(false),
    )
    .title("Inspector");
    if state.ui.focused_pane == crate::app::state::FocusedPane::Inspector {
        block = block.border_style(Style::default().fg(p.accent));
    }
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    let header_two_rows = inner_area.width < 28;
    let header_height = if header_two_rows { 2 } else { 1 };
    let inspector_chunks =
        ratatui::layout::Layout::vertical([Constraint::Length(header_height), Constraint::Min(0)])
            .split(inner_area);

    let header_lines = if header_two_rows {
        let header_tabs = if use_short {
            ["Prev", "Diff", "Srch", "Logs", "Rcnt", "Git"]
        } else {
            ["Preview", "Diff", "Search", "Logs", "Recent", "Git"]
        };
        vec![
            Line::from(
                header_tabs[..3]
                    .iter()
                    .enumerate()
                    .map(|(i, name)| {
                        let idx = i;
                        if idx == active_idx {
                            Span::styled(format!("[*{}*] ", name), Style::default().fg(p.accent))
                        } else {
                            Span::styled(
                                format!("[{}] ", name),
                                Style::default().fg(p.text_secondary),
                            )
                        }
                    })
                    .collect::<Vec<_>>(),
            ),
            Line::from(
                header_tabs[3..]
                    .iter()
                    .enumerate()
                    .map(|(i, name)| {
                        let idx = i + 3;
                        if idx == active_idx {
                            Span::styled(format!("[*{}*] ", name), Style::default().fg(p.accent))
                        } else {
                            Span::styled(
                                format!("[{}] ", name),
                                Style::default().fg(p.text_secondary),
                            )
                        }
                    })
                    .collect::<Vec<_>>(),
            ),
        ]
    } else {
        vec![Line::from(vec![
            Span::styled("Tabs ", Style::default().fg(p.muted)),
            Span::styled(tabs_title, Style::default().fg(p.text_secondary)),
        ])]
    };
    let header = Paragraph::new(header_lines).wrap(Wrap { trim: false });
    f.render_widget(header, inspector_chunks[0]);

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

        // [v0.1.0-beta.24] Phase 14-B 감사 보정: inspector_scroll 사용으로 독립 스크롤
        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((state.ui.inspector_scroll.get(), 0));
        f.render_widget(paragraph, inspector_chunks[1]);
        return;
    }

    // [v0.1.0-beta.18] A-5: 탭별 실체 콘텐츠 렌더링
    // [v0.1.0-beta.19] inspector_tabs 위젯 연동
    use crate::tui::widgets::inspector_tabs;
    match state.ui.active_inspector_tab {
        InspectorTab::Logs => {
            inspector_tabs::render_logs(f, state, inspector_chunks[1]);
        }
        InspectorTab::Search => {
            inspector_tabs::render_search(f, state, inspector_chunks[1]);
        }
        InspectorTab::Recent => {
            inspector_tabs::render_recent(f, state, inspector_chunks[1]);
        }
        InspectorTab::Diff => {
            inspector_tabs::render_diff(f, state, inspector_chunks[1]);
        }
        InspectorTab::Git => {
            inspector_tabs::render_git(f, state, inspector_chunks[1]);
        }
        _ => {
            // Preview (기본)
            inspector_tabs::render_preview(f, state, inspector_chunks[1]);
        }
    }
}

fn draw_composer_toolbar(f: &mut Frame, state: &AppState, area: Rect) {
    let p = state.palette();
    let mut msg = Vec::new();

    for chip in &state.ui.toolbar.chips {
        let (color, bg) = match chip.kind {
            crate::app::state::InputChipKind::Mode => {
                if chip.label == "RUN" {
                    (p.bg_base, Some(p.success))
                } else {
                    (p.bg_base, Some(p.info))
                }
            }
            crate::app::state::InputChipKind::Path => (p.text_secondary, None),
            crate::app::state::InputChipKind::Policy => (p.text_secondary, None),
            crate::app::state::InputChipKind::Hint => (p.muted, None),
            crate::app::state::InputChipKind::Context => (p.accent, None),
        };

        let mut style = Style::default().fg(color);
        if let Some(b) = bg {
            style = style.bg(b);
        }
        if chip.kind == crate::app::state::InputChipKind::Hint {
            style = style.add_modifier(ratatui::style::Modifier::ITALIC);
        }

        let display_text = match chip.kind {
            crate::app::state::InputChipKind::Mode => format!(" [{}] ", chip.label),
            crate::app::state::InputChipKind::Path | crate::app::state::InputChipKind::Context => {
                format!("[{}]", truncate_middle(&chip.label, 24))
            }
            crate::app::state::InputChipKind::Hint => format!("[{}]", chip.label),
            _ => format!(" [{}] ", chip.label),
        };

        msg.push(Span::styled(display_text, style));
        msg.push(Span::raw(" "));
    }

    if state.ui.toolbar.multiline {
        msg.push(Span::styled("[Multiline] ", Style::default().fg(p.accent)));
    }

    let pgh = Paragraph::new(Line::from(msg)).style(Style::default().bg(p.bg_base));
    f.render_widget(pgh, area);
}

fn draw_composer(f: &mut Frame, state: &AppState, area: Rect) {
    // [v0.1.0-beta.21] 동적 팔레트 참조
    let p = state.palette();

    let mut block = crate::tui::widgets::block_with_borders(
        Borders::TOP,
        state
            .domain
            .settings
            .as_ref()
            .map(|s| s.use_ascii_borders)
            .unwrap_or(false),
    )
    .title("Composer")
    .border_style(Style::default().fg(p.text_primary));
    if state.ui.focused_pane == crate::app::state::FocusedPane::Composer {
        block = block.border_style(Style::default().fg(p.accent));
    }
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
        let f_block = crate::tui::widgets::block_with_borders(
            Borders::ALL,
            state
                .domain
                .settings
                .as_ref()
                .map(|s| s.use_ascii_borders)
                .unwrap_or(false),
        )
        .title("File Select (@)")
        .border_style(Style::default().fg(p.text_primary));

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
        let menu_block = crate::tui::widgets::block_with_borders(
            Borders::ALL,
            state
                .domain
                .settings
                .as_ref()
                .map(|s| s.use_ascii_borders)
                .unwrap_or(false),
        )
        .title("Commands (/)")
        .border_style(Style::default().fg(p.text_primary));

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

fn draw_command_palette(f: &mut Frame, state: &AppState) {
    let p = state.palette();
    let width = 60.min(f.area().width.saturating_sub(4));
    let height = 15.min(f.area().height.saturating_sub(4));
    let area = ratatui::layout::Rect {
        x: (f.area().width.saturating_sub(width)) / 2,
        y: (f.area().height.saturating_sub(height)) / 2,
        width,
        height,
    };

    let block = crate::tui::widgets::block_with_borders(
        Borders::ALL,
        state
            .domain
            .settings
            .as_ref()
            .map(|s| s.use_ascii_borders)
            .unwrap_or(false),
    )
    .title(" Command Palette (Ctrl+K) ")
    .border_style(Style::default().fg(p.accent));

    let mut lines = vec![
        Line::from(vec![Span::raw(format!("> {}_", state.ui.palette.query))]),
        Line::from(""),
    ];

    if state.ui.palette.results.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "No commands found",
            Style::default().fg(p.danger),
        )]));
    } else {
        for (idx, cmd) in state.ui.palette.results.iter().enumerate().take(8) {
            let prefix = if idx == state.ui.palette.cursor {
                "▶ "
            } else {
                "  "
            };
            let style = if idx == state.ui.palette.cursor {
                Style::default().fg(p.accent)
            } else {
                Style::default().fg(p.text_primary)
            };

            let shortcut_str = cmd.shortcut_hint.unwrap_or("");
            lines.push(Line::from(vec![
                Span::styled(format!("{}{:<18} ", prefix, cmd.title), style),
                Span::styled(
                    format!("{:<10} ", cmd.category),
                    Style::default().fg(p.muted),
                ),
                Span::styled(
                    shortcut_str.to_string(),
                    Style::default().fg(p.text_secondary),
                ),
            ]));
        }
    }

    let pgh = Paragraph::new(lines).block(block);
    f.render_widget(ratatui::widgets::Clear, area);
    f.render_widget(pgh, area);
}

fn draw_trust_gate(f: &mut Frame, state: &AppState) {
    use ratatui::widgets::Clear;

    let p = state.palette();
    let root = match &state.ui.trust_gate.popup {
        crate::app::state::TrustGatePopup::Open { root } => root,
        _ => return,
    };

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Length(12),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(f.area());

    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1];

    f.render_widget(Clear, popup_area);

    let mut lines = vec![
        Line::from(vec![Span::styled(
            "Workspace Trust Gate",
            Style::default()
                .fg(p.warning)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "[!] Do you trust the authors of the files in this folder?",
            Style::default().fg(p.text_primary),
        )]),
        Line::from(vec![Span::styled(
            "This provides AI agent access to your files and execute commands.",
            Style::default().fg(p.text_secondary),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Path: ", Style::default().fg(p.text_primary)),
            Span::styled(root.clone(), Style::default().fg(p.accent)),
        ]),
        Line::from(""),
    ];

    let options = [
        "Trust Workspace & Remember",
        "Trust this time only",
        "No, I don't trust it (Restricted Mode)",
    ];

    for (i, opt) in options.iter().enumerate() {
        if i == state.ui.trust_gate.cursor_index {
            lines.push(Line::from(vec![Span::styled(
                format!(" > {}", opt),
                Style::default()
                    .fg(p.bg_elevated)
                    .bg(p.accent)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )]));
        } else {
            lines.push(Line::from(vec![Span::styled(
                format!("   {}", opt),
                Style::default().fg(p.text_secondary),
            )]));
        }
    }

    let block = crate::tui::widgets::block_with_borders(
        Borders::ALL,
        state
            .domain
            .settings
            .as_ref()
            .map(|s| s.use_ascii_borders)
            .unwrap_or(false),
    )
    .border_style(Style::default().fg(p.warning))
    .title(" Security Verification ");

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(ratatui::layout::Alignment::Left);

    f.render_widget(paragraph, popup_area);
}
