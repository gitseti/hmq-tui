use crate::action::{Action, Item};
use crate::action::Action::Submit;
use crate::components::editor::Editor;
use crate::components::list_with_details::{ListWithDetails, State};
use crate::components::tabs::TabComponent;
use crate::components::{list_with_details, Component};
use crate::config::Config;
use crate::hivemq_rest_client::{create_behavior_policy, delete_behavior_policy, fetch_behavior_policies, fetch_schemas};
use crate::mode::Mode;
use crate::mode::Mode::Main;
use crate::tui::Frame;
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hivemq_openapi::models::{Backup, BehaviorPolicy};
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, ListItem, ListState};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub struct BehaviorPoliciesTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    list_with_details: ListWithDetails<'a, BehaviorPolicy>,
    new_item_editor: Option<Editor<'a>>,
}

impl BehaviorPoliciesTab<'_> {
    pub fn new(hivemq_address: String) -> Self {
        let list_with_details = ListWithDetails::<BehaviorPolicy>::builder()
            .list_title("B. Policies")
            .details_title("B. Policy")
            .hivemq_address(hivemq_address.clone())
            .list_fn(Arc::new(fetch_behavior_policies))
            .delete_fn(Arc::new(delete_behavior_policy))
            .item_selector(|item| {
                match item {
                    Item::BehaviorPolicyItem(policy) => Some(policy),
                    _ => None
                }
            })
            .build();
        BehaviorPoliciesTab {
            hivemq_address,
            tx: None,
            list_with_details,
            new_item_editor: None,
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
        if let Some(editor) = &mut self.new_item_editor {
            if KeyCode::Char('n') == key.code && key.modifiers == KeyModifiers::CONTROL {
                return Ok(Some(Submit));
            }
            editor.handle_key_events(key)
        } else {
            self.list_with_details.handle_key_events(key)
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let list_action = self.list_with_details.update(action.clone());
        if let Ok(Some(action)) = list_action {
            return Ok(Some(action));
        }

        match action {
            Action::Escape => {
                if let Some(editor) = &mut self.new_item_editor {
                    self.new_item_editor = None;
                    return Ok(Some(Action::SwitchMode(Mode::Main)));
                }
            }
            Action::NewItem => {
                self.list_with_details.unfocus();
                self.new_item_editor =
                    Some(Editor::writeable("Create New Behavior Policy".to_owned()));
                return Ok(Some(Action::SwitchMode(Mode::Editing)));
            }
            Action::Submit => {
                if let Some(editor) = &mut self.new_item_editor {
                    let text = editor.get_text();
                    let host = self.hivemq_address.clone();
                    let tx = self.tx.clone().unwrap();
                    tokio::spawn(async move {
                        let result = create_behavior_policy(host, text).await;
                        tx.send(Action::BehaviorPolicyCreated(result)).unwrap();
                    });
                }
            }
            Action::BehaviorPolicyCreated(result) => {
                self.new_item_editor = None;
                match result {
                    Ok(behavior_policy) => {
                        let id = behavior_policy.id.clone();
                        self.list_with_details.put(id.clone(), behavior_policy);
                        self.list_with_details.select_item(id)
                    }
                    Err(error) => {
                        self.list_with_details
                            .details_error("Behavior Policy creation failed".to_owned(), error);
                    }
                }
                return Ok(Some(Action::SwitchMode(Main)));
            }
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let component = self
            .new_item_editor
            .as_mut()
            .map(|x| x as &mut dyn Component);
        self.list_with_details.draw_custom(f, area, component)?;
        Ok(())
    }
}

impl TabComponent for BehaviorPoliciesTab<'_> {
    fn get_name(&self) -> &str {
        "B. Policies"
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
