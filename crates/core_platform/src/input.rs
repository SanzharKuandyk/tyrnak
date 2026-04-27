//! Input state tracking — keyboard, mouse, and scroll.

use std::collections::HashSet;
use winit::keyboard::KeyCode;

/// Mouse button identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    /// Any other numbered button (winit Button6..Button32 mapped to index 6..32).
    Other(u8),
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(btn: winit::event::MouseButton) -> Self {
        use winit::event::MouseButton as WB;
        match btn {
            WB::Left => MouseButton::Left,
            WB::Right => MouseButton::Right,
            WB::Middle => MouseButton::Middle,
            WB::Back => MouseButton::Back,
            WB::Forward => MouseButton::Forward,
            other => {
                // winit 0.31 uses Button6..Button32 as individual variants.
                // Map them to a numeric index via their discriminant.
                let byte = other as u8;
                MouseButton::Other(byte)
            }
        }
    }
}

/// Snapshot of all input state for the current frame.
#[derive(Debug, Clone)]
pub struct InputState {
    /// Currently pressed keyboard keys.
    pub keys_held: HashSet<KeyCode>,
    /// Keys pressed this frame (just went down).
    pub keys_pressed: HashSet<KeyCode>,
    /// Keys released this frame (just went up).
    pub keys_released: HashSet<KeyCode>,
    /// Current mouse position in logical pixels (relative to window).
    pub mouse_position: (f64, f64),
    /// Mouse movement delta this frame.
    pub mouse_delta: (f64, f64),
    /// Currently held mouse buttons.
    pub mouse_buttons_held: HashSet<MouseButton>,
    /// Mouse buttons pressed this frame.
    pub mouse_buttons_pressed: HashSet<MouseButton>,
    /// Mouse buttons released this frame.
    pub mouse_buttons_released: HashSet<MouseButton>,
    /// Scroll delta this frame (horizontal, vertical).
    pub scroll_delta: (f32, f32),
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

impl InputState {
    /// Create a fresh input state with nothing pressed.
    pub fn new() -> Self {
        Self {
            keys_held: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            mouse_position: (0.0, 0.0),
            mouse_delta: (0.0, 0.0),
            mouse_buttons_held: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_buttons_released: HashSet::new(),
            scroll_delta: (0.0, 0.0),
        }
    }

    /// Clear per-frame transient state (pressed/released/deltas).
    /// Call this at the start of each frame before processing new events.
    pub fn begin_frame(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_buttons_pressed.clear();
        self.mouse_buttons_released.clear();
        self.mouse_delta = (0.0, 0.0);
        self.scroll_delta = (0.0, 0.0);
    }

    /// Record a key press.
    pub fn key_down(&mut self, key: KeyCode) {
        if self.keys_held.insert(key) {
            self.keys_pressed.insert(key);
        }
    }

    /// Record a key release.
    pub fn key_up(&mut self, key: KeyCode) {
        self.keys_held.remove(&key);
        self.keys_released.insert(key);
    }

    /// Record mouse movement.
    pub fn mouse_moved(&mut self, x: f64, y: f64) {
        let dx = x - self.mouse_position.0;
        let dy = y - self.mouse_position.1;
        self.mouse_delta.0 += dx;
        self.mouse_delta.1 += dy;
        self.mouse_position = (x, y);
    }

    /// Record a mouse button press.
    pub fn mouse_button_down(&mut self, button: MouseButton) {
        if self.mouse_buttons_held.insert(button) {
            self.mouse_buttons_pressed.insert(button);
        }
    }

    /// Record a mouse button release.
    pub fn mouse_button_up(&mut self, button: MouseButton) {
        self.mouse_buttons_held.remove(&button);
        self.mouse_buttons_released.insert(button);
    }

    /// Record scroll input.
    pub fn scroll(&mut self, dx: f32, dy: f32) {
        self.scroll_delta.0 += dx;
        self.scroll_delta.1 += dy;
    }

    /// Check if a key is currently held.
    pub fn is_key_held(&self, key: KeyCode) -> bool {
        self.keys_held.contains(&key)
    }

    /// Check if a key was just pressed this frame.
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// Check if a key was just released this frame.
    pub fn is_key_released(&self, key: KeyCode) -> bool {
        self.keys_released.contains(&key)
    }

    /// Check if a mouse button is currently held.
    pub fn is_mouse_held(&self, button: MouseButton) -> bool {
        self.mouse_buttons_held.contains(&button)
    }

    /// Check if a mouse button was just pressed this frame.
    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    /// Check if a mouse button was just released this frame.
    pub fn is_mouse_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons_released.contains(&button)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_press_and_hold() {
        let mut input = InputState::new();
        input.key_down(KeyCode::KeyW);
        assert!(input.is_key_pressed(KeyCode::KeyW));
        assert!(input.is_key_held(KeyCode::KeyW));

        // Next frame — still held but not "just pressed"
        input.begin_frame();
        assert!(!input.is_key_pressed(KeyCode::KeyW));
        assert!(input.is_key_held(KeyCode::KeyW));
    }

    #[test]
    fn key_release() {
        let mut input = InputState::new();
        input.key_down(KeyCode::KeyA);
        input.begin_frame();
        input.key_up(KeyCode::KeyA);
        assert!(input.is_key_released(KeyCode::KeyA));
        assert!(!input.is_key_held(KeyCode::KeyA));
    }

    #[test]
    fn mouse_movement_tracking() {
        let mut input = InputState::new();
        input.mouse_moved(100.0, 200.0);
        assert_eq!(input.mouse_position, (100.0, 200.0));
        assert_eq!(input.mouse_delta, (100.0, 200.0));

        input.begin_frame();
        input.mouse_moved(110.0, 205.0);
        assert_eq!(input.mouse_position, (110.0, 205.0));
        assert_eq!(input.mouse_delta, (10.0, 5.0));
    }

    #[test]
    fn mouse_button_press() {
        let mut input = InputState::new();
        input.mouse_button_down(MouseButton::Left);
        assert!(input.is_mouse_pressed(MouseButton::Left));
        assert!(input.is_mouse_held(MouseButton::Left));
        assert!(!input.is_mouse_pressed(MouseButton::Right));
    }

    #[test]
    fn scroll_accumulation() {
        let mut input = InputState::new();
        input.scroll(0.0, 1.0);
        input.scroll(0.0, 2.0);
        assert_eq!(input.scroll_delta, (0.0, 3.0));

        input.begin_frame();
        assert_eq!(input.scroll_delta, (0.0, 0.0));
    }

    #[test]
    fn duplicate_key_press_not_double_counted() {
        let mut input = InputState::new();
        input.key_down(KeyCode::Space);
        input.key_down(KeyCode::Space); // held, not new press
        assert!(input.is_key_pressed(KeyCode::Space));
        assert!(input.is_key_held(KeyCode::Space));
    }
}
