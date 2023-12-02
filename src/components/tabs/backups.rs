use crate::action::Action;
use crate::components::list_with_details::{ListWithDetails, State};
use crate::components::tabs::TabComponent;
use crate::components::{list_with_details, Component};
use crate::config::Config;
use crate::hivemq_rest_client::fetch_backups;
use crate::tui::Frame;
use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::Backup;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, ListItem, ListState};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use tokio::sync::mpsc::UnboundedSender;

pub struct BackupsTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    list_with_details: ListWithDetails<'a, Backup>,
}
impl BackupsTab<'_> {
    pub fn new(hivemq_address: String) -> Self {
        BackupsTab {
            hivemq_address,
            tx: None,
            list_with_details: ListWithDetails::new("Backups".to_owned(), "Backup".to_owned()),
        }
    }
}

impl Component for BackupsTab<'_> {
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
                    let result = fetch_backups(hivemq_address).await;
                    tx.send(Action::BackupsLoadingFinished(result))
                        .expect("Failed to send backups loading finished action")
                });
            }
            Action::BackupsLoadingFinished(result) => match result {
                Ok(backups) => self.list_with_details.update_items(backups),
                Err(msg) => {
                    self.list_with_details.error(&msg);
                }
            },
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        self.list_with_details.draw(f, area).expect("panic");
        Ok(())
    }
}

impl TabComponent for BackupsTab<'_> {
    fn get_name(&self) -> &str {
        "Backups"
    }

    fn get_key_hints(&self) -> Vec<(&str, &str)> {
        vec![
            ("R", "Load"),
            ("C", "Copy JSON"),
            ("ESC", "Escape"),
        ]
    }
}