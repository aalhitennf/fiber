use std::path::Path;
use std::sync::Arc;

use dashmap::DashMap;
use floem::reactive::{use_context, RwSignal};
use fml::{AttributeValue, VariableType};

#[derive(Default)]
pub struct State {
    pub strings: DashMap<String, RwSignal<String>>,
    pub ints: DashMap<String, RwSignal<i64>>,
    pub floats: DashMap<String, RwSignal<f64>>,
    pub(crate) fns: DashMap<String, FnWrap>,
}

pub type StateCtx = Arc<State>;

fn print_state() {
    let state = use_context::<StateCtx>().unwrap();

    log::info!("State\n");

    log::info!("String ({}):", state.strings.len());
    for entry in &state.strings {
        log::info!("\t{} = {}", entry.key(), entry.get_untracked());
    }

    log::info!("\nInts ({}):", state.ints.len());
    for entry in &state.ints {
        log::info!("\t{} = {}", entry.key(), entry.get_untracked());
    }

    log::info!("\nFloats ({}):", state.floats.len());
    for entry in &state.floats {
        log::info!("\t{} = {}", entry.key(), entry.get_untracked());
    }

    log::info!("\nFns ({}):", state.fns.len());
    for entry in &state.fns {
        log::info!("\t{} = {:?}", entry.key(), entry.value());
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FnWrap {
    f: FnPointer,
}

impl From<FnPointer> for FnWrap {
    fn from(f: FnPointer) -> Self {
        Self { f }
    }
}

pub type FnPointer = fn();

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

            if let [t, n, d] = parts[..] {
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
            } else {
                log::warn!("Invalid variable definition: {line}");
            }
        }
    }

    pub fn set_string(&self, key: String, value: String) -> Option<String> {
        let sig = self.strings.get(&key);
        if let Some(sig) = sig {
            sig.set(value.clone());
            // sig.update(|v| *v = value.clone());
            Some(value)
        } else {
            self.strings.insert(key, RwSignal::new(value));
            None
        }
    }

    pub fn set_int(&self, key: String, value: i64) -> Option<i64> {
        if let Some(sig) = self.ints.get(&key) {
            sig.set(value);
            // sig.update(|v| *v = value);
            Some(value)
        } else {
            self.ints.insert(key, RwSignal::new(value));
            None
        }
    }

    pub fn set_float(&self, key: String, value: f64) -> Option<f64> {
        if let Some(sig) = self.floats.get(&key) {
            sig.update(|v| *v = value);
            Some(sig.get_untracked())
        } else {
            self.floats.insert(key, RwSignal::new(value));
            None
        }
    }

    pub fn set_fn(&self, key: String, f: FnPointer) {
        self.fns.insert(key, FnWrap::from(f));
    }

    pub fn set_var(&self, key: String, value: AttributeValue) {
        log::info!("Var set {key}: {value:?}");

        match value {
            AttributeValue::String { value, .. } => {
                if let Some(sig) = self.strings.get(&key) {
                    sig.update(|v| *v = value.to_string());
                } else {
                    self.strings.insert(key, RwSignal::new(value.to_string()));
                }
            }

            AttributeValue::Integer { value, .. } => {
                if let Some(sig) = self.ints.get(&key) {
                    sig.update(|v| *v = value);
                } else {
                    self.ints.insert(key, RwSignal::new(value));
                }
            }

            AttributeValue::Float { value, .. } => {
                if let Some(sig) = self.floats.get(&key) {
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
    pub fn get_string(&self, key: &str) -> Option<RwSignal<String>> {
        self.strings.get(key).map(|r| *r.value())
    }

    #[must_use]
    pub fn get_int(&self, key: &str) -> Option<RwSignal<i64>> {
        self.ints.get(key).map(|r| *r.value())
    }

    #[must_use]
    pub fn get_float(&self, key: &str) -> Option<RwSignal<f64>> {
        self.floats.get(key).map(|r| *r.value())
    }

    #[must_use]
    pub fn get_fn(&self, key: &str) -> Option<FnPointer> {
        self.fns.get(key).map(|w| w.f)
    }
}
