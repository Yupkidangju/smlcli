#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod app;
mod tui;
mod domain;
mod providers;
mod tools;
mod infra;
mod commands;
mod types;

#[cfg(test)]
mod tests;

use app::App;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
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
