# Eryndor Game Content Editor

A web-based game content editor for the Eryndor MMORPG, built with Bevy 0.17 and bevy_egui 0.38. The editor runs as a WASM application in the browser and communicates with the game server via HTTP API.

## Purpose

The editor provides a comprehensive interface for game designers and developers to create and modify all game content without touching code. This includes:

- **Items** - Weapons, armor, consumables with stat bonuses
- **Abilities** - Combat skills with composable effect types
- **NPCs** - Quest givers, trainers, merchants
- **Quests** - Multi-objective quests with rewards and requirements
- **Enemies** - Monster definitions with stats and loot tables
- **Loot Tables** - Drop tables for enemies and chests
- **Zones** - World areas with spawn points

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Eryndor Editor (WASM)                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Items     │  │  Abilities  │  │    NPCs     │   ...   │
│  │   Module    │  │   Module    │  │   Module    │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
│                          │                                  │
│                  ┌───────────────┐                          │
│                  │ Editor State  │                          │
│                  │   (Bevy Res)  │                          │
│                  └───────────────┘                          │
│                          │                                  │
│                  ┌───────────────┐                          │
│                  │  API Events   │                          │
│                  │   (Messages)  │                          │
│                  └───────────────┘                          │
│                          │                                  │
│                  ┌───────────────┐                          │
│                  │  API Client   │                          │
│                  │   (HTTP)      │                          │
│                  └───────────────┘                          │
└──────────────────────────┼──────────────────────────────────┘
                           │ HTTP REST API
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                   Eryndor Server                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                   Editor API                         │   │
│  │   GET/POST/PUT/DELETE /api/editor/{content_type}    │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                  │
│                          ▼                                  │
│            assets/content/{type}/*.json                     │
└─────────────────────────────────────────────────────────────┘
```

## Content Types

### Items
Items define equippable gear with stat bonuses:
- **Types**: Weapon, Helmet, Chest, Legs, Boots, Consumable, QuestItem
- **Stats**: attack_power, defense, max_health, max_mana, crit_chance
- **Grants Ability**: Weapons can grant abilities (e.g., Sword grants Heavy Slash)

### Abilities
Abilities are composable skills with multiple effect types:

**Effect Types:**
- `DirectDamage` - Immediate damage with multiplier
- `DamageOverTime` - Poison/burn effects with duration, ticks, damage_per_tick
- `AreaOfEffect` - AoE with radius and max_targets
- `Buff` - Temporary stat increases (attack_power, defense, move_speed)
- `Debuff` - Status effects (Stun, Root, Slow, Weaken)
- `Mobility` - Dashes/blinks with distance and speed
- `Heal` - HP restoration (flat or percentage)

**Unlock Requirements:**
- `None` - Starting ability
- `Level(n)` - Unlocks at level n
- `Quest(id)` - Unlocks after completing quest
- `WeaponProficiency(weapon, level)` - Requires weapon mastery

### NPCs
Non-player characters with roles:
- **QuestGiver** - Offers quests to players
- **Trainer** - Sells items to players

Each NPC has:
- Position (x, y coordinates)
- Visual appearance (shape, color, size)
- Role-specific data (quests list or items for sale)

### Quests
Quest definitions with:
- **Objectives**: TalkToNpc, KillEnemy (count), ObtainItem (count)
- **Rewards**: Experience points, ability unlocks
- **Requirements**: Weapon/armor proficiency levels

### Enemies
Monster definitions with:
- Combat stats (health, attack, defense, move_speed)
- Visual appearance
- Loot table references

## Running the Editor

### Prerequisites
- Rust with WASM target: `rustup target add wasm32-unknown-unknown`
- Trunk: `cargo install trunk`
- Running Eryndor server on http://127.0.0.1:8080

### Build and Run

```bash
# Start the server first
cargo run -p eryndor_server

# In another terminal, start the editor
cd crates/eryndor_editor
trunk serve --port 4000
```

Or use Bevy CLI with WASM:
```bash
bevy run --no-default-features -p eryndor_editor web
```

Then open http://127.0.0.1:4000 in your browser.

## File Structure

```
crates/eryndor_editor/
├── src/
│   ├── main.rs           # Application entry, Bevy setup, action processing
│   ├── editor_state.rs   # Central state management (EditorState resource)
│   ├── api_client.rs     # HTTP client for server communication
│   ├── api_events.rs     # Message types for async operations
│   ├── ui/
│   │   └── mod.rs        # Main menu bar and tab switching
│   └── modules/
│       ├── items.rs      # Items editor UI
│       ├── abilities.rs  # Abilities editor UI with effect builder
│       ├── npcs.rs       # NPCs editor UI (quest givers, trainers)
│       ├── quests.rs     # Quests editor UI with objectives
│       ├── enemies.rs    # Enemies editor UI
│       ├── loot.rs       # Loot tables editor UI
│       └── zones.rs      # Zones/world editor UI
├── Trunk.toml            # WASM build configuration
└── index.html            # HTML shell for WASM
```

## Content File Format

All content is stored as JSON files in `assets/content/{type}/`:

```
assets/content/
├── abilities/
│   ├── fireball.ability.json
│   ├── heavy_slash.ability.json
│   └── ...
├── items/
│   ├── sword.item.json
│   └── ...
├── npcs/
│   ├── village_elder.npc.json
│   └── ...
├── quests/
│   ├── first_steps.quest.json
│   └── ...
├── enemies/
│   ├── slime.enemy.json
│   └── ...
└── zones/
    └── starter_zone.zone.json
```

Files use descriptive names (not numeric IDs) for easier management.

## Message System

The editor uses Bevy's `bevy_message` crate for async communication:

```rust
// Define a message
#[derive(Message)]
pub struct LoadAbilityListEvent;

// Write messages to trigger actions
load_ability_events.write(LoadAbilityListEvent);

// Read messages in systems
for event in load_ability_events.read() {
    // Process event
}
```

## Adding New Content Types

1. Add data structures in `editor_state.rs`:
   ```rust
   pub struct EditingMyContent { ... }
   pub struct MyContentEditorState { ... }
   ```

2. Add API events in `api_events.rs`:
   ```rust
   #[derive(Message)]
   pub struct LoadMyContentListEvent;
   ```

3. Create UI module in `modules/my_content.rs`

4. Register module in `ui/mod.rs` tab bar

5. Add action processing in `main.rs`

## License

Part of the Eryndor MMORPG project.
