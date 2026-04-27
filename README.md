# smlcli

터미널 중심 AI 에이전트 CLI 도구 (Terminal-native AI Agent CLI)

[한국어](#한국어) | [English](#english) | [日本語](#日本語) | [繁體中文](#繁體中文) | [简体中文](#简体中文)

---

## 한국어

### 소개
`smlcli`는 Codex/OpenCode 계열의 사용감을 갖는 터미널 중심 AI 에이전트 CLI입니다. 앱 실행 시 TUI에 진입하며, `/setting`을 통해 공급자, API 키, 모델, 권한 정책을 설정합니다. 자연어 프롬프트와 `/` 명령어를 통해 파일 탐색, 코드 수정, 명령 실행 및 diff 검토를 지원합니다.

### 주요 기능
- **터미널 중심 TUI**: 마우스 없이 모든 동작을 3단계 이내에 키보드로 처리.
- **다중 공급자 지원**: OpenAI, Anthropic, xAI, OpenRouter, Google (Gemini) 지원.
- **강력한 보안 및 검증**: 파일 쓰기, 쉘 실행 검사, API 키의 로컬 파일 기반 암호화 보관 (`~/.smlcli/config.toml`, ChaCha20Poly1305). 심볼릭 링크를 방어하는 엄격한 샌드박스와 Linux `bwrap` 기반 실제 셸 샌드박스, 프로세스 그룹 소멸 및 환경 변수 격리 기능 제공. 스트리밍 마스킹을 통한 API 키 유출 원천 차단.
- **극한 상황 강건성**: 설정 파일 마이그레이션 실패 시 자동 롤백 및 백업 기능. 디스크 용량 한계(`ENOSPC`) 도달 시 패닉을 방지하는 그레이스풀 폴백, API 네트워크 타임아웃 래핑 및 지수 백오프 기반 재시도, `unicode-width` 기반의 터미널 렌더링 안정성 확보. 대용량 파일/로그 출력의 OOM을 막는 메모리 캡핑(Size Capping) 및 터미널 제목/작업표시줄 진행률(OSC) 동기화 지원. `smlcli doctor` 시스템 진단.
- **Inspect 패널과 Diff 플로우**: 작업 승인 전에 변경될 항목 가시성 확보.
- **지능형 컨텍스트 압축 및 성능 최적화**: 장기 세션 보호를 위한 백그라운드 LLM 요약기 및 토큰 한도 제어(`/tokens`), 디스크 캐시 기반 AST RepoMap 생성으로 대형 레포지토리에서의 속도 확보.
- **클립보드 연동**: `y` 키 단축키 및 시각적 토스트(Toast) 알림을 통한 즉각적인 클립보드 복사 지원.
- **환경 변수 제어**: `allowed_env_vars` 화이트리스트를 통한 도구 실행 환경 제어.
- **@ 로컬 데이터 참조**: `@` 퍼지 파인터를 통해 작업 파일 경로와 컨텍스트를 빠짐없이 LLM에 자동 인라인 삽입.
- **실시간 테마 전환**: `/theme` 명령어로 Default ↔ HighContrast 테마를 즉시 전환. 설정 파일에 자동 저장.
- **Inspector Search 탭**: 타임라인 전체를 대소문자 무시로 실시간 검색 (최대 50건 표시).
- **SSE 스트리밍**: AI 응답을 토큰 단위로 실시간 표시 (OpenRouter/Gemini 대응).
- **JSONL 세션 로그**: 대화 내용을 `~/.smlcli/sessions/`에 자동 기록하여 세션 복원 지원.
- **에이전트 자율성 (Agentic Autonomy)**: 파괴적인 동작 전후 Git 자동 체크포인트와 자가 복구 루프(Self-healing)를 통해 안전한 AI 코드 작성을 보장. `ListDir`, `GrepSearch`, `FetchURL` 등 고급 탐색 도구를 기본 제공.
- **Tree-sitter Repo Map**: AST 파싱 기반 저장소 요약 맵을 통해 AI가 전체 프로젝트 구조를 맥락으로 주입받아 정확한 코드를 수정.
- **플랫폼 지원**: Linux (bash/zsh) 및 Windows (PowerShell/WSL) 동시 지원.

### 빠른 시작
1. 저장소를 클론합니다.
   ```bash
   git clone https://github.com/your-repo/smlcli.git
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

---

## English

### Introduction
`smlcli` is a terminal-native AI agent CLI with Codex/OpenCode-like UX. It boots directly into a TUI where you configure providers, API keys, models, and permission policies via the `/setting` wizard. From there, you can interact using natural language or slash commands to explore files, edit code, execute commands, and review diffs.

### Key Features
- **Keyboard-first TUI**: Reach any primary action within 3 steps without a mouse.
- **Multi-provider**: Supports OpenAI, Anthropic, xAI, OpenRouter, and Google (Gemini).
- **Security-focused**: Local file-based encrypted storage for API keys (~/.smlcli/config.toml), explicit approval flows for file writing and shell execution, strict symlink sandbox protection, real `bwrap`-backed shell sandbox on Linux, process group extermination, and environment variable isolation. Ensures zero API key leakage via stateful streaming masking.
- **Extreme Robustness**: Automatic rollback and backup on configuration migration failures. Graceful fallback on storage full (`ENOSPC`) to prevent panics, `tokio::time::timeout` wrapping and exponential backoff for API network timeouts, safe UTF-8 terminal rendering via `unicode-width`, memory size capping for massive standard outputs to prevent OOM, and terminal title/taskbar progress synchronization via OSC sequences. Includes `smlcli doctor` for system diagnostics.
- **Inspect & Diff Flows**: Guaranteed visibility into upcoming changes before you hit "Approve".
- **Intelligent Context Compaction & Performance Optimization**: Background LLM summarizer for long-session stability, token limit control (`/tokens`), and AST RepoMap generation with disk caching via `cheap_hash` (mtime + file count) for faster performance in large repositories.
- **Clipboard Integration**: Instant clipboard copy support via the `y` hotkey with visual Toast notifications.
- **Environment Variable Control**: Fine-grained execution environment control via the `allowed_env_vars` whitelist.
- **@ Context Injection**: Type `@` to fuzzy-find workspace files and automatically inline their contents.
- **Real-time Theme Switching**: Swap between Default and HighContrast themes using `/theme` instantly. Automatically saved to configuration.
- **Inspector Search**: Perform case-insensitive, real-time searches across your timeline (up to 50 entries).
- **SSE Streaming**: See AI responses token-by-token in real-time (OpenRouter/Gemini compatible).
- **JSONL Session Logging**: Automatic session logs in `~/.smlcli/sessions/` for restoring prior conversations.
- **Agentic Autonomy**: Guarantees safe AI code generation via automated Git checkpoints and self-healing loops before/after destructive actions. Includes advanced tools like `ListDir`, `GrepSearch`, and `FetchURL`.
- **Tree-sitter Repo Map**: Injects AST-parsed repository summary maps into the AI context for accurate code modifications.
- **Cross-platform**: Full support for Linux and Windows.

### Quick Start
1. Clone the repository.
   ```bash
   git clone https://github.com/your-repo/smlcli.git
   cd smlcli
   ```
2. Build and run (or use `./build.sh` for interactive cross-compilation target setups).
   ```bash
   ./build.sh
   # or
   cargo run --release
   ```

---

## 日本語

### 概要
`smlcli` は、Codex/OpenCode のような使用感を持つ、ターミナル中心の AI エージェント CLI ツールです。実行すると TUI が起動し、`/setting` ウィザードを通じてプロバイダー、API キー、モデル、権限ポリシーを設定します。自然言語によるプロンプト操作でファイル操作やコマンド実行が可能です。

### 主な機能
- **ターミナルファースト TUI**: 全ての操作をキーボードだけで迅速に行えます。
- **マルチプロバイダー対応**: OpenAI, Anthropic, xAI, OpenRouter, Google (Gemini) をサポート。
- **堅牢なセキュリティ**: APIキーのローカル暗号化ファイル保存 (~/.smlcli/config.toml)、安全なコマンド実行ポリシー設定、シンボリックリンク保護のサンドボックス、Linuxでの `bwrap` 実サンドボックス実行、プロセスグループ消滅および環境変数の隔離をサポート。ストリーミング中のAPIキー漏洩を完全に防ぐステートフルマスキング。
- **極限環境での安定性**: ディスク容量不足 (`ENOSPC`) 時のパニック防止、LLM APIタイムアウト時の指数バックオフ再試行、`unicode-width` ベースの安全なUTF-8レンダリング、大規模出力時のメモリキャッピング (OOM防止)、およびOSCシーケンスを用いたターミナルタイトル/タスクバー進捗状況の同期。
- **インテリジェント コンテキスト圧縮**: 長期セッション保護のためのバックグラウンド LLM 要約と `/tokens` トークン監視。
- **@ ローカルデータ参照**: `@` ファジー検索から該当ファイルのコンテンツをAIに自動挿入。
- **リアルタイム テーマ切替**: `/theme` コマンドで Default ↔ HighContrast テーマを即座に切り替え可能。
- **Inspector 検索タブ**: タイムライン全体をリアルタイムで全文検索（最大50件表示）。
- **SSE ストリーミング**: AIの回答をトークン単位でリアルタイム表示。
- **JSONL セッションログ**: 会話を自動記録し、セッション復元をサポート。
- **エージェント自律性 (Agentic Autonomy)**: 破壊的な操作の前後で自動化されたGitチェックポイントと自己修復ループにより、安全なAIコード生成を保証します。`ListDir`, `GrepSearch`, `FetchURL` などの高度なツールを内蔵。
- **Tree-sitter Repo Map**: AST解析ベースのリポジトリ概要マップをAIコンテキストに注入し、正確なコード修正を実現します。

### クイックスタート
1. リポジトリをクローンします。
   ```bash
   git clone https://github.com/your-repo/smlcli.git
   cd smlcli
   ```
2. アプリをビルドして実行します。
   ```bash
   ./build.sh
   # または
   cargo run --release
   ```

---

## 繁體中文

### 簡介
`smlcli` 是一款專為終端機設計的 AI 代理 CLI 工具。啟動應用後即進入 TUI 介面，並可透過 `/setting` 安裝設定。支援使用自然語言或斜線指令進行檔案瀏覽、修改與指令執行。

### 核心功能
- **全鍵盤 TUI**: 告別滑鼠，快速進行所有主要指令操作。
- **多平台模型**: 支援 OpenAI, Anthropic, xAI, OpenRouter, Google (Gemini) 等平台。
- **高規格安全**: 使用本地檔案加密 (~/.smlcli/config.toml) 保護 API 密鑰。提供完整變更預覽與權限驗證流程，支援防範符號連結 (Symlink) 沙箱機制，在 Linux 使用 `bwrap` 實體 Shell 沙箱，並具備進程組銷毀與環境變數隔離能力。透過串流遮罩技術徹底杜絕 API 密鑰外洩。
- **極端環境穩定性**: 在磁碟空間不足 (`ENOSPC`) 時自動防護崩潰、API 網路超時的指數退避重試，基於 `unicode-width` 的安全終端渲染，防範 OOM 的大規模輸出記憶體封頂限制，以及支援 OSC 序列的終端機標題與任務欄進度同步。
- **智能上下文壓縮**: 透過後台 LLM 摘要保護長對話串並支持動態代幣(Token)管理。
- **@ 檔案快速參照**: 輸入 `@` 即可使用 Fuzzy Finder 將本地檔案匯入 AI 記憶。
- **即時主題切換**: 透過 `/theme` 指令在 Default 與 HighContrast 主題間即時切換。
- **Inspector 搜索分頁**: 即時全文搜索整個時間線（最多顯示50筆結果）。
- **SSE 串流**: 逐字符即時顯示 AI 回應。
- **JSONL 對話記錄**: 自動記錄對話內容並支援工作階段還原。
- **代理自主性 (Agentic Autonomy)**: 透過破壞性操作前後的自動 Git 檢查點與自我修復循環，確保 AI 程式碼生成的安全性。內建 `ListDir`, `GrepSearch`, `FetchURL` 等進階工具。
- **Tree-sitter Repo Map**: 將基於 AST 解析的儲存庫摘要地圖注入 AI 上下文中，實現精確的程式碼修改。

### 快速開始
1. 複製專案:
   ```bash
   git clone https://github.com/your-repo/smlcli.git
   cd smlcli
   ```
2. 執行程式:
   ```bash
   ./build.sh
   # 或者
   cargo run --release
   ```

---

## 简体中文

### 简介
`smlcli` 是一款以终端为核心的 AI 代理 CLI 工具，提供类似 Codex 的操作体验。运行应用即进入 TUI 界面，通过 `/setting` 快速配置供应商、密钥与模型。支持通过自然语言执行代码修改、命令运行等代理功能。

### 核心功能
- **纯键盘 TUI**: 所有核心操作可通过键盘在3步内完成。
- **多供应商支持**: 兼容 OpenAI, Anthropic, xAI, OpenRouter, Google (Gemini)。
- **安全性优先**: 在执行写入和 Shell 执行前自动生成 Diff，并要求显式权限授权；密钥存入本地加密文件 (~/.smlcli/config.toml)。支持防止符号链接攻击的沙箱，在 Linux 下使用 `bwrap` 提供真实 Shell 沙箱，同时具备进程组销毁与环境变量隔离功能。通过流式掩码彻底防止 API 密钥泄露。
- **极限环境稳定性**: 在磁盘空间不足 (`ENOSPC`) 时自动防护崩溃、API 网络超时提供指数退避重试，基于 `unicode-width` 的安全终端渲染，防止 OOM 的大规模输出内存封顶限制，以及基于 OSC 序列的终端标题与任务栏进度同步功能。
- **智能上下文压缩**: 通过后台 LLM 摘要引擎保护长期会话防止记忆丢失，包含动态 Token 管理。
- **@ 文件快速查询**: 输入 `@` 自动使用 Fuzzy Finder 将本地文件数据嵌入 AI 记忆上下文。
- **实时主题切换**: 通过 `/theme` 命令在 Default 和 HighContrast 主题间即时切换。
- **Inspector 搜索选项卡**: 实时全文搜索整个时间线（最多显示50条结果）。
- **SSE 流式传输**: 逐令牌实时显示 AI 回复。
- **JSONL 会话日志**: 自动记录对话内容并支持会话恢复。
- **代理自主性 (Agentic Autonomy)**: 通过破坏性操作前后的自动 Git 检查点与自我修复循环，确保 AI 代码生成的安全性。内置 `ListDir`, `GrepSearch`, `FetchURL` 等高级探索工具。
- **Tree-sitter Repo Map**: 将基于 AST 解析的仓库摘要地图注入 AI 上下文中，实现精确的代码修改。

### 快速开始
1. 克隆项目
   ```bash
   git clone https://github.com/your-repo/smlcli.git
   cd smlcli
   ```
2. 本地构建并运行
   ```bash
   ./build.sh
   # 或者
   cargo run --release
   ```
