#![allow(clippy::module_name_repetitions)]

mod style;
mod theme;

pub use style::{parser, ColorVariant, StyleCss};
pub use theme::{theme_provider, Theme, ThemeOptions};
