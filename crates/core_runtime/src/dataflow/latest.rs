use std::sync::{Arc, Mutex};

use crate::dataflow::policy::ChannelStats;

pub struct LatestValue<T> {
    pub writer: LatestWriter<T>,
    pub reader: LatestReader<T>,
}

struct LatestInner<T> {
    value: Option<T>,
    stats: ChannelStats,
}

pub struct LatestWriter<T> {
    inner: Arc<Mutex<LatestInner<T>>>,
}

pub struct LatestReader<T> {
    inner: Arc<Mutex<LatestInner<T>>>,
}

impl<T> LatestValue<T> {
    pub fn new() -> Self {
        let inner = Arc::new(Mutex::new(LatestInner {
            value: None,
            stats: ChannelStats::default(),
        }));
        Self {
            writer: LatestWriter {
                inner: inner.clone(),
            },
            reader: LatestReader { inner },
        }
    }
}

impl<T> Clone for LatestWriter<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Clone for LatestReader<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> LatestWriter<T> {
    pub fn write(&self, value: T) {
        let mut inner = self.inner.lock().expect("latest value poisoned");
        if inner.value.is_some() {
            inner.stats.overwritten += 1;
        }
        inner.value = Some(value);
        inner.stats.sent += 1;
    }

    pub fn stats(&self) -> ChannelStats {
        self.inner.lock().expect("latest value poisoned").stats
    }
}

impl<T: Clone> LatestReader<T> {
    pub fn read(&self) -> Option<T> {
        let mut inner = self.inner.lock().expect("latest value poisoned");
        let value = inner.value.clone();
        if value.is_some() {
            inner.stats.received += 1;
        }
        value
    }

    pub fn take(&self) -> Option<T> {
        let mut inner = self.inner.lock().expect("latest value poisoned");
        let value = inner.value.take();
        if value.is_some() {
            inner.stats.received += 1;
        }
        value
    }

    pub fn stats(&self) -> ChannelStats {
        self.inner.lock().expect("latest value poisoned").stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_value_overwrites_old_value() {
        let latest = LatestValue::new();
        latest.writer.write(1);
        latest.writer.write(2);
        assert_eq!(latest.reader.take(), Some(2));
        let stats = latest.reader.stats();
        assert_eq!(stats.sent, 2);
        assert_eq!(stats.overwritten, 1);
        assert_eq!(stats.received, 1);
    }
}
