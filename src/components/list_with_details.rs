use std::cell::RefCell;

use std::rc::Rc;
use std::sync::Arc;

use arboard::Clipboard;
use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use indexmap::IndexMap;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Stylize,
    style::{Color, Modifier, Style, Styled},
    widgets::{block::Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;
use tui::Frame;
use typed_builder::TypedBuilder;
use State::Loaded;

use crate::components::list_with_details::ListPopup::{DeletePopup, ErrorPopup};
use crate::components::popup;
use crate::components::popup::Popup;

use crate::{
    action::{Action, Action::SelectedItem},
    components::{
        editor::Editor,
        item_features::{CreateFn, DeleteFn, ItemSelector, ListFn},
        list_with_details::{
            FocusMode::FocusOnList,
            State::{Loading, LoadingError},
        },
        Component,
    },
    mode::Mode,
    tui,
};

#[derive(TypedBuilder)]
pub struct ListWithDetails<'a, T> {
    #[builder(setter(into))]
    list_title: String,

    #[builder(setter(into))]
    details_title: String,

    #[builder(setter(into))]
    hivemq_address: String,

    #[builder]
    mode: Rc<RefCell<Mode>>,

    #[builder]
    selector: Box<dyn ItemSelector<T>>,

    #[builder(setter(strip_option), default)]
    delete_fn: Option<Arc<dyn DeleteFn>>,

    #[builder(setter(strip_option), default)]
    list_fn: Option<Arc<dyn ListFn>>,

    #[builder(setter(strip_option), default)]
    create_fn: Option<Arc<dyn CreateFn>>,

    #[builder(setter(skip), default =
    Loaded(LoadedState {
    items: IndexMap::new(),
    list: vec ! [],
    focus_mode: FocusMode::FocusOnList(ListState::default()),
    }))]
    state: State<'a, T>,

    #[builder]
    action_tx: UnboundedSender<Action>,

    #[builder(setter(skip), default)]
    new_item_editor: Option<Editor<'a>>,

    #[builder(setter(skip), default)]
    popup: Option<ListPopup>,
}

pub enum State<'a, T> {
    LoadingError(String),
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
    FocusOnDetailsError(String, String),
}

impl<T: Serialize> ListWithDetails<'_, T> {
    pub fn get_standard_mode(&self) -> Mode {
        if self.create_fn.is_some() && self.delete_fn.is_some() {
            Mode::FullTab
        } else if self.delete_fn.is_some() {
            Mode::ReadDeleteTab
        } else {
            Mode::ReadTab
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

    pub fn remove(&mut self, key: String) -> Option<T> {
        let Loaded(loaded) = &mut self.state else {
            return None;
        };

        let Some((index, _, item)) = loaded.items.shift_remove_full(&key) else {
            return None;
        };

        loaded.list.remove(index);

        let list_state = match &mut loaded.focus_mode {
            FocusMode::FocusOnList(list_state) => list_state,
            FocusMode::FocusOnDetails((list_state, _)) => list_state,
            _ => return Some(item),
        };

        if index > loaded.list.len().saturating_sub(1) {
            list_state.select(Some(index.saturating_sub(1)))
        }

        if loaded.list.len() == 0 {
            list_state.select(None);
        }

        Some(item)
    }

    pub fn get(&mut self, key: String) -> Option<&T> {
        if let Loaded(state) = &mut self.state {
            return state.items.get(&key);
        }
        None
    }

    pub fn get_selected(&self) -> Option<(&String, &T)> {
        let Loaded(state) = &self.state else {
            return None;
        };

        let FocusMode::FocusOnList(ref list_state) = state.focus_mode else {
            return None;
        };

        let Some(index) = list_state.selected() else {
            return None;
        };

        let Some((key, item)) = state.items.get_index(index) else {
            return None;
        };

        Some((key, item))
    }

    pub fn list_error(&mut self, msg: &str) {
        self.reset();
        self.state = LoadingError(msg.to_owned());
    }

    pub fn details_error(&mut self, title: String, message: String) {
        if let Loaded(loaded_state) = &mut self.state {
            loaded_state.focus_mode = FocusMode::FocusOnDetailsError(title, message);
        }
    }

    pub fn select_item(&mut self, item_key: String) {
        if let Loaded(LoadedState {
            items,
            list: _,
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
            match mode {
                FocusOnList(_) => {
                    *mode = FocusOnList(ListState::default());
                }
                FocusMode::FocusOnDetails((state, _editor)) => {
                    *mode = FocusOnList(state.clone());
                }
                _ => (),
            }
            *self.mode.borrow_mut() = self.get_standard_mode();
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

        let _list = &mut state.list;

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
            FocusMode::FocusOnDetailsError(_, _) => {
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

        let _list = &mut state.list;

        match &mut state.focus_mode {
            FocusMode::FocusOnList(list_state) => {
                let new_selected = match list_state.selected() {
                    Some(i) if i > 0 => i - 1,
                    _ => return None,
                };

                list_state.select(Some(new_selected));
                Some(state.items.get_index(new_selected).unwrap())
            }
            FocusMode::FocusOnDetailsError(_, _) => {
                state.focus_mode = FocusMode::FocusOnList(ListState::default());
                None
            }
            _ => None,
        }
    }

    fn copy_details_to_clipboard(&mut self) -> Result<(), String> {
        let Loaded(loaded_state) = &mut self.state else {
            return Ok(());
        };

        if let FocusMode::FocusOnList(selected) = &mut loaded_state.focus_mode {
            if let Some(selected) = selected.selected() {
                let item = loaded_state.items.get_index(selected).unwrap();
                let details = serde_json::to_string_pretty(item.1).unwrap();
                let mut clipboard = Clipboard::new().or_else(|err| Err(err.to_string()))?;
                clipboard.set_text(details).unwrap();
            }
        }

        Ok(())
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
        *self.mode.borrow_mut() = Mode::EditorReadOnly;
    }

    fn enter_popup(&mut self, popup: ListPopup) {
        *self.mode.borrow_mut() = match popup {
            DeletePopup { .. } => Mode::ConfirmPopup,
            ErrorPopup { .. } => Mode::ErrorPopup,
        };
        self.popup = Some(popup);
    }

    fn exit_popup(&mut self) {
        if self.popup.is_some() {
            self.popup = None;
            *self.mode.borrow_mut() = self.get_standard_mode();
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
            LoadingError(msg) => {
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
                    Block::default()
                        .style(Style::default().dim())
                        .borders(Borders::ALL)
                        .title(detail_title),
                    detail_layout,
                );
            }
            Loading() => {
                let b = Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Blue))
                    .title(format!("Loading {list_title}..."));
                f.render_widget(b, list_layout);
                f.render_widget(
                    Block::default()
                        .style(Style::default().dim())
                        .borders(Borders::ALL)
                        .title(detail_title),
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
                    FocusMode::FocusOnDetailsError(_, _) => {
                        (ListState::default(), Style::default().dim())
                    }
                };

                let list_style = if custom_component.is_some() || self.new_item_editor.is_some() {
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
                            .bg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    )
                    .set_style(list_style);

                f.render_stateful_widget(list_widget, list_layout, &mut list_state.clone());

                if let Some(custom_component) = custom_component {
                    custom_component.draw(f, detail_layout).unwrap();
                    return Ok(());
                }

                if let Some(editor) = &mut self.new_item_editor {
                    editor.draw(f, detail_layout).unwrap();
                    return Ok(());
                }

                match mode {
                    FocusMode::FocusOnList(list_state) => match list_state.selected() {
                        None => {
                            f.render_widget(
                                Block::default()
                                    .style(Style::default().dim())
                                    .borders(Borders::ALL)
                                    .title(detail_title),
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
                    FocusMode::FocusOnDetailsError(title, message) => {
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
                }
            }
        }

        if let Some(popup) = &mut self.popup {
            let popup: &mut dyn Popup = match popup {
                DeletePopup { popup, .. } => popup,
                ErrorPopup { popup, .. } => popup,
            };
            popup.draw(f, f.size()).unwrap();
        }

        Ok(())
    }
}

impl<T: Serialize> Component for ListWithDetails<'_, T> {

    fn activate(&mut self) -> Result<()> {
        *self.mode.borrow_mut() = self.get_standard_mode();
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if let Some(editor) = &mut self.new_item_editor {
            return editor.handle_key_events(key);
        }

        let state = match &mut self.state {
            LoadingError(_) => {
                self.reset();
                return Ok(None);
            }
            Loading() => return Ok(None),
            Loaded(state) => state,
        };

        if let FocusMode::FocusOnDetails((_, editor)) = &mut state.focus_mode {
            return editor.handle_key_events(key);
        }

        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::LoadAllItems => {
                if let Some(list_fn) = &self.list_fn {
                    let tx = self.action_tx.clone();
                    let hivemq_address = self.hivemq_address.clone();
                    let list_fn = list_fn.clone();
                    let _handle = tokio::spawn(async move {
                        let result = list_fn.list(hivemq_address).await;
                        tx.send(Action::ItemsLoadingFinished { result }).unwrap();
                    });
                }
                self.loading();
            }
            Action::ItemsLoadingFinished { result } => {
                match result {
                    Ok(items) => {
                        let mut unwrapped_items = Vec::with_capacity(items.len());
                        for (id, item) in items {
                            let Some(item) = self.selector.select(item) else {
                                break;
                            };
                            unwrapped_items.push((id, item));
                        }
                        self.update_items(unwrapped_items);
                    }
                    Err(msg) => self.list_error(&msg),
                }
                return Ok(None);
            }
            Action::PrevItem => {
                if let Some((key, _value)) = self.prev_item() {
                    return Ok(Some(SelectedItem(key.to_owned())));
                }
            }
            Action::NextItem => {
                if let Some((key, _value)) = self.next_item() {
                    return Ok(Some(SelectedItem(key.to_owned())));
                }
            }
            Action::FocusDetails => {
                self.focus_on_details();
            }
            Action::Escape => {
                self.unfocus();
                if let Some(_editor) = &mut self.new_item_editor {
                    self.new_item_editor = None;
                }
            }
            Action::Delete => {
                if let Some((id, _)) = self.get_selected() {
                    if self.delete_fn.is_some() {
                        let item_type = self.details_title.clone();
                        let item_id = id.clone();
                        let popup = popup::ConfirmPopup {
                            title: format!("Delete {} '{}'?", item_type, item_id).to_string(),
                            message: format!(
                                "Are you sure you want to delete the {} with id '{}'",
                                item_type, item_id
                            )
                            .to_string(),
                        };
                        self.enter_popup(DeletePopup { popup, item_id })
                    }
                }
            }
            Action::ItemDeleted { item_type, result } => {
                if item_type.eq(&self.details_title) {
                    match result {
                        Ok(id) => {
                            self.remove(id);
                            self.exit_popup();
                        }
                        Err(message) => {
                            let popup = popup::ErrorPopup {
                                title: "Deletion failed".to_string(),
                                message: format!("Failed deletion of item:\n{}", message),
                            };
                            self.enter_popup(ErrorPopup { popup })
                        }
                    };
                }
            }
            Action::NewItem => {
                if self.create_fn.is_some() {
                    self.unfocus();
                    self.new_item_editor = Some(Editor::writeable(
                        format!("New {}", self.details_title).to_owned(),
                    ));
                    *self.mode.borrow_mut() = Mode::Editor;
                    return Ok(None);
                }
            }
            Action::Submit => {
                if let Some(editor) = &mut self.new_item_editor {
                    let text = editor.get_text();
                    let host = self.hivemq_address.clone();
                    let tx = self.action_tx.clone();
                    let create_fn = self.create_fn.clone().unwrap();
                    tokio::spawn(async move {
                        let result = create_fn.create(host, text).await;
                        tx.send(Action::ItemCreated { result }).unwrap();
                    });
                }
            }
            Action::ItemCreated { result } => {
                self.new_item_editor = None;
                match result {
                    Ok(item) => {
                        if let Some((id, item)) = self.selector.select_with_id(item) {
                            self.put(id.clone(), item);
                            self.select_item(id)
                        }
                    }
                    Err(error) => {
                        self.details_error(
                            format!("{} creation failed", self.details_title),
                            error,
                        );
                    }
                }
                *self.mode.borrow_mut() = self.get_standard_mode();
                return Ok(None);
            }
            Action::Enter => self.focus_on_details(),
            Action::Copy => {
                if let Err(message) = self.copy_details_to_clipboard() {
                    let popup = popup::ErrorPopup {
                        title: "Could not copy to clipboard".to_string(),
                        message,
                    };
                    self.enter_popup(ErrorPopup { popup })
                }
            }
            Action::ClosePopup => {
                self.exit_popup();
            }
            Action::ConfirmPopup => {
                if let Some(DeletePopup { item_id, .. }) = &self.popup {
                    if let Some(delete_fn) = &self.delete_fn {
                        let tx = self.action_tx.clone();
                        let host = self.hivemq_address.clone();
                        let delete_fn = delete_fn.clone();
                        let item_type = self.details_title.clone();
                        let item_id = item_id.clone();
                        tokio::spawn(async move {
                            let result = delete_fn.delete(host, item_id).await;
                            tx.send(Action::ItemDeleted { item_type, result }).unwrap();
                        });
                    }
                }
            }
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        self.draw_custom(f, area, None)
    }
}

enum ListPopup {
    DeletePopup {
        popup: popup::ConfirmPopup,
        item_id: String,
    },
    ErrorPopup {
        popup: popup::ErrorPopup,
    },
}
