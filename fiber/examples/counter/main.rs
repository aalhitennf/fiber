use std::sync::Arc;

use fiber::state::{State, StateCtx};
use fiber::AppBuilder;
use floem::reactive::use_context;
use parking_lot::RwLock;

fn main() {
    AppBuilder::from_path("fiber/examples/counter")
        .handlers(vec![increase_counter(), decrease_counter()])
        .run();
}

#[fiber::func]
fn increase_counter() {
    let state = use_context::<StateCtx>().unwrap();

    let val = state.get_int("counter").map(|s| s.get_untracked()).unwrap_or_default();

    state.set_int("counter".to_string(), val + 1);
}

#[fiber::func]
fn decrease_counter(state: Arc<RwLock<State>>) {
    let state = use_context::<StateCtx>().unwrap();

    let val = state.get_int("counter").map(|s| s.get_untracked()).unwrap_or_default();

    state.set_int("counter".to_string(), val - 1);
}
