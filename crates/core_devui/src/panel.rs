/// Trait for debug panels that can be rendered in the dev UI.
pub trait DebugPanel {
    /// Returns the display name of this panel.
    fn name(&self) -> &str;

    /// Renders the panel contents into the given egui `Ui`.
    fn ui(&mut self, ui: &mut egui::Ui);
}
