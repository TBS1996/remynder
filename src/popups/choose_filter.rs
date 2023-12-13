use speki_backend::Id;

use crate::{hsplit2, utils::StatefulList, MyTabData, ReturnType};

use super::*;

pub struct FilterChoice<'a> {
    filter: InputTable<'a, FilterUtil>,
    list: StatefulList<Id>,
    tabdata: MyTabData,
}

impl<'a> FilterChoice<'a> {
    pub fn new(cache: &mut CardCache) -> Self {
        let items = cache.all_ids();
        let list = StatefulList::with_items(items);

        Self {
            list,
            filter: InputTable::new(),
            tabdata: TabData::default(),
        }
    }
}

impl Tab for FilterChoice<'_> {
    type AppState = CardCache;
    type ReturnType = ReturnType;

    fn tabdata_ref(&self) -> &TabData<Self::AppState, Self::ReturnType> {
        &self.tabdata
    }

    fn tab_keyhandler(
        &mut self,
        _cache: &mut Self::AppState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        if self.is_selected(&self.filter)
            && self.tabdata.is_selected
            && key.code == KeyCode::Enter
            && self.filter.is_valid()
        {
            self.resolve_tab(ReturnType::Filter(self.filter.extract_type()))
        }
        true
    }

    fn widgets(
        &mut self,
        area: ratatui::prelude::Rect,
    ) -> Vec<(
        &mut dyn Widget<AppData = Self::AppState>,
        ratatui::prelude::Rect,
    )> {
        let (list, filter) = hsplit2(area, 50, 50);
        vec![(&mut self.filter, filter), (&mut self.list, list)]
    }

    fn tabdata(&mut self) -> &mut mischef::TabData<Self::AppState, Self::ReturnType> {
        &mut self.tabdata
    }

    fn title(&self) -> &str {
        "choose filter"
    }
}
