use mischef::Widget;
use ratatui::{prelude::Rect, Frame};
use speki_backend::cache::CardCache;
use tui_textarea::TextArea;

#[derive(Default, Debug)]
pub struct TextInput<'a> {
    pub text: TextArea<'a>,
    area: Rect,
}

impl Widget for TextInput<'_> {
    type AppData = CardCache;
    fn keyhandler(&mut self, _cache: &mut CardCache, key: crossterm::event::KeyEvent) {
        self.text.input(key);
    }

    fn render(&mut self, f: &mut Frame, _cache: &mut CardCache, area: ratatui::layout::Rect) {
        f.render_widget(self.text.widget(), area);
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
}
