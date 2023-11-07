use super::*;

pub struct FilterChoice<'a> {
    foo: InputTable<'a, FilterUtil>,
    tabdata: TabData<CardCache>,
}

impl<'a> FilterChoice<'a> {
    pub fn new() -> Self {
        Self {
            foo: InputTable::new(),
            tabdata: TabData::default(),
        }
    }
}

impl Tab for FilterChoice<'_> {
    type AppState = CardCache;

    fn tab_keyhandler(
        &mut self,
        _cache: &mut Self::AppState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        if self.foo.is_selected(&self.tabdata.cursor)
            && self.tabdata.is_selected
            && key.code == KeyCode::Enter
            && self.foo.is_valid()
        {
            self.resolve_tab(Box::new(self.foo.extract_type()))
        }
        true
    }

    fn set_selection(&mut self, area: ratatui::prelude::Rect) {
        self.foo.set_area(area);
        self.tabdata.areas.push(area);
    }

    fn tabdata(&mut self) -> &mut mischef::TabData<Self::AppState> {
        &mut self.tabdata
    }

    fn widgets(&mut self) -> Vec<&mut dyn mischef::Widget<AppData = Self::AppState>> {
        vec![&mut self.foo]
    }

    fn title(&self) -> &str {
        "choose filter"
    }
}
