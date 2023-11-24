use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

use tabs::{addcards::CardAdder, *};

use browse::Browser;
use mischef::{App, Retning, Tab};
use ratatui::prelude::*;
use review::ReviewCard;
use speki_backend::{cache::CardCache as CardCacheInner, card::SavedCard, Id};
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

        let review = ReviewCard::new(&mut cache);
        let add_cards = CardAdder::new(&mut cache);
        let browse = Browser::new(&mut cache, false);
        let stats = Stats::new(&mut cache);
        let tabs: Vec<Box<dyn Tab<AppState = CardCache>>> = vec![
            Box::new(review),
            Box::new(add_cards),
            Box::new(browse),
            Box::new(stats),
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
pub struct Pipeline<T> {
    pre: Vec<T>,
    current: Option<T>,
    done: Vec<T>,
}

impl<T> Pipeline<T> {
    pub fn new(items: Vec<T>) -> Self {
        let qty = items.len();

        let mut s = Self {
            pre: items,
            current: None,
            done: Vec::with_capacity(qty),
        };
        s.next();

        s
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

    pub fn tot_qty(&self) -> usize {
        self.pre.len() + if self.current.is_some() { 1 } else { 0 } + self.done.len()
    }
}
