// [v0.1.0-beta.18] Phase 10: CLI Entry Modes 구현.
// clap derive 기반으로 서브커맨드 (run/doctor/sessions/version) 지원.
// 서브커맨드 없이 실행하면 기본 인터랙티브 TUI 모드로 진입.
// --help 플래그는 clap이 자동 처리하여 도움말 출력 후 종료.
// [v0.1.0-beta.18] Phase 9-C: 전역 #![allow] 최소화 — dead_code만 유지 (MVP 미사용 코드 허용).

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
        Some(Commands::Doctor) => run_doctor(),
        Some(Commands::Sessions) => run_sessions(),
    }
}

/// 인터랙티브 TUI 모드: 기존 메인 루프 실행
async fn run_interactive() -> Result<()> {
    // 1. 패닉 훅 등록: 패닉 시에도 raw 모드 복구 보장
    tui::terminal::install_panic_hook();

    // 2. 터미널 진입 (Raw 모드, Alternate Screen)
    let mut terminal = tui::terminal::init_terminal()?;

    // 3. 앱 실행
    let (events, tx) = app::event_loop::EventLoop::new(std::time::Duration::from_millis(250));
    let mut app = App::new(tx);
    let res = app.run(&mut terminal, events).await;

    // 4. 앱 종료 후 터미널 정리 정돈
    tui::terminal::restore_terminal()?;

    res
}

/// Doctor: 환경 진단 — API 키 유무, 설정 파일 존재 여부, 시스템 정보 출력
fn run_doctor() -> Result<()> {
    println!("🩺 smlcli doctor — 환경 진단\n");

    // 설정 파일 확인
    match infra::config_store::load_config() {
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
    match infra::session_log::SessionLogger::list_sessions() {
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
fn run_sessions() -> Result<()> {
    println!("📋 smlcli sessions — 저장된 세션 목록\n");

    match infra::session_log::SessionLogger::list_sessions() {
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
