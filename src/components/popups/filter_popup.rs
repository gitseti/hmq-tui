use std::ops::Not;
use std::vec;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use itertools::Itertools;
use ratatui::layout::Alignment::Center;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Span, Style, Stylize};
use ratatui::style::Styled;
use ratatui::symbols;
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use tui_textarea::{CursorMove, TextArea};
use crate::action::Action;
use crate::components::popups::filter_popup::Tab::{JsonPathSearch, KeywordSearch};
use crate::components::popups::Popup;
use crate::tui::Frame;

pub struct FilterPopup<'a> {
    title: &'static str,
    selected: usize,
    tab_names: Vec<&'static str>,
    tabs: Vec<Tab<'a>>,
}

pub enum Tab<'a> {
    KeywordSearch {
        text_area: TextArea<'a>,
        is_regex_checked: bool,
    },
    JsonPathSearch {
        text_area_json_path: TextArea<'a>,
        text_area_query: TextArea<'a>,
        selected_text_area: usize,
        is_regex_checked: bool,
    },
}

impl<'a> FilterPopup<'a> {
    pub fn new() -> Self {
        FilterPopup {
            title: "Filter Items",
            selected: 0,
            tab_names: vec![" Keyword Search ", " JSON Path Search "],
            tabs: vec![Self::new_keyword_search(), Self::new_json_path_search()],
        }
    }

    fn new_keyword_search() -> Tab<'a> {
        let mut text_area = TextArea::default();
        text_area.set_cursor_line_style(Style::default());
        text_area.set_placeholder_text("Enter a keyword to search...");
        text_area.set_block(Block::default().borders(Borders::ALL).title("Keyword"));

        KeywordSearch {
            text_area,
            is_regex_checked: true,
        }
    }

    fn new_json_path_search() -> Tab<'a> {
        let mut text_area_json_path = TextArea::new(vec!["$.".to_owned()]);
        text_area_json_path.set_cursor_line_style(Style::default());
        text_area_json_path.move_cursor(CursorMove::Jump(0, 2));
        text_area_json_path.set_block(Block::default().borders(Borders::ALL).title("JSON Path"));

        let mut text_area_query = TextArea::default();
        text_area_query.set_cursor_line_style(Style::default());
        text_area_query.set_placeholder_text("Enter query");
        text_area_query.set_block(Block::default().borders(Borders::ALL).title("Query").dim());

        JsonPathSearch {
            text_area_json_path,
            text_area_query,
            selected_text_area: 0,
            is_regex_checked: true,
        }
    }

    pub fn get_selected_tab(&self) -> &Tab {
        return &self.tabs[self.selected];
    }

    pub fn handle_key_events(&mut self, mut key: KeyEvent) -> color_eyre::Result<Option<Action>> {
        if key.code.eq(&KeyCode::Enter) {
            return color_eyre::eyre::Ok(None);
        }

        if key.code.eq(&KeyCode::Tab) {
            self.selected = (self.selected + 1) % self.tabs.len();
            return Ok(None);
        }

        if cfg!(target_os = "macos") {
            // crossterm doesn't handle these macos keybindings correctly
            if key.modifiers == KeyModifiers::ALT {
                match key.code {
                    KeyCode::Char('5') => {
                        key.code = KeyCode::Char('[');
                        key.modifiers = KeyModifiers::NONE;
                    }
                    KeyCode::Char('6') => {
                        key.code = KeyCode::Char(']');
                        key.modifiers = KeyModifiers::NONE;
                    }
                    KeyCode::Char('8') => {
                        key.code = KeyCode::Char('{');
                        key.modifiers = KeyModifiers::NONE;
                    }
                    KeyCode::Char('9') => {
                        key.code = KeyCode::Char('}');
                        key.modifiers = KeyModifiers::NONE;
                    }
                    _ => {}
                }
            }
        }

        match &mut self.tabs[self.selected] {
            KeywordSearch { text_area, is_regex_checked, .. } => {
                if key.code == KeyCode::F(1) {
                    *is_regex_checked = is_regex_checked.not();
                } else {
                    text_area.input(key);
                }
            }
            JsonPathSearch { selected_text_area, is_regex_checked, text_area_json_path, text_area_query } => {
                match key.code {
                    KeyCode::Up => *selected_text_area = {
                        0
                    },
                    KeyCode::Down => *selected_text_area = 1,
                    KeyCode::F(1) => *is_regex_checked = is_regex_checked.not(),
                    _ => ()
                };

                let (focus_text_area, dimmed_text_area) = if *selected_text_area == 0 {
                    (text_area_json_path, text_area_query)
                } else {
                    (text_area_query, text_area_json_path)
                };

                if let Some(block) = focus_text_area.block() {
                    focus_text_area.set_block(block.clone().not_dim())
                }

                if let Some(block) = dimmed_text_area.block() {
                    dimmed_text_area.set_block(block.clone().dim())
                }

                focus_text_area.input(key);
            }
        }

        color_eyre::eyre::Ok(None)
    }
}

impl<'a> Popup for FilterPopup<'a> {
    fn draw_popup(&mut self, f: &mut Frame<'_>, popup_area: Rect) -> color_eyre::Result<()> {
        let selected_tab = &self.tabs[self.selected];
        let block = Block::default()
            .title(self.title.clone())
            .title_alignment(Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        let inner = block.inner(popup_area);

        let tab_names = self.tab_names.clone();
        let tabs = Tabs::new(tab_names)
            .highlight_style(Style::default().bg(Color::Blue).not_dim().underlined())
            .style(Style::default().dim())
            .select(self.selected)
            .padding("", "");

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(1)
            .constraints(vec![Constraint::Min(1), Constraint::Min(1), Constraint::Percentage(100), Constraint::Min(1)])
            .split(inner);
        let tab_layout = layout[0];
        f.render_widget(
            Block::default().borders(Borders::BOTTOM).dim(),
            layout[1],
        );
        let popup_body = layout[2];
        let popup_footer = layout[3];

        f.render_widget(tabs, tab_layout);

        let footer = Paragraph::new("[Tab] Select Search  [Esc] Close  [Enter] Filter")
            .alignment(Center)
            .style(Style::default().fg(Color::Blue));

        f.render_widget(block, popup_area);

        match selected_tab {
            KeywordSearch { text_area, is_regex_checked, .. } => {
                let keyword_search_layout = Layout::vertical(Constraint::from_lengths([3, 2]))
                    .split(popup_body);

                f.render_widget(text_area.widget(), keyword_search_layout[0]);

                let regex_checkbox = if *is_regex_checked {
                    Paragraph::new("(*) RegEx Support [F1]")
                } else {
                    Paragraph::new("( ) RegEx Support [F1]")
                };

                f.render_widget(regex_checkbox, keyword_search_layout[1]);
            }
            JsonPathSearch { text_area_json_path, text_area_query, is_regex_checked, .. } => {
                let json_path_search_layout = Layout::vertical(Constraint::from_lengths([7, 2]))
                    .split(popup_body);

                let filter_layout = Layout::vertical(Constraint::from_lengths([3, 1, 3]))
                    .split(json_path_search_layout[0]);

                f.render_widget(text_area_json_path.widget(), filter_layout[0]);
                f.render_widget(Paragraph::new("Switch [↑ ↓]"), filter_layout[1]);
                f.render_widget(text_area_query.widget(), filter_layout[2]);

                let regex_checkbox = if *is_regex_checked {
                    Paragraph::new("(*) RegEx Support [F1]")
                } else {
                    Paragraph::new("( ) RegEx Support [F1]")
                };

                f.render_widget(regex_checkbox, json_path_search_layout[1]);
            }
        }

        f.render_widget(footer, popup_footer);

        color_eyre::eyre::Ok(())
    }

    fn percent_y(&self) -> u16 {
        60
    }
}