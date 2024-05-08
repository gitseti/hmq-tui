use color_eyre::eyre::{Ok, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment::Center, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};
use tui_textarea::TextArea;
use crate::action::Action;
use crate::tui::Frame;

pub struct ConfirmPopup {
    pub title: String,
    pub message: String,
}

impl Popup for ConfirmPopup {
    fn draw_popup(&mut self, f: &mut crate::tui::Frame<'_>, popup_area: Rect) -> Result<()> {
        draw_default_popup(
            f,
            popup_area,
            self.title.clone(),
            self.message.clone(),
            Color::Blue,
            "[Esc] Close  [Enter] Confirm".to_string(),
        );
        Ok(())
    }
}

pub struct ErrorPopup {
    pub title: String,
    pub message: String,
}

impl Popup for ErrorPopup {
    fn draw_popup(&mut self, f: &mut crate::tui::Frame<'_>, popup_area: Rect) -> Result<()> {
        draw_default_popup(
            f,
            popup_area,
            self.title.clone(),
            self.message.clone(),
            Color::Red,
            "[Esc] Close".to_string(),
        );
        Ok(())
    }
}

fn draw_default_popup(
    f: &mut crate::tui::Frame<'_>,
    popup_area: Rect,
    title: String,
    message: String,
    color: Color,
    footer: String,
) {
    let block = Block::default()
        .title(title)
        .title_alignment(Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));

    let inner = block.inner(popup_area);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(100), Constraint::Min(1)])
        .split(inner);
    let popup_body = layout[0];
    let popup_footer = layout[1];

    let message = Paragraph::new(message)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(color));

    let footer = Paragraph::new(footer)
        .alignment(Center)
        .style(Style::default().fg(color));

    f.render_widget(block, popup_area);
    f.render_widget(message, popup_body);
    f.render_widget(footer, popup_footer);
}

pub trait Popup {
    fn draw_popup(&mut self, f: &mut crate::tui::Frame<'_>, popup_area: Rect) -> Result<()>;
    fn draw(&mut self, f: &mut crate::tui::Frame<'_>, _area: Rect) -> Result<()> {
        let popup_area = popup_rect(60, 40, f.size());
        f.render_widget(Dim, f.size()); // Dim the whole tui area
        f.render_widget(Clear, popup_area); // Reset the area for the popup
        Ok(self.draw_popup(f, popup_area)?)
    }
}

fn popup_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub struct InputPopup<'a> {
    pub title: String,
    pub text_area: TextArea<'a>
}

impl<'a> InputPopup<'a> {

    pub fn new(title: &str) -> Self {
        let mut text_area = TextArea::default();
        text_area.set_cursor_line_style(Style::default());
        text_area.set_placeholder_text("Enter a filter term");
        InputPopup {
            title: title.to_string(),
            text_area
        }
    }

    pub fn handle_key_events(&mut self, mut key: KeyEvent) -> Result<Option<Action>> {
        if key.code.eq(&KeyCode::Enter) {
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

        self.text_area.input(key);
        Ok(None)
    }

    pub fn get_text(&self) -> String {
        self.text_area.lines().join("\n")
    }
}

impl<'a> Popup for InputPopup<'a> {
    fn draw_popup(&mut self, f: &mut Frame<'_>, popup_area: Rect) -> Result<()> {
        let block = Block::default()
            .title(self.title.clone())
            .title_alignment(Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        let inner = block.inner(popup_area);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100), Constraint::Min(1)])
            .split(inner);
        let popup_body = layout[0];
        let popup_footer = layout[1];

        let footer = Paragraph::new("[Esc] Close  [Enter] Filter")
            .alignment(Center)
            .style(Style::default().fg(Color::Blue));

        f.render_widget(block, popup_area);
        f.render_widget(self.text_area.widget(), popup_body);
        f.render_widget(footer, popup_footer);

        Ok(())
    }
}


#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Dim;

impl Widget for Dim {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf.get_mut(x, y).set_style(Style::default().dim());
            }
        }
    }
}
