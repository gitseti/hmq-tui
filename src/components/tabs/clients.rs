use crate::action::Action;
use crate::cli::Cli;
use crate::components::editor::Editor;
use crate::components::home::Home;
use crate::components::list_with_details::ListWithDetails;
use crate::components::tabs::TabComponent;
use crate::components::Component;
use crate::hivemq_rest_client::{fetch_client_details, fetch_client_ids};
use crate::mode::Mode;
use crate::{hivemq_rest_client, tui};
use clap::builder::Str;
use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::ClientDetails;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use serde_json::json;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::sleep;
use tui::Frame;

pub struct Clients<'a> {
    tx: Option<UnboundedSender<Action>>,
    hivemq_address: String,
    list_with_details: ListWithDetails<'a, Option<ClientDetails>>,
}

impl Clients<'_> {
    pub fn new(hivemq_address: String) -> Self {
        Clients {
            tx: None,
            hivemq_address: hivemq_address.clone(),
            list_with_details: ListWithDetails::new(
                "Clients".to_owned(),
                "Client Details".to_owned(),
                hivemq_address
            ),
        }
    }
}

impl Component for Clients<'_> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        self.list_with_details.handle_key_events(key)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let list_action = self.list_with_details.update(action.clone());
        match list_action {
            Ok(Some(Action::SelectedItem(key))) => {
                if let Some(None) = self.list_with_details.get(key.to_owned()) {
                    let tx = self.tx.clone().unwrap();
                    let hivemq_address = self.hivemq_address.clone();
                    let _ = tokio::spawn(async move {
                        let result = fetch_client_details(key, hivemq_address).await;
                        tx.send(Action::ClientDetailsLoadingFinished(result))
                            .expect("Failed to send client details loading finished action");
                    });
                }
            }
            Ok(Some(Action::SwitchMode(_))) => {
                return list_action;
            }
            _ => {}
        }

        match action {
            Action::LoadAllItems => {
                let tx = self.tx.clone().unwrap();
                let hivemq_address = self.hivemq_address.clone();
                let handle = tokio::spawn(async move {
                    let result = fetch_client_ids(hivemq_address).await;
                    tx.send(Action::ClientIdsLoadingFinished(result))
                        .expect("Failed to send client ids loading finished action");
                });
            }
            Action::ClientDetailsLoadingFinished(result) => match result {
                Ok((client_id, details)) => {
                    self.list_with_details.put(client_id, Some(details));
                }
                Err(msg) => {
                    self.list_with_details.error(&msg);
                }
            },
            Action::ClientIdsLoadingFinished(result) => match result {
                Ok(client_ids) => {
                    let mut details = Vec::with_capacity(client_ids.len());
                    for client_id in client_ids {
                        details.push((client_id, None));
                    }
                    self.list_with_details.update_items(details);
                }
                Err(msg) => {
                    self.list_with_details.error(&msg);
                }
            },
            _ => (),
        };

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        self.list_with_details.draw(f, area).unwrap();
        Ok(())
    }
}

impl TabComponent for Clients<'_> {
    fn get_name(&self) -> &str {
        "Clients"
    }

    fn get_key_hints(&self) -> Vec<(&str, &str)> {
        vec![("R", "Load"), ("C", "Copy JSON"), ("ESC", "Escape")]
    }
}
