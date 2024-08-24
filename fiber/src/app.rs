use std::path::{Path, PathBuf};

use log::LevelFilter;

use crate::state::{FnPointer, State};

pub struct App {
    path: PathBuf,
    state: State,
    handlers: Option<Vec<(String, FnPointer)>>,
}

impl Default for App {
    fn default() -> Self {
        App::new()
    }
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        let path = PathBuf::from("./fiber");
        App {
            state: State::default(),
            path,
            handlers: None,
        }
    }

    #[must_use]
    pub fn enable_logging(self) -> Self {
        env_logger::builder()
            .filter_module("wgpu_hal", LevelFilter::Error)
            .filter_module("wgpu_core", LevelFilter::Error)
            .filter_module("naga", LevelFilter::Error)
            .filter_module("floem_cosmic_text", LevelFilter::Error)
            .filter_level(LevelFilter::Info)
            .init();

        log::info!("Logging enabled");

        self
    }

    /// # Panics
    /// Panics if given path doesn't exists
    #[must_use]
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().join("fiber").canonicalize().expect("Invalid path");

        App {
            state: State::default(),
            path,
            handlers: None,
        }
    }

    #[must_use]
    pub fn handlers(mut self, handlers: Vec<(String, FnPointer)>) -> Self {
        self.handlers = Some(handlers);
        self
    }

    /// # Panics
    /// Panics if creating Runtime fails
    #[cfg(debug_assertions)]
    pub fn run(mut self) {
        use floem::ext_event::create_signal_from_channel;
        use floem::reactive::{create_effect, provide_context, RwSignal};
        use floem::views::{dyn_view, Decorators};
        use floem::IntoView;

        use crate::observer::SourceObserver;
        use crate::theme::{theme_provider, StyleCss, Theme, ThemeOptions};
        use crate::{builders, StateCtx};

        self.state.read_vars(&self.path.join("main.vars"));

        if let Some(handlers) = self.handlers.take() {
            for h in handlers {
                self.state.add_handler(h);
            }
        }

        let (sender, receiver) = crossbeam_channel::unbounded();

        let observer = RwSignal::new(SourceObserver::new(&self.path, sender).expect("Failed to create Runtime"));
        let state = StateCtx::new(self.state);
        let theme = RwSignal::new(Theme::from_path(&self.path).expect("Invalid theme path"));

        let observer_event = create_signal_from_channel(receiver.clone());
        let theme_event = create_signal_from_channel(theme.get_untracked().channel.1);

        provide_context(observer);
        provide_context(state);
        provide_context(theme);

        create_effect(move |_| {
            if observer_event.get().is_some() {
                observer.update(SourceObserver::update);
                log::info!("Sources reloaded");
            }
        });

        create_effect(move |_| {
            if theme_event.get().is_some() {
                theme.update(Theme::reload);
                log::info!("Css reloaded");
            }
        });

        let theme_provider = theme_provider(
            move || {
                dyn_view(move || observer.with(|rt| builders::source(rt.main()).into_any()))
                    .css(&["body"])
                    .debug_name("Body")
            },
            ThemeOptions::with_path(self.path.join("styles")),
        );

        floem::launch(|| theme_provider);
    }

    /// # Panics
    #[cfg(not(debug_assertions))]
    pub fn run(mut self) {
        self.state.read_vars(&self.path.join("main.vars"));

        if let Some(handlers) = self.handlers.take() {
            for h in handlers {
                self.state.add_handler(h);
            }
        }

        let state = StateCtx::new(self.state);
        let theme = RwSignal::new(Theme::from_path(&self.path).expect("Invalid theme path"));

        provide_context(state);
        provide_context(theme);

        let theme_provider = theme_provider(
            move || {
                // TODO This probably don't need to be dyn_view on release build and could be
                // TODO scoped down to specific views/nodes
                dyn_view(move || builders::source(&include_str!("../../examples/stateful/fiber/main.fml")))
                    .css(&["body"])
                    .debug_name("Body")
            },
            ThemeOptions::with_path(self.path.join("styles")),
        );

        floem::launch(|| theme_provider);
    }
}
