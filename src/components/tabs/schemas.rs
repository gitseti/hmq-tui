use crate::action::Action;
use crate::action::Action::Submit;
use crate::components::editor::Editor;
use crate::components::list_with_details::{ListWithDetails, State};
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
use hivemq_openapi::models::Schema;
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
    new_item_editor: Option<Editor<'a>>,
}

impl SchemasTab<'_> {
    pub fn new(hivemq_address: String) -> Self {
        let mut list_with_details = ListWithDetails::new("Schemas".to_owned(), "Schema".to_owned(), hivemq_address.clone());
        list_with_details.register_delete_fn(delete_schema);
        SchemasTab {
            hivemq_address,
            tx: None,
            list_with_details,
            new_item_editor: None,
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
                    let result = fetch_schemas(hivemq_address).await;
                    tx.send(Action::SchemasLoadingFinished(result))
                        .expect("Failed to send schemas loading finished action")
                });
                Ok(None)
            }
            Action::SchemasLoadingFinished(result) => {
                match result {
                    Ok(schemas) => self.list_with_details.update_items(schemas),
                    Err(msg) => {
                        self.list_with_details.error(&msg)
                    }
                }
                Ok(None)
            },
            Action::Escape => {
                if let Some(editor) = &mut self.new_item_editor {
                    self.new_item_editor = None;
                    Ok(Some(Action::SwitchMode(Mode::Main)))
                } else {
                    Ok(None)
                }
            }
            Action::NewItem => {
                self.list_with_details.unfocus();
                self.new_item_editor = Some(Editor::writeable("Create New Schema".to_owned()));
                Ok(Some(Action::SwitchMode(Mode::Editing)))
            }
            Action::Submit => {
                if let Some(editor) = &mut self.new_item_editor {
                    let text = editor.get_text();
                    let host = self.hivemq_address.clone();
                    let tx = self.tx.clone().unwrap();
                    tokio::spawn(async move {
                        let result = create_schema(host, text).await;
                        tx.send(Action::SchemaCreated(result)).unwrap();
                    });
                }
                Ok(None)
            }
            Action::SchemaCreated(result) => {
                self.new_item_editor = None;
                match result {
                    Ok(schema) => {
                        let schema_id = schema.id.clone();
                        self.list_with_details.put(schema_id.clone(), schema);
                        self.list_with_details.select_item(schema_id)
                    }
                    Err(error) => {
                        self.list_with_details
                            .details_error("Schema creation failed".to_owned(), error);
                    }
                }
                Ok(Some(Action::SwitchMode(Main)))
            }
            _ => Ok(None)
        }
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

impl TabComponent for SchemasTab<'_> {
    fn get_name(&self) -> &str {
        "Schemas"
    }

    fn get_key_hints(&self) -> Vec<(&str, &str)> {
        vec![
            ("R", "Load"),
            ("N", "New Schema"),
            ("D", "Delete Schema"),
            ("C", "Copy JSON"),
            ("CTRL + N", "Submit"),
            ("ESC", "Escape"),
        ]
    }
}