use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::Rect;
use speki_backend::{
    cache::CardCache,
    card::{Card, SavedCard},
    categories::Category,
};

use mischef::{Tab, View, Widget};

use crate::{
    popups::{CatChoice, PopUpState},
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
    view: View,
    choose_category: Option<CatChoice<'a>>,
    message: String,
    pub popstate: PopUpState<SavedCard>,
}

impl Default for AddCard<'_> {
    fn default() -> Self {
        Self {
            front: Default::default(),
            back: Default::default(),
            category: Category::root(),
            status_bar: StatusBar::default(),
            view: View::default(),
            choose_category: None,
            popstate: PopUpState::Continue,
            message: String::default(),
        }
    }
}

impl<'a> AddCard<'a> {
    pub fn new(message: String, category: Category) -> Self {
        let mut s = Self {
            category,
            message,
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
    type AppData = CardCache;

    fn set_selection(&mut self, area: ratatui::prelude::Rect) {
        let (status, front, back) = split_area(area);

        self.front.set_area(front);
        self.back.set_area(back);
        self.status_bar.set_area(status);

        self.view.areas.extend([front, back]);
    }

    fn view(&mut self) -> &mut View {
        &mut self.view
    }

    fn widgets(&mut self) -> Vec<&mut dyn Widget<AppData = Self::AppData>> {
        vec![&mut self.front, &mut self.back, &mut self.status_bar]
    }

    fn title(&self) -> &str {
        "new card"
    }

    fn check_popup_value(&mut self, _cache: &mut CardCache) {
        let mut flag = false;
        if let Some(popup) = &self.choose_category {
            match &popup.popup_state {
                PopUpState::Exit => {
                    // todo fix
                    flag = true;
                }
                PopUpState::Continue => {}
                PopUpState::Resolve(category) => {
                    self.category = category.to_owned();
                    self.refresh();
                    flag = true;
                }
            }
        }
        if flag {
            self.choose_category = None;
        }
    }

    fn pop_up(&mut self) -> Option<&mut dyn Tab<AppData = CardCache>> {
        self.choose_category
            .as_mut()
            .map(|c| c as &mut dyn Tab<AppData = CardCache>)
    }

    fn tab_keyhandler(&mut self, cache: &mut CardCache, key: KeyEvent) -> bool {
        let cursor = *self.cursor();

        if self.view.is_selected && self.front.is_selected(&cursor) && key.code == KeyCode::Enter {
            self.view.move_to_area(self.back.area());
            return false;
        }

        if !self.selected() && key.code == KeyCode::Char('c') {
            self.choose_category = Some(CatChoice::new());
        }

        if self.view.is_selected && self.back.is_selected(&cursor) && key.code == KeyCode::Enter {
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

            self.popstate = PopUpState::Resolve(card.save_new_card(&self.category, cache));

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
