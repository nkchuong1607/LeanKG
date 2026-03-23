use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

pub struct FileWatcher {
    watcher: RecommendedWatcher,
    watch_path: PathBuf,
    rx: Receiver<Result<Event, notify::Error>>,
}

impl FileWatcher {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, notify::Error> {
        let (tx, rx) = channel();

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(Ok(event));
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )?;

        let mut watcher = watcher;
        watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

        Ok(Self {
            watcher,
            watch_path: path.as_ref().to_path_buf(),
            rx,
        })
    }

    pub fn watch_path(&self) -> &Path {
        &self.watch_path
    }

    pub fn try_recv_event(&self) -> Option<Event> {
        self.rx.try_recv().ok().and_then(|r| r.ok())
    }

    pub fn recv_event(&self) -> Option<Event> {
        self.rx.recv().ok().and_then(|r| r.ok())
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        let _ = self.watcher.unwatch(self.watch_path.as_path());
    }
}
