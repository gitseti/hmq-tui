use crate::action::Action;
use crate::components::list_with_details::{ListWithDetails, State};
use crate::components::tabs::TabComponent;
use crate::components::{list_with_details, Component};
use crate::config::Config;
use crate::hivemq_rest_client::{create_behavior_policy, fetch_behavior_policies};
use crate::tui::Frame;
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hivemq_openapi::models::BehaviorPolicy;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, ListItem, ListState};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use tokio::sync::mpsc::UnboundedSender;
use crate::components::editor::Editor;
use crate::mode::Mode;
use crate::mode::Mode::Main;

pub struct BehaviorPoliciesTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    list_with_details: ListWithDetails<'a, BehaviorPolicy>,
    new_item_editor: Option<Editor<'a>>,
}

impl BehaviorPoliciesTab<'_> {
    pub fn new(hivemq_address: String) -> Self {
        BehaviorPoliciesTab {
            hivemq_address,
            tx: None,
            list_with_details: ListWithDetails::new("Policies".to_owned(), "Policy".to_owned()),
            new_item_editor: None
        }
    }
}

impl Component for BehaviorPoliciesTab<'_> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if let Some(editor) = &mut self.new_item_editor {
            if KeyCode::Enter == key.code && key.modifiers == KeyModifiers::ALT {
                return Ok(None);
            }
            editor.handle_key_events(key)
        } else {
            self.list_with_details.handle_key_events(key)
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let list_action = self.list_with_details.update(action.clone());
        match list_action {
            Ok(Some(Action::SwitchMode(_))) => {
                return list_action;
            }
            _ => {}
        }

        match action {
            Action::LoadAllItems => {
                let tx = self.tx.clone().unwrap();
                let hivemq_address = self.hivemq_address.clone();
                let handle = tokio::spawn(async move {
                    let result = fetch_behavior_policies(hivemq_address).await;
                    tx.send(Action::BehaviorPoliciesLoadingFinished(result))
                        .expect("Failed to send behavior policies loading finished action")
                });
            }
            Action::BehaviorPoliciesLoadingFinished(result) => match result {
                Ok(policies) => self.list_with_details.update_items(policies),
                Err(msg) => {
                    self.list_with_details.error(&msg);
                }
            },
            Action::Escape => {
                if let Some(editor) = &mut self.new_item_editor {
                    self.new_item_editor = None;
                    return Ok(Some(Action::SwitchMode(Mode::Main)));
                }
            }
            Action::NewItem => {
                self.list_with_details.unfocus();
                self.new_item_editor = Some(Editor::writeable("Create New Behavior Policy".to_owned()));
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
        vec![("R", "Load"), ("C", "Copy")]
    }
}
