use std::{marker::PhantomData, str::FromStr};

use mischef::{Tab, TabData, Widget};
use ratatui::{
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{List, ListItem},
    Frame,
};
use strum::IntoEnumIterator;

use crate::{tabs::review::CardAction, utils::StatefulList, CardCache};

fn get_enum_options<T: IntoEnumIterator + std::fmt::Display>() -> Vec<String> {
    T::iter().map(|x| x.to_string()).collect()
}

pub struct EnumChoice<T: IntoEnumIterator> {
    list: StatefulList<String>,
    tabdata: TabData<CardCache>,
    _marker: PhantomData<T>,
}

impl<T> EnumChoice<T>
where
    T: IntoEnumIterator + std::fmt::Display + FromStr,
{
    pub fn new() -> Self {
        let items = get_enum_options::<T>();
        if items.is_empty() {
            panic!("plz no unit enums");
        }
        let mut list = StatefulList::with_items(items);
        list.next();

        Self {
            list,
            tabdata: TabData::default(),
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.list.items.len()
    }

    pub fn current_item(&self) -> T
    where
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        let selected = self.list.selected().unwrap();
        T::from_str(selected).unwrap()
    }
}

impl<T: strum::IntoEnumIterator> Widget for EnumChoice<T> {
    type AppData = CardCache;

    fn keyhandler(&mut self, _cache: &mut CardCache, key: crossterm::event::KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Up => self.list.previous(),
            crossterm::event::KeyCode::Down => self.list.next(),
            crossterm::event::KeyCode::Char('k') => self.list.previous(),
            crossterm::event::KeyCode::Char('j') => self.list.next(),
            _ => {}
        }
    }

    fn render(&mut self, f: &mut Frame, _cache: &mut CardCache, area: Rect) {
        let items: Vec<ListItem> = self
            .list
            .items
            .iter()
            .map(|i| {
                let lines = vec![Line::from(i.clone())];
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
        let mut state = self.list.state.clone();
        f.render_stateful_widget(items, area, &mut state);
    }
}

impl Tab for EnumChoice<CardAction> {
    type AppState = CardCache;

    fn widgets(&mut self, area: Rect) -> Vec<(&mut dyn Widget<AppData = Self::AppState>, Rect)> {
        vec![(self, area)]
    }

    fn title(&self) -> &str {
        "choose card action"
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState> {
        &mut self.tabdata
    }

    fn tabdata_ref(&self) -> &TabData<Self::AppState> {
        &self.tabdata
    }
}
