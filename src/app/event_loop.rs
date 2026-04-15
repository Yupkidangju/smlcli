use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task;

pub enum Event {
    Tick,
    Input(KeyEvent),
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

        // Timer Task
        task::spawn(async move {
            let mut interval = tokio::time::interval(tick_rate);
            loop {
                interval.tick().await;
                if tick_tx.send(Event::Tick).await.is_err() {
                    break;
                }
            }
        });

        // Crossterm Event Polling Task (Blocking)
        task::spawn_blocking(move || {
            loop {
                if event::poll(Duration::from_millis(50)).unwrap_or(false)
                    && let Ok(CrosstermEvent::Key(key)) = event::read()
                    && key.kind == KeyEventKind::Press
                    && tx.blocking_send(Event::Input(key)).is_err()
                {
                    break;
                }
            }
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
