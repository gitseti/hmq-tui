use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, ListItem, ListState};
use tokio::sync::mpsc::UnboundedSender;
use crate::action::Action;
use crate::components::{Component, views};
use crate::components::tab_components::TabComponent;
use crate::tui::Frame;
use color_eyre::eyre::Result;
use hivemq_openapi::models::DataPolicy;
use crate::components::views::{DetailsView, State};
use crate::config::Config;
use crate::hivemq_rest_client::fetch_data_policies;

pub struct Policies<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    details_view: DetailsView<'a, DataPolicy>,
}

impl Policies<'_> {
    pub fn new(hivemq_address: &String) -> Self {
        Policies {
            hivemq_address: hivemq_address.clone(),
            tx: None,
            details_view: DetailsView::new("Policies".to_string(), "Policy".to_string())
        }
    }
}

impl Component for Policies<'_> {
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
            Action::Reload => {
                self.details_view.loading();

                let tx = self.tx.clone().unwrap();
                let hivemq_address = self.hivemq_address.clone();
                let handle = tokio::spawn(async move {
                    let result = fetch_data_policies(hivemq_address).await;
                    tx.send(Action::DataPoliciesLoadingFinished(result)).expect("Failed to send data policies loading finished action")
                });
            },
            Action::DataPoliciesLoadingFinished(result) => {
                match result {
                    Ok(policies) => {
                        self.details_view.update_items(policies)
                    }
                    Err(msg) => {
                        self.details_view.error(&msg);
                    }
                }
            }
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        self.details_view.draw(f, area).expect("panic");
        Ok(())
    }
}

impl TabComponent for Policies<'_> {
    fn get_name(&self) -> &str {
        "D. Policies"
    }

    fn get_key_hints(&self) -> Vec<(&str, &str)> {
        vec![]
    }
}