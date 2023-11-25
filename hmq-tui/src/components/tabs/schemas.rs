use crate::action::Action;
use crate::components::tabs::TabComponent;
use crate::components::list_with_details::{ListWithDetails, State};
use crate::components::{list_with_details, Component};
use crate::config::Config;
use crate::hivemq_rest_client::fetch_schemas;
use crate::tui::Frame;
use color_eyre::eyre::Result;
use hivemq_openapi::models::Schema;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, ListItem, ListState};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crossterm::event::KeyEvent;
use tokio::sync::mpsc::UnboundedSender;

pub struct SchemasTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    list_with_details: ListWithDetails<'a, Schema>,
}

impl SchemasTab<'_> {
    pub fn new(hivemq_address: String) -> Self {
        SchemasTab {
            hivemq_address: hivemq_address.clone(),
            tx: None,
            list_with_details: ListWithDetails::new("Schemas".to_owned(), "Schema".to_owned()),
        }
    }
}

impl Component for SchemasTab<'_> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        self.list_with_details.handle_key_events(key)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let _ = self.list_with_details.update(action.clone());
        match action {
            Action::LoadAllItems => {
                let tx = self.tx.clone().unwrap();
                let hivemq_address = self.hivemq_address.clone();
                let handle = tokio::spawn(async move {
                    let result = fetch_schemas(hivemq_address).await;
                    tx.send(Action::SchemasLoadingFinished(result))
                        .expect("Failed to send schemas loading finished action")
                });
            }
            Action::SchemasLoadingFinished(result) => match result {
                Ok(schemas) => self.list_with_details.update_items(schemas),
                Err(msg) => {
                    self.list_with_details.error(&msg);
                }
            },
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        self.list_with_details.draw(f, area).unwrap();
        Ok(())
    }
}

impl TabComponent for SchemasTab<'_> {
    fn get_name(&self) -> &str {
        "Schemas"
    }

    fn get_key_hints(&self) -> Vec<(&str, &str)> {
        vec![("R", "Load"), ("C", "Copy")]
    }
}
