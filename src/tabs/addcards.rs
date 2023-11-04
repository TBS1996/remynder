use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::Rect;
use speki_backend::{cache::CardCache, card::Card, categories::Category, Id};

use mischef::{Tab, View, Widget};

use crate::{
    popups::{AddCard, CatChoice, PopUpState},
    split_off,
    utils::{StatusBar, TextInput},
    vsplit2,
};

pub struct CardAdder<'a> {
    add_card: AddCard<'a>,
}

impl CardAdder<'_> {
    pub fn new() -> Self {
        Self {
            add_card: AddCard::new("create new card".into(), Category::root()),
        }
    }
}

impl Tab for CardAdder<'_> {
    type AppData = CardCache;

    fn set_selection(&mut self, area: Rect) {
        self.add_card.set_selection(area);
    }

    fn view(&mut self) -> &mut View {
        self.add_card.view()
    }

    fn widgets(&mut self) -> Vec<&mut dyn Widget<AppData = Self::AppData>> {
        self.add_card.widgets()
    }

    fn title(&self) -> &str {
        "add cards"
    }

    fn pop_up(&mut self) -> Option<&mut dyn Tab<AppData = Self::AppData>> {
        Some(&mut self.add_card)
    }

    fn check_popup_value(&mut self, _appdata: &mut CardCache) {
        match &self.add_card.popstate {
            PopUpState::Exit => {}
            PopUpState::Continue => {}
            PopUpState::Resolve(c) => {
                *self = Self {
                    add_card: AddCard::new("create new card".to_string(), c.category().to_owned()),
                }
            }
        }
    }
}
