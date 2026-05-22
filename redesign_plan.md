# smlcli TUI 디자인 리팩토링 최종 설계 참조 문서 (redesign_plan.md)

> [!IMPORTANT]
> 본 문서는 [AI_IMPLEMENTATION_DOC_STANDARD.md](file:///mnt/Projects_SSD/rust/smlcli/AI_IMPLEMENTATION_DOC_STANDARD.md)의 **"Reference Grade"** 표준 규격을 100% 충족하도록 설계되었습니다.
> 이 문서의 목적은 추가 기획 회의 없이 구현자가 실제 Rust 및 `ratatui` + `crossterm` 환경에서 리팩토링 코드를 즉시 작성 및 검증할 수 있도록 모든 의사결정을 동결하고, 구체적인 수치와 데이터 계약을 제시하는 것입니다.
> 
> D3D Protocol(Global Rules)에 의거하여, 마스터플랜인 [spec.md](file:///mnt/Projects_SSD/rust/smlcli/spec.md) 및 디자인 규격서 [designs.md](file:///mnt/Projects_SSD/rust/smlcli/designs.md)의 방향성을 완벽하게 계승하며, **UTF-8 인코딩**과 **한국어 문서 표준**을 엄격히 준수합니다.

---

## 1. 스코프 정의 및 경계 격리 (Scope Closure)

### 1.1 프로젝트 정체성
* `smlcli`는 Codex/OpenCode 계열의 정밀성과 Modern IDE의 인체공학적 감각을 제공하는 **터미널 네이티브 작업 제어 콘솔(Terminal-Native Task Console)**이다.
* 본 리팩토링은 기존 `smlcli` TUI의 텍스트 위주 평면 레이아웃을 제공된 웹 기반 현대적 디자인 `stitch_modern_tui_redesign` (Terminal Precision 테마 및 블록 기반 콘솔)의 극초프리미엄 비주얼 감각으로 이식하는 것을 핵심 골자로 한다.

### 1.2 명시적 목표 (Goals)
1. **Terminal Precision 테마 이식:** 딥 네이비 기반의 4단계 톤 레이어(#051424, #122131, #1c2b3c, #010f1f) 및 퍼플(#d0bcff), 오렌지(#ffb869)의 악센트 RGB 팔레트 적용.
2. **블록 기반 타임라인 고도화:** 각 대화/도구 턴(Turn)을 독립된 `ratatui::widgets::Block` 단위로 캡슐화하고 좌측 세로선 액센트(`Borders::LEFT`)를 통해 역할을 시각적으로 강제 그룹화.
3. **Tree of Thoughts UI 트리 융합:** `spec.md` 966라인의 `depth: 1` 들여쓰기 지침과 트리선 기호(`└─`, `├─`)를 타임라인 세로선과 오차 없이 시각적으로 결합.
4. **반응형 3분할 뷰포트 레이아웃:** 100자 및 80자 가로폭 기준에 대응하여 상단바, 인스펙터 탭바, 하단 Composer 칩들이 레이아웃을 유기적으로 조절하고 깨짐을 원천 차단.
5. **글로벌 다국어 및 소스코드 한국어 주석:** TUI 내 노출되는 모든 정적 라벨을 다국어(한/영/일/중) 사전에 연동하며, 코드의 주석은 **오직 한국어만** 사용하여 상세히 동작 원리를 기술.

### 1.3 성공 기준 (Success Criteria)
* **clippy 빌드 무결성:** `cargo clippy --all-targets -- -D warnings` 명령 수행 시 경고가 0개여야 한다.
* **TUI 반응형 가드 통과:** 가로폭 `100` 칼럼 미만 시 우측 인스펙터가 드로어(Drawer, F2로 오버레이) 모드로 전환되며, `80` 칼럼 미만 시 CWD(작업 디렉토리) 및 Policy 칩이 우선 숨겨지고 상단바가 중요 상태(Model, Mode, Ctx%)만 노출해야 한다.
* **Diff 렌더링 성능 최적화:** 5000라인의 대용량 Diff 화면에서도 CPU 스파이크가 발생하지 않도록 캐싱 맵 적중률을 95% 이상으로 유지해야 한다.
* **모달 렌더링 좌표 정확도:** 터미널 창 중심점(Center Point) 기준 정중앙 정렬을 수식으로 완벽히 통제하여 오버레이 렌더링 시 겹침 현상이 없어야 한다.

### 1.4 비목표 (Non-Goals)
* LLM API 연동 엔진, ChaCha20Poly1305 설정 저장소의 파일 입출력 메커니즘, Shell 실행 샌드박스의 로직 등 비주얼 렌더링 계층 외부의 코어 시스템을 변경하는 것은 명확히 범위 외로 격리한다.
* Textual 또는 Bubble Tea 등 Rust 외의 다른 언어/프레임워크로 TUI 라이브러리를 전면 교체하지 않는다.

---

## 2. 동결된 디자인 결정 (Frozen Decisions)

### 2.1 컬러 시스템 명세 (Semantic Palette)
TUI 전용 `Semantic Palette` 값을 아래 RGB 수치로 고정하며, 이 외의 하드코딩된 임의의 ANSI/RGB 색상 사용을 전면 불허한다.

```rust
// src/tui/palette.rs 내의 DEFAULT_PALETTE 동결 정의
pub const DEFAULT_PALETTE: Palette = Palette {
    bg_base: Color::Rgb(5, 20, 36),        // #051424 (기본 뷰포트 배경)
    bg_panel: Color::Rgb(18, 33, 49),      // #122131 (상단바, 인스펙터 탭바, 툴바)
    bg_elevated: Color::Rgb(28, 43, 60),   // #1c2b3c (모달 내부, 포커스된 블록)
    bg_lowest: Color::Rgb(1, 15, 31),      // #010f1f (Composer 입력창 배경)
    accent: Color::Rgb(208, 188, 255),     // #d0bcff (User 턴, active 테두리, 프롬프트)
    success: Color::Rgb(129, 201, 149),    // #81c995 (DONE 상태, Git 세이브 성공)
    warning: Color::Rgb(255, 184, 105),    // #ffb869 (Needs Approval 턴, 경고 배지)
    danger: Color::Rgb(255, 180, 171),     // #ffb4ab (Error 상태, 실패 배지)
    info: Color::Rgb(190, 198, 224),       // #bec6e0 (System 턴, 일반 메타 텍스트)
    text_primary: Color::Rgb(212, 228, 250), // #d4e4fa (기본 코드 리터럴, 강한 텍스트)
    text_secondary: Color::Rgb(203, 195, 215), // #cbc3d7 (설명, 주석, 타임스탬프)
    outline: Color::Rgb(149, 142, 160),     // #958ea0 (기본 테두리선, 아이콘 기호)
};
```

### 2.2 터미널 레이아웃 수치 정의 (Concrete Numbers)
TUI 그리드 렌더링에 사용되는 레이아웃 좌표 및 가드 한계값을 아래와 같이 규격화한다.

* **TopAppBar 높이:** 고정 `1` 라인.
* **Bottom Composer 높이:** 고정 `3` 라인.
  - Toolbar (Row 1): `1` 라인.
  - Input Box (Row 2): `2` 라인 (상하 패딩 0셀, Inset 배경).
* **Inspector 폭 분할 비율:**
  - 가로 너비 `>= 100` 칼럼 시: 우측 `30%` 고정 점유 (최소 `32` 칼럼, 최대 `48` 칼럼 클램프 가드 적용).
  - 가로 너비 `< 100` 칼럼 시: 인스펙터 패널 자동 닫힘. `F2` 키를 눌렀을 때만 우측 `35` 칼럼 크기의 Drawer로 화면 우측에 오버레이 렌더링.
* **타임라인 최소 확보폭:** 무조건 `>= 45` 칼럼을 보장해야 하며, 보장되지 않을 경우 레이아웃 렌더링을 일시 중단하고 `Window Too Small (Min Width: 80)` 가이드라인 화면을 출력한다.

---

## 3. 데이터 계약 및 타입 정의 (Typed Contracts)

인터랙션 흐름과 렌더링 상태를 엄격히 바인딩하기 위해 핵심 Rust Data Type을 고정 정의한다.

### 3.1 턴(Turn) 상태 및 표시 모드
```rust
// src/app/state.rs 또는 src/tui/widgets/mod.rs

/// 턴 블록의 동작 상태를 정의하는 열거형
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BlockStatus {
    Running,       // 동작 실행 중
    Done,          // 정상 완료
    Error,         // 실행 실패/에러 발생
    NeedsApproval, // 유저의 승인 대기 중 (주황색 테마)
    Reverted,      // 변경 사항 롤백 완료
}

/// 턴 블록의 타임라인 내 레이아웃 표시 크기 모드
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BlockDisplayMode {
    Collapsed,     // 요약 모드 (3줄 미리보기)
    Expanded,      // 전체 확장 모드
}

/// TUI에 포커싱될 수 있는 활성 패널
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FocusedPane {
    Timeline,      // 중앙 타임라인 히스토리
    Inspector,     // 우측 상세 보기
    Composer,      // 하단 입력 상자
    Palette,       // Ctrl+K 액션 팔레트 (플로팅 오버레이)
    Questionnaire, // 요구사항 확인 설문조사 (플로팅 오버레이)
}

/// 인스펙터에 노출되는 5대 핵심 탭
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum InspectorTab {
    Preview, // 파일 미리보기
    Diff,    // 변경 사항 코드 디프
    Logs,    // 시스템/에이전트 비동기 실행 로그
    Search,  // Grep 등 실시간 검색 뷰
    Git,     // Git 스테이징 및 리포트 상태
}
```

### 3.2 UI 액션 인터페이스 (CTA 및 API 이벤트 계약)
각 UI 조작 발생 시 시스템으로 라우팅되는 이벤트 메시지 타입의 규격을 정의한다.

```rust
/// UI 조작에 따라 발생하는 TUI Action 정의
#[derive(Debug, Clone, PartialEq)]
pub enum TuiAction {
    /// Inspector 탭 스위칭 (Alt+1 ~ Alt+5 연동)
    SwitchTab(InspectorTab),
    /// 턴 블록의 상세 상태 변경 (Approve/Reject 등)
    SetBlockApproval { block_id: String, approved: bool },
    /// Ctrl+K 액션 팔레트 fuzzy search 버퍼 갱신
    UpdatePaletteQuery(String),
    /// 액션 팔레트 리스트 선택 실행
    ExecutePaletteAction(String),
    /// Questionnaire 설문지 답변 선택
    SubmitQuestionAnswer { question_id: String, option_index: usize },
    /// 터미널 창 리사이즈 감지 이벤트
    ResizeTerminal { width: u16, height: u16 },
}

/// src/tui/widgets/inspector_tabs.rs
/// Diff 렌더링 성능 가속을 위한 라인 단위 캐싱 맵
pub struct DiffLineCache {
    /// 파일 경로별 파싱된 tui Line 컬렉션 캐시
    pub cache_map: std::collections::HashMap<std::path::PathBuf, Vec<ratatui::text::Line<'static>>>,
    /// 캐시 타임스탬프 (버전 Bumping용)
    pub last_updated: std::time::Instant,
}
```

---

## 4. 실데이터 및 아스키 렌더링 사양 (Real Data Samples)

### 4.1 UI 다국어 사전 실데이터 (`src/tui/i18n.rs`)
다국어 렌더링에 사용되는 정적 메시지 사전 데이터의 구조와 기본 키값을 완전히 규정한다.

```rust
use std::collections::HashMap;

pub struct I18nManager {
    pub current_lang: String, // "ko", "en", "ja", "zh_TW", "zh_CN"
    translations: HashMap<String, HashMap<String, String>>,
}

impl I18nManager {
    pub fn new(lang: &str) -> Self {
        let mut translations = HashMap::new();
        
        // 1. 한국어 사전 (ko)
        let mut ko = HashMap::new();
        ko.insert("badge_done".to_string(), "완료".to_string());
        ko.insert("badge_pending".to_string(), "대기 중".to_string());
        ko.insert("badge_approval".to_string(), "승인 필요".to_string());
        ko.insert("btn_approve".to_string(), "승인 (Enter)".to_string());
        ko.insert("btn_reject".to_string(), "반려 (Esc)".to_string());
        ko.insert("questionnaire_title".to_string(), "📋 요구사항 확인".to_string());
        ko.insert("tab_preview".to_string(), "미리보기".to_string());
        ko.insert("tab_diff".to_string(), "디프".to_string());
        ko.insert("tab_logs".to_string(), "로그".to_string());
        ko.insert("tab_search".to_string(), "검색".to_string());
        ko.insert("tab_git".to_string(), "깃".to_string());
        translations.insert("ko".to_string(), ko);
        
        // 2. 영어 사전 (en)
        let mut en = HashMap::new();
        en.insert("badge_done".to_string(), "DONE".to_string());
        en.insert("badge_pending".to_string(), "PENDING".to_string());
        en.insert("badge_approval".to_string(), "NEEDS APPROVAL".to_string());
        en.insert("btn_approve".to_string(), "Approve (Enter)".to_string());
        en.insert("btn_reject".to_string(), "Reject (Esc)".to_string());
        en.insert("questionnaire_title".to_string(), "📋 Questionnaire".to_string());
        en.insert("tab_preview".to_string(), "PREVIEW".to_string());
        en.insert("tab_diff".to_string(), "DIFF".to_string());
        en.insert("tab_logs".to_string(), "LOGS".to_string());
        en.insert("tab_search".to_string(), "SEARCH".to_string());
        en.insert("tab_git".to_string(), "GIT".to_string());
        translations.insert("en".to_string(), en);

        // 3. 일본어 사전 (ja)
        let mut ja = HashMap::new();
        ja.insert("badge_done".to_string(), "完了".to_string());
        ja.insert("badge_pending".to_string(), "待機中".to_string());
        ja.insert("badge_approval".to_string(), "承認が必要".to_string());
        ja.insert("btn_approve".to_string(), "承認 (Enter)".to_string());
        ja.insert("btn_reject".to_string(), "却下 (Esc)".to_string());
        ja.insert("questionnaire_title".to_string(), "📋 要件確認".to_string());
        ja.insert("tab_preview".to_string(), "プレビュー".to_string());
        ja.insert("tab_diff".to_string(), "差分".to_string());
        ja.insert("tab_logs".to_string(), "ログ".to_string());
        ja.insert("tab_search".to_string(), "検索".to_string());
        ja.insert("tab_git".to_string(), "ギット".to_string());
        translations.insert("ja".to_string(), ja);

        // 4. 중국어 번체 사전 (zh_TW)
        let mut zh_tw = HashMap::new();
        zh_tw.insert("badge_done".to_string(), "完成".to_string());
        zh_tw.insert("badge_pending".to_string(), "等待中".to_string());
        zh_tw.insert("badge_approval".to_string(), "需要批准".to_string());
        zh_tw.insert("btn_approve".to_string(), "批准 (Enter)".to_string());
        zh_tw.insert("btn_reject".to_string(), "拒絕 (Esc)".to_string());
        zh_tw.insert("questionnaire_title".to_string(), "📋 確認需求".to_string());
        zh_tw.insert("tab_preview".to_string(), "預覽".to_string());
        zh_tw.insert("tab_diff".to_string(), "對比".to_string());
        zh_tw.insert("tab_logs".to_string(), "日誌".to_string());
        zh_tw.insert("tab_search".to_string(), "搜尋".to_string());
        zh_tw.insert("tab_git".to_string(), "版本控制".to_string());
        translations.insert("zh_TW".to_string(), zh_tw);

        // 5. 중국어 간체 사전 (zh_CN)
        let mut zh_cn = HashMap::new();
        zh_cn.insert("badge_done".to_string(), "完成".to_string());
        zh_cn.insert("badge_pending".to_string(), "等待中".to_string());
        zh_cn.insert("badge_approval".to_string(), "需要批准".to_string());
        zh_cn.insert("btn_approve".to_string(), "批准 (Enter)".to_string());
        zh_cn.insert("btn_reject".to_string(), "拒绝 (Esc)".to_string());
        zh_cn.insert("questionnaire_title".to_string(), "📋 确认需求".to_string());
        zh_cn.insert("tab_preview".to_string(), "预览".to_string());
        zh_cn.insert("tab_diff".to_string(), "对比".to_string());
        zh_cn.insert("tab_logs".to_string(), "日志".to_string());
        zh_cn.insert("tab_search".to_string(), "搜索".to_string());
        zh_cn.insert("tab_git".to_string(), "版本控制".to_string());
        translations.insert("zh_CN".to_string(), zh_cn);

        Self {
            current_lang: lang.to_string(),
            translations,
        }
    }

    pub fn tr(&self, key: &str) -> &str {
        self.translations
            .get(&self.current_lang)
            .and_then(|dict| dict.get(key))
            .map(|s| s.as_str())
            .unwrap_or(key)
    }
}
```

### 4.2 TUI 그리드 아스키 실데이터 목업 (가로 80칼럼 타임라인)
격자 너비 `80` 칼럼 제한 환경에서 렌더링되어야 하는 실제 아스키 문자 맵을 정밀 픽셀로 구성한다.

```text
01234567890123456789012345678901234567890123456789012345678901234567890123456789 (80cols)
╭───────────────────────────────────────────────────────────── Timeline ───╮
│ 🏷️  042  [DONE]                                                          │
│ ┃  ❯ 1부터 100까지 더하는 파이썬 코드 작성                                 │
│ ┃  I will create a Python script to sum numbers from 1 to 100.           │
│ ┃                                                                        │
│ ┃  └─ ⚙️  WriteFile: sum_1_to_100.py (depth: 1)                           │
│ ┃     [DONE] sum_1_to_100.py 생성 완료                                    │
│                                                                          │
│ 🏷️  043  [NEEDS APPROVAL]                                                │
│ ⚠️  Modifying settings.rs to enable background workers.                   │
│ ⚠️  ╭── settings.rs ────────────────────────────────────────── +12 -4 ──╮  │
│ ⚠️  │ 42 | struct AppConfig {                                           │  │
│ ⚠️  │ 43 | -   workers: u32,                                            │  │
│ ⚠️  │ 44 | +   pub background_workers: u32,                             │  │
│ ⚠️  │ 45 | }                                                            │  │
│ ⚠️  ╰──────────────────────────────────────────────────────────────────╯  │
│ ⚠️  [ Approve (Enter) ]    [ Reject (Esc) ]                              │
╰──────────────────────────────────────────────────────────────────────────╯
```

---

## 5. 극도로 세분화된 구현 로드맵 (Micro Tasks Specification)

리팩토링의 안정성과 컴파일 무결성을 완전히 통제하기 위해, 4개 Phase를 총 **12개의 마이크로 구현 태스크**로 더 구체적이고 정밀하게 쪼개어 단계별 정적 분석 검사(Clippy) 및 렌더러 동작 확인을 거치며 빌드하도록 정의한다.

### 5.1 Phase 1: Palette & i18n 기반 인프라 개편

#### [Task 1.1] Palette & Theme 리모델링 (palette.rs)
* **대상 소스 파일:** `src/tui/palette.rs`
* **수정 로직 및 수치:**
  - `Palette` 구조체에 `bg_lowest: ratatui::style::Color` 및 `outline: ratatui::style::Color` 필드를 명시적으로 추가한다.
  - `DEFAULT_PALETTE`를 [2.1절 컬러 명세](file:///mnt/Projects_SSD/rust/smlcli/redesign_plan.md#2.1-컬러-시스템-명세-(semantic-palette))에 정의된 11개의 RGB 스케일 값으로 전면 업데이트한다.
  - 고대비 모드인 `HIGH_CONTRAST_PALETTE`에도 가시성 확보를 위해 `bg_lowest = Color::Black`, `outline = Color::White` 필드값을 추가 매핑한다.
* **주석 규정:** `DEFAULT_PALETTE`가 정의된 파일 시작 부분에 **한국어**로 "Terminal Precision 테마의 딥 네이비(#051424) 및 Inset Composer의 블랙 딥 네이비(#010f1f) 색상 깊이(Depth) 렌더링 토큰 매핑 가이드"를 서술적으로 상세히 기록한다.

#### [Task 1.2] 다국어(i18n.rs) 매니저 구현 (i18n.rs)
* **대상 소스 파일:** `src/tui/i18n.rs` [NEW]
* **수정 로직 및 수치:**
  - `I18nManager` 구조체를 생성하고, 다국어 사전을 `HashMap<String, HashMap<String, String>>` 형태로 메모리에 탑재한다.
  - 지원하는 5개 언어(`ko`, `en`, `ja`, `zh_TW`, `zh_CN`)의 정적 메시지 칩(완료, 대기 중, 승인 필요, 미리보기, 디프 등)을 100% 매핑하여 수록한다.
  - 사전 키가 존재하지 않거나, 등록되지 않은 언어인 경우 기본 `en` 메시지를 Fallback하여 반환하는 `tr(&self, key: &str) -> &str` 함수를 설계한다.
* **주석 규정:** 다국어 맵의 키 조회 복잡도가 $O(1)$로 보장되도록 `HashMap`을 사용한 설계 근거 및 Fallback 흐름을 한국어 주석으로 기재한다.

#### [Task 1.3] i18n 환경설정 동적 연동 (app/mod.rs, app/config.rs)
* **대상 소스 파일:** `src/app/mod.rs`, `src/app/config.rs`
* **수정 로직 및 수치:**
  - 설정 파일 `smlcli.toml` 또는 시스템 환경 변수 `LANG`, `LC_ALL` 등으로부터 현재 언어 환경을 파싱하여 `AppState.i18n` 인스턴스를 초기화하는 파이프라인을 연동한다.
  - 환경 변수에서 감지된 언어가 5개 지원 언어 목록에 없으면 `"en"`을 디폴트로 주입한다.

---

### 5.2 Phase 2: Timeline Block-based Layout 고도화

#### [Task 2.1] Timeline Block 구조체 컬렉션 마이그레이션 (app/state.rs)
* **대상 소스 파일:** `src/app/state.rs`
* **수정 로직 및 수치:**
  - [3.1절 타입 정의](file:///mnt/Projects_SSD/rust/smlcli/redesign_plan.md#3.1-턴(turn)-상태-및-표시-모드)에 기술된 `BlockStatus`, `BlockDisplayMode`, `FocusedPane` 열거형 타입을 작성한다.
  - 기존의 단순 문자열 기반 히스토리 벡터 `Vec<String>`를 구조화된 `Vec<TimelineBlock>` 형태로 교체 마이그레이션한다.
  - `TimelineBlock`은 고유 `id: String`, `kind: TimelineBlockKind`, `status: BlockStatus`, `display_mode: BlockDisplayMode`, `timestamp: u64`를 포함한다.

#### [Task 2.2] Borders::LEFT 및 unicode-width 정렬 렌더링 (tui/layout.rs)
* **대상 소스 파일:** `src/tui/layout.rs` 내 `draw_timeline`
* **수정 로직 및 수치:**
  - 타임라인 내 개별 `TimelineBlock`을 그릴 때, `Block::default().borders(Borders::LEFT)`를 설정하고, `BlockStatus`에 따라 지정된 테마 컬러(`accent` 퍼플, `warning` 오렌지, `danger` 라이트레드 등)를 적용한다.
  - `unicode-width` 크레이트의 `UnicodeWidthStr::width` 함수를 직접 호출하여, 한글/일어/중국어 등의 광폭 문자(Full-width, 2셀 점유)가 섞여 있어도 좌측 바 라인으로부터 정확히 `1` 셀의 여백을 띄우고 정렬되도록 렌더링 좌표 계산식을 보정한다.

#### [Task 2.3] Tree of Thoughts 깊이(depth) 시각화 융합 (tui/layout.rs)
* **대상 소스 파일:** `src/tui/layout.rs`
* **수정 로직 및 수치:**
  - 에이전트의 내부 도구 호출(Tool Execution), 쉘 실행 등 `depth: 1` 이상의 하위 작업 카드를 드로잉할 때, `depth` 변수 깊이 1당 정확히 `4` 칸의 스페이스 들여쓰기를 삽입한다.
  - 들여쓰기 후 `└─ ⚙️` 또는 `├─ ⚙️` 트리 아스키 브랜치 기호를 출력하며, 이 트리 기호의 글자색을 `DEFAULT_PALETTE.outline` 회색으로 칠하여 시각적 간섭을 제한한다.

---

### 5.3 Phase 3: Multi-viewport Layout & Inspector Tabs 개선

#### [Task 3.1] Inspector Tab UI 설계 및 단축키 바인딩 (tui/widgets/inspector_tabs.rs)
* **대상 소스 파일:** `src/tui/widgets/inspector_tabs.rs`
* **수정 로직 및 수치:**
  - 상단 탭 헤더 영역에 `[▶ PREVIEW ◀] [  DIFF  ] [  LOGS  ] [  SEARCH  ] [  GIT  ]` 형태의 5대 탭바를 생성한다.
  - 포커스 상태인 탭은 보라색 배경(`DEFAULT_PALETTE.accent`)에 검은색 볼드 글씨로 처리하고, 선택되지 않은 일반 탭은 `bg_panel` 배경에 `outline` 회색 글씨로 감춘다.
  - TUI 입력 이벤트 루프(`src/tui/mod.rs` 또는 `src/tui/event.rs`)에서 `Alt+1` ~ `Alt+5` 입력 감지 시, `TuiAction::SwitchTab(InspectorTab)`을 발송하여 실시간으로 탭 화면을 동적 전환한다.

#### [Task 3.2] HashMap 기반 Diff 라인 캐싱 가속기 구현 (tui/widgets/inspector_tabs.rs)
* **대상 소스 파일:** `src/tui/widgets/inspector_tabs.rs`
* **수정 로직 및 수치:**
  - `DiffLineCache` 구조체와 전역/로컬 캐시 인스턴스를 구축한다.
  - 5000라인급 대용량 Diff 화면 전환 시 렌더링 지연을 제거하기 위해, 파싱된 `Vec<Line<'static>>`를 `HashMap<PathBuf, Vec<Line<'static>>>`에 버퍼 캐싱한다.
  - 파일의 메타데이터 수정 시각(`modified_time`)을 조회하여 마지막 캐싱 타임스탬프보다 최신일 때만 `similar` 디프 분석 및 파싱을 재수행하고, 그렇지 않으면 캐시 적중(Cache Hit) 상태로 즉시 파싱 없이 드로잉한다.

#### [Task 3.3] Composer Inset 레이아웃 및 █ 커서 연동 (tui/layout.rs)
* **대상 소스 파일:** `src/tui/layout.rs`
* **수정 로직 및 수치:**
  - Composer 입력 영역의 배경을 `DEFAULT_PALETTE.bg_lowest`(#010f1f) 색상으로 꽉 찬 단색 픽셀 영역으로 드로잉하여 입력창의 깊이감을 입체화한다.
  - 프롬프트 `❯` 기호를 보라색(`accent`)으로 강조 출력하며, 텍스트 버퍼 입력 포커스가 켜져 있을 때 버퍼 맨 끝에 문자 `█`가 500ms 주기로 점멸하여 깜빡이는 TUI 터미널 네이티브 커서 루틴을 수식으로 구현한다.

---

### 5.4 Phase 4: Floating Modals & Command Palette 완성

#### [Task 4.1] Questionnaire 플로팅 모달 수학적 정렬 (tui/widgets/questionnaire.rs)
* **대상 소스 파일:** `src/tui/widgets/questionnaire.rs`
* **수정 로직 및 수치:**
  - 화면의 전체 가로폭 `W`와 세로폭 `H`를 매 프레임 파싱하여 플로팅 설문 모달의 `Rect`를 다음 정밀 정수식으로 도출한다.
    * `width = W * 60 / 100` (최소 50, 최대 90 클램프)
    * `height = H * 45 / 100` (최소 15, 최대 25 클램프)
    * `x = (W - width) / 2`
    * `y = (H - height) / 2`
  - Ratatui의 `Clear` 위젯을 먼저 드로잉하여 모달 배후의 타임라인 텍스트를 깨끗하게 지우고(Punch through), 모달 외곽 테두리를 보라색(`accent`) 둥근 유니코드(`Rounded`)로 설정한다.
  - 방향키(`Up`/`Down`) 입력에 따른 설문 옵션 포커스 변경과 다국어 텍스트 매핑 처리를 탑재한다.

#### [Task 4.2] Ctrl+K Fuzzy Command Palette 오버레이 (tui/layout.rs)
* **대상 소스 파일:** `src/tui/layout.rs` 내 `draw_command_palette` [NEW]
* **수정 로직 및 수치:**
  - 유저가 언제든 `Ctrl+K` 단축키를 입력하면 화면 전체에 불투명 어두운 회색 장막을 드리우고, 정중앙에 가로 `50%`, 세로 `30%` 크기의 커맨드 팔레트 검색창을 생성한다.
  - 검색창 최상단에 `🔍 ` 돋보기 기호와 함께 실시간 입력 중인 텍스트 버퍼가 렌더링된다.
  - 입력 문자열과 매칭되는 TUI 액션 리스트(예: "Theme Switch", "Toggle Inspector", "Change Language to English", "Check git status" 등) 10개 이상에 대해 간단한 문자열 포함 감지(Sub-string matching) fuzzy 필터링을 수행하여 실시간으로 후보군을 아래에 리스트로 노출하고, `Enter`를 누르면 즉시 해당 액션을 라우팅 실행한다.

---

## 6. spec.md 및 designs.md 문서 동기화 계획

D3D Protocol의 핵심 규칙인 **"문서 정합성(SPEC_IS_LAW)"**에 입각하여, 디자인 및 아키텍처 구조의 대대적인 변화를 하위 문서에 실시간 동기화하기 위한 갱신 체크리스트를 실행한다.

### 6.1 `spec.md` 동기화 Checklist
* [x] **Phase 15 UX 현대화 세부 스펙 갱신:** 본 `redesign_plan.md`에서 확정 및 동결한 `DEFAULT_PALETTE` 및 `BlockDisplayMode` 등의 타입 정보를 `spec.md` 내에 완전하게 이식하여 합친다.
* [x] **Tree of Thoughts UX 조화:** `spec.md` 966라인의 트리 설계와 좌측 세로줄 렌더러의 정합성이 완벽히 동기화되도록 명세 내용을 업데이트한다.

### 6.2 `designs.md` 동기화 Checklist
* [x] **4절 상단바 레이아웃 구조도 갱신:** 80자 및 100자 가로폭에 대응하는 반응형 상단바 구조의 아스키 아트를 반영한다.
* [x] **5절 타임라인 턴 블록 갱신:** `designs.md` 내에 기재된 타임라인 아스키 가이드를 `Borders::LEFT` 2px 및 배지 칩 조합 구조의 현대적 디자인으로 전면 리프레시한다.
* [x] **6절 인스펙터 탭바 구조 갱신:** `PREVIEW`, `DIFF`, `LOGS`, `SEARCH`, `GIT` 다중 탭 아키텍처 및 탭 스위칭 수치를 designs.md에 업데이트한다.

---

## 7. 검증 계획 및 명령어 (Verification Path)

### 7.1 컴파일 및 정적 Clippy 검증
각 태스크가 끝날 때마다 컴파일 에러 및 코드 완성도를 정밀 감사한다.
```bash
# 1. UTF-8 인코딩 손실 여부 확인 (CP949 등 깨짐 원천 방지)
iconv -f UTF-8 -t UTF-8 src/tui/palette.rs > /dev/null

# 2. Cargo Clippy 정적 경고 감사 실행 (모든 warnings 컴파일 에러로 승격)
cargo clippy --all-targets -- -D warnings

# 3. 프로젝트 전체 정상 컴파일 빌드 확인
cargo build --release
```

### 7.2 런타임 엣지 케이스 수동 검증 시나리오

#### 시나리오 A: 터미널 창 동적 리사이즈 및 붕괴 가드
1. `smlcli` TUI를 구동한 후 터미널 크기를 가로 `120` 칼럼 상태에서 서서히 마우스로 드래그하여 축소한다.
2. 가로 `100` 칼럼이 되는 순간 우측 인스펙터가 화면 점유를 멈추고 닫히는지 확인하며, `F2` 키 입력 시 `35` 칼럼 너비의 Drawer 형태로 화면 상단에 떠올라 렌더링되는지 육안 확인한다.
3. 가로 `80` 칼럼 미만 시 상단바의 CWD 및 Policy 칩이 깨짐이나 개행 충돌 없이 즉각 안전 축약되는지 검증한다.

#### 시나리오 B: Tree of Thoughts 깊이 융합 및 턴 배지 검증
1. 에이전트에게 파일 읽기 및 쓰기 명령을 결합하여 호출한다.
2. 타임라인에 그려지는 메인 턴의 좌측 세로선이 `Palette.accent` 보라색(`┃`)으로 출력되는지 확인한다.
3. 하위 도구 실행이 일어날 때 `└─ ⚙️` 트리선이 `depth: 1` 상태로 정확하게 `4`칸 공백 들여쓰기된 채 깔끔한 하늘색으로 분기 렌더링되는지 확인한다.

#### 시나리오 C: 📋 요구사항 확인 모달 다국어 스위칭
1. `ko` 환경에서 Questionnaire 모달에 진입하여 타이틀이 `📋 요구사항 확인`으로 출력되고, `React` 옵션이 보라색 배경으로 포커스되는지 확인한다.
2. 인프라 언어 설정을 `en`으로 임시 스위칭하여 타이틀이 `📋 Questionnaire`로 잘 나타나는지 확인하고, 포커스가 변경될 때 겹침이나 잘림 에러가 없는지 픽셀 단위로 대조 감사를 마친다.
