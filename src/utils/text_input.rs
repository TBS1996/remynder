use mischef::{TabData, Widget};
use ratatui::{
    style::Style,
    widgets::{Paragraph, Wrap},
    Frame,
};
use tui_textarea::TextArea;

use crate::CardCache;

#[derive(Default, Debug)]
pub struct TextInput<'a> {
    pub text: TextArea<'a>,
    is_selected: bool,
    pub hide_text: bool,
}

impl TextInput<'_> {
    pub fn new(s: String) -> Self {
        let lines: Vec<String> = s.split('\n').map(|x| x.to_string()).collect();
        Self {
            text: TextArea::new(lines),
            is_selected: false,
            hide_text: false,
        }
    }

    pub fn clear(&mut self) {
        self.text = TextArea::new(vec![]);
    }

    pub fn get_text(&self) -> String {
        self.text.lines().join("\n")
    }
}

impl Widget for TextInput<'_> {
    type AppData = CardCache;
    fn keyhandler(&mut self, _cache: &mut CardCache, key: crossterm::event::KeyEvent) {
        self.text.input(key);
    }

    fn main_render(
        &mut self,
        f: &mut Frame,
        app_data: &mut Self::AppData,
        is_selected: bool,
        cursor: mischef::Pos,
        area: ratatui::prelude::Rect,
    ) {
        let rect = self.draw_titled_border(f, is_selected, cursor, area);
        self.is_selected = is_selected && TabData::<(), ()>::isitselected(area, cursor);
        self.render(f, app_data, rect);
    }

    fn render(&mut self, f: &mut Frame, _cache: &mut CardCache, area: ratatui::layout::Rect) {
        //f.render_widget(self.text.widget(), area);
        let mut lines = self.text.lines().to_vec();
        let cursor = self.text.cursor();
        let row = &mut lines[cursor.0];
        if self.is_selected {
            *row = replace_or_append_char(row, '█', cursor.1);
        }

        let mut text = lines.join("\n");

        if self.hide_text {
            text.clear();
        }

        f.render_widget(
            Paragraph::new(text).wrap(Wrap { trim: true }).style(Style {
                //fg: Some(to_color(self.text.clone())),
                ..Default::default()
            }),
            area,
        );
    }
}

fn replace_or_append_char(input: &mut String, replacement: char, idx: usize) -> String {
    let mut result = String::with_capacity(input.len() + 1);
    let mut count = 0;
    let mut flag = false;

    for ch in input.chars() {
        count += 1;
        if count > idx && !flag {
            result.push(replacement);
            flag = true;
        } else {
            result.push(ch);
        }
    }

    if count <= idx && !flag {
        result.push(replacement);
    }

    result
}
