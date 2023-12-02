use crate::action::Action;
use crate::action::Action::{SelectedItem, SwitchMode};
use arboard::Clipboard;
use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use futures::future::err;
use indexmap::IndexMap;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::{Color, Modifier, Style, Styled};
use ratatui::widgets::block::Block;
use ratatui::widgets::{Borders, List, ListItem, ListState, Paragraph, Widget, Wrap};
use serde::Serialize;
use std::fmt::Display;
use tui::Frame;
use State::Loaded;

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
    focus_mode: FocusMode<'a>,
}

pub enum FocusMode<'a> {
    FocusOnList(ListState),
    FocusOnDetails((ListState, Editor<'a>)),
    NoFocus,
    Error(String, String),
}

impl<T: Serialize> ListWithDetails<'_, T> {
    pub fn new(list_title: String, details_title: String) -> Self {
        let loaded_state = LoadedState {
            items: IndexMap::new(),
            list: vec![],
            focus_mode: FocusMode::FocusOnList(ListState::default()),
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
            focus_mode: FocusMode::FocusOnList(ListState::default()),
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
            loaded_state.focus_mode = FocusMode::Error(title, message);
        }
    }

    pub fn select_item(&mut self, item_key: String) {
        if let Loaded(LoadedState {
            items,
            list,
            focus_mode: mode,
            ..
        }) = &mut self.state
        {
            let index = items.get_index_of(&item_key);
            *mode = FocusMode::FocusOnList(ListState::default().with_selected(index));
        }
    }

    pub fn unfocus(&mut self) {
        if let Loaded(LoadedState {
            focus_mode: mode, ..
        }) = &mut self.state
        {
            *mode = FocusMode::NoFocus;
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

        match &mut state.focus_mode {
            FocusMode::FocusOnList(list_state) => {
                let new_selected = match list_state.selected() {
                    None if state.list.len() != 0 => 0,
                    Some(i) if i + 1 < state.list.len() => i + 1,
                    _ => return None,
                };

                list_state.select(Some(new_selected));
                Some(state.items.get_index(new_selected).unwrap())
            }
            FocusMode::Error(_, _) => {
                state.focus_mode = FocusMode::FocusOnList(ListState::default());
                None
            }
            _ => None,
        }
    }

    fn prev_item(&mut self) -> Option<(&String, &T)> {
        let Loaded(state) = &mut self.state else {
            return None;
        };

        let list = &mut state.list;

        match &mut state.focus_mode {
            FocusMode::FocusOnList(list_state) => {
                let new_selected = match list_state.selected() {
                    Some(i) if i > 0 => i - 1,
                    _ => return None,
                };

                list_state.select(Some(new_selected));
                Some(state.items.get_index(new_selected).unwrap())
            }
            FocusMode::Error(_, _) => {
                state.focus_mode = FocusMode::FocusOnList(ListState::default());
                None
            }
            _ => None,
        }
    }

    fn copy_details_to_clipboard(&mut self) {
        let Loaded(loaded_state) = &mut self.state else {
            return;
        };

        if let FocusMode::FocusOnList(selected) = &mut loaded_state.focus_mode {
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
            (*state).focus_mode = FocusMode::FocusOnList(list_state);
        };
    }

    fn focus_on_details(&mut self) {
        let Loaded(state) = &mut self.state else {
            return;
        };

        let FocusMode::FocusOnList(list_state) = &state.focus_mode else {
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

        (*state).focus_mode = FocusMode::FocusOnDetails((list_state.clone(), editor));
    }

    fn is_focus_on_details(&mut self) -> bool {
        let Loaded(state) = &mut self.state else {
            return false;
        };

        match state.focus_mode {
            FocusMode::FocusOnDetails(_) => true,
            _ => false,
        }
    }

    pub fn draw_custom(
        &mut self,
        f: &mut Frame<'_>,
        area: Rect,
        custom_component: Option<&mut dyn Component>,
    ) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
            .split(area);

        let list_layout = layout[0];
        let detail_layout = layout[1];
        let detail_title = self.details_title.clone();
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
                f.render_widget(p, list_layout);
                f.render_widget(
                    Block::default().borders(Borders::ALL).title(detail_title),
                    detail_layout,
                );
            }
            Loading() => {
                let b = Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::LightBlue))
                    .title(format!("Loading {list_title}..."));
                f.render_widget(b, area);
                f.render_widget(
                    Block::default().borders(Borders::ALL).title(detail_title),
                    detail_layout,
                );
            }
            Loaded(state) => {
                let LoadedState {
                    items,
                    focus_mode: mode,
                    list,
                } = state;

                let (list_state, list_style) = match mode {
                    FocusMode::FocusOnList(list_state) => {
                        (list_state.clone(), Style::default().not_dim())
                    }
                    FocusMode::FocusOnDetails((list_state, _)) => {
                        (list_state.clone(), Style::default().dim())
                    }
                    FocusMode::Error(_, _) => (ListState::default(), Style::default().dim()),
                    FocusMode::NoFocus => (ListState::default(), Style::default().dim()),
                };

                let list_style = if custom_component.is_some() {
                    Style::default().dim()
                } else {
                    list_style
                };

                let list_widget = List::new(list.clone())
                    .block(Block::default().borders(Borders::ALL).title(format!(
                        "{} ({}/{})",
                        list_title,
                        list_state.selected().map_or(0, |i| i + 1),
                        list.len()
                    )))
                    .highlight_style(
                        Style::default()
                            .bg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )
                    .set_style(list_style);

                f.render_stateful_widget(list_widget, list_layout, &mut list_state.clone());

                if let Some(custom_component) = custom_component {
                    custom_component.draw(f, detail_layout).unwrap();
                    return Ok(());
                }

                match mode {
                    FocusMode::FocusOnList(list_state) => match list_state.selected() {
                        None => {
                            f.render_widget(
                                Block::default().borders(Borders::ALL).title(detail_title),
                                detail_layout,
                            );
                        }
                        Some(selected) => {
                            let item = items.get_index(selected).unwrap();
                            let mut editor = Editor::readonly(
                                serde_json::to_string_pretty(item.1).unwrap(),
                                self.details_title.to_owned(),
                            );
                            editor.unfocus();
                            editor.draw(f, detail_layout).unwrap();
                        }
                    },
                    FocusMode::FocusOnDetails((_, editor)) => {
                        editor.draw(f, detail_layout).unwrap();
                    }
                    FocusMode::Error(title, message) => {
                        let p = Paragraph::new(message.clone())
                            .wrap(Wrap { trim: true })
                            .style(Style::default().fg(Color::Red))
                            .block(
                                Block::default()
                                    .borders(Borders::ALL)
                                    .title(title.to_owned()),
                            );
                        f.render_widget(p, detail_layout);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

impl<T: Serialize> Component for ListWithDetails<'_, T> {
    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let Loaded(state) = &mut self.state else {
            return Ok(None);
        };

        if let FocusMode::FocusOnDetails((_, editor)) = &mut state.focus_mode {
            return editor.handle_key_events(key);
        }

        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Loaded(LoadedState {
            focus_mode: mode, ..
        }) = &mut self.state
        {
            if let FocusMode::FocusOnDetails((selected, _)) = mode {
                if action == Action::Escape {
                    *mode = FocusMode::FocusOnList(selected.clone());
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
            Action::Escape => {
                if let Loaded(LoadedState {
                    focus_mode: mode, ..
                }) = &mut self.state
                {
                    *mode = FocusMode::FocusOnList(ListState::default());
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
        self.draw_custom(f, area, None)
    }
}
