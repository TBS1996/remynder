use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::Rect;
use speki_backend::{card::Card, categories::Category, Id};

use mischef::{Tab, TabData, Widget};

use crate::{
    popups::CatChoice,
    split_off,
    utils::{TextDisplay, TextInput},
    vsplit2, CardCache, MyTabData, ReturnType,
};

#[derive(Clone, Debug)]
pub enum DependencyStatus {
    Dependent(Id),
    Dependency(Id),
}

#[derive(Debug, Default)]
pub struct AddCard<'a> {
    front: TextInput<'a>,
    back: TextInput<'a>,
    status_bar: TextDisplay,
    category: Category,
    tabdata: MyTabData,
    dependency: Option<DependencyStatus>,
    message: String,
}

impl<'a> AddCard<'a> {
    pub fn new(
        message: impl Into<String>,
        category: Category,
        dependency: Option<DependencyStatus>,
    ) -> Self {
        let mut s = Self {
            category,
            message: message.into(),
            dependency,
            ..Default::default()
        };
        s.refresh();
        s
    }

    fn refresh(&mut self) {
        self.status_bar.text = format!("{}    {}", self.message, self.category.print_full());
    }
}

fn split_area(area: Rect) -> (Rect, Rect, Rect) {
    let (status, area) = split_off(area, 1, crate::Retning::Up);
    let (front, back) = vsplit2(area, 50, 50);
    (status, front, back)
}

impl Tab for AddCard<'_> {
    type AppState = CardCache;
    type ReturnType = ReturnType;

    fn widgets(&mut self, area: Rect) -> Vec<(&mut dyn Widget<AppData = Self::AppState>, Rect)> {
        let (status, front, back) = split_area(area);

        vec![
            (&mut self.status_bar, status),
            (&mut self.front, front),
            (&mut self.back, back),
        ]
    }

    fn pre_render_hook(&mut self, _app_data: &mut Self::AppState) {
        if !self.tabdata().first_pass {
            let id = self.front.id();
            self.move_to_id(&id);
            self.tabdata.is_selected = true;
        }
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState, Self::ReturnType> {
        &mut self.tabdata
    }

    fn tabdata_ref(&self) -> &TabData<Self::AppState, Self::ReturnType> {
        &self.tabdata
    }

    fn title(&self) -> &str {
        "new card"
    }

    fn handle_popup_value(&mut self, _app_data: &mut Self::AppState, value: ReturnType) {
        if let ReturnType::Category(category) = value {
            self.category = category;
        }
        self.refresh();
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
            let dependency = self.dependency.clone();
            let old_self = std::mem::take(self);
            self.category = old_self.category;
            self.refresh();

            let mut card = Card::new_simple(
                old_self.front.text.into_lines().join("\n"),
                old_self.back.text.into_lines().join("\n"),
            );

            if card.front.is_empty() {
                return false;
            };

            card.finished = key.code == KeyCode::Enter;
            let card = card.save_new_card(&self.category, &mut cache.inner.lock().unwrap());

            match dependency {
                Some(DependencyStatus::Dependency(id)) => cache.set_dependency(card.id(), id),
                Some(DependencyStatus::Dependent(id)) => cache.set_dependency(id, card.id()),
                None => {}
            };

            self.resolve_tab(ReturnType::SavedCard(card));

            return false;
        }

        true
    }

    fn tab_keyhandler_deselected(&mut self, _cache: &mut CardCache, key: KeyEvent) -> bool {
        if key.code == KeyCode::Char('c') {
            self.set_popup(Box::new(CatChoice::new()));
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let _area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 100,
        };
    }
}
