use crossterm::event::KeyCode;
use mischef::{Tab, TabData};
use speki_backend::{card::Card, categories::Category};

use crate::{widgets::file_finder::FileFinder, CardCache, ReturnType};

pub struct Importer {
    file_finder: FileFinder,
    tab_data: TabData<CardCache, ReturnType>,
}

impl Importer {
    pub fn new() -> Self {
        Self {
            file_finder: FileFinder::new(),
            tab_data: TabData::default(),
        }
    }
}

impl Tab for Importer {
    type AppState = CardCache;
    type ReturnType = ReturnType;

    fn tab_keyhandler_selected(
        &mut self,
        cache: &mut Self::AppState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        if key.code == KeyCode::Enter {
            if let Some(p) = self.file_finder.selected() {
                let cards = Card::import(p);
                let category = Category::root().append("imports");
                for card in cards {
                    card.save_new_card(&category, &mut cache.inner.lock().unwrap());
                }
            };
        }

        true
    }

    fn widgets(
        &mut self,
        area: ratatui::prelude::Rect,
    ) -> Vec<(
        &mut dyn mischef::Widget<AppData = Self::AppState>,
        ratatui::prelude::Rect,
    )> {
        vec![(&mut self.file_finder, area)]
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState, Self::ReturnType> {
        &mut self.tab_data
    }

    fn tabdata_ref(&self) -> &TabData<Self::AppState, Self::ReturnType> {
        &self.tab_data
    }

    fn title(&self) -> &str {
        "import"
    }
}
