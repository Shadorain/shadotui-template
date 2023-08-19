use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode}, execute,
};
use ratatui::{backend::CrosstermBackend as Backend, Terminal};
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};

use super::components::{Base, Component};

pub type Frame<'a> = ratatui::Frame<'a, Backend<std::io::Stderr>>;

pub struct Tui {
    terminal: Terminal<Backend<std::io::Stderr>>,
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().expect("Failed to cleanup terminal.");
    }
}

impl Tui {
    pub fn new() -> Result<Self> {
        Ok(Self { terminal: Terminal::new(Backend::new(std::io::stderr()))? })
    }

    pub fn enter(&self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        Ok(execute!(
            std::io::stderr(),
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )?)
    }

    pub fn exit(&self) -> Result<()> {
        execute!(
            std::io::stderr(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        )?;
        Ok(disable_raw_mode()?)
    }

    pub fn suspend(&self) -> Result<()> {
        self.exit()?;
        #[cfg(not(windows))]
        Ok(signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Message {
    Render,
    Stop,
    Suspend,
}

pub struct TerminalHandler {
    pub task: JoinHandle<()>,
    tx: mpsc::UnboundedSender<Message>,
}

impl TerminalHandler {
    pub fn new(home: Arc<Mutex<Base>>) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

        let task = tokio::spawn(async move {
            let mut t = Tui::new()
                .context(anyhow!("Unable to create terminal"))
                .unwrap();
            t.enter().unwrap();
            loop {
                match rx.recv().await {
                    Some(Message::Stop) => {
                        t.exit().unwrap_or_default();
                        break;
                    }
                    Some(Message::Suspend) => {
                        t.suspend().unwrap_or_default();
                        break;
                    }
                    Some(Message::Render) => {
                        let mut h = home.lock().await;
                        t.terminal.draw(|f| {
                            h.render(f, f.size());
                        })
                        .unwrap();
                    }
                    None => {}
                }
            }
        });
        Self { task, tx }
    }

    pub fn suspend(&self) -> Result<()> {
        Ok(self.tx.send(Message::Suspend)?)
    }

    pub fn stop(&self) -> Result<()> {
        Ok(self.tx.send(Message::Stop)?)
    }

    pub fn render(&self) -> Result<()> {
        Ok(self.tx.send(Message::Render)?)
    }
}
