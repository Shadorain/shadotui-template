use anyhow::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::{Action, Component, Frame, Message};

#[derive(Default)]
pub struct Other {}

impl Component for Other {
    fn init(
        &mut self,
        _: UnboundedSender<Action>,
        _message_tx: Option<UnboundedSender<Message>>,
    ) -> Result<()> {
        Ok(())
    }

    fn render(&mut self, f: &mut Frame<'_>, area: Rect) {
        let w = Paragraph::new("HI!").block(
            Block::new()
                .borders(Borders::ALL)
                .title("Other Window")
                .green(),
        );
        f.render_widget(w, area);
    }
}
