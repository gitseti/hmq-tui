use crate::action::Action;
use crate::components::tabs::TabComponent;
use crate::components::list_with_details::{ListWithDetails, State};
use crate::components::{list_with_details, Component};
use crate::config::Config;
use crate::hivemq_rest_client::fetch_data_policies;
use crate::tui::Frame;
use color_eyre::eyre::Result;
use hivemq_openapi::models::DataPolicy;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, ListItem, ListState};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crossterm::event::KeyEvent;
use tokio::sync::mpsc::UnboundedSender;

pub struct DataPoliciesTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    list_with_details: ListWithDetails<'a, DataPolicy>,
}

impl DataPoliciesTab<'_> {
    pub fn new(hivemq_address: String) -> Self {
        DataPoliciesTab {
            hivemq_address,
            tx: None,
            list_with_details: ListWithDetails::new("Policies".to_owned(), "Policy".to_owned()),
        }
    }
}

impl Component for DataPoliciesTab<'_> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        self.list_with_details.send_key_event(key);
        Ok(None)
    }


    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Up => {
                self.list_with_details.prev_item();
            },
            Action::Down => {
                self.list_with_details.next_item();
            },
            Action::Copy => {
                self.list_with_details.copy_details_to_clipboard();
            }
            Action::Enter | Action::Right => {
                self.list_with_details.focus_on_details();
            },
            Action::Reload => {
                self.list_with_details.loading();

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
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        self.list_with_details.draw(f, area).expect("panic");
        Ok(())
    }
}

impl TabComponent for DataPoliciesTab<'_> {
    fn get_name(&self) -> &str {
        "D. Policies"
    }

    fn get_key_hints(&self) -> Vec<(&str, &str)> {
        vec![("R", "Load"), ("C", "Copy")]
    }
}
