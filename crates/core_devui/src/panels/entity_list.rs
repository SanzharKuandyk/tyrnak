use crate::panel::DebugPanel;

/// A single entry in the entity list panel.
pub struct EntityEntry {
    pub label: String,
    pub id: u32,
    pub details: String,
}

/// Displays a scrollable list of entities with basic info.
pub struct EntityListPanel {
    entries: Vec<EntityEntry>,
}

impl EntityListPanel {
    /// Creates a new empty `EntityListPanel`.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Replaces the current entries with the provided list.
    pub fn update_entries(&mut self, entries: Vec<EntityEntry>) {
        self.entries = entries;
    }
}

impl Default for EntityListPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugPanel for EntityListPanel {
    fn name(&self) -> &str {
        "Entity List"
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        if self.entries.is_empty() {
            ui.label("No entities.");
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for entry in &self.entries {
                ui.group(|ui| {
                    ui.label(format!("[{}] {}", entry.id, entry.label));
                    ui.small(&entry.details);
                });
            }
        });
    }
}
