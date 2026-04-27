//! Platform configuration for window creation.

/// Configuration for creating the application window and platform layer.
#[derive(Debug, Clone)]
pub struct PlatformConfig {
    /// Window title.
    pub title: String,
    /// Initial window width in logical pixels.
    pub width: u32,
    /// Initial window height in logical pixels.
    pub height: u32,
    /// Whether to enable vsync (present mode).
    pub vsync: bool,
    /// Whether the window is resizable.
    pub resizable: bool,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            title: "Tyrnak Engine".to_string(),
            width: 1280,
            height: 720,
            vsync: true,
            resizable: true,
        }
    }
}

impl PlatformConfig {
    /// Create a new config with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set window dimensions.
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set vsync.
    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    /// Set resizable.
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
}
