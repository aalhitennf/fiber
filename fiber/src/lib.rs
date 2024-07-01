#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::cast_precision_loss
)]

pub mod runtime;
pub mod theme;

use floem::peniko::Color;
use floem::reactive::{use_context, RwSignal};
use floem::style::Style;
use floem::unit::{PxPct, PxPctAuto};
use floem::views::{button, container, h_stack_from_iter, text, text_input, v_stack_from_iter, Decorators};
use floem::{AnyView, IntoView};
use fml::{Attribute, AttributeValue, Element, ElementKind, Node};
use theme::{parser, Theme};

pub mod observer;

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
