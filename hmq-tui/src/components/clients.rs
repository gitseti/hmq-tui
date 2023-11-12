use std::collections::HashMap;
use std::ops::Deref;
use std::time::Duration;
use clap::builder::Str;
use color_eyre::eyre::Result;
use hivemq_openapi::models::ClientDetails;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use serde_json::json;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::sleep;
use tui::Frame;
use crate::{hivemq_rest_client, tui};
use crate::action::Action;
use crate::cli::Cli;
use crate::components::Component;
use crate::components::home::Home;
use crate::components::tab_components::TabComponent;
use crate::hivemq_rest_client::{fetch_client_details, fetch_client_ids};


pub struct Clients {
    command_tx: Option<UnboundedSender<Action>>,
    state: ListState,
    client_ids: Vec<String>,
    client_details: HashMap<String, Result<String, String>>,
    is_loading_client_ids: bool,
    clients_loading_error: Option<String>
}

impl Clients {
    pub fn new() -> Self {
        Clients {
            command_tx: None,
            state: ListState::default(),
            client_ids: vec![],
            client_details: HashMap::new(),
            is_loading_client_ids: false,
            clients_loading_error: None
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i < self.client_ids.len() - 1 {
                    i + 1
                } else {
                    return;
                }
            }
            None => {
                if self.client_ids.is_empty() {
                    return;
                }
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
                if self.client_ids.is_empty() {
                    return;
                }
                0
            }
        };
        self.state.select(Some(i))
    }

    fn reset(&mut self) {
        self.state.select(None)
    }

    fn load_client_ids(&mut self) {
        let tx = self.command_tx.clone().unwrap();
        let handle = tokio::spawn(async move {
            tx.send(Action::ClientsLoading).expect("Could not send loading action"); // The action channel is expected to be open
            let client_ids = fetch_client_ids(String::from("http://localhost:8888")).await;

            match client_ids {
                Ok(ids) => {
                    tx.send(Action::ClientsLoaded(ids)).expect("Could not send loaded action");
                }
                Err(err) => {
                    tx.send(Action::ClientsLoadingFailed(err)).expect("Could not send loading failed action");
                }
            }
        });
    }

    fn load_client_details(&mut self, client_id: &String) {
        let tx = self.command_tx.clone().unwrap();
        let client_id = client_id.clone();
        let handle = tokio::spawn(async move {
            let client_details = fetch_client_details(client_id.clone(),String::from("http://localhost:8888")).await;

            match client_details {
                Ok(ref details) => {
                    tx.send(Action::ClientDetailsLoaded(client_id.clone(), serde_json::to_string_pretty(&client_details).unwrap())).expect("Could not send loaded action");
                }
                Err(err) => {
                    tx.send(Action::ClientDetailsLoadingFailed(client_id.clone(), err)).expect("Could not send client details loading failed action")
                }
            }
        });
    }
}

impl Component for Clients {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if !self.is_loading_client_ids {
            match action {
                Action::Reload => {
                    self.client_details.clear();
                    self.clients_loading_error = None;
                    self.load_client_ids();
                }
                Action::Up => {
                    self.prev();
                }
                Action::Down => {
                    self.next();
                }
                Action::Escape => {
                    self.reset()
                }
                Action::ClientsLoading => {
                    self.is_loading_client_ids = true;
                }
                Action::ClientDetailsLoaded(client_id, details) => {
                    self.client_details.insert(client_id, Ok(details));
                }
                Action::ClientDetailsLoadingFailed(client_id, err) => {
                    self.client_details.insert(client_id, Err(err));
                }
                _ => ()
            };
        } else {
            match action {
                Action::ClientsLoaded(items) => {
                    self.client_ids = items;
                    self.is_loading_client_ids = false;
                }
                Action::ClientsLoadingFailed(err) => {
                    self.clients_loading_error = Some(err);
                    self.is_loading_client_ids = false;
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

        if self.is_loading_client_ids {
            f.render_widget(Block::default().borders(Borders::ALL).title("Loading Clients..."), client_details_view[0]);
            f.render_widget(Block::default().borders(Borders::ALL).title("Details"), client_details_view[1]);
        } else if self.clients_loading_error.is_some() {
            let msg = self.clients_loading_error.clone().unwrap();
            let p = Paragraph::new(msg.as_str())
                .style(Style::default().fg(Color::Red))
                .wrap(Wrap { trim: true });
            f.render_widget(p.block(Block::default().borders(Borders::ALL).title("Loading Clients failed")), client_details_view[0]);
            f.render_widget(Block::default().borders(Borders::ALL).title("Details"), client_details_view[1]);
        } else {
            // client id list
            let mut list_items = vec![];
            for item in &self.client_ids {
                list_items.push(ListItem::new(item.as_str()));
            }
            let selected_client = match self.state.selected() {
                None => {
                    0
                }
                Some(selected) => {
                    selected + 1
                }
            };
            let total_clients = self.client_ids.len();
            let items = List::new(list_items)
                .block(Block::default().borders(Borders::ALL)
                .title(format!("Clients ({selected_client}/{total_clients})")))
                .highlight_style(
                    Style::default()
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                );

            f.render_stateful_widget(items, client_details_view[0], &mut self.state);

            // client details
            match self.state.selected() {
                None => {
                    f.render_widget(Block::default().borders(Borders::ALL).title("Details"), client_details_view[1]);
                }
                Some(selected) => {
                    match self.client_details.get(self.client_ids[selected].as_str()) {
                        None => {
                            self.load_client_details(&self.client_ids[selected].clone());
                            f.render_widget(Block::default().borders(Borders::ALL).title("Details"), client_details_view[1]);
                        }
                        Some(details) => {

                            match details {
                                Ok(details) => {
                                    let paragraph = Paragraph::new(details.as_str()).block(Block::default().borders(Borders::ALL).title("Details"));
                                    f.render_widget(paragraph, client_details_view[1]);
                                }
                                Err(err) => {
                                    let p = Paragraph::new(err.as_str())
                                        .style(Style::default().fg(Color::Red))
                                        .wrap(Wrap { trim: true });
                                    f.render_widget(p.block(Block::default().borders(Borders::ALL).title("Loading Client Details failed")), client_details_view[1]);
                                }
                            }

                        }
                    }
                }
            };
        }

        Ok(())
    }
}

impl TabComponent for Clients {
    fn get_name(&self) -> String {
        "Client Details".to_string()
    }
}