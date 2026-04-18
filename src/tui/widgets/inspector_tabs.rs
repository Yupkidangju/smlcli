// [v0.1.0-beta.19] Inspector 탭 전용 위젯 모듈.
// spec.md §3.8에 명시된 Search, Logs, Recent 탭의 렌더링 로직을 구현.
// [v0.1.0-beta.21] 모든 색상 참조를 state.palette() 동적 참조로 전환.

use crate::app::state::AppState;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn render_preview(f: &mut Frame, state: &AppState, area: Rect) {
    let p = state.palette();
    
    if state.ui.timeline.is_empty() {
        let text = "No active blocks.\n\nTimeline is empty.";
        let paragraph = Paragraph::new(text).style(Style::default().fg(p.muted));
        f.render_widget(paragraph, area);
        return;
    }

    let cursor = state.ui.timeline_cursor.min(state.ui.timeline.len().saturating_sub(1));
    let block = &state.ui.timeline[cursor];
    
    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(format!("Block: {}", block.title), Style::default().fg(p.accent).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));
    
    for section in &block.body {
        match section {
            crate::app::state::BlockSection::Markdown(msg) => {
                for line in msg.lines() {
                    lines.push(Line::from(Span::styled(line, Style::default().fg(p.text_primary))));
                }
            }
            crate::app::state::BlockSection::ToolSummary { tool_name, summary } => {
                lines.push(Line::from(Span::styled(format!("Tool: {}", tool_name), Style::default().fg(p.info))));
                for line in summary.lines() {
                    lines.push(Line::from(Span::styled(line, Style::default().fg(p.text_secondary))));
                }
            }
            crate::app::state::BlockSection::KeyValueTable(entries) => {
                for (k, v) in entries {
                    lines.push(Line::from(vec![
                        Span::styled(format!("{}: ", k), Style::default().fg(p.accent)),
                        Span::styled(v.clone(), Style::default().fg(p.text_primary)),
                    ]));
                }
            }
            crate::app::state::BlockSection::CodeFence { language, content } => {
                let lang_str = language.as_deref().unwrap_or("text");
                lines.push(Line::from(Span::styled(format!("```{}", lang_str), Style::default().fg(p.muted))));
                for line in content.lines() {
                    lines.push(Line::from(Span::styled(line, Style::default().fg(p.text_primary))));
                }
                lines.push(Line::from(Span::styled("```", Style::default().fg(p.muted))));
            }
        }
        lines.push(Line::from(""));
    }

    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false })
        .scroll((state.ui.inspector_scroll, 0));

    f.render_widget(para, area);
}

/// Diff 탭 렌더링: 승인 대기 중인 변경사항 미리보기.
pub fn render_diff(f: &mut Frame, state: &AppState, area: Rect) {
    let p = state.palette();
    
    if let Some(diff) = &state.runtime.approval.diff_preview {
        let lines: Vec<Line> = diff.lines().map(|line| {
            let color = if line.starts_with('+') {
                p.success
            } else if line.starts_with('-') {
                p.danger
            } else {
                p.text_primary
            };
            Line::from(Span::styled(line, Style::default().fg(color)))
        }).collect();

        let para = Paragraph::new(lines)
            .block(Block::default().borders(Borders::NONE))
            .wrap(Wrap { trim: false })
            .scroll((state.ui.inspector_scroll, 0));
        f.render_widget(para, area);
    } else {
        let text = "No pending diffs.\n\nDiffs will appear here after file write proposals.";
        let paragraph = Paragraph::new(text).style(Style::default().fg(p.muted));
        f.render_widget(paragraph, area);
    }
}

/// Logs 탭 렌더링: runtime.logs_buffer의 내용을 스트리밍 스타일로 표시.
pub fn render_logs(f: &mut Frame, state: &AppState, area: Rect) {
    // [v0.1.0-beta.21] 동적 팔레트 참조
    let p = state.palette();
    let mut log_lines = Vec::new();

    if state.runtime.logs_buffer.is_empty() {
        log_lines.push(Line::from(Span::styled(
            " (No logs recorded in this session) ",
            Style::default().fg(p.muted).add_modifier(Modifier::ITALIC),
        )));
    } else {
        // 성능을 위해 최근 100줄만 표시
        let start = state.runtime.logs_buffer.len().saturating_sub(100);
        for (i, log) in state.runtime.logs_buffer[start..].iter().enumerate() {
            let color = if i % 2 == 0 {
                p.text_secondary
            } else {
                p.text_primary
            };
            log_lines.push(Line::from(Span::styled(
                log.as_str(),
                Style::default().fg(color),
            )));
        }
    }

    // [v0.1.0-beta.24] Phase 14-B 감사 보정: inspector_scroll 사용으로 독립 스크롤
    let para = Paragraph::new(log_lines)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false })
        .scroll((state.ui.inspector_scroll, 0));

    f.render_widget(para, area);
}

/// [v0.1.0-beta.20] Search 탭 렌더링: 타임라인 내의 텍스트 검색 결과 표시.
/// Composer 입력 버퍼를 검색어로 사용하여 타임라인 전체를 대소문자 무시로 필터링.
/// 검색어가 비어있으면 전체 텍스트 콘텐츠를 인덱스 형태로 표시한다.
pub fn render_search(f: &mut Frame, state: &AppState, area: Rect) {


    // [v0.1.0-beta.21] 동적 팔레트 참조
    let p = state.palette();

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        " [ Timeline Search ] ",
        Style::default().fg(p.accent),
    )));
    lines.push(Line::from(""));

    // 검색어: Composer 입력 버퍼를 참조
    let query = state.ui.composer.input_buffer.trim().to_lowercase();
    let query_display = if query.is_empty() {
        "(전체 타임라인 인덱스)".to_string()
    } else {
        format!("검색어: \"{}\"", state.ui.composer.input_buffer.trim())
    };
    lines.push(Line::from(Span::styled(
        format!(" {} ", query_display),
        Style::default().fg(p.info),
    )));
    lines.push(Line::from(""));

    // 타임라인 엔트리에서 텍스트를 추출하고 검색 필터링
    let mut match_count = 0usize;
    for (idx, block) in state.ui.timeline.iter().enumerate() {
        let label = match block.kind {
            crate::app::state::TimelineBlockKind::Conversation => "Conv",
            crate::app::state::TimelineBlockKind::ToolRun => "Tool",
            crate::app::state::TimelineBlockKind::Approval => "Appr",
            crate::app::state::TimelineBlockKind::Notice => "Sys",
            crate::app::state::TimelineBlockKind::Help => "Help",
        };

        let mut text = block.title.clone();
        for section in &block.body {
            match section {
                crate::app::state::BlockSection::Markdown(s) => {
                    text.push(' ');
                    text.push_str(s);
                }
                crate::app::state::BlockSection::ToolSummary { summary, .. } => {
                    text.push(' ');
                    text.push_str(summary);
                }
                crate::app::state::BlockSection::KeyValueTable(entries) => {
                    for (k, v) in entries {
                        text.push_str(&format!(" {}:{}", k, v));
                    }
                }
                crate::app::state::BlockSection::CodeFence { content, .. } => {
                    text.push(' ');
                    text.push_str(content);
                }
            }
        }

        // 대소문자 무시 검색 필터
        if !query.is_empty() && !text.to_lowercase().contains(&query) {
            continue;
        }

        match_count += 1;
        if match_count > 50 {
            continue; // 최대 50건만 표시
        }

        // 텍스트 첫 줄만 잘라서 표시
        let preview = truncate_str(&text, 60);
        let label_color = match label {
            "User" => p.accent,
            "AI" | "AI…" => p.success,
            "Sys" => p.warning,
            _ => p.info,
        };

        lines.push(Line::from(vec![
            Span::styled(format!(" #{:<3} ", idx + 1), Style::default().fg(p.muted)),
            Span::styled(format!("{:<5}", label), Style::default().fg(label_color)),
            Span::styled(preview, Style::default().fg(p.text_primary)),
        ]));
    }

    // 결과 요약
    lines.push(Line::from(""));
    let summary_text = if match_count == 0 {
        if state.ui.timeline.is_empty() {
            " (타임라인이 비어 있습니다) ".to_string()
        } else {
            " (검색 결과 없음) ".to_string()
        }
    } else if match_count > 50 {
        format!(" {} 건 중 50건 표시", match_count)
    } else {
        format!(" {} 건 검색됨", match_count)
    };
    lines.push(Line::from(Span::styled(
        summary_text,
        Style::default().fg(p.muted).add_modifier(Modifier::ITALIC),
    )));

    // [v0.1.0-beta.24] Phase 14-B 감사 보정: inspector_scroll 사용으로 독립 스크롤
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false })
        .scroll((state.ui.inspector_scroll, 0));
    f.render_widget(para, area);
}

/// 문자열을 최대 max_len까지 잘라서 반환하는 유틸리티.
/// 긴 텍스트의 첫 줄만 추출하고 초과 시 "…" 표시.
fn truncate_str(s: &str, max_len: usize) -> String {
    let first_line = s.lines().next().unwrap_or("");
    if first_line.chars().count() > max_len {
        format!("{}…", first_line.chars().take(max_len).collect::<String>())
    } else {
        first_line.to_string()
    }
}

/// Recent 탭 렌더링: 최근에 실행된 도구들의 요약 목록 표시
pub fn render_recent(f: &mut Frame, state: &AppState, area: Rect) {
    // [v0.1.0-beta.21] 동적 팔레트 참조
    let p = state.palette();

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        " [ Recent Tools ] ",
        Style::default().fg(p.success),
    )));
    lines.push(Line::from(""));

    let tool_entries: Vec<_> = state
        .ui
        .timeline
        .iter()
        .filter_map(|block| {
            if block.kind == crate::app::state::TimelineBlockKind::ToolRun {
                let mut summary = "";
                for section in &block.body {
                    if let crate::app::state::BlockSection::ToolSummary { summary: s, .. } = section {
                        summary = s;
                        break;
                    }
                }
                Some((&block.title, &block.status, summary))
            } else {
                None
            }
        })
        .rev()
        .take(10)
        .collect();

    if tool_entries.is_empty() {
        lines.push(Line::from(" (No tools executed yet) "));
    } else {
        for (name, status, summary) in tool_entries {
            let status_char = match status {
                crate::app::state::BlockStatus::Done => "✅",
                crate::app::state::BlockStatus::Error => "❌",
                _ => "⏳",
            };
            lines.push(Line::from(vec![
                Span::styled(format!(" {} ", status_char), Style::default()),
                Span::styled(format!("{:<15}", name), Style::default().fg(p.accent)),
                Span::styled(
                    format!(" : {}", summary),
                    Style::default().fg(p.text_secondary),
                ),
            ]));
        }
    }

    // [v0.1.0-beta.24] Phase 14-B 감사 보정: inspector_scroll 사용으로 독립 스크롤
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false })
        .scroll((state.ui.inspector_scroll, 0));
    f.render_widget(para, area);
}
