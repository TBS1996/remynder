use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::Rect;
use speki_backend::{cache::CardCache, card::Card, categories::Category};

use mischef::{Tab, TabData, Widget};

use crate::{
    popups::CatChoice,
    split_off,
    utils::{StatusBar, TextInput},
    vsplit2,
};

#[derive(Debug)]
pub struct AddCard<'a> {
    front: TextInput<'a>,
    back: TextInput<'a>,
    status_bar: StatusBar,
    category: Category,
    tabdata: TabData<CardCache>,
    message: String,
}

impl Default for AddCard<'_> {
    fn default() -> Self {
        Self {
            front: Default::default(),
            back: Default::default(),
            category: Category::root(),
            status_bar: StatusBar::default(),
            tabdata: TabData::default(),
            message: String::default(),
        }
    }
}

impl<'a> AddCard<'a> {
    pub fn new(message: impl Into<String>, category: Category) -> Self {
        let mut s = Self {
            category,
            message: message.into(),
            ..Default::default()
        };
        s.refresh();
        s
    }

    fn refresh(&mut self) {
        self.status_bar.text = format!("{}    {}", self.message, self.category.print_full());
    }
}

fn split_area(area: Rect) -> (Rect, Rect, Rect) {
    let (status, area) = split_off(area, 1, crate::Retning::Up);
    let (front, back) = vsplit2(area, 50, 50);
    (status, front, back)
}

impl Tab for AddCard<'_> {
    type AppState = CardCache;

    fn set_selection(&mut self, area: ratatui::prelude::Rect) {
        let (status, front, back) = split_area(area);

        self.front.set_area(front);
        self.back.set_area(back);
        self.status_bar.set_area(status);

        self.tabdata.areas.extend([front, back]);
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState> {
        &mut self.tabdata
    }

    fn widgets(&mut self) -> Vec<&mut dyn Widget<AppData = Self::AppState>> {
        vec![&mut self.front, &mut self.back, &mut self.status_bar]
    }

    fn title(&self) -> &str {
        "new card"
    }

    fn handle_popup_value(
        &mut self,
        _app_data: &mut Self::AppState,
        value: Box<dyn std::any::Any>,
    ) {
        let category = value.downcast::<Category>().unwrap();
        self.category = *category;
        self.refresh();
    }

    fn tab_keyhandler(&mut self, cache: &mut CardCache, key: KeyEvent) -> bool {
        let cursor = self.cursor();

        if self.tabdata.is_selected && self.front.is_selected(&cursor) && key.code == KeyCode::Enter
        {
            self.tabdata.move_to_area(self.back.area());
            return false;
        }

        if !self.selected() && key.code == KeyCode::Char('c') {
            self.set_popup(Box::new(CatChoice::new()));
        }

        if self.tabdata.is_selected && self.back.is_selected(&cursor) && key.code == KeyCode::Enter
        {
            let old_self = std::mem::take(self);
            self.category = old_self.category;
            self.refresh();

            let card = Card::new_simple(
                old_self.front.text.into_lines().join("\n"),
                old_self.back.text.into_lines().join("\n"),
            );

            if card.front.text.is_empty() {
                return false;
            };

            self.resolve_tab(Box::new(card.save_new_card(&self.category, cache)));

            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 100,
        };

        dbg!(split_area(area));
    }
}
