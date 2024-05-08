use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::DataPolicy;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action::{ItemCreated, ItemDeleted, ItemsLoadingFinished};
use crate::action::ListWithDetailsAction;
use crate::components::list_with_details::Features;
use crate::mode::Mode;
use crate::repository::Repository;
use crate::services::data_policy_service::DataPolicyService;
use crate::{
    action::Action,
    components::{list_with_details::ListWithDetails, tabs::TabComponent, Component},
    tui::Frame,
};

pub struct DataPoliciesTab<'a> {
    action_tx: UnboundedSender<Action>,
    list_with_details: ListWithDetails<'a, DataPolicy>,
    service: Arc<DataPolicyService>,
    item_name: &'static str,
}

impl DataPoliciesTab<'_> {
    pub fn new(
        action_tx: UnboundedSender<Action>,
        hivemq_address: String,
        mode: Rc<RefCell<Mode>>,
        sqlite_pool: &Pool<SqliteConnectionManager>,
    ) -> Self {
        let repository =
            Repository::<DataPolicy>::init(sqlite_pool, "data_policies", |val| val.id.clone())
                .unwrap();
        let repository = Arc::new(repository);
        let service = Arc::new(DataPolicyService::new(repository.clone(), &hivemq_address));
        let item_name = "Data Policy";
        let list_with_details = ListWithDetails::<DataPolicy>::builder()
            .list_title("Data Policies")
            .item_name("Data Policy")
            .mode(mode)
            .repository(repository.clone())
            .features(
                Features::builder()
                    .deletable()
                    .creatable()
                    .updatable()
                    .build(),
            )
            .build();
        DataPoliciesTab {
            action_tx,
            list_with_details,
            service,
            item_name,
        }
    }
}

impl Component for DataPoliciesTab<'_> {
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
                        let result = service.delete_data_policy(&item).await;
                        let action = ItemDeleted { item_name, result };
                        tx.send(action)
                            .expect("Data Policies: Failed to send ItemDeleted action");
                    });
                }
                ListWithDetailsAction::Create(item) => {
                    let service = self.service.clone();
                    let tx = self.action_tx.clone();
                    let item_name = String::from(self.item_name);
                    let _ = tokio::spawn(async move {
                        let result = service.create_data_policy(&item).await;
                        let action = ItemCreated { item_name, result };
                        tx.send(action)
                            .expect("Data Policies: Failed to send ItemCreated action");
                    });
                }
                ListWithDetailsAction::Update(item) => {
                    let service = self.service.clone();
                    let tx = self.action_tx.clone();
                    let item_name = String::from(self.item_name);
                    let _ = tokio::spawn(async move {
                        let result = service.update_data_policy(&item).await;
                        let action = ItemCreated { item_name, result };
                        tx.send(action)
                            .expect("Data Policies: Failed to send ItemCreated action");
                    });
                }
            }
        }

        match action {
            Action::LoadAllItems => {
                let service = self.service.clone();
                let tx = self.action_tx.clone();
                let item_name = String::from(self.item_name);
                let _ = tokio::spawn(async move {
                    let result = service.load_data_policies().await;
                    let action = ItemsLoadingFinished { item_name, result };
                    tx.send(action)
                        .expect("Data Policies: Failed to send ItemsLoadingFinished action");
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

impl TabComponent for DataPoliciesTab<'_> {
    fn get_name(&self) -> &str {
        "Data Policies"
    }
}
