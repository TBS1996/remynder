use std::ops::ControlFlow;

use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    prelude::Rect,
    style::Style,
    widgets::{Block, Borders},
    Frame,
};
use speki_backend::cache::CardCache;

use crate::Retning;

#[derive(Debug, Clone, Copy, Default)]
pub struct Pos {
    x: u16,
    y: u16,
}

impl Pos {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

impl View {
    pub fn _debug_show_cursor(&self, f: &mut Frame) {
        f.set_cursor(self.cursor.x, self.cursor.y);
    }

    pub fn validate_pos(&mut self) {
        for area in self.areas.iter() {
            if self.is_selected(*area) {
                return;
            }
        }
        if !self.areas.is_empty() {
            self.move_to_area(self.areas[0]);
        }
    }

    pub fn move_to_area(&mut self, area: Rect) {
        let x = area.x + area.width / 2;
        let y = area.y + area.height / 2;
        self.cursor = Pos::new(x, y);
    }

    pub fn is_selected(&self, area: Rect) -> bool {
        Self::isitselected(area, &self.cursor)
    }

    fn is_valid_pos(&self, pos: &Pos) -> bool {
        for area in &self.areas {
            if Self::isitselected(*area, pos) {
                return true;
            }
        }
        false
    }

    fn current_area(&self) -> &Rect {
        self.areas
            .iter()
            .find(|area| Self::isitselected(**area, &self.cursor))
            .unwrap()
    }

    pub fn isitselected(area: Rect, cursor: &Pos) -> bool {
        cursor.x >= area.left()
            && cursor.x < area.right()
            && cursor.y >= area.top()
            && cursor.y < area.bottom()
    }

    pub fn move_right(&mut self) {
        let current_area = self.current_area();
        let new_pos = Pos {
            x: current_area.right(),
            y: self.cursor.y,
        };
        if self.is_valid_pos(&new_pos) {
            self.cursor = new_pos;
        }
    }

    pub fn move_down(&mut self) {
        let current_area = self.current_area();
        let new_pos = Pos {
            y: current_area.bottom(),
            x: self.cursor.x,
        };
        if self.is_valid_pos(&new_pos) {
            self.cursor = new_pos;
        }
    }

    pub fn move_up(&mut self) {
        let current_area = self.current_area();
        let new_pos = Pos {
            x: self.cursor.x,
            y: current_area.top().saturating_sub(1),
        };
        if self.is_valid_pos(&new_pos) {
            self.cursor = new_pos;
        }
    }

    pub fn move_left(&mut self) {
        let current_area = self.current_area();
        let new_pos = Pos {
            x: current_area.left().saturating_sub(1),
            y: self.cursor.y,
        };
        if self.is_valid_pos(&new_pos) {
            self.cursor = new_pos;
        }
    }

    pub fn navigate(&mut self, direction: Retning) {
        match direction {
            Retning::Up => self.move_up(),
            Retning::Down => self.move_down(),
            Retning::Left => self.move_left(),
            Retning::Right => self.move_right(),
        }
    }
}

pub trait Widget {
    fn keyhandler(&mut self, cache: &mut CardCache, key: KeyEvent);
    fn main_render(&mut self, f: &mut Frame, cache: &mut CardCache, view: &View) {
        let rect = self.draw_titled_border(f, view);
        self.render(f, cache, rect);
    }

    fn render(&mut self, f: &mut Frame, cache: &mut CardCache, area: Rect);
    fn area(&self) -> Rect;
    fn set_area(&mut self, area: Rect);
    fn title(&self) -> &str {
        ""
    }

    fn draw_titled_border(&self, f: &mut Frame, view: &View) -> Rect {
        let block = Block::default().title(self.title()).borders(Borders::ALL);

        let block = if View::isitselected(self.area(), &view.cursor) {
            if view.is_selected {
                block.border_style(Style {
                    fg: Some(ratatui::style::Color::Red),
                    ..Default::default()
                })
            } else {
                block.border_style(Style {
                    fg: Some(ratatui::style::Color::Black),
                    ..Default::default()
                })
            }
        } else {
            block.border_style(Style {
                fg: Some(ratatui::style::Color::White),
                ..Default::default()
            })
        };

        let rect = self.area();

        if rect.width < 3 || rect.height < 3 {
            return rect;
        }

        f.render_widget(block, rect);

        Rect {
            x: rect.x + 1,
            y: rect.y + 1,
            width: rect.width.saturating_sub(2),
            height: rect.height.saturating_sub(2),
        }
    }

    fn is_selected(&self, cursor: &Pos) -> bool {
        View::isitselected(self.area(), cursor)
    }
}

#[derive(Clone, Debug, Default)]
pub struct View {
    pub areas: Vec<Rect>,
    pub cursor: Pos,
    pub is_selected: bool,
}

pub trait Tab {
    fn entry_keyhandler(&mut self, key: Event, cache: &mut CardCache) -> ControlFlow<()> {
        if let Some(popup) = self.pop_up() {
            return popup.entry_keyhandler(key, cache);
        }

        let key = match key {
            Event::Key(x) => x,
            // todo find out why it doesnt work
            Event::Mouse(x) => {
                self.view().cursor = Pos {
                    y: x.row,
                    x: x.column,
                };
                return ControlFlow::Continue(());
            }
            _ => {
                return ControlFlow::Continue(());
            }
        };

        if !self.selected() && key.code == KeyCode::Esc {
            return ControlFlow::Break(());
        } else if self.selected() && key.code == KeyCode::Esc {
            self.view().is_selected = false;
            return ControlFlow::Continue(());
        } else if let Ok(ret) = Retning::try_from(key) {
            if !self.selected() {
                self.navigate(ret);
                return ControlFlow::Continue(());
            }
        }

        if self.tab_keyhandler(cache, key) {
            if !self.selected() && key.code == KeyCode::Char(' ') || key.code == KeyCode::Enter {
                self.view().is_selected = true;
            } else {
                self.widget_keyhandler(cache, key);
            }
        }

        self.after_keyhandler(cache);

        ControlFlow::Continue(())
    }

    // Keyhandling that requires the state of the object.
    // bool represents whether the tab 'captures' the key
    // or passes it onto the widget in focus
    fn tab_keyhandler(
        &mut self,
        _cache: &mut speki_backend::cache::CardCache,
        _key: crossterm::event::KeyEvent,
    ) -> bool {
        true
    }

    // Keyhandler that only mutates the widget itself.
    fn widget_keyhandler(
        &mut self,
        cache: &mut speki_backend::cache::CardCache,
        key: crossterm::event::KeyEvent,
    ) {
        let cursor = *self.cursor();
        for widget in self.widgets() {
            if widget.is_selected(&cursor) {
                widget.keyhandler(cache, key);
                return;
            }
        }
    }

    fn entry_render(&mut self, f: &mut Frame, cache: &mut CardCache, area: Rect) {
        self.check_popup_value();

        match self.pop_up() {
            Some(pop_up) => pop_up.entry_render(f, cache, area),
            None => {
                self.view().areas.clear();
                self.set_selection(area);
                self.view().validate_pos();
                self.render(f, cache);
            }
        }
    }

    fn render(&mut self, f: &mut ratatui::Frame, cache: &mut speki_backend::cache::CardCache) {
        let view = self.view().clone();
        for widget in self.widgets() {
            widget.main_render(f, cache, &view);
        }
    }

    fn after_keyhandler(&mut self, _cache: &mut CardCache) {}

    fn set_selection(&mut self, area: Rect);

    fn view(&mut self) -> &mut View;

    fn cursor(&mut self) -> &Pos {
        &self.view().cursor
    }

    fn selected(&mut self) -> bool {
        self.view().is_selected
    }

    fn navigate(&mut self, dir: Retning) {
        self.view().navigate(dir);
    }

    fn widgets(&mut self) -> Vec<&mut dyn Widget>;

    fn title(&self) -> &str;

    fn pop_up(&mut self) -> Option<&mut dyn Tab> {
        None
    }

    // Check if the popup has resolved
    fn check_popup_value(&mut self) {}
}
