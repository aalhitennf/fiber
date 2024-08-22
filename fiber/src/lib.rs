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

use std::fmt::Debug;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};

use floem::ext_event::create_signal_from_channel;
use floem::reactive::{create_effect, provide_context, untrack, use_context, RwSignal, Scope};
use floem::views::{dyn_view, Decorators};
use floem::IntoView;
use log::LevelFilter;
use runtime::Runtime;
use state::{FnPointer, State};
use theme::{theme_provider, StyleCss, Theme, ThemeOptions};

mod observer;

// Export macros
pub use fiber_macro::{async_func, func};

// Export common structs
pub use state::StateCtx;

pub struct App {
    log: bool,
    path: PathBuf,
    state: State,
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        let path = PathBuf::from("./fiber");
        App {
            log: true,
            state: State::new(&path),
            path,
        }
    }

    /// # Panics
    /// Panics if given path doesn't exists
    #[must_use]
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().join("fiber").canonicalize().expect("Invalid path");

        App {
            log: true,
            state: State::new(&path),
            path,
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
    pub fn handlers(self, handlers: Vec<(String, FnPointer)>) -> Self {
        for h in handlers {
            self.state.add_handler(h);
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

        let state = StateCtx::new(self.state);
        let theme = RwSignal::new(Theme::from_path(&self.path).expect("Invalid theme path"));

        provide_context(state);
        provide_context(theme);

        let theme_provider = theme_provider(
            move || {
                // TODO This probably don't need to be dyn_view on release build and could be
                // TODO scoped down to specific views/nodes
                dyn_view(move || builders::source(&include_str!("../../examples/async_tokio/fiber/main.fml")))
                    .css(&["body"])
                    .debug_name("Body")
            },
            ThemeOptions::with_path(self.path.join("styles")),
        );

        floem::launch(|| theme_provider);
    }
}

// pub fn spawn<T: Send + 'static>(f: impl Future<Output = T> + Send + 'static) {
//     tokio::task::spawn(f);
// }

// pub fn task_runner<T, F, U>(task: F, updater: U)
// where
//     T: Send + Clone + 'static,
//     F: Future<Output = T> + Send + 'static,
//     U: FnOnce(StateCtx, T) + Copy + 'static,
// {
//     let (sender, receiver) = crossbeam_channel::unbounded::<T>();
//     let sig = create_signal_from_channel(receiver);

//     create_effect(move |_| {
//         if let Some(value) = sig.get() {
//             let state = use_context::<StateCtx>().unwrap();
//             updater(state, value);
//         }
//     });

//     let task_wrap = async move {
//         let value = task.await;
//         sender.send(value).unwrap();
//     };

//     tokio::spawn(task_wrap);
// }

pub fn run_task<T>(task: AsyncTask<T>)
where
    T: Send + Clone + 'static,
    // F: Future<Output = T> + Send + 'static,
    // U: FnOnce(StateCtx, T) + Copy + 'static,
{
    // create_effect(move |_| {
    //     if let Some(value) = sig.get() {
    //         let state = use_context::<StateCtx>().unwrap();
    //         (task.callback)(state, value);
    //     }
    // });

    let task_wrap = async move {
        let value = task.future.await;
        task.sender.send(value).unwrap();
    };

    tokio::spawn(task_wrap);
}

pub struct AsyncTask<T>
where
    T: Send + Clone + 'static,
{
    pub(crate) sender: crossbeam_channel::Sender<T>,
    pub(crate) future: Pin<Box<dyn Future<Output = T> + Send>>,
}

static COUNT: AtomicUsize = AtomicUsize::new(0);

impl<T> AsyncTask<T>
where
    T: Send + Clone + Debug + 'static,
{
    // TODO This most likely leaks memory every time called
    pub fn new<F, U>(future: F, callback: U) -> Self
    where
        F: Future<Output = T> + 'static + Send,
        U: Fn(StateCtx, T) + 'static,
    {
        let scope = Scope::new();

        let (sender, receiver) = crossbeam_channel::unbounded();

        let sig = create_signal_from_channel(receiver);

        scope.create_effect(move |_| {
            if let Some(value) = sig.get() {
                let state = use_context::<StateCtx>().unwrap();
                COUNT.fetch_add(1, Ordering::Relaxed);
                callback(state, value);
                // TODO Maybe untracking sig would do somethings here?
                // TODO No idea if this is necessary
                scope.dispose();
            }
        });

        AsyncTask {
            sender,
            future: Box::pin(future),
        }
    }
}

// impl<T> Drop for AsyncTask<T> {
//     fn drop(&mut self) {
//         self.future
//     }
// }
