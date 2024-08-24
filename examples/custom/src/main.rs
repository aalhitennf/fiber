use fiber::{App, StateCtx};

fn main() {
    App::from_path("./examples/custom")
        .enable_logging()
        .handlers(vec![increase_counter(), decrease_counter()])
        .run();
}

#[fiber::task]
fn increase_counter(state: StateCtx) {
    state.update::<i64>("counter", |val| *val += 1);
}

#[fiber::task]
fn decrease_counter(state: StateCtx) {
    state.update::<i64>("counter", |val| *val -= 1);
}
