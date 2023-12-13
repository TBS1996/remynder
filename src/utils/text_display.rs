use ratatui::{
    style::Style,
    widgets::{Paragraph, Wrap},
    Frame,
};

use mischef::Widget;

use crate::CardCache;

#[derive(Default, Debug)]
pub struct TextDisplay {
    pub text: String,
}

impl TextDisplay {
    pub fn new(s: String) -> Self {
        Self { text: s }
    }
}

impl Widget for TextDisplay {
    type AppData = CardCache;

    fn keyhandler(&mut self, _cache: &mut CardCache, _key: crossterm::event::KeyEvent) {}

    fn render(&mut self, f: &mut Frame, _cache: &mut CardCache, area: ratatui::layout::Rect) {
        f.render_widget(
            Paragraph::new(self.text.as_str())
                .wrap(Wrap { trim: true })
                .style(Style {
                    //fg: Some(to_color(self.text.clone())),
                    ..Default::default()
                }),
            area,
        );
    }
}
