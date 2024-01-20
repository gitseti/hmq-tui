use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    sync::Arc,
};

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::{Backup, Script, TraceRecording, TraceRecordingItem};
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, ListItem, ListState},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::{Action, Item},
    components::{
        item_features::ItemSelector,
        list_with_details,
        list_with_details::{ListWithDetails, State},
        tabs::TabComponent,
        Component,
    },
    config::Config,
    hivemq_rest_client::{
        delete_trace_recording, fetch_backups, fetch_schemas, fetch_trace_recordings,
    },
    tui::Frame,
};

pub struct TraceRecordingsTab<'a> {
    hivemq_address: String,
    tx: Option<UnboundedSender<Action>>,
    list_with_details: ListWithDetails<'a, TraceRecording>,
}

pub struct TraceRecordingSelector;

impl ItemSelector<TraceRecording> for TraceRecordingSelector {
    fn select(&self, item: Item) -> Option<TraceRecording> {
        match item {
            Item::TraceRecordingItem(trace_recording_item) => Some(trace_recording_item),
            _ => None,
        }
    }

    fn select_with_id(&self, item: Item) -> Option<(String, TraceRecording)> {
        let Some(item) = self.select(item) else {
            return None;
        };

        let Some(id) = &item.start_at else {
            return None;
        };

        Some((id.clone(), item))
    }
}

impl TraceRecordingsTab<'_> {
    pub fn new(hivemq_address: String) -> Self {
        let list_with_details = ListWithDetails::<TraceRecording>::builder()
            .list_title("Trace Recordings")
            .details_title("Trace Recording")
            .hivemq_address(hivemq_address.clone())
            .list_fn(Arc::new(fetch_trace_recordings))
            .delete_fn(Arc::new(delete_trace_recording))
            .selector(Box::new(TraceRecordingSelector))
            .build();
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
        vec![("R", "Load"), ("C", "Copy"), ("ESC", "Escape")]
    }
}
