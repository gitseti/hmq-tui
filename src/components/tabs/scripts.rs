use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::Script;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::mode::Mode;
use crate::{
    action::{Action, Item},
    components::{
        item_features::ItemSelector, list_with_details::ListWithDetails, tabs::TabComponent,
        Component,
    },
    hivemq_rest_client::{create_script, delete_script, fetch_scripts},
    tui::Frame,
};

pub struct ScriptsTab<'a> {
    action_tx: UnboundedSender<Action>,
    list_with_details: ListWithDetails<'a, Script>,
}

pub struct ScriptSelector;

impl ItemSelector<Script> for ScriptSelector {
    fn select(&self, item: Item) -> Option<Script> {
        match item {
            Item::ScriptItem(script) => Some(script),
            _ => None,
        }
    }

    fn select_with_id(&self, item: Item) -> Option<(String, Script)> {
        self.select(item).map(|item| (item.id.clone(), item))
    }
}

impl ScriptsTab<'_> {
    pub fn new(action_tx: UnboundedSender<Action>, hivemq_address: String, mode: Rc<RefCell<Mode>>) -> Self {
        let list_with_details = ListWithDetails::<Script>::builder()
            .list_title("Scripts")
            .item_name("Script")
            .hivemq_address(hivemq_address.clone())
            .mode(mode)
            .action_tx(action_tx.clone())
            .list_fn(Arc::new(fetch_scripts))
            .delete_fn(Arc::new(delete_script))
            .create_fn(Arc::new(create_script))
            .item_selector(Box::new(ScriptSelector))
            .build();
        ScriptsTab {
            action_tx,
            list_with_details,
        }
    }
}

impl Component for ScriptsTab<'_> {

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

impl TabComponent for ScriptsTab<'_> {
    fn get_name(&self) -> &str {
        "Scripts"
    }
}
