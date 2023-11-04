use crossterm::event::KeyCode;
use ratatui::{
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use speki_backend::{cache::CardCache, Id};
use tui_textarea::TextArea;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use mischef::Widget;

#[derive(Default, Debug)]
pub struct StatusBar {
    pub text: String,
    area: Rect,
}

impl Widget for StatusBar {
    type AppData = CardCache;

    fn keyhandler(&mut self, _cache: &mut CardCache, _key: crossterm::event::KeyEvent) {}

    fn render(&mut self, f: &mut Frame, _cache: &mut CardCache, area: ratatui::layout::Rect) {
        f.render_widget(
            Paragraph::new(self.text.as_str()).wrap(Wrap { trim: true }),
            area,
        );
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
}
