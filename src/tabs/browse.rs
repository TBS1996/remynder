use std::str::FromStr;

use crossterm::event::KeyCode;
use mischef::{Tab, TabData, Widget};
use rand::seq::SliceRandom;
use rand::thread_rng;
use speki_backend::{filter::FilterUtil, Id};
use strum_macros::{EnumIter, EnumString};

use crate::popups::CardInspector;
use crate::utils::{
    card_dependencies, card_dependents, StatefulList, StatefulTree, TextDisplay, TreeWidget,
};

use crate::widgets::enum_choice::EnumChoice;
use crate::widgets::table_thing::InputTable;
use crate::{hsplit2, split_off, vsplit2, CardCache};

use super::review::{card_info, CardAction, CardActionTrait};

#[derive(EnumString, EnumIter, strum_macros::Display)]
pub enum Sorter {
    LastModified,
    RecallRate,
    AlphaBetical,
    Shuffled,
}

pub struct Browser<'a> {
    filter: FilterUtil,
    card_list: StatefulList<Id>,
    front_card: TextDisplay,
    back_card: TextDisplay,
    info: TextDisplay,
    dependencies: TreeWidget<'a, Id>,
    dependents: TreeWidget<'a, Id>,
    filter_input: InputTable<'a, FilterUtil>,
    tab_data: TabData<CardCache>,
    sort_choice: EnumChoice<Sorter>,
    sort_dir: bool,
    is_popup: bool,
    cache_len: usize,
}

impl CardActionTrait for Browser<'_> {}

impl Browser<'_> {
    pub fn new(cache: &mut CardCache, is_popup: bool) -> Self {
        let filter = FilterUtil::default();
        let list = StatefulList::with_items(cache.all_ids());
        let mune = EnumChoice::<Sorter>::new();
        Self {
            filter,
            card_list: list,
            front_card: TextDisplay::default(),
            back_card: TextDisplay::default(),
            dependencies: TreeWidget::new_with_items("Dependencies".into(), vec![]),
            dependents: TreeWidget::new_with_items("Dependents".into(), vec![]),
            tab_data: TabData::default(),
            filter_input: InputTable::new(),
            sort_choice: mune,
            sort_dir: false,
            is_popup,
            info: TextDisplay::default(),
            cache_len: cache.card_qty(),
        }
    }

    fn update_list(&mut self, cache: &mut CardCache) {
        let cards = cache.all_ids();
        let filtered = self
            .filter
            .evaluate_cards(cards, &mut cache.inner.lock().unwrap());
        self.card_list = StatefulList::with_items(filtered);
    }
}

impl Tab for Browser<'_> {
    type AppState = CardCache;

    fn handle_popup_value(&mut self, cache: &mut Self::AppState, filter: Box<dyn std::any::Any>) {
        if let Ok(filter) = filter.downcast() {
            self.filter = *filter;
            self.update_list(cache);
        }
    }

    fn widgets(
        &mut self,
        area: ratatui::prelude::Rect,
    ) -> Vec<(
        &mut dyn Widget<AppData = Self::AppState>,
        ratatui::prelude::Rect,
    )> {
        let (list, sidebar) = hsplit2(area, 50, 50);
        let (up, down) = vsplit2(sidebar, 50, 50);
        let (filter, info) = hsplit2(up, 50, 50);
        let (sides, deps) = hsplit2(down, 50, 50);
        let (front, back) = vsplit2(sides, 50, 50);
        let (filter, mune) = split_off(
            filter,
            self.sort_choice.len() as u16 + 2,
            mischef::Retning::Down,
        );
        let (dpy, dpt) = vsplit2(deps, 50, 50);

        vec![
            (&mut self.info, info),
            (&mut self.filter_input, filter),
            (&mut self.front_card, front),
            (&mut self.back_card, back),
            (&mut self.card_list, list),
            (&mut self.sort_choice, mune),
            (&mut self.dependencies, dpy),
            (&mut self.dependents, dpt),
        ]
    }

    fn title(&self) -> &str {
        "browse"
    }

    fn pre_render_hook(&mut self, cache: &mut Self::AppState) {
        if cache.card_qty() != self.cache_len {
            self.update_list(cache);
            self.cache_len = cache.card_qty();
        }
    }

    fn tab_keyhandler_deselected(
        &mut self,
        _cache: &mut Self::AppState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        if key.code == KeyCode::Char('/') {
            self.tab_data.is_selected = true;
            self.move_to_id(self.filter_input.id().as_str());
            self.filter_input.inner.state.select(Some(0));
            false
        } else {
            true
        }
    }

    fn tab_keyhandler_selected(
        &mut self,
        cache: &mut Self::AppState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        if self.is_selected(&self.filter_input) {
            if self.filter_input.is_valid() && key.code == KeyCode::Enter {
                self.filter = self.filter_input.extract_type();
                let all_ids = cache.all_ids();
                let filtered = self
                    .filter
                    .evaluate_cards(all_ids, &mut cache.inner.lock().unwrap());
                self.card_list = StatefulList::with_items(filtered);
            }
        } else if self.is_selected(&self.sort_choice) {
            if key.code == KeyCode::Enter {
                match self.sort_choice.current_item() {
                    Sorter::LastModified => self
                        .card_list
                        .items
                        .sort_by_key(|id| cache.get_ref(*id).last_modified()),
                    Sorter::RecallRate => self.card_list.items.sort_by_key(|id| {
                        (cache.get_ref(*id).recall_rate().unwrap_or_default() * 100.) as u32
                    }),
                    Sorter::AlphaBetical => self
                        .card_list
                        .items
                        .sort_by_key(|id| cache.get_ref(*id).front_text().to_string()),
                    Sorter::Shuffled => {
                        self.card_list.items.shuffle(&mut thread_rng());
                    }
                }
                self.card_list.state.select(Some(0));

                if self.sort_dir {
                    self.card_list.items.reverse();
                }
                self.sort_dir = !self.sort_dir;
            }
        } else if self.is_selected(&self.card_list) {
            if key.code == KeyCode::Enter {
                let selected = self.card_list.selected();
                if let Some(selected) = selected {
                    if self.is_popup {
                        self.resolve_tab(Box::new(*selected));
                    } else {
                        let card_inspector = CardInspector::new(*selected, cache);
                        self.set_popup(Box::new(card_inspector));
                    }
                }
            }

            if let KeyCode::Char(c) = key.code {
                if let Ok(card_action) = CardAction::from_str(c.to_string().as_str()) {
                    let Some(card) = self.card_list.selected().map(|id| cache.get_owned(*id))
                    else {
                        return false;
                    };

                    self.evaluate(card.id(), cache, card_action);
                }
            }
        }

        true
    }

    fn after_keyhandler(&mut self, cache: &mut CardCache) {
        self.front_card.text.clear();
        self.back_card.text.clear();
        self.dependencies.tree = StatefulTree::with_items(vec![]);
        self.dependents.tree = StatefulTree::with_items(vec![]);

        if let Some(card_id) = self.card_list.selected() {
            let card_id = *card_id;
            let Some(card) = cache.try_get_ref(card_id) else {
                return;
            };

            self.front_card.text = card.front_text().to_owned();
            self.back_card.text = card.back_text().to_owned();
            self.dependencies
                .replace_items(card_dependencies(card_id, cache));

            self.dependents
                .replace_items(card_dependents(card_id, cache));

            self.info.text = card_info(card.id(), cache);
        }
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState> {
        &mut self.tab_data
    }

    fn tabdata_ref(&self) -> &TabData<Self::AppState> {
        &self.tab_data
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
