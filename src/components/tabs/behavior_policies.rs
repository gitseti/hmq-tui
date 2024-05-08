use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::BehaviorPolicy;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::{Action, Item},
    components::{
        Component, item_features::ItemSelector, list_with_details::ListWithDetails,
        tabs::TabComponent,
    }
    ,
    tui::Frame,
};
use crate::action::Action::{ItemCreated, ItemDeleted, ItemsLoadingFinished};
use crate::action::ListWithDetailsAction;
use crate::components::list_with_details::Features;
use crate::mode::Mode;
use crate::repository::Repository;
use crate::services::behavior_policy_service::BehaviorPolicyService;

pub struct BehaviorPoliciesTab<'a> {
    action_tx: UnboundedSender<Action>,
    list_with_details: ListWithDetails<'a, BehaviorPolicy>,
    service: Arc<BehaviorPolicyService>,
    item_name: &'static str,
}

pub struct BehaviorPolicySelector;

impl ItemSelector<BehaviorPolicy> for BehaviorPolicySelector {
    fn select(&self, item: Item) -> Option<BehaviorPolicy> {
        match item {
            Item::BehaviorPolicyItem(policy) => Some(policy),
            _ => None,
        }
    }

    fn select_with_id(&self, item: Item) -> Option<(String, BehaviorPolicy)> {
        self.select(item).map(|item| (item.id.clone(), item))
    }
}

impl BehaviorPoliciesTab<'_> {
    pub fn new(
        action_tx: UnboundedSender<Action>,
        hivemq_address: String,
        mode: Rc<RefCell<Mode>>,
    ) -> Self {
        let repository = Repository::<BehaviorPolicy>::init(&Pool::new(SqliteConnectionManager::memory()).unwrap(), "behavior_policies", |val| val.id.clone()).unwrap();
        let repository = Arc::new(repository);
        let service = Arc::new(BehaviorPolicyService::new(repository.clone(), &hivemq_address));
        let item_name = "Behavior Policy";
        let list_with_details = ListWithDetails::<BehaviorPolicy>::builder()
            .list_title("Behavior Policies")
            .item_name(item_name.clone())
            .hivemq_address(hivemq_address.clone())
            .mode(mode)
            .action_tx(action_tx.clone())
            .repository(repository.clone())
            .features(Features::builder().deletable().updatable().creatable().build())
            .build();
        BehaviorPoliciesTab {
            action_tx,
            list_with_details,
            service,
            item_name,
        }
    }
}

impl Component for BehaviorPoliciesTab<'_> {
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
                        let result = service.delete_behavior_policy(&item).await;
                        let action = ItemDeleted { item_name, result };
                        tx.send(action)
                            .expect("Behavior Policies: Failed to send ItemDeleted action");
                    });
                }
                ListWithDetailsAction::Create(item) => {
                    let service = self.service.clone();
                    let tx = self.action_tx.clone();
                    let item_name = String::from(self.item_name);
                    let _ = tokio::spawn(async move {
                        let result = service.create_behavior_policy(&item).await;
                        let action = ItemCreated { item_name, result };
                        tx.send(action)
                            .expect("Behavior Policies: Failed to send ItemCreated action");
                    });
                }
                ListWithDetailsAction::Update(item) => {
                    let service = self.service.clone();
                    let tx = self.action_tx.clone();
                    let item_name = String::from(self.item_name);
                    let _ = tokio::spawn(async move {
                        let result = service.update_behavior_policy(&item).await;
                        let action = ItemCreated { item_name, result };
                        tx.send(action)
                            .expect("Behavior Policies: Failed to send ItemCreated action");
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
                    let result = service.load_behavior_policies().await;
                    let action = ItemsLoadingFinished { item_name, result };
                    tx.send(action)
                        .expect("Behavior Policies: Failed to send ItemsLoadingFinished action");
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

impl TabComponent for BehaviorPoliciesTab<'_> {
    fn get_name(&self) -> &str {
        "Behavior Policies"
    }
}
