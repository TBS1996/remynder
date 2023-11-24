use std::collections::BTreeMap;

use crossterm::event::KeyEvent;
use mischef::Retning;
use ratatui::prelude::Rect;
use speki_backend::Id;

use crate::{
    split_off,
    utils::{StatefulList, TextDisplay},
};

use super::*;

#[derive(Default)]
pub struct CardFinder {
    search: TextDisplay,
    cards: StatefulList<Id>,
    tab_data: TabData<CardCache>,
    index: Indexer,
}

impl CardFinder {
    pub fn new(cache: &mut CardCache) -> Self {
        let index = Indexer::new(cache);
        let cards = StatefulList::with_items(index.inner.get("").unwrap().clone());

        Self {
            index,
            cards,
            ..Default::default()
        }
    }
}

impl Tab for CardFinder {
    type AppState = CardCache;

    fn tab_keyhandler(
        &mut self,
        cache: &mut Self::AppState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        if key.code == KeyCode::Down {
            self.cards.next();
        } else if key.code == KeyCode::Up {
            self.cards.previous();
        } else if key.code == KeyCode::Esc {
            self.exit_tab();
        } else if key.code == KeyCode::Enter {
            if let Some(card) = self.cards.selected().cloned() {
                self.resolve_tab(Box::new(card));
            }
        } else {
            self.index.input(&key, cache);
            let mut cards = self.index.current().clone();
            cards.sort_by_key(|card| cache.get_ref(*card).dependent_ids().len());
            cards.reverse();
            self.cards
                .state
                .select(if !cards.is_empty() { Some(0) } else { None });
            self.cards.items = cards;
            self.search.text = self.index.search.clone();
        }
        false
    }

    fn pre_render_hook(&mut self, _app_data: &mut Self::AppState) {
        self.tab_data.is_selected = true;
        self.move_to_id(&self.cards.id());
    }

    fn pre_keyhandler_hook(&mut self, key: crossterm::event::KeyEvent) {
        self.tab_data.is_selected = key.code != KeyCode::Esc;
    }

    fn widgets(
        &mut self,
        area: Rect,
    ) -> Vec<(
        &mut dyn Widget<AppData = Self::AppState>,
        ratatui::prelude::Rect,
    )> {
        let (search, list) = split_off(area, 3, Retning::Up);

        vec![(&mut self.search, search), (&mut self.cards, list)]
    }

    fn title(&self) -> &str {
        "card finder"
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState> {
        &mut self.tab_data
    }

    fn tabdata_ref(&self) -> &TabData<Self::AppState> {
        &self.tab_data
    }
}

#[derive(Default)]
struct Indexer {
    search: String,
    inner: BTreeMap<String, Vec<Id>>,
}

impl Indexer {
    fn new(cache: &mut CardCache) -> Self {
        let mut m = BTreeMap::new();
        let ids = cache.all_ids();
        m.insert("".into(), ids);

        Self {
            search: "".into(),
            inner: m,
        }
    }

    fn current(&self) -> &Vec<Id> {
        self.inner.get(&self.search).unwrap()
    }

    fn input(&mut self, key: &KeyEvent, cache: &mut CardCache) {
        if key.code == KeyCode::Backspace {
            self.search.pop();
            return;
        }

        let KeyCode::Char(c) = key.code else {
            return;
        };

        let old_search = self.search.clone();
        self.search.push(c);

        // We've already cached, so return early
        if self.inner.contains_key(&self.search) {
            return;
        }

        // the new cached vec of the new position.
        let mut new_vec = vec![];

        // we know that when we add a new character, the new matching ids
        // are a subset of the previous ones, so we save some time then.
        for id in self.inner.get(&old_search).unwrap() {
            let card = cache.get_ref(*id);
            if card.matches_search(&self.search) {
                new_vec.push(*id);
            }
        }

        self.inner.insert(self.search.clone(), new_vec);
    }
}
