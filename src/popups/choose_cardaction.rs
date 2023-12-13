use crossterm::event::KeyCode;
use mischef::{Tab, TabData};
use speki_backend::Id;

use crate::{widgets::enum_choice::EnumChoice, CardAction, CardActionTrait, CardCache};

pub struct ActionPicker {
    cards: Vec<Id>,
    choice: EnumChoice<CardAction>,
    tab_data: TabData<CardCache>,
}

impl CardActionTrait for ActionPicker {}

impl ActionPicker {
    pub fn new(cards: Vec<Id>) -> Self {
        Self {
            cards,
            choice: EnumChoice::new(),
            tab_data: TabData::default(),
        }
    }
}

impl AsRef<TabData<CardCache>> for ActionPicker {
    fn as_ref(&self) -> &TabData<CardCache> {
        &self.tab_data
    }
}

impl AsMut<TabData<CardCache>> for ActionPicker {
    fn as_mut(&mut self) -> &mut TabData<CardCache> {
        &mut self.tab_data
    }
}

impl Tab for ActionPicker {
    type AppState = CardCache;

    fn widgets(
        &mut self,
        area: ratatui::prelude::Rect,
    ) -> Vec<(
        &mut dyn mischef::Widget<AppData = Self::AppState>,
        ratatui::prelude::Rect,
    )> {
        vec![(&mut self.choice, area)]
    }

    fn tab_keyhandler_selected(
        &mut self,
        cache: &mut Self::AppState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        if key.code == KeyCode::Enter {
            let action = self.choice.current_item();
            let cards = self.cards.clone();
            for card in cards {
                self.evaluate(card, cache, action);
            }
            self.exit_tab();
            return false;
        }
        true
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState> {
        self.as_mut()
    }

    fn tabdata_ref(&self) -> &TabData<Self::AppState> {
        self.as_ref()
    }

    fn title(&self) -> &str {
        "pick card action"
    }
}
