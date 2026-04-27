//! Command buffer for batching player-issued commands.
//!
//! Commands arrive from the network or AI systems and are collected here
//! until the tick pipeline drains them for deterministic processing.

use core_proto::Command;

/// Collects [`Command`]s until the tick pipeline is ready to process them.
pub struct CommandBuffer {
    commands: Vec<Command>,
}

impl CommandBuffer {
    /// Create an empty buffer.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Enqueue a single command.
    pub fn push(&mut self, cmd: Command) {
        self.commands.push(cmd);
    }

    /// Drain all queued commands, returning them in insertion order.
    pub fn drain(&mut self) -> Vec<Command> {
        std::mem::take(&mut self.commands)
    }

    /// Number of queued commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for CommandBuffer {
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
    fn buffer_push_and_drain() {
        let mut buf = CommandBuffer::new();
        buf.push(Command::Stop {
            entity: proto_id(0),
        });
        buf.push(Command::MoveTo {
            entity: proto_id(1),
            position: Vec3::ONE,
        });
        assert_eq!(buf.len(), 2);
        assert!(!buf.is_empty());

        let cmds = buf.drain();
        assert_eq!(cmds.len(), 2);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn buffer_drain_empty() {
        let mut buf = CommandBuffer::new();
        let cmds = buf.drain();
        assert!(cmds.is_empty());
    }
}
