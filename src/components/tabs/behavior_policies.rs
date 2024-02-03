use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::BehaviorPolicy;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::mode::Mode;
use crate::{
    action::{Action, Item},
    components::{
        item_features::ItemSelector, list_with_details::ListWithDetails, tabs::TabComponent,
        Component,
    },
    hivemq_rest_client::{create_behavior_policy, delete_behavior_policy, fetch_behavior_policies},
    tui::Frame,
};
use crate::hivemq_rest_client::update_behavior_policy;

pub struct BehaviorPoliciesTab<'a> {
    action_tx: UnboundedSender<Action>,
    list_with_details: ListWithDetails<'a, BehaviorPolicy>,
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
    pub fn new(action_tx: UnboundedSender<Action>, hivemq_address: String, mode: Rc<RefCell<Mode>>) -> Self {
        let list_with_details = ListWithDetails::<BehaviorPolicy>::builder()
            .list_title("Behavior Policies")
            .item_name("Behavior Policy")
            .hivemq_address(hivemq_address.clone())
            .mode(mode)
            .action_tx(action_tx.clone())
            .list_fn(Arc::new(fetch_behavior_policies))
            .delete_fn(Arc::new(delete_behavior_policy))
            .create_fn(Arc::new(create_behavior_policy))
            .update_fn(Arc::new(update_behavior_policy))
            .item_selector(Box::new(BehaviorPolicySelector))
            .build();
        BehaviorPoliciesTab {
            action_tx,
            list_with_details,
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
        let list_action = self.list_with_details.update(action.clone());
        if let Ok(Some(action)) = list_action {
            return Ok(Some(action));
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
