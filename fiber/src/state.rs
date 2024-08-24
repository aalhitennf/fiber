use std::any::Any;
use std::fmt::Display;
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;

use dashmap::DashMap;
use floem::reactive::RwSignal;
use floem::{AnyView, IntoView, View, ViewId};
use fml::VariableType;

pub trait Viewable: View + Any {
    fn into_anyview(&self) -> AnyView;
}

#[derive(Default)]
pub struct State {
    pub(crate) fns: DashMap<String, FnPointer>,
    pub(crate) variables: DashMap<VariableKey, RwSignal<Box<dyn Any>>>,
    pub(crate) viewables: DashMap<String, RwSignal<Vec<Box<dyn Viewable>>>>,
}

#[derive(Clone)]
struct Koira {
    id: ViewId,
    val: i64,
}

impl View for Koira {
    fn id(&self) -> floem::ViewId {
        self.id
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct VariableKey {
    name: String,
    ty: String,
}

impl Display for VariableKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}

impl VariableKey {
    pub fn new<T>(name: &str) -> Self {
        let ty = std::any::type_name::<T>().to_string();
        Self {
            name: name.to_string(),
            ty,
        }
    }
}

#[derive(Clone)]
pub struct StateCtx(Rc<State>);

impl StateCtx {
    pub fn new(state: State) -> Self {
        Self(Rc::new(state))
    }
}

impl Deref for StateCtx {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[fiber_macro::task(debug)]
fn dbg_print_state(state: StateCtx) {
    log::info!("State");

    log::info!("");

    log::info!("Variables ({}):", state.variables.len());
    for entry in &state.variables {
        log::info!("\t{}", entry.key());
    }

    log::info!("");

    log::info!("Viewables ({}):", state.viewables.len());
    for entry in &state.viewables {
        log::info!("\t{}", entry.key());
    }

    log::info!("");

    log::info!("Fns ({}):", state.fns.len());
    for entry in &state.fns {
        log::info!("\t{} = Fn", entry.key());
    }
}

pub type FnPointer = fn();

impl State {
    #[must_use]
    #[allow(unused)]
    pub(crate) fn new(path: &Path) -> Self {
        let mut state = State::default();
        state.read_vars(path);
        state
    }

    pub(crate) fn read_vars(&mut self, path: &Path) {
        self.add_handler(dbg_print_state());

        let Ok(content) = std::fs::read_to_string(path) else {
            log::error!("No vars file: {path:?}");
            return;
        };

        for line in content.lines() {
            let parts = line.split([':', ' ']).collect::<Vec<_>>();

            if let [t, name, d] = parts[..] {
                let kind = VariableType::from(t);

                match kind {
                    VariableType::String | VariableType::Unknown => {
                        log::info!("Created String variable: {name}");
                        let boxed_val: Box<dyn Any> = Box::new(d.to_string());
                        self.variables
                            .insert(VariableKey::new::<String>(name), RwSignal::new(boxed_val));
                    }
                    VariableType::Integer => {
                        log::info!("Created i64 variable: {name}");
                        let boxed_val: Box<dyn Any> = Box::new(d.parse::<i64>().unwrap_or_default());
                        self.variables
                            .insert(VariableKey::new::<i64>(name), RwSignal::new(boxed_val));
                    }
                    VariableType::Float => {
                        log::info!("Created f64 variable: {name}");
                        let boxed_val: Box<dyn Any> = Box::new(d.parse::<f64>().unwrap_or_default());
                        self.variables
                            .insert(VariableKey::new::<f64>(name), RwSignal::new(boxed_val));
                    }
                };
            } else {
                log::warn!("Invalid variable definition: {line}");
            }
        }
    }

    pub fn set_fn(&self, key: String, f: FnPointer) {
        self.fns.insert(key, f);
    }

    /// # Panics
    /// Panics if the handler already exists
    pub fn add_handler(&self, (key, f): (String, FnPointer)) {
        let name = key.replace("_fibr_", "");
        assert!(
            self.fns.insert(name.clone(), f).is_none(),
            "Handler already exists: {name}"
        );
    }

    #[must_use]
    pub fn get<T>(&self, key: &str) -> Option<RwSignal<Box<dyn Any>>> {
        self.variables.view(&VariableKey::new::<T>(key), |_, v| *v)
    }

    pub fn get_view(&self, key: &str) -> Option<RwSignal<Vec<Box<dyn Viewable>>>> {
        self.viewables.view(key, |_, v| *v)
    }

    pub fn set<T: 'static>(&self, key: &str, value: T) {
        if let Some(sig) = self.variables.get(&VariableKey::new::<T>(key)) {
            let new_rc: Box<dyn Any> = Box::new(value);
            sig.set(new_rc);
        } else {
            log::error!("No var {key}");
        }
    }

    pub fn insert_view<T: Viewable + Clone + 'static>(&self, key: &str, value: Vec<T>) {
        let dyn_items = value
            .into_iter()
            .map(|v| {
                let iv: Box<dyn Viewable> = Box::new(v);
                iv
            })
            .collect::<Vec<_>>();

        self.viewables.insert(key.to_string(), RwSignal::new(dyn_items));
    }

    pub fn update<T: 'static>(&self, key: &str, f: impl FnOnce(&mut T)) {
        if let Some(sig) = self.variables.get(&VariableKey::new::<T>(key)) {
            sig.update(|v| {
                if let Some(vv) = (*v).downcast_mut::<T>() {
                    f(vv);
                }
            });
        } else {
            log::error!("No var {key}");
        }
    }

    pub fn update_view<T: Viewable + Clone + 'static>(&self, key: &str, f: impl FnOnce(&mut Vec<T>)) {
        if let Some(sig) = self.viewables.get(key) {
            let mut items = sig.with_untracked(|v| {
                let mut items = Vec::with_capacity(v.len());

                for item in v {
                    if let Some(v) = (item as &dyn Any).downcast_ref::<T>() {
                        items.push(v.clone());
                    } else {
                        eprintln!("cast failed in task: {}", item.debug_name());
                    }
                }

                items
            });

            f(&mut items);

            let d_cast = items
                .into_iter()
                .map(|i| {
                    let b: Box<dyn Viewable> = Box::new(i);
                    b
                })
                .collect::<Vec<_>>();

            sig.set(d_cast);
        }
    }

    #[must_use]
    pub fn get_fn(&self, key: &str) -> Option<FnPointer> {
        self.fns.get(key).map(|w| *w)
    }
}
