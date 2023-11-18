use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Index;
use arboard::Clipboard;
use tui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use color_eyre::eyre::Result;
use futures::stream::iter;
use indexmap::IndexMap;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Widget, Wrap};
use serde::Serialize;
use State::Loaded;

use crate::{tui};
use crate::components::views::State::{Error, Loading};


pub enum State<'a, T> {
    Error(String),
    Loading(),
    Loaded(IndexMap<String, T>, Vec<ListItem<'a>>, ListState),
}

pub struct DetailsView<'a, T> {
    pub list_title: String,
    pub details_title: String,
    pub state: State<'a, T>,
}

impl<T: Serialize> DetailsView<'_, T> {
    pub fn new(list_title: String, details_title: String) -> Self {
        DetailsView {
            list_title,
            details_title,
            state: Loaded(IndexMap::new(), vec![], ListState::default()),
        }
    }

    pub fn reset(&mut self) {
        self.state = Loaded(IndexMap::new(), vec![], ListState::default())
    }

    pub fn update_items(&mut self, items: Vec<(String, T)>) {
        self.reset();
        if let Loaded(map, list, state) = &mut self.state {
            for item in items {
                map.insert(item.0.clone(), item.1);
                list.push(ListItem::new(item.0.clone()));
            }
        }
    }

    pub fn error(&mut self, msg: &str) {
        self.reset();
        self.state = Error(msg.to_string());
    }

    pub fn loading(&mut self) {
        self.reset();
        self.state = Loading();
    }

    pub fn next_item(&mut self) {
        if let Loaded(map, list, state) = &mut self.state {
            let new_selected = match state.selected() {
                None if list.len() != 0 => 0,
                Some(i) if i + 1 < list.len() => i + 1,
                _ => return
            };

            state.select(Some(new_selected));
        }
    }

    pub fn prev_item(&mut self) {
        if let Loaded(map, list, state) = &mut self.state {
            let new_selected = match state.selected() {
                Some(i) if i > 0 => i - 1,
                _ => return
            };

            state.select(Some(new_selected));
        }
    }

    pub fn copy_details_to_clipboard(&mut self) {
        if let Loaded(map, _, state) = &mut self.state {
            if let Some(selected) = state.selected() {
                let item = map.get_index(selected).unwrap();
                let details = serde_json::to_string_pretty(item.1).unwrap();
                let mut clipboard = Clipboard::new().unwrap();
                clipboard.set_text(details).unwrap();
            }
        }
    }

    pub fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Ratio(1, 3),
                Constraint::Ratio(2, 3),
            ])
            .split(area);

        let list_view = layout[0];
        let detail_view = layout[1];
        let list_title = self.list_title.clone();
        let detail_title = self.details_title.clone();

        match &mut self.state {
            Error(msg) => {
                let p = Paragraph::new(msg.clone())
                    .wrap(Wrap { trim: true })
                    .style(Style::default().fg(Color::Red))
                    .block(Block::default().borders(Borders::ALL).title(format!("Loading {list_title} failed")));
                f.render_widget(p, list_view);
                f.render_widget(Block::default().borders(Borders::ALL).title(detail_title), detail_view);
            }
            Loading() => {
                let b = Block::default().borders(Borders::ALL)
                    .style(Style::default().fg(Color::LightBlue))
                    .title(format!("Loading {list_title}..."));
                f.render_widget(b, list_view);
                f.render_widget(Block::default().borders(Borders::ALL).title(detail_title), detail_view);
            }
            Loaded(map, list, state) => {
                let items = List::new(&**list)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(format!("{} ({}/{})", list_title, state.selected().map_or(0, |i| i + 1), list.len())))
                    .highlight_style(
                        Style::default()
                            .bg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    );
                f.render_stateful_widget(items, list_view, state);

                match state.selected() {
                    None => {
                        f.render_widget(Block::default().borders(Borders::ALL).title(detail_title), detail_view);
                    }
                    Some(selected) => {
                        let item = map.get_index(selected).unwrap();
                        let p = Paragraph::new(serde_json::to_string_pretty(item.1).unwrap()).block(Block::default().borders(Borders::ALL).title(detail_title));
                        f.render_widget(p, detail_view);
                    }
                }
            }
        }

        Ok(())
    }
}