use floem::keyboard::{Key, Modifiers, NamedKey};
use floem::peniko::Color;
use floem::reactive::{use_context, RwSignal};
use floem::style::Style;
use floem::unit::{PxPct, PxPctAuto};
use floem::views::{button, container, h_stack_from_iter, label, text, text_input, v_stack_from_iter, Decorators};
use floem::{AnyView, IntoView, View};
use fml::{Attribute, AttributeValue, Element, ElementKind, Node, VariableType};

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
    view.on_key_up(Key::Named(NamedKey::F11), Modifiers::empty(), move |_| id.inspect())
}

fn node(node: &Node) -> AnyView {
    match node {
        Node::Element(e) => element_to_anyview(e),
        Node::Text(t) => text(t.content).into_any(),
    }
}

fn element_to_anyview(elem: &Element) -> AnyView {
    let elem_value_key = format!("value_{}", elem.id);

    let attrs = elem
        .attributes
        .iter()
        .fold(Style::new(), |s, attr| attr_to_style(attr, s));

    let value_var_name = elem.get_attr("value").map(|a| a.to_string());

    if value_var_name.is_some() {
        log::info!("value_var_name = {value_var_name:?}");
    }

    match &elem.kind {
        ElementKind::Root => {
            let children = elem.children.iter().map(node).collect::<Vec<_>>();
            container(children).style(Style::size_full).css(&["root"]).into_any()
        }

        ElementKind::Box => {
            let children = elem.children.iter().map(node).collect::<Vec<_>>();
            container(children).css(&["box"]).into_any()
        }

        ElementKind::Label => {
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

                match var.kind {
                    VariableType::String => {
                        let value = state
                            .get_string(name)
                            .unwrap_or_else(|| RwSignal::new("String {} not in state".to_string()))
                            .get()
                            .to_string();

                        content.update(|c| *c = c.replace(var.full_match, &value));
                    }
                    VariableType::Integer => {
                        let value = state
                            .get_int(name)
                            .unwrap_or_else(|| RwSignal::new(0))
                            .get()
                            .to_string();

                        content.update(|c| *c = c.replace(var.full_match, &value));
                    }
                    VariableType::Float => {
                        let value = state
                            .get_float(name)
                            .unwrap_or_else(|| RwSignal::new(0.0))
                            .get()
                            .to_string();

                        content.update(|c| *c = c.replace(var.full_match, &value));
                    }
                    VariableType::Unknown => {
                        log::warn!("Unsupported inline variable type {:?}", var.kind);
                    }
                }
            }

            drop(state);

            label(move || content.get()).into_any()
        }

        ElementKind::Button => {
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
                        let state = state.clone();
                        onclick_fn(state);
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

            button.css(&["button"])
        }

        ElementKind::HStack => {
            let children = elem.children.iter().map(node);
            h_stack_from_iter(children).css(&["hstack"]).into_any()
        }

        ElementKind::VStack => {
            let children = elem.children.iter().map(node);
            v_stack_from_iter(children).css(&["vstack"]).into_any()
        }

        ElementKind::Input => {
            let state = use_context::<StateCtx>().unwrap();

            let buffer = state
                .get_string(&value_var_name.unwrap_or_else(|| elem_value_key.clone()))
                .unwrap_or_else(|| RwSignal::new(format!("Var {elem_value_key} not found")));

            text_input(buffer).into_any()
        }
        _ => text("other").into_any(),
    }
    .style(move |s| s.apply(attrs.clone()))
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
