use std::path::{Path, PathBuf};
use std::rc::Rc;

use crossbeam_channel::Sender;

use crate::observer::FileObserver;

pub struct Runtime {
    _observer: Rc<FileObserver>,
    source: String,
    path: PathBuf,
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
            path: path.to_path_buf(),
        })
    }

    pub(crate) fn update_source(&mut self) {
        match std::fs::read_to_string(&self.path.join("main.fml")) {
            Ok(new_source) => self.source = new_source,
            Err(e) => {
                log::error!("{e}");
            }
        }
    }

    pub(crate) fn source(&self) -> &str {
        &self.source
    }
}
