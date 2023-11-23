use arboard::Clipboard;
use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use indexmap::IndexMap;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Modifier, Style, Styled};
use ratatui::widgets::block::Block;
use ratatui::widgets::{Borders, List, ListItem, ListState, Paragraph, Wrap};
use serde::Serialize;
use std::fmt::Display;
use tui::Frame;
use State::Loaded;

use crate::components::editor::Editor;
use crate::components::list_with_details::State::{Error, Loading};
use crate::components::Component;
use crate::tui;

pub enum State<'a, T> {
    Error(String),
    Loading(),
    Loaded(LoadedState<'a, T>),
}

pub struct LoadedState<'a, T> {
    items: IndexMap<String, T>,
    list: Vec<ListItem<'a>>,
    list_state: ListState,
    focus: Focus<'a>,
}

pub enum Focus<'a> {
    OnList,
    OnDetails(Editor<'a>),
}

pub struct ListWithDetails<'a, T> {
    pub list_title: String,
    pub details_title: String,
    pub state: State<'a, T>,
}

impl<T: Serialize> ListWithDetails<'_, T> {
    pub fn new(list_title: String, details_title: String) -> Self {
        let loaded_state = LoadedState {
            items: IndexMap::new(),
            list: vec![],
            list_state: ListState::default(),
            focus: Focus::OnList,
        };
        ListWithDetails {
            list_title,
            details_title,
            state: Loaded(loaded_state),
        }
    }

    pub fn reset(&mut self) {
        self.state = Loaded(LoadedState {
            items: IndexMap::new(),
            list: vec![],
            list_state: Default::default(),
            focus: Focus::OnList,
        })
    }

    pub fn update_items(&mut self, items: Vec<(String, T)>) {
        self.reset();
        if let Loaded(state) = &mut self.state {
            for item in items {
                state.items.insert(item.0.clone(), item.1);
                state.list.push(ListItem::new(item.0.clone()));
            }
        }
    }

    pub fn put(&mut self, key: String, value: T) {
        if let Loaded(state) = &mut self.state {
            let old_value = state.items.insert(key.to_owned(), value);
            if old_value.is_none() {
                state.list.push(ListItem::new(key.clone()));
            }
        }
    }

    pub fn error(&mut self, msg: &str) {
        self.reset();
        self.state = Error(msg.to_owned());
    }

    pub fn loading(&mut self) {
        self.reset();
        self.state = Loading();
    }

    pub fn next_item(&mut self) -> Option<(&String, &T)> {
        if let Loaded(state) = &mut self.state {
            let new_selected = match state.list_state.selected() {
                None if state.list.len() != 0 => 0,
                Some(i) if i + 1 < state.list.len() => i + 1,
                _ => return None,
            };

            state.list_state.select(Some(new_selected));
            Some(state.items.get_index(new_selected).unwrap())
        } else {
            None
        }
    }

    pub fn prev_item(&mut self) -> Option<(&String, &T)> {
        if let Loaded(state) = &mut self.state {
            let new_selected = match state.list_state.selected() {
                Some(i) if i > 0 => i - 1,
                _ => return None,
            };

            state.list_state.select(Some(new_selected));
            Some(state.items.get_index(new_selected).unwrap())
        } else {
            None
        }
    }

    pub fn copy_details_to_clipboard(&mut self) {
        if let Loaded(state) = &mut self.state {
            if let Some(selected) = state.list_state.selected() {
                let item = state.items.get_index(selected).unwrap();
                let details = serde_json::to_string_pretty(item.1).unwrap();
                let mut clipboard = Clipboard::new().unwrap();
                clipboard.set_text(details).unwrap();
            }
        }
    }

    pub fn focus_on_list(&mut self) {
        if let Loaded(state) = &mut self.state {
            (*state).focus = Focus::OnList;
        };
    }

    pub fn focus_on_details(&mut self) {
        let Loaded(state) = &mut self.state else {
            return;
        };

        let Focus::OnList = state.focus else {
            return;
        };

        let Some(selected) = state.list_state.selected() else {
            return;
        };

        let Some((_, value)) = state.items.get_index(selected) else {
            return;
        };

        let mut editor = Editor::readonly(
            serde_json::to_string_pretty(value).unwrap(),
            self.details_title.to_owned(),
        );

        editor.focus();

        (*state).focus = Focus::OnDetails(editor);
    }

    pub fn is_focus_on_details(&mut self) -> bool{
        let Loaded(state) = &mut self.state else {
            return false;
        };

        match state.focus {
            Focus::OnList => false,
            Focus::OnDetails(_) => true
        }
    }

    pub fn send_key_event(&mut self, key: KeyEvent) {
        let Loaded(state) = &mut self.state else {
            return;
        };

        let Focus::OnDetails(editor) = &mut state.focus else {
            return;
        };

        if let KeyCode::Esc = key.code {
            self.focus_on_list();
            return;
        };

        (*editor).handle_key_events(key).unwrap();
    }

    pub fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
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
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("Loading {list_title} failed")),
                    );
                f.render_widget(p, list_view);
                f.render_widget(
                    Block::default().borders(Borders::ALL).title(detail_title),
                    detail_view,
                );
            }
            Loading() => {
                let b = Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::LightBlue))
                    .title(format!("Loading {list_title}..."));
                f.render_widget(b, list_view);
                f.render_widget(
                    Block::default().borders(Borders::ALL).title(detail_title),
                    detail_view,
                );
            }
            Loaded(state) => {
                let LoadedState {
                    items,
                    list,
                    list_state,
                    focus,
                } = state;

                let list_style = match focus {
                    Focus::OnList => Style::default(),
                    Focus::OnDetails(_) => Style::default().dim(),
                };

                let details_color = match focus {
                    Focus::OnList => Color::Gray,
                    Focus::OnDetails(_) => Color::default(),
                };

                let list_widget = List::new(list.clone()) //FIXME: Keep whole list in memory
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!(
                                "{} ({}/{})",
                                list_title,
                                list_state.selected().map_or(0, |i| i + 1),
                                list.len()
                            )),
                    )
                    .highlight_style(
                        Style::default()
                            .bg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ).set_style(list_style);
                f.render_stateful_widget(list_widget, list_view, list_state);

                match focus {
                    Focus::OnList => match list_state.selected() {
                        None => {
                            f.render_widget(
                                Block::default()
                                    .borders(Borders::ALL)
                                    .style(Style::default().fg(details_color))
                                    .title(detail_title),
                                detail_view,
                            );
                        }
                        Some(selected) => {
                            let item = items.get_index(selected).unwrap();
                            let mut editor = Editor::readonly(
                                serde_json::to_string_pretty(item.1).unwrap(),
                                self.details_title.to_owned(),
                            );
                            editor.unfocus();
                            editor.draw(f, detail_view).unwrap();
                        }
                    },
                    Focus::OnDetails(editor) => {
                        editor.draw(f, detail_view).unwrap();
                    }
                }
            }
        }

        Ok(())
    }
}
