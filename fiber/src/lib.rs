#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::cast_precision_loss
)]

pub mod runtime;
pub mod signal;
pub mod state;
pub mod theme;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use floem::ext_event::create_signal_from_channel;
use floem::keyboard::{Key, Modifiers, NamedKey};
use floem::peniko::Color;
use floem::reactive::{create_effect, provide_context, use_context, RwSignal};
use floem::style::Style;
use floem::unit::{PxPct, PxPctAuto};
use floem::views::{
    button, container, dyn_container, dyn_view, h_stack_from_iter, label, text, text_input, v_stack_from_iter,
    Decorators, TextEditor,
};
use floem::{AnyView, IntoView, View};
use fml::{parse, Attribute, AttributeValue, Element, ElementKind, Node, TextElement, VariableType};
use log::LevelFilter;
use parking_lot::RwLock;
use runtime::Runtime;
use state::{FnWrap, State};
use theme::{parser, theme_provider, StyleCss, Theme, ThemeOptions};

pub mod observer;

// Export macros
pub use fiber_macro::func;

pub struct AppBuilder {
    log: bool,
    path: PathBuf,
    state: State,
}

impl AppBuilder {
    /// # Panics
    /// Panics if given path doesn't exists
    #[must_use]
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        assert!(path.as_ref().exists());

        AppBuilder {
            log: true,
            path: path.as_ref().to_path_buf(),
            state: State::new(path.as_ref()),
        }
    }

    #[must_use]
    pub fn log(mut self, v: bool) -> Self {
        self.log = v;
        self
    }

    fn set_logging(&self) {
        if self.log {
            env_logger::builder()
                .filter_module("wgpu_hal", LevelFilter::Error)
                .filter_module("wgpu_core", LevelFilter::Error)
                .filter_module("naga", LevelFilter::Error)
                .filter_module("floem_cosmic_text", LevelFilter::Error)
                .filter_level(LevelFilter::Info)
                .init();

            log::info!("Logging OK");
        }
    }

    #[must_use]
    pub fn handlers(mut self, handlers: Vec<(String, fn(Arc<RwLock<State>>))>) -> Self {
        for (name, f) in handlers {
            self.state.fns.insert(name.replace("_fibr_", ""), FnWrap::from(f));
        }

        self
    }

    /// # Panics
    /// Panics if creating Runtime fails
    pub fn run(self) {
        self.set_logging();

        let (sender, receiver) = crossbeam_channel::unbounded();

        let runtime = RwSignal::new(Runtime::new(&self.path, sender).expect("Failed to create Runtime"));
        let state = Arc::new(RwLock::new(self.state));
        let theme = RwSignal::new(Theme::from_path(&self.path).expect("Invalid theme path"));

        let runtime_event_sig = create_signal_from_channel(receiver.clone());
        let theme_event_sig = create_signal_from_channel(theme.get_untracked().channel.1);

        provide_context(runtime);
        provide_context(state);
        provide_context(theme);

        create_effect(move |_| {
            if runtime_event_sig.get().is_some() {
                runtime.update(Runtime::update_source);
                log::info!("Sources reloaded");
            }
        });

        create_effect(move |_| {
            if theme_event_sig.get().is_some() {
                theme.update(Theme::reload);
                log::info!("Css reloaded");
            }
        });

        let theme_provider = theme_provider(
            move || {
                dyn_view(move || runtime.with(|rt| build_view(rt.source()).into_any()))
                    .css(&["body"])
                    .debug_name("Body")
            },
            ThemeOptions::with_path(self.path.join("styles")),
        );

        floem::launch(|| theme_provider);
    }
}

fn build_view(source: &str) -> impl View {
    let start = std::time::SystemTime::now();

    let view = match parse(source) {
        Ok(node) => c_node_to_view(&node),
        Err(e) => text(e).into_any(),
    }
    .style(Style::size_full)
    .keyboard_navigatable();

    let end = start.elapsed().unwrap();
    log::info!("View built in {}ms", end.as_millis());

    let id = view.id();
    view.on_key_up(Key::Named(NamedKey::F11), Modifiers::empty(), move |_| id.inspect())
}

pub fn c_node_to_view(node: &Node) -> AnyView {
    match node {
        Node::Text(t) => text(t.content).css(&["text"]).into_any(),
        Node::Element(elem) => element_to_anyview(elem),
    }
}

// fn text_element_to_anyview(elem: &TextElement) -> AnyView {
//     if elem.variable_refs.is_empty() {
//         text(elem.content).css(&["text"]).into_any()
//     } else {
//         let state = use_context::<Arc<RwLock<State>>>().unwrap();
//         let state = state.read();

//         let mut content = elem.content.to_string();

//         for var in &elem.variable_refs {
//             let Some((_, name)) = var.name().split_once(':') else {
//                 log::error!("Invalid variable {:?}", var);
//                 continue;
//             };

//             match var.kind {
//                 VariableType::String => {
//                     let sig = state.get_string(name);
//                     if let Some(sig) = sig {
//                         content = content.replace(var.full_match, &sig.get());
//                     }
//                 }
//                 VariableType::Integer => {
//                     let sig = state.get_int(name);
//                     if let Some(sig) = sig {
//                         let val = sig.get();
//                         content = content.replace(var.full_match, &val.to_string());
//                     }
//                 }
//                 VariableType::Float => {
//                     let sig = state.get_float(name);
//                     if let Some(sig) = sig {
//                         // content = content.replace(var.full_match, &sig.get().to_string());
//                     }
//                 }
//                 _ => {
//                     log::warn!("Unsupported inline variable type {:?}", var.kind);
//                 }
//             }
//         }

//         label(move || content.clone()).into_any()
//     }
// }

// Crashing because net
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
            let children = elem.children.iter().map(|n| c_node_to_view(n)).collect::<Vec<_>>();
            container(children).style(Style::size_full).css(&["root"]).into_any()
        }
        ElementKind::Box => {
            let children = elem.children.iter().map(|n| c_node_to_view(n)).collect::<Vec<_>>();
            container(children).css(&["box"]).into_any()
        }
        // ElementKind::Text => {
        //     elem.children.iter().for_each(|e| {
        //         if let Node::Text(t) = e {
        //             if !t.variable_refs.is_empty() {
        //                 log::warn!("<text> element doesn't support inline variables, use <label> instead.\nSource: '{}'\nVars '{:?}'", t.content, t.variable_refs);
        //             }
        //         }
        //     });

        //     children.into_any()
        // }
        ElementKind::Label => {
            // let children = elem.children.iter().map(|n| c_node_to_view(n)).collect::<Vec<_>>();
            if elem.children.is_empty() {
                return text("").into_any();
            }

            if elem.children.iter().any(|e| matches!(e, Node::Element(_))) {
                return text("Label can have only one text element as child").into_any();
            }

            let Some(Node::Text(t)) = elem.children.first() else {
                return text("Label can have only one text element as child").into_any();
            };

            let state = use_context::<Arc<RwLock<State>>>().unwrap();

            let content = RwSignal::new(t.content.to_string());

            // let replace_var_int = |var: &str, sig: RwSignal<i64>| {
            //     let val_str = sig.get().to_string();
            // };

            for var in &t.variable_refs {
                let Some((_, name)) = var.name().split_once(':') else {
                    log::error!("Invalid variable {:?}", var);
                    continue;
                };

                match var.kind {
                    VariableType::String => {}
                    VariableType::Integer => {
                        let value = state
                            .read()
                            .get_int(name)
                            .unwrap_or_else(|| RwSignal::new(0))
                            .get()
                            .to_string();

                        content.update(|c| *c = c.replace(var.full_match, &value));
                    }
                    VariableType::Float => {}
                    _ => {
                        log::warn!("Unsupported inline variable type {:?}", var.kind);
                    }
                }
            }

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
                let state = use_context::<Arc<RwLock<State>>>().unwrap();
                let f = state.read().get_fn(&value.to_string());

                if let Some(onclick_fn) = f {
                    button = button.on_click_cont(move |_| {
                        onclick_fn(state.clone());
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
            let children = elem.children.iter().map(|n| c_node_to_view(n)).collect::<Vec<_>>();
            h_stack_from_iter(children).css(&["hstack"]).into_any()
        }
        ElementKind::VStack => {
            let children = elem.children.iter().map(|n| c_node_to_view(n)).collect::<Vec<_>>();
            v_stack_from_iter(children).css(&["vstack"]).into_any()
        }
        ElementKind::Input => {
            let state = use_context::<Arc<RwLock<State>>>().unwrap();

            let buffer = state
                .read()
                .get_string(&value_var_name.unwrap_or_else(|| elem_value_key.clone()))
                .unwrap_or_else(|| RwSignal::new(format!("Var {elem_value_key} not found")));

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
        AttributeValue::Variable { .. } => todo!("Get value from runtime"),
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
        AttributeValue::Variable { .. } => todo!("Get value from runtime"),
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
