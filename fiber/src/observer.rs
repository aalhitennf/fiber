use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crossbeam_channel::Sender;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

#[derive(Clone)]
pub struct SourceObserver {
    _observer: Rc<FileObserver>,
    source_map: SourceMap,
    path: PathBuf,
}

impl SourceObserver {
    /// # Errors
    /// Fails if file observer can't be created
    pub fn new(path: &Path, sender: Sender<()>) -> Result<Self, Box<dyn std::error::Error>> {
        let observer = FileObserver::new(path, sender, true)?;
        log::info!("Runtime observing {path:?}");
        let source_map = SourceMap::try_from(path)?;

        Ok(SourceObserver {
            _observer: Rc::new(observer),
            source_map,
            path: path.to_path_buf(),
        })
    }

    pub fn update(&mut self) {
        if let Ok(new_map) = SourceMap::try_from(self.path.as_path()) {
            self.source_map = new_map;
        } else {
            log::error!("Source map update failed!");
        }
    }

    pub fn main(&self) -> &str {
        &self.source_map.main
    }

    pub fn component(&self, name: &str) -> Option<&String> {
        self.source_map.components.get(name)
    }
}

#[derive(Clone)]
pub struct SourceMap {
    pub main: String,
    pub components: HashMap<String, String>,
}

impl TryFrom<&Path> for SourceMap {
    type Error = std::io::Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let main = std::fs::read_to_string(path.join("main.fml"))?;

        let mut components = HashMap::new();

        let mut read_dir = |path: &Path| {
            let Ok(dir_entry) = std::fs::read_dir(path) else {
                log::warn!("Failed to read dir: {:?}", path);
                return;
            };

            for entry in dir_entry {
                let Ok(entry) = entry else {
                    log::warn!("Invalid entry: {entry:?}");
                    continue;
                };

                let Ok(meta) = entry.metadata() else {
                    log::warn!("Failed to read entry metadata: {entry:?}");
                    continue;
                };

                let path = entry.path();

                if !meta.is_file() {
                    continue;
                }

                if !path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("fml")) {
                    continue;
                }

                let Some(name) = path.file_stem() else {
                    log::warn!("Failed to get file stem from: {:?}", path);
                    continue;
                };

                let Some(name) = name.to_str() else {
                    log::warn!("Failed to create str from OsStr: {:?}", name);
                    continue;
                };

                let Ok(source) = std::fs::read_to_string(entry.path()) else {
                    log::warn!("Failed to read file content: {:?}", path);
                    continue;
                };

                if components.insert(name.to_string(), source).is_some() {
                    log::warn!("Duplicate component: {name}");
                }

                log::info!("Added component: {name}");
            }
        };

        if path.join("components").exists() {
            read_dir(&path.join("components"));
        }

        Ok(Self { main, components })
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
