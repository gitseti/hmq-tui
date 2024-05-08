use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::Script;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::mode::Mode;
use crate::{
    action::{Action, Item},
    components::{
        item_features::ItemSelector, list_with_details::ListWithDetails, tabs::TabComponent,
        Component,
    },
    hivemq_rest_client::{create_script, delete_script, fetch_scripts},
    tui::Frame,
};
use crate::action::Action::{ItemCreated, ItemDeleted, ItemsLoadingFinished};
use crate::action::ListWithDetailsAction;
use crate::components::list_with_details::Features;
use crate::repository::Repository;
use crate::services::scripts_service::ScriptService;

pub struct ScriptsTab<'a> {
    action_tx: UnboundedSender<Action>,
    list_with_details: ListWithDetails<'a, Script>,
    service: Arc<ScriptService>,
    item_name: &'static str,
}

pub struct ScriptSelector;

impl ItemSelector<Script> for ScriptSelector {
    fn select(&self, item: Item) -> Option<Script> {
        match item {
            Item::ScriptItem(script) => Some(script),
            _ => None,
        }
    }

    fn select_with_id(&self, item: Item) -> Option<(String, Script)> {
        self.select(item).map(|item| (item.id.clone(), item))
    }
}

impl ScriptsTab<'_> {
    pub fn new(
        action_tx: UnboundedSender<Action>,
        hivemq_address: String,
        mode: Rc<RefCell<Mode>>,
    ) -> Self {
        let repository = Repository::<Script>::init(&Pool::new(SqliteConnectionManager::memory()).unwrap(), "scripts", |val| val.id.clone()).unwrap();
        let repository = Arc::new(repository);
        let service = Arc::new(ScriptService::new(repository.clone(), &hivemq_address));
        let item_name = "Script";
        let list_with_details = ListWithDetails::<Script>::builder()
            .list_title("Scripts")
            .item_name("Script")
            .hivemq_address(hivemq_address.clone())
            .mode(mode)
            .action_tx(action_tx.clone())
            .repository(repository.clone())
            .features(Features::builder().creatable().deletable().build())
            .build();
        ScriptsTab {
            action_tx,
            list_with_details,
            service,
            item_name
        }
    }
}

impl Component for ScriptsTab<'_> {
    fn activate(&mut self) -> Result<()> {
        self.list_with_details.activate()
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        self.list_with_details.handle_key_events(key)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Ok(Some(action)) = self.list_with_details.update(action.clone()) {
            let Action::LWD(lwd_action) = action else {
                return Ok(Some(action));
            };

            match lwd_action {
                ListWithDetailsAction::Delete(item) => {
                    let service = self.service.clone();
                    let tx = self.action_tx.clone();
                    let item_name = String::from(self.item_name);
                    let _ = tokio::spawn(async move {
                        let result = service.delete_script(&item).await;
                        let action = ItemDeleted { item_name, result };
                        tx.send(action)
                            .expect("Scripts: Failed to send ItemDeleted action");
                    });
                }
                ListWithDetailsAction::Create(item) => {
                    let service = self.service.clone();
                    let tx = self.action_tx.clone();
                    let item_name = String::from(self.item_name);
                    let _ = tokio::spawn(async move {
                        let result = service.create_script(&item).await;
                        let action = ItemCreated { item_name, result };
                        tx.send(action)
                            .expect("Scripts: Failed to send ItemCreated action");
                    });
                }
                _ => {}
            }
        }

        match action {
            Action::LoadAllItems => {
                let service = self.service.clone();
                let tx = self.action_tx.clone();
                let item_name = String::from(self.item_name);
                let _ = tokio::spawn(async move {
                    let result = service.load_scripts().await;
                    let action = ItemsLoadingFinished { item_name, result };
                    tx.send(action)
                        .expect("Scripts: Failed to send ItemsLoadingFinished action");
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

impl TabComponent for ScriptsTab<'_> {
    fn get_name(&self) -> &str {
        "Scripts"
    }
}
