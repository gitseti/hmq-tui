use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::Backup;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::{Action, Item}, components::{
    Component, item_features::ItemSelector, list_with_details::ListWithDetails,
    tabs::TabComponent,
}, tui::Frame};
use crate::components::list_with_details::Features;
use crate::mode::Mode;
use crate::repository::Repository;
use crate::services::backups_service::BackupService;

pub struct BackupsTab<'a> {
    hivemq_address: String,
    action_tx: UnboundedSender<Action>,
    list_with_details: ListWithDetails<'a, Backup>,
    service: Arc<BackupService>,
    item_name: &'static str,
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
    pub fn new(
        action_tx: UnboundedSender<Action>,
        hivemq_address: String,
        mode: Rc<RefCell<Mode>>,
    ) -> Self {
        let repository = Repository::<Backup>::init(&Pool::new(SqliteConnectionManager::memory()).unwrap(), "backups", |val| val.id.clone().unwrap()).unwrap();
        let repository = Arc::new(repository);
        let service = Arc::new(BackupService::new(repository.clone(), &hivemq_address));
        let item_name = "Backup";
        let list_with_details = ListWithDetails::<Backup>::builder()
            .list_title("Backups")
            .item_name(item_name)
            .hivemq_address(hivemq_address.clone())
            .mode(mode)
            .base_mode(Mode::BackupTab)
            .action_tx(action_tx.clone())
            .repository(repository.clone())
            .features(Features::builder().build())
            .build();
        BackupsTab {
            hivemq_address,
            action_tx,
            list_with_details,
            service,
            item_name,
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
        let _ = self.list_with_details.update(action.clone());

        match action {
            Action::StartBackup => {
                let service = self.service.clone();
                let tx = self.action_tx.clone();
                let item_name = self.item_name.clone().to_string();
                tokio::spawn(async move {
                    let result = service.start_backup().await;
                    tx.send(Action::ItemCreated { item_name, result }).unwrap();
                });
            }
            Action::LoadAllItems => {
                let service = self.service.clone();
                let tx = self.action_tx.clone();
                let item_name = self.item_name.clone().to_string();
                tokio::spawn(async move {
                    let result = service.load_backups().await;
                    tx.send(Action::ItemsLoadingFinished { item_name, result }).unwrap();
                });
            }
            _ => ()
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
