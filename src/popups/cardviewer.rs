use speki_backend::Id;
use tui_textarea::TextArea;

use crate::{
    utils::{StatusBar, TextInput},
    vsplit2,
};

use super::*;

pub struct CardInspector<'a> {
    card: Id,
    front: TextInput<'a>,
    back: TextInput<'a>,
    tab_data: TabData<CardCache>,
}

impl<'a> Tab for CardInspector<'a> {
    type AppState = CardCache;

    fn widgets(&mut self) -> Vec<&mut dyn Widget<AppData = Self::AppState>> {
        vec![&mut self.front, &mut self.back]
    }

    fn title(&self) -> &str {
        "inspect card"
    }

    fn set_selection(&mut self, area: ratatui::prelude::Rect) {
        let (front, back) = vsplit2(area, 50, 50);

        self.front.set_area(front);
        self.back.set_area(back);

        self.tabdata().areas.extend([front, back]);
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState> {
        &mut self.tab_data
    }
}
