use crate::components::popups::{draw_default_popup, Popup};
use ratatui::layout::Rect;
use ratatui::prelude::Color;

pub struct ErrorPopup {
    pub title: String,
    pub message: String,
}

impl Popup for ErrorPopup {
    fn draw_popup(
        &mut self,
        f: &mut crate::tui::Frame<'_>,
        popup_area: Rect,
    ) -> color_eyre::Result<()> {
        draw_default_popup(
            f,
            popup_area,
            self.title.clone(),
            self.message.clone(),
            Color::Red,
            "[Esc] Close".to_string(),
        );
        color_eyre::eyre::Ok(())
    }
}
