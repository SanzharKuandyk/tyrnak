use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use core_inspect::DataflowSnapshot;
use core_runtime::ChannelStats;

pub type StatsSampler = Arc<dyn Fn() -> ChannelStats + Send + Sync>;

pub struct DataflowProbe {
    pub name: &'static str,
    pub kind: &'static str,
    pub sample: StatsSampler,
}

pub struct DataflowProbeThread {
    stop: Arc<AtomicBool>,
    join: JoinHandle<()>,
}

impl DataflowProbeThread {
    pub fn shutdown(self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = self.join.join();
    }
}

pub fn launch_dataflow_probe_thread(
    probes: Vec<DataflowProbe>,
    poll_interval: Duration,
) -> DataflowProbeThread {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_flag = stop.clone();
    let join = thread::Builder::new()
        .name("inspector-dataflow".to_string())
        .spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                for probe in &probes {
                    let stats = (probe.sample)();
                    core_inspect::record_dataflow(DataflowSnapshot {
                        name: probe.name.to_string(),
                        kind: probe.kind.to_string(),
                        sent: stats.sent,
                        received: stats.received,
                        overwritten: stats.overwritten,
                        rejected: stats.rejected,
                    });
                }
                thread::sleep(poll_interval);
            }
        })
        .expect("failed to spawn dataflow probe thread");

    DataflowProbeThread { stop, join }
}
