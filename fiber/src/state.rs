use std::collections::HashMap;

use floem::reactive::{use_context, RwSignal};
use xxhash_rust::xxh64::Xxh64Builder;

#[derive(Clone)]
pub struct State {
    vars: HashMap<String, String, Xxh64Builder>,
    // vars: Rc<RefCell<HashMap<String, String, Xxh64Builder>>>,
    fns: HashMap<String, FnWrap, Xxh64Builder>,
    // fns: Rc<RefCell<HashMap<String, FnWrap, Xxh64Builder>>>,
}

fn print_state() {
    let state = use_context::<RwSignal<State>>().unwrap();
    state.with_untracked(|s| {
        log::info!("State status");
        log::info!("Vars: {}", s.vars.len());
        log::info!("Fns: {}", s.fns.len());
    })
}

#[derive(Clone, Copy)]
pub struct FnWrap {
    f: fn() -> (),
}

impl From<fn() -> ()> for FnWrap {
    fn from(f: FnPointer) -> Self {
        Self { f }
    }
}

type FnPointer = fn();

impl State {
    #[must_use]
    pub fn new() -> State {
        let mut state = State {
            vars: HashMap::default(),
            // vars: Rc::new(RefCell::new(HashMap::default())),
            fns: HashMap::default(),
            // fns: Rc::new(RefCell::new(HashMap::default())),
        };

        state.set_fn("print_state".to_string(), print_state);

        state
    }

    pub fn set_var(&mut self, key: String, value: String) {
        self.vars.insert(key, value);
    }

    #[must_use]
    pub fn get_var(&self, key: &str) -> Option<&String> {
        // TODO Clone hater 666
        self.vars.get(key)
    }

    pub fn set_fn(&mut self, key: String, f: FnPointer) {
        self.fns.insert(key, FnWrap::from(f));
    }

    pub fn get_fn(&self, key: &str) -> Option<FnPointer> {
        self.fns.get(key).map(|w| w.f)
    }
}
