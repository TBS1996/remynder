mod choose_category;
pub use choose_category::*;

mod add_card;
pub use add_card::*;

#[derive(Debug)]
pub enum PopUpState<T: std::fmt::Debug> {
    Exit,
    Continue,
    Resolve(T),
}

impl<T: std::fmt::Debug> PopUpState<T> {
    pub fn value(&self) -> Option<&T> {
        match self {
            PopUpState::Exit => None,
            PopUpState::Continue => None,
            PopUpState::Resolve(t) => Some(t),
        }
    }
}

impl<T: std::fmt::Debug> Default for PopUpState<T> {
    fn default() -> Self {
        Self::Continue
    }
}
