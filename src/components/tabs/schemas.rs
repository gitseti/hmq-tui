use std::sync::Arc;
use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::Schema;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::{Action, Item},
    components::{
        item_features::ItemSelector, list_with_details::ListWithDetails, tabs::TabComponent,
        Component,
    },
    hivemq_rest_client::{create_schema, delete_schema, fetch_schemas},
    tui::Frame,
};

pub struct SchemasTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    list_with_details: ListWithDetails<'a, Schema>,
}

pub struct SchemaSelector;

impl ItemSelector<Schema> for SchemaSelector {
    fn select(&self, item: Item) -> Option<Schema> {
        match item {
            Item::SchemaItem(schema) => Some(schema),
            _ => None,
        }
    }

    fn select_with_id(&self, item: Item) -> Option<(String, Schema)> {
        self.select(item).map(|item| (item.id.clone(), item))
    }
}

impl SchemasTab<'_> {
    pub fn new(hivemq_address: String) -> Self {
        let list_with_details = ListWithDetails::<Schema>::builder()
            .list_title("Schemas")
            .details_title("Schema")
            .hivemq_address(hivemq_address.clone())
            .list_fn(Arc::new(fetch_schemas))
            .delete_fn(Arc::new(delete_schema))
            .create_fn(Arc::new(create_schema))
            .selector(Box::new(SchemaSelector))
            .build();

        SchemasTab {
            hivemq_address,
            tx: None,
            list_with_details,
        }
    }
}

impl Component for SchemasTab<'_> {
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

impl TabComponent for SchemasTab<'_> {
    fn get_name(&self) -> &str {
        "Schemas"
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
