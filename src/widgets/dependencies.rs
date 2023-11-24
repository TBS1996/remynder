use mischef::Widget;
use ratatui::prelude::Rect;
use speki_backend::Id;

use crate::{utils::TreeWidget, CardCache};

pub struct Dependencies<'a> {
    tree: TreeWidget<'a, Id>,
}

impl Widget for Dependencies<'_> {
    type AppData = CardCache;

    fn keyhandler(&mut self, app_data: &mut Self::AppData, key: crossterm::event::KeyEvent) {
        self.tree.keyhandler(app_data, key);
    }

    fn render(&mut self, f: &mut ratatui::Frame, app_data: &mut Self::AppData, area: Rect) {
        self.tree.render(f, app_data, area);
    }
}
