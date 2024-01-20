use color_eyre::eyre::{Ok, Result};
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment::Center, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
    Frame,
};
use serde::Serialize;
use tokio::sync::mpsc::{self, UnboundedSender};

use super::Component;
use crate::{
    action::{self, Action},
    config::Config,
    tui::Event,
};

pub struct ConfirmPopup {
    pub title: String,
    pub message: String,
    pub tx: UnboundedSender<Action>,
    pub action: Action,
}

impl Popup for ConfirmPopup {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Escape => Ok(Some(Action::ClosePopup)),
            Action::Enter => {
                self.tx.send(self.action.clone()).unwrap();
                Ok(Some(Action::ClosePopup))
            }
            _ => Ok(None),
        }
    }

    fn draw_popup(&mut self, f: &mut crate::tui::Frame<'_>, popup_area: Rect) -> Result<()> {
        draw_default_popup(
            f,
            popup_area,
            self.title.clone(),
            self.message.clone(),
            Color::Blue,
            "[ESC] Escape  [ENTER] Confirm".to_string(),
        );
        Ok(())
    }
}

pub struct ErrorPopup {
    pub title: String,
    pub message: String,
}

impl Popup for ErrorPopup {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Escape => Ok(Some(Action::ClosePopup)),
            _ => Ok(None),
        }
    }

    fn draw_popup(&mut self, f: &mut crate::tui::Frame<'_>, popup_area: Rect) -> Result<()> {
        draw_default_popup(
            f,
            popup_area,
            self.title.clone(),
            self.message.clone(),
            Color::Red,
            "[ESC] Escape".to_string(),
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
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }
    fn draw_popup(&mut self, f: &mut crate::tui::Frame<'_>, popup_area: Rect) -> Result<()>;

    fn draw(&mut self, f: &mut crate::tui::Frame<'_>, area: Rect) -> Result<()> {
        let popup_area = popup_rect(60, 60, f.size());
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
