use std::time::Duration;

/// A configurable fixed-rate loop that maintains consistent timing.
pub struct TickLoop {
    tick_rate_hz: f64,
    tick_interval: Duration,
    tick_count: u64,
}

impl TickLoop {
    /// Create a new `TickLoop` with the given tick rate in hertz.
    pub fn new(tick_rate_hz: f64) -> Self {
        let tick_interval = Duration::from_secs_f64(1.0 / tick_rate_hz);
        Self {
            tick_rate_hz,
            tick_interval,
            tick_count: 0,
        }
    }

    /// Returns the configured tick rate in hertz.
    pub fn tick_rate(&self) -> f64 {
        self.tick_rate_hz
    }

    /// Returns the interval between ticks.
    pub fn tick_interval(&self) -> Duration {
        self.tick_interval
    }

    /// Returns the number of ticks that have elapsed.
    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }

    /// Increments the tick counter by one.
    pub fn increment(&mut self) {
        self.tick_count += 1;
    }

    /// Returns the fixed delta time as `1.0 / tick_rate` in f32.
    pub fn fixed_dt(&self) -> f32 {
        (1.0 / self.tick_rate_hz) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_count_increments() {
        let mut tick_loop = TickLoop::new(60.0);
        assert_eq!(tick_loop.tick_count(), 0);
        tick_loop.increment();
        assert_eq!(tick_loop.tick_count(), 1);
        tick_loop.increment();
        tick_loop.increment();
        assert_eq!(tick_loop.tick_count(), 3);
    }

    #[test]
    fn fixed_dt_60hz() {
        let tick_loop = TickLoop::new(60.0);
        let expected = 1.0_f32 / 60.0;
        assert!((tick_loop.fixed_dt() - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn fixed_dt_30hz() {
        let tick_loop = TickLoop::new(30.0);
        let expected = 1.0_f32 / 30.0;
        assert!((tick_loop.fixed_dt() - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn fixed_dt_128hz() {
        let tick_loop = TickLoop::new(128.0);
        let expected = 1.0_f32 / 128.0;
        assert!((tick_loop.fixed_dt() - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_interval_matches_rate() {
        let tick_loop = TickLoop::new(60.0);
        let expected = Duration::from_secs_f64(1.0 / 60.0);
        assert_eq!(tick_loop.tick_interval(), expected);
    }

    #[test]
    fn tick_rate_returns_configured_value() {
        let tick_loop = TickLoop::new(144.0);
        assert!((tick_loop.tick_rate() - 144.0).abs() < f64::EPSILON);
    }
}
