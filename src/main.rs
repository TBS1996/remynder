use std::{
    any::Any,
    collections::BTreeSet,
    fmt::Debug,
    fs::{read_to_string, File},
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use popups::{AddCard, CardFinder, CatChoice, DependencyStatus};
use rodio::{Decoder, OutputStream, Source};
use sentry::types::Uuid;
use strum_macros::{EnumIter, EnumString};
use tabs::{addcards::CardAdder, *};

use browse::Browser;
use mischef::{App, Retning, Tab};
use ratatui::prelude::*;
use review::ReviewMenu;
use speki_backend::{
    cache::{CardCache as CardCacheInner, IncRead},
    card::{Card, IsSuspended},
    categories::{Category, CategoryMeta},
    common::current_time,
    saved_card::SavedCard,
    Id,
};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

mod popups;
mod tabs;
mod utils;
mod widgets;

#[derive(Debug, Clone, Default)]
pub struct CardCache {
    pub inner: Arc<Mutex<CardCacheInner>>,
}

impl CardCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(CardCacheInner::new())),
        }
    }

    pub fn all_ids(&self) -> Vec<Id> {
        self.inner.lock().unwrap().all_ids()
    }

    pub fn card_qty(&self) -> usize {
        self.inner.lock().unwrap().card_qty()
    }

    pub fn try_get_ref(&self, id: Id) -> Option<Arc<SavedCard>> {
        self.inner.lock().unwrap().try_get_ref(id)
    }

    pub fn get_ref(&self, id: Id) -> Arc<SavedCard> {
        self.inner.lock().unwrap().get_ref(id)
    }

    pub fn get_owned(&self, id: Id) -> SavedCard {
        self.inner.lock().unwrap().get_owned(id)
    }

    pub fn ids_as_vec(&self) -> Vec<Id> {
        self.inner.lock().unwrap().ids_as_vec()
    }

    pub fn dependents(&mut self, id: Id) -> BTreeSet<Id> {
        self.inner.lock().unwrap().dependents(id)
    }

    pub fn dependencies(&mut self, id: Id) -> BTreeSet<Id> {
        self.inner.lock().unwrap().dependencies(id)
    }

    pub fn set_dependency(&mut self, dependent: Id, dependency: Id) {
        self.inner
            .lock()
            .unwrap()
            .set_dependency(dependent, dependency)
    }

    pub fn delete_card(&mut self, id: Id) {
        self.inner.lock().unwrap().delete_card(id)
    }
    pub fn clear_dependencies(&mut self, id: Id) {
        self.inner.lock().unwrap().clear_dependencies(id);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = sentry::init(("https://94a749520f9a39941b13f7559b94e9ea@o4504644012736512.ingest.sentry.io/4506144752205824", sentry::ClientOptions {
        release: sentry::release_name!(),
        // To set a uniform sample rate
        traces_sample_rate: 1.0,
        // The Rust SDK does not currently support `traces_sampler`
        ..Default::default()
    }));

    tracing_subscriber::Registry::default()
        .with(sentry::integrations::tracing::layer())
        .init();

    std::env::set_var("RUST_BACKTRACE", "1");

    let mut app = {
        let mut cache = CardCache::new();
        //CardCacheInner::reset_serialize();

        let review = ReviewMenu::new();
        let add_cards = CardAdder::new(&mut cache);
        let browse = Browser::new(&mut cache, false);
        let stats = Stats::new(&mut cache);
        let import = Importer::new();
        let incread = IncrementalReading::new();
        let tabs: Vec<Box<dyn Tab<AppState = CardCache>>> = vec![
            Box::new(review),
            Box::new(add_cards),
            Box::new(browse),
            Box::new(incread),
            Box::new(stats),
            Box::new(import),
        ];

        App::new(cache, tabs)
    };

    app.run();

    Ok(())
}

pub fn split_off(area: Rect, length: u16, direction: Retning) -> (Rect, Rect) {
    let constraints = match direction {
        Retning::Up | Retning::Left => vec![Constraint::Length(length), Constraint::Min(0)],
        Retning::Down | Retning::Right => vec![Constraint::Min(0), Constraint::Length(length)],
    };

    let direction = match direction {
        Retning::Up => Direction::Vertical,
        Retning::Down => Direction::Vertical,
        Retning::Left => Direction::Horizontal,
        Retning::Right => Direction::Horizontal,
    };
    let chunks = Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(area)
        .to_vec();

    (chunks[0], chunks[1])
}

pub fn vsplit2(area: Rect, a: u16, b: u16) -> (Rect, Rect) {
    let chunks = splitter_percent(area, Direction::Vertical, vec![a, b]);
    (chunks[0], chunks[1])
}

pub fn hsplit2(area: Rect, a: u16, b: u16) -> (Rect, Rect) {
    let chunks = splitter_percent(area, Direction::Horizontal, vec![a, b]);
    (chunks[0], chunks[1])
}

pub fn vsplit3(area: Rect, a: u16, b: u16, c: u16) -> (Rect, Rect, Rect) {
    let chunks = splitter_percent(area, Direction::Vertical, vec![a, b, c]);
    (chunks[0], chunks[1], chunks[2])
}

pub fn hsplit3(area: Rect, a: u16, b: u16, c: u16) -> (Rect, Rect, Rect) {
    let chunks = splitter_percent(area, Direction::Horizontal, vec![a, b, c]);
    (chunks[0], chunks[1], chunks[2])
}

fn splitter_percent(area: Rect, dir: Direction, splits: Vec<u16>) -> Vec<Rect> {
    let constraints: Vec<Constraint> = splits.into_iter().map(Constraint::Percentage).collect();
    splitter(area, dir, constraints)
}

fn _split2(area: Rect, dir: Direction, a: Constraint, b: Constraint) -> (Rect, Rect) {
    let chunks = splitter(area, dir, vec![a, b]);
    (chunks[0], chunks[1])
}

fn _split3(
    area: Rect,
    dir: Direction,
    a: Constraint,
    b: Constraint,
    c: Constraint,
) -> (Rect, Rect, Rect) {
    let chunks = splitter(area, dir, vec![a, b, c]);
    (chunks[0], chunks[1], chunks[2])
}

fn splitter(area: Rect, dir: Direction, constraints: Vec<Constraint>) -> Vec<Rect> {
    Layout::default()
        .direction(dir)
        .constraints(constraints)
        .split(area)
        .to_vec()
}

/// approximate!!!!
pub fn line_qty(text: &str, area: Rect) -> u16 {
    let char_qty = text.chars().count() as u16;
    char_qty / area.width + 2
}

/// Represents a bunch of items getting processed one by one.
#[derive(Debug)]
pub struct Pipeline<T> {
    pre: Vec<T>,
    current: Option<T>,
    done: Vec<T>,
}

impl<T: Debug> Pipeline<T> {
    pub fn new(items: Vec<T>) -> Self {
        let qty = items.len();

        Self {
            pre: items,
            current: None,
            done: Vec::with_capacity(qty),
        }
    }

    pub fn next(&mut self) {
        if let Some(val) = self.current.take() {
            self.done.push(val);
        }

        self.current = self.pre.pop();
    }

    pub fn current(&self) -> Option<&T> {
        self.current.as_ref()
    }

    pub fn is_done(&self) -> bool {
        self.pre.is_empty() && self.current.is_none()
    }

    pub fn finished_qty(&self) -> usize {
        self.done.len()
    }

    pub fn unfinished_qty(&self) -> usize {
        self.tot_qty() - self.finished_qty()
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.finished_qty(), self.tot_qty())
    }

    pub fn tot_qty(&self) -> usize {
        self.pre.len() + if self.current.is_some() { 1 } else { 0 } + self.done.len()
    }
}

pub fn open_text(p: &Path) {
    std::process::Command::new("open")
        .arg(p)
        .status()
        .expect("Failed to open file");
}

pub trait CardActionTrait: Tab<AppState = CardCache> {
    fn evaluate_current(&mut self, cache: &mut CardCache, action: CardAction)
    where
        Self: CurrentCard,
    {
        for card in self.selected_cards() {
            self.evaluate(card, cache, action);
        }
    }

    fn evaluate(&mut self, card: Id, cache: &mut CardCache, action: CardAction) {
        let mut card = cache.get_owned(card);
        match action {
            CardAction::ChangeCategory => {
                let p = CatChoice::new();
                let the_card = card.id();
                let the_cache = cache.clone();
                let f = move |x: &Box<dyn Any>| {
                    let category: &Category = x.downcast_ref().unwrap();
                    let card = the_cache.get_owned(the_card);
                    card.move_card(category, &mut the_cache.inner.lock().unwrap());
                };

                self.set_popup_with_modifier(Box::new(p), Box::new(f));
            }
            CardAction::ReverseDependency => {
                let inner: Card = card.clone().into();
                let mut new_card = inner.clone();
                new_card.id = Uuid::new_v4();
                let mut new_card =
                    new_card.save_new_card(card.category(), &mut cache.inner.lock().unwrap());
                new_card.switch_sides();
                cache.set_dependency(card.id(), new_card.id());
            }
            CardAction::Open => open_text(card.path().as_path()),
            CardAction::ToggleSuspend => card.toggle_suspend(),
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
                let mut the_cache = cache.clone();
                let card_id = card.id();
                let f = move |x: &Box<dyn Any>| {
                    let new_card: Id = *x.downcast_ref().unwrap();
                    the_cache.set_dependency(new_card, card_id);
                };

                let popup = CardFinder::new(cache);
                self.set_popup_with_modifier(Box::new(popup), Box::new(f));
            }
            CardAction::OldDependency => {
                let mut the_cache = cache.clone();
                let card_id = card.id();
                let f = move |x: &Box<dyn Any>| {
                    let new_card: Id = *x.downcast_ref().unwrap();
                    the_cache.set_dependency(card_id, new_card);
                };

                let popup = CardFinder::new(cache);
                self.set_popup_with_modifier(Box::new(popup), Box::new(f));
            }
            CardAction::Delete => cache.delete_card(card.id()),
            CardAction::NewRelated => {}
            CardAction::OldRelated => {}
            CardAction::ClearDependencies => cache.clear_dependencies(card.id()),
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

#[derive(EnumIter, strum_macros::Display, Clone, Copy, EnumString)]
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
    ReverseDependency,
    ClearDependencies,
    ChangeCategory,
}

impl CardAction {
    pub fn from_char(s: &str) -> Result<Self, ()> {
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
            // "c" => Ok(Self::Menu),
            "p" => Ok(Self::DecrPriority),
            "P" => Ok(Self::IncrPriority),
            "z" => Ok(Self::ChangeCategory),
            _ => Err(()),
        }
    }
}

pub fn play_audio(file_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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
