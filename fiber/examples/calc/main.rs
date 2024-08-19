use fiber::AppBuilder;

fn main() {
    AppBuilder::from_path("./fiber/examples/calc").run();
    // fiber::create_app("./fiber/examples/calc", true);
}
