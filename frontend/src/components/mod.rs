use yew::{Callback, Properties};

pub mod cube;
pub mod algorithm;
pub mod steps;
pub mod min_max;
pub mod variants;
pub mod niss;

#[derive(Properties, Clone, PartialEq)]
pub struct DefaultProps<T: Properties + Clone + PartialEq> {
    pub data: T,
    pub on_changed: Callback<T>,
}