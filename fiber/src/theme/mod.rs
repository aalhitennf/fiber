pub mod parser;
mod theme;

use floem::reactive::{use_context, RwSignal};
use floem::views::Decorators;
use floem::View;

pub use theme::{theme_provider, Theme, ThemeOptions};

#[derive(Clone, Copy, Debug)]
pub enum ColorVariant {
    Normal,
    Success,
    Warn,
    Alert,
    Ghost,
}

pub trait StyleCss: View {
    #[must_use]
    fn css(self, keys: &'static [&'static str]) -> Self;
}

impl<V> StyleCss for V
where
    V: View + 'static,
{
    fn css(self, keys: &'static [&'static str]) -> Self {
        let theme = use_context::<RwSignal<Theme>>().unwrap();
        self.style(move |s| theme.get().apply_classes(s, keys))
    }
}
