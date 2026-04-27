// [v0.1.0-beta.24] Phase 14-B: 마우스 캡처 추가.
// EnableMouseCapture를 통해 마우스 휠 이벤트를 수신하여
// 타임라인/인스펙터 패널별 독립 스크롤을 지원한다.

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout};

pub type TuiTerminal = Terminal<CrosstermBackend<Stdout>>;

pub struct TerminalGuard {
    pub terminal: TuiTerminal,
}

impl TerminalGuard {
    pub fn init() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    /// 서브 프로세스 종료 직후 터미널 잔상(Ghosting) 제거 및 커서 명시적 재설정.
    pub fn clear_and_reset(&mut self) -> Result<()> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            crossterm::cursor::Show,
            crossterm::cursor::MoveTo(0, 0)
        )?;
        self.terminal.clear()?;
        Ok(())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = restore_terminal();
    }
}

impl std::ops::Deref for TerminalGuard {
    type Target = TuiTerminal;
    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl std::ops::DerefMut for TerminalGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

pub fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

pub fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // [v1.3.0] 패닉 시 즉각적인 터미널 복구 강제
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        );
        original_hook(panic_info);
    }));
}
