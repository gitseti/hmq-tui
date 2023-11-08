use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tracing::span;
use client_details::ClientDetails;

use super::{Component, Frame};
use crate::{
    action::Action,
    config::{Config, KeyBindings},
    components::client_details,
};
use crate::components::policies::Policies;
use crate::components::tab_components::TabComponent;
use crate::tui::Event;

pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    tabs: [Box<dyn TabComponent>; 2],
    is_loading: bool,
    active_tab: usize
}

impl Home {
    pub fn new() -> Self {
        return Home {
            command_tx: None,
            config: Config::default(),
            tabs: [Box::new(ClientDetails::new()), Box::new(Policies::new())],
            is_loading: false,
            active_tab: 0
        };
    }

    pub fn next_tab(&mut self) {
        if self.active_tab < self.tabs.len() - 1 {
            self.active_tab = self.active_tab + 1;
        }
    }

    pub fn prev_tab(&mut self) {
        if self.active_tab > 0 {
            self.active_tab = self.active_tab - 1;
        }
    }
}

impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx.clone());
        for tab in self.tabs.iter_mut() {
            tab.register_action_handler(tx.clone())?;
        }
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        for tab in self.tabs.iter_mut() {
            tab.register_config_handler(config.clone())?;
        }
        Ok(())
    }

    fn init(&mut self, area: Rect) -> Result<()> {
        for tab in self.tabs.iter_mut() {
            tab.init(area)?;
        }
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let tab_action = self.tabs[self.active_tab].handle_events(event.clone())?;
        if tab_action.is_some() {
            self.command_tx.clone().unwrap().send(tab_action.unwrap())?;
        }

        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let tab_action = self.tabs[self.active_tab].handle_key_events(key.clone())?;
        if tab_action.is_some() {
            self.command_tx.clone().unwrap().send(tab_action.unwrap())?;
        }

        Ok(None)
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let tab_action = self.tabs[self.active_tab].handle_mouse_events(mouse.clone())?;
        if tab_action.is_some() {
            self.command_tx.clone().unwrap().send(tab_action.unwrap())?;
        }

        Ok(None)
    }


    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let tab_action = self.tabs[self.active_tab].update(action.clone())?;
        if tab_action.is_some() {
            self.command_tx.clone().unwrap().send(tab_action.unwrap())?;
        }

        match action {
            Action::NextTab => {
                self.next_tab()
            }
            Action::PrevTab => {
                self.prev_tab()
            }
            _ => ()
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(1),
                Constraint::Percentage(100),
                Constraint::Min(1),
            ])
            .split(f.size());
        let tab_area = layout[1];


        let mut spans = vec![];
        for (i, tab) in self.tabs.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(format!("|"),))
            }


            let style = if i == self.active_tab  {
                Style::default().bg(Color::Green).bold()
            } else {
                Style::default()
            };

            let name = tab.get_name();
            let text = Span::styled(format!(" {name} [{i}] "), style);
            spans.push(text)
        }
        f.render_widget(Paragraph::new(Line::from(spans)), layout[0]);

        let active_tab = &mut self.tabs[self.active_tab];
        active_tab.draw(f, tab_area)?;

        f.render_widget(Paragraph::new("[Q] Quit | [Tab] Next Tab | [Backtab] Previous Tab"),    layout[2]);

        Ok(())
    }
}
