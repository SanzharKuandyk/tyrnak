use std::path::{Path, PathBuf};
use std::sync::mpsc;

use notify::{EventKind, RecursiveMode, Watcher};
use tracing::warn;

/// Simplified file-system event.
pub struct FileEvent {
    pub path: PathBuf,
    pub kind: FileEventKind,
}

/// The kind of file-system change observed.
pub enum FileEventKind {
    Created,
    Modified,
    Removed,
}

/// Watches directories for file changes using the OS-recommended backend.
pub struct FileWatcher {
    _watcher: notify::RecommendedWatcher,
    events_rx: mpsc::Receiver<notify::Result<notify::Event>>,
}

impl FileWatcher {
    /// Create a new file watcher.
    pub fn new() -> Result<Self, notify::Error> {
        let (tx, rx) = mpsc::channel();
        let watcher = notify::recommended_watcher(tx)?;
        Ok(Self {
            _watcher: watcher,
            events_rx: rx,
        })
    }

    /// Start watching a directory (recursively).
    pub fn watch(&mut self, path: &Path) -> Result<(), notify::Error> {
        self._watcher.watch(path, RecursiveMode::Recursive)
    }

    /// Drain all pending events without blocking.
    pub fn poll_events(&self) -> Vec<FileEvent> {
        let mut events = Vec::new();
        while let Ok(result) = self.events_rx.try_recv() {
            match result {
                Ok(event) => {
                    let kind = match event.kind {
                        EventKind::Create(_) => Some(FileEventKind::Created),
                        EventKind::Modify(_) => Some(FileEventKind::Modified),
                        EventKind::Remove(_) => Some(FileEventKind::Removed),
                        _ => None,
                    };
                    if let Some(kind) = kind {
                        for path in event.paths {
                            events.push(FileEvent {
                                path,
                                kind: match &kind {
                                    FileEventKind::Created => FileEventKind::Created,
                                    FileEventKind::Modified => FileEventKind::Modified,
                                    FileEventKind::Removed => FileEventKind::Removed,
                                },
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!("File watcher error: {e}");
                }
            }
        }
        events
    }
}
