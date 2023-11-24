use ratatui::prelude::Rect;
use speki_backend::{card::SavedCard, categories::Category};

use mischef::{PopUpState, Tab, TabData, Widget};

use crate::{popups::AddCard, CardCache};

/// Just a thin wrapper around AddCard because I wanted a popup that creates a single card,
/// and this keeps the code dry.
pub struct CardAdder<'a> {
    add_card: AddCard<'a>,
    category: Category,
    tab_data: TabData<CardCache>,
}

impl CardAdder<'_> {
    pub fn new(cache: &mut CardCache) -> Self {
        let category = cache
            .all_ids()
            .first()
            .map(|id| cache.get_ref(*id).category().clone())
            .unwrap_or_default();

        let mut s = Self {
            add_card: AddCard::new("create new card", Category::root(), None),
            category: category.clone(),
            tab_data: TabData::default(),
        };

        s.set_popup(Box::new(AddCard::new("add new card", category, None)));
        s
    }
}

impl Tab for CardAdder<'_> {
    type AppState = CardCache;

    fn tabdata_ref(&self) -> &TabData<Self::AppState> {
        self.add_card.tabdata_ref()
    }

    fn handle_popup_value(&mut self, _: &mut Self::AppState, card: Box<dyn std::any::Any>) {
        let card: SavedCard = *card.downcast().unwrap();
        let category = card.category().to_owned();
        self.category = category.clone();
    }

    fn pre_render_hook(&mut self, _app_data: &mut Self::AppState) {
        if self.pop_up().is_none() {
            self.set_popup(Box::new(AddCard::new(
                "add new card",
                self.category.clone(),
                None,
            )));
        }
    }

    fn tab_keyhandler(
        &mut self,
        app_data: &mut Self::AppState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        self.add_card.tab_keyhandler(app_data, key)
    }

    fn widgets(&mut self, area: Rect) -> Vec<(&mut dyn Widget<AppData = Self::AppState>, Rect)> {
        self.add_card.widgets(area)
    }

    fn remove_popup(&mut self) {
        self.tabdata().popup = None;
        self.tab_data.popup_state = PopUpState::Exit;
    }

    fn title(&self) -> &str {
        "add cards"
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState> {
        self.add_card.tabdata()
    }
}
