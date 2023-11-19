use crate::action::Action;
use crate::components::tabs::TabComponent;
use crate::components::views::{DetailsView, State};
use crate::components::{views, Component};
use crate::config::Config;
use crate::hivemq_rest_client::fetch_behavior_policies;
use crate::tui::Frame;
use color_eyre::eyre::Result;
use hivemq_openapi::models::BehaviorPolicy;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, ListItem, ListState};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use tokio::sync::mpsc::UnboundedSender;

pub struct BehaviorPoliciesTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    details_view: DetailsView<'a, BehaviorPolicy>,
}

impl BehaviorPoliciesTab<'_> {
    pub fn new(hivemq_address: String) -> Self {
        BehaviorPoliciesTab {
            hivemq_address,
            tx: None,
            details_view: DetailsView::new("Policies".to_string(), "Policy".to_string()),
        }
    }
}

impl Component for BehaviorPoliciesTab<'_> {
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
                    let result = fetch_behavior_policies(hivemq_address).await;
                    tx.send(Action::BehaviorPoliciesLoadingFinished(result))
                        .expect("Failed to send behavior policies loading finished action")
                });
            }
            Action::BehaviorPoliciesLoadingFinished(result) => match result {
                Ok(policies) => self.details_view.update_items(policies),
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

impl TabComponent for BehaviorPoliciesTab<'_> {
    fn get_name(&self) -> &str {
        "B. Policies"
    }

    fn get_key_hints(&self) -> Vec<(&str, &str)> {
        vec![("R", "Load"), ("C", "Copy")]
    }
}
