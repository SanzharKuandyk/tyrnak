# Tyrnak Codebase Overview

> Stubs and module-declaration-only files omitted. Only files with real logic listed.

---

## app_server

| File | Description |
|------|-------------|
| `app_server/src/main.rs` | Entry point; initializes tracing, builds runtime, starts supervisor loop |
| `app_server/src/bootstrap.rs` | Assembles server from config; spawns simulation + network services |
| `app_server/src/brokers.rs` | Creates message brokers and dataflow probes for inter-service channels |
| `app_server/src/config.rs` | `ServerConfig` with runtime, simulation, and network sub-configs; defaults |
| `app_server/src/console.rs` | Parses stdin commands (quit/pause/resume); spawns reader thread |
| `app_server/src/inspector.rs` | Dataflow probe thread; records and logs channel statistics |
| `app_server/src/messages.rs` | `SimInboxMsg`, `SimOutboxMsg`, `NetInboxMsg`, `NetOutboxMsg` enums |
| `app_server/src/validation.rs` | Pre-startup config validation (tick rate, intervals, channel topology) |
| `app_server/src/services/network_service.rs` | `EngineService` impl; polls network, routes client events and messages |
| `app_server/src/services/simulation_service.rs` | `EngineService` impl; runs fixed tick loop, processes commands, extracts snapshots |
| `app_server/src/supervisor.rs` | Supervisor loop; monitors console commands, service health, publishes summaries |

---

## core_assets

| File | Description |
|------|-------------|
| `core_assets/src/lib.rs` | `AssetStore`, `AssetHandle`, `AssetLoader` trait; integration tests |
| `core_assets/src/handle.rs` | `AssetId`, `AssetIdGen`, `AssetHandle<T>`, `AssetStatus` types |
| `core_assets/src/loader.rs` | `AssetLoader` trait: `extensions()` + `load()`; `AssetLoadError` |
| `core_assets/src/store.rs` | `HashMap`-backed store; path-to-id mapping; lifecycle management |
| `core_assets/src/watcher.rs` | `FileWatcher` via `notify` crate; wraps Create/Modify/Remove events |

---

## core_audio

| File | Description |
|------|-------------|
| `core_audio/src/engine.rs` | `AudioEngine` trait: `play_sound`, `stop_music`, `set_listener_position`, etc. |
| `core_audio/src/stub.rs` | `StubAudioEngine` no-op impl; logs via `tracing::debug`; tracks ids and volumes |

---

## core_devui

| File | Description |
|------|-------------|
| `core_devui/src/lib.rs` | `DebugPanel` trait; `PanelRegistry`; tests |
| `core_devui/src/panel.rs` | `DebugPanel` trait definition (egui-based) |
| `core_devui/src/panels/entity_list.rs` | `EntityListPanel`; scrollable entity inspector with ID, label, details |
| `core_devui/src/panels/fps.rs` | `FpsPanel`; rolling frame-time tracking; ASCII sparkline; average FPS |
| `core_devui/src/registry.rs` | `PanelRegistry`; owns all panels, manages visibility, draws menu + windows |

---

## core_inspect

| File | Description |
|------|-------------|
| `core_inspect/src/lib.rs` | Tracing system; captures spans, events, stack samples; hotspot stats; JSON export |

---

## core_math

| File | Description |
|------|-------------|
| `core_math/src/lib.rs` | Re-exports `glam` types + all primitives/transforms/utils; integration tests |
| `core_math/src/primitives.rs` | `AABB`, `Circle`, `Rect`, `Ray`; containment and intersection tests |
| `core_math/src/transform.rs` | `SimTransform` (yaw + uniform scale), `RenderTransform` (quat + per-axis scale); conversion |
| `core_math/src/utils.rs` | `lerp`, `inverse_lerp`, `remap`, `normalize_angle`, `angle_lerp`, `clamp01` |

---

## core_net

| File | Description |
|------|-------------|
| `core_net/src/lib.rs` | Networking facade; `renet` abstraction; serialization integration tests |
| `core_net/src/channel.rs` | `Delivery` enum, `ChannelSpec`, `ChannelProfile`; `default_moba_channel_profile()` |
| `core_net/src/types.rs` | `ClientId` re-export; `NetEvent` enum (Connected/Disconnected) |
| `core_net/src/client.rs` | `NetClient`; postcard serialization; send/receive typed messages |
| `core_net/src/server.rs` | `NetServer`; client event polling; broadcast and unicast message sending |

---

## core_platform

| File | Description |
|------|-------------|
| `core_platform/src/config.rs` | `PlatformConfig`; window builder; vsync and resizable options |
| `core_platform/src/events.rs` | `PlatformEvent` enum (WindowResized, KeyInput, MouseMoved, Scroll) |
| `core_platform/src/input.rs` | `InputState`; per-frame pressed/held/released tracking; mouse position + scroll delta |

---

## core_proto

| File | Description |
|------|-------------|
| `core_proto/src/lib.rs` | Data contract façade; serialization tests for commands, events, snapshots |
| `core_proto/src/commands.rs` | `Command` enum: MoveTo, Stop, CastAbility, BuyItem, AttackTarget |
| `core_proto/src/ids.rs` | `EntityId`, `AbilityId`, `ItemId`, `PlayerId`, `Target`; generational indexing |
| `core_proto/src/events.rs` | `GameEvent` enum: UnitMoved, DamageApplied, UnitDied, AbilityCast, Projectile, etc. |
| `core_proto/src/snapshot.rs` | `VisualType`, `EntitySnapshot`, `RenderSnapshot`, `NetSnapshot` |

---

## core_render

| File | Description |
|------|-------------|
| `core_render/src/backend.rs` | `RenderBackend` trait: `begin_frame`, `render_snapshot`, `end_frame`, `resize`, `size` |
| `core_render/src/camera.rs` | `Camera2D` orthographic; screen↔world conversion; view-projection matrix; zoom + pan |
| `core_render/src/null_renderer.rs` | No-op renderer for headless tests; tracks frame count; trace logging |
| `core_render/src/wgpu_renderer.rs` | Full `wgpu` renderer; shader compilation, pipelines, vertex buffers, uniform management |

---

## core_runtime

| File | Description |
|------|-------------|
| `core_runtime/src/broker.rs` | `kanal` wrapper: `BrokerTx`, `BrokerRx`, `BrokerPair`; send/recv/drain/try_send |
| `core_runtime/src/service.rs` | `EngineService` trait; `ServiceContext` for service↔runtime communication |
| `core_runtime/src/control.rs` | `ControlMsg` enum (Start/Pause/Resume/Stop/Shutdown); `ServiceFault` with severity |
| `core_runtime/src/health.rs` | `ServiceState`, `ServiceHealth`, `ServiceStatusSnapshot`, `RuntimeSnapshot`, `RuntimeSummary` |
| `core_runtime/src/runtime_host.rs` | `RuntimeHost`; coordinates services; `spawn_service`, `broadcast_control`, `join_all` |
| `core_runtime/src/lifecycle.rs` | `AppLifecycle` state machine (Init→Running→Paused→ShuttingDown); transition guards; tests |
| `core_runtime/src/tick_loop.rs` | Fixed-rate tick loop; `tick_count`, `tick_interval`, `fixed_dt()` |
| `core_runtime/src/dataflow/command_queue.rs` | `CommandQueue`; queued command delivery; queue + overflow policies |
| `core_runtime/src/dataflow/event_stream.rs` | `EventStream`; multi-consumer event delivery semantics |
| `core_runtime/src/dataflow/latest_value.rs` | `LatestValue`; overwrites-on-new-write channel (last-write-wins) |
| `core_runtime/src/dataflow/stats.rs` | `ChannelStats`; tracks sent/dropped/received counts per channel |

---

## game_core

| File | Description |
|------|-------------|
| `game_core/src/lib.rs` | ECS simulation entry; world + tick pipeline + commands + events; integration test |
| `game_core/src/world.rs` | `World`; owns `EntityAllocator` + all `ComponentStore`s; spawn/despawn |
| `game_core/src/entity.rs` | `EntityId` generational index; `EntityAllocator` with free-list reuse; tests |
| `game_core/src/component.rs` | `ComponentStore<T>` sparse-array storage; component types (Position, Health, Velocity, …) |
| `game_core/src/collision.rs` | Collision queries and raycast operations |
| `game_core/src/snapshot.rs` | Snapshot extraction for rendering and networking |
| `game_core/src/tick.rs` | `TickEngine`; orchestrates all systems in order; manages tick count |
| `game_core/src/effects/registry.rs` | `EffectRegistry`; registers and looks up effect processors |
| `game_core/src/effects/processor.rs` | `EffectProcessor` trait; `EffectInstance`, `StackPolicy`, `EffectKind` |
| `game_core/src/rules/ability_registry.rs` | `AbilityRegistry`; registers ability definitions |
| `game_core/src/rules/damage_rule.rs` | `DamageRule`; pluggable damage calculation |
| `game_core/src/rules/item_registry.rs` | `ItemRegistry`; item stat modifiers and passive effects |
| `game_core/src/rules/simulation_registry.rs` | `SimulationRegistry`; aggregates all rule sub-registries |
| `game_core/src/systems/movement.rs` | `MovementSystem`; applies velocity to position each tick |
| `game_core/src/systems/combat.rs` | `CombatSystem`; attack resolution and targeting logic |
| `game_core/src/systems/damage.rs` | `DamageSystem`; applies damage events, reduces health |
| `game_core/src/systems/ability.rs` | `AbilitySystem`; processes ability cast commands |
| `game_core/src/systems/death.rs` | `DeathSystem`; detects zero-health entities, queues despawn |
| `game_core/src/systems/effect.rs` | `EffectSystem`; ticks active effects, applies modifiers |
| `game_core/src/systems/item.rs` | `ItemSystem`; processes item purchase commands, applies stat changes |
| `game_core/src/state/tick_queues.rs` | `TickQueues`; `SpawnRequest`/`DespawnRequest` deferred operation queues |

---

## game_moba

| File | Description |
|------|-------------|
| `game_moba/src/lib.rs` | `bootstrap_sandbox_match()`; creates 2-unit test scenario (hero + enemy) |
| `game_moba/src/registry.rs` | `moba_simulation_registry()`; wires MOBA-specific rules and effects |
| `game_moba/src/abilities/` | MOBA ability definitions and registration |
| `game_moba/src/items/` | MOBA item definitions and stat registry |
| `game_moba/src/effects/` | MOBA status effects: Slow, Stun, Silence with `EffectProcessor` impls |

---

## tool_inspector

| File | Description |
|------|-------------|
| `tool_inspector/src/main.rs` | Standalone trace-visualization tool; reads `core_inspect` snapshots |

---

## Reading Plan

Work from lowest-level building blocks up to full server bootstrap. Each layer depends on the previous.

### Layer 0 — Primitives & IDs
1. `core_math/src/primitives.rs` — spatial types used everywhere
2. `core_math/src/transform.rs` — sim vs render transform distinction
3. `core_math/src/utils.rs` — math helpers
4. `core_proto/src/ids.rs` — generational IDs that flow through every system
5. `core_proto/src/commands.rs` — input side of the simulation
6. `core_proto/src/events.rs` — output side of the simulation
7. `core_proto/src/snapshot.rs` — render/net data contract

### Layer 1 — Runtime Infrastructure
8. `core_runtime/src/broker.rs` — the channel primitive everything uses
9. `core_runtime/src/dataflow/command_queue.rs` — how commands flow in
10. `core_runtime/src/dataflow/event_stream.rs` — how events flow out
11. `core_runtime/src/dataflow/latest_value.rs` — snapshot delivery
12. `core_runtime/src/control.rs` — service lifecycle signals
13. `core_runtime/src/health.rs` — service health types
14. `core_runtime/src/service.rs` — `EngineService` contract
15. `core_runtime/src/lifecycle.rs` — app state machine
16. `core_runtime/src/tick_loop.rs` — fixed-rate tick mechanics
17. `core_runtime/src/runtime_host.rs` — host that owns all services

### Layer 2 — ECS Core
18. `game_core/src/entity.rs` — generational entity allocator
19. `game_core/src/component.rs` — component stores + component types
20. `game_core/src/world.rs` — world: entity + component glue
21. `game_core/src/state/tick_queues.rs` — deferred spawn/despawn
22. `game_core/src/snapshot.rs` — extract world state for render/net
23. `game_core/src/collision.rs` — spatial queries

### Layer 3 — Game Systems
24. `game_core/src/effects/processor.rs` — effect trait + kinds
25. `game_core/src/effects/registry.rs` — effect registration
26. `game_core/src/rules/damage_rule.rs` — pluggable damage
27. `game_core/src/rules/ability_registry.rs`
28. `game_core/src/rules/item_registry.rs`
29. `game_core/src/rules/simulation_registry.rs` — aggregated registries
30. `game_core/src/systems/movement.rs`
31. `game_core/src/systems/combat.rs`
32. `game_core/src/systems/damage.rs`
33. `game_core/src/systems/ability.rs`
34. `game_core/src/systems/effect.rs`
35. `game_core/src/systems/death.rs`
36. `game_core/src/systems/item.rs`
37. `game_core/src/tick.rs` — how systems are ordered and executed
38. `game_core/src/lib.rs` — full simulation integration test

### Layer 4 — Networking
39. `core_net/src/channel.rs` — channel profiles and delivery semantics
40. `core_net/src/types.rs` — `NetEvent`
41. `core_net/src/server.rs`
42. `core_net/src/client.rs`
43. `core_net/src/lib.rs` — serialization tests confirm the contract

### Layer 5 — MOBA Instantiation
44. `game_moba/src/effects/` — concrete effect impls
45. `game_moba/src/abilities/`
46. `game_moba/src/items/`
47. `game_moba/src/registry.rs`
48. `game_moba/src/lib.rs` — sandbox match shows full wiring

### Layer 6 — Server Assembly
49. `app_server/src/messages.rs` — inter-service message types
50. `app_server/src/config.rs`
51. `app_server/src/validation.rs` — understand constraints before reading bootstrap
52. `app_server/src/brokers.rs`
53. `app_server/src/services/simulation_service.rs`
54. `app_server/src/services/network_service.rs`
55. `app_server/src/supervisor.rs`
56. `app_server/src/bootstrap.rs`
57. `app_server/src/console.rs`
58. `app_server/src/inspector.rs`
59. `app_server/src/main.rs` — everything comes together here

### Layer 7 — Observability & Render
60. `core_inspect/src/lib.rs` — trace capture and hotspot analysis
61. `core_render/src/backend.rs` — render trait
62. `core_render/src/camera.rs` — math-heavy; read primitives first
63. `core_render/src/null_renderer.rs` — headless path
64. `core_render/src/wgpu_renderer.rs` — GPU path; heaviest file

### Layer 8 — Supporting Systems
65. `core_assets/src/handle.rs` → `store.rs` → `loader.rs` → `watcher.rs`
66. `core_audio/src/engine.rs` → `stub.rs`
67. `core_platform/src/events.rs` → `input.rs` → `config.rs`
68. `core_devui/src/panel.rs` → `registry.rs` → `panels/fps.rs` → `panels/entity_list.rs`
