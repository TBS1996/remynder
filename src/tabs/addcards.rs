use ratatui::prelude::Rect;
use speki_backend::{cache::CardCache, card::SavedCard, categories::Category};

use mischef::{PopUpState, Tab, TabData, Widget};

use crate::popups::AddCard;

/// Just a thin wrapper around AddCard because I wanted a popup that creates a single card,
/// and this keeps the code dry.
pub struct CardAdder<'a> {
    add_card: AddCard<'a>,
    tab_data: TabData<CardCache>,
}

impl CardAdder<'_> {
    pub fn new() -> Self {
        let mut s = Self {
            add_card: AddCard::new("create new card", Category::root()),
            tab_data: TabData::default(),
        };

        s.set_popup(Box::new(AddCard::new("add new card", Category::root())));
        s
    }
}

impl Tab for CardAdder<'_> {
    type AppState = CardCache;

    fn handle_popup_value(&mut self, _: &mut Self::AppState, card: Box<dyn std::any::Any>) {
        let card: SavedCard = *card.downcast().unwrap();
        let category = dbg!(card.category().to_owned());
        self.set_popup(Box::new(AddCard::new("add new card", category)));
    }

    fn remove_popup(&mut self) {
        self.tabdata().popup = None;
        self.tab_data.popup_state = PopUpState::Exit;
    }

    fn set_selection(&mut self, area: Rect) {
        self.add_card.set_selection(area);
    }

    fn widgets(&mut self) -> Vec<&mut dyn Widget<AppData = Self::AppState>> {
        self.add_card.widgets()
    }

    fn title(&self) -> &str {
        "add cards"
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState> {
        self.add_card.tabdata()
    }
}
