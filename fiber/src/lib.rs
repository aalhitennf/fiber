#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::cast_precision_loss
)]

mod app;
mod builders;
mod runtime;
mod signal;
pub mod state;
mod theme;

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use floem::ext_event::create_signal_from_channel;
use floem::reactive::{use_context, Scope};

mod observer;

// Export macros
pub use fiber_macro::task;

// Export common structs
pub use app::App;
pub use state::StateCtx;

pub fn run_task<T>(task: AsyncTask<T>)
where
    T: Send + Clone + 'static,
{
    let task_wrap = async move {
        let value = task.future.await;
        if let Err(e) = task.sender.send(value) {
            log::error!("AsyncTask failed to return value: {e}");
        }
    };

    tokio::task::spawn(task_wrap);
}

pub struct AsyncTask<T>
where
    T: Send + Clone + 'static,
{
    pub(crate) sender: crossbeam_channel::Sender<T>,
    pub(crate) future: Pin<Box<dyn Future<Output = T> + Send>>,
}

impl<T> AsyncTask<T>
where
    T: Send + Clone + Debug + 'static,
{
    // TODO This most likely leaks memory every time called
    /// # Panics
    /// Panics if `StateCtx` not set (never)
    pub fn new<F, U>(future: F, callback: U) -> Self
    where
        F: Future<Output = T> + 'static + Send,
        U: Fn(&StateCtx, T) + 'static,
    {
        let scope = Scope::new();

        let (sender, receiver) = crossbeam_channel::unbounded();

        let sig = create_signal_from_channel(receiver);

        scope.create_effect(move |_| {
            if let Some(value) = sig.get() {
                let state = use_context::<StateCtx>().unwrap();

                callback(&state, value);
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
