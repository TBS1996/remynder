use std::time::Duration;

use crossterm::event::KeyCode;
use speki_backend::{common::current_time, review::Grade, Id};

use mischef::{Retning, Tab, TabData, Widget};

use ratatui::prelude::*;

use crate::{
    hsplit2, split_off,
    tabs::review::CurrentCard,
    utils::{card_dependencies, card_dependents, TextDisplay, TextInput, TreeWidget},
    vsplit2, CardAction, CardActionTrait, CardCache, MyTabData, Pipeline, ReturnType,
};

pub struct CardReviewer<'a> {
    pub cards: Pipeline<Id>,
    pub dependencies: TreeWidget<'a, Id>,
    pub dependents: TreeWidget<'a, Id>,
    pub front: TextInput<'a>,
    pub back: TextInput<'a>,
    pub card_info: TextDisplay,
    pub info: TextDisplay,
    pub tab_data: MyTabData,
}

impl CardReviewer<'_> {
    pub fn new(cards: Vec<Id>, cache: &mut CardCache) -> Self {
        let mut myself = Self {
            cards: Pipeline::new(cards),
            dependencies: TreeWidget::new_with_items("Dependencies".into(), vec![]),
            dependents: TreeWidget::new_with_items("Dependents".into(), vec![]),
            tab_data: TabData::default(),
            front: TextInput::default(),
            back: TextInput::default(),
            card_info: TextDisplay::default(),
            info: TextDisplay::default(),
        };
        myself.cards.next();
        myself.refresh(cache);
        myself
    }

    fn clear(&mut self) {
        self.dependencies.clear();
        self.dependents.clear();
        self.front.clear();
        self.back.clear();
        self.card_info = TextDisplay::default();
        self.info = TextDisplay::default();
    }

    // call this only when new card
    fn refresh(&mut self, cache: &mut CardCache) {
        self.clear();
        let Some(card) = self.cards.current() else {
            return;
        };

        let card = cache.get_owned(*card);

        self.front = TextInput::new(card.front_text().to_owned());
        self.back = TextInput::new(card.back_text().to_owned());
        self.back.hide_text = true;
        self.dependencies
            .replace_items(card_dependencies(card.id(), cache));

        self.dependents
            .replace_items(card_dependents(card.id(), cache));
        self.card_info = TextDisplay::new(card_info(card.id(), cache));
        self.info = TextDisplay::new(format!("{:?}", self.cards.progress()));
    }

    // call this whenever a change to the inputs;
    fn update_card(&mut self, cache: &mut CardCache) {
        let Some(card) = self.cards.current().copied() else {
            return;
        };

        let mut card = cache.get_owned(card);

        let front_text = self.front.get_text();
        if !front_text.is_empty() {
            card.set_front_text(front_text.as_str());
        }
        let back_text = self.back.get_text();

        if !back_text.is_empty() {
            card.set_back_text(back_text.as_str());
        }

        self.dependencies
            .replace_items(card_dependencies(card.id(), cache));

        self.dependents
            .replace_items(card_dependents(card.id(), cache));
        self.card_info = TextDisplay::new(card_info(card.id(), cache));
        self.info = TextDisplay::new(format!("{:?}", self.cards.progress()));
    }

    fn play_front_audio(&mut self, cache: &mut CardCache) {
        self.evaluate_current(cache, CardAction::PlayFrontAudio);
    }
}

pub fn card_info(card: Id, cache: &mut CardCache) -> String {
    let card = cache.get_ref(card);
    let suspended = card.is_suspended();
    let finished = card.is_finished();
    let resolved = card.is_resolved(&mut cache.inner.lock().unwrap());
    let stability = card.stability().map(|d| d.as_secs_f32() / 86400.);
    let reviews = card.reviews().len();
    let recall_rate = card.recall_rate();
    let lapses = card.lapses();
    let priority = card.priority().as_float();
    let last_review = ((current_time() + Duration::from_secs(10))
        - card
            .reviews()
            .last()
            .map(|r| r.timestamp)
            .unwrap_or(current_time()))
    .as_secs_f32()
        / 86400.;
    let importance = card.weighted_importance(&mut cache.inner.lock().unwrap());

    format!("suspended: {}\nfinished: {}\nresolved: {}\nstability: {:?}\nreviews: {}\nrecall rate: {:?}\nlapses: {}\nlast review: {:.2} days\npriority : {priority}\nweighted importance: {importance}", suspended, finished, resolved, stability, reviews, recall_rate, lapses, last_review)
}

impl Tab for CardReviewer<'_> {
    type AppState = CardCache;
    type ReturnType = ReturnType;

    fn tabdata_ref(&self) -> &TabData<Self::AppState, Self::ReturnType> {
        &self.tab_data
    }

    fn widgets(&mut self, area: Rect) -> Vec<(&mut dyn Widget<AppData = Self::AppState>, Rect)> {
        let (info_bar, area) = split_off(area, 1, Retning::Up);

        let (card_area, info_area) = hsplit2(area, 50, 50);
        let (dependency_area, card_info_area) = vsplit2(info_area, 50, 50);
        let (dependency_area, dependents_area) = vsplit2(dependency_area, 50, 50);
        let (card_area, _) = vsplit2(card_area, 50, 50);
        let (front, back) = vsplit2(card_area, 50, 50);

        vec![
            (&mut self.front, front),
            (&mut self.back, back),
            (&mut self.card_info, card_info_area),
            (&mut self.dependencies, dependency_area),
            (&mut self.dependents, dependents_area),
            (&mut self.info, info_bar),
        ]
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState, Self::ReturnType> {
        &mut self.tab_data
    }

    fn tab_keyhandler_deselected(
        &mut self,
        cache: &mut Self::AppState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        let Some(card) = self.cards.current else {
            self.cards.next();
            self.refresh(cache);
            return true;
        };

        let is_finished = cache.get_ref(card).is_finished();

        let key = key.code;

        if let KeyCode::Char(c) = key {
            if let Ok(action) = CardAction::from_char(c.to_string().as_str()) {
                self.evaluate_current(cache, action);
            } else {
                match c {
                    ' ' => {
                        if self.back.hide_text {
                            self.back.hide_text = false;
                            self.evaluate_current(cache, CardAction::PlayBackAudio);
                            return false;
                        }
                    }
                    'n' => {
                        self.cards.next();
                        self.refresh(cache);
                    }
                    _ => {
                        if let Ok(grade) = c.to_string().parse::<Grade>() {
                            if is_finished && !self.back.hide_text {
                                cache.get_owned(card).new_review(grade, Duration::default());
                                self.cards.next();
                                self.refresh(cache);
                                return false;
                            }
                        }
                    }
                }
            }
        };

        self.update_card(cache);

        true
    }

    fn title(&self) -> &str {
        "review"
    }
}

impl CurrentCard for CardReviewer<'_> {
    fn selected_card(&self) -> Option<Id> {
        self.cards.current().copied()
    }
}

impl CardActionTrait for CardReviewer<'_> {}
