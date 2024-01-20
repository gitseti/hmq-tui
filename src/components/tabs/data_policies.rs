use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    sync::Arc,
};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hivemq_openapi::models::{Backup, BehaviorPolicy, DataPolicy};
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, ListItem, ListState},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::{Action, Action::Submit, Item},
    components::{
        editor::Editor,
        item_features::ItemSelector,
        list_with_details,
        list_with_details::{ListWithDetails, State},
        tabs::TabComponent,
        Component,
    },
    config::Config,
    hivemq_rest_client::{
        create_data_policy, delete_data_policy, fetch_data_policies, fetch_schemas,
    },
    mode::{Mode, Mode::Main},
    tui::Frame,
};

pub struct DataPoliciesTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
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
    pub fn new(hivemq_address: String) -> Self {
        let list_with_details = ListWithDetails::<DataPolicy>::builder()
            .list_title("Data Policies")
            .details_title("Data Policy")
            .hivemq_address(hivemq_address.clone())
            .selector(Box::new(DataPolicySelector))
            .list_fn(Arc::new(fetch_data_policies))
            .delete_fn(Arc::new(delete_data_policy))
            .create_fn(Arc::new(create_data_policy))
            .build();
        DataPoliciesTab {
            hivemq_address,
            tx: None,
            list_with_details,
        }
    }
}

impl Component for DataPoliciesTab<'_> {
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

impl TabComponent for DataPoliciesTab<'_> {
    fn get_name(&self) -> &str {
        "Data Policies"
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
