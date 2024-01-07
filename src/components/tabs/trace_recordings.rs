use crate::action::Action;
use crate::components::list_with_details::{ListWithDetails, State};
use crate::components::tabs::TabComponent;
use crate::components::{list_with_details, Component};
use crate::config::Config;
use crate::hivemq_rest_client::{delete_trace_recording, fetch_backups, fetch_trace_recordings};
use crate::tui::Frame;
use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::TraceRecording;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, ListItem, ListState};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use tokio::sync::mpsc::UnboundedSender;

pub struct TraceRecordingsTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    list_with_details: ListWithDetails<'a, TraceRecording>,
}

impl TraceRecordingsTab<'_> {
    pub fn new(hivemq_address: String) -> Self {
        let mut list_with_details = ListWithDetails::new(
            "Trace Recordings".to_owned(),
            "Trace Recording".to_owned(),
            hivemq_address.clone()
        );
        list_with_details.register_delete_fn(delete_trace_recording);
        TraceRecordingsTab {
            hivemq_address,
            tx: None,
            list_with_details,
        }
    }
}

impl Component for TraceRecordingsTab<'_> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.tx = Some(tx.clone());
        self.list_with_details.register_action_handler(tx)?;
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        self.list_with_details.handle_key_events(key)
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
                    let result = fetch_trace_recordings(hivemq_address).await;
                    tx.send(Action::TraceRecordingsLoadingFinished(result))
                        .expect("Failed to send backups loading finished action")
                });
            }
            Action::TraceRecordingsLoadingFinished(result) => match result {
                Ok(backups) => self.list_with_details.update_items(backups),
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

impl TabComponent for TraceRecordingsTab<'_> {
    fn get_name(&self) -> &str {
        "Trace Recordings"
    }

    fn get_key_hints(&self) -> Vec<(&str, &str)> {
        vec![("R", "Load"), ("D", "Delete Trace Recording"), ("C", "Copy JSON"), ("ESC", "Escape")]
    }
}
