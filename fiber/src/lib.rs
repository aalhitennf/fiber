#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::cast_precision_loss
)]

mod app;
mod builders;
mod observer;
pub mod state;
pub mod task;
mod theme;

// Export macros
pub use fiber_macro::task;

// Export common structs
pub use app::App;
pub use state::StateCtx;
pub use theme::StyleCss;
