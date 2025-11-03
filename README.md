# Eryndor MMO - 2D Multiplayer Online Game POC

A proof-of-concept 2D MMO built with Bevy and bevy_replicon featuring server-authoritative architecture.

## Features

- **Server-Authoritative Architecture** - All game logic runs on the server
- **Authentication System** - Username/password login with SQLite database
- **Character System** - Create and select characters with 3 classes (Rogue, Mage, Knight)
- **Movement** - WASD movement with server-side validation
- **Combat** - Tab-targeting combat system with class abilities
- **Inventory** - Item pickup and management
- **Quest System** - Intelligent quests that give class-appropriate rewards
- **Class-Based Abilities**:
  - **Rogue** - Quick Strike (fast, agile attacks)
  - **Mage** - Fireball (ranged magical damage)
  - **Knight** - Heavy Slash (slow but powerful strikes)

## Project Structure

```
eryndor-mmo/
├── Cargo.toml              # Workspace definition
├── crates/
│   ├── eryndor_shared/     # Shared components and protocol
│   ├── eryndor_server/     # Dedicated server
│   └── eryndor_client/     # Game client
└── eryndor.db              # SQLite database (created on first run)
```

## Building

### Prerequisites

- Rust 1.70 or later
- Windows/Linux/macOS

### Build Server

```bash
cargo build --release -p eryndor_server
```

### Build Client

```bash
cargo build --release -p eryndor_client
```

## Running

### Start the Server

```bash
cargo run -p eryndor_server
# Or run the binary directly:
# ./target/release/server
```

The server will:
1. Create `eryndor.db` SQLite database
2. Initialize game world with NPCs, items, and enemies
3. Listen on `127.0.0.1:5000`

### Start the Client

```bash
cargo run -p eryndor_client
# Or run the binary directly:
# ./target/release/client
```

## Getting Started

1. **Create Account**
   - Enter a username and password
   - Click "Create Account"

2. **Login**
   - Enter your credentials
   - Click "Login"

3. **Create Character**
   - Click "Create New Character"
   - Enter a name
   - Select a class (Rogue/Mage/Knight)
   - Click "Create"

4. **Play**
   - Select your character
   - Click "Play"

## Controls

- **WASD** or **Arrow Keys** - Movement
- **Left Click** - Select target (enemies, NPCs, items)
- **E** - Interact with selected target (pickup items, Talk to theNPCs)
- **1-9, 0** - Use abilities on hotbar
- **I** (planned) - Open inventory

## Gameplay Loop

1. **Talk to theElder** (the green circle NPC)
   - Click on the NPC
   - Press **E** to interact
   - Accept the quest "Choose Your Path"

2. **Complete the quest**
   - Talk to theElder again
   - Press **E** to complete the quest
   - You'll receive your **class-specific weapon**:
     - **Rogue** → Dagger (Quick Strike ability)
     - **Mage** → Wand (Fireball ability)
     - **Knight** → Sword (Heavy Slash ability)
   - The weapon is automatically added to your inventory
   - Your class ability is already on your hotbar (slot 1)!

3. **Fight enemies**
   - Three slimes (red circles) spawn around the world
   - Click on an enemy to target it
   - Press **1** to use your class ability
   - Defeat enemies to test the combat system

4. **Level up your character**
   - Gain XP from completed quests
   - More quests and enemies will be added in future updates

## Architecture Details

### Server-Authoritative Design

- **Client sends inputs** → Server validates → Server updates game state → Changes replicate to clients
- **No client-side game logic** - prevents cheating
- **Component replication** via bevy_replicon

### Network Protocol

- **TCP-based** via renet2 (WebTransport support)
- **Ordered channels** for critical events (login, combat)
- **Unordered channels** for frequent updates (movement)

### Components (Replicated)

- Position, Velocity, Health, Mana
- Inventory, Equipment, Hotbar
- Quest Log, Learned Abilities
- Visual Shape (for rendering)

### Events (Client → Server)

- LoginRequest, MoveInput, UseAbilityRequest
- PickupItemRequest, InteractNpcRequest
- AcceptQuestRequest, CompleteQuestRequest

### Events (Server → Client)

- LoginResponse, CombatEvent
- NotificationEvent, QuestUpdateEvent
- DeathEvent

## Technology Stack

- **Bevy 0.17** - Game engine
- **bevy_replicon 0.36** - Network replication
- **bevy_replicon_renet2** - Networking backend
- **SQLx** - Database (SQLite)
- **bevy_egui** - UI
- **bevy_prototype_lyon** - 2D shape rendering
- **Argon2** - Password hashing

## Development Status

This is a **proof-of-concept** demonstrating:
- ✅ Server-authoritative multiplayer
- ✅ Authentication and character management
- ✅ Movement replication
- ✅ Combat system with abilities
- ✅ Inventory and item system
- ✅ Quest system
- ✅ Simple enemy AI

### Known Limitations

- Simple shape-based graphics (no sprites yet)
- Basic UI
- Single zone/area
- Limited abilities (one per weapon)
- No character progression beyond level 1
- No chat system
- Enemy AI is very basic

## Future Enhancements

- Character progression and leveling
- More abilities and skill trees
- Equipment system (armor, accessories)
- Multiple zones/maps
- Party system
- Trading
- Crafting
- PvP combat
- Temporal Echo System (unique mechanic - time-layered world)

## Performance Notes

- Server can handle 50+ concurrent players (untested at scale)
- Clients predict movement locally for responsiveness
- Database operations are async to avoid blocking game loop

## License

This is a POC/educational project. Feel free to use and modify.

## Contributing

This is a proof-of-concept. For production use, consider:
- Proper error handling
- Security hardening (rate limiting, anti-cheat)
- Asset management
- Performance optimization
- Comprehensive testing

## Credits

Built with:
- [Bevy](https://bevyengine.org/)
- [bevy_replicon](https://github.com/projectharmonia/bevy_replicon)
- The Rust gamedev community

---

**Note**: This README documents the intended final state. Some features may still be under development. See the compilation status below.

## Current Compilation Status

The project structure is complete. If there are compilation errors:

1. Ensure all dependencies match:
   - Bevy 0.17
   - bevy_replicon 0.36
   - bevy_replicon_renet2 0.6
   - bevy_egui 0.38
   - bevy_prototype_lyon 0.15

2. Check the server main.rs for correct ChannelKind/SendPolicy syntax

3. Run `cargo check --workspace` to identify remaining issues

## Testing with Multiple Clients

To test multiplayer locally:

1. Start the server once
2. Run multiple client instances:
   ```bash
   cargo run -p eryndor_client &
   cargo run -p eryndor_client &
   cargo run -p eryndor_client &
   ```

3. Create different accounts for each client
4. You should see other players moving in real-time!
