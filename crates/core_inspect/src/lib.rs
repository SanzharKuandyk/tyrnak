use std::collections::{HashMap, VecDeque};
use std::fs;
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadSnapshot {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanSnapshot {
    pub trace_id: u64,
    pub parent_trace_id: Option<u64>,
    pub thread_id: String,
    pub thread_name: Option<String>,
    pub name: String,
    pub target: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub started_at_micros: u64,
    pub ended_at_micros: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSnapshot {
    pub thread_id: String,
    pub thread_name: Option<String>,
    pub name: String,
    pub target: String,
    pub timestamp_micros: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackSample {
    pub thread_id: String,
    pub thread_name: Option<String>,
    pub label: String,
    pub timestamp_micros: u64,
    pub backtrace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSnapshot {
    pub captured_at_micros: u64,
    pub threads: Vec<ThreadSnapshot>,
    pub spans: Vec<SpanSnapshot>,
    pub events: Vec<EventSnapshot>,
    pub stack_samples: Vec<StackSample>,
    pub dataflows: Vec<DataflowSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotStat {
    pub target: String,
    pub function: String,
    pub call_count: u64,
    pub total_duration_micros: u64,
    pub max_duration_micros: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceView {
    pub name: String,
    pub thread_id: String,
    pub thread_name: Option<String>,
    pub span_count: usize,
    pub active_span_count: usize,
    pub last_span_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataflowSnapshot {
    pub name: String,
    pub kind: String,
    pub sent: u64,
    pub received: u64,
    pub overwritten: u64,
    pub rejected: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedTraceSnapshot {
    pub trace: TraceSnapshot,
    pub hotspots: Vec<HotspotStat>,
    pub services: Vec<ServiceView>,
    pub active_stacks: HashMap<String, Vec<SpanSnapshot>>,
}

#[derive(Debug, Clone, Copy)]
pub struct TraceConfig {
    pub max_spans: usize,
    pub max_events: usize,
    pub max_stack_samples: usize,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            max_spans: 8_192,
            max_events: 8_192,
            max_stack_samples: 512,
        }
    }
}

#[derive(Clone)]
pub struct TraceStore {
    inner: Arc<Mutex<TraceStoreInner>>,
}

impl fmt::Debug for TraceStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TraceStore").finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct TraceStoreInner {
    config: TraceConfig,
    started_at: Instant,
    next_trace_id: u64,
    spans_by_span_id: HashMap<u64, u64>,
    spans: VecDeque<SpanSnapshot>,
    events: VecDeque<EventSnapshot>,
    stack_samples: VecDeque<StackSample>,
    dataflows: HashMap<String, DataflowSnapshot>,
    threads: HashMap<String, ThreadSnapshot>,
}

#[derive(Debug, Clone, Copy)]
struct TraceId(u64);

static GLOBAL_TRACE_STORE: OnceLock<TraceStore> = OnceLock::new();

pub fn build_trace_layer(config: TraceConfig) -> (TraceStore, InspectLayer) {
    let store = TraceStore::new(config);
    let layer = InspectLayer {
        store: store.clone(),
    };
    (store, layer)
}

pub fn install_global_trace_store(store: TraceStore) -> Result<(), TraceStore> {
    GLOBAL_TRACE_STORE.set(store)
}

pub fn global_trace_store() -> Option<TraceStore> {
    GLOBAL_TRACE_STORE.get().cloned()
}

pub fn capture_stack_sample(label: &'static str) {
    let Some(store) = global_trace_store() else {
        return;
    };
    store.capture_stack_sample(label);
}

impl TraceStore {
    pub fn new(config: TraceConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(TraceStoreInner {
                config,
                started_at: Instant::now(),
                next_trace_id: 1,
                spans_by_span_id: HashMap::new(),
                spans: VecDeque::new(),
                events: VecDeque::new(),
                stack_samples: VecDeque::new(),
                dataflows: HashMap::new(),
                threads: HashMap::new(),
            })),
        }
    }

    pub fn snapshot(&self) -> TraceSnapshot {
        let inner = self.inner.lock().expect("trace store poisoned");
        TraceSnapshot {
            captured_at_micros: inner.started_at.elapsed().as_micros() as u64,
            threads: inner.threads.values().cloned().collect(),
            spans: inner.spans.iter().cloned().collect(),
            events: inner.events.iter().cloned().collect(),
            stack_samples: inner.stack_samples.iter().cloned().collect(),
            dataflows: {
                let mut values: Vec<_> = inner.dataflows.values().cloned().collect();
                values.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.kind.cmp(&b.kind)));
                values
            },
        }
    }

    pub fn derived_snapshot(&self) -> DerivedTraceSnapshot {
        let trace = self.snapshot();
        derive_snapshot(trace)
    }

    pub fn export_snapshot_json<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), std::io::Error> {
        let snapshot = self.derived_snapshot();
        let json = serde_json::to_string_pretty(&snapshot)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        fs::write(path, json)
    }

    pub fn capture_stack_sample(&self, label: &str) {
        let (thread_id, thread_name) = current_thread();
        let mut inner = self.inner.lock().expect("trace store poisoned");
        let max_samples = inner.config.max_stack_samples;
        let timestamp_micros = inner.started_at.elapsed().as_micros() as u64;
        inner
            .threads
            .entry(thread_id.clone())
            .or_insert(ThreadSnapshot {
                id: thread_id.clone(),
                name: thread_name.clone(),
            });
        push_bounded(
            &mut inner.stack_samples,
            max_samples,
            StackSample {
                thread_id,
                thread_name,
                label: label.to_string(),
                timestamp_micros,
                backtrace: std::backtrace::Backtrace::force_capture().to_string(),
            },
        );
    }

    pub fn record_dataflow(&self, snapshot: DataflowSnapshot) {
        let mut inner = self.inner.lock().expect("trace store poisoned");
        inner.dataflows.insert(snapshot.name.clone(), snapshot);
    }
}

pub fn import_snapshot_json<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<DerivedTraceSnapshot, std::io::Error> {
    let json = fs::read_to_string(path)?;
    serde_json::from_str(&json).map_err(|error| std::io::Error::other(error.to_string()))
}

pub fn record_dataflow(snapshot: DataflowSnapshot) {
    let Some(store) = global_trace_store() else {
        return;
    };
    store.record_dataflow(snapshot);
}

pub struct InspectLayer {
    store: TraceStore,
}

impl<S> Layer<S> for InspectLayer
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let metadata = attrs.metadata();
        let parent_trace_id = resolve_parent_trace_id(&ctx, attrs.parent());
        let (thread_id, thread_name) = current_thread();

        let mut inner = self.store.inner.lock().expect("trace store poisoned");
        let max_spans = inner.config.max_spans;
        let started_at_micros = inner.started_at.elapsed().as_micros() as u64;
        inner
            .threads
            .entry(thread_id.clone())
            .or_insert(ThreadSnapshot {
                id: thread_id.clone(),
                name: thread_name.clone(),
            });

        let trace_id = inner.next_trace_id;
        inner.next_trace_id += 1;
        inner.spans_by_span_id.insert(id.into_u64(), trace_id);

        push_bounded(
            &mut inner.spans,
            max_spans,
            SpanSnapshot {
                trace_id,
                parent_trace_id,
                thread_id,
                thread_name,
                name: metadata.name().to_string(),
                target: metadata.target().to_string(),
                file: metadata.file().map(str::to_owned),
                line: metadata.line(),
                started_at_micros,
                ended_at_micros: None,
            },
        );

        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(TraceId(trace_id));
        }
    }

    fn on_record(&self, _span: &Id, _values: &Record<'_>, _ctx: Context<'_, S>) {}

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let (thread_id, thread_name) = current_thread();

        let mut inner = self.store.inner.lock().expect("trace store poisoned");
        let max_events = inner.config.max_events;
        let timestamp_micros = inner.started_at.elapsed().as_micros() as u64;
        inner
            .threads
            .entry(thread_id.clone())
            .or_insert(ThreadSnapshot {
                id: thread_id.clone(),
                name: thread_name.clone(),
            });

        push_bounded(
            &mut inner.events,
            max_events,
            EventSnapshot {
                thread_id,
                thread_name,
                name: metadata.name().to_string(),
                target: metadata.target().to_string(),
                timestamp_micros,
            },
        );
    }

    fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
        let mut inner = self.store.inner.lock().expect("trace store poisoned");
        let Some(trace_id) = inner.spans_by_span_id.remove(&id.into_u64()) else {
            return;
        };
        let ended_at_micros = inner.started_at.elapsed().as_micros() as u64;
        if let Some(span) = inner
            .spans
            .iter_mut()
            .rev()
            .find(|span| span.trace_id == trace_id)
        {
            span.ended_at_micros = Some(ended_at_micros);
        }
    }
}

fn resolve_parent_trace_id<S>(ctx: &Context<'_, S>, explicit_parent: Option<&Id>) -> Option<u64>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let current_parent = ctx.lookup_current().map(|span| span.id());
    let parent_id = explicit_parent.map(ToOwned::to_owned).or(current_parent)?;
    let span = ctx.span(&parent_id)?;
    span.extensions()
        .get::<TraceId>()
        .map(|trace_id| trace_id.0)
}

fn current_thread() -> (String, Option<String>) {
    let thread = std::thread::current();
    (
        format!("{:?}", thread.id()),
        thread.name().map(str::to_owned),
    )
}

fn derive_snapshot(trace: TraceSnapshot) -> DerivedTraceSnapshot {
    let mut hotspot_map: HashMap<(String, String), HotspotStat> = HashMap::new();
    let mut active_stacks: HashMap<String, Vec<SpanSnapshot>> = HashMap::new();
    let mut spans_by_thread: HashMap<String, Vec<&SpanSnapshot>> = HashMap::new();

    for span in &trace.spans {
        let duration = span
            .ended_at_micros
            .unwrap_or(trace.captured_at_micros)
            .saturating_sub(span.started_at_micros);
        let key = (span.target.clone(), span.name.clone());
        let entry = hotspot_map.entry(key.clone()).or_insert(HotspotStat {
            target: key.0.clone(),
            function: key.1.clone(),
            call_count: 0,
            total_duration_micros: 0,
            max_duration_micros: 0,
        });
        entry.call_count += 1;
        entry.total_duration_micros += duration;
        entry.max_duration_micros = entry.max_duration_micros.max(duration);

        spans_by_thread
            .entry(span.thread_id.clone())
            .or_default()
            .push(span);
        if span.ended_at_micros.is_none() {
            active_stacks
                .entry(span.thread_id.clone())
                .or_default()
                .push(span.clone());
        }
    }

    for spans in active_stacks.values_mut() {
        spans.sort_by_key(|span| span.started_at_micros);
    }

    let mut services = Vec::new();
    for thread in &trace.threads {
        let thread_spans = spans_by_thread.get(&thread.id).cloned().unwrap_or_default();
        let service_name = thread
            .name
            .as_deref()
            .and_then(|name| name.strip_prefix("service-"))
            .map(str::to_owned)
            .unwrap_or_else(|| thread.name.clone().unwrap_or_else(|| thread.id.clone()));
        let active_span_count = active_stacks
            .get(&thread.id)
            .map(|spans| spans.len())
            .unwrap_or(0);
        let last_span_name = thread_spans
            .iter()
            .max_by_key(|span| span.started_at_micros)
            .map(|span| format!("{}::{}", span.target, span.name));
        services.push(ServiceView {
            name: service_name,
            thread_id: thread.id.clone(),
            thread_name: thread.name.clone(),
            span_count: thread_spans.len(),
            active_span_count,
            last_span_name,
        });
    }
    services.sort_by(|a, b| a.name.cmp(&b.name));

    let mut hotspots: Vec<_> = hotspot_map.into_values().collect();
    hotspots.sort_by(|a, b| {
        b.total_duration_micros
            .cmp(&a.total_duration_micros)
            .then_with(|| b.call_count.cmp(&a.call_count))
    });

    DerivedTraceSnapshot {
        trace,
        hotspots,
        services,
        active_stacks,
    }
}

fn push_bounded<T>(buffer: &mut VecDeque<T>, max_len: usize, value: T) {
    if buffer.len() >= max_len {
        buffer.pop_front();
    }
    buffer.push_back(value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::info_span;
    use tracing_subscriber::prelude::*;

    #[test]
    fn captures_nested_spans() {
        let (store, layer) = build_trace_layer(TraceConfig::default());
        let subscriber = tracing_subscriber::registry().with(layer);
        tracing::subscriber::with_default(subscriber, || {
            let root = info_span!("root");
            let _entered = root.enter();
            let child = info_span!("child");
            let _child_entered = child.enter();
            tracing::info!("event");
        });

        let snapshot = store.snapshot();
        assert!(snapshot.spans.iter().any(|span| span.name == "root"));
        assert!(snapshot.spans.iter().any(|span| span.name == "child"));
        assert!(!snapshot.events.is_empty());
        let child = snapshot
            .spans
            .iter()
            .find(|span| span.name == "child")
            .expect("child span");
        assert!(child.parent_trace_id.is_some());
    }

    #[test]
    fn records_named_dataflows() {
        let store = TraceStore::new(TraceConfig::default());
        store.record_dataflow(DataflowSnapshot {
            name: "simulation -> network latest snapshot".to_string(),
            kind: "latest_value".to_string(),
            sent: 7,
            received: 5,
            overwritten: 2,
            rejected: 0,
        });

        let snapshot = store.snapshot();
        assert_eq!(snapshot.dataflows.len(), 1);
        assert_eq!(snapshot.dataflows[0].name, "simulation -> network latest snapshot");
        assert_eq!(snapshot.dataflows[0].overwritten, 2);
    }
}
