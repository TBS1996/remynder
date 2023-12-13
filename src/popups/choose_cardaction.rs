use crossterm::event::KeyCode;
use mischef::{Tab, TabData};
use speki_backend::Id;

use crate::{
    widgets::enum_choice::EnumChoice, CardAction, CardActionTrait, CardCache, MyTabData, ReturnType,
};

pub struct ActionPicker {
    cards: Vec<Id>,
    choice: EnumChoice<CardAction>,
    tab_data: MyTabData,
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

impl AsRef<MyTabData> for ActionPicker {
    fn as_ref(&self) -> &MyTabData {
        &self.tab_data
    }
}

impl AsMut<MyTabData> for ActionPicker {
    fn as_mut(&mut self) -> &mut MyTabData {
        &mut self.tab_data
    }
}

impl Tab for ActionPicker {
    type AppState = CardCache;
    type ReturnType = ReturnType;

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

    fn tabdata(&mut self) -> &mut TabData<Self::AppState, Self::ReturnType> {
        self.as_mut()
    }

    fn tabdata_ref(&self) -> &TabData<Self::AppState, Self::ReturnType> {
        self.as_ref()
    }

    fn title(&self) -> &str {
        "pick card action"
    }
}
