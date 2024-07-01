use std::path::{Path, PathBuf};

use crossbeam_channel::Receiver;
use floem::{
    ext_event::create_signal_from_channel,
    keyboard::{Key, Modifiers, NamedKey},
    reactive::{create_effect, RwSignal},
    style::Style,
    views::{dyn_view, text, Decorators},
    IntoView, View,
};

use fiber::{
    c_node_to_view,
    observer::FileObserver,
    theme::{theme_provider, ThemeOptions},
};
use fml::parse;
use log::LevelFilter;

fn app_view(path: impl AsRef<Path> + 'static, receiver: Receiver<()>) -> impl View {
    let sig = create_signal_from_channel(receiver);

    let source = RwSignal::new(std::fs::read_to_string(&path).expect("Cannot read source"));

    create_effect(move |_| {
        if let Some(_) = sig.get() {
            log::info!("File changed");

            match std::fs::read_to_string(&path) {
                Ok(new_source) => source.set(new_source),
                Err(e) => source.set(e.to_string()),
            }
        }
    });

    let view = dyn_view(move || {
        source.with(|s| match parse(&s) {
            Ok(node) => c_node_to_view(&node),
            Err(e) => text(e).into_any(),
        })
    })
    .style(Style::size_full)
    .keyboard_navigatable();

    let id = view.id();
    view.on_key_up(Key::Named(NamedKey::F11), Modifiers::empty(), move |_| id.inspect())
}

fn main() {
    env_logger::builder()
        .filter_module("wgpu_hal", LevelFilter::Error)
        .filter_module("wgpu_core", LevelFilter::Error)
        .filter_module("naga", LevelFilter::Error)
        .filter_module("floem_cosmic_text", LevelFilter::Error)
        .filter_level(LevelFilter::Info)
        .init();

    log::info!("Logging OK");

    let path = PathBuf::from("fiber/examples/basic.fml")
        .canonicalize()
        .expect("Invalid path");

    if !path.exists() {
        panic!("Invalid path");
    }

    let (sender, receiver) = crossbeam_channel::unbounded();

    let observer = RwSignal::new(FileObserver::new(&path, sender.clone(), false));

    let theme_provider = theme_provider(
        move || app_view(path.clone(), receiver.clone()),
        ThemeOptions::with_path("fiber/examples/basic.css"),
    );

    floem::launch(|| theme_provider);
}