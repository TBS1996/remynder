use crossterm::event::KeyCode;
use mischef::{Tab, TabData, Widget};
use speki_backend::{cache::CardCache, filter::FilterUtil, Id};

use crate::utils::{StatefulList, StatusBar};

use crate::widgets::table_thing::InputTable;
use crate::{hsplit2, vsplit2};

pub struct Browser<'a> {
    filter: FilterUtil,
    list: StatefulList<Id>,
    front_card: StatusBar,
    back_card: StatusBar,
    card_list: InputTable<'a, FilterUtil>,
    tab_data: TabData<CardCache>,
}

impl Browser<'_> {
    pub fn new(cache: &mut CardCache) -> Self {
        let filter = FilterUtil::default();
        let list = StatefulList::with_items(cache.all_ids());
        Self {
            filter,
            list,
            front_card: StatusBar::default(),
            back_card: StatusBar::default(),
            tab_data: TabData::default(),
            card_list: InputTable::new(),
        }
    }

    fn update_list(&mut self, cache: &mut CardCache) {
        let cards = cache.all_ids();
        let filtered = self.filter.evaluate_cards(cards, cache);
        self.list = StatefulList::with_items(filtered);
    }
}

impl Tab for Browser<'_> {
    type AppState = CardCache;

    fn handle_popup_value(&mut self, cache: &mut Self::AppState, filter: Box<dyn std::any::Any>) {
        let filter: FilterUtil = *filter.downcast().unwrap();
        self.filter = filter;
        self.update_list(cache);
    }

    fn set_selection(&mut self, area: ratatui::prelude::Rect) {
        let (list, sidebar) = hsplit2(area, 50, 50);
        let (up, down) = vsplit2(sidebar, 50, 50);

        self.card_list.set_area(up);
        self.back_card.set_area(down);
        self.list.set_area(list);

        self.tab_data.areas.extend([up, down, list]);
    }

    fn widgets(&mut self) -> Vec<&mut dyn mischef::Widget<AppData = Self::AppState>> {
        vec![&mut self.card_list, &mut self.back_card, &mut self.list]
    }

    fn title(&self) -> &str {
        "browse"
    }

    fn tab_keyhandler(
        &mut self,
        cache: &mut Self::AppState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        if self.card_list.is_selected(&self.tab_data.cursor)
            && self.tab_data.is_selected
            && self.card_list.is_valid()
            && key.code == KeyCode::Enter
        {
            self.filter = self.card_list.extract_type();
            let all_ids = cache.all_ids();
            let filtered = self.filter.evaluate_cards(all_ids, cache);
            self.list = StatefulList::with_items(filtered);
        }

        let cursor = self.cursor();
        if self.list.is_selected(&cursor) && key.code == KeyCode::Enter && self.selected() {
            let selected = self.list.selected();
            if let Some(selected) = selected {
                self.resolve_tab(Box::new(*selected));
            }
        }

        true
    }

    fn after_keyhandler(&mut self, cache: &mut CardCache) {
        match self.list.selected() {
            Some(card_id) => {
                let card = cache.get_ref(*card_id);
                self.front_card.text = card.front_text().to_owned();
                self.back_card.text = card.back_text().to_owned();
            }
            None => {
                self.front_card.text.clear();
                self.back_card.text.clear();
            }
        };
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState> {
        &mut self.tab_data
    }
}

mod macros {

    #[macro_export]
    macro_rules! define_widgets_and_areas {
    ($self:ident, [$(($widget:ident, $area:expr)),*]) => {
        // Set areas for widgets and store them in the view
        $(
            $self.$widget.set_area($area);
            $self.view.areas.push($area);
        )*

        // Implement the widgets method using the widgets defined in the tuple
        fn widgets(&mut self) -> Vec<&mut dyn mischef::Widget<AppData = Self::AppData>> {
            vec![$(&mut self.$widget),*]
        }
    };
}
}
