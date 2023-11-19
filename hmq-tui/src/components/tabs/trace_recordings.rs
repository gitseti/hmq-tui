use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, ListItem, ListState};
use tokio::sync::mpsc::UnboundedSender;
use crate::action::Action;
use crate::components::{Component, views};
use crate::tui::Frame;
use color_eyre::eyre::Result;
use hivemq_openapi::models::{TraceRecording};
use crate::components::tabs::TabComponent;
use crate::components::views::{DetailsView, State};
use crate::config::Config;
use crate::hivemq_rest_client::{fetch_backups, fetch_trace_recordings};

pub struct TraceRecordingsTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    details_view: DetailsView<'a, TraceRecording>,
}

impl TraceRecordingsTab<'_> {
    pub fn new(hivemq_address: String) -> Self {
        TraceRecordingsTab {
            hivemq_address,
            tx: None,
            details_view: DetailsView::new("Trace Recordings".to_string(), "Trace Recording".to_string())
        }
    }
}

impl Component for TraceRecordingsTab<'_> {
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
                    let result = fetch_trace_recordings(hivemq_address).await;
                    tx.send(Action::TraceRecordingsLoadingFinished(result)).expect("Failed to send backups loading finished action")
                });
            },
            Action::TraceRecordingsLoadingFinished(result) => {
                match result {
                    Ok(backups) => {
                        self.details_view.update_items(backups)
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

impl TabComponent for TraceRecordingsTab<'_> {
    fn get_name(&self) -> &str {
        "Trace Recordings"
    }

    fn get_key_hints(&self) -> Vec<(&str, &str)> {
        vec![("R", "Load"), ("C", "Copy")]
    }
}