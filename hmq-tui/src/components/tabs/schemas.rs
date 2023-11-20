use crate::action::Action;
use crate::components::tabs::TabComponent;
use crate::components::views::{DetailsView, State};
use crate::components::{views, Component};
use crate::config::Config;
use crate::hivemq_rest_client::fetch_schemas;
use crate::tui::Frame;
use color_eyre::eyre::Result;
use hivemq_openapi::models::Schema;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, ListItem, ListState};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use tokio::sync::mpsc::UnboundedSender;

pub struct SchemasTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    details_view: DetailsView<'a, Schema>,
}

impl SchemasTab<'_> {
    pub fn new(hivemq_address: String) -> Self {
        SchemasTab {
            hivemq_address: hivemq_address.clone(),
            tx: None,
            details_view: DetailsView::new("Schemas".to_owned(), "Schema".to_owned()),
        }
    }
}

impl Component for SchemasTab<'_> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Up => {
                self.details_view.prev_item();
            }
            Action::Down => {
                self.details_view.next_item();
            }
            Action::Copy => {
                self.details_view.copy_details_to_clipboard();
            }
            Action::Reload => {
                self.details_view.loading();

                let tx = self.tx.clone().unwrap();
                let hivemq_address = self.hivemq_address.clone();
                let handle = tokio::spawn(async move {
                    let result = fetch_schemas(hivemq_address).await;
                    tx.send(Action::SchemasLoadingFinished(result))
                        .expect("Failed to send schemas loading finished action")
                });
            }
            Action::SchemasLoadingFinished(result) => match result {
                Ok(schemas) => self.details_view.update_items(schemas),
                Err(msg) => {
                    self.details_view.error(&msg);
                }
            },
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        self.details_view.draw(f, area).expect("panic");
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
