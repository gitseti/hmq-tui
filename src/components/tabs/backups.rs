use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use futures::future::err;
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
use crate::components::popup::ErrorPopup;
use crate::hivemq_rest_client::start_backup;

pub struct BackupsTab<'a> {
    hivemq_address: String,
    action_tx: UnboundedSender<Action>,
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
    pub fn new(action_tx: UnboundedSender<Action>, hivemq_address: String, mode: Rc<RefCell<Mode>>) -> Self {
        let list_with_details = ListWithDetails::<Backup>::builder()
            .list_title("Backups")
            .item_name("Backup")
            .hivemq_address(hivemq_address.clone())
            .mode(mode)
            .base_mode(Mode::BackupTab)
            .action_tx(action_tx.clone())
            .list_fn(Arc::new(fetch_backups))
            .item_selector(Box::new(BackupSelector))
            .build();
        BackupsTab {
            hivemq_address,
            action_tx,
            list_with_details,
        }
    }
}

impl Component for BackupsTab<'_> {
    fn activate(&mut self) -> Result<()> {
        self.list_with_details.activate()
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        self.list_with_details.handle_key_events(key)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::StartBackup => {
                let tx = self.action_tx.clone();
                let hivemq_address = self.hivemq_address.clone();
                tokio::spawn(async move {
                    let result = start_backup(hivemq_address).await;
                    tx.send(Action::ItemCreated { result }).unwrap();
                });
            }
            _ => {
                let _ = self.list_with_details.update(action.clone());
            }
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
}
