//! # app_server
//!
//! Headless server executable — runs simulation + networking with no renderer.
//!
//! Phase 2 wires the new runtime host into explicit services:
//! `SimulationService` and `NetworkService`.

mod bootstrap;
mod brokers;
mod config;
mod console;
mod inspector;
mod messages;
mod services;
mod supervisor;
mod validation;

use crate::bootstrap::bootstrap_server;
use crate::config::ServerConfig;
use tracing_subscriber::prelude::*;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (trace_store, inspect_layer) =
        core_inspect::build_trace_layer(core_inspect::TraceConfig::default());
    let _ = core_inspect::install_global_trace_store(trace_store.clone());
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(tracing_subscriber::EnvFilter::new("info")))
        .with(inspect_layer)
        .init();

    let startup_span = tracing::info_span!("app_server_main");
    let _entered = startup_span.enter();

    let _inspector_thread = tool_inspector::launch_trace_window(trace_store.clone());

    let config = ServerConfig::default();
    let mut boot = bootstrap_server(&config)?;

    boot.supervisor.run(&mut boot.runtime)?;

    boot.runtime.request_shutdown()?;
    drop(boot.console_thread);
    boot.runtime.join_all()?;
    boot.dataflow_probe_thread.shutdown();
    let trace_snapshot = trace_store.snapshot();
    info!(
        spans = trace_snapshot.spans.len(),
        events = trace_snapshot.events.len(),
        threads = trace_snapshot.threads.len(),
        dataflows = trace_snapshot.dataflows.len(),
        "captured in-process trace snapshot"
    );
    info!("server runtime stopped");
    Ok(())
}
