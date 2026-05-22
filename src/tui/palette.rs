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
// [v3.9.0] 11대 RGB 고스케일 세맨틱 팔레트 명세 도입.
//   디자인 현대화 계획(redesign_plan.md) 및 마스터플랜(spec.md)에 따라 bg_lowest 및 outline 필드를 선언하고
//   DEFAULT_PALETTE와 HIGH_CONTRAST_PALETTE의 색상을 동결 디자인 RGB 규격으로 리모델링 완료.

use ratatui::style::Color;

// === 유틸리티 ===

/// [v0.1.0-beta.18] tick 기반 thinking 스피너 문자 배열.
/// tick_count % 4로 인덱싱하여 ◐ ◓ ◑ ◒ 순환 표시.
pub const SPINNER_FRAMES: [char; 8] = ['⠁', '⠂', '⠄', '⡀', '⢀', '⠠', '⠐', '⠈'];

// === [v3.9.0] 테마 전환 시스템 및 11대 고스케일 세맨틱 팔레트 ===

/// 팔레트 구조체: 테마별 색상을 캡슐화.
/// [v3.9.0] bg_lowest, outline 필드가 명시적으로 추가되어 총 13개 필드로 구성됨.
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
    pub bg_lowest: Color, // [v3.9.0] Composer 입력창 등 최저층 배경색
    pub text_primary: Color,
    pub text_secondary: Color,
    pub outline: Color, // [v3.9.0] 기본 테두리선 및 구조물 표현색
}

/// 기본 (Default) 팔레트: 침전 딥 네이비 기반의 고스케일 UI 팔레트.
/// [v3.9.0] redesign_plan.md §2.1 컬러 시스템 동결 사양 적용.
pub const DEFAULT_PALETTE: Palette = Palette {
    info: Color::Rgb(190, 198, 224), // #bec6e0 (System 턴, 일반 메타 텍스트)
    success: Color::Rgb(129, 201, 149), // #81c995 (DONE 상태, Git 세이브 성공)
    warning: Color::Rgb(255, 184, 105), // #ffb869 (Needs Approval 턴, 경고 배지)
    danger: Color::Rgb(255, 180, 171), // #ffb4ab (Error 상태, 실패 배지)
    muted: Color::Rgb(107, 114, 128), // #6b7280 (기존 호환성 유지용 보조 메타 색상)
    accent: Color::Rgb(208, 188, 255), // #d0bcff (User 턴, active 테두리, 프롬프트)
    bg_base: Color::Rgb(5, 20, 36),  // #051424 (기본 뷰포트 배경)
    bg_panel: Color::Rgb(18, 33, 49), // #122131 (상단바, 인스펙터 탭바, 툴바)
    bg_elevated: Color::Rgb(28, 43, 60), // #1c2b3c (모달 내부, 포커스된 블록)
    bg_lowest: Color::Rgb(1, 15, 31), // #010f1f (Composer 입력창 배경)
    text_primary: Color::Rgb(212, 228, 250), // #d4e4fa (기본 코드 리터럴, 강한 텍스트)
    text_secondary: Color::Rgb(203, 195, 215), // #cbc3d7 (설명, 주석, 타임스탬프)
    outline: Color::Rgb(149, 142, 160), // #958ea0 (기본 테두리선, 아이콘 기호)
};

/// 고대비 (High Contrast) 팔레트: 접근성 지원을 위해 색상 대비 극대화.
/// [v3.9.0] bg_lowest(Black) 및 outline(White) 필드 추가 매핑 완료.
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
    bg_lowest: Color::Black,
    text_primary: Color::White,
    text_secondary: Color::Gray,
    outline: Color::White,
};

/// 테마 이름 문자열로부터 적절한 Palette 참조를 반환.
/// "high_contrast" → HIGH_CONTRAST_PALETTE, 그 외 → DEFAULT_PALETTE.
pub fn get_palette(theme: &str) -> &'static Palette {
    match theme {
        "high_contrast" => &HIGH_CONTRAST_PALETTE,
        _ => &DEFAULT_PALETTE,
    }
}
