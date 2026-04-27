# Architecture

This document describes the target system structure. It is an architecture reference, not a rollout plan.

## Principles

- Authoritative simulation
- Clear module boundaries
- Plain data at module boundaries
- Message-passing across threads
- Rendering, networking, and platform kept outside simulation
- Game rules kept outside generic engine support

## Layer Model

```text
app_* / tool_*
  -> game_*
  -> core_runtime, core_net, core_render, core_platform, core_assets,
     core_audio, core_devui, core_inspect

game_moba
  -> game_core
  -> core_proto
  -> core_math

game_core
  -> core_proto
  -> core_math
  -> core_inspect

core_net
  -> core_proto
  -> core_runtime

core_render
  -> core_proto
  -> core_math
```

Dependency direction is one-way:

```text
core_*   <- reusable engine support
  ^
  |
game_*   <- game rules and content
  ^
  |
app_*    <- composition roots and executables
```

`core_*` never depends on `game_*`. `game_*` never depends on `app_*`.

## Runtime Topology

Long-lived subsystems are services. Each service owns its loop and communicates through named brokers rather than direct thread-to-thread coupling.

```text
RuntimeHost
  -> ControlBroker -> SimulationService
  -> ControlBroker -> NetworkService
  -> ControlBroker -> RenderService
  -> ControlBroker -> InputService

InputService
  -> CommandBroker -> SimulationService

NetworkService
  -> CommandBroker -> SimulationService
  <- SnapshotBroker <- SimulationService
  <- EventBroker <- SimulationService

RenderService
  <- SnapshotBroker <- SimulationService
  <- EventBroker <- SimulationService
```

Server shape:

```text
Remote Clients
  -> NetworkService
  -> CommandBroker
  -> SimulationService
  -> SnapshotBroker
  -> NetworkService
  -> Remote Clients
```

Client shape:

```text
Platform/Input
  -> InputService
  -> CommandBroker
  -> SimulationService or remote server

SimulationService or remote snapshots
  -> SnapshotBroker
  -> RenderService
  -> wgpu
```

## Services And Brokers

### Runtime host

Owns:
- service startup and shutdown
- named service threads
- control-plane senders
- fault aggregation
- clean join on exit

### Services

Each service owns:
- one subsystem loop
- one execution context
- control message handling
- health and fault reporting

Lifecycle vocabulary:
- `Created`
- `Starting`
- `Running`
- `Paused`
- `Stopping`
- `Stopped`
- `Faulted`

Control vocabulary:
- `Start`
- `Pause`
- `Resume`
- `Stop`
- `Shutdown`

### Brokers

- `ControlBroker`: runtime host to services
- `CommandBroker`: input and network command ingress to simulation
- `SnapshotBroker`: simulation snapshots to network and rendering
- `EventBroker`: simulation events to observers, tooling, audio, and UI

Broker payloads are plain data. Architectural names matter more than channel implementation details.

## Simulation Core

`game_core` is the authoritative simulation boundary.

Responsibilities:
- world state
- entities and component storage
- command ingestion
- fixed-tick execution
- combat, movement, abilities, items, buffs, deaths, vision
- event production
- snapshot extraction for rendering, networking, and replay

Constraints:
- no GPU APIs
- no windowing or OS APIs
- no transport-specific networking
- no direct ownership of rendering state

Tick shape:

```text
commands
  -> movement
  -> abilities
  -> projectiles
  -> combat
  -> buffs and effects
  -> deaths and spawns
  -> vision
  -> events + snapshots
```

Data model:
- entity IDs and typed handles
- flat storage / data-oriented layout
- explicit event log
- explicit command buffer
- snapshots as read-only extracted data

## Data Contracts

Cross-module APIs stay small and data-oriented.

```text
GameAPI
  - submit_commands(...)
  - tick(...)
  - render_snapshot() -> RenderSnapshot
  - net_snapshot() -> NetSnapshot

RenderAPI
  - render(snapshot)

NetAPI
  - send_commands(...)
  - receive_snapshot() -> NetSnapshot
```

Boundary types are plain structs, IDs, arrays, buffers, and message enums. Internal engine types do not leak across crate boundaries.

## Rendering, Networking, Assets

### Rendering

`core_render` consumes `RenderSnapshot` and owns:
- visibility consumption
- batching
- GPU resource management
- command encoding
- presentation

CPU to GPU flow:

```text
simulation -> render snapshot -> batching -> GPU upload -> draw -> present
```

### Networking

`core_net` implements a server-authoritative model:

```text
client commands -> server simulation -> snapshots/events -> clients
```

The network layer transports commands, snapshots, and connection events. It does not own game rules.

### Assets

`core_assets` owns:
- asset handles
- source loading and decoding
- cooked/runtime resource preparation
- caches
- hot-reload plumbing

## Crate Roles

```text
app_client       window, input, render wiring, net client, dev UI
app_server       headless runtime composition and net server
tool_inspector   runtime inspection UI

game_core        deterministic simulation and snapshots
game_moba        MOBA-specific content and rules

core_runtime     service model, brokers, host, lifecycle
core_proto       shared commands, events, snapshots, IDs
core_net         networking transport and protocol wiring
core_render      rendering backend and render API
core_platform    platform and window integration
core_assets      assets and resource pipeline
core_audio       audio integration
core_devui       debug and developer UI
core_inspect     tracing and runtime inspection support
core_math        shared math types and helpers
core_physics     collision and physics support
```

## Non-Negotiable Rules

- Simulation never touches GPU APIs.
- Renderer never owns authoritative game state.
- Networking never owns game logic.
- App crates wire systems together but do not absorb engine logic.
- Cross-thread communication goes through brokers.
- Cross-crate boundaries expose data contracts, not internal implementation types.
- Game-specific concepts stay in `game_*`, not `core_*`.
