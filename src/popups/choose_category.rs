use std::path::PathBuf;

use crossterm::event::KeyCode;
use mischef::{Tab, TabData, Widget};

use speki_backend::categories::Category;
use tui_tree_widget::TreeItem;

use crate::{utils::TreeWidget, CardCache, MyTabData, ReturnType};

#[derive(Debug)]
pub struct CatChoice<'a> {
    tree: TreeWidget<'a, PathBuf>,
    tabdata: MyTabData,
}

impl Tab for CatChoice<'_> {
    type AppState = CardCache;
    type ReturnType = ReturnType;

    fn widgets(
        &mut self,
        area: ratatui::prelude::Rect,
    ) -> Vec<(
        &mut dyn Widget<AppData = Self::AppState>,
        ratatui::prelude::Rect,
    )> {
        vec![(&mut self.tree, area)]
    }

    fn tabdata(&mut self) -> &mut TabData<Self::AppState, Self::ReturnType> {
        &mut self.tabdata
    }

    fn title(&self) -> &str {
        "choose category"
    }

    fn tab_keyhandler(
        &mut self,
        cache: &mut Self::AppState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        if key.code == KeyCode::Enter {
            if let Some(p) = self.tree.selected() {
                let category = Category::from_dir_path(p.as_path());
                self.resolve_tab(ReturnType::Category(category));
            }
        } else if key.code == KeyCode::Esc {
            self.exit_tab();
        }

        self.tree.keyhandler(cache, key);
        false
    }

    fn tabdata_ref(&self) -> &TabData<Self::AppState, Self::ReturnType> {
        &self.tabdata
    }
}

impl CatChoice<'_> {
    pub fn new() -> Self {
        let b = build_tree_item(speki_backend::paths::get_cards_path());
        let tree = TreeWidget::new_with_items("choose category".into(), vec![b]);
        let view = TabData {
            is_selected: true,
            ..Default::default()
        };

        Self {
            tree,
            tabdata: view,
        }
    }
}

fn build_tree_item(path: PathBuf) -> TreeItem<'static, PathBuf> {
    let dir_name = path.file_name().unwrap().to_str().unwrap().to_string();

    let subdirs = tordir::DirEntry::load_dirs(path.as_path());
    let mut children = Vec::new();

    for subdir in subdirs {
        let child_item = build_tree_item(subdir);
        children.push(child_item);
    }

    TreeItem::new(path, dir_name.clone(), children).unwrap()
}
