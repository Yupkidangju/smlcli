// [v0.1.0-beta.18] Phase 9-A: Semantic Palette 모듈.
// 모든 TUI 색상을 의미 기반(semantic)으로 통일하여 UI 일관성을 확보한다.
// 하드코딩된 Color::Yellow, Color::DarkGray 등을 이 상수로 교체.
// designs.md §21 참조.
// [v0.1.0-beta.20] 고대비 팔레트 추가 및 테마 전환 API.
//   designs.md §21.3/§21.4 요구사항: /theme 명령어로 Default ↔ HighContrast 전환 지원.

use ratatui::style::Color;

// === 기본 테마 전경색 (Foreground) — Default Palette ===

/// 시스템 알림, 상태 정보, 일반 강조 — 파랑
pub const INFO: Color = Color::Rgb(96, 165, 250);

/// 성공 메시지, 완료 표시, diff 추가 라인 — 초록
pub const SUCCESS: Color = Color::Rgb(74, 222, 128);

/// 승인 대기, context 경고, 주의 필요 — 앰버
pub const WARNING: Color = Color::Rgb(251, 191, 36);

/// 에러 메시지, 보안 차단, diff 삭제 라인 — 빨강
pub const DANGER: Color = Color::Rgb(248, 113, 113);

/// 비활성 텍스트, 힌트, 보조 정보 — 회색
pub const MUTED: Color = Color::Rgb(107, 114, 128);

/// 강조 표시, 선택 상태, 활성 탭 — 보라
pub const ACCENT: Color = Color::Rgb(167, 139, 250);

// === 기본 테마 배경색 (Background) ===

/// 전체 화면 배경 — 진한 네이비
pub const BG_BASE: Color = Color::Rgb(17, 24, 39);

/// 패널 배경 (Inspector, 상태바 등) — 어두운 네이비
pub const BG_PANEL: Color = Color::Rgb(31, 41, 55);

/// 카드/팝업/모달 배경 — 중간 네이비
pub const BG_ELEVATED: Color = Color::Rgb(55, 65, 81);

// === 기본 테마 텍스트 ===

/// 기본 텍스트 색상 — 밝은 회백색
pub const TEXT_PRIMARY: Color = Color::Rgb(229, 231, 235);

/// 보조 텍스트 색상 — 중간 회색
pub const TEXT_SECONDARY: Color = Color::Rgb(156, 163, 175);

// === 유틸리티 ===

/// [v0.1.0-beta.18] tick 기반 thinking 스피너 문자 배열.
/// tick_count % 4로 인덱싱하여 ◐ ◓ ◑ ◒ 순환 표시.
pub const SPINNER_FRAMES: [char; 8] = ['⠁', '⠂', '⠄', '⡀', '⢀', '⠠', '⠐', '⠈'];

/// [v0.1.0-beta.18] tool 실행 중 배지 토글 문자.
/// tick_count % 2로 인덱싱하여 ● / ○ 깜빡임.
pub const TOOL_BADGE: [char; 2] = ['●', '○'];

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
