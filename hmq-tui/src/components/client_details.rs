use std::ops::Deref;
use std::time::Duration;
use clap::builder::Str;
use color_eyre::eyre::Result;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::sleep;
use tui::Frame;
use crate::{hivemq_rest_client, tui};
use crate::action::Action;
use crate::cli::Cli;
use crate::components::Component;
use crate::components::home::Home;
use crate::components::tab_components::TabComponent;


pub struct ClientDetails {
    command_tx: Option<UnboundedSender<Action>>,
    state: ListState,
    items: Vec<String>,
    is_loading: bool,
}

impl ClientDetails {
    pub fn new() -> Self {
        ClientDetails {
            command_tx: None,
            state: ListState::default(),
            items: vec![],
            is_loading: false,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i < self.items.len() - 1 {
                    i + 1
                } else {
                    return;
                }
            }
            None => {
                0
            }
        };
        self.state.select(Some(i))
    }

    fn prev(&mut self) {
        let i = match self.state.selected() {
            Some(i) => if i == 0 {
                0
            } else {
                i - 1
            },
            None => {
                0
            }
        };
        self.state.select(Some(i))
    }

    fn reset(&mut self) {
        self.state.select(None)
    }

    fn download_details(&mut self) {
        let tx = self.command_tx.clone();
        let handle = tokio::spawn(async {
            tx.clone().unwrap().send(Action::Loading)?;
            sleep(Duration::from_secs_f64(1.0)).await;
            let mut clients = vec![];
            for i in 0..10 {
                clients.push(format!("client-{i}"))
            }
            tx.unwrap().send(Action::Loaded(clients))
        });
    }
}

impl Component for ClientDetails {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if !self.is_loading {
            match action {
                Action::Up => {
                    self.prev();
                }
                Action::Down => {
                    if self.items.is_empty() {
                        self.download_details();
                    } else {
                        self.next();
                    }
                },
                Action::Escape => {
                    self.reset()
                }
                Action::Loading => {
                    self.is_loading = true;
                }
                _ => ()
            };
        } else {
            match action {
                Action::Loaded(items) => {
                    self.items = items;
                    self.is_loading = false;
                }
                _ => ()
            }
        }

        Ok(None)
    }


    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let client_details_view = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(75),
            ])
            .split(area);

        if self.is_loading {
            f.render_widget(Block::default().borders(Borders::ALL).title("Loading Clients..."), client_details_view[0]);
        } else {
            let mut list_items = vec![];
            for item in &self.items {
                list_items.push(ListItem::new(item.as_str()));
            }
            let items = List::new(list_items)
                .block(Block::default().borders(Borders::ALL).title("Clients"))
                .highlight_style(
                    Style::default()
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                );

            f.render_stateful_widget(items, client_details_view[0], &mut self.state);
        }

        f.render_widget(Block::default().borders(Borders::ALL).title("Details"), client_details_view[1]);

        Ok(())
    }
}

impl TabComponent for ClientDetails {
    fn get_name(&self) -> String {
        "Client Details".to_string()
    }
}