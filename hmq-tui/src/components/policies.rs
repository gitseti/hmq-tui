use std::collections::HashMap;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, ListItem, ListState};
use tokio::sync::mpsc::UnboundedSender;
use crate::action::Action;
use crate::components::{Component, views};
use crate::components::tab_components::TabComponent;
use crate::tui::Frame;
use color_eyre::eyre::Result;
use crate::components::views::DetailsView;

pub struct Policies<'a> {
    command_tx: Option<UnboundedSender<Action>>,
    details_view: DetailsView<'a, String>,
}

impl Policies<'_> {
    pub fn new() -> Self {
        Policies {
            command_tx: None,
            details_view: DetailsView::new("Policies".to_string(), "Policy".to_string())
        }
    }
}

impl Component for Policies<'_> {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Up => {
                self.details_view.prev_item();
            }
            Action::Down => {
                self.details_view.next_item();
            }
            Action::Reload => {
                let vec = vec![("One", "Hello World!".to_string()), ("Two", "Moin, Moion".to_string()), ("Three", "foobar".to_string())];
                self.details_view.update_items(vec);
            }
            Action::Left => {
                let vec = vec![("foobar", "Lorem Ipsum!"), ("foobar3", "Lorem Ipsum!"), ("foobar6", "Lorem Ipsum!")];
                self.details_view.error("Failed to load client details")
            }
            Action::Right => {
                self.details_view.loading();
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
        "Policies"
    }

    fn get_key_hints(&self) -> Vec<(&str, &str)> {
        vec![]
    }
}