use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use hivemq_openapi::models::ClientDetails;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use tui::Frame;
use Action::LoadAllItems;

use crate::action::Action::ClientDetailsLoadingFinished;
use crate::components::list_with_details::Features;
use crate::mode::Mode;
use crate::repository::{Repository, RepositoryError};
use crate::services::client_details_service::ClientDetailsService;
use crate::{
    action::Action,
    components::{list_with_details::ListWithDetails, tabs::TabComponent, Component},
    tui,
};

pub struct Clients<'a> {
    action_tx: UnboundedSender<Action>,
    list_with_details: ListWithDetails<'a, ClientDetails>,
    service: Arc<ClientDetailsService>,
    repository: Arc<Repository<ClientDetails>>,
}

impl<'a> Clients<'a> {
    pub fn new(
        action_tx: UnboundedSender<Action>,
        hivemq_address: String,
        mode: Rc<RefCell<Mode>>,
        sqlite_pool: &Pool<SqliteConnectionManager>,
    ) -> Self {
        let repository = Arc::new(
            Repository::<ClientDetails>::init(sqlite_pool, "client_details", |details| {
                details.id.clone().unwrap()
            }, "id")
            .unwrap(),
        );
        let client_details_service = ClientDetailsService::new(repository.clone(), &hivemq_address);
        let service = Arc::new(client_details_service);
        let list_with_details = ListWithDetails::<ClientDetails>::builder()
            .list_title("Clients")
            .item_name("Client Details")
            .mode(mode)
            .features(Features::builder().build())
            .repository(repository.clone())
            .build();
        Clients {
            action_tx,
            list_with_details,
            service,
            repository,
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
        let _ = self.list_with_details.update(action.clone());

        match action {
            LoadAllItems => {
                let service = self.service.clone();
                let tx = self.action_tx.clone();
                let _ = tokio::spawn(async move {
                    let result = service.load_details().await;
                    let action = match result {
                        Ok(_) => ClientDetailsLoadingFinished(Ok(())),
                        Err(msg) => ClientDetailsLoadingFinished(Err(msg)),
                    };
                    tx.send(action)
                        .expect("Failed to send ClientDetailsLoadingFinished action");
                });
            }
            ClientDetailsLoadingFinished(result) => match result {
                Ok(_) => {
                    let result = self.repository.find_all();
                    match result {
                        Ok(client_details) => {
                            let items: Vec<String> = client_details
                                .into_iter()
                                .filter(|item| item.id.is_some())
                                .map(|item| item.id.clone().unwrap())
                                .collect();
                            self.list_with_details.set_items(items, None);
                        }
                        Err(repo_err) => {
                            let repo_err = match repo_err {
                                RepositoryError::SerdeError(err) => err.to_string(),
                                RepositoryError::SqlError(err) => err.to_string(),
                            };
                            self.list_with_details.list_error(&repo_err);
                        }
                    }
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
