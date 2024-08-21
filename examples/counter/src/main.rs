use fiber::state::StateCtx;
use fiber::AppBuilder;
use floem::reactive::use_context;

fn main() {
    AppBuilder::from_path("./examples/counter/fiber")
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
fn decrease_counter() {
    let state = use_context::<StateCtx>().unwrap();

    let val = state.get_int("counter").map(|s| s.get_untracked()).unwrap_or_default();

    state.set_int("counter".to_string(), val - 1);
}
