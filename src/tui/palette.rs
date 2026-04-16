// [v0.1.0-beta.18] Phase 9-A: Semantic Palette 모듈.
// 모든 TUI 색상을 의미 기반(semantic)으로 통일하여 UI 일관성을 확보한다.
// 하드코딩된 Color::Yellow, Color::DarkGray 등을 이 상수로 교체.
// designs.md §21 참조.

use ratatui::style::Color;

// === 전경색 (Foreground) ===

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

// === 배경색 (Background) ===

/// 전체 화면 배경 — 진한 네이비
pub const BG_BASE: Color = Color::Rgb(17, 24, 39);

/// 패널 배경 (Inspector, 상태바 등) — 어두운 네이비
pub const BG_PANEL: Color = Color::Rgb(31, 41, 55);

/// 카드/팝업/모달 배경 — 중간 네이비
pub const BG_ELEVATED: Color = Color::Rgb(55, 65, 81);

// === 텍스트 ===

/// 기본 텍스트 색상 — 밝은 회백색
pub const TEXT_PRIMARY: Color = Color::Rgb(229, 231, 235);

/// 보조 텍스트 색상 — 중간 회색
pub const TEXT_SECONDARY: Color = Color::Rgb(156, 163, 175);

// === 유틸리티 ===

/// [v0.1.0-beta.18] tick 기반 thinking 스피너 문자 배열.
/// tick_count % 4로 인덱싱하여 ◐ ◓ ◑ ◒ 순환 표시.
pub const SPINNER_FRAMES: [char; 4] = ['◐', '◓', '◑', '◒'];

/// [v0.1.0-beta.18] tool 실행 중 배지 토글 문자.
/// tick_count % 2로 인덱싱하여 ● / ○ 깜빡임.
pub const TOOL_BADGE: [char; 2] = ['●', '○'];
