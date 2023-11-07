use mischef::Widget;
use ratatui::prelude::Rect;
use speki_backend::{cache::CardCache, Id};

use crate::utils::TreeWidget;

pub struct Dependencies<'a> {
    tree: TreeWidget<'a, Id>,
    area: Rect,
}

impl Widget for Dependencies<'_> {
    type AppData = CardCache;

    fn keyhandler(&mut self, app_data: &mut Self::AppData, key: crossterm::event::KeyEvent) {
        self.tree.keyhandler(app_data, key);
    }

    fn render(&mut self, f: &mut ratatui::Frame, app_data: &mut Self::AppData, area: Rect) {
        self.tree.render(f, app_data, area);
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
}
