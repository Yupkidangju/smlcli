# smlcli

터미널 중심 AI 에이전트 CLI 도구 (Terminal-native AI Agent CLI)

[한국어](#한국어) | [English](#english) | [日本語](#日本語) | [繁體中文](#繁體中文) | [简体中文](#简体中文)

---

## 한국어

### 소개
`smlcli`는 Codex/OpenCode 계열의 사용감을 갖는 터미널 중심 AI 에이전트 CLI입니다. 앱 실행 시 TUI에 진입하며, `/setting`을 통해 공급자, API 키, 모델, 권한 정책을 설정합니다. 자연어 프롬프트와 `/` 명령어를 통해 파일 탐색, 코드 수정, 명령 실행 및 diff 검토를 지원합니다.

### 주요 기능
- **터미널 중심 TUI**: 마우스 없이 모든 동작을 3단계 이내에 키보드로 처리.
- **다중 공급자 지원**: OpenRouter, Google (Gemini) 지원 (추후 확장 가능).
- **강력한 보안 및 검증**: 파일 쓰기, 쉘 실행 검사, API 키의 로컬 OS Keyring 암호화 보관.
- **Inspect 패널과 Diff 플로우**: 작업 승인 전에 변경될 항목 가시성 확보.
- **지능형 컨텍스트 압축**: 장기 세션 보호를 위한 백그라운드 LLM 요약기 및 토큰 한도 제어(`/tokens`).
- **@ 로컬 데이터 참조**: `@` 퍼지 파인터를 통해 작업 파일 경로와 컨텍스트를 빠짐없이 LLM에 자동 인라인 삽입.
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
- **Multi-provider**: Supports OpenRouter and Google (Gemini) initially.
- **Security-focused**: Local OS keyring storage for API keys, explicit approval flows for file writing and shell execution.
- **Inspector & Diff Previews**: High visibility of what is changing before you approve it.
- **Intelligent Compaction**: LLM-based background summarization and `/tokens` budgeting to protect long sessions.
- **@ Context Injection**: Fuzzy search your filesystem with `@` and inline file data safely without closing Composer.
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
- **マルチプロバイダー対応**: OpenAI, Anthropic, Google, OpenRouter 等をサポート。
- **堅牢なセキュリティ**: APIキーのローカル OS Keyring 保存、安全なコマンド実行ポリシー設定。
- **インテリジェント コンテキスト圧縮**: 長期セッション保護のためのバックグラウンド LLM 要約と `/tokens` トークン監視。
- **@ ローカルデータ参照**: `@` ファジー検索から該当ファイルのコンテンツをAIに自動挿入。

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
- **多平台模型**: 支援 OpenAI, Anthropic, Google, OpenRouter 等平台。
- **高規格安全**: 使用本地 Keyring 搭配本機加密，保護 API 密鑰。提供完整變更預覽與權限驗證流程。
- **智能上下文壓縮**: 透過後台 LLM 摘要保護長對話串並支持動態代幣(Token)管理。
- **@ 檔案快速參照**: 輸入 `@` 即可使用 Fuzzy Finder 將本地檔案匯入 AI 記憶。

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
- **多供应商支持**: 兼容 OpenAI, Anthropic, Google, OpenRouter 及自定义节点。
- **安全性优先**: 在执行写入和 Shell 执行前自动生成 Diff，并要求显式权限授权；密钥均存入系统 Keyring。
- **智能上下文压缩**: 通过后台 LLM 摘要引擎保护长期会话防止记忆丢失，包含动态 Token 管理。
- **@ 文件快速查询**: 输入 `@` 自动使用 Fuzzy Finder 将本地文件数据嵌入 AI 记忆上下文。

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
