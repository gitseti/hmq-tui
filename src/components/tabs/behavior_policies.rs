use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::BehaviorPolicy;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::{Action, Item},
    components::{
        item_features::ItemSelector, list_with_details::ListWithDetails, tabs::TabComponent,
        Component,
    },
    hivemq_rest_client::{create_behavior_policy, delete_behavior_policy, fetch_behavior_policies},
    tui::Frame,
};

pub struct BehaviorPoliciesTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
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
    pub fn new(hivemq_address: String) -> Self {
        let list_with_details = ListWithDetails::<BehaviorPolicy>::builder()
            .list_title("Behavior Policies")
            .details_title("Behavior Policy")
            .hivemq_address(hivemq_address.clone())
            .list_fn(Arc::new(fetch_behavior_policies))
            .delete_fn(Arc::new(delete_behavior_policy))
            .create_fn(Arc::new(create_behavior_policy))
            .selector(Box::new(BehaviorPolicySelector))
            .build();
        BehaviorPoliciesTab {
            hivemq_address,
            tx: None,
            list_with_details,
        }
    }
}

impl Component for BehaviorPoliciesTab<'_> {
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

    fn get_key_hints(&self) -> Vec<(&str, &str)> {
        vec![
            ("R", "Load"),
            ("N", "New"),
            ("D", "Delete"),
            ("C", "Copy"),
            ("CTRL + N", "Submit"),
            ("ESC", "Escape"),
        ]
    }
}
