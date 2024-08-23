use fiber::{App, StateCtx};

fn main() {
    App::from_path("./examples/counter")
        .handlers(vec![increase_counter(), decrease_counter()])
        .run();
}

#[fiber::func]
fn increase_counter(state: StateCtx) {
    state.update::<i64>("counter", |val| *val += 1);
}

#[fiber::func]
fn decrease_counter(state: StateCtx) {
    state.update::<i64>("counter", |val| *val -= 1);
}
