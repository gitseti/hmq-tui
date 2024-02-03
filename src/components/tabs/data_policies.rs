use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::DataPolicy;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::mode::Mode;
use crate::{
    action::{Action, Item},
    components::{
        item_features::ItemSelector, list_with_details::ListWithDetails, tabs::TabComponent,
        Component,
    },
    hivemq_rest_client::{
        create_data_policy, delete_data_policy, fetch_data_policies, update_data_policy,
    },
    tui::Frame,
};

pub struct DataPoliciesTab<'a> {
    action_tx: UnboundedSender<Action>,
    list_with_details: ListWithDetails<'a, DataPolicy>,
}

pub struct DataPolicySelector;

impl ItemSelector<DataPolicy> for DataPolicySelector {
    fn select(&self, item: Item) -> Option<DataPolicy> {
        match item {
            Item::DataPolicyItem(policy) => Some(policy),
            _ => None,
        }
    }

    fn select_with_id(&self, item: Item) -> Option<(String, DataPolicy)> {
        self.select(item).map(|item| (item.id.clone(), item))
    }
}

impl DataPoliciesTab<'_> {
    pub fn new(
        action_tx: UnboundedSender<Action>,
        hivemq_address: String,
        mode: Rc<RefCell<Mode>>,
    ) -> Self {
        let list_with_details = ListWithDetails::<DataPolicy>::builder()
            .list_title("Data Policies")
            .item_name("Data Policy")
            .hivemq_address(hivemq_address.clone())
            .mode(mode)
            .action_tx(action_tx.clone())
            .item_selector(Box::new(DataPolicySelector))
            .list_fn(Arc::new(fetch_data_policies))
            .delete_fn(Arc::new(delete_data_policy))
            .create_fn(Arc::new(create_data_policy))
            .update_fn(Arc::new(update_data_policy))
            .build();
        DataPoliciesTab {
            action_tx,
            list_with_details,
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

impl TabComponent for DataPoliciesTab<'_> {
    fn get_name(&self) -> &str {
        "Data Policies"
    }
}
