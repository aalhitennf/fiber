use fiber::{App, StateCtx};

#[tokio::main]
async fn main() {
    App::from_path("./examples/async_tokio")
        .handlers(vec![increase_delayed(), exit()])
        .run();
}

fn increase_delayed_callback(state: StateCtx, value: i64) {
    state.update_int("counter", |val| *val += value);
}

#[fiber::async_func(increase_delayed_callback)]
async fn increase_delayed() -> i64 {
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    1
}

#[fiber::func]
fn exit() {
    std::process::exit(0);
}
