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

    // [v0.1.0-beta.22] Wrap + scroll 적용 — PageUp/PageDown으로 탐색 가능
    let para = Paragraph::new(log_lines)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false })
        .scroll((state.ui.timeline_scroll, 0));

    f.render_widget(para, area);
}

/// [v0.1.0-beta.20] Search 탭 렌더링: 타임라인 내의 텍스트 검색 결과 표시.
/// Composer 입력 버퍼를 검색어로 사용하여 타임라인 전체를 대소문자 무시로 필터링.
/// 검색어가 비어있으면 전체 텍스트 콘텐츠를 인덱스 형태로 표시한다.
pub fn render_search(f: &mut Frame, state: &AppState, area: Rect) {
    use crate::app::state::TimelineEntryKind;

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
    for (idx, entry) in state.ui.timeline.iter().enumerate() {
        let (label, text) = match &entry.kind {
            TimelineEntryKind::UserMessage(s) => ("User", s.as_str()),
            TimelineEntryKind::AssistantMessage(s) => ("AI", s.as_str()),
            TimelineEntryKind::AssistantDelta(s) => ("AI…", s.as_str()),
            TimelineEntryKind::SystemNotice(s) => ("Sys", s.as_str()),
            TimelineEntryKind::ToolCard {
                tool_name, summary, ..
            } => {
                // 도구 카드: 이름과 요약을 결합하여 검색
                let combined = format!("{}: {}", tool_name, summary);
                if query.is_empty() || combined.to_lowercase().contains(&query) {
                    match_count += 1;
                    if match_count <= 50 {
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!(" #{:<3} ", idx + 1),
                                Style::default().fg(p.muted),
                            ),
                            Span::styled("Tool ", Style::default().fg(p.warning)),
                            Span::styled(
                                truncate_str(&combined, 60),
                                Style::default().fg(p.text_primary),
                            ),
                        ]));
                    }
                }
                continue;
            }
            TimelineEntryKind::ApprovalCard { tool_name, detail } => {
                let combined = format!("Approve {}: {}", tool_name, detail);
                if query.is_empty() || combined.to_lowercase().contains(&query) {
                    match_count += 1;
                    if match_count <= 50 {
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!(" #{:<3} ", idx + 1),
                                Style::default().fg(p.muted),
                            ),
                            Span::styled("Appr ", Style::default().fg(p.warning)),
                            Span::styled(
                                truncate_str(&combined, 60),
                                Style::default().fg(p.text_primary),
                            ),
                        ]));
                    }
                }
                continue;
            }
            TimelineEntryKind::CompactSummary(s) => ("Σ", s.as_str()),
        };

        // 대소문자 무시 검색 필터
        if !query.is_empty() && !text.to_lowercase().contains(&query) {
            continue;
        }

        match_count += 1;
        if match_count > 50 {
            continue; // 최대 50건만 표시
        }

        // 텍스트 첫 줄만 잘라서 표시
        let preview = truncate_str(text, 60);
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

    // [v0.1.0-beta.22] Wrap + scroll 적용
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false })
        .scroll((state.ui.timeline_scroll, 0));
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
        .filter_map(|e| {
            if let crate::app::state::TimelineEntryKind::ToolCard {
                tool_name,
                status,
                summary,
            } = &e.kind
            {
                Some((tool_name, status, summary))
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
                crate::app::state::ToolStatus::Done => "✅",
                crate::app::state::ToolStatus::Error => "❌",
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

    // [v0.1.0-beta.22] Wrap + scroll 적용
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false })
        .scroll((state.ui.timeline_scroll, 0));
    f.render_widget(para, area);
}
