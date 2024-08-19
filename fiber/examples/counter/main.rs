use std::sync::Arc;

use fiber::state::State;
use fiber::AppBuilder;
use parking_lot::RwLock;

#[fiber::main]
fn main() {
    AppBuilder::from_path("fiber/examples/counter")
        .handlers(vec![increase_counter(), decrease_counter()])
        .run();
}

#[fiber::func]
fn increase_counter(state: Arc<RwLock<State>>) {
    let val = state
        .read()
        .get_int("counter")
        .map(|s| s.get_untracked())
        .unwrap_or_default();

    state.write().set_int("counter".to_string(), val + 1);
}

#[fiber::func]
fn decrease_counter(state: Arc<RwLock<State>>) {
    let val = state
        .read()
        .get_int("counter")
        .map(|s| s.get_untracked())
        .unwrap_or_default();

    // match state.try_write_for(Duration::from_secs(1)) {
    //     Some(mut lock) => {
    state.write().set_int("counter".to_string(), val - 1);
    //     }
    //     None => {
    //         log::error!("Failed to acquire state lock");
    //     }
    // }
}
