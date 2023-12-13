use std::time::Duration;

use crossterm::event::KeyCode;
use rand::seq::SliceRandom;
use speki_backend::{cache::Cards, filter::FilterUtil, Id};

use mischef::{Tab, TabData, Widget};

use ratatui::prelude::*;
use strum_macros::{EnumIter, EnumString};

use crate::{popups::CardReviewer, widgets::enum_choice::EnumChoice, CardCache};

// Like review filter but all of the dependencies have to be strong memories
fn confident_filter() -> FilterUtil {
    let dependencies = FilterUtil {
        min_recall_rate: Some(0.95),
        min_stability: Some(Duration::from_secs(86400)),
        ..FilterUtil::new_valid()
    };

    FilterUtil {
        all_dependencies: Some(Box::new(dependencies)),
        max_recall_rate: Some(0.9),
        ..FilterUtil::new_valid()
    }
}

#[derive(EnumString, EnumIter, strum_macros::Display)]
enum MenuChoice {
    Review,
}

pub struct ReviewMenu {
    option: EnumChoice<MenuChoice>,
    pub tab_data: TabData<CardCache>,
}

impl ReviewMenu {
    pub fn new() -> Self {
        Self {
            option: EnumChoice::<MenuChoice>::new(),
            tab_data: TabData::default(),
        }
    }
}

impl Tab for ReviewMenu {
    type AppState = CardCache;

    fn tabdata_ref(&self) -> &TabData<Self::AppState> {
        &self.tab_data
    }

    fn widgets(&mut self, area: Rect) -> Vec<(&mut dyn Widget<AppData = Self::AppState>, Rect)> {
        vec![(&mut self.option, area)]
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState> {
        &mut self.tab_data
    }

    fn tab_keyhandler_selected(
        &mut self,
        cache: &mut Self::AppState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        if self.is_selected(&self.option) && key == KeyCode::Enter.into() {
            match self.option.current_item() {
                MenuChoice::Review => {
                    let cards = Cards(cache.all_ids().into_iter().collect());
                    let cards: Vec<Id> = cards
                        .filter_importance(1., &mut cache.inner.lock().unwrap())
                        .0
                        .into_iter()
                        .collect();

                    let f = confident_filter();

                    let mut cards = f.evaluate_cards(cards, &mut cache.inner.lock().unwrap());
                    cards.retain(|card| cache.get_ref(*card).older_than(1.0));

                    //cards.shuffle(&mut rand::thread_rng());

                    let rev = CardReviewer::new(cards, cache);
                    self.set_popup(Box::new(rev));
                }
            };
        }
        true
    }

    fn title(&self) -> &str {
        "review"
    }
}

pub trait CurrentCard {
    // if you wanna use only multiple cards then implemented 'selected_cards' and let this one return a 'None'.
    fn selected_card(&self) -> Option<Id>;

    // Override this if your type supports having multiple cards selected.
    fn selected_cards(&self) -> Vec<Id> {
        match self.selected_card() {
            Some(card) => vec![card],
            None => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    fn nonlinear_scale(input: f64, expected: f64, max_output: f64) -> f64 {
        let steepness = 1.0 / expected;
        let midpoint = expected;

        let output = max_output / (1.0 + f64::exp(-steepness * (input - midpoint)));

        output
    }

    #[test]
    fn foobar() {
        let max_output = 10.;
        let expected = 9.;

        for input in 0..100 {
            dbg!(nonlinear_scale(0 as f64, expected, max_output));
        }
    }
}
