use std::{any::Any, fs::File, io::BufReader, path::PathBuf, str::FromStr, time::Duration};

use crossterm::event::KeyCode;
use rodio::{Decoder, OutputStream, Source};
use speki_backend::{card::IsSuspended, common::current_time, filter::FilterUtil, Id};

use mischef::{Retning, Tab, TabData, Widget};

use ratatui::prelude::*;
use strum_macros::EnumIter;

use crate::{
    hsplit2,
    popups::{AddCard, CardFinder, CardInspector, DependencyStatus, FilterChoice},
    split_off,
    utils::{card_dependencies, card_dependents, TextDisplay, TextInput, TreeWidget},
    vsplit2, CardCache, Pipeline,
};

impl std::fmt::Debug for ReviewCard<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReviewCard")
            .field("cards", &self.cards.tot_qty())
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
        ..FilterUtil::new_valid()
    };

    FilterUtil {
        all_dependencies: Some(Box::new(dependencies)),
        ..FilterUtil::new_valid()
    }
}

pub enum CallBack {
    CardInspector,
    Filter,
    None,
}

enum Mode {
    Weighted,
    Conditional,
    NewCards,
}

pub struct ReviewCard<'a> {
    pub cards: Pipeline<Id>,
    pub filter: FilterUtil,
    pub show_back: bool,
    pub dependencies: TreeWidget<'a, Id>,
    pub dependents: TreeWidget<'a, Id>,
    pub front: TextInput<'a>,
    pub back: TextInput<'a>,
    pub card_info: TextDisplay,
    pub info: TextDisplay,
    pub tab_data: TabData<CardCache>,
    pub popup_callback: CallBack,
}

impl ReviewCard<'_> {
    pub fn new(cache: &mut CardCache) -> Self {
        let mut myself = Self {
            cards: Pipeline::new(vec![]),
            filter: confident_filter(),
            show_back: false,
            dependencies: TreeWidget::new_with_items("Dependencies".into(), vec![]),
            dependents: TreeWidget::new_with_items("Dependents".into(), vec![]),
            tab_data: TabData::default(),
            front: TextInput::default(),
            back: TextInput::default(),
            card_info: TextDisplay::default(),
            info: TextDisplay::default(),
            popup_callback: CallBack::None, // dummy
        };

        myself.filtering(cache);
        myself.play_front_audio(cache);
        myself
    }

    fn play_front_audio(&mut self, cache: &mut CardCache) {
        if let Some(card) = self.cards.current() {
            self.evaluate(*card, cache, CardAction::PlayFrontAudio);
        }
    }

    fn filtering(&mut self, cache: &mut CardCache) {
        let all_cards = cache.ids_as_vec();
        let mut cards = self
            .filter
            .evaluate_cards(all_cards.clone(), &mut cache.inner.lock().unwrap());
        cards.sort_by_key(|card| {
            let card = cache.get_ref(*card);
            (card.weighted_importance(&mut cache.inner.lock().unwrap()) * 100.) as i32
        });
        cards.retain(|card| {
            cache
                .get_ref(*card)
                .weighted_importance(&mut cache.inner.lock().unwrap())
                > 0.55
        });
        self.cards = Pipeline::new(cards);

        self.refresh(cache);
    }

    fn clear(&mut self) {
        self.front = Default::default();
        self.back = Default::default();
        self.dependencies.replace_items(vec![]);
        self.card_info = Default::default();
    }

    fn refresh(&mut self, cache: &mut CardCache) {
        let Some(card_id) = self.cards.current() else {
            return;
        };

        let Some(card) = cache.try_get_ref(*card_id) else {
            self.next_card(cache);
            return;
        };

        self.front.text.insert_str(card.front_text().to_string());

        self.dependencies
            .replace_items(card_dependencies(card.id(), cache));

        self.dependents
            .replace_items(card_dependents(card.id(), cache));

        if self.show_back {
            self.back.text.insert_str(card.back_text().to_string());
        }

        self.info.text = {
            let category = card.category();
            let (a, b) = self.cards_progress();
            format!("{}    {}/{}", category.print_full(), a, b)
        };
        self.card_info.text = card_info(card.id(), cache);
    }

    fn cards_progress(&self) -> (usize, usize) {
        let a = self.cards.finished_qty();
        let b = self.cards.tot_qty();
        (a, b)
    }

    /// Returns true if theres more cards
    fn next_card(&mut self, cache: &mut CardCache) -> bool {
        self.show_back = false;
        self.cards.next();
        self.clear();

        if !self.cards.is_done() {
            self.refresh(cache);
            self.play_front_audio(cache);
            true
        } else {
            false
        }
    }

    fn handle_filter(&mut self, cache: &mut CardCache, filter: Box<dyn Any>) {
        let filter: FilterUtil = *filter.downcast().unwrap();
        self.filter = filter;
        self.filtering(cache);
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

impl Tab for ReviewCard<'_> {
    type AppState = CardCache;

    fn tabdata_ref(&self) -> &TabData<Self::AppState> {
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

    fn tabdata(&mut self) -> &mut TabData<Self::AppState> {
        &mut self.tab_data
    }

    fn handle_popup_value(&mut self, cache: &mut Self::AppState, value: Box<dyn Any>) {
        match self.popup_callback {
            CallBack::Filter => self.handle_filter(cache, value),
            CallBack::CardInspector => {}
            CallBack::None => {}
        }
        self.popup_callback = CallBack::None;
        self.refresh(cache);
    }

    fn tab_keyhandler_deselected(
        &mut self,
        cache: &mut Self::AppState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        let Some(card) = self.cards.current().copied() else {
            return true;
        };

        let is_finished = cache.get_ref(card).is_finished();

        let key = key.code;

        if let KeyCode::Char(c) = key {
            if let Ok(action) = CardAction::from_str(c.to_string().as_str()) {
                self.evaluate(card, cache, action);
            } else {
                match c {
                    ' ' => {
                        if !self.show_back {
                            self.show_back = true;
                            self.evaluate(card, cache, CardAction::PlayBackAudio);
                            self.refresh(cache);
                            return false;
                        }
                    }
                    'n' => {
                        self.next_card(cache);
                    }
                    'v' => {
                        *self = Self::new(cache);
                        self.clear();
                        self.refresh(cache);
                    }
                    'u' => {
                        self.set_popup(Box::new(FilterChoice::new(cache)));
                        self.popup_callback = CallBack::Filter;
                    }
                    _ => {
                        if let Ok(grade) = c.to_string().parse::<speki_backend::card::Grade>() {
                            if is_finished && self.show_back {
                                cache.get_owned(card).new_review(grade, Duration::default());
                                self.next_card(cache);
                                return false;
                            }
                        }
                    }
                }
            }
        };
        self.refresh(cache);
        true
    }

    fn tab_keyhandler_selected(
        &mut self,
        cache: &mut Self::AppState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        let key = key.code;

        if self.is_selected(&self.dependents) && key == KeyCode::Enter {
            if let Some(id) = self.dependents.selected() {
                let inspect_card = CardInspector::new(id, cache);
                self.set_popup(Box::new(inspect_card));
                self.popup_callback = CallBack::CardInspector;
            }
        } else if self.is_selected(&self.dependencies) && key == KeyCode::Enter {
            if let Some(id) = self.dependencies.selected() {
                let inspect_card = CardInspector::new(id, cache);
                self.set_popup(Box::new(inspect_card));
                self.popup_callback = CallBack::CardInspector;
            }
        }

        //self.refresh(cache);
        true
    }

    fn title(&self) -> &str {
        "review"
    }
}

impl CardActionTrait for ReviewCard<'_> {}

pub trait CardActionTrait: Tab<AppState = CardCache> {
    fn evaluate(&mut self, card: Id, cache: &mut CardCache, action: CardAction) {
        let mut card = cache.get_owned(card);
        match action {
            CardAction::Open => {
                std::process::Command::new("open")
                    .arg(card.as_path())
                    .status()
                    .expect("Failed to open file");
            }
            CardAction::ToggleSuspend => {
                card.toggle_suspend();
            }
            CardAction::ToggleFinish => {
                let is_finished = card.is_finished();
                card.set_finished(!is_finished);
            }

            CardAction::TempSuspend => {
                let later = current_time() + Duration::from_secs(86400 * 14);
                card.set_suspended(IsSuspended::TrueUntil(later));
            }
            CardAction::NewDependent => {
                let x = Box::new(AddCard::new(
                    "Add new dependent",
                    card.category().to_owned(),
                    DependencyStatus::Dependency(card.id()).into(),
                ));

                self.set_popup(x);
            }
            CardAction::NewDependency => {
                let x = Box::new(AddCard::new(
                    "Add new dependent",
                    card.category().to_owned(),
                    DependencyStatus::Dependent(card.id()).into(),
                ));

                self.set_popup(x);
            }
            CardAction::OldDependent => {
                let the_cache = cache.clone();
                let card_id = card.id();
                let f = move |x: &Box<dyn Any>| {
                    let new_card: Id = *x.downcast_ref().unwrap();
                    let mut card = the_cache.get_owned(card_id);
                    card.set_dependent(new_card, &mut the_cache.inner.lock().unwrap());
                };

                let x = CardFinder::new(cache);

                self.set_popup_with_modifier(Box::new(x), Box::new(f));
            }
            CardAction::OldDependency => {
                let the_cache = cache.clone();
                let card_id = card.id();
                let f = move |x: &Box<dyn Any>| {
                    let new_card: Id = *x.downcast_ref().unwrap();
                    let mut card = the_cache.get_owned(card_id);
                    card.set_dependency(new_card, &mut the_cache.inner.lock().unwrap());
                };

                let x = CardFinder::new(cache);

                self.set_popup_with_modifier(Box::new(x), Box::new(f));
            }
            CardAction::Delete => {
                card.delete(&mut cache.inner.lock().unwrap());
            }
            CardAction::NewRelated => {}
            CardAction::OldRelated => {}
            CardAction::ClearHistory => card.clear_history(),
            CardAction::SwitchSides => card.switch_sides(),
            CardAction::Suspend => card.set_suspended(IsSuspended::True),
            CardAction::PlayFrontAudio => {
                if let Some(path) = card.front_audio_path() {
                    play_audio(path.clone()).ok();
                }
            }
            CardAction::PlayBackAudio => {
                if let Some(path) = card.back_audio_path() {
                    play_audio(path.clone()).ok();
                }
            }
            CardAction::SetPriority => {}
            CardAction::DecrPriority => card.decr_priority(),
            CardAction::IncrPriority => card.incr_priority(),
            CardAction::ClearPriority => card.clear_priority(),
            CardAction::Menu => {}
        }
    }
}

// Define a macro to easily collect widgets into a vector of trait objects.
#[derive(EnumIter, strum_macros::Display)]
pub enum CardAction {
    Open,
    ToggleSuspend,
    TempSuspend,
    ToggleFinish,
    NewDependent,
    NewDependency,
    NewRelated,
    OldDependent,
    OldDependency,
    OldRelated,
    Delete,
    ClearHistory,
    SwitchSides,
    Suspend,
    PlayFrontAudio,
    PlayBackAudio,
    Menu,
    SetPriority,
    IncrPriority,
    DecrPriority,
    ClearPriority,
}

impl FromStr for CardAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "o" => Ok(Self::Open),
            "s" => Ok(Self::ToggleSuspend),
            "S" => Ok(Self::TempSuspend),
            "y" => Ok(Self::OldDependency),
            "t" => Ok(Self::OldDependent),
            "Y" => Ok(Self::NewDependency),
            "T" => Ok(Self::NewDependent),
            "D" => Ok(Self::Delete),
            "f" => Ok(Self::ToggleFinish),
            "r" => Ok(Self::OldRelated),
            "R" => Ok(Self::NewRelated),
            "c" => Ok(Self::Menu),
            "p" => Ok(Self::DecrPriority),
            "P" => Ok(Self::IncrPriority),
            _ => Err(()),
        }
    }
}

pub trait CurrentCard {
    fn current_card(&self) -> Option<Id>;
}

fn play_audio(file_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    std::thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let Ok(file) = File::open(&file_path) else {
            dbg!("file path not found:", file_path.display());
            return;
        };

        let source = Decoder::new(BufReader::new(file)).unwrap();

        // Play audio on a separate thread
        stream_handle.play_raw(source.convert_samples()).unwrap();

        // You might want to handle how long to keep this thread alive,
        // or how to stop playback. This sleep is just a placeholder.
        std::thread::sleep(std::time::Duration::from_secs(10));
    });

    Ok(())
}
