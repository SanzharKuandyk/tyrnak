use core_net::ClientId;
use core_proto::{Command, GameEvent, NetSnapshot};

/// Messages sent into the simulation thread.
#[derive(Debug, Clone)]
pub enum SimInboxMsg {
    RemoteCommand {
        client_id: ClientId,
        command: Command,
    },
}

/// Messages emitted by the simulation thread.
#[derive(Debug, Clone)]
pub enum SimOutboxMsg {
    Events(Vec<GameEvent>),
}

/// Messages sent into the network thread.
#[derive(Debug, Clone)]
pub enum NetInboxMsg {
    Snapshot(NetSnapshot),
}

/// Messages emitted by the network thread.
#[derive(Debug, Clone)]
pub enum NetOutboxMsg {
    Connected {
        client_id: ClientId,
    },
    Disconnected {
        client_id: ClientId,
    },
    Command {
        client_id: ClientId,
        command: Command,
    },
}
