// [v0.1.0-beta.18] Phase 10: CLI Entry Modes 구현.
// [v0.1.0-beta.19] 비동기 I/O 전환에 따른 main 루프 및 서브커맨드 await 적용.

#![allow(dead_code)]

mod app;
mod commands;
mod domain;
mod infra;
mod providers;
mod tools;
mod tui;
mod types;

#[cfg(test)]
mod tests;

use anyhow::Result;
use app::App;
use clap::{Parser, Subcommand};

/// smlcli — 로컬 AI 런타임 CLI 에이전트
#[derive(Parser)]
#[command(
    name = "smlcli",
    version = env!("CARGO_PKG_VERSION"),
    about = "로컬 AI 런타임 CLI 에이전트 — 터미널에서 직접 작동하는 자율 코딩 어시스턴트",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// [v0.1.0-beta.18] 사용 가능한 서브커맨드
#[derive(Subcommand)]
enum Commands {
    /// 인터랙티브 TUI 모드로 실행 (기본값)
    Run,
    /// 시스템 환경 진단 (API 키, 설정, 의존성 등)
    Doctor,
    /// 저장된 세션 목록 조회
    Sessions,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // 서브커맨드가 없거나 'run'이면 인터랙티브 TUI 진입
        None | Some(Commands::Run) => run_interactive().await,
        Some(Commands::Doctor) => run_doctor().await,
        Some(Commands::Sessions) => run_sessions().await,
    }
}

/// 인터랙티브 TUI 모드: 기존 메인 루프 실행
async fn run_interactive() -> Result<()> {
    // 1. 패닉 훅 등록
    tui::terminal::install_panic_hook();

    // 2. 터미널 진입 (RAII)
    let mut terminal = tui::terminal::TerminalGuard::init()?;

    // 3. 앱 실행
    let (events, tx) = app::event_loop::EventLoop::new(std::time::Duration::from_millis(250));

    // [v0.1.0-beta.19] App::new 도 비동기 초기화 필요 (설정 로드)
    let mut app = App::new(tx).await;
    app.run(&mut terminal, events).await
}

/// Doctor: 환경 진단
async fn run_doctor() -> Result<()> {
    println!("🩺 smlcli doctor — 환경 진단\n");

    // 설정 파일 확인
    match infra::config_store::load_config().await {
        Ok(Some(settings)) => {
            println!("✅ 설정 파일: 로드 완료");
            println!("   공급자: {}", settings.default_provider);
            println!("   모델: {}", settings.default_model);
        }
        Ok(None) => {
            println!("⚠️  설정 파일: 없음 — 'smlcli run'으로 시작 후 설정 마법사를 완료하세요");
        }
        Err(e) => {
            println!("❌ 설정 파일 로드 오류: {}", e);
        }
    }

    // 세션 디렉토리 확인
    match infra::session_log::SessionLogger::list_sessions().await {
        Ok(sessions) => {
            println!("📁 세션 파일: {}개", sessions.len());
        }
        Err(e) => {
            println!("⚠️  세션 디렉토리 조회 오류: {}", e);
        }
    }

    // 시스템 정보
    println!("\n📊 시스템 정보:");
    println!("   OS: {} {}", std::env::consts::OS, std::env::consts::ARCH);
    println!("   Rust: {}", env!("CARGO_PKG_VERSION"));

    let home = dirs::home_dir().unwrap_or_default();
    let config_dir = home.join(".smlcli");
    println!("   설정 경로: {}", config_dir.display());

    println!("\n진단 완료.");
    Ok(())
}

/// Sessions: 저장된 세션 목록 표시
async fn run_sessions() -> Result<()> {
    println!("📋 smlcli sessions — 저장된 세션 목록\n");

    match infra::session_log::SessionLogger::list_sessions().await {
        Ok(sessions) => {
            if sessions.is_empty() {
                println!("저장된 세션이 없습니다.");
            } else {
                println!("{:<40} {:>10} {:>8}", "파일명", "크기", "메시지");
                println!("{}", "-".repeat(60));
                for (name, size, lines) in &sessions {
                    let size_str = if *size > 1024 {
                        format!("{:.1} KB", *size as f64 / 1024.0)
                    } else {
                        format!("{} B", size)
                    };
                    println!("{:<40} {:>10} {:>8}", name, size_str, lines);
                }
                println!("\n총 {}개 세션", sessions.len());
            }
        }
        Err(e) => {
            println!("세션 목록 조회 오류: {}", e);
        }
    }

    Ok(())
}
