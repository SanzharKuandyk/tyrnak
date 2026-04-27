//! # core_assets
//!
//! Asset management system — typed handles, async loading, caching, and hot-reload.
//!
//! Provides `AssetHandle<T>`, `AssetStore`, and the `AssetLoader<T>` trait for
//! user-defined asset types. Uses papaya for concurrent caching and notify for
//! filesystem watching.

pub mod handle;
pub mod loader;
pub mod store;
pub mod watcher;

pub use handle::{AssetHandle, AssetId, AssetIdGen, AssetStatus};
pub use loader::{AssetLoadError, AssetLoader};
pub use store::AssetStore;
pub use watcher::{FileEvent, FileEventKind, FileWatcher};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn asset_id_gen_unique() {
        let id_gen = AssetIdGen::new();
        let mut ids = HashSet::new();
        for _ in 0..1000 {
            assert!(ids.insert(id_gen.next()));
        }
    }

    #[test]
    fn asset_handle_creation() {
        let id = AssetId::new(42);
        let handle: AssetHandle<String> = AssetHandle::new(id);
        assert_eq!(handle.id(), id);
        assert_eq!(handle.id().raw(), 42);
    }

    #[test]
    fn asset_handle_is_copy() {
        let handle: AssetHandle<u32> = AssetHandle::new(AssetId::new(1));
        let copy = handle;
        assert_eq!(handle.id(), copy.id());
    }

    #[test]
    fn store_insert_get_is_ready() {
        let mut store = AssetStore::<String>::new();
        let handle = store.insert("test.txt", "hello".to_owned());

        assert!(store.is_ready(&handle));
        let arc = store.get_ready(&handle).unwrap();
        assert_eq!(*arc, "hello");
    }

    #[test]
    fn store_get_by_path() {
        let mut store = AssetStore::<u32>::new();
        let handle = store.insert("data/mesh.obj", 99);

        let found = store.get_by_path("data/mesh.obj").unwrap();
        assert_eq!(found.id(), handle.id());

        assert!(store.get_by_path("nonexistent").is_none());
    }

    #[test]
    fn store_remove() {
        let mut store = AssetStore::<u32>::new();
        let handle = store.insert("asset.bin", 1);
        assert!(store.is_ready(&handle));

        store.remove(&handle);
        assert!(!store.is_ready(&handle));
        assert!(store.get(&handle).is_none());
        assert!(store.get_by_path("asset.bin").is_none());
    }

    #[test]
    fn store_loading_then_ready_lifecycle() {
        let mut store = AssetStore::<Vec<u8>>::new();
        let handle = store.set_loading("image.png");

        // Should be loading, not ready
        assert!(!store.is_ready(&handle));
        assert!(store.get_ready(&handle).is_none());
        assert!(matches!(store.get(&handle), Some(AssetStatus::Loading)));

        // Transition to ready
        store.set_ready(handle.id(), vec![1, 2, 3]);
        assert!(store.is_ready(&handle));
        let data = store.get_ready(&handle).unwrap();
        assert_eq!(&*data, &[1, 2, 3]);
    }

    #[test]
    fn store_loading_then_error() {
        let mut store = AssetStore::<String>::new();
        let handle = store.set_loading("broken.dat");

        store.set_error(handle.id(), "file not found".to_owned());
        assert!(!store.is_ready(&handle));
        match store.get(&handle) {
            Some(AssetStatus::Error(msg)) => assert_eq!(msg, "file not found"),
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn asset_load_error_display() {
        let err = AssetLoadError {
            path: "textures/wall.png".to_owned(),
            message: "corrupt header".to_owned(),
        };
        let s = format!("{err}");
        assert_eq!(s, "Failed to load 'textures/wall.png': corrupt header");
    }
}
