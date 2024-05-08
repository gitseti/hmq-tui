use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use arboard::Clipboard;
use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use indexmap::IndexSet;
use ratatui::text::Span;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Stylize,
    style::{Color, Modifier, Style, Styled},
    widgets::{block::Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use typed_builder::TypedBuilder;

use tui::Frame;
use ListPopup::FilterPopup;
use LoadingState::Loaded;

use crate::action::ListWithDetailsAction;
use crate::components::list_with_details::FocusMode::Editing;
use crate::components::list_with_details::ListPopup::{DeletePopup, ErrorPopup};
use crate::components::popup;
use crate::components::popup::{InputPopup, Popup};
use crate::repository::Repository;
use crate::{
    action::{Action, Action::SelectedItem},
    components::{
        editor::Editor,
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
pub struct Features {
    #[builder(setter(strip_bool))]
    pub deletable: bool,

    #[builder(setter(strip_bool))]
    pub creatable: bool,

    #[builder(setter(strip_bool))]
    updatable: bool,
}

#[derive(TypedBuilder)]
pub struct ListWithDetails<'a, T: Serialize + DeserializeOwned> {
    #[builder(setter(into))]
    list_title: String,

    #[builder(setter(into))]
    item_name: String,

    #[builder]
    features: Features,

    #[builder]
    repository: Arc<Repository<T>>,

    #[builder]
    mode: Rc<RefCell<Mode>>,

    #[builder(default =
    if features.creatable && features.deletable {
    Mode::FullTab
    } else if features.deletable {
    Mode::ReadDeleteTab
    } else {
    Mode::ReadTab
    }
    )]
    base_mode: Mode,

    #[builder(setter(skip), default =
    Loaded {
    items: IndexSet::new(),
    list: vec ! [],
    focus_mode: FocusMode::Scrolling(ListState::default()),
    filter: None
    })]
    loading_state: LoadingState<'a>,

    #[builder(setter(skip), default)]
    new_item_editor: Option<Editor<'a>>,

    #[builder(setter(skip), default)]
    popup: Option<ListPopup<'a>>,
}

pub enum LoadingState<'a> {
    LoadingError(String),
    Loading,
    Loaded {
        items: IndexSet<String>,
        list: Vec<ListItem<'a>>,
        focus_mode: FocusMode<'a>,
        filter: Option<String>,
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

impl<'a, T: Serialize + DeserializeOwned> ListWithDetails<'a, T> {
    pub fn reset(&mut self) {
        self.loading_state = Loaded {
            items: IndexSet::new(),
            list: vec![],
            focus_mode: FocusMode::Scrolling(ListState::default()),
            filter: None,
        };
    }

    pub fn set_items(&mut self, new_items: Vec<String>, filter: Option<String>) {
        self.loading_state = Loaded {
            items: IndexSet::new(),
            list: vec![],
            focus_mode: Scrolling(ListState::default()),
            filter,
        };
        if let Loaded { items, list, .. } = &mut self.loading_state {
            for item in new_items {
                items.insert(item.clone());
                list.push(ListItem::new(item.clone()));
            }
        }
    }

    pub fn put(&mut self, key: String) {
        if let Loaded { items, list, .. } = &mut self.loading_state {
            let old_value = items.insert(key.to_owned());
            if old_value {
                list.push(ListItem::new(key.clone()));
            }
        }
    }

    pub fn remove(&mut self, key: String) {
        let Loaded {
            items,
            list,
            focus_mode,
            ..
        } = &mut self.loading_state
        else {
            return;
        };

        let Some((index, _item)) = items.shift_remove_full(&key) else {
            return;
        };

        list.remove(index);

        let list_state = match focus_mode {
            FocusMode::Scrolling(list_state) => list_state,
            FocusMode::Editing { list_state, .. } => list_state,
            _ => return,
        };

        if index > list.len().saturating_sub(1) {
            list_state.select(Some(index.saturating_sub(1)))
        }

        if list.len() == 0 {
            list_state.select(None);
        }
    }

    pub fn get(&self, key: &str) -> Option<T> {
        if let Loaded { items, .. } = &self.loading_state {
            if let None = items.get(key) {
                return None;
            }
            match self.repository.find_by_id(key) {
                Ok(item) => Some(item),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn get_selected(&self) -> Option<(&String, T)> {
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

        let Some(key) = items.get_index(index) else {
            return None;
        };

        match self.repository.find_by_id(key) {
            Ok(item) => Some((key, item)),
            Err(_) => None,
        }
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

    fn next_item(&mut self) -> Option<(&String, T)> {
        let Loaded {
            list, focus_mode, ..
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
                self.get_selected()
            }
            FocusMode::DetailsError { .. } => {
                *focus_mode = FocusMode::Scrolling(ListState::default());
                None
            }
            _ => None,
        }
    }

    fn prev_item(&mut self) -> Option<(&String, T)> {
        let Loaded { focus_mode, .. } = &mut self.loading_state else {
            return None;
        };

        match focus_mode {
            FocusMode::Scrolling(list_state) => {
                let new_selected = match list_state.selected() {
                    Some(i) if i > 0 => i - 1,
                    _ => return None,
                };

                list_state.select(Some(new_selected));
                self.get_selected()
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
                let key = items.get_index(selected).unwrap();
                let item = self.repository.find_by_id(key).unwrap();
                let details = serde_json::to_string_pretty(&item).unwrap();
                let mut clipboard = Clipboard::new().or_else(|err| Err(err.to_string()))?;
                clipboard.set_text(details).unwrap();
            }
        }

        Ok(())
    }

    fn inspect(&mut self) {
        let Some((_, item)) = self.get_selected() else {
            return;
        };

        let Loaded { focus_mode, .. } = &mut self.loading_state else {
            return;
        };

        let FocusMode::Scrolling(list_state) = focus_mode else {
            return;
        };

        let Some(_selected) = list_state.selected() else {
            return;
        };

        let item = serde_json::to_string_pretty(&item).unwrap();
        if self.features.updatable {
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
    fn confirm_popup(&mut self) -> Option<Action> {
        let Some(popup) = &self.popup else {
            return None;
        };

        match popup {
            DeletePopup { item_id, .. } => {
                Some(Action::LWD(ListWithDetailsAction::Delete(item_id.clone())))
            }
            ErrorPopup { .. } => None,
            FilterPopup { popup, .. } => {
                let result = self.repository.find_ids_by("$", popup.get_text().as_str());
                self.set_items(result.unwrap(), Some(popup.get_text().to_string()));
                self.exit_popup();
                None
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

    fn handle_item_updated(&mut self, result: Result<String, String>) {
        self.set_scrolling_mode();
        match result {
            Err(message) => {
                self.details_error(format!("{} update failed", self.item_name), message);
            }
            _ => {}
        }
    }

    fn handle_item_created(&mut self, result: Result<String, String>) {
        self.new_item_editor = None;
        match result {
            Ok(key) => {
                if self.repository.find_by_id(&key).is_ok() {
                    self.put(key.clone());
                    self.select_item(key);
                }
            }
            Err(error) => {
                self.details_error(format!("{} creation failed", self.item_name), error);
            }
        }
        *self.mode.borrow_mut() = self.base_mode;
    }

    fn update_item(&mut self) -> Option<Action> {
        if let Loaded {
            focus_mode: Editing { editor, .. },
            ..
        } = &self.loading_state
        {
            Some(Action::LWD(ListWithDetailsAction::Update(
                editor.get_text(),
            )))
        } else {
            None
        }
    }

    fn create_item(&mut self) -> Option<Action> {
        if let Some(editor) = &mut self.new_item_editor {
            Some(Action::LWD(ListWithDetailsAction::Create(
                editor.get_text(),
            )))
        } else {
            None
        }
    }

    fn enter_new_item_editor(&mut self) {
        if self.features.creatable {
            self.set_scrolling_mode();
            self.new_item_editor = Some(Editor::writeable(
                format!("New {}", self.item_name).to_owned(),
            ));
            *self.mode.borrow_mut() = Mode::CreateEditor;
        }
    }

    fn handle_item_deleted(&mut self, item_name: String, result: Result<String, String>) {
        if item_name.eq(&self.item_name) {
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
            if self.features.deletable {
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

    pub fn popup_filter(&mut self) {
        let popup = InputPopup::new("Filter Items");
        self.enter_popup(FilterPopup { popup })
    }

    fn handle_loading_finished(&mut self, result: Result<(), String>) {
        match result {
            Ok(_) => {
                let items = self.repository.find_all_ids().unwrap();
                self.set_items(items, None);
            }
            Err(msg) => self.list_error(&msg),
        }
    }

    pub fn error_popup(&mut self, popup: popup::ErrorPopup) {
        self.enter_popup(ErrorPopup { popup });
    }

    fn enter_popup(&mut self, popup: ListPopup<'a>) {
        *self.mode.borrow_mut() = match popup {
            DeletePopup { .. } => Mode::ConfirmPopup,
            ErrorPopup { .. } => Mode::ErrorPopup,
            FilterPopup { .. } => Mode::FilterPopup,
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
                filter,
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

                let mut title_spans = Vec::with_capacity(2);
                let title = Span::raw(format!(
                    "{} ({}/{})",
                    list_title,
                    list_state.selected().map_or(0, |i| i + 1),
                    list.len()
                ));
                title_spans.push(title);
                if let Some(filter_str) = filter {
                    let filter_title = Span::default()
                        .content(format!(" filtered by '{}'", &filter_str))
                        .style(Style::default().fg(Color::Blue));
                    title_spans.push(filter_title);
                }

                let list_widget = List::new(list.clone())
                    .block(Block::default().borders(Borders::ALL).title(title_spans))
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
                            let item = self.repository.find_by_id(item).unwrap();
                            let mut editor = Editor::readonly(
                                serde_json::to_string_pretty(&item).unwrap(),
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
                FilterPopup { popup } => popup,
            };
            popup.draw(f, f.size()).unwrap();
        }

        Ok(())
    }
}

impl<T: Serialize + DeserializeOwned> Component for ListWithDetails<'_, T> {
    fn activate(&mut self) -> Result<()> {
        *self.mode.borrow_mut() = self.base_mode;
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if let Some(ListPopup::FilterPopup { popup }) = &mut self.popup {
            return popup.handle_key_events(key);
        }

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
                self.loading();
            }
            Action::ItemsLoadingFinished { item_name, result } => {
                if self.item_name.eq(&item_name) {
                    self.handle_loading_finished(result);
                }
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
                if let Loaded {
                    filter: Some(_), ..
                } = &self.loading_state
                {
                    let items = self.repository.find_all_ids().unwrap();
                    self.set_items(items, None);
                }
            }
            Action::Delete => {
                self.popup_delete_confirmation();
            }
            Action::Filter => {
                self.popup_filter();
            }
            Action::ItemDeleted { item_name, result } => {
                self.handle_item_deleted(item_name, result);
            }
            Action::NewItem => {
                self.enter_new_item_editor();
            }
            Action::CreateItem => {
                return Ok(self.create_item());
            }
            Action::UpdateItem => {
                return Ok(self.update_item());
            }
            Action::ItemCreated { item_name, result } => {
                if self.item_name.eq(&item_name) {
                    self.handle_item_created(result);
                }
            }
            Action::ItemUpdated { item_name, result } => {
                if self.item_name.eq(&item_name) {
                    self.handle_item_updated(result);
                }
            }
            Action::Enter => self.inspect(),
            Action::Copy => {
                self.copy_json();
            }
            Action::ClosePopup => {
                self.exit_popup();
            }
            Action::ConfirmPopup => {
                return Ok(self.confirm_popup());
            }
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        self.draw_custom(f, area, None)
    }
}

enum ListPopup<'a> {
    DeletePopup {
        popup: popup::ConfirmPopup,
        item_id: String,
    },
    ErrorPopup {
        popup: popup::ErrorPopup,
    },
    FilterPopup {
        popup: popup::InputPopup<'a>,
    },
}
