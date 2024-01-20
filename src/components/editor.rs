use std::iter::FromIterator;

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::{block::Block, Borders},
};
use tui::Frame;
use tui_textarea::{CursorMove, TextArea};

use crate::{action::Action, components::Component, tui};

pub struct Editor<'a> {
    textarea: TextArea<'a>,
    title: String,
    readonly: bool,
}

impl Editor<'_> {
    fn new(readonly: bool, text: String, title: String) -> Self {
        let mut textarea = TextArea::from_iter(text.lines());
        textarea.set_line_number_style(Style::default().dim());
        textarea.set_cursor_line_style(Style::default().not_underlined());
        textarea.move_cursor(CursorMove::Top);

        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(title.to_owned()),
        );

        Editor {
            textarea,
            title,
            readonly,
        }
    }

    pub fn readonly(text: String, title: String) -> Self {
        Editor::new(true, text, title)
    }

    pub fn writeable(title: String) -> Self {
        return Editor::new(false, "".to_owned(), title.to_owned());
    }

    pub fn focus(&mut self) {
        self.textarea.set_style(Style::default().not_dim());
        self.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(self.title.to_owned()),
        );
        self.textarea
            .set_cursor_style(Style::default().bg(Color::Blue));
    }

    pub fn unfocus(&mut self) {
        self.textarea.set_style(Style::default().dim());
        self.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .dim()
                .title(self.title.to_owned()),
        );
        self.textarea.set_cursor_style(Style::default().hidden());
    }

    pub fn get_text(&mut self) -> String {
        self.textarea.lines().join("\n")
    }
}

impl Component for Editor<'_> {
    fn handle_key_events(&mut self, mut key: KeyEvent) -> Result<Option<Action>> {
        if self.readonly {
            match key.code {
                KeyCode::Left => {}
                KeyCode::Right => {}
                KeyCode::Up => {}
                KeyCode::Down => {}
                KeyCode::Home => {}
                KeyCode::End => {}
                KeyCode::PageUp => {}
                KeyCode::PageDown => {}
                KeyCode::Esc => {}
                _ => return Ok(None),
            }
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

        self.textarea.input(key);
        Ok(None)
    }
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        f.render_widget(self.textarea.widget(), area);
        Ok(())
    }
}
