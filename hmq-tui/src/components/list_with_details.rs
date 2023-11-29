use arboard::Clipboard;
use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use indexmap::IndexMap;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Modifier, Style, Styled};
use ratatui::widgets::block::Block;
use ratatui::widgets::{Borders, List, ListItem, ListState, Paragraph, Widget, Wrap};
use serde::Serialize;
use std::fmt::Display;
use futures::future::err;
use tui::Frame;
use State::Loaded;
use crate::action::Action;
use crate::action::Action::{SelectedItem, SwitchMode};

use crate::components::editor::Editor;
use crate::components::list_with_details::State::{Error, Loading};
use crate::components::Component;
use crate::mode::Mode;
use crate::tui;


pub struct ListWithDetails<'a, T> {
    list_title: String,
    details_title: String,
    state: State<'a, T>,
}

pub enum State<'a, T> {
    Error(String),
    Loading(),
    Loaded(LoadedState<'a, T>),
}

pub struct LoadedState<'a, T> {
    items: IndexMap<String, T>,
    list: Vec<ListItem<'a>>,
    mode: LoadedMode<'a>,
}

pub enum LoadedMode<'a> {
    FocusOnList(ListState),
    FocusOnDetails((ListState, Editor<'a>)),
    Error((String, String)),
    Loading,
}

impl<T: Serialize> ListWithDetails<'_, T> {
    pub fn new(list_title: String, details_title: String) -> Self {
        let loaded_state = LoadedState {
            items: IndexMap::new(),
            list: vec![],
            mode: LoadedMode::FocusOnList(ListState::default()),
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
            mode: LoadedMode::FocusOnList(ListState::default()),
        });
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

    pub fn get(&mut self, key: String) -> Option<&T> {
        if let Loaded(state) = &mut self.state {
            return state.items.get(&key);
        }
        None
    }

    pub fn error(&mut self, msg: &str) {
        self.reset();
        self.state = Error(msg.to_owned());
    }

    pub fn details_error(&mut self, title: String, message: String) {
        if let Loaded(loaded_state) = &mut self.state {
            loaded_state.mode = LoadedMode::Error((title, message));
        }
    }

    fn loading(&mut self) {
        self.reset();
        self.state = Loading();
    }

    fn next_item(&mut self) -> Option<(&String, &T)> {
        let Loaded(state) = &mut self.state else {
            return None;
        };

        let list = &mut state.list;


        if let LoadedMode::FocusOnList(list_state) = &mut state.mode {
            let new_selected = match list_state.selected() {
                None if state.list.len() != 0 => 0,
                Some(i) if i + 1 < state.list.len() => i + 1,
                _ => return None,
            };

            list_state.select(Some(new_selected));
            Some(state.items.get_index(new_selected).unwrap())
        } else {
            None
        }
    }

    fn prev_item(&mut self) -> Option<(&String, &T)> {
        let Loaded(state) = &mut self.state else {
            return None;
        };

        let list = &mut state.list;

        if let LoadedMode::FocusOnList(list_state) = &mut state.mode {
            let new_selected = match list_state.selected() {
                Some(i) if i > 0 => i - 1,
                _ => return None,
            };

            list_state.select(Some(new_selected));
            Some(state.items.get_index(new_selected).unwrap())
        } else {
            None
        }
    }

    fn copy_details_to_clipboard(&mut self) {
        let Loaded(loaded_state) = &mut self.state else {
            return;
        };


        if let LoadedMode::FocusOnList(selected) = &mut loaded_state.mode {
            if let Some(selected) = selected.selected() {
                let item = loaded_state.items.get_index(selected).unwrap();
                let details = serde_json::to_string_pretty(item.1).unwrap();
                let mut clipboard = Clipboard::new().unwrap();
                clipboard.set_text(details).unwrap();
            }
        }
    }

    fn focus_on_list(&mut self, list_state: ListState) {
        if let Loaded(state) = &mut self.state {
            (*state).mode = LoadedMode::FocusOnList(list_state);
        };
    }

    fn focus_on_details(&mut self) {
        let Loaded(state) = &mut self.state else {
            return;
        };

        let LoadedMode::FocusOnList(list_state) = &state.mode else {
            return;
        };

        let Some(selected) = list_state.selected() else {
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

        (*state).mode = LoadedMode::FocusOnDetails((list_state.clone(), editor));
    }

    fn is_focus_on_details(&mut self) -> bool {
        let Loaded(state) = &mut self.state else {
            return false;
        };

        match state.mode {
            LoadedMode::FocusOnDetails(_) => true,
            _ => false
        }
    }

    pub fn draw_list(&mut self, f: &mut Frame<'_>, area: Rect, dim: bool) {
        let list_title = self.list_title.clone();

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
                f.render_widget(p, area)
            }
            Loading() => {
                let b = Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::LightBlue))
                    .title(format!("Loading {list_title}..."));
                f.render_widget(b, area)
            }
            Loaded(state) => {
                let LoadedState {
                    list,
                    mode,
                    ..
                } = state;

                let style = if dim {
                    Style::default().dim()
                } else {
                    Style::default()
                };

                let list_state = match &state.mode {
                    LoadedMode::FocusOnList(list_state) => list_state.clone(),
                    LoadedMode::FocusOnDetails((list_state, _)) => list_state.clone(),
                    LoadedMode::Error(message) => ListState::default(),
                    LoadedMode::Loading => ListState::default()
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
                    )
                    .set_style(style);

                f.render_stateful_widget(list_widget, area, &mut list_state.clone());
            }
        }
    }
}

impl<T: Serialize> Component for ListWithDetails<'_, T> {
    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let Loaded(state) = &mut self.state else {
            return Ok(None);
        };

        if let LoadedMode::FocusOnDetails((_, editor)) = &mut state.mode {
            return editor.handle_key_events(key);
        }

        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Loaded(LoadedState { mode, ..}) = &mut self.state {
            if let LoadedMode::FocusOnDetails((selected, _)) = mode {
                if action == Action::Escape {
                    *mode = LoadedMode::FocusOnList(selected.clone());
                    return Ok(Some(SwitchMode(Mode::Main)));
                }
                return Ok(None);
            }
        }

        match action {
            Action::PrevItem => {
                if let Some((key, value)) = self.prev_item() {
                    return Ok(Some(SelectedItem(key.to_owned())));
                }
            }
            Action::NextItem => {
                if let Some((key, value)) = self.next_item() {
                    return Ok(Some(SelectedItem(key.to_owned())));
                }
            }
            Action::FocusDetails => {
                self.focus_on_details();
                if self.is_focus_on_details() {
                    return Ok(Some(SwitchMode(Mode::Editing)));
                }
            }
            Action::Enter => self.focus_on_details(),
            Action::LoadAllItems => self.loading(),
            Action::Copy => self.copy_details_to_clipboard(),
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
            .split(area);

        let list_view = layout[0];
        let detail_view = layout[1];
        let detail_title = self.details_title.clone();

        if let Loaded(LoadedState { mode, .. }) = &self.state {
            let dim = match mode {
                LoadedMode::FocusOnDetails(_) => true,
                _ => false
            };
            self.draw_list(f, list_view, dim);
        } else {
            self.draw_list(f, list_view, false);
        };


        match &mut self.state {
            Error(msg) => {
                f.render_widget(
                    Block::default().borders(Borders::ALL).title(detail_title),
                    detail_view,
                );
            }
            Loading() => {
                f.render_widget(
                    Block::default().borders(Borders::ALL).title(detail_title),
                    detail_view,
                );
            }
            Loaded(state) => {
                let LoadedState { items, mode, .. } = state;
                let (dim, details_color) = match mode {
                    LoadedMode::FocusOnDetails(_) => (true, Color::default()),
                    _ => (false, Color::Gray)
                };

                match mode {
                    LoadedMode::FocusOnList(list_state) => match list_state.selected() {
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
                    LoadedMode::FocusOnDetails((_, editor)) => {
                        editor.draw(f, detail_view).unwrap();
                    }
                    LoadedMode::Error((title, message)) => {
                        let p = Paragraph::new(message.clone())
                            .wrap(Wrap { trim: true })
                            .style(Style::default().fg(Color::Red))
                            .block(
                                Block::default()
                                    .borders(Borders::ALL)
                                    .title(title.to_owned()),
                            );
                        f.render_widget(p, detail_view);
                    }
                    LoadedMode::Loading => {}
                }
            }
        }

        Ok(())
    }
}
