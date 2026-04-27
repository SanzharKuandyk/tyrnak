use std::collections::HashMap;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use core_inspect::{DataflowSnapshot, DerivedTraceSnapshot, SpanSnapshot, TraceStore};
use eframe::egui;

pub fn launch_trace_window(trace_store: TraceStore) -> JoinHandle<()> {
    thread::Builder::new()
        .name("inspector-window".to_string())
        .spawn(move || {
            let options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([1560.0, 960.0])
                    .with_min_inner_size([1080.0, 700.0])
                    .with_title("Tyrnak Inspector"),
                ..Default::default()
            };

            let app_factory = move |_cc: &eframe::CreationContext<'_>| {
                Ok::<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>>(Box::new(
                    TraceInspectorApp::new(trace_store.clone()),
                ))
            };

            if let Err(error) = eframe::run_native("Tyrnak Inspector", options, Box::new(app_factory))
            {
                tracing::error!(%error, "failed to launch inspector window");
            }
        })
        .expect("failed to spawn inspector window thread")
}

struct TraceInspectorApp {
    store: TraceStore,
    filters: InspectorFilters,
    export_path: PathBuf,
    import_path: PathBuf,
    last_status: Option<String>,
    imported_snapshot: Option<DerivedTraceSnapshot>,
}

#[derive(Default)]
struct InspectorFilters {
    thread_filter: String,
    module_filter: String,
    function_filter: String,
    selected_thread: Option<String>,
    active_only: bool,
}

impl TraceInspectorApp {
    fn new(store: TraceStore) -> Self {
        let default_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".ai")
            .join("trace_snapshot.json");
        Self {
            store,
            filters: InspectorFilters::default(),
            export_path: default_path.clone(),
            import_path: default_path,
            last_status: None,
            imported_snapshot: None,
        }
    }

    fn current_snapshot(&self) -> DerivedTraceSnapshot {
        self.imported_snapshot
            .clone()
            .unwrap_or_else(|| self.store.derived_snapshot())
    }
}

impl eframe::App for TraceInspectorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(200));
        let snapshot = self.current_snapshot();

        egui::TopBottomPanel::top("inspector_header")
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(17, 24, 39))
                    .inner_margin(egui::Margin::same(12)),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.heading(
                        egui::RichText::new("Tyrnak Inspector")
                            .size(24.0)
                            .color(egui::Color32::from_rgb(236, 253, 245)),
                    );
                    ui.separator();
                    stat_chip(
                        ui,
                        "Threads",
                        snapshot.trace.threads.len().to_string(),
                        [15, 118, 110],
                    );
                    stat_chip(
                        ui,
                        "Spans",
                        snapshot.trace.spans.len().to_string(),
                        [30, 64, 175],
                    );
                    stat_chip(
                        ui,
                        "Events",
                        snapshot.trace.events.len().to_string(),
                        [180, 83, 9],
                    );
                    stat_chip(
                        ui,
                        "Samples",
                        snapshot.trace.stack_samples.len().to_string(),
                        [126, 34, 206],
                    );
                    stat_chip(
                        ui,
                        "Channels",
                        snapshot.trace.dataflows.len().to_string(),
                        [2, 132, 199],
                    );
                    stat_chip(
                        ui,
                        "Captured",
                        format!("{} us", snapshot.trace.captured_at_micros),
                        [91, 33, 182],
                    );
                    if let Some(status) = &self.last_status {
                        ui.separator();
                        ui.label(egui::RichText::new(status).color(egui::Color32::LIGHT_GREEN));
                    }
                });
            });

        egui::SidePanel::left("filters_panel")
            .resizable(true)
            .default_width(320.0)
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(248, 250, 252))
                    .inner_margin(egui::Margin::same(10)),
            )
            .show(ctx, |ui| {
                ui.heading("Filters");
                ui.separator();
                ui.label("Thread");
                ui.text_edit_singleline(&mut self.filters.thread_filter);
                ui.label("Module");
                ui.text_edit_singleline(&mut self.filters.module_filter);
                ui.label("Function");
                ui.text_edit_singleline(&mut self.filters.function_filter);
                ui.checkbox(&mut self.filters.active_only, "Active spans only");
                ui.separator();
                ui.heading("Session");
                ui.label(format!("Export: {}", self.export_path.display()));
                if ui.button("Export JSON").clicked() {
                    match self.store.export_snapshot_json(&self.export_path) {
                        Ok(()) => {
                            self.last_status =
                                Some(format!("exported {}", self.export_path.display()));
                        }
                        Err(error) => {
                            self.last_status = Some(format!("export failed: {error}"));
                        }
                    }
                }
                ui.label(format!("Import: {}", self.import_path.display()));
                if ui.button("Load JSON").clicked() {
                    match core_inspect::import_snapshot_json(&self.import_path) {
                        Ok(imported) => {
                            self.imported_snapshot = Some(imported);
                            self.last_status =
                                Some(format!("loaded {}", self.import_path.display()));
                        }
                        Err(error) => {
                            self.last_status = Some(format!("import failed: {error}"));
                        }
                    }
                }
                if ui.button("Use Live Trace").clicked() {
                    self.imported_snapshot = None;
                    self.last_status = Some("switched to live trace".to_string());
                }
                ui.separator();
                draw_services_panel(&snapshot, &mut self.filters, ui);
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(241, 245, 249))
                    .inner_margin(egui::Margin::same(10)),
            )
            .show(ctx, |ui| {
                draw_thread_lanes(&snapshot, &self.filters, ui);
                ui.add_space(10.0);
                ui.columns(4, |columns| {
                    draw_active_stacks(&snapshot, &self.filters, &mut columns[0]);
                    draw_module_tree(&snapshot, &self.filters, &mut columns[1]);
                    draw_hotspots_and_samples(&snapshot, &self.filters, &mut columns[2]);
                    draw_dataflow_panel(&snapshot, &mut columns[3]);
                });
                ui.add_space(10.0);
                draw_event_list(&snapshot, &self.filters, ui);
            });
    }
}

fn stat_chip(ui: &mut egui::Ui, label: &str, value: String, color: [u8; 3]) {
    let fill = egui::Color32::from_rgb(color[0], color[1], color[2]);
    egui::Frame::default()
        .fill(fill)
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .color(egui::Color32::WHITE)
                        .strong(),
                );
                ui.label(egui::RichText::new(value).color(egui::Color32::WHITE));
            });
        });
}

fn draw_services_panel(
    snapshot: &DerivedTraceSnapshot,
    filters: &mut InspectorFilters,
    ui: &mut egui::Ui,
) {
    ui.heading("Services");
    ui.separator();
    for service in &snapshot.services {
        let selected = filters.selected_thread.as_ref() == Some(&service.thread_id);
        let label = format!(
            "{}  [{} spans | {} active]",
            service.name, service.span_count, service.active_span_count
        );
        if ui.selectable_label(selected, label).clicked() {
            filters.selected_thread = Some(service.thread_id.clone());
            filters.thread_filter = service
                .thread_name
                .clone()
                .unwrap_or_else(|| service.thread_id.clone());
        }
        if let Some(last_span) = &service.last_span_name {
            ui.small(format!("last: {last_span}"));
        }
        ui.add_space(4.0);
    }
    if ui.button("Clear Service Focus").clicked() {
        filters.selected_thread = None;
    }
}

fn draw_thread_lanes(snapshot: &DerivedTraceSnapshot, filters: &InspectorFilters, ui: &mut egui::Ui) {
    ui.heading("Thread Lanes");
    ui.separator();

    let total = snapshot.trace.captured_at_micros.max(1) as f32;
    egui::ScrollArea::horizontal().show(ui, |ui| {
        for thread in &snapshot.trace.threads {
            if !thread_matches(thread.name.as_deref().unwrap_or(&thread.id), filters) {
                continue;
            }
            if let Some(selected) = &filters.selected_thread
                && selected != &thread.id
            {
                continue;
            }

            let title = thread.name.as_deref().unwrap_or(&thread.id);
            ui.label(egui::RichText::new(title).strong());
            let available_width = ui.available_width().max(700.0);
            let lane_height = 28.0;
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(available_width, lane_height),
                egui::Sense::hover(),
            );
            ui.painter()
                .rect_filled(rect, 6.0, egui::Color32::from_rgb(226, 232, 240));

            for span in filtered_spans(&snapshot.trace.spans, filters)
                .filter(|span| span.thread_id == thread.id)
            {
                let start = rect.left() + (span.started_at_micros as f32 / total) * rect.width();
                let end_us = span
                    .ended_at_micros
                    .unwrap_or(snapshot.trace.captured_at_micros)
                    .max(span.started_at_micros + 1);
                let end = rect.left() + (end_us as f32 / total) * rect.width();
                let span_rect = egui::Rect::from_min_max(
                    egui::pos2(start, rect.top()),
                    egui::pos2(end.max(start + 2.0), rect.bottom()),
                );
                ui.painter()
                    .rect_filled(span_rect, 4.0, color_for_target(&span.target));
            }

            ui.add_space(8.0);
        }
    });
}

fn draw_active_stacks(
    snapshot: &DerivedTraceSnapshot,
    filters: &InspectorFilters,
    ui: &mut egui::Ui,
) {
    ui.heading("Active Stacks");
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for thread in &snapshot.trace.threads {
            if !thread_matches(thread.name.as_deref().unwrap_or(&thread.id), filters) {
                continue;
            }
            if let Some(selected) = &filters.selected_thread
                && selected != &thread.id
            {
                continue;
            }

            let active = snapshot
                .active_stacks
                .get(&thread.id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|span| span_matches(span, filters))
                .collect::<Vec<_>>();

            egui::Frame::group(ui.style())
                .fill(egui::Color32::WHITE)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(thread.name.as_deref().unwrap_or(&thread.id)).strong(),
                    );
                    if active.is_empty() {
                        ui.small("no active spans");
                    } else {
                        for span in active {
                            ui.label(format!("{}::{}", span.target, span.name));
                        }
                    }
                });
            ui.add_space(6.0);
        }
    });
}

fn draw_module_tree(snapshot: &DerivedTraceSnapshot, filters: &InspectorFilters, ui: &mut egui::Ui) {
    ui.heading("Modules & Functions");
    ui.separator();

    let mut children_by_parent: HashMap<Option<u64>, Vec<&SpanSnapshot>> = HashMap::new();
    for span in filtered_spans(&snapshot.trace.spans, filters) {
        children_by_parent
            .entry(span.parent_trace_id)
            .or_default()
            .push(span);
    }

    for spans in children_by_parent.values_mut() {
        spans.sort_by_key(|span| span.started_at_micros);
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        if let Some(roots) = children_by_parent.get(&None) {
            for root in roots {
                draw_span_node(ui, root, &children_by_parent, 0, snapshot.trace.captured_at_micros);
            }
        }
    });
}

fn draw_span_node(
    ui: &mut egui::Ui,
    span: &SpanSnapshot,
    children_by_parent: &HashMap<Option<u64>, Vec<&SpanSnapshot>>,
    depth: usize,
    trace_end_micros: u64,
) {
    let duration = span
        .ended_at_micros
        .unwrap_or(trace_end_micros)
        .saturating_sub(span.started_at_micros);

    ui.horizontal(|ui| {
        ui.add_space(depth as f32 * 18.0);
        ui.label(
            egui::RichText::new(format!("{}::{}", span.target, span.name))
                .strong()
                .color(egui::Color32::from_rgb(15, 23, 42)),
        );
        ui.small(format!("[{} us]", duration));
        ui.small(
            span.file
                .as_ref()
                .zip(span.line)
                .map(|(file, line)| format!("{file}:{line}"))
                .unwrap_or_else(|| span.thread_name.clone().unwrap_or_else(|| span.thread_id.clone())),
        );
    });

    if let Some(children) = children_by_parent.get(&Some(span.trace_id)) {
        for child in children {
            draw_span_node(ui, child, children_by_parent, depth + 1, trace_end_micros);
        }
    }
}

fn draw_hotspots_and_samples(
    snapshot: &DerivedTraceSnapshot,
    filters: &InspectorFilters,
    ui: &mut egui::Ui,
) {
    ui.heading("Hotspots");
    ui.separator();
    egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
        for hotspot in snapshot.hotspots.iter().filter(|hotspot| {
            (filters.module_filter.is_empty() || hotspot.target.contains(&filters.module_filter))
                && (filters.function_filter.is_empty()
                    || hotspot.function.contains(&filters.function_filter))
        }).take(50)
        {
            egui::Frame::group(ui.style()).fill(egui::Color32::WHITE).show(ui, |ui| {
                ui.label(
                    egui::RichText::new(format!("{}::{}", hotspot.target, hotspot.function))
                        .strong(),
                );
                ui.small(format!("calls: {}", hotspot.call_count));
                ui.small(format!("total: {} us", hotspot.total_duration_micros));
                ui.small(format!("max: {} us", hotspot.max_duration_micros));
            });
            ui.add_space(6.0);
        }
    });

    ui.add_space(10.0);
    ui.heading("Stack Samples");
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| {
        for sample in snapshot.trace.stack_samples.iter().rev().filter(|sample| {
            (filters.thread_filter.is_empty()
                || sample
                    .thread_name
                    .as_deref()
                    .unwrap_or(&sample.thread_id)
                    .contains(&filters.thread_filter))
                && (filters.function_filter.is_empty()
                    || sample.label.contains(&filters.function_filter))
        }).take(20)
        {
            egui::CollapsingHeader::new(format!(
                "{} [{} us]",
                sample.label, sample.timestamp_micros
            ))
            .default_open(false)
            .show(ui, |ui| {
                ui.small(format!(
                    "thread: {}",
                    sample.thread_name.as_deref().unwrap_or(&sample.thread_id)
                ));
                ui.separator();
                ui.monospace(&sample.backtrace);
            });
        }
    });
}

fn draw_dataflow_panel(snapshot: &DerivedTraceSnapshot, ui: &mut egui::Ui) {
    ui.heading("Channels");
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        if snapshot.trace.dataflows.is_empty() {
            ui.small("no channel samples recorded");
            return;
        }

        for channel in &snapshot.trace.dataflows {
            draw_dataflow_card(channel, ui);
            ui.add_space(6.0);
        }
    });
}

fn draw_dataflow_card(channel: &DataflowSnapshot, ui: &mut egui::Ui) {
    egui::Frame::group(ui.style())
        .fill(egui::Color32::WHITE)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(&channel.name)
                    .strong()
                    .color(egui::Color32::from_rgb(15, 23, 42)),
            );
            ui.small(format!("kind: {}", channel.kind));
            ui.separator();
            ui.small(format!("sent: {}", channel.sent));
            ui.small(format!("received: {}", channel.received));
            if channel.overwritten > 0 {
                ui.small(format!("overwritten: {}", channel.overwritten));
            }
            if channel.rejected > 0 {
                ui.small(format!("rejected: {}", channel.rejected));
            }
        });
}

fn draw_event_list(snapshot: &DerivedTraceSnapshot, filters: &InspectorFilters, ui: &mut egui::Ui) {
    ui.heading("Events");
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for event in snapshot.trace.events.iter().rev().filter(|event| {
            (filters.thread_filter.is_empty()
                || event
                    .thread_name
                    .as_deref()
                    .unwrap_or(&event.thread_id)
                    .contains(&filters.thread_filter))
                && (filters.module_filter.is_empty() || event.target.contains(&filters.module_filter))
                && (filters.function_filter.is_empty() || event.name.contains(&filters.function_filter))
        }).take(500)
        {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::WHITE)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&event.name)
                            .strong()
                            .color(egui::Color32::from_rgb(30, 41, 59)),
                    );
                    ui.small(format!("target: {}", event.target));
                    ui.small(format!(
                        "thread: {}",
                        event.thread_name.as_deref().unwrap_or(&event.thread_id)
                    ));
                    ui.small(format!("timestamp: {} us", event.timestamp_micros));
                });
            ui.add_space(6.0);
        }
    });
}

fn filtered_spans<'a>(
    spans: &'a [SpanSnapshot],
    filters: &'a InspectorFilters,
) -> impl Iterator<Item = &'a SpanSnapshot> {
    spans.iter().filter(move |span| span_matches(span, filters))
}

fn span_matches(span: &SpanSnapshot, filters: &InspectorFilters) -> bool {
    let thread_label = span.thread_name.as_deref().unwrap_or(&span.thread_id);
    if !filters.thread_filter.is_empty() && !thread_label.contains(&filters.thread_filter) {
        return false;
    }
    if let Some(selected) = &filters.selected_thread
        && &span.thread_id != selected
    {
        return false;
    }
    if !filters.module_filter.is_empty() && !span.target.contains(&filters.module_filter) {
        return false;
    }
    if !filters.function_filter.is_empty() && !span.name.contains(&filters.function_filter) {
        return false;
    }
    if filters.active_only && span.ended_at_micros.is_some() {
        return false;
    }
    true
}

fn thread_matches(thread_label: &str, filters: &InspectorFilters) -> bool {
    filters.thread_filter.is_empty() || thread_label.contains(&filters.thread_filter)
}

fn color_for_target(target: &str) -> egui::Color32 {
    let mut hash = 0u32;
    for byte in target.as_bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(*byte as u32);
    }
    let r = 60 + (hash & 0x7f) as u8;
    let g = 70 + ((hash >> 7) & 0x7f) as u8;
    let b = 80 + ((hash >> 14) & 0x7f) as u8;
    egui::Color32::from_rgb(r, g, b)
}
