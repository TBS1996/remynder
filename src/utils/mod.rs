use std::{io::Read, path::Path};

use crossterm::event::KeyCode;
use ratatui::{
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{List, ListItem, ListState},
    Frame,
};
use speki_backend::Id;

use tui_tree_widget::{Tree, TreeItem, TreeState};

use mischef::Widget;

mod text_input;
pub use text_input::*;

mod text_display;
pub use text_display::*;

use crate::CardCache;

fn card_dependent_inner(card: Id, cache: &mut CardCache) -> TreeItem<'static, Id> {
    let dependents = cache.dependents(card);

    let mut children = Vec::new();
    for dependent in dependents {
        let child_item = card_dependent_inner(dependent, cache);
        children.push(child_item);
    }

    // Create a tree item for the current directory.
    TreeItem::new(card, cache.get_ref(card).front_text().to_string(), children).unwrap()
}

fn card_dependencies_inner(card: Id, cache: &mut CardCache) -> TreeItem<'static, Id> {
    let dependents = cache.dependencies(card);

    let mut children = Vec::new();
    for dependent in dependents {
        let child_item = card_dependencies_inner(dependent, cache);
        children.push(child_item);
    }

    // Create a tree item for the current directory.
    TreeItem::new(card, cache.get_ref(card).front_text().to_string(), children).unwrap()
}

pub fn card_dependents(card: Id, cache: &mut CardCache) -> Vec<TreeItem<'static, Id>> {
    let dependents = cache.dependents(card);
    let mut vec = vec![];

    for dependent in dependents {
        vec.push(card_dependent_inner(dependent, cache));
    }

    vec
}

pub fn card_dependencies(card: Id, cache: &mut CardCache) -> Vec<TreeItem<'static, Id>> {
    let card = cache.get_ref(card);
    let dependencies = card.dependency_ids();
    let mut vec = vec![];

    for dependency in dependencies {
        vec.push(card_dependencies_inner(*dependency, cache));
    }

    vec
}

#[derive(Debug)]
pub struct StatefulTree<'a, T> {
    pub state: TreeState<T>,
    pub items: Vec<TreeItem<'a, T>>,
}

impl<'a, T: Default + Eq + Clone + PartialEq + std::hash::Hash> StatefulTree<'a, T> {
    pub fn with_items(items: Vec<TreeItem<'a, T>>) -> Self {
        Self {
            state: TreeState::default(),
            items,
        }
    }

    pub fn first(&mut self) {
        self.state.select_first(&self.items);
    }

    pub fn last(&mut self) {
        self.state.select_last(&self.items);
    }

    pub fn down(&mut self) {
        self.state.key_down(&self.items);
    }

    pub fn up(&mut self) {
        self.state.key_up(&self.items);
    }

    pub fn left(&mut self) {
        self.state.key_left();
    }

    pub fn right(&mut self) {
        self.state.key_right();
    }
}

#[derive(Debug)]
pub struct TreeWidget<'a, T> {
    pub tree: StatefulTree<'a, T>,
    pub title: String,
}

impl<'a, T: Default + Eq + Clone + PartialEq + std::hash::Hash> TreeWidget<'a, T> {
    pub fn replace_items(&mut self, items: Vec<TreeItem<'a, T>>) {
        let tree = StatefulTree::with_items(items);
        self.tree = tree;
        self.open_all();
    }

    pub fn selected(&self) -> Option<T> {
        self.tree.state.selected().last().cloned()
    }

    pub fn clear(&mut self) {
        self.tree = StatefulTree::with_items(vec![]);
    }

    fn open_all(&mut self) {
        for _ in 0..50 {
            self.tree.right();
            self.tree.down();
        }

        for _ in 0..50 {
            self.tree.up();
        }
    }

    pub fn new_with_items(title: String, items: Vec<TreeItem<'a, T>>) -> Self {
        let tree = StatefulTree::with_items(items);

        let mut x = Self { tree, title };
        x.open_all();
        x
    }
}

impl<T: Default + Eq + Clone + PartialEq + std::hash::Hash> Widget for TreeWidget<'_, T> {
    type AppData = CardCache;
    fn keyhandler(&mut self, _cache: &mut CardCache, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Up => self.tree.up(),
            KeyCode::Down => self.tree.down(),
            KeyCode::Left => self.tree.left(),
            KeyCode::Char('k') => self.tree.up(),
            KeyCode::Char('j') => self.tree.down(),
            KeyCode::Char('h') => self.tree.left(),
            KeyCode::Char('l') => self.tree.right(),
            KeyCode::Right => self.tree.right(),
            KeyCode::Home => self.tree.first(),
            KeyCode::End => self.tree.last(),
            _ => {}
        };
    }

    fn render(&mut self, f: &mut Frame, _cache: &mut CardCache, area: Rect) {
        f.render_stateful_widget(
            Tree::new(self.tree.items.clone())
                .unwrap()
                .highlight_style(Style {
                    fg: Some(Color::Red),
                    ..Default::default()
                }),
            area,
            &mut self.tree.state,
        );
    }

    fn title(&self) -> &str {
        self.title.as_str()
    }
}

#[derive(Default)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }
        StatefulList { state, items }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn selected_mut(&mut self) -> Option<&mut T> {
        match self.state.selected() {
            Some(c) => Some(&mut self.items[c]),
            None => None,
        }
    }

    pub fn selected(&self) -> Option<&T> {
        match self.state.selected() {
            Some(c) => Some(&self.items[c]),
            None => None,
        }
    }
}

impl Widget for StatefulList<Id> {
    type AppData = CardCache;

    fn keyhandler(&mut self, _cache: &mut CardCache, key: crossterm::event::KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Up => self.previous(),
            crossterm::event::KeyCode::Down => self.next(),
            crossterm::event::KeyCode::Char('k') => self.previous(),
            crossterm::event::KeyCode::Char('j') => self.next(),
            _ => {}
        }
    }

    fn render(&mut self, f: &mut Frame, cache: &mut CardCache, area: Rect) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|i| {
                let front = cache
                    .try_get_ref(*i)
                    .map(|i| i.front_text().to_owned())
                    .unwrap_or("----".to_string());
                let lines = vec![Line::from(front)];
                ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items)
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        // We can now render the item list
        let mut state = self.state.clone();
        f.render_stateful_widget(items, area, &mut state);
    }
}

pub fn _read_text_file<P: AsRef<Path>>(path: P) -> Option<String> {
    let path = path.as_ref();
    if !path.is_file() {
        return None;
    }

    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return None,
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => Some(contents),
        Err(_) => None,
    }
}
