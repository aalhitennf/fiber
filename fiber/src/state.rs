use std::any::Any;
use std::fmt::Display;
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;

use dashmap::DashMap;
use floem::reactive::RwSignal;
use fml::VariableType;

#[derive(Default)]
pub struct State {
    pub(crate) fns: DashMap<String, FnPointer>,
    pub(crate) variables: DashMap<VariableKey, RwSignal<Box<dyn Any>>>,
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
pub struct StateCtx(Arc<State>);

impl StateCtx {
    pub fn new(state: State) -> Self {
        Self(Arc::new(state))
    }
}

impl Deref for StateCtx {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[fiber_macro::func(debug)]
fn dbg_print_state(state: StateCtx) {
    log::info!("State");

    log::info!("");

    log::info!("Variables ({}):", state.variables.len());
    for entry in &state.variables {
        log::info!("\t{}", entry.key());
    }

    // log::info!("String ({}):", state.strings.len());
    // for entry in &state.strings {
    //     log::info!("\t{} = {}", entry.key(), entry.get_untracked());
    // }

    // log::info!("");
    // log::info!("Ints ({}):", state.ints.len());
    // for entry in &state.ints {
    //     log::info!("\t{} = {}", entry.key(), entry.get_untracked());
    // }

    // log::info!("");

    // log::info!("Floats ({}):", state.floats.len());
    // for entry in &state.floats {
    //     log::info!("\t{} = {}", entry.key(), entry.get_untracked());
    // }

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

        let Ok(content) = std::fs::read_to_string(&path) else {
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
                        // let boxed_val: Rc<dyn Any> = Rc::new(d.to_string());
                        // self.variables.insert(name.clone(), RwSignal::new(boxed_val));
                        // self.strings.insert(name, RwSignal::new(d.to_string()));
                    }
                    VariableType::Integer => {
                        log::info!("Created i64 variable: {name}");
                        let boxed_val: Box<dyn Any> = Box::new(d.parse::<i64>().unwrap_or_default());
                        self.variables
                            .insert(VariableKey::new::<i64>(name), RwSignal::new(boxed_val));
                        // let boxed_val: Rc<dyn Any> = Rc::new(d.parse::<i64>().unwrap_or_default());
                        // self.variables.insert(name.clone(), RwSignal::new(boxed_val));
                        // self.ints
                        // .insert(name, RwSignal::new(d.parse::<i64>().unwrap_or_default()));
                    }
                    VariableType::Float => {
                        log::info!("Created f64 variable: {name}");
                        let boxed_val: Box<dyn Any> = Box::new(d.parse::<f64>().unwrap_or_default());
                        self.variables
                            .insert(VariableKey::new::<f64>(name), RwSignal::new(boxed_val));
                        // let boxed_val: Rc<dyn Any> = Rc::new(d.parse::<f64>().unwrap_or_default());
                        // self.variables.insert(name.clone(), RwSignal::new(boxed_val));
                        // self.floats
                        // .insert(name, RwSignal::new(d.parse::<f64>().unwrap_or_default()));
                    }
                };
            } else {
                log::warn!("Invalid variable definition: {line}");
            }
        }
    }

    // pub fn set_string(&self, key: String, value: String) -> Option<String> {
    //     let sig = self.strings.get(&key);
    //     if let Some(sig) = sig {
    //         sig.set(value.clone());
    //         // sig.update(|v| *v = value.clone());
    //         Some(value)
    //     } else {
    //         self.strings.insert(key, RwSignal::new(value));
    //         None
    //     }
    // }

    // pub fn set_int(&self, key: String, value: i64) -> Option<i64> {
    //     if let Some(sig) = self.ints.get(&key) {
    //         sig.set(value);
    //         // sig.update(|v| *v = value);
    //         Some(value)
    //     } else {
    //         self.ints.insert(key, RwSignal::new(value));
    //         None
    //     }
    // }

    // pub fn set_float(&self, key: String, value: f64) -> Option<f64> {
    //     if let Some(sig) = self.floats.get(&key) {
    //         sig.update(|v| *v = value);
    //         Some(sig.get_untracked())
    //     } else {
    //         self.floats.insert(key, RwSignal::new(value));
    //         None
    //     }
    // }

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

    // pub fn create_var(&self, key: String, value: AttributeValue) {
    //     log::info!("Var set {key}: {value:?}");

    //     match value {
    //         AttributeValue::String { value, .. } => {
    //             if let Some(sig) = self.strings.get(&key) {
    //                 sig.update(|v| *v = value.to_string());
    //             } else {
    //                 self.strings.insert(key, RwSignal::new(value.to_string()));
    //             }
    //         }

    //         AttributeValue::Integer { value, .. } => {
    //             if let Some(sig) = self.ints.get(&key) {
    //                 sig.update(|v| *v = value);
    //             } else {
    //                 self.ints.insert(key, RwSignal::new(value));
    //             }
    //         }

    //         AttributeValue::Float { value, .. } => {
    //             if let Some(sig) = self.floats.get(&key) {
    //                 sig.update(|v| *v = value);
    //             } else {
    //                 self.floats.insert(key, RwSignal::new(value));
    //             }
    //         }

    //         AttributeValue::Variable { name, .. } => match name.kind {
    //             VariableType::Integer => {
    //                 self.set(name.name, i64::default());
    //             }
    //             VariableType::Float => {
    //                 self.set(name.name, f64::default());
    //             }
    //             VariableType::String | VariableType::Unknown => {
    //                 self.set(name.name, String::default());
    //             }
    //         },
    //     }
    // }

    pub fn get<T>(&self, key: &str) -> Option<RwSignal<Box<dyn Any>>> {
        let sig = self.variables.get(&VariableKey::new::<T>(key))?;

        log::info!("Found {key}");

        Some(sig.value().clone())
    }

    pub fn set<T: 'static>(&self, key: &str, value: T) {
        if let Some(sig) = self.variables.get(&VariableKey::new::<T>(key)) {
            let new_rc: Box<dyn Any> = Box::new(value);
            sig.set(new_rc);
        } else {
            log::error!("No var {key}");
        }
    }

    pub fn update<T: 'static>(&self, key: &str, f: impl FnOnce(&mut T)) {
        if let Some(sig) = self.variables.get(&VariableKey::new::<T>(key)) {
            sig.update(|v| {
                if let Some(vv) = (*v).downcast_mut::<T>() {
                    f(vv);
                }
            });
        } else {
            log::warn!("No var {key}");
        }
    }

    // pub fn update<T>(&self, key: &str, f: impl FnOnce(&T)) {
    //     if let Some(sig) = self.variables.get(key) {
    //         sig.update(|v| {
    //             if let Some(vv) = (v).downcast_ref::<T>() {
    //                 f(vv);
    //             }
    //         });
    //     } else {
    //         log::warn!("No var {key}");
    //     }
    // }

    // #[must_use]
    // pub fn get_string(&self, key: &str) -> Option<RwSignal<String>> {
    //     self.strings.get(key).map(|r| *r.value())
    // }

    // #[must_use]
    // pub fn get_int(&self, key: &str) -> Option<RwSignal<i64>> {
    //     self.ints.get(key).map(|r| *r.value())
    // }

    // #[must_use]
    // pub fn get_float(&self, key: &str) -> Option<RwSignal<f64>> {
    //     self.floats.get(key).map(|r| *r.value())
    // }

    #[must_use]
    pub fn get_fn(&self, key: &str) -> Option<FnPointer> {
        self.fns.get(key).map(|w| *w)
    }

    // pub fn update_string(&self, key: &str, f: impl FnOnce(&mut String)) {
    //     if let Some(sig) = self.strings.get(key) {
    //         sig.update(f);
    //     } else {
    //         log::warn!("No string var {key}");
    //     }
    // }

    // pub fn update_int(&self, key: &str, f: impl FnOnce(&mut i64)) {
    //     if let Some(sig) = self.ints.get(key) {
    //         sig.update(f);
    //     } else {
    //         log::warn!("No int var {key}");
    //     }
    // }

    // pub fn update_float(&self, key: &str, f: impl FnOnce(&mut f64)) {
    //     if let Some(sig) = self.floats.get(key) {
    //         sig.update(f);
    //     } else {
    //         log::warn!("No float var {key}");
    //     }
    // }
}
