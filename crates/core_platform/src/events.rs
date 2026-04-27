//! Platform events — engine-friendly wrappers over winit events.

use winit::keyboard::KeyCode;
use crate::input::MouseButton;

/// High-level platform events consumed by the engine.
#[derive(Debug, Clone)]
pub enum PlatformEvent {
    /// Window was resized to new logical dimensions.
    WindowResized { width: u32, height: u32 },
    /// Window gained or lost focus.
    Focused(bool),
    /// User requested window close (X button, Alt+F4, etc.).
    CloseRequested,
    /// Keyboard key state change.
    KeyInput {
        key: KeyCode,
        pressed: bool,
    },
    /// Mouse cursor moved to a new position (logical pixels).
    MouseMoved { x: f64, y: f64 },
    /// Mouse button state change.
    MouseInput {
        button: MouseButton,
        pressed: bool,
    },
    /// Mouse scroll (horizontal, vertical).
    Scroll { dx: f32, dy: f32 },
    /// The window's scale factor changed (e.g., moved to a different DPI monitor).
    ScaleFactorChanged { scale_factor: f64 },
    /// A new frame should be rendered (redraw requested).
    RedrawRequested,
}
