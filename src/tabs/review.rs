use std::time::Duration;

use crossterm::event::KeyCode;
use speki_backend::{
    cache::CardCache,
    card::{Grade, IsSuspended, Review},
    filter::FilterUtil,
    Id,
};

use mischef::{App, Retning, Tab, View, Widget};

use ratatui::prelude::*;

use crate::{
    hsplit2,
    popups::AddCard,
    split3, split_off,
    utils::{card_dependencies, StatusBar, TreeWidget},
    vsplit2,
};

use Constraint as Bound;

pub enum PopUp<'a> {
    NewDependency(AddCard<'a>),
    NewDependent(AddCard<'a>),
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
    pub view: View,
    pub popup: Option<PopUp<'a>>,
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

impl ReviewCard<'_> {
    pub fn new(cache: &mut CardCache) -> Self {
        let all_cards = cache.ids_as_vec();
        let filter = confident_filter();
        let mut filtered = filter.evaluate_cards(all_cards.clone(), cache);

        filtered.retain(|card| {
            let card = cache.get_ref(*card);
            count_lapses(card.reviews()) < 4
        });

        let mut x = Self {
            filtered,
            completed: vec![],
            filter,
            show_back: false,
            dependencies: TreeWidget::new_with_items("Dependencies".into(), vec![]),
            view: View::default(),
            front: StatusBar::default(),
            back: StatusBar::default(),
            status: StatusBar::default(),
            card_info: StatusBar::default(),
            info: StatusBar::default(),
            popup: None,
        };

        x.refresh(cache);

        x
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
    type AppData = CardCache;

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

        self.view
            .areas
            .extend([front, back, card_info_area, dependency_area, info_bar]);
    }

    fn view(&mut self) -> &mut View {
        &mut self.view
    }

    fn pop_up(&mut self) -> Option<&mut dyn Tab<AppData = Self::AppData>> {
        self.popup.as_mut().map(|p| match p {
            PopUp::NewDependency(p) => p as &mut dyn Tab<AppData = Self::AppData>,
            PopUp::NewDependent(p) => p as &mut dyn Tab<AppData = Self::AppData>,
        })
    }

    fn check_popup_value(&mut self, cache: &mut CardCache) {
        let mut flag = false;
        match &self.popup {
            Some(PopUp::NewDependency(c)) => {
                if let Some(val) = c.popstate.value() {
                    if let Some(current) = self.current_card() {
                        let mut current_card = cache.get_owned(current);
                        current_card.set_dependency(val.id(), cache);
                    }
                    flag = true;
                }
            }
            Some(PopUp::NewDependent(c)) => {
                if let Some(val) = c.popstate.value() {
                    if let Some(current) = self.current_card() {
                        let mut current_card = cache.get_owned(current);
                        current_card.set_dependent(val.id(), cache);
                    }
                    flag = true;
                }
            }
            None => return,
        };
        if flag {
            self.popup = None;
            self.refresh(cache);
        }
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
                's' => {
                    card.set_suspended(IsSuspended::True);
                    return false;
                }
                'Y' => {
                    self.popup = Some(PopUp::NewDependency(AddCard::new(
                        "Add new dependency".into(),
                        card.category().to_owned(),
                    )));
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

    fn widgets(&mut self) -> Vec<&mut dyn mischef::Widget<AppData = Self::AppData>> {
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
