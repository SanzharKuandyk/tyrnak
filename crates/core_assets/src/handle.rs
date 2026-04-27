use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetId(u64);

impl AssetId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn raw(&self) -> u64 {
        self.0
    }
}

/// Generates unique asset IDs.
pub struct AssetIdGen {
    next: AtomicU64,
}

impl AssetIdGen {
    pub fn new() -> Self {
        Self {
            next: AtomicU64::new(1),
        }
    }

    pub fn next(&self) -> AssetId {
        AssetId(self.next.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for AssetIdGen {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum AssetStatus<T> {
    Loading,
    Ready(Arc<T>),
    Error(String),
}

#[derive(Debug, Clone, Copy)]
pub struct AssetHandle<T> {
    pub id: AssetId,
    _marker: std::marker::PhantomData<T>,
}

impl<T> AssetHandle<T> {
    pub fn new(id: AssetId) -> Self {
        Self {
            id,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn id(&self) -> AssetId {
        self.id
    }
}

impl<T> PartialEq for AssetHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for AssetHandle<T> {}

impl<T> std::hash::Hash for AssetHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
