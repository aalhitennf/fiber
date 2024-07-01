use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crossbeam_channel::{Receiver, Sender};
use floem::ext_event::create_signal_from_channel;
use floem::reactive::{create_effect, provide_context, RwSignal};
use floem::style::Style;
use floem::views::{container, Container, Decorators};
use floem::View;

use crate::observer::FileObserver;
use crate::theme::parser::{Selector, StyleBlock, StyleParser};
use crate::theme::StyleCss;

#[derive(Clone)]
pub struct Theme {
    path: PathBuf,
    channel: (Sender<()>, Receiver<()>),
    map: HashMap<String, Style>,
    _observer: Rc<FileObserver>,
}

impl Theme {
    fn read_styles(&self) -> Vec<StyleBlock> {
        let files = std::fs::read_dir(&self.path)
            .expect("Cannot read path {path}")
            .filter_map(Result::ok)
            .filter_map(|e| {
                e.path()
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("css"))
                    .then_some(e.path())
            });

        let combined = files.flat_map(std::fs::read_to_string).fold(String::new(), |mut s, c| {
            s.push_str(&c);
            s
        });

        StyleParser::blocks(&combined)
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn reload(&mut self) {
        #[cfg(debug_assertions)]
        let now = std::time::SystemTime::now();

        self.map.clear();

        // Parse and convert
        for block in self.read_styles() {
            let style: Style = block.clone().into();

            for selector in &block.selectors {
                let new_style = style.clone();

                let to_modify = self.map.remove(&selector.class).unwrap_or_default();

                let to_insert = match selector.selector {
                    Some(Selector::Active) => to_modify.active(|_| new_style),
                    Some(Selector::Disabled) => to_modify.disabled(|_| new_style),
                    Some(Selector::Focus) => to_modify.focus(|_| new_style),
                    Some(Selector::Hover) => to_modify.hover(|_| new_style),
                    None => to_modify.apply(new_style),
                };

                self.map.insert(selector.class.clone(), to_insert);
            }
        }

        #[cfg(debug_assertions)]
        {
            let elaps = std::time::SystemTime::now()
                .duration_since(now)
                .expect("Time is going backwards");

            log::info!("Styles parsed in {}ms", elaps.as_millis());
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if `path` does not exist or the user does not have
    /// permission to read it.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref().to_path_buf();
        let channel = crossbeam_channel::unbounded();
        let observer = FileObserver::new(&path, channel.0.clone(), true)?;

        let mut theme = Theme {
            path,
            _observer: Rc::new(observer),
            channel,
            map: HashMap::default(),
        };

        theme.reload();

        Ok(theme)
    }

    /// # Errors
    ///
    /// Will return `Err` if `path` does not exist or the user does not have
    /// permission to read it.
    pub fn change_path<P: AsRef<Path>>(&mut self, new_path: P) -> Result<(), Box<dyn std::error::Error>> {
        let new = Self::from_path(new_path)?;
        let _ = std::mem::replace(self, new);
        Ok(())
    }

    #[must_use]
    pub fn get_styles(&self) -> Iter<String, Style> {
        self.map.iter()
    }

    #[must_use]
    pub fn get_style(&self, key: &str) -> Option<&Style> {
        self.map.get(key)
    }

    #[must_use]
    pub fn apply_classes(&self, s: Style, keys: &[&str]) -> Style {
        keys.iter()
            .fold(s, |s, key| s.apply_opt(self.get_style(key), |s, t| s.apply(t.clone())))
    }
}

pub struct ThemeOptions {
    path: PathBuf,
    overrides: Option<PathBuf>,
}

impl ThemeOptions {
    #[must_use]
    pub fn with_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            overrides: None,
        }
    }

    #[must_use]
    pub fn overrides<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.overrides = Some(path.as_ref().to_path_buf());
        self
    }
}

/// Wraps given view in "body" class and provides `Theme` as context
/// # Panics
///
/// Panics if `path` doesn't point to a existing folder.
pub fn theme_provider<V, F>(child: F, options: ThemeOptions) -> Container
where
    F: Fn() -> V,
    V: View + 'static,
{
    let theme = Theme::from_path(options.path).expect("Invalid theme path");
    let observer_event = create_signal_from_channel(theme.channel.1.clone());

    let theme = RwSignal::new(theme);

    create_effect(move |_| {
        if let Some(()) = observer_event.get() {
            theme.update(Theme::reload);
            log::info!("Css reloaded");
        }
    });

    provide_context(theme);

    container(child()).css(&["body"]).debug_name("Body")
}
