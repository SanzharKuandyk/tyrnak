use std::collections::VecDeque;

use crate::panel::DebugPanel;

/// Tracks frame times and displays FPS information.
pub struct FpsPanel {
    frame_times: VecDeque<f32>,
    max_samples: usize,
}

impl FpsPanel {
    /// Creates a new `FpsPanel` with a default of 120 samples.
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120),
            max_samples: 120,
        }
    }

    /// Records a frame time (in seconds).
    pub fn record_frame_time(&mut self, dt: f32) {
        if self.frame_times.len() >= self.max_samples {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(dt);
    }

    /// Returns the average FPS based on recorded frame times.
    pub fn average_fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.frame_times.iter().sum();
        let avg_dt = sum / self.frame_times.len() as f32;
        if avg_dt > 0.0 {
            1.0 / avg_dt
        } else {
            0.0
        }
    }
}

impl Default for FpsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugPanel for FpsPanel {
    fn name(&self) -> &str {
        "FPS"
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let fps = self.average_fps();
        ui.label(format!("Average FPS: {fps:.1}"));

        // Simple sparkline rendered as text — show the last few frame times as a bar chart.
        if !self.frame_times.is_empty() {
            let max_dt = self.frame_times.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let bar_chars = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
            let sparkline: String = self
                .frame_times
                .iter()
                .map(|&dt| {
                    let normalized = if max_dt > 0.0 { dt / max_dt } else { 0.0 };
                    let idx = (normalized * (bar_chars.len() - 1) as f32)
                        .round()
                        .clamp(0.0, (bar_chars.len() - 1) as f32) as usize;
                    bar_chars[idx]
                })
                .collect();
            ui.monospace(&sparkline);
        }
    }
}
