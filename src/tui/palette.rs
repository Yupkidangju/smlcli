// [v0.1.0-beta.18] Phase 9-A: Semantic Palette 모듈.
// 모든 TUI 색상을 의미 기반(semantic)으로 통일하여 UI 일관성을 확보한다.
// designs.md §21 참조.
// [v0.1.0-beta.20] 고대비 팔레트 추가 및 테마 전환 API.
//   designs.md §21.3/§21.4 요구사항: /theme 명령어로 Default ↔ HighContrast 전환 지원.
// [v3.7.0] 레거시 개별 상수(INFO, SUCCESS 등)는 Palette 구조체 도입 후 전면 교체되어 삭제됨.
//   삭제 사유: v0.1.0-beta.20에서 Palette 구조체 + get_palette() 패턴으로 전환 완료.
//   삭제된 상수: INFO, SUCCESS, WARNING, DANGER, MUTED, ACCENT,
//              BG_BASE, BG_PANEL, BG_ELEVATED, TEXT_PRIMARY, TEXT_SECONDARY, TOOL_BADGE.
//   삭제된 버전: v3.7.0.

use ratatui::style::Color;

// === 유틸리티 ===

/// [v0.1.0-beta.18] tick 기반 thinking 스피너 문자 배열.
/// tick_count % 4로 인덱싱하여 ◐ ◓ ◑ ◒ 순환 표시.
pub const SPINNER_FRAMES: [char; 8] = ['⠁', '⠂', '⠄', '⡀', '⢀', '⠠', '⠐', '⠈'];

// === [v0.1.0-beta.20] 테마 전환 시스템 ===

/// 팔레트 구조체: 테마별 색상을 캡슐화.
/// get_palette()를 통해 현재 설정된 테마에 맞는 팔레트를 반환.
#[derive(Debug, Clone)]
pub struct Palette {
    pub info: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub muted: Color,
    pub accent: Color,
    pub bg_base: Color,
    pub bg_panel: Color,
    pub bg_elevated: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
}

/// 기본 (Default) 팔레트: 어두운 네이비 배경 + 부드러운 색상.
pub const DEFAULT_PALETTE: Palette = Palette {
    info: Color::Rgb(96, 165, 250),
    success: Color::Rgb(74, 222, 128),
    warning: Color::Rgb(251, 191, 36),
    danger: Color::Rgb(248, 113, 113),
    muted: Color::Rgb(107, 114, 128),
    accent: Color::Rgb(167, 139, 250),
    bg_base: Color::Rgb(17, 24, 39),
    bg_panel: Color::Rgb(31, 41, 55),
    bg_elevated: Color::Rgb(55, 65, 81),
    text_primary: Color::Rgb(229, 231, 235),
    text_secondary: Color::Rgb(156, 163, 175),
};

/// 고대비 (High Contrast) 팔레트: 접근성 지원을 위해 색상 대비 극대화.
/// designs.md §21.3 사양에 따라 순수 원색 위주.
pub const HIGH_CONTRAST_PALETTE: Palette = Palette {
    info: Color::Cyan,
    success: Color::Green,
    warning: Color::Yellow,
    danger: Color::Red,
    muted: Color::DarkGray,
    accent: Color::Magenta,
    bg_base: Color::Black,
    bg_panel: Color::Rgb(16, 16, 16),
    bg_elevated: Color::Rgb(32, 32, 32),
    text_primary: Color::White,
    text_secondary: Color::Gray,
};

/// 테마 이름 문자열로부터 적절한 Palette 참조를 반환.
/// "high_contrast" → HIGH_CONTRAST_PALETTE, 그 외 → DEFAULT_PALETTE.
pub fn get_palette(theme: &str) -> &'static Palette {
    match theme {
        "high_contrast" => &HIGH_CONTRAST_PALETTE,
        _ => &DEFAULT_PALETTE,
    }
}
