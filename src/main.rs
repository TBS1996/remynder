use std::ops::ControlFlow;

use addcards::AddCard;
use browse::Browser;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use review::ReviewCard;
use speki_backend::cache::CardCache;
use tracing::instrument;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};
use ui_library::Tab;

mod addcards;
mod browse;
mod choose_category;
mod review;
mod ui_library;
mod utils;

type Term = ratatui::Terminal<Bakende>;
type Bakende = ratatui::backend::CrosstermBackend<std::io::Stderr>;

pub trait Page {
    fn draw(&self, f: &mut Frame, area: Rect, cache: &mut CardCache);
    fn handle_key(&mut self, key: KeyEvent, cache: &mut CardCache);
}

struct App {
    cache: CardCache,
    terminal: Term,
    tab_idx: usize,
    tabs: Vec<Box<dyn Tab>>,
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("cache", &"~")
            .field("terminal", &self.terminal)
            .field("tab_idx", &self.tab_idx)
            .field("tabs", &"~")
            .finish()
    }
}

impl App {
    fn new() -> Self {
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stderr())).unwrap();
        let mut cache = CardCache::new();
        let review = Box::new(ReviewCard::new(&mut cache));
        let browser = Box::new(Browser::new(&mut cache));

        let add_card = Box::new(AddCard::new());
        let tabs: Vec<Box<dyn Tab>> = vec![review, add_card, browser];

        Self {
            terminal,
            cache,
            tabs,
            tab_idx: 0,
        }
    }

    #[instrument]
    fn draw(&mut self) {
        let idx = self.tab_idx;

        self.terminal
            .draw(|f| {
                let (tab_area, remainder_area) = split_off(f.size(), 3, Retning::Up);

                let tabs = Tabs::new(self.tabs.iter().map(|tab| tab.title()).collect())
                    .block(Block::default().borders(Borders::ALL))
                    .style(Style::default().white())
                    .highlight_style(Style::default().light_red())
                    .select(idx)
                    .divider(symbols::DOT);

                f.render_widget(tabs, tab_area);

                self.tabs[self.tab_idx].entry_render(f, &mut self.cache, remainder_area);
            })
            .unwrap();
    }

    #[instrument]
    fn handle_key(&mut self) -> ControlFlow<()> {
        let key = get_event();

        if let Event::Key(x) = key {
            if x.code == KeyCode::Tab {
                self.go_right()
            } else if x.code == KeyCode::BackTab {
                self.go_left()
            };
        }

        self.tabs[self.tab_idx].entry_keyhandler(key, &mut self.cache)
    }

    fn go_right(&mut self) {
        self.tab_idx = std::cmp::min(self.tab_idx + 1, self.tabs.len() - 1);
    }

    fn go_left(&mut self) {
        if self.tab_idx != 0 {
            self.tab_idx -= 1;
        }
    }
}

fn get_event() -> Event {
    event::read().unwrap()
}

fn get_key_event() -> KeyEvent {
    loop {
        let key = event::read().unwrap();
        if let event::Event::Key(c) = key {
            return c;
        }
    }
}

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

    let mut app = App::new();

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

#[derive(Debug)]
pub enum Retning {
    Up,
    Down,
    Left,
    Right,
}

impl TryFrom<KeyEvent> for Retning {
    type Error = ();

    fn try_from(value: KeyEvent) -> Result<Self, Self::Error> {
        match value.code {
            KeyCode::Left => Ok(Self::Left),
            KeyCode::Right => Ok(Self::Right),
            KeyCode::Up => Ok(Self::Up),
            KeyCode::Down => Ok(Self::Down),
            KeyCode::Char('k') => Ok(Self::Up),
            KeyCode::Char('j') => Ok(Self::Down),
            KeyCode::Char('h') => Ok(Self::Left),
            KeyCode::Char('l') => Ok(Self::Right),
            _ => Err(()),
        }
    }
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
