use std::{any::Any, marker::PhantomData};

use crossterm::event::KeyCode;
use mischef::Widget;
use ratatui::{
    prelude::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{List, ListItem},
};
use speki_backend::{cache::CardCache, filter::FilterUtil};
use tui_textarea::TextArea;

use crate::{create_field, utils::StatefulList};

pub trait FieldsConstructible: Sized {
    fn from_fields(fields: &[Field]) -> Self;
    fn as_fields() -> Fields<'static>;
}

pub struct Fields<'a>(pub Vec<Field<'a>>);

fn parse_value(fields: &[Field], name: &str) -> Box<dyn Any> {
    fields
        .iter()
        .find(|field| field.name == name)
        .unwrap()
        .parse_value()
        .unwrap()
}

impl FieldsConstructible for FilterUtil {
    fn from_fields(fields: &[Field]) -> Self {
        FilterUtil {
            contains: *parse_value(fields, "contains").downcast().unwrap(),
            tags: *parse_value(fields, "tags").downcast().unwrap(),
            suspended: *parse_value(fields, "suspended").downcast().unwrap(),
            resolved: *parse_value(fields, "resolved").downcast().unwrap(),
            pending: *parse_value(fields, "pending").downcast().unwrap(),
            finished: *parse_value(fields, "finished").downcast().unwrap(),
            max_strength: *parse_value(fields, "max_strength").downcast().unwrap(),
            min_strength: *parse_value(fields, "min_strength").downcast().unwrap(),
            max_stability: *parse_value(fields, "max_stability").downcast().unwrap(),
            min_stability: *parse_value(fields, "min_stability").downcast().unwrap(),
            max_recall_rate: *parse_value(fields, "max_recall_rate").downcast().unwrap(),
            min_recall_rate: *parse_value(fields, "min_recall_rate").downcast().unwrap(),
            ..Default::default()
        }
    }

    fn as_fields() -> Fields<'static> {
        Fields(vec![
            create_field!("contains", Option<String>),
            create_field!("tags", Vec<String>),
            create_field!("suspended", Option<bool>),
            create_field!("resolved", Option<bool>),
            create_field!("pending", Option<bool>),
            create_field!("finished", Option<bool>),
            create_field!("max_strength", DurationDays),
            create_field!("min_strength", DurationDays),
            create_field!("max_stability", DurationDays),
            create_field!("min_stability", DurationDays),
            create_field!("max_recall_rate", Option<f32>),
            create_field!("min_recall_rate", Option<f32>),
        ])
    }
}

type Evaluator = Box<dyn Fn(&str) -> Result<Box<dyn Any>, String>>;

pub struct Field<'a> {
    name: String,
    eval: Evaluator,
    value: TextArea<'a>,
}

impl Field<'_> {
    fn is_input_valid(&self) -> bool {
        (self.eval)(self.value_as_str().as_str()).is_ok()
    }

    fn parse_value(&self) -> Result<Box<dyn Any>, String> {
        (self.eval)(self.value_as_str().as_str())
    }

    fn value_as_str(&self) -> String {
        self.value.clone().into_lines().join("\n")
    }
}

pub struct InputTable<'a, T: FieldsConstructible> {
    inner: StatefulList<Field<'a>>,
    area: Rect,
    _marker: PhantomData<T>, // Indicates association with the FieldConvertible type
}

impl<'a, T: FieldsConstructible> InputTable<'a, T> {
    pub fn new() -> Self {
        let fields = T::as_fields();

        Self {
            inner: StatefulList::with_items(fields.0),
            area: Rect::default(),
            _marker: PhantomData,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.inner.items.iter().all(|field| field.is_input_valid())
    }

    pub fn extract_type(&self) -> T {
        T::from_fields(&self.inner.items)
    }
}

impl<T: FieldsConstructible> Widget for InputTable<'_, T> {
    type AppData = CardCache;

    fn keyhandler(&mut self, _app_data: &mut Self::AppData, key: crossterm::event::KeyEvent) {
        if key.code == KeyCode::Up {
            self.inner.previous();
        } else if key.code == KeyCode::Down {
            self.inner.next();
        } else if let Some(field) = self.inner.selected_mut() {
            field.value.input(key);
        }
    }

    fn render(
        &mut self,
        f: &mut ratatui::Frame,
        _cache: &mut Self::AppData,
        area: ratatui::prelude::Rect,
    ) {
        let items: Vec<ListItem> = self
            .inner
            .items
            .iter_mut()
            .map(|field| {
                let accepted = field.is_input_valid();
                let x = format!("{}: {}", field.name.as_str(), field.value_as_str().as_str());
                let lines = vec![Line::from(x)];
                let style = if accepted {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                };
                ListItem::new(lines).style(style)
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");

        // We can now render the item list
        let mut state = self.inner.state.clone();
        f.render_stateful_widget(items, area, &mut state);
    }

    fn area(&self) -> ratatui::prelude::Rect {
        self.area
    }

    fn set_area(&mut self, area: ratatui::prelude::Rect) {
        self.area = area;
    }
}

pub mod r#macro {

    #[macro_export]
    macro_rules! create_field {
        // Special case for a Duration represented in days as f32
        ($name:expr, DurationDays) => {{
            Field {
                name: $name.to_string(),
                eval: Box::new(|input: &str| -> Result<Box<dyn Any>, String> {
                    if input.trim().is_empty() {
                        Ok(Box::new(Option::<std::time::Duration>::None))
                    } else {
                        Ok(input
                            .trim()
                            .parse::<f64>()
                            .map_err(|e| e.to_string())
                            .and_then(|days| {
                                // Convert days to seconds, then to Duration
                                let seconds = days * 86400.0;
                                let duration =
                                    std::time::Duration::from_secs_f64((seconds as f64) as f64);
                                Ok(Box::new(Some(duration)))
                            })?)
                    }
                }),
                value: TextArea::default(),
            }
        }};

        ($name:expr, Option<$type:ty>) => {{
            Field {
                name: $name.to_string(),
                eval: Box::new(|input: &str| -> Result<Box<dyn Any>, String> {
                    if input.trim().is_empty() {
                        Ok(Box::new(None::<$type>))
                    } else {
                        Ok(input
                            .trim()
                            .parse::<$type>()
                            .map(Some)
                            .map(Box::new)
                            .map_err(|e| e.to_string())?)
                    }
                }),
                value: TextArea::default(),
            }
        }};

        // Match when a Vec type is used (e.g., Vec<f64>)
        ($name:expr, Vec<$type:ty>) => {{
            Field {
                name: $name.to_string(),
                eval: Box::new(|input: &str| -> Result<Box<dyn Any>, String> {
                    let result: Result<Vec<$type>, _> = input
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::parse)
                        .collect();
                    Ok(result.map(Box::new).map_err(|e| e.to_string())?)
                }),
                value: TextArea::default(),
            }
        }};

        // Match when a normal type is used (e.g., f64, i32, etc.)
        ($name:expr, $type:ty) => {{
            Field {
                name: $name.to_string(),
                eval: Box::new(|input: &str| -> Result<Box<dyn Any>, String> {
                    input
                        .trim()
                        .parse::<$type>()
                        .map(Box::new)
                        .map_err(|e| e.to_string())
                }),
                value: TextArea::default(),
            }
        }};
    }
}
