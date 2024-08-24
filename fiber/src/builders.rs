use std::any::Any;

use floem::keyboard::{Key, Modifiers, NamedKey};
use floem::peniko::Color;
use floem::reactive::{use_context, RwSignal};
use floem::style::Style;
use floem::unit::{PxPct, PxPctAuto};
use floem::views::{
    button, container, dyn_view, empty, h_stack_from_iter, label, stack_from_iter, text,
    text_input, v_stack_from_iter, Decorators,
};
use floem::{AnyView, IntoView, View};
use fml::{Attribute, AttributeValue, Element, ElementKind, Node, VariableName, VariableType};

use crate::observer::SourceObserver;
use crate::state::Viewable;
use crate::theme::parser::{parse_color, parse_px_pct, parse_pxpctauto};
use crate::theme::{StyleCss, Theme};
use crate::StateCtx;

pub(crate) fn source(source: &str) -> impl View {
    let start = std::time::SystemTime::now();

    let view = match fml::parse(source) {
        Ok(root_node) => node(&root_node),
        Err(e) => text(e).into_any(),
    }
    .style(Style::size_full)
    .keyboard_navigatable();

    let end = start.elapsed().unwrap();
    log::info!("View built in {}ms", end.as_millis());

    let id = view.id();
    view.on_key_up(Key::Named(NamedKey::F11), Modifiers::empty(), move |_| {
        id.inspect()
    })
}

fn node(node: &Node) -> AnyView {
    match node {
        Node::Element(e) => element_to_anyview(e),
        Node::Text(t) => text(t.content).into_any(),
    }
}

// TODO Too many lines
fn element_to_anyview(elem: &Element) -> AnyView {
    let style_attrs = elem
        .attributes
        .iter()
        .fold(Style::new(), |s, attr| attr_to_style(attr, s));

    match &elem.kind {
        ElementKind::Root => build_root(elem),
        ElementKind::Box => build_box(elem),
        ElementKind::Label => build_label(elem),
        ElementKind::Button => build_button(elem),
        ElementKind::HStack => build_hstack(elem),
        ElementKind::VStack => build_vstack(elem),
        ElementKind::Input => build_input(elem),
        ElementKind::List => build_list(elem),
        ElementKind::Custom(name) => build_custom(name),
        other => text(format!("Element '{other:?}' not implemented yet")).into_any(),
    }
    .style(move |s| s.apply(style_attrs.clone()))
}

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

fn build_root(elem: &Element) -> AnyView {
    let children = elem.children.clone().iter().map(node).collect::<Vec<_>>();
    container(children)
        .style(Style::size_full)
        .css("root")
        .into_any()
}

fn build_box(elem: &Element) -> AnyView {
    let children = elem.children.clone().iter().map(node).collect::<Vec<_>>();
    container(children).css("box").into_any()
}

fn build_label(elem: &Element) -> AnyView {
    if elem.children.is_empty() {
        return text("").into_any();
    }

    if elem.children.iter().any(|e| matches!(e, Node::Element(_))) {
        return text("Label can have only one text element as child").into_any();
    }

    let Some(Node::Text(t)) = elem.children.first() else {
        return text("Label can have only one text element as child").into_any();
    };

    let state = use_context::<StateCtx>().unwrap();

    let content = RwSignal::new(t.content.to_string());

    for var in &t.variable_refs {
        let Some((_, name)) = var.name().split_once(':') else {
            log::error!("Invalid variable {:?}", var);
            continue;
        };

        // TODO Ugly maps. Maybe create state function with default as arg or restrict T to impl Default
        match var.kind {
            VariableType::String => {
                let value = state
                    .get::<String>(name)
                    .map(move |s| {
                        s.with(|v| v.downcast_ref::<String>().cloned().unwrap_or_default())
                    })
                    .unwrap_or_default()
                    .to_string();

                content.update(|c| *c = c.replace(var.full_match, &value));
            }
            VariableType::Integer => {
                let value = state
                    .get::<i64>(name)
                    .map(move |s| s.with(|v| v.downcast_ref::<i64>().copied().unwrap_or_default())) // TODO Ugly
                    .unwrap_or_default()
                    .to_string();

                content.update(|c| *c = c.replace(var.full_match, &value));
            }
            VariableType::Float => {
                let value = state
                    .get::<f64>(name)
                    .map(move |s| s.with(|v| v.downcast_ref::<f64>().copied().unwrap_or_default())) // TODO Ugly
                    .unwrap_or_default()
                    .to_string();

                content.update(|c| *c = c.replace(var.full_match, &value));
            }
            VariableType::Unknown => {
                log::warn!("Unsupported inline variable type {:?}", var.kind);
            }
        }
    }
    label(move || content.get()).into_any()
}

fn build_button(elem: &Element) -> AnyView {
    let mut button = if let Some(Node::Text(t)) = elem.children.first() {
        let val = t.content.to_string();
        button(move || val.clone()).into_any()
    } else {
        let id = elem.id;
        button(move || format!("Button {id}")).into_any()
    };

    if let Some(value) = elem.get_attr("onclick") {
        let state = use_context::<StateCtx>().unwrap();
        let f = state.get_fn(&value.to_string());

        if let Some(onclick_fn) = f {
            button = button.on_click_cont(move |_| {
                onclick_fn();
            });
        } else {
            let fn_name = value.to_string();
            button = button.on_click_stop(move |_| {
                log::warn!("Button onclick fn '{fn_name}' not set");
            });
        }
    } else {
        log::debug!("Button without onclick attribute");
    }

    button.css("button")
}

fn build_hstack(elem: &Element) -> AnyView {
    let children = elem.children.iter().map(node);
    h_stack_from_iter(children).css("hstack").into_any()
}

fn build_vstack(elem: &Element) -> AnyView {
    let children = elem.children.iter().map(node);
    v_stack_from_iter(children).css("vstack").into_any()
}

fn build_input(elem: &Element) -> AnyView {
    let name = elem
        .get_attr("value")
        .map_or_else(|| format!("value_{}", elem.id), |attr| attr.to_string());

    let state = use_context::<StateCtx>().unwrap();

    // TODO Probably very terrible
    if let Some(sig) = state.get::<String>(&name) {
        let s = (&sig as &dyn Any)
            .downcast_ref::<RwSignal<String>>()
            .unwrap();
        text_input(*s).into_any()
    } else {
        text_input(RwSignal::new(format!("Var {name} not found"))).into_any()
    }
}

fn build_list(elem: &Element) -> AnyView {
    let Some(attr) = elem.attributes.iter().find(|a| a.name == "items") else {
        log::warn!("List has no attribute 'items'");
        return container(empty()).into_any();
    };

    let Attribute {
        name: _,
        value:
            AttributeValue::Variable {
                name:
                    VariableName {
                        kind: VariableType::Unknown,
                        name: varname,
                    },
                ..
            },
    } = attr
    else {
        log::warn!("List attribute 'items' must be variable");
        return container(empty()).into_any();
    };

    let state = use_context::<StateCtx>().unwrap();

    // let Some(items_sig) = state.get_view(varname) else {
    //     log::warn!("State has no variable '{varname}'");
    //     return container(empty()).into_any();
    // };

    let Some(items_sig) = state.get::<Vec<Box<dyn Viewable>>>(varname) else {
        log::warn!("State has no variable '{varname}'");
        return container(empty()).into_any();
    };

    let style_attrs = elem
        .attributes
        .iter()
        .fold(Style::new(), |s, attr| attr_to_style(attr, s));

    dyn_view(move || {
        let style_attrs = style_attrs.clone();
        let items = items_sig.with(|s| {
            if let Some(v) = (*s).downcast_ref::<Vec<Box<dyn Viewable>>>() {
                v.iter().map(|f| f.into_anyview()).collect::<Vec<_>>()
            } else {
                log::error!("Cast to Viewable failed in build_list");
                Vec::new()
            }
        });
        stack_from_iter(items).style(move |s| s.apply(style_attrs.clone()))
    })
    .into_any()
}

fn build_custom(name: &str) -> AnyView {
    // TODO Not good thing
    let source_map = use_context::<RwSignal<SourceObserver>>().unwrap();
    if let Some(source) = source_map.get().component(name) {
        match fml::parse(source) {
            Ok(n) => node(&n),
            Err(e) => text(e.to_string()).into_any(),
        }
    } else {
        text(format!("Component not found: {name}")).into_any()
    }
}

fn attr_value_to_px_pct(value: AttributeValue) -> PxPct {
    match value {
        AttributeValue::String { value, .. } => parse_px_pct(value).unwrap_or(PxPct::Px(0.0)),
        AttributeValue::Float { value, .. } => PxPct::Px(value),
        AttributeValue::Integer { value, .. } => PxPct::Px(value as f64),
        AttributeValue::Variable { .. } => todo!("Get value from runtime"),
    }
}

fn attr_value_to_px_pct_auto(value: AttributeValue) -> PxPctAuto {
    match value {
        AttributeValue::String { value, .. } => {
            if value == "auto" {
                PxPctAuto::Auto
            } else {
                parse_pxpctauto(value).unwrap_or(PxPctAuto::Auto)
            }
        }
        AttributeValue::Float { value, .. } => PxPctAuto::Px(value),
        AttributeValue::Integer { value, .. } => PxPctAuto::Px(value as f64),
        AttributeValue::Variable { .. } => todo!("Get value from runtime"),
    }
}

fn attr_value_to_color(value: AttributeValue) -> Color {
    if let AttributeValue::String { value, .. } = value {
        parse_color(value).unwrap_or(Color::WHITE)
    } else {
        Color::WHITE
    }
}
