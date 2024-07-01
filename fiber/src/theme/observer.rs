use std::path::{Path, PathBuf};

use crossbeam_channel::Sender;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

#[allow(dead_code)]
pub struct FileObserver {
    watcher: RecommendedWatcher,
    path: PathBuf,
}

impl FileObserver {
    pub fn new(
        path: &Path,
        o_tx: Sender<()>,
        recursive: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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

        Ok(FileObserver {
            watcher,
            path: path.to_path_buf(),
        })
    }
}
