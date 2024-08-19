use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use floem::reactive::RwSignal;
use fml::{AttributeValue, VariableType};
use parking_lot::RwLock;

#[derive(Default)]
pub struct State {
    pub strings: HashMap<String, RwSignal<String>>,
    pub ints: HashMap<String, RwSignal<i64>>,
    pub floats: HashMap<String, RwSignal<f64>>,
    pub(crate) fns: HashMap<String, FnWrap>,
}

// pub struct StateCtx(Arc<RwLock<State>>);

// impl Deref for StateCtx {
//     type Target = Arc<RwLock<State>>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl DerefMut for StateCtx {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

fn print_state(state: Arc<RwLock<State>>) {
    log::info!("State status\n");
    let state = state.read();

    log::info!("String ({}):", state.strings.len());
    for (k, v) in &state.strings {
        log::info!("\t{} = {}", k, v.get_untracked());
    }

    log::info!("\nInts ({}):", state.ints.len());
    for (k, v) in &state.ints {
        log::info!("\t{} = {}", k, v.get_untracked());
    }

    log::info!("\nFloats ({}):", state.floats.len());
    for (k, v) in &state.floats {
        log::info!("\t{} = {}", k, v.get_untracked());
    }

    log::info!("\nFns ({}):", state.fns.len());
    for (k, v) in &state.fns {
        log::info!("\t{} = {:?}", k, v);
    }
}

#[derive(Debug)]
pub struct FnWrap {
    f: FnPointer,
}

impl From<FnPointer> for FnWrap {
    fn from(f: FnPointer) -> Self {
        Self { f }
    }
}

pub type FnPointer = fn(Arc<RwLock<State>>);

impl State {
    #[must_use]
    pub fn new(path: &Path) -> Self {
        let mut state = State::default();
        state.read_vars(path);

        state.set_fn("dbg_print_state".to_string(), print_state);

        state
    }

    fn read_vars(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref().join("main.vars");
        let Ok(content) = std::fs::read_to_string(&path) else {
            log::error!("No vars file: {path:?}");
            return;
        };

        for line in content.lines() {
            let parts = line.split([':', ' ']).collect::<Vec<_>>();

            match parts[..] {
                [t, n, d] => {
                    let kind = VariableType::from(t);
                    let name = n.to_string();
                    match kind {
                        VariableType::String | VariableType::Unknown => {
                            self.strings.insert(name, RwSignal::new(d.to_string()));
                        }
                        VariableType::Integer => {
                            self.ints
                                .insert(name, RwSignal::new(d.parse::<i64>().unwrap_or_default()));
                        }
                        VariableType::Float => {
                            self.floats
                                .insert(name, RwSignal::new(d.parse::<f64>().unwrap_or_default()));
                        }
                    };
                }

                _ => log::warn!("Invalid variable definition: {line}"),
            }
        }
    }

    pub fn set_string(&mut self, key: String, value: String) -> Option<String> {
        if let Some(sig) = self.strings.get_mut(&key) {
            sig.update(|v| *v = value);
            Some(sig.get_untracked())
        } else {
            self.strings.insert(key, RwSignal::new(value));
            None
        }
    }

    pub fn set_int(&mut self, key: String, value: i64) -> Option<i64> {
        if let Some(sig) = self.ints.get_mut(&key) {
            sig.update(|v| *v = value);
            Some(sig.get_untracked())
        } else {
            self.ints.insert(key, RwSignal::new(value));
            None
        }
    }

    pub fn set_float(&mut self, key: String, value: f64) -> Option<f64> {
        if let Some(sig) = self.floats.get_mut(&key) {
            sig.update(|v| *v = value);
            Some(sig.get_untracked())
        } else {
            self.floats.insert(key, RwSignal::new(value));
            None
        }
    }

    pub fn set_fn(&mut self, key: String, f: FnPointer) {
        self.fns.insert(key, FnWrap::from(f));
    }

    pub fn set_var(&mut self, key: String, value: AttributeValue) {
        log::info!("Var set {key}: {value:?}");

        match value {
            AttributeValue::String { value, .. } => {
                if let Some(sig) = self.strings.get_mut(&key) {
                    sig.update(|v| *v = value.to_string());
                } else {
                    self.strings.insert(key, RwSignal::new(value.to_string()));
                }
            }

            AttributeValue::Integer { value, .. } => {
                if let Some(sig) = self.ints.get_mut(&key) {
                    sig.update(|v| *v = value);
                } else {
                    self.ints.insert(key, RwSignal::new(value));
                }
            }

            AttributeValue::Float { value, .. } => {
                if let Some(sig) = self.floats.get_mut(&key) {
                    sig.update(|v| *v = value);
                } else {
                    self.floats.insert(key, RwSignal::new(value));
                }
            }

            AttributeValue::Variable { name, .. } => match name.kind {
                VariableType::Integer => {
                    self.set_int(name.name.to_string(), i64::default());
                }
                VariableType::Float => {
                    self.set_float(name.name.to_string(), f64::default());
                }
                VariableType::String | VariableType::Unknown => {
                    self.set_string(name.name.to_string(), String::default());
                }
            },
        }
    }

    #[must_use]
    pub fn get_string(&self, key: &str) -> Option<&RwSignal<String>> {
        self.strings.get(key)
    }

    #[must_use]
    pub fn get_int(&self, key: &str) -> Option<&RwSignal<i64>> {
        self.ints.get(key)
    }

    #[must_use]
    pub fn get_float(&self, key: &str) -> Option<&RwSignal<f64>> {
        self.floats.get(key)
    }

    #[must_use]
    pub fn get_fn(&self, key: &str) -> Option<FnPointer> {
        self.fns.get(key).map(|w| w.f)
    }
}
