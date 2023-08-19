use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{mpsc, Mutex};

use super::{
    components::{Base, Component},
    Action, EventHandler, TerminalHandler,
};

pub struct App {
    tick_rate: (u64, u64),
    should_quit: bool,
    should_suspend: bool,

    base: Arc<Mutex<Base>>,
}

impl App {
    pub fn new(tick_rate: (u64, u64)) -> Result<Self> {
        let home = Arc::new(Mutex::new(Base::new()));
        Ok(Self {
            tick_rate,
            base: home,
            should_quit: false,
            should_suspend: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();

        self.base.lock().await.init(action_tx.clone())?;

        let mut terminal = TerminalHandler::new(self.base.clone());
        let mut event = EventHandler::new(self.tick_rate, self.base.clone(), action_tx.clone());

        loop {
            if let Some(action) = action_rx.recv().await {
                match action {
                    Action::RenderTick => terminal.render()?,
                    Action::Quit => self.should_quit = true,
                    Action::Suspend => self.should_suspend = true,
                    Action::Resume => self.should_suspend = false,
                    _ => {
                        if let Some(_action) = self.base.lock().await.dispatch(action) {
                            action_tx.send(_action)?
                        };
                    }
                }
            }
            if self.should_suspend {
                terminal.suspend()?;
                event.stop();
                terminal.task.await?;
                event.task.await?;
                terminal = TerminalHandler::new(self.base.clone());
                event = EventHandler::new(self.tick_rate, self.base.clone(), action_tx.clone());
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::RenderTick)?;
            } else if self.should_quit {
                terminal.stop()?;
                event.stop();
                terminal.task.await?;
                event.task.await?;
                break;
            }
        }
        Ok(())
    }
}
