use speki_backend::{cache::CardCache, filter::FilterUtil, Id};

use crate::{
    hsplit2,
    ui_library::{Tab, View, Widget},
    utils::{StatefulList, StatusBar},
    vsplit2,
};

pub struct Browser {
    filter: FilterUtil,
    list: StatefulList<Id>,
    front_card: StatusBar,
    back_card: StatusBar,
    view: View,
}

impl Browser {
    pub fn new(cache: &mut CardCache) -> Self {
        let filter = FilterUtil::default();
        let list = StatefulList::with_items(cache.all_ids());
        Self {
            filter,
            list,
            front_card: StatusBar::default(),
            back_card: StatusBar::default(),
            view: View::default(),
        }
    }

    fn update_list(&mut self, cache: &mut CardCache) {
        let cards = cache.all_ids();
        let filtered = self.filter.evaluate_cards(cards, cache);
        self.list = StatefulList::with_items(filtered);
    }
}

impl Tab for Browser {
    fn set_selection(&mut self, area: ratatui::prelude::Rect) {
        let (list, sidebar) = hsplit2(area, 50, 50);
        let (up, down) = vsplit2(sidebar, 50, 50);

        self.front_card.set_area(up);
        self.back_card.set_area(down);
        self.list.set_area(list);

        self.view.areas.extend([up, down, list]);
    }

    fn view(&mut self) -> &mut crate::ui_library::View {
        &mut self.view
    }

    fn widgets(&mut self) -> Vec<&mut dyn crate::ui_library::Widget> {
        vec![
            &mut self.front_card as &mut dyn crate::ui_library::Widget,
            &mut self.back_card as &mut dyn crate::ui_library::Widget,
            &mut self.list as &mut dyn crate::ui_library::Widget,
        ]
    }

    fn title(&self) -> &str {
        "browse"
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
}
