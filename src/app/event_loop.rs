// [v0.1.0-beta.24] Phase 14-B: 마우스 이벤트 지원 추가.
// CrosstermEvent::Mouse를 Event::Mouse로 전달하여
// 마우스 휠 스크롤을 패널별로 라우팅할 수 있게 함.

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task;

// [v3.7.0] Resize variant의 필드는 이벤트 라우팅에서 참조되지만 clippy가 직접 읽기를 감지 못함.
// 이벤트는 bounded 단일 소비자 채널에서 move된다. Action 전체를 boxing하면 모든
// 키/도구 이벤트에 불필요한 할당이 추가되므로 의도적으로 인라인 보관한다.
#[allow(dead_code, clippy::large_enum_variant)]
pub enum Event {
    Tick,
    Input(KeyEvent),
    /// [v0.1.0-beta.24] Phase 14-B: 마우스 이벤트 (휠 스크롤 등)
    Mouse(MouseEvent),
    Action(crate::app::action::Action),
    Quit,
    /// [v1.5.0] 터미널 리사이즈 이벤트
    Resize(u16, u16),
}

pub struct EventLoop {
    rx: mpsc::Receiver<Event>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    tasks: Vec<task::JoinHandle<()>>,
}

impl EventLoop {
    pub fn new(tick_rate: Duration) -> (Self, mpsc::Sender<Event>) {
        let (tx, rx) = mpsc::channel(100);
        let tick_tx = tx.clone();
        let app_tx = tx.clone();
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        // 타이머 태스크: tick_rate 간격으로 Tick 이벤트 전송
        let tick_stop = stop.clone();
        let tick_task = task::spawn(async move {
            let mut interval = tokio::time::interval(tick_rate);
            while !tick_stop.load(std::sync::atomic::Ordering::Acquire) {
                interval.tick().await;
                if tick_tx.send(Event::Tick).await.is_err() {
                    break;
                }
            }
        });

        // Crossterm 이벤트 폴링 (블로킹 태스크)
        // [v0.1.0-beta.24] 키 이벤트와 마우스 이벤트를 모두 수신
        let input_stop = stop.clone();
        let input_task = task::spawn_blocking(move || {
            while !input_stop.load(std::sync::atomic::Ordering::Acquire) {
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
                        Ok(CrosstermEvent::Resize(w, h)) => {
                            if tx.blocking_send(Event::Resize(w, h)).is_err() {
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
        let signal_task = task::spawn(async move {
            #[cfg(unix)]
            {
                let mut sigterm =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                        .unwrap();
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

        (
            Self {
                rx,
                stop,
                tasks: vec![tick_task, input_task, signal_task],
            },
            app_tx,
        )
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Event channel closed"))
    }

    pub async fn shutdown(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Release);
        self.rx.close();
        let tasks = std::mem::take(&mut self.tasks);
        for task in tasks {
            task.abort();
            let _ = tokio::time::timeout(std::time::Duration::from_millis(250), task).await;
        }
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Release);
        for task in &self.tasks {
            task.abort();
        }
    }
}
