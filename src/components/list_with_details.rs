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
use LoadingState::Loaded;

use crate::components::list_with_details::ListPopup::{DeletePopup, ErrorPopup};
use crate::components::popup;
use crate::components::popup::Popup;

use crate::action::Item;
use crate::components::item_features::UpdateFn;
use crate::components::list_with_details::FocusMode::Editing;
use crate::{
    action::{Action, Action::SelectedItem},
    components::{
        editor::Editor,
        item_features::{CreateFn, DeleteFn, ItemSelector, ListFn},
        list_with_details::{
            FocusMode::Scrolling,
            LoadingState::{Loading, LoadingError},
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
    item_name: String,

    #[builder(setter(into))]
    hivemq_address: String,

    #[builder]
    mode: Rc<RefCell<Mode>>,

    #[builder]
    item_selector: Box<dyn ItemSelector<T>>,

    #[builder(setter(strip_option), default)]
    delete_fn: Option<Arc<dyn DeleteFn>>,

    #[builder(setter(strip_option), default)]
    list_fn: Option<Arc<dyn ListFn>>,

    #[builder(setter(strip_option), default)]
    create_fn: Option<Arc<dyn CreateFn>>,

    #[builder(setter(strip_option), default)]
    update_fn: Option<Arc<dyn UpdateFn>>,

    #[builder(default =
    if create_fn.is_some() && delete_fn.is_some() {
    Mode::FullTab
    } else if delete_fn.is_some() {
    Mode::ReadDeleteTab
    } else {
    Mode::ReadTab
    }
    )]
    base_mode: Mode,

    #[builder(setter(skip), default =
    Loaded {
    items: IndexMap::new(),
    list: vec ! [],
    focus_mode: FocusMode::Scrolling(ListState::default()),
    })]
    loading_state: LoadingState<'a, T>,

    #[builder]
    action_tx: UnboundedSender<Action>,

    #[builder(setter(skip), default)]
    new_item_editor: Option<Editor<'a>>,

    #[builder(setter(skip), default)]
    popup: Option<ListPopup>,
}

pub enum LoadingState<'a, T> {
    LoadingError(String),
    Loading,
    Loaded {
        items: IndexMap<String, T>,
        list: Vec<ListItem<'a>>,
        focus_mode: FocusMode<'a>,
    },
}

pub enum FocusMode<'a> {
    Scrolling(ListState),
    Editing {
        list_state: ListState,
        editor: Editor<'a>,
    },
    DetailsError {
        title: String,
        message: String,
    },
}

impl<T: Serialize> ListWithDetails<'_, T> {
    pub fn reset(&mut self) {
        self.loading_state = Loaded {
            items: IndexMap::new(),
            list: vec![],
            focus_mode: FocusMode::Scrolling(ListState::default()),
        };
    }

    pub fn set_items(&mut self, new_items: Vec<(String, T)>) {
        self.reset();
        if let Loaded { items, list, .. } = &mut self.loading_state {
            for item in new_items {
                items.insert(item.0.clone(), item.1);
                list.push(ListItem::new(item.0.clone()));
            }
        }
    }

    fn fetch_items(&mut self) {
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

    pub fn put(&mut self, key: String, value: T) {
        if let Loaded { items, list, .. } = &mut self.loading_state {
            let old_value = items.insert(key.to_owned(), value);
            if old_value.is_none() {
                list.push(ListItem::new(key.clone()));
            }
        }
    }

    pub fn remove(&mut self, key: String) -> Option<T> {
        let Loaded {
            items,
            list,
            focus_mode,
        } = &mut self.loading_state
        else {
            return None;
        };

        let Some((index, _, item)) = items.shift_remove_full(&key) else {
            return None;
        };

        list.remove(index);

        let list_state = match focus_mode {
            FocusMode::Scrolling(list_state) => list_state,
            FocusMode::Editing { list_state, .. } => list_state,
            _ => return Some(item),
        };

        if index > list.len().saturating_sub(1) {
            list_state.select(Some(index.saturating_sub(1)))
        }

        if list.len() == 0 {
            list_state.select(None);
        }

        Some(item)
    }

    pub fn get(&mut self, key: String) -> Option<&T> {
        if let Loaded { items, .. } = &mut self.loading_state {
            items.get(&key)
        } else {
            None
        }
    }

    pub fn get_selected(&self) -> Option<(&String, &T)> {
        let Loaded {
            items, focus_mode, ..
        } = &self.loading_state
        else {
            return None;
        };

        let FocusMode::Scrolling(ref list_state) = focus_mode else {
            return None;
        };

        let Some(index) = list_state.selected() else {
            return None;
        };

        let Some((key, item)) = items.get_index(index) else {
            return None;
        };

        Some((key, item))
    }

    pub fn list_error(&mut self, msg: &str) {
        self.reset();
        self.loading_state = LoadingError(msg.to_owned());
    }

    pub fn details_error(&mut self, title: String, message: String) {
        if let Loaded { focus_mode, .. } = &mut self.loading_state {
            *focus_mode = FocusMode::DetailsError { title, message };
        }
    }

    pub fn select_item(&mut self, item_key: String) {
        if let Loaded {
            items, focus_mode, ..
        } = &mut self.loading_state
        {
            let index = items.get_index_of(&item_key);
            *focus_mode = FocusMode::Scrolling(ListState::default().with_selected(index));
        }
    }

    pub fn set_scrolling_mode(&mut self) {
        if let Loaded { focus_mode, .. } = &mut self.loading_state {
            match focus_mode {
                Scrolling(_) => {
                    *focus_mode = Scrolling(ListState::default());
                }
                FocusMode::Editing { list_state, .. } => {
                    *focus_mode = Scrolling(list_state.clone());
                }
                _ => (),
            }
            *self.mode.borrow_mut() = self.base_mode;
        }
    }

    fn loading(&mut self) {
        self.reset();
        self.loading_state = Loading;
    }

    fn next_item(&mut self) -> Option<(&String, &T)> {
        let Loaded {
            list,
            focus_mode,
            items,
        } = &mut self.loading_state
        else {
            return None;
        };

        match focus_mode {
            FocusMode::Scrolling(list_state) => {
                let new_selected = match list_state.selected() {
                    None if list.len() != 0 => 0,
                    Some(i) if i + 1 < list.len() => i + 1,
                    _ => return None,
                };

                list_state.select(Some(new_selected));
                Some(items.get_index(new_selected).unwrap())
            }
            FocusMode::DetailsError { .. } => {
                *focus_mode = FocusMode::Scrolling(ListState::default());
                None
            }
            _ => None,
        }
    }

    fn prev_item(&mut self) -> Option<(&String, &T)> {
        let Loaded {
            items,
            list: _,
            focus_mode,
        } = &mut self.loading_state
        else {
            return None;
        };

        match focus_mode {
            FocusMode::Scrolling(list_state) => {
                let new_selected = match list_state.selected() {
                    Some(i) if i > 0 => i - 1,
                    _ => return None,
                };

                list_state.select(Some(new_selected));
                Some(items.get_index(new_selected).unwrap())
            }
            FocusMode::DetailsError { .. } => {
                *focus_mode = FocusMode::Scrolling(ListState::default());
                None
            }
            _ => None,
        }
    }

    fn copy_details_to_clipboard(&mut self) -> Result<(), String> {
        let Loaded {
            focus_mode, items, ..
        } = &mut self.loading_state
        else {
            return Ok(());
        };

        if let FocusMode::Scrolling(selected) = focus_mode {
            if let Some(selected) = selected.selected() {
                let item = items.get_index(selected).unwrap();
                let details = serde_json::to_string_pretty(item.1).unwrap();
                let mut clipboard = Clipboard::new().or_else(|err| Err(err.to_string()))?;
                clipboard.set_text(details).unwrap();
            }
        }

        Ok(())
    }

    fn inspect(&mut self) {
        let Loaded {
            items, focus_mode, ..
        } = &mut self.loading_state
        else {
            return;
        };

        let FocusMode::Scrolling(list_state) = focus_mode else {
            return;
        };

        let Some(selected) = list_state.selected() else {
            return;
        };

        let Some((_, item)) = items.get_index(selected) else {
            return;
        };

        let item = serde_json::to_string_pretty(item).unwrap();
        if let Some(_update_fn) = &self.update_fn {
            let update_editor =
                Editor::writeable_with_text(format!("Update {}", self.item_name), item.to_owned());
            *focus_mode = Editing {
                list_state: list_state.clone(),
                editor: update_editor,
            };

            *self.mode.borrow_mut() = Mode::UpdateEditor;
        } else {
            let mut editor = Editor::readonly(item, self.item_name.to_owned());

            editor.focus();

            *focus_mode = FocusMode::Editing {
                list_state: list_state.clone(),
                editor,
            };
            *self.mode.borrow_mut() = Mode::EditorReadOnly;
        }
    }
    fn confirm_popup(&mut self) {
        if let Some(DeletePopup { item_id, .. }) = &self.popup {
            if let Some(delete_fn) = &self.delete_fn {
                let tx = self.action_tx.clone();
                let host = self.hivemq_address.clone();
                let delete_fn = delete_fn.clone();
                let item_type = self.item_name.clone();
                let item_id = item_id.clone();
                tokio::spawn(async move {
                    let result = delete_fn.delete(host, item_id).await;
                    tx.send(Action::ItemDeleted { item_type, result }).unwrap();
                });
            }
        }
    }

    fn copy_json(&mut self) {
        if let Err(message) = self.copy_details_to_clipboard() {
            let popup = popup::ErrorPopup {
                title: "Could not copy to clipboard".to_string(),
                message,
            };
            self.enter_popup(ErrorPopup { popup })
        }
    }

    fn handle_item_updated(&mut self, result: Result<Item, String>) {
        self.set_scrolling_mode();
        match result {
            Ok(item) => {
                if let Some((id, item)) = self.item_selector.select_with_id(item) {
                    self.put(id.clone(), item);
                    self.select_item(id)
                }
            }
            Err(message) => {
                self.details_error(format!("{} update failed", self.item_name), message);
            }
        }
    }

    fn handle_item_created(&mut self, result: Result<Item, String>) {
        self.new_item_editor = None;
        match result {
            Ok(item) => {
                if let Some((id, item)) = self.item_selector.select_with_id(item) {
                    self.put(id.clone(), item);
                    self.select_item(id)
                }
            }
            Err(error) => {
                self.details_error(format!("{} creation failed", self.item_name), error);
            }
        }
        *self.mode.borrow_mut() = self.base_mode;
    }

    fn update_item(&mut self) {
        if let Loaded {
            focus_mode: Editing { editor, list_state },
            items,
            ..
        } = &self.loading_state
        {
            let (id, _) = items.get_index(list_state.selected().unwrap()).unwrap();
            let id = id.clone();
            let text = editor.get_text();
            let host = self.hivemq_address.clone();
            let tx = self.action_tx.clone();
            let update_fn = self.update_fn.clone().unwrap();
            tokio::spawn(async move {
                let result = update_fn.update(host, id, text).await;
                tx.send(Action::ItemUpdated { result }).unwrap();
            });
        }
    }

    fn create_item(&mut self) {
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

    fn enter_new_item_editor(&mut self) {
        if self.create_fn.is_some() {
            self.set_scrolling_mode();
            self.new_item_editor = Some(Editor::writeable(
                format!("New {}", self.item_name).to_owned(),
            ));
            *self.mode.borrow_mut() = Mode::CreateEditor;
        }
    }

    fn handle_item_deleted(&mut self, item_type: String, result: Result<String, String>) {
        if item_type.eq(&self.item_name) {
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

    fn popup_delete_confirmation(&mut self) {
        if let Some((id, _)) = self.get_selected() {
            if self.delete_fn.is_some() {
                let item_type = self.item_name.clone();
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

    fn handle_loading_finished(&mut self, result: Result<Vec<(String, Item)>, String>) {
        match result {
            Ok(items) => {
                let mut unwrapped_items = Vec::with_capacity(items.len());
                for (id, item) in items {
                    let Some(item) = self.item_selector.select(item) else {
                        break;
                    };
                    unwrapped_items.push((id, item));
                }
                self.set_items(unwrapped_items);
            }
            Err(msg) => self.list_error(&msg),
        }
    }

    pub fn error_popup(&mut self, popup: popup::ErrorPopup) {
        self.enter_popup(ErrorPopup { popup });
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
            *self.mode.borrow_mut() = self.base_mode;
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
        let detail_title = self.item_name.clone();
        let list_title = self.list_title.clone();

        match &mut self.loading_state {
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
            Loading => {
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
            Loaded {
                items,
                focus_mode,
                list,
            } => {
                let (list_state, list_style) = match focus_mode {
                    FocusMode::Scrolling(list_state) => {
                        (list_state.clone(), Style::default().not_dim())
                    }
                    FocusMode::Editing { list_state, .. } => {
                        (list_state.clone(), Style::default().dim())
                    }
                    FocusMode::DetailsError { .. } => {
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

                match focus_mode {
                    FocusMode::Scrolling(list_state) => match list_state.selected() {
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
                                self.item_name.to_owned(),
                            );
                            editor.unfocus();
                            editor.draw(f, detail_layout).unwrap();
                        }
                    },
                    FocusMode::Editing { editor, .. } => {
                        editor.draw(f, detail_layout).unwrap();
                    }
                    FocusMode::DetailsError { title, message } => {
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
        *self.mode.borrow_mut() = self.base_mode;
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if let Some(editor) = &mut self.new_item_editor {
            return editor.handle_key_events(key);
        }

        let focus_mode = match &mut self.loading_state {
            LoadingError(_) => {
                self.reset();
                return Ok(None);
            }
            Loading => return Ok(None),
            Loaded { focus_mode, .. } => focus_mode,
        };

        match focus_mode {
            FocusMode::Editing { editor, .. } => {
                return editor.handle_key_events(key);
            }
            _ => (),
        }

        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::LoadAllItems => {
                self.fetch_items();
            }
            Action::ItemsLoadingFinished { result } => {
                self.handle_loading_finished(result);
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
            Action::Inspect => {
                self.inspect();
            }
            Action::Escape => {
                self.set_scrolling_mode();
                if let Some(_) = &mut self.new_item_editor {
                    self.new_item_editor = None;
                }
            }
            Action::Delete => {
                self.popup_delete_confirmation();
            }
            Action::ItemDeleted { item_type, result } => {
                self.handle_item_deleted(item_type, result);
            }
            Action::NewItem => {
                self.enter_new_item_editor();
            }
            Action::CreateItem => {
                self.create_item();
            }
            Action::UpdateItem => {
                self.update_item();
            }
            Action::ItemCreated { result } => {
                self.handle_item_created(result);
            }
            Action::ItemUpdated { result } => {
                self.handle_item_updated(result);
            }
            Action::Enter => self.inspect(),
            Action::Copy => {
                self.copy_json();
            }
            Action::ClosePopup => {
                self.exit_popup();
            }
            Action::ConfirmPopup => {
                self.confirm_popup();
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
