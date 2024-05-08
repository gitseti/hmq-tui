use std::cell::RefCell;
use std::rc::Rc;

use color_eyre::eyre::{Ok, Result};
use crossterm::event::{KeyEvent, MouseEvent};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    components::tabs::{
        backups::BackupsTab, behavior_policies::BehaviorPoliciesTab, clients::Clients,
        data_policies::DataPoliciesTab, schemas::SchemasTab, scripts::ScriptsTab,
        trace_recordings::TraceRecordingsTab, TabComponent,
    },
    config::Config,
    mode::Mode,
    tui::Event,
};

use super::{Component, Frame};

pub struct Home {
    action_tx: UnboundedSender<Action>,
    config: Config,
    mode: Rc<RefCell<Mode>>,
    tabs: [Box<dyn TabComponent>; 7],
    active_tab: usize,
}

impl Home {
    pub fn new(
        action_tx: UnboundedSender<Action>,
        config: Config,
        hivemq_address: String,
        mode: Rc<RefCell<Mode>>,
    ) -> Self {
        let sqlite_pool = Pool::new(SqliteConnectionManager::memory()).unwrap();
        return Home {
            action_tx: action_tx.clone(),
            config,
            mode: mode.clone(),
            tabs: [
                Box::new(Clients::new(
                    action_tx.clone(),
                    hivemq_address.to_owned(),
                    mode.clone(),
                    &sqlite_pool,
                )),
                Box::new(SchemasTab::new(
                    action_tx.clone(),
                    hivemq_address.to_owned(),
                    mode.clone(),
                    &sqlite_pool,
                )),
                Box::new(ScriptsTab::new(
                    action_tx.clone(),
                    hivemq_address.to_owned(),
                    mode.clone(),
                    &sqlite_pool,
                )),
                Box::new(DataPoliciesTab::new(
                    action_tx.clone(),
                    hivemq_address.to_owned(),
                    mode.clone(),
                    &sqlite_pool,
                )),
                Box::new(BehaviorPoliciesTab::new(
                    action_tx.clone(),
                    hivemq_address.to_owned(),
                    mode.clone(),
                    &sqlite_pool,
                )),
                Box::new(TraceRecordingsTab::new(
                    action_tx.clone(),
                    hivemq_address.to_owned(),
                    mode.clone(),
                    &sqlite_pool,
                )),
                Box::new(BackupsTab::new(
                    action_tx.clone(),
                    hivemq_address.to_owned(),
                    mode.clone(),
                    &sqlite_pool,
                )),
            ],
            active_tab: 0,
        };
    }

    pub fn select_tab(&mut self, index: usize) {
        if index != self.active_tab && index < self.tabs.len() {
            self.active_tab = index;
            self.tabs[self.active_tab].activate().unwrap();
        }
    }

    pub fn next_tab(&mut self) {
        if self.active_tab < self.tabs.len() - 1 {
            self.active_tab = self.active_tab + 1;
            self.tabs[self.active_tab].activate().unwrap();
        }
    }

    pub fn prev_tab(&mut self) {
        if self.active_tab > 0 {
            self.active_tab = self.active_tab - 1;
            self.tabs[self.active_tab].activate().unwrap();
        }
    }
}

impl Component for Home {
    fn init(&mut self, area: Rect) -> Result<()> {
        for tab in self.tabs.iter_mut() {
            tab.init(area)?;
        }
        let _mode = self.tabs[self.active_tab].activate();
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let tab_action = self.tabs[self.active_tab].handle_events(event.clone())?;
        if tab_action.is_some() {
            self.action_tx.send(tab_action.unwrap())?;
        }

        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let tab_action = self.tabs[self.active_tab].handle_key_events(key)?;
        if tab_action.is_some() {
            self.action_tx.send(tab_action.unwrap())?;
        }

        Ok(None)
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let tab_action = self.tabs[self.active_tab].handle_mouse_events(mouse)?;
        if tab_action.is_some() {
            self.action_tx.send(tab_action.unwrap())?;
        }

        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::SelectTab(tab) => self.select_tab(tab),
            Action::NextTab => self.next_tab(),
            Action::PrevTab => self.prev_tab(),
            _ => {
                self.tabs[self.active_tab].update(action).unwrap();
            }
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, _area: Rect) -> Result<()> {
        let tabs = &mut self.tabs;
        let _active_tab = &tabs[self.active_tab];
        let max_width = f.size().width;

        let mode = self.mode.borrow();
        let key_hints = self.config.keybindings.display_names.get(&*mode).unwrap();
        let key_bindings = split_at_width(key_hints, max_width);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(1),
                Constraint::Min(1),
                Constraint::Percentage(100),
                Constraint::Min(key_bindings.len() as u16),
            ])
            .split(f.size());

        let header_area = layout[0];
        let header_ruler_area = layout[1];
        let tab_area = layout[2];
        let footer_area = layout[3];

        // Create Header
        let titles: Vec<String> = tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| format!(" {} [{}] ", tab.get_name().to_string(), index + 1))
            .collect();
        let header = Tabs::new(titles.to_vec())
            .highlight_style(Style::default().bg(Color::Blue).not_dim().underlined())
            .style(Style::default().dim())
            .select(self.active_tab)
            .padding("", "")
            .divider("");
        f.render_widget(header, header_area);
        f.render_widget(
            Block::default().borders(Borders::BOTTOM).dim(),
            header_ruler_area,
        );

        // Create Tab
        (&mut tabs[self.active_tab]).draw(f, tab_area)?;

        // Create Footer
        f.render_widget(Paragraph::new(key_bindings), footer_area);

        Ok(())
    }
}

fn split_at_width(items: &Vec<String>, max_width: u16) -> Vec<Line> {
    let mut current_width: u16 = 0;
    let mut lines: Vec<Line> = Vec::new();
    let mut current_line: Vec<Span> = Vec::new();
    for item in items {
        let span1 = Span::default()
            .style(Style::default().bg(Color::Blue))
            .content(item);
        let span2 = Span::default().content(" ");
        let item_len = item.chars().count() + 1;
        if (current_width + item_len as u16 > max_width) && current_width != 0 {
            lines.push(Line::from(current_line.clone()));
            current_width = 0;
            current_line.clear();
        }
        current_width += item_len as u16;
        current_line.push(span1);
        current_line.push(span2);
    }
    lines.push(Line::from(current_line));
    lines
}
