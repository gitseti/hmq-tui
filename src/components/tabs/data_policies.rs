use crate::action::Action;
use crate::action::Action::Submit;
use crate::components::editor::Editor;
use crate::components::list_with_details::{ListWithDetails, State};
use crate::components::tabs::TabComponent;
use crate::components::{list_with_details, Component};
use crate::config::Config;
use crate::hivemq_rest_client::{create_data_policy, delete_data_policy, fetch_data_policies};
use crate::mode::Mode;
use crate::mode::Mode::Main;
use crate::tui::Frame;
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hivemq_openapi::models::DataPolicy;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, ListItem, ListState};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use tokio::sync::mpsc::UnboundedSender;

pub struct DataPoliciesTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    list_with_details: ListWithDetails<'a, DataPolicy>,
    new_item_editor: Option<Editor<'a>>,
}

impl DataPoliciesTab<'_> {
    pub fn new(hivemq_address: String) -> Self {
        let mut list_with_details = ListWithDetails::new("Policies".to_owned(), "Policy".to_owned(), hivemq_address.clone());
        list_with_details.register_delete_fn(delete_data_policy);
        DataPoliciesTab {
            hivemq_address,
            tx: None,
            list_with_details,
            new_item_editor: None,
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
            Action::LoadAllItems => {
                let tx = self.tx.clone().unwrap();
                let hivemq_address = self.hivemq_address.clone();
                let handle = tokio::spawn(async move {
                    let result = fetch_data_policies(hivemq_address).await;
                    tx.send(Action::DataPoliciesLoadingFinished(result))
                        .expect("Failed to send data policies loading finished action")
                });
            }
            Action::DataPoliciesLoadingFinished(result) => match result {
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
                self.new_item_editor = Some(Editor::writeable("Create New Data Policy".to_owned()));
                return Ok(Some(Action::SwitchMode(Mode::Editing)));
            }
            Action::Submit => {
                if let Some(editor) = &mut self.new_item_editor {
                    let text = editor.get_text();
                    let host = self.hivemq_address.clone();
                    let tx = self.tx.clone().unwrap();
                    tokio::spawn(async move {
                        let result = create_data_policy(host, text).await;
                        tx.send(Action::DataPolicyCreated(result)).unwrap();
                    });
                }
            }
            Action::DataPolicyCreated(result) => {
                self.new_item_editor = None;
                match result {
                    Ok(data_policy) => {
                        let id = data_policy.id.clone();
                        self.list_with_details.put(id.clone(), data_policy);
                        self.list_with_details.select_item(id)
                    }
                    Err(error) => {
                        self.list_with_details
                            .details_error("Data Policy creation failed".to_owned(), error);
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

impl TabComponent for DataPoliciesTab<'_> {
    fn get_name(&self) -> &str {
        "D. Policies"
    }

    fn get_key_hints(&self) -> Vec<(&str, &str)> {
        vec![
            ("R", "Load"),
            ("N", "New Policy"),
            ("D", "Delete Policy"),
            ("C", "Copy JSON"),
            ("CTRL + N", "Submit"),
            ("ESC", "Escape"),
        ]
    }
}
