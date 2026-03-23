// FileWatcher tests

use leankg::watcher::FileWatcher;
use tempfile::TempDir;

#[test]
fn test_file_watcher_creation() {
    let tmp = TempDir::new().unwrap();
    let watcher = FileWatcher::new(tmp.path());
    assert!(watcher.is_ok());
}

#[test]
fn test_file_watcher_watch_path() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().to_path_buf();
    let watcher = FileWatcher::new(&path).unwrap();
    assert_eq!(watcher.watch_path(), path);
}
