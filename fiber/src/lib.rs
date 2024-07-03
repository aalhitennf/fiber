#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::cast_precision_loss
)]

pub mod runtime;
pub mod state;
pub mod theme;

use std::path::Path;

use floem::ext_event::create_signal_from_channel;
use floem::keyboard::{Key, Modifiers, NamedKey};
use floem::peniko::Color;
use floem::reactive::{create_effect, provide_context, use_context, RwSignal};
use floem::style::Style;
use floem::unit::{PxPct, PxPctAuto};
use floem::views::{
    button, container, dyn_view, empty, h_stack_from_iter, text, text_input, v_stack_from_iter, Decorators,
};
use floem::{AnyView, IntoView, View};
use fml::{parse, Attribute, AttributeValue, Element, ElementKind, Node};
use log::LevelFilter;
use runtime::Runtime;
use state::State;
use theme::{parser, theme_provider, StyleCss, Theme, ThemeOptions};

pub mod observer;

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
    let theme_event_sig = create_signal_from_channel(theme.get_untracked().channel.1);

    provide_context(runtime);
    provide_context(state);
    provide_context(theme);

    create_effect(move |_| {
        if let Some(_) = runtime_event_sig.get() {
            runtime.update(Runtime::update_source);
            log::info!("Sources reloaded");
        }
    });

    create_effect(move |_| {
        if let Some(_) = theme_event_sig.get() {
            theme.update(Theme::reload);
            log::info!("Css reloaded");
        }
    });

    let theme_provider = theme_provider(
        move || {
            dyn_view(move || {
                let source = runtime.with(|rt| rt.source().to_string());
                let state = use_context::<RwSignal<State>>().unwrap();

                let mut view = empty().into_any();
                let view_ref = &mut view;
                state.update(move |s| *view_ref = build_view(&source, s).into_any());
                view
            })
            .css(&["body"])
            .debug_name("Body")
        },
        ThemeOptions::with_path(path.join("styles")),
    );

    floem::launch(|| theme_provider);
}

fn build_view(source: &str, state: &mut State) -> impl View {
    let start = std::time::SystemTime::now();

    let view = match parse(&source) {
        Ok(node) => c_node_to_view(&node, state),
        Err(e) => text(e).into_any(),
    }
    .style(Style::size_full)
    .keyboard_navigatable();

    let end = start.elapsed().unwrap();
    log::info!("View built in {}ms", end.as_millis());

    let id = view.id();
    view.on_key_up(Key::Named(NamedKey::F11), Modifiers::empty(), move |_| id.inspect())
}

pub fn c_node_to_view(node: &Node, state: &mut State) -> AnyView {
    match node {
        Node::Text(t) => text(t).into_any(),
        Node::Element(elem) => element_to_anyview(elem, state),
    }
}

// Crashing because net
fn element_to_anyview(elem: &Element, state: &mut State) -> AnyView {
    let value_key = format!("value_{}", elem.id);

    let children = elem
        .children
        .iter()
        .map(|n| c_node_to_view(n, state))
        .collect::<Vec<_>>();

    let attrs = elem
        .attributes
        .iter()
        .fold(Style::new(), |s, attr| attr_to_style(attr, s));

    if let Some(value) = elem.get_attr("value") {
        state.set_var(value_key.clone(), value.to_string());
    }

    match &elem.kind {
        ElementKind::Root => container(children).style(Style::size_full).into_any(),
        ElementKind::Box => container(children).into_any(),
        ElementKind::Text => children.into_any(),
        ElementKind::Button => {
            let button = if let Some(Node::Text(t)) = elem.children.first() {
                let val = (*t).to_string();
                button(move || val.clone()).into_any()
            } else {
                button(|| "Button").into_any()
            };

            if let Some(value) = elem.get_attr("onclick") {
                match state.get_fn(&value.to_string()) {
                    Some(onclick_fn) => {
                        return button.on_click_cont(move |_| {
                            log::info!("FOUND FN");
                            onclick_fn();
                        });
                    }
                    None => {
                        return button.on_click_stop(|_| {
                            log::warn!("NO FN :(:/");
                        });
                    }
                }
            } else {
                log::error!("NO ONCLICK ATTR");
            }

            button.css(&["button"])
        }
        ElementKind::HStack => h_stack_from_iter(children).into_any(),
        ElementKind::VStack => v_stack_from_iter(children).into_any(),
        ElementKind::Input => {
            let value = state.get_var(&value_key).map(|v| v.to_string()).unwrap_or_default();
            let buffer = RwSignal::new(value);

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
