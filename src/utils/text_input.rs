use mischef::Widget;
use ratatui::Frame;
use tui_textarea::TextArea;

use crate::CardCache;

#[derive(Default, Debug)]
pub struct TextInput<'a> {
    pub text: TextArea<'a>,
}

impl TextInput<'_> {
    pub fn new(s: String) -> Self {
        let lines: Vec<String> = s.split('\n').map(|x| x.to_string()).collect();
        Self {
            text: TextArea::new(lines),
        }
    }
}

impl Widget for TextInput<'_> {
    type AppData = CardCache;
    fn keyhandler(&mut self, _cache: &mut CardCache, key: crossterm::event::KeyEvent) {
        self.text.input(key);
    }

    fn render(&mut self, f: &mut Frame, _cache: &mut CardCache, area: ratatui::layout::Rect) {
        f.render_widget(self.text.widget(), area);
    }
}
