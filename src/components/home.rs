use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tracing::span;

use super::{Component, Frame};
use crate::components::tabs::backups::BackupsTab;
use crate::components::tabs::behavior_policies::BehaviorPoliciesTab;
use crate::components::tabs::clients::Clients;
use crate::components::tabs::data_policies::DataPoliciesTab;
use crate::components::tabs::schemas::SchemasTab;
use crate::components::tabs::trace_recordings::TraceRecordingsTab;
use crate::components::tabs::TabComponent;
use crate::tui::Event;
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};

pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    tabs: [Box<dyn TabComponent>; 6],
    active_tab: usize,
}

impl Home {
    pub fn new(hivemq_address: String) -> Self {
        return Home {
            command_tx: None,
            config: Config::default(),
            tabs: [
                Box::new(Clients::new(hivemq_address.to_owned())),
                Box::new(SchemasTab::new(hivemq_address.to_owned())),
                Box::new(DataPoliciesTab::new(hivemq_address.to_owned())),
                Box::new(BehaviorPoliciesTab::new(hivemq_address.to_owned())),
                Box::new(TraceRecordingsTab::new(hivemq_address.to_owned())),
                Box::new(BackupsTab::new(hivemq_address.to_owned())),
            ],
            active_tab: 0,
        };
    }

    pub fn select_tab(&mut self, index: usize) {
        if index != self.active_tab && index < self.tabs.len() {
            self.active_tab = index;
        }
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
        let tab_action = self.tabs[self.active_tab].handle_key_events(key)?;
        if tab_action.is_some() {
            self.command_tx.clone().unwrap().send(tab_action.unwrap())?;
        }

        Ok(None)
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let tab_action = self.tabs[self.active_tab].handle_mouse_events(mouse)?;
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
            Action::SelectTab(tab) => self.select_tab(tab),
            Action::NextTab => self.next_tab(),
            Action::PrevTab => self.prev_tab(),
            _ => {}
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

        let spans: Vec<Span> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let style = if i == self.active_tab {
                    Style::default().bg(Color::Green).bold()
                } else {
                    Style::default()
                };

                let index = i + 1;
                let name = tab.get_name();
                Span::styled(format!(" [{index}] {name} "), style)
            })
            .collect();
        f.render_widget(Paragraph::new(Line::from(spans)), layout[0]);

        let active_tab = &mut self.tabs[self.active_tab];
        active_tab.draw(f, tab_area)?;

        let mappings = active_tab
            .get_key_hints()
            .iter()
            .map(|(key, value)| format!(" [{key}] {value} "))
            .collect::<Vec<_>>();
        let mappings = mappings.join("");
        f.render_widget(Paragraph::new(mappings), layout[2]);

        Ok(())
    }
}