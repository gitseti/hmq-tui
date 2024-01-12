use crate::action::{Action, Item};
use crate::action::Action::Submit;
use crate::components::editor::Editor;
use crate::components::list_with_details::{ListWithDetails, State};
use crate::components::tabs::TabComponent;
use crate::components::{list_with_details, Component};
use crate::config::Config;
use crate::hivemq_rest_client::{create_script, delete_script, fetch_schemas, fetch_scripts};
use crate::mode::Mode;
use crate::mode::Mode::Main;
use crate::tui::Frame;
use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hivemq_openapi::models::{Backup, Schema, Script};
use libc::{creat, printf};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, ListItem, ListState, Paragraph, Wrap};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use crate::components::item_features::ItemSelector;

pub struct ScriptsTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    list_with_details: ListWithDetails<'a, Script>,
    new_item_editor: Option<Editor<'a>>,
}

struct ScriptSelector;

impl ItemSelector<Script> for ScriptSelector {
    fn select(&self, item: Item) -> Option<Script> {
        match item {
            Item::ScriptItem(script) => Some(script),
            _ => None
        }
    }

    fn select_with_id(&self, item: Item) -> Option<(String, Script)> {
        self.select(item).map(|item| (item.id.clone(), item))
    }
}

impl ScriptsTab<'_> {
    pub fn new(hivemq_address: String) -> Self {
        let list_with_details = ListWithDetails::<Script>::builder()
            .list_title("Scripts")
            .details_title("Script")
            .hivemq_address(hivemq_address.clone())
            .list_fn(Arc::new(fetch_scripts))
            .delete_fn(Arc::new(delete_script))
            .create_fn(Arc::new(create_script))
            .selector(Box::new(ScriptSelector))
            .build();
        ScriptsTab {
            hivemq_address,
            tx: None,
            list_with_details,
            new_item_editor: None,
        }
    }
}

impl Component for ScriptsTab<'_> {
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

impl TabComponent for ScriptsTab<'_> {
    fn get_name(&self) -> &str {
        "Scripts"
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
