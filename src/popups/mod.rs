use crossterm::event::KeyCode;
use mischef::{Tab, TabData, Widget};
use speki_backend::{cache::CardCache, filter::FilterUtil};

use crate::widgets::table_thing::InputTable;

mod choose_category;
pub use choose_category::*;

mod add_card;
pub use add_card::*;

mod choose_filter;
pub use choose_filter::*;

mod cardviewer;
pub use cardviewer::*;
