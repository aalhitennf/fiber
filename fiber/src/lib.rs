#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::cast_precision_loss
)]

mod app;
mod builders;
mod runtime;
pub mod state;
pub mod task;
mod theme;

mod observer;

// Export macros
pub use fiber_macro::task;

// Export common structs
pub use app::App;
pub use state::StateCtx;
