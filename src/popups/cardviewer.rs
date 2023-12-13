use speki_backend::Id;

use crate::{
    hsplit2,
    utils::{TextInput, TreeWidget},
    vsplit2, MyTabData, ReturnType,
};

use super::*;

pub struct CardInspector<'a> {
    _card: Id,
    front: TextInput<'a>,
    back: TextInput<'a>,
    dependencies: TreeWidget<'a, Id>,
    dependents: TreeWidget<'a, Id>,
    tab_data: MyTabData,
}

impl CardInspector<'_> {
    pub fn new(card_id: Id, cache: &mut CardCache) -> Self {
        let card = cache.get_ref(card_id);
        let f = TextInput::new(card.front_text().to_string());
        let b = TextInput::new(card.back_text().to_string());

        Self {
            _card: card_id,
            front: f,
            back: b,
            tab_data: TabData::default(),
            dependencies: TreeWidget::new_with_items("Dependencies".into(), vec![]),
            dependents: TreeWidget::new_with_items("Dependents".into(), vec![]),
        }
    }
}

impl<'a> Tab for CardInspector<'a> {
    type AppState = CardCache;
    type ReturnType = ReturnType;

    fn title(&self) -> &str {
        "inspect card"
    }

    fn widgets(
        &mut self,
        area: ratatui::prelude::Rect,
    ) -> Vec<(
        &mut dyn Widget<AppData = Self::AppState>,
        ratatui::prelude::Rect,
    )> {
        let (left, right) = hsplit2(area, 50, 50);
        let (front, back) = vsplit2(left, 50, 50);

        let (dependencies, dependents) = vsplit2(right, 50, 50);

        vec![
            (&mut self.front, front),
            (&mut self.back, back),
            (&mut self.dependencies, dependencies),
            (&mut self.dependents, dependents),
        ]
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState, Self::ReturnType> {
        &mut self.tab_data
    }

    fn tabdata_ref(&self) -> &TabData<Self::AppState, Self::ReturnType> {
        &self.tab_data
    }
}
