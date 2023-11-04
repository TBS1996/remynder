use crossterm::event::KeyCode;
use ratatui::{
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use speki_backend::{cache::CardCache, Id};
use tui_textarea::TextArea;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::ui_library::Widget;

fn card_dependencies_inner(card: Id, cache: &mut CardCache) -> TreeItem<'static, Id> {
    let binding = cache.get_ref(card);
    let dependencies = binding.dependency_ids();

    let mut children = Vec::new();
    for dependency in dependencies {
        let child_item = card_dependencies_inner(*dependency, cache);
        children.push(child_item);
    }

    // Create a tree item for the current directory.
    TreeItem::new(card, cache.get_ref(card).front_text().to_string(), children).unwrap()
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

#[derive(Default, Debug)]
pub struct StatusBar {
    pub text: String,
    area: Rect,
}

impl Widget for StatusBar {
    fn keyhandler(&mut self, _cache: &mut CardCache, _key: crossterm::event::KeyEvent) {}

    fn render(&mut self, f: &mut Frame, _cache: &mut CardCache, area: ratatui::layout::Rect) {
        f.render_widget(
            Paragraph::new(self.text.as_str()).wrap(Wrap { trim: true }),
            area,
        );
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
}

#[derive(Default, Debug)]
pub struct TextInput<'a> {
    pub text: TextArea<'a>,
    area: Rect,
}

impl Widget for TextInput<'_> {
    fn keyhandler(&mut self, _cache: &mut CardCache, key: crossterm::event::KeyEvent) {
        self.text.input(key);
    }

    fn render(&mut self, f: &mut Frame, _cache: &mut CardCache, area: ratatui::layout::Rect) {
        f.render_widget(self.text.widget(), area);
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
}

#[derive(Debug)]
pub struct TreeWidget<'a, T> {
    pub tree: StatefulTree<'a, T>,
    pub area: Rect,
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

        let mut x = Self {
            tree,
            area: Rect::default(),
            title,
        };
        x.open_all();
        x
    }
}

impl<T: Default + Eq + Clone + PartialEq + std::hash::Hash> Widget for TreeWidget<'_, T> {
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

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }

    fn title(&self) -> &str {
        self.title.as_str()
    }
}

pub struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
    area: Rect,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
            area: Rect::default(),
        }
    }

    fn next(&mut self) {
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

    fn previous(&mut self) {
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

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn selected(&self) -> Option<&T> {
        match self.state.selected() {
            Some(c) => Some(&self.items[c]),
            None => None,
        }
    }
}

impl Widget for StatefulList<Id> {
    fn keyhandler(&mut self, cache: &mut CardCache, key: crossterm::event::KeyEvent) {
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
                let front = cache.get_ref(*i).front_text().to_owned();
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

    fn area(&self) -> Rect {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
}
