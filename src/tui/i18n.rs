// [v3.9.0] Phase 1-B: 다국어(i18n.rs) 매니저 모듈.
// TUI 렌더링에 사용되는 정적 메시지 사전 데이터의 구조와 5대 언어(ko, en, ja, zh_TW, zh_CN) 지원을 정의한다.
// 모든 텍스트 사전은 메모리상의 HashMap에 탑재되며 tr() 함수를 통해 동적으로 로컬라이징된 문자열을 반환한다.
// designs.md 및 redesign_plan.md §4.1 사양 반영.

use std::collections::HashMap;

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
        translations.insert("en".to_string(), en);

        // 3. 일본어 사전 (ja)
        let mut ja = HashMap::new();
        ja.insert("badge_done".to_string(), "完了".to_string());
        ja.insert("badge_pending".to_string(), "待機중".to_string()); // 待機中
        ja.insert("badge_approval".to_string(), "承認が必要".to_string());
        ja.insert("btn_approve".to_string(), "承認 (Enter)".to_string());
        ja.insert("btn_reject".to_string(), "却下 (Esc)".to_string());
        ja.insert("questionnaire_title".to_string(), "📋 要件確認".to_string());
        ja.insert("tab_preview".to_string(), "プレビュー".to_string());
        ja.insert("tab_diff".to_string(), "差分".to_string());
        ja.insert("tab_logs".to_string(), "ログ".to_string());
        ja.insert("tab_search".to_string(), "検索".to_string());
        ja.insert("tab_recent".to_string(), "最近".to_string());
        ja.insert("tab_git".to_string(), "ギット".to_string());
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
        translations.insert("zh_CN".to_string(), zh_cn);

        // 환경 변수 파싱 등의 편의를 위해 소문자 또는 대소문자 변환 정규화를 지원한다.
        let normalized_lang = match lang.to_lowercase().as_str() {
            "ko" | "ko_kr" => "ko",
            "ja" | "ja_jp" => "ja",
            "zh_tw" | "zh-tw" | "zh-hant" => "zh_TW",
            "zh_cn" | "zh-cn" | "zh-hans" => "zh_CN",
            _ => "en", // 기본값은 영어
        };

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
}
