use std::ops::ControlFlow;
use tabs::{addcards::CardAdder, *};

use browse::Browser;
use mischef::{App, Retning, Tab};
use ratatui::prelude::*;
use review::ReviewCard;
use speki_backend::cache::CardCache;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

mod popups;
mod tabs;
mod utils;
mod widgets;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;

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

    let mut cache = CardCache::new();

    let mut app = {
        let review = ReviewCard::new(&mut cache);
        let add_cards = CardAdder::new();
        let browse = Browser::new(&mut cache);
        let tabs: Vec<Box<dyn Tab<AppState = CardCache>>> =
            vec![Box::new(review), Box::new(add_cards), Box::new(browse)];

        App::new(cache, tabs)
    };

    loop {
        app.draw();

        match app.handle_key() {
            ControlFlow::Continue(_) => continue,
            ControlFlow::Break(_) => break,
        }
    }

    crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

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

fn split3(
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
