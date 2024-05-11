use color_eyre::eyre::{Ok, Result};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment::Center, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::tui::Frame;

pub mod confirm_popup;
pub mod error_popup;
pub mod filter_popup;


pub trait Popup {
    fn draw_popup(&mut self, f: &mut Frame<'_>, popup_area: Rect) -> Result<()>;

    fn percent_y(&self) -> u16 {
        40
    }

    fn percent_x(&self) -> u16 {
        60
    }
    fn draw(&mut self, f: &mut Frame<'_>, _area: Rect) -> Result<()> {
        let popup_area = popup_rect(self.percent_x(), self.percent_y(), f.size());
        f.render_widget(Dim, f.size()); // Dim the whole tui area
        f.render_widget(Clear, popup_area); // Reset the area for the popup
        Ok(self.draw_popup(f, popup_area)?)
    }
}

pub fn draw_default_popup(
    f: &mut Frame<'_>,
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
