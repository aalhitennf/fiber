#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::cast_precision_loss
)]

pub mod runtime;
pub mod state;
pub mod theme;

use std::path::{Path, PathBuf};

use crossbeam_channel::Receiver;
use floem::ext_event::create_signal_from_channel;
use floem::keyboard::{Key, Modifiers, NamedKey};
use floem::peniko::Color;
use floem::reactive::{create_effect, provide_context, use_context, RwSignal};
use floem::style::Style;
use floem::unit::{PxPct, PxPctAuto};
use floem::views::{button, container, dyn_view, h_stack_from_iter, text, text_input, v_stack_from_iter, Decorators};
use floem::{AnyView, IntoView, View};
use fml::{parse, Attribute, AttributeValue, Element, ElementKind, Node};
use log::LevelFilter;
use runtime::Runtime;
use state::State;
use theme::{parser, theme_provider, StyleCss, Theme, ThemeOptions};

pub mod observer;

fn app_view(path: PathBuf, receiver: Receiver<()>) -> impl View {
    let sig = create_signal_from_channel(receiver);

    let source = RwSignal::new(std::fs::read_to_string(&path).expect("Cannot read source"));

    create_effect(move |_| {
        sig.get();

        match std::fs::read_to_string(&path) {
            Ok(new_source) => source.set(new_source),
            Err(e) => source.set(e.to_string()),
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

pub fn create_app(entrypoint: impl AsRef<Path>, logging: bool) {
    let path = entrypoint.as_ref().canonicalize().expect("Invalid path");

    if logging {
        env_logger::builder()
            .filter_module("wgpu_hal", LevelFilter::Error)
            .filter_module("wgpu_core", LevelFilter::Error)
            .filter_module("naga", LevelFilter::Error)
            .filter_module("floem_cosmic_text", LevelFilter::Error)
            .filter_level(LevelFilter::Info)
            .init();

        log::info!("Logging OK");
    }

    if !path.exists() {
        panic!("Path does not exists");
    }

    let (sender, receiver) = crossbeam_channel::unbounded();
    let runtime = RwSignal::new(Runtime::new(&path, sender).expect("Failed to create Runtime"));
    let state = RwSignal::new(State::new());
    let theme = RwSignal::new(Theme::from_path(&path).expect("Invalid theme path"));

    let runtime_event_sig = create_signal_from_channel(receiver.clone());
    // let theme_event_sig = create_signal_from_channel(receiver);

    provide_context(runtime);
    provide_context(state);
    provide_context(theme);

    let ef_path = path.clone();
    create_effect(move |_| {
        state.track();
        // theme.track();
        runtime_event_sig.with(|_| ());
        // theme_event_sig.with(|_| ());

        match std::fs::read_to_string(ef_path.clone()) {
            Ok(new_source) => runtime.update(|rt| rt.update_source(new_source)),
            Err(e) => runtime.update(|rt| rt.update_source(e.to_string())),
        }

        theme.update(Theme::reload);

        log::info!("Sources reloaded");
    });

    // let root_view =
    //     dyn_view(move || runtime.with(|rt| container(build_view(rt.source())).css(&["body"]).debug_name("Body")));

    let theme_path = path.clone().join("styles");
    let theme_provider = theme_provider(
        move || {
            dyn_view(move || runtime.with(|rt| container(build_view(rt.source())).css(&["body"]).debug_name("Body")))
        },
        ThemeOptions::with_path(theme_path),
    );

    floem::launch(|| theme_provider);
}

fn build_view(source: &str) -> impl View {
    let view = match parse(&source) {
        Ok(node) => c_node_to_view(&node),
        Err(e) => text(e).into_any(),
    }
    .style(Style::size_full)
    .keyboard_navigatable();

    let id = view.id();
    view.on_key_up(Key::Named(NamedKey::F11), Modifiers::empty(), move |_| id.inspect())
}

pub fn c_node_to_view(node: &Node) -> AnyView {
    match node {
        Node::Text(t) => text(t).into_any(),
        Node::Element(elem) => element_to_anyview(elem),
    }
}

fn element_to_anyview(elem: &Element) -> AnyView {
    let children = elem.children.iter().map(c_node_to_view).collect::<Vec<_>>();

    let attrs = {
        elem.attributes
            .iter()
            .fold(Style::new(), |s, attr| attr_to_style(attr, s))
    };

    match &elem.kind {
        ElementKind::Root => container(children).style(Style::size_full).into_any(),
        ElementKind::Box => container(children).into_any(),
        ElementKind::Text => children.into_any(),
        ElementKind::Button => {
            if let Some(Node::Text(t)) = elem.children.first() {
                let val = (*t).to_string();
                button(move || val.clone()).into_any()
            } else {
                button(|| "Button").into_any()
            }
        }
        ElementKind::HStack => h_stack_from_iter(children).into_any(),
        ElementKind::VStack => v_stack_from_iter(children).into_any(),
        ElementKind::Input => {
            let buffer = RwSignal::new(String::new());
            text_input(buffer).into_any()
        }
        _ => text("other").into_any(),
    }
    .style(move |s| s.apply(attrs.clone()))
}

#[inline]
fn attr_to_style<'a>(attr: &'a Attribute<'a>, s: Style) -> Style {
    match attr.name.as_ref() {
        "class" => {
            if let AttributeValue::String { value, .. } = attr.value {
                let theme = use_context::<RwSignal<Theme>>().unwrap();
                let classes = value.split_whitespace().collect::<Vec<_>>();
                theme.get().apply_classes(s, &classes)
            } else {
                s
            }
        }
        "gap" => s.gap(attr_value_to_px_pct(attr.value)),
        "width" => s.width(attr_value_to_px_pct_auto(attr.value)),
        "height" => s.height(attr_value_to_px_pct_auto(attr.value)),
        "margin" => s.margin(attr_value_to_px_pct_auto(attr.value)),
        "padding" => s.padding(attr_value_to_px_pct(attr.value)),
        "color" => s.color(attr_value_to_color(attr.value)),
        _ => s,
    }
}

#[inline]
fn attr_value_to_px_pct(value: AttributeValue) -> PxPct {
    match value {
        AttributeValue::String { value, .. } => parser::parse_px_pct(value).unwrap_or(PxPct::Px(0.0)),
        AttributeValue::Float { value, .. } => PxPct::Px(value),
        AttributeValue::Integer { value, .. } => PxPct::Px(value as f64),
        _ => todo!("Get value from runtime"),
    }
}

#[inline]
fn attr_value_to_px_pct_auto(value: AttributeValue) -> PxPctAuto {
    match value {
        AttributeValue::String { value, .. } => {
            if value == "auto" {
                PxPctAuto::Auto
            } else {
                parser::parse_pxpctauto(value).unwrap_or(PxPctAuto::Auto)
            }
        }
        AttributeValue::Float { value, .. } => PxPctAuto::Px(value),
        AttributeValue::Integer { value, .. } => PxPctAuto::Px(value as f64),
        _ => todo!("Get value from runtime"),
    }
}

#[inline]
fn attr_value_to_color(value: AttributeValue) -> Color {
    if let AttributeValue::String { value, .. } = value {
        parser::parse_color(value).unwrap_or(Color::WHITE)
    } else {
        Color::WHITE
    }
}
