#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::cast_precision_loss
)]

pub mod builders;
pub mod runtime;
pub mod signal;
pub mod state;
pub mod theme;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use floem::ext_event::create_signal_from_channel;
use floem::reactive::{create_effect, provide_context, RwSignal};
use floem::views::{dyn_view, Decorators};
use floem::IntoView;
use log::LevelFilter;
use runtime::Runtime;
use state::{FnWrap, State};
use theme::{theme_provider, StyleCss, Theme, ThemeOptions};

mod observer;

// Export macros
pub use fiber_macro::func;

// Export common structs
pub use state::StateCtx;

pub struct AppBuilder {
    log: bool,
    path: PathBuf,
    state: State,
}

impl AppBuilder {
    /// # Panics
    /// Panics if given path doesn't exists
    #[must_use]
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        assert!(path.as_ref().exists());

        AppBuilder {
            log: true,
            path: path.as_ref().to_path_buf(),
            state: State::new(path.as_ref()),
        }
    }

    #[must_use]
    pub fn log(mut self, v: bool) -> Self {
        self.log = v;
        self
    }

    fn set_logging(&self) {
        if self.log {
            env_logger::builder()
                .filter_module("wgpu_hal", LevelFilter::Error)
                .filter_module("wgpu_core", LevelFilter::Error)
                .filter_module("naga", LevelFilter::Error)
                .filter_module("floem_cosmic_text", LevelFilter::Error)
                .filter_level(LevelFilter::Info)
                .init();

            log::info!("Logging OK");
        }
    }

    #[must_use]
    pub fn handlers(self, handlers: Vec<(String, fn(StateCtx))>) -> Self {
        for (name, f) in handlers {
            if self
                .state
                .fns
                .insert(name.replace("_fibr_", ""), FnWrap::from(f))
                .is_some()
            {
                log::warn!("Duplicate fn '{name}'");
            }
        }

        self
    }

    /// # Panics
    /// Panics if creating Runtime fails
    #[cfg(debug_assertions)]
    pub fn run(self) {
        self.set_logging();

        let (sender, receiver) = crossbeam_channel::unbounded();

        let runtime = RwSignal::new(Runtime::new(&self.path, sender).expect("Failed to create Runtime"));
        let state = StateCtx::new(self.state);
        let theme = RwSignal::new(Theme::from_path(&self.path).expect("Invalid theme path"));

        let runtime_event_sig = create_signal_from_channel(receiver.clone());
        let theme_event_sig = create_signal_from_channel(theme.get_untracked().channel.1);

        provide_context(runtime);
        provide_context(state);
        provide_context(theme);

        create_effect(move |_| {
            if runtime_event_sig.get().is_some() {
                runtime.update(Runtime::update_source);
                log::info!("Sources reloaded");
            }
        });

        create_effect(move |_| {
            if theme_event_sig.get().is_some() {
                theme.update(Theme::reload);
                log::info!("Css reloaded");
            }
        });

        let theme_provider = theme_provider(
            move || {
                dyn_view(move || runtime.with(|rt| builders::source(rt.source()).into_any()))
                    .css(&["body"])
                    .debug_name("Body")
            },
            ThemeOptions::with_path(self.path.join("styles")),
        );

        floem::launch(|| theme_provider);
    }

    /// # Panics
    #[cfg(not(debug_assertions))]
    pub fn run(self) {
        self.set_logging();

        let state = Arc::new(self.state);
        let theme = RwSignal::new(Theme::from_path(&self.path).expect("Invalid theme path"));

        provide_context(state);
        provide_context(theme);

        let theme_provider = theme_provider(
            move || {
                // TODO This probably don't need to be dyn_view on release build and could be
                // TODO scoped down to specific views/nodes
                dyn_view(move || build::source(&include_str!("../../examples/counter/fiber/main.fml")))
                    .css(&["body"])
                    .debug_name("Body")
            },
            ThemeOptions::with_path(self.path.join("styles")),
        );

        floem::launch(|| theme_provider);
    }
}
