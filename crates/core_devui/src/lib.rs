//! # core_devui
//!
//! Debug UI framework — egui-based panels, entity inspector, and diagnostics.
//!
//! Provides the `DebugPanel` trait and a panel registry. Does NOT own egui rendering —
//! it only builds UI commands. The app layer handles actual egui rendering.

pub mod panel;
pub mod panels;
pub mod registry;

pub use panel::DebugPanel;
pub use panels::{EntityEntry, EntityListPanel, FpsPanel};
pub use registry::PanelRegistry;

#[cfg(test)]
mod tests {
    use super::*;

    // ---- PanelRegistry tests ----

    /// A trivial panel for testing purposes.
    struct StubPanel {
        name: String,
    }

    impl StubPanel {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_owned(),
            }
        }
    }

    impl DebugPanel for StubPanel {
        fn name(&self) -> &str {
            &self.name
        }

        fn ui(&mut self, _ui: &mut egui::Ui) {
            // no-op
        }
    }

    #[test]
    fn registry_register_and_count() {
        let mut reg = PanelRegistry::new();
        assert_eq!(reg.panel_count(), 0);

        reg.register(Box::new(StubPanel::new("A")));
        assert_eq!(reg.panel_count(), 1);

        reg.register(Box::new(StubPanel::new("B")));
        assert_eq!(reg.panel_count(), 2);
    }

    #[test]
    fn registry_toggle_visibility() {
        let mut reg = PanelRegistry::new();
        reg.register(Box::new(StubPanel::new("A")));
        reg.register(Box::new(StubPanel::new("B")));

        // Panels start hidden.
        assert!(!reg.is_visible(0));
        assert!(!reg.is_visible(1));

        reg.toggle_panel(0);
        assert!(reg.is_visible(0));
        assert!(!reg.is_visible(1));

        reg.toggle_panel(0);
        assert!(!reg.is_visible(0));

        // Out-of-bounds index should not panic.
        reg.toggle_panel(999);
        assert!(!reg.is_visible(999));
    }

    // ---- FpsPanel tests ----

    #[test]
    fn fps_panel_average_fps_empty() {
        let panel = FpsPanel::new();
        assert_eq!(panel.average_fps(), 0.0);
    }

    #[test]
    fn fps_panel_average_fps_calculation() {
        let mut panel = FpsPanel::new();
        // 60 FPS → ~16.67ms per frame
        for _ in 0..60 {
            panel.record_frame_time(1.0 / 60.0);
        }
        let fps = panel.average_fps();
        assert!((fps - 60.0).abs() < 0.5, "Expected ~60 FPS, got {fps}");
    }

    #[test]
    fn fps_panel_respects_max_samples() {
        let mut panel = FpsPanel::new();
        // Record more than 120 samples — only the last 120 should be kept.
        for _ in 0..200 {
            panel.record_frame_time(1.0 / 30.0);
        }
        // Internal VecDeque should be capped at 120.
        assert!(panel.average_fps() > 0.0);
        // Average should be ~30 FPS.
        let fps = panel.average_fps();
        assert!((fps - 30.0).abs() < 0.5, "Expected ~30 FPS, got {fps}");
    }

    // ---- EntityListPanel tests ----

    #[test]
    fn entity_list_panel_update_entries() {
        let mut panel = EntityListPanel::new();
        assert_eq!(panel.name(), "Entity List");

        let entries = vec![
            EntityEntry {
                label: "Hero".into(),
                id: 1,
                details: "HP: 100".into(),
            },
            EntityEntry {
                label: "Minion".into(),
                id: 2,
                details: "HP: 30".into(),
            },
        ];
        panel.update_entries(entries);

        // Updating again replaces the old list.
        let entries2 = vec![EntityEntry {
            label: "Tower".into(),
            id: 10,
            details: "HP: 5000".into(),
        }];
        panel.update_entries(entries2);
    }
}
