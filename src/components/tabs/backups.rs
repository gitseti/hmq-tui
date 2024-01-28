use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::Backup;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::mode::Mode;
use crate::{
    action::{Action, Item},
    components::{
        item_features::ItemSelector, list_with_details::ListWithDetails, tabs::TabComponent,
        Component,
    },
    hivemq_rest_client::fetch_backups,
    tui::Frame,
};

pub struct BackupsTab<'a> {
    tx: Option<UnboundedSender<Action>>,
    list_with_details: ListWithDetails<'a, Backup>,
}

pub struct BackupSelector;

impl ItemSelector<Backup> for BackupSelector {
    fn select(&self, item: Item) -> Option<Backup> {
        match item {
            Item::BackupItem(backup) => Some(backup),
            _ => None,
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
    pub fn new(hivemq_address: String, mode: Rc<RefCell<Mode>>) -> Self {
        let list_with_details = ListWithDetails::<Backup>::builder()
            .list_title("Backups")
            .details_title("Backup")
            .hivemq_address(hivemq_address.clone())
            .mode(mode)
            .list_fn(Arc::new(fetch_backups))
            .selector(Box::new(BackupSelector))
            .build();
        BackupsTab {
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
    fn activate(&mut self) -> Result<()> {
        self.list_with_details.activate()
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        self.list_with_details.handle_key_events(key)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let _ = self.list_with_details.update(action.clone());
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
}
