use crossterm::event::KeyCode;
use mischef::{Tab, TabData, Widget};
use speki_backend::{cache::IncRead, card::Card};

use crate::{
    hsplit2, open_text, split_off,
    utils::{TextDisplay, TextInput},
    vsplit2, CardCache, ReturnType,
};

pub struct IncrementalReading<'a> {
    incs: Vec<IncRead>,
    title: TextDisplay,
    info: TextDisplay,
    front: TextInput<'a>,
    back: TextInput<'a>,
    tab_data: TabData<CardCache, ReturnType>,
    idx: usize,
}

const CHAR_LENGTH: usize = 500;

impl IncrementalReading<'_> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut incs = IncRead::load_all();
        if incs.is_empty() {
            incs = vec![IncRead::new_empty()];
        }

        let mut s = Self {
            incs,
            idx: 0,
            tab_data: TabData::default(),
            front: TextInput::default(),
            back: TextInput::default(),
            info: TextDisplay::default(),
            title: TextDisplay::default(),
        };
        s.reset();
        s
    }

    pub fn new_single(inc: IncRead) -> Self {
        let mut s = Self {
            incs: vec![inc],
            idx: 0,
            tab_data: TabData::default(),
            front: TextInput::default(),
            back: TextInput::default(),
            info: TextDisplay::default(),
            title: TextDisplay::default(),
        };
        s.reset();
        s
    }

    pub fn current_inc(&mut self) -> &mut IncRead {
        &mut self.incs[self.idx]
    }

    pub fn reset(&mut self) {
        self.current_inc().save();

        self.title.text = self.current_inc().title();
        let percentage = self.current_inc().percentage() * 100.;
        let progress = format!("        {:.2}%", percentage);
        self.title.text.push_str(progress.as_str());

        let mut text = self.current_inc().text_slice(CHAR_LENGTH);
        if self.current_inc().is_done() {
            text.push_str("\n--------ALL DONE---------");
        }

        self.info.text = text;
    }

    pub fn next(&mut self) {
        self.idx = (self.idx + 1) % self.incs.len()
    }

    pub fn prev(&mut self) {
        self.idx = (self.idx + self.incs.len() - 1) % self.incs.len();
    }

    pub fn new_inc(&mut self) {
        let new = IncRead::new_empty();
        self.incs.push(new);
    }
}

impl Tab for IncrementalReading<'_> {
    type AppState = CardCache;
    type ReturnType = ReturnType;

    fn tab_keyhandler_deselected(
        &mut self,
        _app_data: &mut Self::AppState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        let KeyCode::Char(c) = key.code else {
            return true;
        };

        match c {
            'd' => self.current_inc().increment(CHAR_LENGTH),
            'a' => self.current_inc().decrement(CHAR_LENGTH),
            'D' => self.next(),
            'A' => self.prev(),
            'n' => self.new_inc(),
            'r' => self.current_inc().reload_task(),
            'o' => open_text(self.current_inc().path().as_path()),
            'e' => {
                let idx = self.idx;
                self.next();

                // keep going to next until either we find one that's not marked done,
                // or we've made a roundtrip.
                while self.idx != idx && self.current_inc().is_done() {
                    self.next();
                }
            }
            'q' => {
                let idx = self.idx;
                self.prev();

                // keep going to next until either we find one that's not marked done,
                // or we've made a roundtrip.
                while self.idx != idx && self.current_inc().is_done() {
                    self.prev();
                }
            }
            _ => return true,
        }
        self.reset();

        true
    }

    fn tab_keyhandler_selected(
        &mut self,
        cache: &mut Self::AppState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        if self.is_selected(&self.front) && key.code == KeyCode::Enter {
            self.move_to_id(self.back.id().as_str());
            return false;
        } else if self.is_selected(&self.back) && key.code == KeyCode::Char('`')
            || key.code == KeyCode::Enter
        {
            let mut card = Card::new_simple(self.front.get_text(), self.back.get_text());

            if card.front.is_empty() {
                return false;
            };

            let category = self.current_inc().category();

            card.finished = key.code == KeyCode::Enter;
            let card = card.save_new_card(category, &mut cache.inner.lock().unwrap());

            self.current_inc().new_card(card.id());

            self.front.clear();
            self.back.clear();

            self.move_to_id(self.front.id().as_str());

            return false;
        }
        true
    }

    fn widgets(
        &mut self,
        area: ratatui::prelude::Rect,
    ) -> Vec<(
        &mut dyn mischef::Widget<AppData = Self::AppState>,
        ratatui::prelude::Rect,
    )> {
        let (title, area) = split_off(area, 3, mischef::Retning::Up);
        let (info, card) = hsplit2(area, 50, 50);
        let (front, back) = vsplit2(card, 50, 50);

        vec![
            (&mut self.title, title),
            (&mut self.info, info),
            (&mut self.front, front),
            (&mut self.back, back),
        ]
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState, Self::ReturnType> {
        &mut self.tab_data
    }

    fn tabdata_ref(&self) -> &TabData<Self::AppState, Self::ReturnType> {
        &self.tab_data
    }

    fn title(&self) -> &str {
        "incremental reading"
    }
}
