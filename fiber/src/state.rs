use std::{any::Any, borrow::Cow, cell::RefCell, collections::HashMap, rc::Rc};

use xxhash_rust::xxh64::Xxh64Builder;

#[derive(Clone)]
pub struct State<'a> {
    vars: Rc<RefCell<HashMap<Cow<'a, str>, Box<dyn Any>, Xxh64Builder>>>,
}

impl<'a> State<'a> {
    #[must_use]
    pub fn new() -> Self {
        State {
            vars: Rc::new(RefCell::new(HashMap::default())),
        }
    }

    pub fn set_var<T: 'static>(&mut self, key: &'a str, value: T) -> Option<T> {
        if let Some(Ok(value)) = self
            .vars
            .borrow_mut()
            .insert(Cow::Borrowed(key), Box::new(value))
            .map(|v| v.downcast::<T>())
        {
            Some(*value)
        } else {
            None
        }
    }

    pub fn remove_var<T: 'static>(&mut self, key: &str) -> Option<T> {
        if let Some(Ok(value)) = self.vars.borrow_mut().remove(key).map(|v| v.downcast::<T>()) {
            Some(*value)
        } else {
            None
        }
    }

    #[must_use]
    pub fn get_var<T: 'static>(&self, key: &str) -> Option<&T> {
        if let Some(Some(value)) = self.vars.borrow().get(key).map(|v| v.downcast_ref::<&T>()) {
            Some(value)
        } else {
            None
        }
    }
}
