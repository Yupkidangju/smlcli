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
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;
use unicode_width::UnicodeWidthStr;

pub struct RenderCache {
    pub lines_cache: HashMap<usize, Line<'static>>,
    pub is_dirty: bool,
    pub last_total_lines: usize,
}

// [v2.1.0] Phase 29: 스크롤 앵커 상태 관리
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScrollAnchor {
    pub absolute_line_index: usize,
    pub active: bool,
}

thread_local! {
    static LOGS_RENDER_CACHE: std::cell::RefCell<RenderCache> = std::cell::RefCell::new(RenderCache {
        lines_cache: HashMap::new(),
        is_dirty: true,
        last_total_lines: 0,
    });
}

fn ansi_regex() -> &'static Regex {
    static ANSI_RE: OnceLock<Regex> = OnceLock::new();
    ANSI_RE.get_or_init(|| Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap())
}

pub fn render_preview(f: &mut Frame, state: &AppState, area: Rect) {
    let p = state.palette();

    if state.ui.timeline.is_empty() {
        let text = "No active blocks.\n\nTimeline is empty.";
        let paragraph = Paragraph::new(text).style(Style::default().fg(p.muted));
        f.render_widget(paragraph, area);
        return;
    }

    let cursor = state
        .ui
        .timeline_cursor
        .min(state.ui.timeline.len().saturating_sub(1));
    let block = &state.ui.timeline[cursor];

    let mut lines = Vec::new();
    lines.push(Line::from(vec![Span::styled(
        format!("Block: {}", block.title),
        Style::default().fg(p.accent).add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    for section in &block.body {
        match section {
            crate::app::state::BlockSection::Markdown(msg) => {
                for line in msg.lines() {
                    lines.push(Line::from(Span::styled(
                        line,
                        Style::default().fg(p.text_primary),
                    )));
                }
            }
            crate::app::state::BlockSection::ToolSummary { tool_name, summary } => {
                lines.push(Line::from(Span::styled(
                    format!("Tool: {}", tool_name),
                    Style::default().fg(p.info),
                )));
                for line in summary.lines() {
                    lines.push(Line::from(Span::styled(
                        line,
                        Style::default().fg(p.text_secondary),
                    )));
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
                lines.push(Line::from(Span::styled(
                    format!("```{}", lang_str),
                    Style::default().fg(p.muted),
                )));
                for line in content.lines() {
                    lines.push(Line::from(Span::styled(
                        line,
                        Style::default().fg(p.text_primary),
                    )));
                }
                lines.push(Line::from(Span::styled(
                    "```",
                    Style::default().fg(p.muted),
                )));
            }
        }
        lines.push(Line::from(""));
    }

    let top_scroll = if lines.len() > area.height as usize {
        lines
            .len()
            .saturating_sub(area.height as usize)
            .saturating_sub(state.ui.inspector_scroll.get() as usize) as u16
    } else {
        0
    };

    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false })
        .scroll((top_scroll, 0));

    f.render_widget(para, area);
}

/// Diff 탭 렌더링: 승인 대기 중인 변경사항 미리보기.
pub fn render_diff(f: &mut Frame, state: &AppState, area: Rect) {
    let p = state.palette();

    if let Some(diff) = &state.runtime.approval.diff_preview {
        let lines: Vec<Line> = diff
            .lines()
            .map(|line| {
                let color = if line.starts_with('+') {
                    p.success
                } else if line.starts_with('-') {
                    p.danger
                } else {
                    p.text_primary
                };
                Line::from(Span::styled(line, Style::default().fg(color)))
            })
            .collect();

        let top_scroll = if lines.len() > area.height as usize {
            lines
                .len()
                .saturating_sub(area.height as usize)
                .saturating_sub(state.ui.inspector_scroll.get() as usize) as u16
        } else {
            0
        };

        let para = Paragraph::new(lines)
            .block(Block::default().borders(Borders::NONE))
            .wrap(Wrap { trim: false })
            .scroll((top_scroll, 0));
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
    // [v0.1.0-beta.26] logs_buffer는 비동기 태스크가 직접 수정하지 않고,
    // 모두 Event::Action -> handle_action 경유로 직렬화되어 반영된다.
    // 따라서 렌더링 시점에는 단일 이벤트 루프 소유권 아래 일관된 스냅샷을 읽는다.

    let total_lines = state.runtime.logs_buffer.len();
    if total_lines == 0 {
        let para = Paragraph::new(vec![Line::from(Span::styled(
            " (No logs recorded in this session) ",
            Style::default().fg(p.muted).add_modifier(Modifier::ITALIC),
        ))])
        .block(Block::default().borders(Borders::NONE));
        f.render_widget(para, area);
        return;
    }

    // [v1.0.0] 윈도우 기반 렌더링 인덱싱 오류 수정 (Panic 방지)
    let display_height = area.height as usize;
    let current_scroll = state.ui.inspector_scroll.get() as usize;

    // 스크롤 상한선 설정
    let mut clamped_scroll = current_scroll.clamp(0, total_lines.saturating_sub(display_height));

    // inspector_scroll은 bottom-up (최신부터) 기준이므로 변환
    let mut start_idx = total_lines
        .saturating_sub(display_height)
        .saturating_sub(clamped_scroll);

    // [v2.1.0] Phase 29: Scroll Anchoring
    let last_height = state.ui.last_inspector_height.get();
    let mut anchor = state.ui.inspector_anchor.get();

    if last_height != display_height && last_height > 0 && anchor.active {
        // 리사이즈 발생: 앵커 기준으로 start_idx 강제 고정
        start_idx = anchor
            .absolute_line_index
            .min(total_lines.saturating_sub(display_height));

        // 역계산 (re-calc): 새로운 start_idx에 맞게 clamped_scroll 및 state.ui.inspector_scroll 갱신
        clamped_scroll = total_lines
            .saturating_sub(display_height)
            .saturating_sub(start_idx);
        state.ui.inspector_scroll.set(clamped_scroll as u16);
    } else {
        // 평상시: 현재 렌더링될 start_idx를 앵커로 기억
        anchor.absolute_line_index = start_idx;
        anchor.active = true;
    }

    state.ui.last_inspector_height.set(display_height);
    state.ui.inspector_anchor.set(anchor);

    let end_idx = (start_idx + display_height).min(total_lines);

    let mut lines_to_render = Vec::with_capacity(display_height);

    LOGS_RENDER_CACHE.with(|cache_ref| {
        let mut cache = cache_ref.borrow_mut();

        if cache.last_total_lines != total_lines {
            cache.is_dirty = true;
            cache.last_total_lines = total_lines;
            // 만약 완전히 리셋되었다면 캐시 클리어
            if total_lines == 0 {
                cache.lines_cache.clear();
            }
        }

        for (i, log) in state.runtime.logs_buffer[start_idx..end_idx]
            .iter()
            .enumerate()
        {
            let abs_idx = start_idx + i;

            if let Some(cached_line) = cache.lines_cache.get(&abs_idx) {
                lines_to_render.push(cached_line.clone());
                continue;
            }

            let color = if abs_idx.is_multiple_of(2) {
                p.text_secondary
            } else {
                p.text_primary
            };

            // [v1.3.0] ANSI 코드 제거 (Strip)
            let clean_log = ansi_regex().replace_all(log.as_str(), "");
            // [v1.4.0] 긴 출력에 의한 Soft Wrap CPU 스파이크를 방지하기 위해 200자로 Hard Wrap (자르기)
            let max_width = 250;
            let clean_str = if clean_log.width() > max_width {
                let mut w = 0;
                let mut res = String::new();
                for c in clean_log.chars() {
                    let cw = c.to_string().width();
                    if w + cw > max_width {
                        break;
                    }
                    res.push(c);
                    w += cw;
                }
                format!("{}... (truncated for UI perf)", res)
            } else {
                clean_log.into_owned()
            };

            let line = Line::from(Span::styled(clean_str, Style::default().fg(color)));

            cache.lines_cache.insert(abs_idx, line.clone());
            lines_to_render.push(line);
        }
    });

    let para = Paragraph::new(lines_to_render).block(Block::default().borders(Borders::NONE));

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
            crate::app::state::TimelineBlockKind::GitCommit => "Git",
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

    let top_scroll = if lines.len() > area.height as usize {
        lines
            .len()
            .saturating_sub(area.height as usize)
            .saturating_sub(state.ui.inspector_scroll.get() as usize) as u16
    } else {
        0
    };

    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false })
        .scroll((top_scroll, 0));
    f.render_widget(para, area);
}

/// 문자열을 실제 화면 폭(width) 기준으로 최대 max_width까지 잘라서 반환하는 유틸리티.
/// 긴 텍스트의 첫 줄만 추출하고 초과 시 "…" 표시.
fn truncate_str(s: &str, max_width: usize) -> String {
    let first_line = s.lines().next().unwrap_or("");
    if first_line.width() > max_width {
        let mut width = 0;
        let mut result = String::new();
        for c in first_line.chars() {
            let cw = c.to_string().width();
            if width + cw > max_width.saturating_sub(1) {
                break;
            }
            result.push(c);
            width += cw;
        }
        format!("{}…", result)
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
                    if let crate::app::state::BlockSection::ToolSummary { summary: s, .. } = section
                    {
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

    let top_scroll = if lines.len() > area.height as usize {
        lines
            .len()
            .saturating_sub(area.height as usize)
            .saturating_sub(state.ui.inspector_scroll.get() as usize) as u16
    } else {
        0
    };

    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false })
        .scroll((top_scroll, 0));
    f.render_widget(para, area);
}

// [v3.0.0] Phase 40: Git-Native Integration - Git 히스토리 탭 렌더링
struct GitCache {
    history: Vec<crate::infra::git_engine::CheckpointEntry>,
    last_fetch: std::time::Instant,
}

thread_local! {
    static GIT_RENDER_CACHE: std::cell::RefCell<GitCache> = std::cell::RefCell::new(GitCache {
        history: Vec::new(),
        last_fetch: std::time::Instant::now() - std::time::Duration::from_secs(100),
    });
}

pub fn render_git(f: &mut Frame, state: &AppState, area: Rect) {
    let p = state.palette();

    let mut history_lines = Vec::new();

    GIT_RENDER_CACHE.with(|cache_ref| {
        let mut cache = cache_ref.borrow_mut();
        // 2초마다 갱신
        if cache.last_fetch.elapsed().as_secs() >= 2 {
            let cwd = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string());
            // [v2.5.1] 감사 MEDIUM-1: prefix 필터로 smlcli 생성 커밋만 표시
            let prefix = state
                .domain
                .settings
                .as_ref()
                .map(|s| s.git_integration.commit_prefix.as_str())
                .unwrap_or("smlcli: ");
            if let Ok(history) = crate::infra::git_engine::GitEngine::list_history(&cwd, prefix, 50)
            {
                cache.history = history;
            }
            cache.last_fetch = std::time::Instant::now();
        }
        history_lines = cache.history.clone();
    });

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        " [ Git Commit History ] ",
        Style::default().fg(p.success),
    )));
    lines.push(Line::from(""));

    if history_lines.is_empty() {
        lines.push(Line::from(Span::styled(
            " (No Git history found or not a Git repository) ",
            Style::default().fg(p.muted).add_modifier(Modifier::ITALIC),
        )));
    } else {
        for entry in history_lines {
            let short_hash = entry.commit_hash.chars().take(7).collect::<String>();
            lines.push(Line::from(vec![
                Span::styled(format!("{:<8} ", short_hash), Style::default().fg(p.accent)),
                Span::styled(
                    format!("{} ", entry.author),
                    Style::default().fg(p.text_secondary),
                ),
            ]));
            for msg_line in entry.message.lines() {
                if !msg_line.trim().is_empty() {
                    lines.push(Line::from(Span::styled(
                        format!("    {}", msg_line),
                        Style::default().fg(p.text_primary),
                    )));
                }
            }
            lines.push(Line::from(""));
        }
    }

    let top_scroll = if lines.len() > area.height as usize {
        lines
            .len()
            .saturating_sub(area.height as usize)
            .saturating_sub(state.ui.inspector_scroll.get() as usize) as u16
    } else {
        0
    };

    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false })
        .scroll((top_scroll, 0));
    f.render_widget(para, area);
}
