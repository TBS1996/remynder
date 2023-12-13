use std::{collections::BTreeMap, path::PathBuf};

use crossterm::event::KeyCode;
use mischef::Widget;
use ratatui::{
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{List, ListItem},
};
use tordir::DirEntry;

use crate::{utils::StatefulList, CardCache};

pub struct FileFinder {
    list: StatefulList<DirEntry>,
    dir: PathBuf,
    show_files: bool,
    indices: BTreeMap<PathBuf, usize>,
}

impl FileFinder {
    pub fn new() -> Self {
        let dir = dirs::home_dir().unwrap();
        let mut s = Self {
            show_files: true,
            list: StatefulList::with_items(vec![]),
            dir,
            indices: BTreeMap::default(),
        };
        s.update_list();
        s
    }

    pub fn selected(&self) -> Option<PathBuf> {
        self.list.selected().map(|e| e.clone().into())
    }

    pub fn update_list(&mut self) {
        let mut items = DirEntry::load_all(self.dir.as_path());
        items.retain(|entry| match entry {
            DirEntry::Dir(_) => true,
            DirEntry::File(_) => self.show_files,
        });

        let idx = self
            .indices
            .get(&self.dir)
            .filter(|_| !items.is_empty())
            .map(|idx| (*idx).clamp(0, items.len() - 1));

        self.list.state.select(idx);

        self.list = StatefulList::with_items(items);
    }
}

impl Widget for FileFinder {
    type AppData = CardCache;

    fn keyhandler(&mut self, _app_data: &mut Self::AppData, key: crossterm::event::KeyEvent) {
        let KeyCode::Char(c) = key.code else { return };

        match c {
            'j' => self.list.next(),
            'k' => self.list.previous(),
            'h' => {
                if let Some(idx) = self.list.state.selected() {
                    self.indices.insert(self.dir.clone(), idx);
                }

                self.dir.pop();
                self.update_list();
            }
            'l' => {
                if let Some(DirEntry::Dir(p)) = self.list.selected() {
                    if let Some(idx) = self.list.state.selected() {
                        self.indices.insert(self.dir.clone(), idx);
                    }

                    self.dir = p.clone();
                    self.update_list();
                }
            }
            _ => {}
        }
    }

    fn render(
        &mut self,
        f: &mut ratatui::Frame,
        _app_data: &mut Self::AppData,
        area: ratatui::prelude::Rect,
    ) {
        let dir_entries: Vec<&DirEntry> = self.list.items.iter().collect();

        let items: Vec<ListItem> = dir_entries
            .iter()
            .map(|i| {
                let mut s = i.as_string();
                if i.is_dir() {
                    s.push('/');
                }

                let lines = vec![Line::from(s)];
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
