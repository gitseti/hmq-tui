use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
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
use crate::components::tabs::TabComponent;
use crate::hivemq_rest_client::{fetch_client_details, fetch_client_ids};


pub struct Clients {
    tx: Option<UnboundedSender<Action>>,
    hivemq_address: String,
    selected_client: ListState,
    client_ids: Vec<String>,
    client_details: HashMap<String, Result<String, String>>,
    is_loading_client_ids: bool,
    client_ids_loading_error: Option<String>,
    is_focus_details: bool,
}

impl Clients {
    pub fn new(hivemq_address: String) -> Self {
        Clients {
            tx: None,
            hivemq_address,
            selected_client: ListState::default(),
            client_ids: Vec::new(),
            client_details: HashMap::new(),
            is_loading_client_ids: false,
            client_ids_loading_error: None,
            is_focus_details: false,
        }
    }

    fn next(&mut self) {
        let selected = match self.selected_client.selected() {
            None if !self.client_ids.is_empty() => 0,
            Some(i) if i + 1 < self.client_ids.len() => i + 1,
            _ => return
        };
        self.selected_client.select(Some(selected))
    }

    fn prev(&mut self) {
        let selected = match self.selected_client.selected() {
            Some(i) if i > 0 => i - 1,
            _ => return
        };
        self.selected_client.select(Some(selected))
    }

    fn reset(&mut self) {
        self.selected_client.select(None)
    }

    fn load_client_ids(&mut self) {
        self.client_details.clear();
        self.client_ids_loading_error = None;

        let tx = self.tx.clone().unwrap();
        let hivemq_address = self.hivemq_address.clone();
        let handle = tokio::spawn(async move {
            tx.send(Action::ClientIdsLoading).expect("Could not send loading action"); // The action channel is expected to be open
            let client_ids = fetch_client_ids(hivemq_address).await;



            match client_ids {
                Ok(ids) => {
                    tx.send(Action::ClientIdsLoaded { client_ids: ids} ).expect("Could not send loaded action");
                }
                Err(error) => {
                    tx.send(Action::ClientIdsLoadingFailed { error }).expect("Could not send loading failed action");
                }
            }
        });
    }

    fn client_ids_loaded(&mut self, client_ids: Vec<String>) {
        self.client_ids = client_ids;
        self.is_loading_client_ids = false;
    }

    fn client_ids_loading_failed(&mut self, err: String) {
        self.client_ids_loading_error = Some(err);
        self.is_loading_client_ids = false;
    }

    fn load_client_details(&mut self, client_id: &String) {
        let tx = self.tx.clone().unwrap();
        let hivemq_address = self.hivemq_address.clone();
        let client_id = client_id.clone();
        let handle = tokio::spawn(async move {
            let client_details = fetch_client_details(client_id.clone(), hivemq_address).await;

            match client_details {
                Ok(ref details) => {
                    tx.send(Action::ClientDetailsLoaded {
                        client_id: client_id.clone(),
                        details: serde_json::to_string_pretty(&client_details).unwrap()
                    }).expect("Could not send loaded action");
                }
                Err(err) => {
                    tx.send(Action::ClientDetailsLoadingFailed {
                        client_id: client_id.clone(),
                        error: err }
                    ).expect("Could not send client details loading failed action")
                }
            }
        });
    }

    fn focus(&mut self) {
        let selected = match self.selected_client.selected() {
            None => {
                return;
            }
            Some(selected) => {
                selected
            }
        };

        let result = match self.client_details.get(self.client_ids[selected].as_str()) {
            None => {
                return;
            }
            Some(result) => {
                result
            }
        };

        let details = match result {
            Ok(details) => {
                details
            }
            Err(_) => {
                return;
            }
        };

        self.is_focus_details = true;
    }

    fn unfocus(&mut self) {
        self.is_focus_details = false;
    }

    fn draw_loading_client_ids(&self, f: &mut Frame, layout: &Rc<[Rect]>) {
        f.render_widget(Block::default().borders(Borders::ALL).title("Loading Clients..."), layout[0]);
        f.render_widget(Block::default().borders(Borders::ALL).title("Details"), layout[1]);
    }

    fn draw_loading_error(&mut self, f: &mut Frame, layout: &Rc<[Rect]>) {
        let msg = self.client_ids_loading_error.clone().unwrap();
        let p = Paragraph::new(msg.as_str())
            .style(Style::default().fg(Color::Red))
            .wrap(Wrap { trim: true });
        f.render_widget(p.block(Block::default().borders(Borders::ALL).title("Loading Clients failed")), layout[0]);
        f.render_widget(Block::default().borders(Borders::ALL).title("Details"), layout[1]);
    }
}

impl Component for Clients {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if self.is_focus_details {
            match action {
                Action::Escape => {
                    self.unfocus();
                }
                Action::Left => {
                    self.unfocus();
                }
                _ => ()
            }
        } else if !self.is_loading_client_ids {
            match action {
                Action::Reload => {
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
                Action::ClientIdsLoading => {
                    self.is_loading_client_ids = true;
                }
                Action::ClientDetailsLoaded {client_id, details} => {
                    self.client_details.insert(client_id, Ok(details));
                }
                Action::ClientDetailsLoadingFailed {client_id, error} => {
                    self.client_details.insert(client_id, Err(error));
                }
                Action::Right => {
                    self.focus();
                }
                _ => ()
            };
        } else {
            match action {
                Action::ClientIdsLoaded { client_ids } => {
                    self.client_ids_loaded(client_ids);
                }
                Action::ClientIdsLoadingFailed { error } => {
                    self.client_ids_loading_failed(error);
                }
                _ => ()
            }
        }

        Ok(None)
    }


    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Ratio(1, 3),
                Constraint::Ratio(2, 3),
            ])
            .split(area);

        if self.is_loading_client_ids {
            self.draw_loading_client_ids(f, &layout);
        } else if self.client_ids_loading_error.is_some() {
            self.draw_loading_error(f, &layout);
        } else {
            let mut list_items = Vec::new();
            for item in &self.client_ids {
                list_items.push(ListItem::new(item.as_str()));
            }

            let selected_client = match self.selected_client.selected() {
                None => {
                    0
                }
                Some(selected) => {
                    selected + 1
                }
            };
            let total_clients = self.client_ids.len();
            let items = List::new(list_items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Clients ({selected_client}/{total_clients})")))
                .highlight_style(
                    Style::default()
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                );

            f.render_stateful_widget(items, layout[0], &mut self.selected_client);

            // client details
            match self.selected_client.selected() {
                None => {
                    f.render_widget(Block::default().borders(Borders::ALL).title("Details"), layout[1]);
                }
                Some(selected) => {
                    match self.client_details.get(self.client_ids[selected].as_str()) {
                        None => {
                            self.load_client_details(&self.client_ids[selected].clone());
                            f.render_widget(Block::default().borders(Borders::ALL).title("Details"), layout[1]);
                        }
                        Some(details) => {
                            match details {
                                Ok(details) => {
                                    let paragraph = Paragraph::new(details.as_str())
                                        .block(Block::default()
                                            .borders(Borders::ALL)
                                            .title("Details"));
                                    f.render_widget(paragraph, layout[1]);
                                }
                                Err(err) => {
                                    let p = Paragraph::new(err.as_str())
                                        .style(Style::default().fg(Color::Red))
                                        .wrap(Wrap { trim: true });
                                    f.render_widget(p.block(Block::default().borders(Borders::ALL).title("Loading Client Details failed")), layout[1]);
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
    fn get_name(&self) -> &str {
        "Clients"
    }

    fn get_key_hints(&self) -> Vec<(&str, &str)> {
        vec![("R", "Load")]
    }
}