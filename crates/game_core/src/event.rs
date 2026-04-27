//! Event log for simulation-produced events.
//!
//! Each tick the simulation emits [`GameEvent`]s (movement, damage, death, etc.)
//! into this log. Consumers (renderer, network, replay) read the events after
//! the tick completes, then the log is cleared for the next tick.

use core_proto::GameEvent;

/// Accumulates [`GameEvent`]s produced during a single tick.
pub struct EventLog {
    events: Vec<GameEvent>,
}

impl EventLog {
    /// Create an empty log.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Record an event.
    pub fn push(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    /// Drain all events, returning them and leaving the log empty.
    pub fn drain(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.events)
    }

    /// Borrow the current event slice without consuming it.
    pub fn events(&self) -> &[GameEvent] {
        &self.events
    }

    /// Number of events in the log.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Discard all events.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    fn proto_id(index: u32) -> core_proto::EntityId {
        core_proto::EntityId::new(index, 0)
    }

    #[test]
    fn log_push_and_events() {
        let mut log = EventLog::new();
        log.push(GameEvent::UnitSpawned {
            entity: proto_id(0),
            position: Vec3::ZERO,
        });
        assert_eq!(log.len(), 1);
        assert!(!log.is_empty());
        assert_eq!(log.events().len(), 1);
    }

    #[test]
    fn log_drain() {
        let mut log = EventLog::new();
        log.push(GameEvent::UnitSpawned {
            entity: proto_id(0),
            position: Vec3::ZERO,
        });
        let events = log.drain();
        assert_eq!(events.len(), 1);
        assert!(log.is_empty());
    }

    #[test]
    fn log_clear() {
        let mut log = EventLog::new();
        log.push(GameEvent::UnitSpawned {
            entity: proto_id(0),
            position: Vec3::ZERO,
        });
        log.push(GameEvent::UnitSpawned {
            entity: proto_id(1),
            position: Vec3::ONE,
        });
        log.clear();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn log_empty_by_default() {
        let log = EventLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
        assert!(log.events().is_empty());
    }
}
