use crate::action::{Action, Item};
use crate::components::list_with_details::{ListWithDetails, State};
use crate::components::tabs::TabComponent;
use crate::components::{list_with_details, Component};
use crate::config::Config;
use crate::hivemq_rest_client::{delete_schema, fetch_backups, fetch_schemas};
use crate::tui::Frame;
use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::{Backup, BehaviorPolicy, Schema};
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, ListItem, ListState};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use crate::components::item_features::{ItemSelector, ListFn};

pub struct BackupsTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    list_with_details: ListWithDetails<'a, Backup>,
}

struct BackupSelector;

impl ItemSelector<Backup> for BackupSelector {
    fn select(&self, item: Item) -> Option<Backup> {
        match item {
            Item::BackupItem(backup) => Some(backup),
            _ => None
        }
    }

    fn select_with_id(&self, item: Item) -> Option<(String, Backup)> {
        let Some(item) = self.select(item) else {
            return None;
        };

        let Some(id) = &item.id else {
            return None;
        };

        Some((id.clone(), item))
    }
}

impl BackupsTab<'_> {
    pub fn new(hivemq_address: String) -> Self {
        let list_with_details = ListWithDetails::<Backup>::builder()
            .list_title("Backups")
            .details_title("Backup")
            .hivemq_address(hivemq_address.clone())
            .list_fn(Arc::new(fetch_backups))
            .selector(Box::new(BackupSelector))
            .build();
        BackupsTab {
            hivemq_address: hivemq_address.clone(),
            tx: None,
            list_with_details,
        }
    }
}

impl Component for BackupsTab<'_> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.tx = Some(tx.clone());
        self.list_with_details.register_action_handler(tx)?;
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

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        self.list_with_details.draw(f, area)
    }
}

impl TabComponent for BackupsTab<'_> {
    fn get_name(&self) -> &str {
        "Backups"
    }

    fn get_key_hints(&self) -> Vec<(&str, &str)> {
        vec![("R", "Load"), ("C", "Copy JSON"), ("ESC", "Escape")]
    }
}
