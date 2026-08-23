# smlcli

터미널 중심 AI 에이전트 CLI 도구 (Terminal-native AI Agent CLI)

[한국어](#한국어) | [English](#english) | [日本語](#日本語) | [繁體中文](#繁體中文) | [简体中文](#简体中文)

---

## 한국어

### 소개
`smlcli`는 Codex/OpenCode 계열의 사용감을 갖는 터미널 중심 AI 에이전트 CLI입니다. 앱 실행 시 TUI에 진입하며, `/setting`을 통해 공급자, API 키, 모델, 권한 정책을 설정합니다. 자연어 프롬프트와 `/` 명령어를 통해 파일 탐색, 코드 수정, 명령 실행 및 diff 검토를 지원합니다.

### 주요 기능
- **터미널 중심 TUI**: 마우스 없이 모든 동작을 3단계 이내에 키보드로 처리.
- **다중 공급자 지원**: OpenAI, Anthropic, xAI, OpenRouter, Google (Gemini) 및 LM Studio 로컬 프로바이더 지원 (API Key 스킵 및 Base URL 입력 분기).
- **보안 경계 및 검증**: 파일 쓰기·셸 실행 승인, API 키의 로컬 암호화 보관, workspace path 검증을 제공합니다. Linux `bwrap`는 실제 namespace capability가 있을 때만 사용하고 실패 시 실행을 거부합니다. 앱이 소유한 shell/MCP process group은 cancel/quit 시 정리하며, streaming redaction은 알려진 키의 노출 위험을 줄이지만 범용 secret detector는 아닙니다.
- **장애 경계 보강**: 설정·세션 저장 실패를 사용자에게 표시하고, API 요청 및 대용량 파일/로그에 timeout·size cap을 적용합니다. `unicode-width` 기반 터미널 렌더링과 `smlcli doctor` 진단을 제공합니다.
- **Inspect 패널과 Diff 플로우**: 작업 승인 전에 변경될 항목 가시성 확보.
- **지능형 컨텍스트 압축 및 성능 최적화**: 장기 세션 보호를 위한 백그라운드 LLM 요약기 및 토큰 한도 제어(`/tokens`), 디스크 캐시 기반 AST RepoMap 생성으로 대형 레포지토리에서의 속도 확보.
- **클립보드 연동**: `y` 키 단축키 및 시각적 토스트(Toast) 알림을 통한 즉각적인 클립보드 복사 지원.
- **환경 변수 제어**: `allowed_env_vars` 화이트리스트를 통한 도구 실행 환경 제어.
- **@ 로컬 데이터 참조**: `@` 퍼지 파인더로 canonical workspace 내부의 regular UTF-8 파일을 선택해, 파일별·턴별 제한 안에서 LLM 컨텍스트에 인라인 삽입.
- **실시간 테마 전환**: `/theme` 명령어로 Default ↔ HighContrast 테마를 즉시 전환. 설정 파일에 자동 저장.
- **Inspector Search 탭**: 타임라인 전체를 대소문자 무시로 실시간 검색 (최대 50건 표시).
- **SSE 스트리밍**: AI 응답을 토큰 단위로 실시간 표시 (OpenRouter/Gemini 대응).
- **JSONL 세션 로그**: 대화 내용을 `~/.smlcli/sessions/`에 자동 기록하여 세션 복원 지원.
- **에이전트 자율성 (Agentic Autonomy)**: 명시적 승인과 제한된 자가 복구 루프 아래 `ListDir`, `GrepSearch`, `FetchURL` 등의 도구를 제공합니다. 사용자 WIP 보호를 위해 앱의 자동 hard-reset과 자동 커밋 적용 경로는 비활성화되어 있습니다.
- **오프라인 수동 Fallback**: 로컬/오프라인 환경 및 API 핑 실패 시에도 마법사에서 `"✏ 직접 입력..."` 모드를 제공하여 모델명을 수동으로 임의 지정할 수 있어 연결 차단 없이 onboarding 완수 가능.
- **Tree-sitter Repo Map**: AST 파싱 기반 저장소 요약 맵을 통해 AI가 전체 프로젝트 구조를 맥락으로 주입받아 정확한 코드를 수정.
- **플랫폼 타깃**: 릴리스 워크플로는 Linux musl과 Windows MSVC를 대상으로 하며, 각 플랫폼 지원 판정은 해당 runner의 build/smoke 성공을 기준으로 합니다.
- **v3.9.0 TUI 현대화 개편**: 반응형 3분할 뷰포트, 인스펙터 `Tab`/`Shift+Tab` 전환, 점멸 커서와 공통 레이아웃/hit-test geometry를 제공합니다.
- **Workspace Harness 진단**: `smlcli doctor`, `/workspace show`, `/status`가 동일한 snapshot 기준으로 OS, Host Shell, Exec Shell, canonical workspace root, trust/deny 상태, Linux `/workspace` sandbox mount 정책을 표시합니다.
- **Workspace Harness 강제 적용**: 현재 OS/root/shell/sandbox/trust snapshot을 system prompt, 도구 실행 전 preflight, 세션 로그에 연결하여 환경 오해로 인한 잘못된 명령 실행을 줄입니다.

### 빠른 시작
1. 저장소를 클론합니다.
   ```bash
   git clone https://github.com/Yupkidangju/smlcli.git
   cd smlcli
   ```
2. 앱을 빌드하고 실행합니다. (`build.sh` 빌드 도구를 실행해 OS 호환성을 갖춘 릴리스 바이너리를 추출할 수도 있습니다.)
   ```bash
   ./build.sh
   # 혹은 바로 실행
   cargo run --release
   ```
3. 처음 실행하면 설정 마법사(`/setting`)가 자동으로 시작됩니다.

### 설정 및 권한
- `smlcli`는 파일 쓰기, 쉘 실행 등에 대해 PLAN과 RUN 모드를 제공하며, 설정 마법사에서 권한 정책(Safe Starter, Balanced, Strict)을 사전에 정의할 수 있습니다.

### 트러블슈팅 (Troubleshooting)
- **가상/헤드리스 터미널 크기**: 자동 테스트는 대표 breakpoint와 `(100, 30)` 가상 크기를 격리 검증합니다. 실제 터미널/에뮬레이터 차이는 물리 TTY에서 별도 확인할 수 있습니다.
- **Slash Command 하위 명령 입력**: `/` 입력 후 자동완성 메뉴가 떠도 입력 문자는 Composer에 그대로 남습니다. `/workspace ` 뒤에서는 `show`, `trust`, `deny`, `clear` 후보를 이어서 선택하거나 직접 타이핑한 뒤 Enter로 실행합니다.
- **Workspace Harness 확인**: Linux에서 sandbox가 활성화된 경우 `ExecShell` 내부 작업 경로는 `/workspace`로 표준화됩니다. `smlcli doctor`의 `Workspace Harness 상태` 섹션에서 실제 OS/셸/root/trust/sandbox 값을 확인하세요.
- **환경 mismatch 방지**: Linux/Windows 전용 셸 명령 혼동은 즉시 자동 실행하지 않고 Notice/Ask 경로로 승격됩니다.

---

## English

### Introduction
`smlcli` is a terminal-native AI agent CLI with Codex/OpenCode-like UX. It boots directly into a TUI where you configure providers, API keys, models, and permission policies via the `/setting` wizard. From there, you can interact using natural language or slash commands to explore files, edit code, execute commands, and review diffs.

### Key Features
- **Keyboard-first TUI**: Reach any primary action within 3 steps without a mouse.
- **Multi-provider**: Supports OpenAI, Anthropic, xAI, OpenRouter, Google (Gemini), and LM Studio local provider (skips API Key, supports Base URL customization).
- **Security boundaries**: Local encrypted API-key storage, explicit write/shell approvals, and canonical workspace path checks. Linux `bwrap` is used only when a real namespace capability probe succeeds; otherwise execution fails closed. Owned shell/MCP process groups are terminated on cancel/quit. Stateful redaction reduces known-key leakage risk and is covered by split-secret regression tests; it is not a universal secret detector.
- **Bounded failure handling**: Configuration/session write failures are surfaced to the user, and API calls plus large file/process output have timeout and size limits. Terminal rendering uses `unicode-width`; `smlcli doctor` reports the active environment.
- **Inspect & Diff Flows**: Write flows that require approval expose the proposed change before approval; policy-approved and direct command paths are reported separately.
- **Intelligent Context Compaction & Performance Optimization**: Background LLM summarizer for long-session stability, token limit control (`/tokens`), and AST RepoMap generation with disk caching via `cheap_hash` (mtime + file count) for faster performance in large repositories.
- **Clipboard Integration**: Instant clipboard copy support via the `y` hotkey with visual Toast notifications.
- **Environment Variable Control**: Fine-grained execution environment control via the `allowed_env_vars` whitelist.
- **@ Context Injection**: Type `@` to fuzzy-find workspace files and automatically inline their contents.
- **Real-time Theme Switching**: Swap between Default and HighContrast themes using `/theme` instantly. Automatically saved to configuration.
- **Inspector Search**: Perform case-insensitive, real-time searches across your timeline (up to 50 entries).
- **SSE Streaming**: See AI responses token-by-token in real-time (OpenRouter/Gemini compatible).
- **JSONL Session Logging**: Automatic session logs in `~/.smlcli/sessions/` for restoring prior conversations.
- **Agentic Autonomy**: Provides tools such as `ListDir`, `GrepSearch`, and `FetchURL` behind explicit policy checks and bounded recovery loops. Application-level automatic hard reset and auto-commit application are disabled to preserve user WIP.
- **Offline Manual Fallback**: Always provides `"✏ 직접 입력..."` (Manual Input) mode during setup even if the local server is offline or the models API ping fails, enabling seamless wizard completion.
- **Tree-sitter Repo Map**: Injects AST-parsed repository summary maps into the AI context for accurate code modifications.
- **Platform targets**: The release workflow targets Linux musl and Windows MSVC; support claims depend on successful build/smoke jobs for each runner.
- **v3.9.0 TUI Modernization**: Provides a responsive 3-viewport layout, focused `Tab`/`Shift+Tab` inspector navigation, a blinking cursor, and shared render/hit-test geometry.
- **Workspace Harness Diagnostics**: `smlcli doctor`, `/workspace show`, and `/status` share one snapshot for OS, host shell, exec shell, canonical workspace root, trust/deny state, and Linux `/workspace` sandbox mount policy.
- **Workspace Harness Enforcement**: The live OS/root/shell/sandbox/trust snapshot is connected to the system prompt, tool preflight, and session logs to reduce wrong-environment command execution.

### Quick Start
1. Clone the repository.
   ```bash
   git clone https://github.com/Yupkidangju/smlcli.git
   cd smlcli
   ```
2. Build and run (or use `./build.sh` for interactive cross-compilation target setups).
   ```bash
   ./build.sh
   # or
   cargo run --release
   ```

### Troubleshooting
- **Virtual/headless terminal dimensions**: Automated tests cover representative breakpoints and an isolated `(100, 30)` virtual size. Emulator-specific behavior can still require a physical-TTY check.
- **Slash Command subcommands**: When `/` opens autocomplete, typed text stays in the Composer. After `/workspace `, choose or type `show`, `trust`, `deny`, or `clear`, then press Enter to run the completed command.
- **Workspace Harness checks**: When Linux sandboxing is enabled, the `ExecShell` guest working directory is standardized as `/workspace`. Check the `Workspace Harness` section in `smlcli doctor` for the live OS/shell/root/trust/sandbox values.
- **Environment mismatch guard**: Linux/Windows-specific shell command confusion is promoted to Notice/Ask instead of immediate automatic execution.

---

## 日本語

### 概要
`smlcli` は、Codex/OpenCode のような使用感を持つ、ターミナル中心の AI エージェント CLI ツールです。実行すると TUI が起動し、`/setting` ウィザードを通じてプロバイダー、API キー、モデル、権限ポリシーを設定します。自然言語によるプロンプト操作でファイル操作やコマンド実行が可能です。

### 主な機能
- **ターミナルファースト TUI**: 全ての操作をキーボードだけで迅速に行えます。
- **マルチプロバイダー対応**: OpenAI, Anthropic, xAI, OpenRouter, Google (Gemini) および LM Studio ローカルプロバイダーをサポート（APIキーのスキップ、Base URLのカスタム入力を提供）。
- **セキュリティ境界**: APIキーのローカル暗号化保存、書込み／Shell承認、canonical workspace path検証を提供します。Linux `bwrap` は namespace capability probe 成功時のみ使用し、失敗時は実行を拒否します。所有する process group は cancel/終了時に回収します。既知キーの streaming redaction は漏えいリスクを下げますが、汎用の secret detector ではありません。
- **極限環境での安定性**: ディスク容量不足 (`ENOSPC`) 時のパニック防止、LLM APIタイムアウト時の指数バックオフ再試行、`unicode-width` ベースの安全なUTF-8レンダリング、大規模出力時のメモリキャッピング (OOM防止)、およびOSCシーケンスを用いたターミナルタイトル/タスクバー進捗状況の同期。
- **インテリジェント コンテキスト圧縮**: 長期セッション保護のためのバックグラウンド LLM 要約と `/tokens` トークン監視。
- **@ ローカルデータ参照**: canonical workspace 内の regular UTF-8 ファイルを選択し、ファイル／ターンごとの上限内でAIコンテキストに挿入。
- **リアルタイム テーマ切替**: `/theme` コマンドで Default ↔ HighContrast テーマを即座に切り替え可能。
- **Inspector 検索タブ**: タイムライン全体をリアルタイムで全文検索（最大50件表示）。
- **SSE ストリーミング**: AIの回答をトークン単位でリアルタイム表示。
- **JSONL セッションログ**: 会話を自動記録し、セッション復元をサポート。
- **エージェント自律性 (Agentic Autonomy)**: 明示的なポリシー確認と制限付き回復ループの下で `ListDir`, `GrepSearch`, `FetchURL` などを提供します。ユーザーWIP保護のため、アプリによる自動 hard reset／auto-commit 適用は無効です。
- **オフラインでの手動フォールバック**: ローカル/オフライン環境やAPIの疎通確認（PING）が失敗した場合でも、設定ウィザードで「✏ 直接入力...」モードを提供し、モデル名を手動で任意に入力して接続を中断することなくオンボーディングを完了できます。
- **Tree-sitter Repo Map**: AST解析ベースのリポジトリ概要マップをAIコンテキストに注入し、正確なコード修正を実現します。
- **プラットフォームターゲット**: リリース workflow は Linux musl と Windows MSVC を対象とし、各 runner の build/smoke 成功をサポート判定に使用します。
- **v3.9.0 TUI近代化改修**: レスポンシブな3分割ビューポート、`Tab`/`Shift+Tab`インスペクター操作、点滅カーソルと共通 render/hit-test geometry を提供。
- **Workspace Harness 診断**: `smlcli doctor`, `/workspace show`, `/status` は同じ snapshot を使い、OS、Host Shell、Exec Shell、canonical workspace root、trust/deny 状態、Linux `/workspace` sandbox mount 方針を表示します。
- **Workspace Harness 強制適用**: 現在の OS/root/shell/sandbox/trust snapshot を system prompt、tool preflight、session log に接続し、環境誤認による誤実行を減らします。

### クイックスタート
1. リポジトリをクローンします。
   ```bash
   git clone https://github.com/Yupkidangju/smlcli.git
   cd smlcli
   ```
2. アプリをビルドして実行します。
   ```bash
   ./build.sh
   # または
   cargo run --release
   ```

### トラブルシューティング
- **仮想／ヘッドレス環境のサイズ**: 自動テストは代表 breakpoint と `(100, 30)` の仮想サイズを検証します。端末固有の挙動は物理TTYで追加確認できます。
- **Slash Command のサブコマンド入力**: `/` で自動補完メニューが開いても、入力した文字は Composer に残ります。`/workspace ` の後は `show`, `trust`, `deny`, `clear` を選択または直接入力し、Enter で実行できます。
- **Workspace Harness 確認**: Linux sandbox が有効な場合、`ExecShell` 内部の作業パスは `/workspace` に標準化されます。`smlcli doctor` の `Workspace Harness` セクションで実際の OS/shell/root/trust/sandbox 値を確認してください。
- **環境 mismatch 防止**: Linux/Windows 専用 shell command の混同は即時自動実行せず、Notice/Ask 経路へ昇格します。

---

## 繁體中文

### 簡介
`smlcli` 是一款專為終端機設計的 AI 代理 CLI 工具。啟動應用後即進入 TUI 介面，並可透過 `/setting` 安裝設定。支援使用自然語言或斜線指令進行檔案瀏覽、修改與指令執行。

### 核心功能
- **全鍵盤 TUI**: 告別滑鼠，快速進行所有主要指令操作。
- **多平台模型**: 支援 OpenAI, Anthropic, xAI, OpenRouter, Google (Gemini) 及 LM Studio 本地提供者（提供 API Key 跳過與自訂 Base URL 輸入分流）。
- **安全邊界**: 本機加密保存 API 密鑰，提供寫入／Shell 核准與 canonical workspace path 驗證。Linux `bwrap` 僅在 namespace capability probe 成功時使用，否則拒絕執行。程式會在取消／結束時清理其擁有的 process group；已知密鑰的串流遮罩可降低洩漏風險，但不是通用 secret detector。
- **極端環境穩定性**: 在磁碟空間不足 (`ENOSPC`) 時自動防護崩潰、API 網路超時的指數退避重試，基於 `unicode-width` 的安全終端渲染，防範 OOM 的大規模輸出記憶體封頂限制，以及支援 OSC 序列的終端機標題與任務欄進度同步。
- **智能上下文壓縮**: 透過後台 LLM 摘要保護長對話串並支持動態代幣(Token)管理。
- **@ 檔案快速參照**: 從 canonical workspace 選擇 regular UTF-8 檔案，並在單檔／單輪上限內加入 AI 上下文。
- **即時主題切換**: 透過 `/theme` 指令在 Default 與 HighContrast 主題間即時切換。
- **Inspector 搜索分頁**: 即時全文搜索整個時間線（最多顯示50筆結果）。
- **SSE 串流**: 逐字符即時顯示 AI 回應。
- **JSONL 對話記錄**: 自動記錄對話內容並支援工作階段還原。
- **代理自主性 (Agentic Autonomy)**: 在明確政策檢查與有限恢復循環下提供 `ListDir`, `GrepSearch`, `FetchURL` 等工具。為保護使用者 WIP，應用程式的自動 hard reset／auto-commit 套用路徑已停用。
- **離線手動 Fallback**: 在本地/離線環境或 API Ping 失敗時，設定精靈仍提供「✏ 離線手動輸入...」模式，允許手動指定模型名稱，確保在無連線狀態下也能順利完成 Onboarding。
- **Tree-sitter Repo Map**: 將基於 AST 解析的儲存庫摘要地圖注入 AI 上下文中，實現精確的程式碼修改。
- **平台目標**: 發行 workflow 以 Linux musl 與 Windows MSVC 為目標，平台支援以各 runner 的 build/smoke 成功為準。
- **v3.9.0 TUI 現代化改編**: 提供響應式 3 分割視埠、Inspector `Tab`/`Shift+Tab` 操作、游標動畫及共用 render/hit-test geometry。
- **Workspace Harness 診斷**: `smlcli doctor`、`/workspace show`、`/status` 使用同一份 snapshot 顯示 OS、Host Shell、Exec Shell、canonical workspace root、trust/deny 狀態與 Linux `/workspace` sandbox mount 政策。
- **Workspace Harness 強制套用**: 將目前 OS/root/shell/sandbox/trust snapshot 串接到 system prompt、tool preflight 與 session log，降低環境誤判造成的錯誤執行。

### 快速開始
1. 複製專案:
   ```bash
   git clone https://github.com/Yupkidangju/smlcli.git
   cd smlcli
   ```
2. 執行程式:
   ```bash
   ./build.sh
   # 或者
   cargo run --release
   ```

### 疑難排解
- **虛擬／Headless 終端尺寸**: 自動測試涵蓋代表性 breakpoint 與 `(100, 30)` 虛擬尺寸；終端模擬器差異仍可透過實體 TTY 額外確認。
- **Slash Command 子指令輸入**: `/` 開啟自動完成選單時，輸入文字仍會保留在 Composer。輸入 `/workspace ` 後可選擇或直接輸入 `show`, `trust`, `deny`, `clear`，再按 Enter 執行完整指令。
- **Workspace Harness 檢查**: Linux sandbox 啟用時，`ExecShell` 內部工作路徑會標準化為 `/workspace`。請在 `smlcli doctor` 的 `Workspace Harness` 區段確認實際 OS/shell/root/trust/sandbox 值。
- **環境 mismatch 防護**: Linux/Windows 專用 shell command 混用時不會立即自動執行，而會升級到 Notice/Ask 流程。

---

## 简体中文

### 简介
`smlcli` 是一款以终端为核心的 AI 代理 CLI 工具，提供类似 Codex 的操作体验。运行应用即进入 TUI 界面，通过 `/setting` 快速配置供应商、密钥与模型。支持通过自然语言执行代码修改、命令运行等代理功能。

### 核心功能
- **纯键盘 TUI**: 所有核心操作可通过键盘在3步内完成。
- **多供应商支持**: 兼容 OpenAI, Anthropic, xAI, OpenRouter, Google (Gemini) 以及 LM Studio 本地供应商（支持跳过 API 密钥并分流输入 Base URL）。
- **安全边界**: 本地加密保存 API 密钥，提供写入／Shell 审批与 canonical workspace path 验证。Linux `bwrap` 仅在 namespace capability probe 成功时使用，否则拒绝执行。程序会在取消／退出时清理其拥有的 process group；已知密钥的流式遮罩可降低泄漏风险，但不是通用 secret detector。
- **极限环境稳定性**: 在磁盘空间不足 (`ENOSPC`) 时自动防护崩溃、API 网络超时提供指数退避重试，基于 `unicode-width` 的安全终端渲染，防止 OOM 的大规模输出内存封顶限制，以及基于 OSC 序列的终端标题与任务栏进度同步功能。
- **智能上下文压缩**: 通过后台 LLM 摘要引擎保护长期会话防止记忆丢失，包含动态 Token 管理。
- **@ 文件快速查询**: 从 canonical workspace 选择 regular UTF-8 文件，并在单文件／单轮上限内加入 AI 上下文。
- **实时主题切换**: 通过 `/theme` 命令在 Default 和 HighContrast 主题间即时切换。
- **Inspector 搜索选项卡**: 实时全文搜索整个时间线（最多显示50条结果）。
- **SSE 流式传输**: 逐令牌实时显示 AI 回复。
- **JSONL 会话日志**: 自动记录对话内容并支持会话恢复。
- **代理自主性 (Agentic Autonomy)**: 在明确策略检查与有限恢复循环下提供 `ListDir`, `GrepSearch`, `FetchURL` 等工具。为保护用户 WIP，应用程序的自动 hard reset／auto-commit 应用路径已停用。
- **离线手动 Fallback**: 在本地/离线环境或 API Ping 失败时，配置向导中仍提供“✏ 直接输入...”模式，支持手动指定任意模型名称，确保在无网络连接状态下也能顺利完成 Onboarding。
- **Tree-sitter Repo Map**: 将基于 AST 解析的仓库摘要地图注入 AI 上下文中，实现精确的代码修改。
- **平台目标**: 发布 workflow 以 Linux musl 与 Windows MSVC 为目标，平台支持以各 runner 的 build/smoke 成功为准。
- **v3.9.0 TUI 现代化改编**: 提供响应式 3 分割视口、Inspector `Tab`/`Shift+Tab` 操作、光标动画和共享 render/hit-test geometry。
- **Workspace Harness 诊断**: `smlcli doctor`、`/workspace show`、`/status` 使用同一份 snapshot 显示 OS、Host Shell、Exec Shell、canonical workspace root、trust/deny 状态以及 Linux `/workspace` sandbox mount 策略。
- **Workspace Harness 强制应用**: 将当前 OS/root/shell/sandbox/trust snapshot 连接到 system prompt、tool preflight 与 session log，降低环境误判导致的错误执行。

### 快速开始
1. 克隆项目
   ```bash
   git clone https://github.com/Yupkidangju/smlcli.git
   cd smlcli
   ```
2. 本地构建并运行
   ```bash
   ./build.sh
   # 或者
   cargo run --release
   ```

### 疑难解答
- **虚拟／Headless 终端尺寸**: 自动测试覆盖代表性 breakpoint 与 `(100, 30)` 虚拟尺寸；终端模拟器差异仍可通过物理 TTY 追加确认。
- **Slash Command 子命令输入**: `/` 打开自动补全菜单时，输入文字仍会保留在 Composer 中。输入 `/workspace ` 后可选择或直接输入 `show`, `trust`, `deny`, `clear`，再按 Enter 执行完整命令。
- **Workspace Harness 检查**: Linux sandbox 启用时，`ExecShell` 内部工作路径会标准化为 `/workspace`。请在 `smlcli doctor` 的 `Workspace Harness` 区段确认实际 OS/shell/root/trust/sandbox 值。
- **环境 mismatch 防护**: Linux/Windows 专用 shell command 混用时不会立即自动执行，而会升级到 Notice/Ask 流程。
