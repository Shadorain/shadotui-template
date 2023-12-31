use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::{self, UnboundedSender};
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Action, Component, Frame, Message, Other};

#[derive(Default, Copy, Clone, PartialEq, Eq)]
enum Mode {
    #[default]
    Normal,
    Insert,
    Processing,
}

#[derive(Default)]
pub struct Base {
    counter: usize,
    input: Input,
    mode: Mode,
    ticker: usize,

    other: Other,
    show_other: bool,

    action_tx: Option<mpsc::UnboundedSender<Action>>,
    message_tx: Option<mpsc::UnboundedSender<Message>>,
}

impl Base {
    pub fn new() -> Self {
        Self::default()
    }

    fn tick(&mut self) {
        self.ticker = self.ticker.saturating_add(1);
    }

    fn schedule_increment(&mut self, i: usize) {
        let tx = self.action_tx.clone().unwrap();
        tokio::spawn(async move {
            tx.send(Action::EnterProcessing).unwrap();
            tokio::time::sleep(Duration::from_secs(5)).await;
            tx.send(Action::Increment(i)).unwrap();
            tx.send(Action::ExitProcessing).unwrap();
        });
    }

    fn schedule_decrement(&mut self, i: usize) {
        let tx = self.action_tx.clone().unwrap();
        tokio::spawn(async move {
            tx.send(Action::EnterProcessing).unwrap();
            tokio::time::sleep(Duration::from_secs(5)).await;
            tx.send(Action::Decrement(i)).unwrap();
            tx.send(Action::ExitProcessing).unwrap();
        });
    }

    fn increment(&mut self, i: usize) {
        self.counter = self.counter.saturating_add(i);
    }

    fn decrement(&mut self, i: usize) {
        self.counter = self.counter.saturating_sub(i);
    }
}

impl Component for Base {
    fn init(
        &mut self,
        tx: UnboundedSender<Action>,
        message_tx: Option<mpsc::UnboundedSender<Message>>,
    ) -> anyhow::Result<()> {
        self.action_tx = Some(tx.clone());
        self.message_tx = message_tx.clone();

        self.other.init(tx, message_tx)?;
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match self.mode {
            Mode::Normal | Mode::Processing => match key.code {
                KeyCode::Char('q') => Action::Quit,
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
                KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Action::Suspend
                }
                KeyCode::Char('l') => Action::ToggleShowLogger,
                KeyCode::Char('j') => Action::ScheduleIncrement,
                KeyCode::Char('k') => Action::ScheduleDecrement,
                KeyCode::Char('/') => Action::EnterInsert,
                _ => Action::Tick,
            },
            Mode::Insert => match key.code {
                KeyCode::Esc => Action::EnterNormal,
                KeyCode::Enter => Action::CompleteInput(self.input.to_string()),
                _ => {
                    self.input.handle_event(&Event::Key(key));
                    Action::Update
                }
            },
        }
    }

    fn dispatch(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::Tick => self.tick(),
            Action::ToggleShowLogger => self.show_other = !self.show_other,
            Action::ScheduleIncrement => self.schedule_increment(1),
            Action::ScheduleDecrement => self.schedule_decrement(1),
            Action::Increment(i) => self.increment(i),
            Action::Decrement(i) => self.decrement(i),
            Action::EnterNormal => {
                self.mode = Mode::Normal;
            }
            Action::CompleteInput(s) => {
                if let Some(tx) = &self.message_tx {
                    tx.send(Message::HelloWorld(s)).unwrap();
                }
                return Some(Action::EnterNormal);
            }
            Action::EnterInsert => {
                self.mode = Mode::Insert;
            }
            Action::EnterProcessing => {
                self.mode = Mode::Processing;
            }
            Action::ExitProcessing => {
                // TODO: Make this go to previous mode instead
                self.mode = Mode::Normal;
            }
            _ => (),
        }
        None
    }

    fn render(&mut self, f: &mut Frame<'_>, rect: Rect) {
        let rect = if self.show_other {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(rect);
            self.other.render(f, chunks[1]);
            chunks[0]
        } else {
            rect
        };

        let rects = Layout::default()
            .constraints([Constraint::Percentage(100), Constraint::Min(3)].as_ref())
            .split(rect);

        f.render_widget(
            Paragraph::new(format!(
                "Press j or k to increment or decrement.\n\nCounter: {}\n\nTicker: {}",
                self.counter, self.ticker
            ))
            .block(
                Block::default()
                    .title("Template")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(match self.mode {
                        Mode::Processing => Style::default().fg(Color::Yellow),
                        _ => Style::default(),
                    })
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center),
            rects[0],
        );
        let width = rects[1].width.max(3) - 3; // keep 2 for borders and 1 for cursor
        let scroll = self.input.visual_scroll(width as usize);
        let input = Paragraph::new(self.input.value())
            .style(match self.mode {
                Mode::Insert => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .scroll((0, scroll as u16))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(vec![
                        Span::raw("Enter Input Mode "),
                        Span::styled("(Press ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            "/",
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Gray),
                        ),
                        Span::styled(" to start, ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            "ESC",
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Gray),
                        ),
                        Span::styled(" to finish)", Style::default().fg(Color::DarkGray)),
                    ])),
            );
        f.render_widget(input, rects[1]);
        if self.mode == Mode::Insert {
            f.set_cursor(
                (rects[1].x + 1 + self.input.cursor() as u16).min(rects[1].x + rects[1].width - 2),
                rects[1].y + 1,
            )
        }
    }
}
