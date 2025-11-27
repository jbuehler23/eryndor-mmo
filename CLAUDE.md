# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Eryndor MMO is a proof-of-concept 2D multiplayer online game built with Bevy 0.17 and bevy_replicon 0.36. It uses a server-authoritative architecture where all game logic runs on the server to prevent cheating.

## Build Commands

```bash
# Development (uses cargo aliases from .cargo/config.toml)
cargo server-dev      # Run server with debug symbols
cargo client-dev      # Run native client

# Release builds
cargo server          # Release server
cargo client          # Release client
cargo build-all       # Build all crates

# WASM client (requires: cargo install bevy_cli)
cd crates/eryndor_client && bevy build web     # Dev WASM build
cd crates/eryndor_client && bevy build --yes web  # CI/automated

# Checks
cargo check-all       # Check all workspace crates compile
cargo clippy --all-targets --all-features -- -D warnings
```

## Testing Multiplayer

Always test with multiple clients:
```bash
# Terminal 1: Server
cargo server-dev

# Terminal 2 & 3: Clients
cargo client-dev
cargo client-dev
```

Create different accounts for each client and verify players see each other.

## Architecture

### Workspace Structure
- `crates/eryndor_shared/` - Shared components (`components.rs`) and network protocol (`protocol.rs`)
- `crates/eryndor_server/` - Headless server with auth, combat, movement, inventory, quest systems
- `crates/eryndor_client/` - Native + WASM client with input, UI, rendering
- `crates/eryndor_editor/` - World editor web app

### Server-Authoritative Flow
1. Client sends input events (movement, ability use, interact)
2. Server validates and processes
3. Server updates game state
4. bevy_replicon replicates component changes to clients
5. Client renders and predicts movement locally

### Key Technologies
- **Bevy 0.17** - ECS game engine
- **bevy_replicon 0.36** - Component replication
- **bevy_renet2** - UDP/WebTransport/WebSocket transport
- **avian2d** - 2D physics
- **SQLite + sqlx** - Database persistence
- **bevy_egui** - UI

### Network Events
- Client → Server: `LoginRequest`, `MoveInput`, `UseAbilityRequest`, `PickupItemRequest`, `InteractNpcRequest`
- Server → Client: `LoginResponse`, `CombatEvent`, `NotificationEvent`, `QuestUpdateEvent`

## JSON Content System

Game content is data-driven in `assets/content/`:
```
assets/content/
├── abilities/   # *.ability.json
├── enemies/     # *.enemy.json
├── items/       # *.item.json
├── quests/      # *.quest.json
├── npcs/        # *.npc.json
└── zones/       # *.zone.json
```

Hot-reload enabled in dev mode via `file_watcher` feature - edit JSON and server reloads automatically.

## Configuration

- `config.toml` - Server configuration (port, admin, security, rate limits, oauth)
- `.env` - Environment variables (SERVER_PORT, JWT_SECRET, DATABASE_PATH, RUST_LOG)
- Database: `eryndor.db` SQLite file created on first server run

## Key Files for Common Tasks

- **Combat/damage**: `crates/eryndor_server/src/combat.rs`
- **Authentication**: `crates/eryndor_server/src/auth.rs`
- **Character management**: `crates/eryndor_server/src/character.rs`
- **Movement validation**: `crates/eryndor_server/src/movement.rs`
- **Client input**: `crates/eryndor_client/src/input.rs`
- **Client UI**: `crates/eryndor_client/src/ui.rs`
- **Shared components**: `crates/eryndor_shared/src/components.rs`
- **Network protocol**: `crates/eryndor_shared/src/protocol.rs`
- **World/NPC spawning**: `crates/eryndor_server/src/world.rs`

## Known Issues

See `TODO.md` for the full list. Critical:
- NPC interaction broken - targeting system doesn't select NPCs properly (needs `Interactable` component)

## Debugging

```bash
RUST_LOG=debug cargo server-dev
RUST_LOG=debug,eryndor_server=trace cargo server-dev
```
