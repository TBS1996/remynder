use mischef::{Tab, TabData};
use speki_backend::common::duration_to_days;

use crate::{utils::TextDisplay, CardCache};

pub struct Stats {
    tab_data: TabData<CardCache>,
    info: TextDisplay,
}

impl Stats {
    pub fn new(cache: &mut CardCache) -> Self {
        let mut reviews = vec![];
        let mut daily_cards = 0.;
        let mut tot_str = 0.;
        let ids = cache.all_ids();
        let mut workload = 0.;

        for id in ids {
            let card = cache.get_ref(id);
            reviews.extend(card.reviews().clone());
            if let Some(stability) = card.stability() {
                if !card.is_suspended()
                    && card.is_finished()
                    && card.is_confidently_resolved(&mut cache.inner.lock().unwrap())
                {
                    let mut str = duration_to_days(&card.strength().unwrap_or_default());
                    tot_str += str;
                    if str < 1. {
                        str = 1.;
                    };

                    workload += 1. / str;
                    let mut stability = duration_to_days(&stability);
                    if stability < 1.0 {
                        stability = 1.0;
                    }
                    daily_cards += 1. / stability;
                }
            }
        }

        let reviews = reviews.len();
        let cards = cache.card_qty();

        let text =
            format!("amount of reviews: {reviews}\ndaily cards: {daily_cards}\ntot str: {tot_str}\nworkload: {workload}\ntot cards: {cards}");
        let info = TextDisplay { text };

        Self {
            tab_data: TabData::default(),
            info,
        }
    }
}

impl Tab for Stats {
    type AppState = CardCache;

    fn widgets(
        &mut self,
        area: ratatui::prelude::Rect,
    ) -> Vec<(
        &mut dyn mischef::Widget<AppData = Self::AppState>,
        ratatui::prelude::Rect,
    )> {
        vec![(&mut self.info, area)]
    }

    fn title(&self) -> &str {
        "stats"
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState> {
        &mut self.tab_data
    }

    fn tabdata_ref(&self) -> &TabData<Self::AppState> {
        &self.tab_data
    }

    fn tab_keyhandler(
        &mut self,
        cache: &mut Self::AppState,
        _key: crossterm::event::KeyEvent,
    ) -> bool {
        *self = Self::new(cache);
        false
    }
}
