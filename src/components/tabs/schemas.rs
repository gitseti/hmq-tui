use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::Schema;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action::{ItemCreated, ItemDeleted, ItemsLoadingFinished};
use crate::action::ListWithDetailsAction;
use crate::components::list_with_details::Features;
use crate::mode::Mode;
use crate::repository::Repository;
use crate::services::schema_service::SchemaService;
use crate::{
    action::Action,
    components::{list_with_details::ListWithDetails, tabs::TabComponent, Component},
    tui::Frame,
};

pub struct SchemasTab<'a> {
    action_tx: UnboundedSender<Action>,
    list_with_details: ListWithDetails<'a, Schema>,
    service: Arc<SchemaService>,
    item_name: &'static str,
}

impl SchemasTab<'_> {
    pub fn new(
        action_tx: UnboundedSender<Action>,
        hivemq_address: String,
        mode: Rc<RefCell<Mode>>,
        sqlite_pool: &Pool<SqliteConnectionManager>,
    ) -> Self {
        let repository =
            Repository::<Schema>::init(sqlite_pool, "schemas", |val| val.id.clone()).unwrap();
        let repository = Arc::new(repository);
        let service = Arc::new(SchemaService::new(repository.clone(), &hivemq_address));
        let item_name = "Schema";
        let list_with_details = ListWithDetails::<Schema>::builder()
            .list_title("Schemas")
            .item_name(item_name)
            .mode(mode)
            .repository(repository)
            .features(Features::builder().creatable().deletable().build())
            .build();

        SchemasTab {
            action_tx,
            list_with_details,
            service,
            item_name,
        }
    }
}

impl Component for SchemasTab<'_> {
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
                        let result = service.delete_schema(&item).await;
                        let action = ItemDeleted { item_name, result };
                        tx.send(action)
                            .expect("Schemas: Failed to send ItemDeleted action");
                    });
                }
                ListWithDetailsAction::Create(item) => {
                    let service = self.service.clone();
                    let tx = self.action_tx.clone();
                    let item_name = String::from(self.item_name);
                    let _ = tokio::spawn(async move {
                        let result = service.create_schema(&item).await;
                        let action = ItemCreated { item_name, result };
                        tx.send(action)
                            .expect("Schemas: Failed to send ItemCreated action");
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
                    let result = service.load_schemas().await;
                    let action = ItemsLoadingFinished { item_name, result };
                    tx.send(action)
                        .expect("Schemas: Failed to send ItemsLoadingFinished action");
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

impl TabComponent for SchemasTab<'_> {
    fn get_name(&self) -> &str {
        "Schemas"
    }
}
