use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::TraceRecording;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::hivemq_rest_client::create_trace_recording;
use crate::mode::Mode;
use crate::{
    action::{Action, Item},
    components::{
        item_features::ItemSelector, list_with_details::ListWithDetails, tabs::TabComponent,
        Component,
    },
    hivemq_rest_client::{delete_trace_recording, fetch_trace_recordings},
    tui::Frame,
};

pub struct TraceRecordingsTab<'a> {
    action_tx: UnboundedSender<Action>,
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
    pub fn new(
        action_tx: UnboundedSender<Action>,
        hivemq_address: String,
        mode: Rc<RefCell<Mode>>,
    ) -> Self {
        let list_with_details = ListWithDetails::<TraceRecording>::builder()
            .list_title("Trace Recordings")
            .item_name("Trace Recording")
            .hivemq_address(hivemq_address.clone())
            .mode(mode)
            .action_tx(action_tx.clone())
            .create_fn(Arc::new(create_trace_recording))
            .list_fn(Arc::new(fetch_trace_recordings))
            .delete_fn(Arc::new(delete_trace_recording))
            .item_selector(Box::new(TraceRecordingSelector))
            .build();
        TraceRecordingsTab {
            action_tx,
            list_with_details,
        }
    }
}

impl Component for TraceRecordingsTab<'_> {
    fn activate(&mut self) -> Result<()> {
        self.list_with_details.activate()
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
        self.list_with_details.draw(f, area).unwrap();
        Ok(())
    }
}

impl TabComponent for TraceRecordingsTab<'_> {
    fn get_name(&self) -> &str {
        "Trace Recordings"
    }
}
