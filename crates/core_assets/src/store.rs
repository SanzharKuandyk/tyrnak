use std::collections::HashMap;
use std::sync::Arc;

use crate::handle::{AssetHandle, AssetId, AssetIdGen, AssetStatus};

/// A generic asset store that maps asset IDs to their statuses.
pub struct AssetStore<T> {
    assets: HashMap<AssetId, AssetStatus<T>>,
    path_to_id: HashMap<String, AssetId>,
    id_gen: AssetIdGen,
}

impl<T: Send + Sync + 'static> AssetStore<T> {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            path_to_id: HashMap::new(),
            id_gen: AssetIdGen::new(),
        }
    }

    /// Insert an asset that is immediately ready. Returns a handle to it.
    pub fn insert(&mut self, path: &str, asset: T) -> AssetHandle<T> {
        let id = self.id_gen.next();
        self.assets.insert(id, AssetStatus::Ready(Arc::new(asset)));
        self.path_to_id.insert(path.to_owned(), id);
        AssetHandle::new(id)
    }

    /// Mark an asset as loading. Returns a handle to track it.
    pub fn set_loading(&mut self, path: &str) -> AssetHandle<T> {
        let id = self.id_gen.next();
        self.assets.insert(id, AssetStatus::Loading);
        self.path_to_id.insert(path.to_owned(), id);
        AssetHandle::new(id)
    }

    /// Mark a previously-loading asset as ready.
    pub fn set_ready(&mut self, id: AssetId, asset: T) {
        self.assets.insert(id, AssetStatus::Ready(Arc::new(asset)));
    }

    /// Mark a previously-loading asset as errored.
    pub fn set_error(&mut self, id: AssetId, error: String) {
        self.assets.insert(id, AssetStatus::Error(error));
    }

    /// Get the status of an asset by its handle.
    pub fn get(&self, handle: &AssetHandle<T>) -> Option<&AssetStatus<T>> {
        self.assets.get(&handle.id())
    }

    /// Look up an asset handle by its file path.
    pub fn get_by_path(&self, path: &str) -> Option<AssetHandle<T>> {
        self.path_to_id.get(path).map(|id| AssetHandle::new(*id))
    }

    /// Returns `true` if the asset is in the `Ready` state.
    pub fn is_ready(&self, handle: &AssetHandle<T>) -> bool {
        matches!(self.assets.get(&handle.id()), Some(AssetStatus::Ready(_)))
    }

    /// Returns the inner `Arc<T>` if the asset is ready.
    pub fn get_ready(&self, handle: &AssetHandle<T>) -> Option<Arc<T>> {
        match self.assets.get(&handle.id()) {
            Some(AssetStatus::Ready(arc)) => Some(Arc::clone(arc)),
            _ => None,
        }
    }

    /// Remove an asset from the store.
    pub fn remove(&mut self, handle: &AssetHandle<T>) {
        self.assets.remove(&handle.id());
        // Also remove path mapping if present
        self.path_to_id.retain(|_, id| *id != handle.id());
    }
}

impl<T: Send + Sync + 'static> Default for AssetStore<T> {
    fn default() -> Self {
        Self::new()
    }
}
