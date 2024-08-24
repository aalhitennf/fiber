use std::path::{Path, PathBuf};
use std::rc::Rc;

use crossbeam_channel::Sender;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

#[derive(Clone)]
pub struct SourceObserver {
    _observer: Rc<FileObserver>,
    source: String,
    path: PathBuf,
}

impl SourceObserver {
    /// # Errors
    /// Fails if file observer can't be created
    pub fn new(path: &Path, sender: Sender<()>) -> Result<Self, Box<dyn std::error::Error>> {
        let observer = FileObserver::new(path, sender, true)?;
        log::info!("Runtime observing {path:?}");
        let source = std::fs::read_to_string(path.join("main.fml"))?;
        log::info!("Main source found ({})", source.len());

        Ok(SourceObserver {
            _observer: Rc::new(observer),
            source,
            path: path.to_path_buf(),
        })
    }

    pub fn update(&mut self) {
        match std::fs::read_to_string(self.path.join("main.fml")) {
            Ok(new_source) => self.source = new_source,
            Err(e) => {
                log::error!("{e}");
            }
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

pub(crate) struct FileObserver {
    _watcher: RecommendedWatcher,
}

impl FileObserver {
    /// # Errors
    /// Panics if initializing notify watcher fails
    pub fn new(path: &Path, o_tx: Sender<()>, recursive: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let p = path.to_path_buf();
        let mut watcher = notify::recommended_watcher(move |res| match res {
            Ok(Event {
                kind: EventKind::Create(_) | EventKind::Modify(_),
                ..
            }) => {
                if let Err(e) = o_tx.send(()) {
                    eprintln!("Observer send error: {e:?}");
                }
            }

            Ok(_) => (),

            Err(e) => {
                eprintln!("Observer error: {p:?}");
                eprintln!("{e}");
            }
        })?;

        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        watcher.watch(path, mode)?;

        Ok(FileObserver { _watcher: watcher })
    }
}
