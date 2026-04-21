// [v0.1.0-beta.24] Phase 14-B: 마우스 이벤트 지원 추가.
// CrosstermEvent::Mouse를 Event::Mouse로 전달하여
// 마우스 휠 스크롤을 패널별로 라우팅할 수 있게 함.

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task;

pub enum Event {
    Tick,
    Input(KeyEvent),
    /// [v0.1.0-beta.24] Phase 14-B: 마우스 이벤트 (휠 스크롤 등)
    Mouse(MouseEvent),
    Action(crate::app::action::Action),
    Quit,
}

pub struct EventLoop {
    rx: mpsc::Receiver<Event>,
}

impl EventLoop {
    pub fn new(tick_rate: Duration) -> (Self, mpsc::Sender<Event>) {
        let (tx, rx) = mpsc::channel(100);
        let tick_tx = tx.clone();
        let app_tx = tx.clone();

        // 타이머 태스크: tick_rate 간격으로 Tick 이벤트 전송
        task::spawn(async move {
            let mut interval = tokio::time::interval(tick_rate);
            loop {
                interval.tick().await;
                if tick_tx.send(Event::Tick).await.is_err() {
                    break;
                }
            }
        });

        // Crossterm 이벤트 폴링 (블로킹 태스크)
        // [v0.1.0-beta.24] 키 이벤트와 마우스 이벤트를 모두 수신
        task::spawn_blocking(move || {
            loop {
                if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                    match event::read() {
                        Ok(CrosstermEvent::Key(key)) if key.kind == KeyEventKind::Press => {
                            if tx.blocking_send(Event::Input(key)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Mouse(mouse)) => {
                            if tx.blocking_send(Event::Mouse(mouse)).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
        });

        // [v1.4.0] 시스템 신호 (SIGINT, SIGTERM) 수신 시 Graceful Shutdown (Event::Quit 전송)
        let signal_tx = app_tx.clone();
        task::spawn(async move {
            #[cfg(unix)]
            {
                let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {}
                    _ = sigterm.recv() => {}
                }
            }
            #[cfg(not(unix))]
            {
                let _ = tokio::signal::ctrl_c().await;
            }
            let _ = signal_tx.send(Event::Quit).await;
        });

        (Self { rx }, app_tx)
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Event channel closed"))
    }
}
