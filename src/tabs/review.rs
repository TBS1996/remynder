use std::{any::Any, time::Duration};

use crossterm::event::KeyCode;
use speki_backend::{
    cache::CardCache,
    card::{Grade, IsSuspended, Review, SavedCard},
    filter::FilterUtil,
    Id,
};

use mischef::{Retning, Tab, TabData, Widget};

use ratatui::prelude::*;

use crate::{
    hsplit2,
    popups::{AddCard, FilterChoice},
    split3, split_off,
    utils::{card_dependencies, StatusBar, TreeWidget},
    vsplit2,
};

use Constraint as Bound;

use super::browse::Browser;

pub enum PopUp<'a> {
    NewDependency(AddCard<'a>),
    NewDependent(AddCard<'a>),
    ChooseFilter(FilterChoice<'a>),
}

pub struct ReviewCard<'a> {
    pub filtered: Vec<Id>,
    pub completed: Vec<Id>,
    pub filter: FilterUtil,
    pub show_back: bool,
    pub dependencies: TreeWidget<'a, Id>,
    pub front: StatusBar,
    pub back: StatusBar,
    pub status: StatusBar,
    pub card_info: StatusBar,
    pub info: StatusBar,
    pub tab_data: TabData<CardCache>,
    pub popup_callback: CallBack,
}

impl std::fmt::Debug for ReviewCard<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReviewCard")
            .field("filtered", &self.filtered)
            .field("completed", &self.completed)
            .field("filter", &"~")
            .field("show_back", &self.show_back)
            .finish()
    }
}

// Like review filter but all of the dependencies have to be strong memories
fn confident_filter() -> FilterUtil {
    let dependencies = FilterUtil {
        min_recall_rate: Some(0.95),
        min_stability: Some(Duration::from_secs(86400)),
        ..FilterUtil::new_review()
    };

    FilterUtil {
        all_dependencies: Some(Box::new(dependencies)),
        ..FilterUtil::new_review()
    }
}

pub enum CallBack {
    NewDependency,
    NewDependent,
    Filter,
    OldDependency,
    OldDependent,
}

impl ReviewCard<'_> {
    pub fn new(cache: &mut CardCache) -> Self {
        let mut myself = Self {
            filtered: vec![],
            completed: vec![],
            filter: confident_filter(),
            show_back: false,
            dependencies: TreeWidget::new_with_items("Dependencies".into(), vec![]),
            tab_data: TabData::default(),
            front: StatusBar::default(),
            back: StatusBar::default(),
            status: StatusBar::default(),
            card_info: StatusBar::default(),
            info: StatusBar::default(),
            popup_callback: CallBack::Filter, // dummy
        };

        myself.filtering(cache);
        myself
    }

    fn filtering(&mut self, cache: &mut CardCache) {
        let all_cards = cache.ids_as_vec();
        self.filtered = self.filter.evaluate_cards(all_cards.clone(), cache);
        self.refresh(cache);
    }

    fn current_card(&self) -> Option<Id> {
        self.filtered.last().copied()
    }

    fn clear(&mut self) {
        self.front = Default::default();
        self.back = Default::default();
        self.dependencies.replace_items(vec![]);
        self.card_info = Default::default();
        self.status = Default::default();
    }

    fn refresh(&mut self, cache: &mut CardCache) {
        let Some(card_id) = self.current_card() else {
            return;
        };

        let card = cache.get_ref(card_id);

        self.front.text = card.front_text().to_string();

        self.dependencies
            .replace_items(card_dependencies(card.id(), cache));

        if self.show_back {
            self.back.text = card.back_text().to_string();
        }

        self.info.text = {
            let category = card.category();
            let (a, b) = self.cards_progress();
            format!("{}    {}/{}", category.print_full(), a, b)
        };
        self.card_info.text = card_info(card.id(), cache);
    }

    fn cards_progress(&self) -> (usize, usize) {
        let a = self.completed.len();
        let b = a + self.filtered.len();
        (a, b)
    }

    /// Returns true if theres more cards
    fn next_card(&mut self, cache: &mut CardCache) -> bool {
        self.show_back = false;

        match self.filtered.pop() {
            Some(card) => {
                self.completed.push(card);

                self.clear();
                self.refresh(cache);
                true
            }
            None => false,
        }
    }

    fn handle_filter(&mut self, cache: &mut CardCache, filter: Box<dyn Any>) {
        let filter: FilterUtil = *filter.downcast().unwrap();
        self.filter = filter;
        self.filtering(cache);
    }

    fn handle_old_dependent(&mut self, cache: &mut CardCache, card: Box<dyn Any>) {
        let id: Id = *card.downcast().unwrap();
        let card = cache.get_owned(id);

        if let Some(id) = self.current_card() {
            cache.get_owned(id).set_dependent(card.id(), cache);
        }
    }

    fn handle_old_dependency(&mut self, cache: &mut CardCache, card: Box<dyn Any>) {
        let id: Id = *card.downcast().unwrap();
        let card = cache.get_owned(id);

        if let Some(id) = self.current_card() {
            cache.get_owned(id).set_dependency(card.id(), cache);
        }
    }

    fn handle_new_dependent(&mut self, cache: &mut CardCache, card: Box<dyn Any>) {
        let card: SavedCard = *card.downcast().unwrap();
        if let Some(id) = self.current_card() {
            cache.get_owned(id).set_dependent(card.id(), cache);
        }
    }

    fn handle_new_dependency(&mut self, cache: &mut CardCache, card: Box<dyn Any>) {
        let card: SavedCard = *card.downcast().unwrap();
        if let Some(id) = self.current_card() {
            cache.get_owned(id).set_dependency(card.id(), cache);
        }
    }
}

fn count_lapses(reviews: &Vec<Review>) -> u32 {
    let mut lapses = 0;

    for review in reviews {
        match review.grade {
            Grade::None => lapses += 1,
            Grade::Late => lapses += 1,
            Grade::Some => lapses = 0,
            Grade::Perfect => lapses = 0,
        };
    }

    lapses
}

fn card_info(card: Id, cache: &mut CardCache) -> String {
    let card = cache.get_ref(card);
    let suspended = card.is_suspended();
    let finished = card.is_finished();
    let resolved = card.is_resolved(cache);
    let stability = card.stability().map(|d| d.as_secs_f32() / 86400.);
    let reviews = card.reviews().len();
    let recall_rate = card.recall_rate();
    let lapses = count_lapses(card.reviews());

    format!("suspended: {}\nfinished: {}\nresolved: {}\nstability: {:?}\nreviews: {}\nrecall rate: {:?}\nlapses: {}", suspended, finished, resolved, stability, reviews, recall_rate, lapses)
}

impl Tab for ReviewCard<'_> {
    type AppState = CardCache;

    fn set_selection(&mut self, area: Rect) {
        let (info_bar, area) = split_off(area, 1, Retning::Up);

        let (card_area, info_area) = hsplit2(area, 50, 50);
        let (card_info_area, dependency_area) = vsplit2(info_area, 50, 50);

        let (front, back, _) = split3(
            card_area,
            Direction::Vertical,
            Bound::Length(5),
            Bound::Length(5),
            Bound::Min(0),
        );

        self.front.set_area(front);
        self.back.set_area(back);
        self.status.set_area(info_area);
        self.card_info.set_area(card_info_area);
        self.dependencies.set_area(dependency_area);
        self.info.set_area(info_bar);

        self.tab_data
            .areas
            .extend([front, back, card_info_area, dependency_area, info_bar]);
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState> {
        &mut self.tab_data
    }

    fn handle_popup_value(&mut self, cache: &mut Self::AppState, value: Box<dyn Any>) {
        match self.popup_callback {
            CallBack::NewDependency => self.handle_new_dependency(cache, value),
            CallBack::NewDependent => self.handle_new_dependent(cache, value),
            CallBack::Filter => self.handle_filter(cache, value),
            CallBack::OldDependency => self.handle_old_dependency(cache, value),
            CallBack::OldDependent => self.handle_old_dependent(cache, value),
        }
        self.refresh(cache);
    }

    fn tab_keyhandler(
        &mut self,
        cache: &mut speki_backend::cache::CardCache,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        let key = key.code;

        let Some(card) = self.current_card() else {
            return true;
        };

        let mut card = cache.get_owned(card);

        if let KeyCode::Char(c) = key {
            match c {
                ' ' => {
                    self.show_back = true;
                    self.refresh(cache);
                    return false;
                }
                'o' => {
                    let p = cache.get_ref(self.current_card().unwrap()).as_path();
                    std::process::Command::new("open")
                        .arg(p.as_path())
                        .status()
                        .expect("Failed to open file");
                }
                's' => {
                    card.set_suspended(IsSuspended::True);
                    return false;
                }
                'f' => {
                    self.set_popup(Box::new(FilterChoice::new()));
                    self.popup_callback = CallBack::Filter;
                }
                'T' => {
                    self.set_popup(Box::new(AddCard::new(
                        "Add new dependent",
                        card.category().to_owned(),
                    )));
                    self.popup_callback = CallBack::NewDependent
                }
                'Y' => {
                    self.set_popup(Box::new(AddCard::new(
                        "Add new dependency",
                        card.category().to_owned(),
                    )));
                    self.popup_callback = CallBack::NewDependency
                }
                'y' => {
                    self.set_popup(Box::new(Browser::new(cache)));
                    self.popup_callback = CallBack::OldDependency;
                }
                't' => {
                    self.set_popup(Box::new(Browser::new(cache)));
                    self.popup_callback = CallBack::OldDependent;
                }
                _ => {
                    if let Ok(grade) = c.to_string().parse::<speki_backend::card::Grade>() {
                        card.new_review(grade, Duration::default());
                        self.next_card(cache);
                        return false;
                    }
                }
            }
        };

        self.refresh(cache);
        true
    }

    fn widgets(&mut self) -> Vec<&mut dyn mischef::Widget<AppData = Self::AppState>> {
        vec![
            &mut self.front,
            &mut self.back,
            &mut self.info,
            &mut self.card_info,
            &mut self.dependencies,
        ]
    }

    fn title(&self) -> &str {
        "review"
    }
}

// Define a macro to easily collect widgets into a vector of trait objects.
