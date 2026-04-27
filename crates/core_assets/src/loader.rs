use std::path::Path;

/// Trait for user-defined asset loaders.
pub trait AssetLoader: Send + Sync {
    type Asset: Send + Sync + 'static;

    /// File extensions this loader supports, e.g. `["png", "jpg"]`.
    fn extensions(&self) -> &[&str];

    /// Load an asset from the given file path.
    fn load(&self, path: &Path) -> Result<Self::Asset, AssetLoadError>;
}

#[derive(Debug, Clone)]
pub struct AssetLoadError {
    pub path: String,
    pub message: String,
}

impl std::fmt::Display for AssetLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to load '{}': {}", self.path, self.message)
    }
}

impl std::error::Error for AssetLoadError {}
