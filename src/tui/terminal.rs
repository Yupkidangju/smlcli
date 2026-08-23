// [v0.1.0-beta.24] Phase 14-B: 마우스 캡처 추가.
// EnableMouseCapture를 통해 마우스 휠 이벤트를 수신하여
// 타임라인/인스펙터 패널별 독립 스크롤을 지원한다.

use anyhow::Result;
use crossterm::{
    cursor::Show,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout};

pub type TuiTerminal = Terminal<CrosstermBackend<Stdout>>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TerminalInitState {
    pub raw_mode: bool,
    pub alternate_screen: bool,
    pub mouse_capture: bool,
}

pub struct TerminalGuard {
    pub terminal: TuiTerminal,
    state: TerminalInitState,
}

impl TerminalGuard {
    pub fn init() -> Result<Self> {
        let mut state = TerminalInitState::default();
        enable_raw_mode()?;
        state.raw_mode = true;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen) {
            let _ = restore_terminal_state(state);
            return Err(error.into());
        }
        state.alternate_screen = true;
        if let Err(error) = execute!(stdout, EnableMouseCapture) {
            let _ = restore_terminal_state(state);
            return Err(error.into());
        }
        state.mouse_capture = true;
        let backend = CrosstermBackend::new(stdout);
        let terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => {
                let _ = restore_terminal_state(state);
                return Err(error.into());
            }
        };
        Ok(Self { terminal, state })
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
        let _ = self.terminal.show_cursor();
        let _ = restore_terminal_state(self.state);
        self.state = TerminalInitState::default();
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
    restore_terminal_state(TerminalInitState {
        raw_mode: true,
        alternate_screen: true,
        mouse_capture: true,
    })
}

fn restore_terminal_state(state: TerminalInitState) -> Result<()> {
    let mut stdout = io::stdout();
    let mut first_error: Option<anyhow::Error> = None;
    if let Err(error) = execute!(stdout, Show) {
        first_error = Some(error.into());
    }
    if state.mouse_capture
        && let Err(error) = execute!(stdout, DisableMouseCapture)
        && first_error.is_none()
    {
        first_error = Some(error.into());
    }
    if state.alternate_screen
        && let Err(error) = execute!(stdout, LeaveAlternateScreen)
        && first_error.is_none()
    {
        first_error = Some(error.into());
    }
    if state.raw_mode
        && let Err(error) = disable_raw_mode()
        && first_error.is_none()
    {
        first_error = Some(error.into());
    }
    first_error.map_or(Ok(()), Err)
}

pub fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));
}
