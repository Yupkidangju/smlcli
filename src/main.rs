// [v0.1.0-beta.18] Phase 10: CLI Entry Modes 구현.
// [v0.1.0-beta.19] 비동기 I/O 전환에 따른 main 루프 및 서브커맨드 await 적용.

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

pub mod shadow {
    shadow_rs::shadow!(build);
}

use anyhow::Result;
use app::App;
use clap::{CommandFactory, Parser, Subcommand};

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
    Doctor {
        #[arg(
            long,
            help = "비정상 종료된 SMLCLI_PID 자식 프로세스들을 찾아 강제 종료합니다"
        )]
        clean_orphans: bool,
    },
    /// 저장된 세션 목록 조회
    Sessions,
    /// 셸 자동완성 스크립트 출력
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // [v2.5.0] Phase 34: 시작 시 백그라운드 스레드로 고아 프로세스 정리
    std::thread::spawn(|| {
        crate::infra::process_reaper::reap_orphans();
    });

    match cli.command {
        // 서브커맨드가 없거나 'run'이면 인터랙티브 TUI 진입
        None | Some(Commands::Run) => run_interactive().await,
        Some(Commands::Doctor { clean_orphans }) => run_doctor(clean_orphans).await,
        Some(Commands::Sessions) => run_sessions().await,
        Some(Commands::Completions { shell }) => {
            run_completions(shell);
            Ok(())
        }
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
async fn run_doctor(clean_orphans: bool) -> Result<()> {
    if clean_orphans {
        println!("🔍 고아 프로세스 스캔 및 정리 중...");
        crate::infra::process_reaper::reap_orphans();
        println!("✅ 프로세스 정리가 완료되었습니다.");
        return Ok(());
    }

    let report = infra::doctor::DoctorReport::run_diagnostics().await;
    report.print_report();
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
            println!("세션 목록을 읽어오는 중 오류가 발생했습니다: {}", e);
        }
    }
    Ok(())
}

fn run_completions(shell: clap_complete::Shell) {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
}
