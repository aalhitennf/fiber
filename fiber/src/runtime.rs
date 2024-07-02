use std::{path::Path, rc::Rc};

use crossbeam_channel::Sender;

use crate::observer::FileObserver;

#[derive(Clone)]
pub struct Runtime {
    _observer: Rc<FileObserver>,
    source: String,
}

impl Runtime {
    pub fn new(path: &Path, sender: Sender<()>) -> Result<Self, Box<dyn std::error::Error>> {
        let observer = FileObserver::new(&path, sender, true)?;
        log::info!("Runtime observing {path:?}");
        let source = std::fs::read_to_string(&path.join("main.fml"))?;
        log::info!("Main source found ({})", source.len());

        Ok(Runtime {
            _observer: Rc::new(observer),
            source,
        })
    }

    pub fn update_source(&mut self, new_source: String) {
        self.source = new_source;
    }

    pub fn source(&self) -> &String {
        &self.source
    }
}
