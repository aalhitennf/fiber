use fiber::{AppBuilder, StateCtx};

fn main() {
    AppBuilder::from_path("./examples/counter/fiber")
        .handlers(vec![increase_counter(), decrease_counter()])
        .run();
}

#[fiber::func]
fn increase_counter(state: StateCtx) {
    let val = state.get_int("counter").map(|s| s.get_untracked());

    if let Some(val) = val {
        state.set_int("counter".to_string(), val + 1);
    }
}

#[fiber::func]
fn decrease_counter(state: StateCtx) {
    let val = state.get_int("counter").map(|s| s.get_untracked());

    if let Some(val) = val {
        state.set_int("counter".to_string(), val - 1);
    }
}
