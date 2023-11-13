use std::collections::HashMap;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, ListState};
use tokio::sync::mpsc::UnboundedSender;
use crate::action::Action;
use crate::components::Component;
use crate::components::tab_components::TabComponent;
use crate::tui::Frame;
use color_eyre::eyre::Result;

pub struct Policies {
    command_tx: Option<UnboundedSender<Action>>,
    state: ListState,
    items: Vec<String>,
    is_loading: bool,
}

impl Policies {
    pub(crate) fn new() -> Self {
        Policies {
            command_tx: None,
            state: ListState::default(),
            items: vec![],
            is_loading: false
        }
    }
}

impl Component for Policies {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        f.render_widget(Block::default().borders(Borders::ALL).title("Policies under construction..."), area);
        Ok(())
    }
}

impl TabComponent for Policies {
    fn get_name(&self) -> String {
        "Policies".to_string()
    }

    fn get_key_hints(&self) -> Vec<(&str, &str)> {
        vec![]
    }
}