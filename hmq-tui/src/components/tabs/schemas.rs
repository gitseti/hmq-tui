use crate::action::Action;
use crate::components::tabs::TabComponent;
use crate::components::list_with_details::{ListWithDetails, State};
use crate::components::{list_with_details, Component};
use crate::config::Config;
use crate::hivemq_rest_client::{create_schema, fetch_schemas};
use crate::tui::Frame;
use color_eyre::eyre::Result;
use hivemq_openapi::models::Schema;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, ListItem, ListState, Paragraph, Wrap};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use tokio::sync::mpsc::UnboundedSender;
use crate::components::editor::Editor;
use crate::mode::Mode;

pub struct SchemasTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    list_with_details: ListWithDetails<'a, Schema>,
    create_editor: Option<Editor<'a>>,
}

impl SchemasTab<'_> {
    pub fn new(hivemq_address: String) -> Self {
        SchemasTab {
            hivemq_address: hivemq_address.clone(),
            tx: None,
            list_with_details: ListWithDetails::new("Schemas".to_owned(), "Schema".to_owned()),
            create_editor: None,
        }
    }
}

impl Component for SchemasTab<'_> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if let Some(editor) = &mut self.create_editor {
            if KeyCode::Enter == key.code && key.modifiers == KeyModifiers::ALT {
                return Ok(None)
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
                    let result = fetch_schemas(hivemq_address).await;
                    tx.send(Action::SchemasLoadingFinished(result))
                        .expect("Failed to send schemas loading finished action")
                });
            }
            Action::Escape => {
                if let Some(editor) = &mut self.create_editor {
                    self.create_editor = None;
                    return Ok(Some(Action::SwitchMode(Mode::Main)));
                }
            }
            Action::NewItem => {
                self.create_editor = Some(Editor::writeable("Create New Schema".to_owned()));
                return Ok(Some(Action::SwitchMode(Mode::Editing)));
            }
            Action::Submit => {
                if let Some(editor) = &mut self.create_editor{
                    let text = editor.get_text();
                    let host = self.hivemq_address.clone();
                    let tx = self.tx.clone().unwrap();
                    tokio::spawn(async move {
                        let result = create_schema(host, text).await;
                        tx.send(Action::SchemaCreated(result)).unwrap();
                    });
                }
            }
            Action::SchemaCreated(result) => {
                match result {
                    Ok(schema) => {
                        self.list_with_details.put(schema.id.to_owned(), schema);
                    }
                    Err(error) => {
                        self.list_with_details.details_error("Schema creation failed".to_owned(), error);
                    }
                }
                return Ok(Some(Action::Escape));
            }
            Action::SchemasLoadingFinished(result) => match result {
                Ok(schemas) => self.list_with_details.update_items(schemas),
                Err(msg) => {
                    self.list_with_details.error(&msg);
                }
            },
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
            .split(area);

        let list_layout = layout[0];
        let editor_layout = layout[1];

        if let Some(editor) = &mut self.create_editor {
            self.list_with_details.draw_list(f, list_layout, true);
            editor.draw(f, editor_layout).unwrap();
        } else {
            self.list_with_details.draw(f, area).unwrap();
        }

        Ok(())
    }
}

impl TabComponent for SchemasTab<'_> {
    fn get_name(&self) -> &str {
        "Schemas"
    }

    fn get_key_hints(&self) -> Vec<(&str, &str)> {
        vec![("R", "Load"), ("C", "Copy")]
    }
}
