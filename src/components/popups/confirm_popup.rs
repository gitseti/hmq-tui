use crate::components::popups::{draw_default_popup, Popup};
use ratatui::layout::Rect;
use ratatui::prelude::Color;

pub struct ConfirmPopup {
    pub title: String,
    pub message: String,
}

impl Popup for ConfirmPopup {
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
            Color::Blue,
            "[Esc] Close  [Enter] Confirm".to_string(),
        );
        color_eyre::eyre::Ok(())
    }
}
