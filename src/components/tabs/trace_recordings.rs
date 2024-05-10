use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::TraceRecording;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action::{ItemCreated, ItemDeleted, ItemsLoadingFinished};
use crate::action::ListWithDetailsAction;
use crate::components::list_with_details::Features;
use crate::mode::Mode;
use crate::repository::Repository;
use crate::services::trace_recordings_service::TraceRecordingService;
use crate::{
    action::Action,
    components::{list_with_details::ListWithDetails, tabs::TabComponent, Component},
    tui::Frame,
};

pub struct TraceRecordingsTab<'a> {
    action_tx: UnboundedSender<Action>,
    list_with_details: ListWithDetails<'a, TraceRecording>,
    service: Arc<TraceRecordingService>,
    item_name: &'static str,
}

impl TraceRecordingsTab<'_> {
    pub fn new(
        action_tx: UnboundedSender<Action>,
        hivemq_address: String,
        mode: Rc<RefCell<Mode>>,
        sqlite_pool: &Pool<SqliteConnectionManager>,
    ) -> Self {
        let repository = Repository::<TraceRecording>::init(
            sqlite_pool,
            "trace_recordings",
            |val| val.name.clone().unwrap(),
            "startAt",
        )
        .unwrap();
        let repository = Arc::new(repository);
        let service = Arc::new(TraceRecordingService::new(
            repository.clone(),
            &hivemq_address,
        ));
        let item_name = "Trace Recording";
        let list_with_details = ListWithDetails::<TraceRecording>::builder()
            .list_title("Trace Recordings")
            .item_name(item_name)
            .mode(mode)
            .repository(repository.clone())
            .features(Features::builder().deletable().creatable().build())
            .build();
        TraceRecordingsTab {
            action_tx,
            list_with_details,
            service,
            item_name,
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
        if let Ok(Some(action)) = self.list_with_details.update(action.clone()) {
            let Action::LWD(lwd_action) = action else {
                return Ok(Some(action));
            };

            match lwd_action {
                ListWithDetailsAction::Delete(item) => {
                    let service = self.service.clone();
                    let tx = self.action_tx.clone();
                    let item_name = String::from(self.item_name);
                    let _ = tokio::spawn(async move {
                        let result = service.delete_trace_recording(&item).await;
                        let action = ItemDeleted { item_name, result };
                        tx.send(action)
                            .expect("Trace Recordings: Failed to send ItemDeleted action");
                    });
                }
                ListWithDetailsAction::Create(item) => {
                    let service = self.service.clone();
                    let tx = self.action_tx.clone();
                    let item_name = String::from(self.item_name);
                    let _ = tokio::spawn(async move {
                        let result = service.create_trace_recording(&item).await;
                        let action = ItemCreated { item_name, result };
                        tx.send(action)
                            .expect("Trace Recordings: Failed to send ItemCreated action");
                    });
                }
                _ => {}
            }
        }

        match action {
            Action::LoadAllItems => {
                let service = self.service.clone();
                let tx = self.action_tx.clone();
                let item_name = String::from(self.item_name);
                let _ = tokio::spawn(async move {
                    let result = service.load_trace_recordings().await;
                    let action = ItemsLoadingFinished { item_name, result };
                    tx.send(action)
                        .expect("Trace Recordings: Failed to send ItemsLoadingFinished action");
                });
            }
            _ => (),
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
