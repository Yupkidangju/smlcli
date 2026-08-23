// [v3.9.0] Phase 1-B: 다국어(i18n.rs) 매니저 모듈.
// TUI 렌더링에 사용되는 정적 메시지 사전 데이터의 구조와 5대 언어(ko, en, ja, zh_TW, zh_CN) 지원을 정의한다.
// 모든 텍스트 사전은 메모리상의 HashMap에 탑재되며 tr() 함수를 통해 동적으로 로컬라이징된 문자열을 반환한다.
// designs.md 및 redesign_plan.md §4.1 사양 반영.

use std::collections::HashMap;

pub fn normalize_locale(lang: &str) -> &'static str {
    let base = lang
        .split(['.', '@'])
        .next()
        .unwrap_or(lang)
        .replace('-', "_")
        .to_ascii_lowercase();
    match base.as_str() {
        "ko" | "ko_kr" => "ko",
        "ja" | "ja_jp" => "ja",
        "zh_tw" | "zh_hant" | "zh_hant_tw" => "zh_TW",
        "zh_cn" | "zh_hans" | "zh_hans_cn" => "zh_CN",
        _ => "en",
    }
}

pub fn preferred_language_from_env() -> String {
    let locale = std::env::var("LC_ALL")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| std::env::var("LANG").ok())
        .unwrap_or_else(|| "en".to_string());
    normalize_locale(&locale).to_string()
}

fn extend_common(dict: &mut HashMap<String, String>, lang: &str) {
    let pairs: &[(&str, &str)] = match lang {
        "ko" => &[
            ("config_title", "설정"),
            ("config_dashboard", "기본 설정 대시보드"),
            ("label_provider", "공급자"),
            ("label_model", "모델"),
            ("label_shell_policy", "셸 정책"),
            ("label_network_policy", "네트워크 정책"),
            ("label_sandbox", "샌드박스"),
            ("enabled", "활성"),
            ("disabled", "비활성"),
            ("none", "없음"),
            ("navigate_hint", "위/아래: 이동 | Enter: 변경 | Esc: 닫기"),
            ("select_provider", "공급자 선택"),
            ("select_model", "모델 선택"),
            ("loading_models", "모델 불러오는 중..."),
            ("error_loading_models", "모델 로드 오류"),
            ("wizard_title", "설정 마법사"),
            ("wizard_step_provider", "[1단계] 공급자 선택"),
            ("wizard_step_base_url", "[2단계] Base URL 입력"),
            ("wizard_step_api_key", "[2단계] API 키 입력"),
            ("wizard_step_model", "[3단계] 모델 선택"),
            ("wizard_step_saving", "설정 저장"),
            ("please_wait", "잠시 기다려 주세요."),
            ("save_error", "저장 오류"),
            (
                "ready_save",
                "설정을 저장할 준비가 되었습니다. Enter를 눌러 시작하세요.",
            ),
            ("api_key_required", "API 키가 필요합니다."),
            ("base_url_required", "Base URL이 필요합니다."),
            ("model_required", "모델 이름이 필요합니다."),
            ("help_title", "키보드 단축키"),
            ("no_active_blocks", "활성 블록이 없습니다."),
            ("timeline_empty", "타임라인이 비어 있습니다."),
            ("no_pending_diffs", "대기 중인 diff가 없습니다."),
            ("no_logs", "이 세션에 기록된 로그가 없습니다."),
            ("terminal_too_small", "터미널 크기가 너무 작습니다."),
        ],
        "ja" => &[
            ("config_title", "設定"),
            ("config_dashboard", "基本設定ダッシュボード"),
            ("label_provider", "プロバイダー"),
            ("label_model", "モデル"),
            ("label_shell_policy", "シェルポリシー"),
            ("label_network_policy", "ネットワークポリシー"),
            ("label_sandbox", "サンドボックス"),
            ("enabled", "有効"),
            ("disabled", "無効"),
            ("none", "なし"),
            ("navigate_hint", "上下: 移動 | Enter: 変更 | Esc: 閉じる"),
            ("select_provider", "プロバイダーを選択"),
            ("select_model", "モデルを選択"),
            ("loading_models", "モデルを読み込み中..."),
            ("error_loading_models", "モデル読込エラー"),
            ("wizard_title", "セットアップウィザード"),
            ("wizard_step_provider", "[手順1] プロバイダー選択"),
            ("wizard_step_base_url", "[手順2] Base URL入力"),
            ("wizard_step_api_key", "[手順2] APIキー入力"),
            ("wizard_step_model", "[手順3] モデル選択"),
            ("wizard_step_saving", "設定を保存"),
            ("please_wait", "お待ちください。"),
            ("save_error", "保存エラー"),
            ("ready_save", "保存準備完了。Enterで開始します。"),
            ("api_key_required", "APIキーが必要です。"),
            ("base_url_required", "Base URLが必要です。"),
            ("model_required", "モデル名が必要です。"),
            ("help_title", "キーボードショートカット"),
            ("no_active_blocks", "アクティブなブロックはありません。"),
            ("timeline_empty", "タイムラインは空です。"),
            ("no_pending_diffs", "保留中の差分はありません。"),
            ("no_logs", "このセッションにログはありません。"),
            ("terminal_too_small", "ターミナルが小さすぎます。"),
        ],
        "zh_TW" => &[
            ("config_title", "設定"),
            ("config_dashboard", "主要設定面板"),
            ("label_provider", "供應商"),
            ("label_model", "模型"),
            ("label_shell_policy", "Shell 政策"),
            ("label_network_policy", "網路政策"),
            ("label_sandbox", "沙箱"),
            ("enabled", "啟用"),
            ("disabled", "停用"),
            ("none", "無"),
            ("navigate_hint", "上下：移動 | Enter：變更 | Esc：關閉"),
            ("select_provider", "選擇供應商"),
            ("select_model", "選擇模型"),
            ("loading_models", "正在載入模型..."),
            ("error_loading_models", "模型載入錯誤"),
            ("wizard_title", "設定精靈"),
            ("wizard_step_provider", "[步驟 1] 選擇供應商"),
            ("wizard_step_base_url", "[步驟 2] 輸入 Base URL"),
            ("wizard_step_api_key", "[步驟 2] 輸入 API 金鑰"),
            ("wizard_step_model", "[步驟 3] 選擇模型"),
            ("wizard_step_saving", "儲存設定"),
            ("please_wait", "請稍候。"),
            ("save_error", "儲存錯誤"),
            ("ready_save", "已準備儲存。按 Enter 開始。"),
            ("api_key_required", "需要 API 金鑰。"),
            ("base_url_required", "需要 Base URL。"),
            ("model_required", "需要模型名稱。"),
            ("help_title", "鍵盤快速鍵"),
            ("no_active_blocks", "沒有作用中的區塊。"),
            ("timeline_empty", "時間軸是空的。"),
            ("no_pending_diffs", "沒有待處理的差異。"),
            ("no_logs", "此工作階段沒有日誌。"),
            ("terminal_too_small", "終端機太小。"),
        ],
        "zh_CN" => &[
            ("config_title", "设置"),
            ("config_dashboard", "主要设置面板"),
            ("label_provider", "提供商"),
            ("label_model", "模型"),
            ("label_shell_policy", "Shell 策略"),
            ("label_network_policy", "网络策略"),
            ("label_sandbox", "沙箱"),
            ("enabled", "启用"),
            ("disabled", "禁用"),
            ("none", "无"),
            ("navigate_hint", "上下：移动 | Enter：更改 | Esc：关闭"),
            ("select_provider", "选择提供商"),
            ("select_model", "选择模型"),
            ("loading_models", "正在加载模型..."),
            ("error_loading_models", "模型加载错误"),
            ("wizard_title", "设置向导"),
            ("wizard_step_provider", "[步骤 1] 选择提供商"),
            ("wizard_step_base_url", "[步骤 2] 输入 Base URL"),
            ("wizard_step_api_key", "[步骤 2] 输入 API 密钥"),
            ("wizard_step_model", "[步骤 3] 选择模型"),
            ("wizard_step_saving", "保存设置"),
            ("please_wait", "请稍候。"),
            ("save_error", "保存错误"),
            ("ready_save", "已准备保存。按 Enter 开始。"),
            ("api_key_required", "需要 API 密钥。"),
            ("base_url_required", "需要 Base URL。"),
            ("model_required", "需要模型名称。"),
            ("help_title", "键盘快捷键"),
            ("no_active_blocks", "没有活动区块。"),
            ("timeline_empty", "时间轴为空。"),
            ("no_pending_diffs", "没有待处理的差异。"),
            ("no_logs", "此会话没有日志。"),
            ("terminal_too_small", "终端太小。"),
        ],
        _ => &[
            ("config_title", "Configuration"),
            ("config_dashboard", "Master Settings Dashboard"),
            ("label_provider", "Provider"),
            ("label_model", "Model"),
            ("label_shell_policy", "Shell Policy"),
            ("label_network_policy", "Network Policy"),
            ("label_sandbox", "Sandbox"),
            ("enabled", "Enabled"),
            ("disabled", "Disabled"),
            ("none", "None"),
            (
                "navigate_hint",
                "Up/Down: navigate | Enter: change | Esc: close",
            ),
            ("select_provider", "Select Provider"),
            ("select_model", "Select Model"),
            ("loading_models", "Loading models..."),
            ("error_loading_models", "Error loading models"),
            ("wizard_title", "Setup Wizard"),
            ("wizard_step_provider", "[Step 1] Select Provider"),
            ("wizard_step_base_url", "[Step 2] Enter Base URL"),
            ("wizard_step_api_key", "[Step 2] Enter API Key"),
            ("wizard_step_model", "[Step 3] Select Model"),
            ("wizard_step_saving", "Save Configuration"),
            ("please_wait", "Please wait."),
            ("save_error", "Save Error"),
            ("ready_save", "Ready to save. Press Enter to start."),
            ("api_key_required", "API Key is required."),
            ("base_url_required", "Base URL is required."),
            ("model_required", "Model name is required."),
            ("help_title", "Keyboard Shortcuts"),
            ("no_active_blocks", "No active blocks."),
            ("timeline_empty", "Timeline is empty."),
            ("no_pending_diffs", "No pending diffs."),
            ("no_logs", "No logs recorded in this session."),
            ("terminal_too_small", "Terminal is too small."),
        ],
    };
    dict.extend(
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string())),
    );
}

/// 다국어 번역 리소스 매니저 구조체.
/// 현재 언어와 언어별 정적 메시지 딕셔너리를 포함한다.
#[derive(Debug, Clone)]
pub struct I18nManager {
    /// 현재 언어 코드 ("ko", "en", "ja", "zh_TW", "zh_CN")
    pub current_lang: String,
    /// 다국어 사전 리소스 맵
    translations: HashMap<String, HashMap<String, String>>,
}

impl I18nManager {
    /// 새 I18nManager 인스턴스를 초기화하고 5대 지원 언어 리소스를 적재한다.
    pub fn new(lang: &str) -> Self {
        let mut translations = HashMap::new();

        // 1. 한국어 사전 (ko)
        let mut ko = HashMap::new();
        ko.insert("badge_done".to_string(), "완료".to_string());
        ko.insert("badge_pending".to_string(), "대기 중".to_string());
        ko.insert("badge_approval".to_string(), "승인 필요".to_string());
        ko.insert("btn_approve".to_string(), "승인 (Enter)".to_string());
        ko.insert("btn_reject".to_string(), "반려 (Esc)".to_string());
        ko.insert(
            "questionnaire_title".to_string(),
            "📋 요구사항 확인".to_string(),
        );
        ko.insert("tab_preview".to_string(), "미리보기".to_string());
        ko.insert("tab_diff".to_string(), "디프".to_string());
        ko.insert("tab_logs".to_string(), "로그".to_string());
        ko.insert("tab_search".to_string(), "검색".to_string());
        ko.insert("tab_recent".to_string(), "최근".to_string());
        ko.insert("tab_git".to_string(), "깃".to_string());
        // [v3.9.0] 설문조사 위젯 전용 한국어 번역 리소스 추가
        ko.insert("question_progress".to_string(), "질문 {}/{}".to_string());
        ko.insert("input_prompt".to_string(), "답변 입력:".to_string());
        ko.insert("custom_input_prompt".to_string(), "직접 입력:".to_string());
        ko.insert(
            "input_hint".to_string(),
            "Enter: 제출  |  Esc: 취소".to_string(),
        );
        ko.insert(
            "select_hint".to_string(),
            "↑↓: 이동  |  Enter: 선택  |  Esc: 취소".to_string(),
        );
        ko.insert("custom_option".to_string(), "✏ 직접 입력...".to_string());
        extend_common(&mut ko, "ko");
        translations.insert("ko".to_string(), ko);

        // 2. 영어 사전 (en)
        let mut en = HashMap::new();
        en.insert("badge_done".to_string(), "DONE".to_string());
        en.insert("badge_pending".to_string(), "PENDING".to_string());
        en.insert("badge_approval".to_string(), "NEEDS APPROVAL".to_string());
        en.insert("btn_approve".to_string(), "Approve (Enter)".to_string());
        en.insert("btn_reject".to_string(), "Reject (Esc)".to_string());
        en.insert(
            "questionnaire_title".to_string(),
            "📋 Questionnaire".to_string(),
        );
        en.insert("tab_preview".to_string(), "PREVIEW".to_string());
        en.insert("tab_diff".to_string(), "DIFF".to_string());
        en.insert("tab_logs".to_string(), "LOGS".to_string());
        en.insert("tab_search".to_string(), "SEARCH".to_string());
        en.insert("tab_recent".to_string(), "RECENT".to_string());
        en.insert("tab_git".to_string(), "GIT".to_string());
        // [v3.9.0] 설문조사 위젯 전용 영어 번역 리소스 추가
        en.insert(
            "question_progress".to_string(),
            "Question {}/{}".to_string(),
        );
        en.insert("input_prompt".to_string(), "Your Answer:".to_string());
        en.insert(
            "custom_input_prompt".to_string(),
            "Direct Input:".to_string(),
        );
        en.insert(
            "input_hint".to_string(),
            "Enter: Submit  |  Esc: Cancel".to_string(),
        );
        en.insert(
            "select_hint".to_string(),
            "↑↓: Move  |  Enter: Select  |  Esc: Cancel".to_string(),
        );
        en.insert("custom_option".to_string(), "✏ Direct Input...".to_string());
        extend_common(&mut en, "en");
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
        ja.insert("tab_recent".to_string(), "最近".to_string());
        ja.insert("tab_git".to_string(), "Git".to_string());
        // [v3.9.0] 설문조사 위젯 전용 일본어 번역 리소스 추가
        ja.insert("question_progress".to_string(), "質問 {}/{}".to_string());
        ja.insert("input_prompt".to_string(), "回答入力:".to_string());
        ja.insert("custom_input_prompt".to_string(), "直接入力:".to_string());
        ja.insert(
            "input_hint".to_string(),
            "Enter: 送信  |  Esc: キャンセル".to_string(),
        );
        ja.insert(
            "select_hint".to_string(),
            "↑↓: 移動  |  Enter: 選択  |  Esc: キャンセル".to_string(),
        );
        ja.insert("custom_option".to_string(), "✏ 直接入力...".to_string());
        extend_common(&mut ja, "ja");
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
        zh_tw.insert("tab_recent".to_string(), "最近".to_string());
        zh_tw.insert("tab_git".to_string(), "版本控制".to_string());
        // [v3.9.0] 설문조사 위젯 전용 중국어 번체 번역 리소스 추가
        zh_tw.insert("question_progress".to_string(), "問題 {}/{}".to_string());
        zh_tw.insert("input_prompt".to_string(), "輸入回答:".to_string());
        zh_tw.insert("custom_input_prompt".to_string(), "直接輸入:".to_string());
        zh_tw.insert(
            "input_hint".to_string(),
            "Enter: 提交  |  Esc: 取消".to_string(),
        );
        zh_tw.insert(
            "select_hint".to_string(),
            "↑↓: 移動  |  Enter: 選擇  |  Esc: 取消".to_string(),
        );
        zh_tw.insert("custom_option".to_string(), "✏ 直接輸入...".to_string());
        extend_common(&mut zh_tw, "zh_TW");
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
        zh_cn.insert("tab_recent".to_string(), "最近".to_string());
        zh_cn.insert("tab_git".to_string(), "版本控制".to_string());
        // [v3.9.0] 설문조사 위젯 전용 중국어 간체 번역 리소스 추가
        zh_cn.insert("question_progress".to_string(), "问题 {}/{}".to_string());
        zh_cn.insert("input_prompt".to_string(), "输入回答:".to_string());
        zh_cn.insert("custom_input_prompt".to_string(), "直接输入:".to_string());
        zh_cn.insert(
            "input_hint".to_string(),
            "Enter: 提交  |  Esc: 取消".to_string(),
        );
        zh_cn.insert(
            "select_hint".to_string(),
            "↑↓: 移动  |  Enter: 选择  |  Esc: 取消".to_string(),
        );
        zh_cn.insert("custom_option".to_string(), "✏ 直接输入...".to_string());
        extend_common(&mut zh_cn, "zh_CN");
        translations.insert("zh_CN".to_string(), zh_cn);

        // 환경 변수 파싱 등의 편의를 위해 소문자 또는 대소문자 변환 정규화를 지원한다.
        let normalized_lang = normalize_locale(lang);

        Self {
            current_lang: normalized_lang.to_string(),
            translations,
        }
    }

    /// 주어진 키에 해당하는 번역 문자열을 반환한다.
    /// [v3.9.0] 라이프타임 정합성을 위해 &'a self와 &'a str key를 바인딩하여 반환 값의 라이프타임을 일치시킴.
    /// clippy::collapsible_if 경고를 우회하기 위해 allow 속성 부여 (unstable let_chains 방지)
    /// 해당 언어 사전에 키가 존재하지 않으면, Fallback으로 영어 사전을 찾고,
    /// 영어 사전에도 없다면 원본 키 문자열을 그대로 반환한다.
    #[allow(clippy::collapsible_if)]
    pub fn tr<'a>(&'a self, key: &'a str) -> &'a str {
        if let Some(dict) = self.translations.get(&self.current_lang) {
            if let Some(val) = dict.get(key) {
                return val.as_str();
            }
        }
        // 영어(en) 사전에서 Fallback 검색
        if let Some(en_dict) = self.translations.get("en") {
            if let Some(val) = en_dict.get(key) {
                return val.as_str();
            }
        }
        key
    }

    #[cfg(test)]
    pub(crate) fn has_translation(&self, lang: &str, key: &str) -> bool {
        self.translations
            .get(normalize_locale(lang))
            .is_some_and(|dictionary| dictionary.contains_key(key))
    }
}
