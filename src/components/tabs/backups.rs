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

use crate::components::list_with_details::Features;
use crate::mode::Mode;
use crate::repository::Repository;
use crate::services::backups_service::BackupService;
use crate::{
    action::Action,
    components::{list_with_details::ListWithDetails, tabs::TabComponent, Component},
    tui::Frame,
};

pub struct BackupsTab<'a> {
    action_tx: UnboundedSender<Action>,
    list_with_details: ListWithDetails<'a, Backup>,
    service: Arc<BackupService>,
    item_name: &'static str,
}

impl BackupsTab<'_> {
    pub fn new(
        action_tx: UnboundedSender<Action>,
        hivemq_address: String,
        mode: Rc<RefCell<Mode>>,
        sqlite_pool: &Pool<SqliteConnectionManager>,
    ) -> Self {
        let repository = Repository::<Backup>::init(
            sqlite_pool,
            "backups",
            |val| val.id.clone().unwrap(),
            "createdAt",
        )
        .unwrap();
        let repository = Arc::new(repository);
        let service = Arc::new(BackupService::new(repository.clone(), &hivemq_address));
        let item_name = "Backup";
        let list_with_details = ListWithDetails::<Backup>::builder()
            .list_title("Backups")
            .item_name(item_name)
            .mode(mode)
            .base_mode(Mode::BackupTab)
            .repository(repository.clone())
            .features(Features::builder().build())
            .build();
        BackupsTab {
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
                let item_name = self.item_name.to_string();
                tokio::spawn(async move {
                    let result = service.start_backup().await;
                    tx.send(Action::ItemCreated { item_name, result }).unwrap();
                });
            }
            Action::LoadAllItems => {
                let service = self.service.clone();
                let tx = self.action_tx.clone();
                let item_name = self.item_name.to_string();
                tokio::spawn(async move {
                    let result = service.load_backups().await;
                    tx.send(Action::ItemsLoadingFinished { item_name, result })
                        .unwrap();
                });
            }
            _ => (),
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
