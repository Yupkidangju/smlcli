// [v3.9.0] Phase 4: Floating Modals & Command Palette 완성
// AskClarification 도구가 호출되면 타임라인 위에 오버레이 모달로
// 질문 폼을 렌더링한다. 객관식은 화살표로 탐색/Enter로 선택,
// 주관식은 텍스트 입력 후 Enter로 제출.
// designs.md 및 spec.md 규격에 따른 가로 60%, 세로 45% 수학적 정렬 및 5대어 다국어(i18n) 키 매핑 실연결.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::domain::questionnaire::QuestionnaireState;

/// Questionnaire 오버레이 위젯.
/// 전체 화면의 중앙에 모달로 렌더링됨.
pub struct QuestionnaireWidget<'a> {
    pub state: &'a QuestionnaireState,
    pub use_ascii_borders: bool,
    pub palette: crate::tui::palette::Palette,
    pub i18n: &'a crate::tui::i18n::I18nManager, // [v3.9.0] 다국어 매니저 연동 추가
}

impl<'a> QuestionnaireWidget<'a> {
    pub fn new(
        state: &'a QuestionnaireState,
        use_ascii_borders: bool,
        palette: crate::tui::palette::Palette,
        i18n: &'a crate::tui::i18n::I18nManager,
    ) -> Self {
        Self {
            state,
            use_ascii_borders,
            palette,
            i18n,
        }
    }

    /// 화면 중앙에 모달 영역을 계산 (가로 60%, 세로 45% 수학적 중앙 정렬).
    pub fn centered_rect(area: Rect) -> Rect {
        let width = ((area.width as u32 * 60) / 100)
            .max(40)
            .min(area.width as u32) as u16;
        let height = ((area.height as u32 * 45) / 100)
            .max(10)
            .min(area.height as u32) as u16;
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        Rect::new(x, y, width, height)
    }
}

impl Widget for QuestionnaireWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let modal_area = Self::centered_rect(area);

        // [v3.9.0] 네이티브 Clear 위젯을 활용하여 이전 화면 잔상을 완전히 소거
        Clear.render(modal_area, buf);

        let Some(question) = self.state.current_question() else {
            return;
        };

        // [v3.9.0] 다국어 사전 키("question_progress")를 기반으로 한 진행도 포맷팅
        let raw_progress_fmt = self.i18n.tr("question_progress");
        let progress_text = raw_progress_fmt
            .replacen("{}", &(self.state.current_index + 1).to_string(), 1)
            .replacen("{}", &self.state.questions.len().to_string(), 1);

        // [v3.9.0] 다국어 타이틀("questionnaire_title")과 진행도 조합
        let title_label = self.i18n.tr("questionnaire_title");
        let header_title = format!(" {} ({}) ", title_label, progress_text);

        let block = Block::default()
            .title(header_title)
            .borders(Borders::ALL)
            .border_set(super::get_border_set(self.use_ascii_borders))
            .border_style(Style::default().fg(self.palette.accent))
            .style(Style::default().bg(self.palette.bg_panel));

        let inner = block.inner(modal_area);
        block.render(modal_area, buf);

        // 질문 제목
        let title_style = Style::default()
            .fg(self.palette.warning)
            .add_modifier(Modifier::BOLD);
        let title_line = Line::from(Span::styled(&question.title, title_style));

        // 질문 제목 렌더링 (2줄 확보)
        let title_para = Paragraph::new(title_line).wrap(Wrap { trim: true });
        let title_area = Rect::new(inner.x, inner.y, inner.width, 2);
        title_para.render(title_area, buf);

        // 옵션 또는 입력 필드 영역
        let options_area = Rect::new(
            inner.x,
            inner.y + 2,
            inner.width,
            inner.height.saturating_sub(2),
        );

        if question.options.is_empty() || self.state.is_custom_input_mode {
            // 주관식 또는 직접 입력 모드: 텍스트 입력 표시
            let prompt_text = if self.state.is_custom_input_mode {
                self.i18n.tr("custom_input_prompt")
            } else {
                self.i18n.tr("input_prompt")
            };
            let input_lines = vec![
                Line::from(Span::styled(
                    prompt_text,
                    Style::default().fg(self.palette.text_secondary),
                )),
                Line::from(Span::styled(
                    format!("▸ {}▏", self.state.custom_input),
                    Style::default().fg(self.palette.text_primary),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    self.i18n.tr("input_hint"),
                    Style::default().fg(self.palette.text_secondary),
                )),
            ];
            Paragraph::new(input_lines).render(options_area, buf);
        } else {
            // 객관식: 옵션 목록 렌더링
            let mut lines: Vec<Line> = Vec::new();

            for (i, option) in question.options.iter().enumerate() {
                let is_selected = i == self.state.option_cursor;
                let marker = if is_selected { "▸ " } else { "  " };
                let style = if is_selected {
                    Style::default()
                        .fg(self.palette.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.palette.text_primary)
                };
                lines.push(Line::from(Span::styled(
                    format!("{}{}", marker, option),
                    style,
                )));
            }

            // allow_custom인 경우 "직접 입력" 선택지 추가
            if question.allow_custom {
                let custom_idx = question.options.len();
                let is_selected = custom_idx == self.state.option_cursor;
                let marker = if is_selected { "▸ " } else { "  " };
                let style = if is_selected {
                    Style::default()
                        .fg(self.palette.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.palette.text_secondary)
                };
                let custom_label = self.i18n.tr("custom_option");
                lines.push(Line::from(Span::styled(
                    format!("{}{}", marker, custom_label),
                    style,
                )));
            }

            // 하단 힌트
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                self.i18n.tr("select_hint"),
                Style::default().fg(self.palette.text_secondary),
            )));

            Paragraph::new(lines).render(options_area, buf);
        }
    }
}
