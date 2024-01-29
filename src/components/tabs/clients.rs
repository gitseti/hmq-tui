use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::ClientDetails;
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::sync::mpsc::UnboundedSender;
use tui::Frame;

use crate::mode::Mode;
use crate::{
    action::{Action, Item},
    components::{
        item_features::ItemSelector, list_with_details::ListWithDetails, tabs::TabComponent,
        Component,
    },
    hivemq_rest_client::{fetch_client_details, fetch_client_ids},
    tui,
};

pub struct Clients<'a> {
    action_tx: UnboundedSender<Action>,
    hivemq_address: String,
    list_with_details: ListWithDetails<'a, Option<ClientDetails>>,
}

pub struct ClientSelector;

impl ItemSelector<Option<ClientDetails>> for ClientSelector {
    fn select(&self, item: Item) -> Option<Option<ClientDetails>> {
        match item {
            Item::ClientItem(client) => Some(Some(client)),
            _ => None,
        }
    }

    fn select_with_id(&self, item: Item) -> Option<(String, Option<ClientDetails>)> {
        let Some(item) = self.select(item) else {
            return None;
        };

        let Some(details) = &item else {
            return None;
        };

        let Some(id) = &details.id else {
            return None;
        };

        Some((id.clone(), item))
    }
}

impl Clients<'_> {
    pub fn new(action_tx: UnboundedSender<Action>, hivemq_address: String, mode: Rc<RefCell<Mode>>) -> Self {
        let list_with_details = ListWithDetails::<Option<ClientDetails>>::builder()
            .list_title("Clients")
            .details_title("Client Details")
            .hivemq_address(hivemq_address.clone())
            .mode(mode)
            .action_tx(action_tx.clone())
            .selector(Box::new(ClientSelector))
            .build();
        Clients {
            action_tx,
            hivemq_address,
            list_with_details,
        }
    }
}

impl Component for Clients<'_> {

    fn activate(&mut self) -> Result<()> {
        self.list_with_details.activate()
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        self.list_with_details.handle_key_events(key)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let list_action = self.list_with_details.update(action.clone());
        match list_action {
            Ok(Some(Action::SelectedItem(key))) => {
                if let Some(None) = self.list_with_details.get(key.to_owned()) {
                    let tx = self.action_tx.clone();
                    let hivemq_address = self.hivemq_address.clone();
                    let _ = tokio::spawn(async move {
                        let result = fetch_client_details(&key, hivemq_address).await;
                        tx.send(Action::ClientDetailsLoadingFinished(result))
                            .expect("Failed to send client details loading finished action");
                    });
                }
            }
            _ => {}
        }

        match action {
            Action::LoadAllItems => {
                let tx = self.action_tx.clone();
                let hivemq_address = self.hivemq_address.clone();
                let _handle = tokio::spawn(async move {
                    let result = fetch_client_ids(hivemq_address).await;
                    tx.send(Action::ClientIdsLoadingFinished(result))
                        .expect("Failed to send client ids loading finished action");
                });
            }
            Action::ClientDetailsLoadingFinished(result) => match result {
                Ok((client_id, details)) => {
                    self.list_with_details.put(client_id, Some(details));
                }
                Err(msg) => {
                    self.list_with_details.list_error(&msg);
                }
            },
            Action::ClientIdsLoadingFinished(result) => match result {
                Ok(client_ids) => {
                    let mut details = Vec::with_capacity(client_ids.len());
                    for client_id in client_ids {
                        details.push((client_id, None));
                    }
                    self.list_with_details.update_items(details);
                }
                Err(msg) => {
                    self.list_with_details.list_error(&msg);
                }
            },
            _ => (),
        };

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        self.list_with_details.draw(f, area).unwrap();
        Ok(())
    }
}

impl TabComponent for Clients<'_> {
    fn get_name(&self) -> &str {
        "Clients"
    }
}
