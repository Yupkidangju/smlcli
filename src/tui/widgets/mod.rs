pub mod config_dashboard;
pub mod input_field;
pub mod inspector_tabs;
// [v3.7.0] Phase 47: Interactive Planning Questionnaire 위젯.
pub mod questionnaire;
pub mod setting_wizard;

/// [v2.5.0] Phase 33: 어댑티브 UI 로케일 지원.
/// 유니코드 지원이 불안정한 환경이나, 사용자가 명시적으로 ASCII 보더를 요청한 경우
/// ASCII 기반 보더셋(+, -, |)을 반환한다.
pub fn get_border_set(use_ascii_borders: bool) -> ratatui::symbols::border::Set<'static> {
    let mut is_ascii_fallback = use_ascii_borders;
    if !is_ascii_fallback && let Ok(lang) = std::env::var("LANG") {
        let lang = lang.to_uppercase();
        if lang == "C" || lang == "POSIX" || !lang.contains("UTF-8") {
            is_ascii_fallback = true;
        }
    }
    if is_ascii_fallback {
        ratatui::symbols::border::Set {
            top_left: "+",
            top_right: "+",
            bottom_left: "+",
            bottom_right: "+",
            horizontal_top: "-",
            horizontal_bottom: "-",
            vertical_left: "|",
            vertical_right: "|",
        }
    } else {
        ratatui::symbols::border::PLAIN
    }
}

pub fn block_with_borders<'a>(
    borders: ratatui::widgets::Borders,
    use_ascii_borders: bool,
) -> ratatui::widgets::Block<'a> {
    ratatui::widgets::Block::default()
        .borders(borders)
        .border_set(get_border_set(use_ascii_borders))
}
