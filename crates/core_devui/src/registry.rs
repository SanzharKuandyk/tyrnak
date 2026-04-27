use crate::panel::DebugPanel;

/// Central registry that owns all debug panels and manages their visibility.
pub struct PanelRegistry {
    panels: Vec<Box<dyn DebugPanel>>,
    visible: Vec<bool>,
}

impl PanelRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            panels: Vec::new(),
            visible: Vec::new(),
        }
    }

    /// Registers a new debug panel. It starts hidden by default.
    pub fn register(&mut self, panel: Box<dyn DebugPanel>) {
        self.panels.push(panel);
        self.visible.push(false);
    }

    /// Returns the number of registered panels.
    pub fn panel_count(&self) -> usize {
        self.panels.len()
    }

    /// Toggles the visibility of the panel at `index`.
    pub fn toggle_panel(&mut self, index: usize) {
        if let Some(v) = self.visible.get_mut(index) {
            *v = !*v;
        }
    }

    /// Returns whether the panel at `index` is currently visible.
    pub fn is_visible(&self, index: usize) -> bool {
        self.visible.get(index).copied().unwrap_or(false)
    }

    /// Draws a menu with checkboxes to toggle panel visibility.
    pub fn draw_menu(&mut self, ui: &mut egui::Ui) {
        for i in 0..self.panels.len() {
            let name = self.panels[i].name().to_owned();
            let mut open = self.visible[i];
            if ui.checkbox(&mut open, &name).changed() {
                self.visible[i] = open;
            }
        }
    }

    /// Draws all visible panels as `egui::Window`s.
    pub fn draw_panels(&mut self, ctx: &egui::Context) {
        for i in 0..self.panels.len() {
            if !self.visible[i] {
                continue;
            }
            let name = self.panels[i].name().to_owned();
            let mut open = self.visible[i];
            egui::Window::new(&name)
                .open(&mut open)
                .show(ctx, |ui| {
                    self.panels[i].ui(ui);
                });
            self.visible[i] = open;
        }
    }
}

impl Default for PanelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
