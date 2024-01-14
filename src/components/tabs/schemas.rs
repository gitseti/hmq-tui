use crate::action::Action::Submit;
use crate::action::{Action, Item};
use crate::components::editor::Editor;
use crate::components::item_features::{ItemSelector, ListFn};
use crate::components::list_with_details::{ListWithDetails, ListWithDetailsBuilder, State};
use crate::components::tabs::TabComponent;
use crate::components::{list_with_details, Component};
use crate::config::Config;
use crate::hivemq_rest_client::{create_schema, create_script, delete_schema, fetch_schemas};
use crate::mode::Mode;
use crate::mode::Mode::Main;
use crate::tui::Frame;
use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hivemq_openapi::models::{DataPolicy, Schema};
use libc::printf;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, ListItem, ListState, Paragraph, Wrap};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{format, Display, Formatter};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub struct SchemasTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    list_with_details: ListWithDetails<'a, Schema>,
}

struct SchemaSelector;

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
