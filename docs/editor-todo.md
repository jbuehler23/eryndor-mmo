# Bevy World Editor - Generic 2D Level Editor

## Overview

A **generic, schema-driven 2D level editor** for Bevy games - combining the best of Tiled and LDtk:

- **100% Schema-Driven** - Editor has ZERO hardcoded game concepts
- **Custom Entity Types** - Define your own types (NPC, Enemy, Item, Trigger, whatever)
- **Custom Enums** - Define dropdowns for any property
- **Entity Templates** - Create reusable prefabs (e.g., "Goblin" template) and drag/drop to place instances
- **File Browser Panel** - Built-in asset browser with drag/drop for templates, tilesets, and images
- **Wang Tile Autotiling** - 47-tile blob for terrain edges
- **Rule-Based Automapping** - Pattern matching for automatic decoration
- **Entity References** - Link entities together (Quest → Reward Items)
- **Flexible Export** - Generic JSON that your game deserializes with custom binders
- **Bevy Native** - Runs as standalone app, easy to integrate

**Design Philosophy:**
The editor is a **generic tool** that knows nothing about NPCs, quests, or abilities. Your game's schema (`schema.json`) teaches the editor what entity types exist and what properties they have. The editor generates forms, validates data, and exports JSON. **Your game's client/server implement custom binders/deserializers** to interpret that JSON and spawn actual game entities.

**Works For Any Game:**
- MMORPGs (NPCs, Quests, Enemies, Items)
- Platformers (Platforms, Hazards, Collectibles, Checkpoints)
- Puzzle Games (Switches, Doors, Triggers, Goals)
- Strategy Games (Units, Buildings, Resources, Spawn Points)
- Adventure Games (Interactables, Dialogue, Puzzles)

---

## Architecture

### Crate Structure

```
crates/
├── eryndor_shared/          # Shared types (unchanged)
├── eryndor_server/          # Game server
├── eryndor_client/          # Game client
└── eryndor_editor/          # NEW: World editor (delete existing, fresh start)
    ├── Cargo.toml
    └── src/
        ├── main.rs          # App entry point
        ├── lib.rs           # Module exports
        ├── app.rs           # EditorApp plugin
        ├── ui/              # egui UI modules
        │   ├── mod.rs
        │   ├── menu_bar.rs      # File, Edit, View, Tools menus
        │   ├── toolbar.rs       # Tool selection (paint, select, entity, automap)
        │   ├── inspector.rs     # Property inspector panel (schema-driven)
        │   ├── tree_view.rs     # Entity relationship tree
        │   ├── file_browser.rs  # Built-in file browser with drag/drop
        │   ├── tileset.rs       # Tileset palette panel
        │   ├── property_window.rs # Pop-up property editor windows
        │   └── dialogs.rs       # File dialogs, confirmations
        ├── templates/       # Entity template system (prefabs)
        │   ├── mod.rs
        │   ├── template.rs      # Template definition and storage
        │   └── instance.rs      # Template instance management
        ├── tools/           # Editor tools
        │   ├── mod.rs
        │   ├── paint.rs         # Tile painting with autotiling
        │   ├── select.rs        # Entity/tile selection tool
        │   ├── entity.rs        # Entity placement tool
        │   ├── eraser.rs        # Eraser tool
        │   └── fill.rs          # Flood fill tool
        ├── autotile/        # Wang tile autotiling system
        │   ├── mod.rs
        │   ├── wang.rs          # 47-tile blob algorithm
        │   └── tileset_config.rs # Autotile tileset configuration
        ├── automap/         # Rule-based automapping
        │   ├── mod.rs
        │   ├── rules.rs         # Automapping rule definitions
        │   └── engine.rs        # Rule execution engine
        ├── schema/          # Schema-defined types system
        │   ├── mod.rs
        │   ├── types.rs         # TypeDef, PropertyDef structures
        │   ├── loader.rs        # Load types.json schema
        │   └── form_generator.rs # Generate egui forms from schema
        ├── map/             # Map data structures
        │   ├── mod.rs
        │   ├── layer.rs         # Tile layers
        │   ├── tileset.rs       # Tileset management
        │   └── entity.rs        # Entity instances
        ├── project/         # Project management
        │   ├── mod.rs
        │   ├── file.rs          # Save/load project (.eryndor)
        │   └── export.rs        # Export to split JSON
        └── render/          # Bevy rendering
            ├── mod.rs
            ├── map_render.rs    # Tile rendering with autotiles
            └── entity_render.rs # Entity gizmos
```

### Dependencies (Cargo.toml)

```toml
[package]
name = "eryndor_editor"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "editor"
path = "src/main.rs"

[dependencies]
bevy = { workspace = true, features = ["bevy_asset", "bevy_render", "bevy_winit", "bevy_sprite", "bevy_ui", "png"] }
bevy_egui = "0.33"
eryndor_shared = { path = "../eryndor_shared" }
serde = { workspace = true }
serde_json = "1.0"
rfd = "0.15"  # Native file dialogs
uuid = { version = "1.0", features = ["v4", "serde"] }
```

---

## Schema System - 100% User-Defined Types

The editor has **no hardcoded game concepts**. Everything is defined in your project's `schema.json`:

### Core Concepts

1. **Enums** - Named lists of values (dropdown options)
2. **Entity Types** - Templates for game objects with properties
3. **Data Types** - Non-placeable data (items in a database, not on the map)
4. **Embedded Types** - Inline objects used as array items

### schema.json Format

```json
{
  "version": 1,
  "project": {
    "name": "My Game",
    "tile_size": 32,
    "default_layer_types": ["Ground", "Collision", "Objects"]
  },

  "enums": {
    "YourEnum": ["Value1", "Value2", "Value3"]
  },

  "data_types": {
    "TypeName": {
      "color": "#HEXCOLOR",
      "icon": "optional_icon_name",
      "properties": [...]
    }
  },

  "entity_types": {
    "TypeName": {
      "color": "#HEXCOLOR",
      "icon": "optional_icon_name",
      "placeable": true,
      "properties": [...]
    }
  },

  "embedded_types": {
    "TypeName": {
      "properties": [...]
    }
  }
}
```

### Example: MMORPG Schema

```json
{
  "version": 1,
  "project": {
    "name": "Eryndor MMO",
    "tile_size": 32
  },

  "enums": {
    "CharacterClass": ["Warrior", "Mage", "Rogue", "Ranger"],
    "WeaponType": ["Sword", "Axe", "Dagger", "Staff", "Bow"],
    "NpcRole": ["Trainer", "Vendor", "QuestGiver", "Villager"],
    "ObjectiveType": ["KillEnemy", "CollectItem", "TalkToNpc"]
  },

  "data_types": {
    "Item": {
      "color": "#FFD700",
      "properties": [
        { "name": "name", "type": "string", "required": true },
        { "name": "attack_power", "type": "int", "default": 0 },
        { "name": "required_class", "type": "enum", "enumType": "CharacterClass" }
      ]
    },
    "Quest": {
      "color": "#4169E1",
      "properties": [
        { "name": "name", "type": "string", "required": true },
        { "name": "objective", "type": "enum", "enumType": "ObjectiveType" },
        { "name": "reward_items", "type": "array", "itemType": "ref", "refType": "Item" },
        { "name": "prerequisite", "type": "ref", "refType": "Quest" }
      ]
    },
    "Ability": {
      "color": "#9932CC",
      "properties": [
        { "name": "name", "type": "string", "required": true },
        { "name": "damage", "type": "int" },
        { "name": "required_weapon", "type": "enum", "enumType": "WeaponType" },
        { "name": "required_proficiency", "type": "int", "min": 0, "max": 100 }
      ]
    }
  },

  "entity_types": {
    "NPC": {
      "color": "#32CD32",
      "placeable": true,
      "properties": [
        { "name": "name", "type": "string", "required": true },
        { "name": "role", "type": "enum", "enumType": "NpcRole" },
        { "name": "quests", "type": "array", "itemType": "ref", "refType": "Quest" },
        { "name": "shop_items", "type": "array", "itemType": "embedded", "embeddedType": "ShopEntry" }
      ]
    },
    "Enemy": {
      "color": "#FF4444",
      "placeable": true,
      "properties": [
        { "name": "name", "type": "string", "required": true },
        { "name": "health", "type": "int", "required": true },
        { "name": "loot", "type": "array", "itemType": "embedded", "embeddedType": "LootEntry" }
      ]
    },
    "SpawnPoint": {
      "color": "#00FFFF",
      "placeable": true,
      "properties": [
        { "name": "spawn_type", "type": "string" }
      ]
    }
  },

  "embedded_types": {
    "ShopEntry": {
      "properties": [
        { "name": "item", "type": "ref", "refType": "Item", "required": true },
        { "name": "price", "type": "int", "required": true }
      ]
    },
    "LootEntry": {
      "properties": [
        { "name": "item", "type": "ref", "refType": "Item", "required": true },
        { "name": "chance", "type": "float", "min": 0, "max": 1, "default": 0.1 }
      ]
    }
  }
}
```

### Example: Platformer Schema

```json
{
  "version": 1,
  "project": { "name": "Super Jumper", "tile_size": 16 },

  "enums": {
    "HazardType": ["Spikes", "Lava", "Crusher", "Laser"],
    "CollectibleType": ["Coin", "Gem", "PowerUp", "Key"]
  },

  "entity_types": {
    "Platform": {
      "color": "#8B4513",
      "placeable": true,
      "properties": [
        { "name": "moving", "type": "bool", "default": false },
        { "name": "speed", "type": "float", "showIf": "moving == true" },
        { "name": "waypoints", "type": "array", "itemType": "point", "showIf": "moving == true" }
      ]
    },
    "Hazard": {
      "color": "#FF0000",
      "placeable": true,
      "properties": [
        { "name": "hazard_type", "type": "enum", "enumType": "HazardType" },
        { "name": "damage", "type": "int", "default": 1 }
      ]
    },
    "Collectible": {
      "color": "#FFD700",
      "placeable": true,
      "properties": [
        { "name": "type", "type": "enum", "enumType": "CollectibleType" },
        { "name": "value", "type": "int", "default": 1 }
      ]
    },
    "Checkpoint": {
      "color": "#00FF00",
      "placeable": true,
      "properties": [
        { "name": "is_start", "type": "bool", "default": false }
      ]
    },
    "Door": {
      "color": "#0000FF",
      "placeable": true,
      "properties": [
        { "name": "target_level", "type": "string" },
        { "name": "required_key", "type": "ref", "refType": "Collectible" }
      ]
    }
  }
}
```

### Property Type Support

| Type | UI Widget | Description |
|------|-----------|-------------|
| `string` | TextEdit | Single-line text input |
| `multiline` | TextEdit (multi) | Multi-line text area |
| `int` | DragValue | Integer with optional min/max |
| `float` | DragValue | Float with optional min/max |
| `bool` | Checkbox | Boolean toggle |
| `enum` | ComboBox | Dropdown from enum values |
| `ref` | ComboBox | Reference to another entity by ID |
| `array` | List + Add button | Array of items/refs/objects |
| `object` | Nested form | Embedded object (like ShopItem) |

### Conditional Properties (`showIf`)

Properties can be conditionally shown based on other property values:
```json
{ "name": "weapon_type", "type": "enum", "enumType": "WeaponType", "showIf": "slot == Weapon" }
```

---

## Wang Tile Autotiling (47-Tile Blob)

### How It Works

Wang blob autotiling automatically selects the correct tile variant based on neighboring tiles of the same terrain type.

```
Neighbor bits (8 directions):
  NW  N  NE       1   2   4
   W  .   E   =   8   .  16
  SW  S  SE      32  64 128

Total: 256 combinations → 47 unique tiles (corners ignored when edge is empty)
```

### Tileset Configuration

Each autotile tileset needs a configuration mapping tile indices to blob patterns:

```json
{
  "name": "grass_water",
  "tile_size": 32,
  "terrain_types": ["grass", "water"],
  "autotile_mappings": {
    "water": {
      "0": 0,      // Isolated (no neighbors)
      "2": 1,      // N only
      "8": 2,      // W only
      "10": 3,     // N+W
      "11": 4,     // N+W+NW
      // ... all 47 patterns
      "255": 46    // All neighbors
    }
  }
}
```

### Autotile Workflow

1. User selects "Water" terrain from palette
2. User paints on map
3. Editor calculates blob index for painted tile + all 8 neighbors
4. Editor looks up correct tile variant for each affected cell
5. Map displays with smooth terrain transitions

---

## Rule-Based Automapping

### Automapping Rules

Like Tiled, define rules that automatically place tiles based on patterns:

```json
{
  "name": "shore_details",
  "description": "Add shore decorations where water meets grass",
  "input_layer": "Ground",
  "output_layer": "Decorations",
  "rules": [
    {
      "name": "shore_rocks",
      "chance": 0.3,
      "pattern": {
        "center": "water",
        "north": "grass"
      },
      "output": "shore_rock_tile"
    },
    {
      "name": "lily_pads",
      "chance": 0.2,
      "pattern": {
        "center": "water",
        "north": "water",
        "south": "water"
      },
      "output": "lily_pad_tile"
    }
  ]
}
```

### Rule Engine

1. User paints tiles on input layer
2. User clicks "Run Automapping" (or enable auto-run)
3. Engine scans all tiles matching rule patterns
4. Engine places output tiles on output layer (with random chance)

### Pattern Matching

```
Pattern positions:
  NW  N  NE
   W  C   E    C = center (required match)
  SW  S  SE

Match types:
  "grass"     - Must be this terrain
  "!grass"    - Must NOT be this terrain
  "*"         - Any terrain (wildcard)
  ["a", "b"]  - Must be one of these
```

---

## Property Window Workflow

### Creating an NPC (Example Flow)

1. **Right-click** in map viewport → "New NPC"
2. **Property Window** opens with schema-generated form:
   ```
   ┌─────────────────────────────────────┐
   │ New NPC                        [X]  │
   ├─────────────────────────────────────┤
   │ Name: [Gorrim the Blacksmith     ]  │
   │ Type: [Blacksmith            ▼]     │
   │                                     │
   │ Dialogue: [+ Create New...]         │
   │                                     │
   │ Items for Sale:                     │
   │ ┌─────────────────────────────────┐ │
   │ │ Iron Sword      100g    [x]    │ │
   │ │ Steel Sword     250g    [x]    │ │
   │ └─────────────────────────────────┘ │
   │ [+ Add Item]                        │
   │                                     │
   │ Quests: (none)                      │
   │ [+ Add Quest]                       │
   │                                     │
   │         [Cancel]  [Create NPC]      │
   └─────────────────────────────────────┘
   ```
3. Click "Create NPC" → NPC placed at cursor position
4. NPC appears in Tree View under "NPCs" with relationship sub-tree

### Reference Dropdowns

When adding a reference (like `loot_table` on Enemy):
- ComboBox shows all existing LootTables by name
- Option to "+ Create New..." opens nested property window
- Created entity auto-selected and added to database

---

## Entity Templates (Prefabs)

Entity templates allow you to create reusable "prefabs" - entities with pre-configured properties that can be placed multiple times.

### How Templates Work

1. **Create a Template**: Create an entity (e.g., "Goblin" enemy), configure its properties, then right-click → "Save as Template"
2. **Template is Saved**: Stored in `templates/{type}/{name}.template.json` (e.g., `templates/Enemy/Goblin.template.json`)
3. **Drag to Place**: Drag template from File Browser onto map to place an instance
4. **Instances Reference Template**: Placed entities store `template_id` + any overridden properties

### Template Structure

```json
{
  "id": "uuid-here",
  "name": "Goblin",
  "type": "Enemy",
  "properties": {
    "health": 50,
    "attack": 10,
    "loot": ["uuid-of-loot-item"]
  }
}
```

### Instance Overrides

When placing a template instance, you can override specific properties:

```json
{
  "id": "instance-uuid",
  "template_id": "goblin-template-uuid",
  "position": [400, 300],
  "overrides": {
    "health": 75  // This goblin has more health than the template default
  }
}
```

### Template Workflow

```
┌─────────────────────────────────────────────────────────┐
│  1. Create entity          2. Configure properties      │
│     ↓                          ↓                        │
│  [+ New Enemy]  →  [Name: Goblin, Health: 50, ...]      │
│                          ↓                              │
│  3. Save as template    4. Template appears in browser  │
│     ↓                          ↓                        │
│  [Right-click → Save]  →  📁 templates/Enemy/Goblin     │
│                          ↓                              │
│  5. Drag to map         6. Instance created             │
│     ↓                          ↓                        │
│  [Drag from browser]   →  Goblin @ (400, 300)           │
└─────────────────────────────────────────────────────────┘
```

---

## File Browser Panel

A built-in asset browser for managing templates, tilesets, and images with drag/drop support.

### Panel Layout

```
┌─────────────────────────────────────┐
│ 📁 File Browser                 [↻] │
├─────────────────────────────────────┤
│ 📁 assets/                          │
│   📁 templates/                     │
│     📁 Enemy/                       │
│       📄 Goblin.template.json       │
│       📄 Spider.template.json       │
│     📁 NPC/                         │
│       📄 Blacksmith.template.json   │
│   📁 tilesets/                      │
│     🖼 terrain.png                  │
│     🖼 decorations.png              │
│   📁 images/                        │
│     🖼 npc_portraits/               │
└─────────────────────────────────────┘
```

### Drag/Drop Actions

| Drag From | Drop To | Action |
|-----------|---------|--------|
| Template file | Map viewport | Place instance of template |
| Template file | Tree view | Add to level's entity list |
| Tileset PNG | Tileset panel | Import tileset |
| Image file | Entity property | Set sprite/icon path |

### File Browser Features

- **Create New Template**: Right-click folder → "New Template..." → Select type from schema
- **Rename/Delete**: Context menu on files
- **Quick Filter**: Type to filter by name
- **Preview**: Hover to see template properties or image thumbnail
- **Refresh**: Detect changes from external editors

---

## Game-Side Integration (Custom Binders)

The editor exports **generic JSON**. Your game's client/server must implement **custom binders/deserializers** to interpret this data.

### Architecture Overview

```
┌─────────────┐     ┌─────────────┐     ┌─────────────────────────┐
│   EDITOR    │     │  EXPORTED   │     │     YOUR GAME           │
│             │ --> │    JSON     │ --> │                         │
│  Generic    │     │  (generic)  │     │  Custom Binders/        │
│  Schema-    │     │             │     │  Deserializers          │
│  Driven     │     │  schema.json│     │                         │
│             │     │  data/*.json│     │  Spawns actual          │
│             │     │  levels/*   │     │  game entities          │
└─────────────┘     └─────────────┘     └─────────────────────────┘
```

### Server-Side Loader Example

```rust
// crates/eryndor_server/src/content_loader.rs

use serde_json::Value;
use std::collections::HashMap;

/// Generic loader for editor-exported data
pub fn load_level(path: &str) -> Result<(), Error> {
    let json: Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;

    for entity in json["entities"].as_array().unwrap() {
        let entity_type = entity["type"].as_str().unwrap();
        let position = parse_position(&entity["position"]);
        let props = &entity["properties"];

        // YOUR GAME'S CUSTOM BINDERS - map generic JSON to game types
        match entity_type {
            "NPC" => spawn_npc(position, props),
            "Enemy" => spawn_enemy(position, props),
            "SpawnPoint" => register_spawn_point(position, props),
            "Chest" => spawn_chest(position, props),
            _ => warn!("Unknown entity type: {}", entity_type),
        }
    }
    Ok(())
}

/// Custom binder: JSON properties → NPC component
fn spawn_npc(pos: Vec2, props: &Value) -> Entity {
    commands.spawn((
        Transform::from_xyz(pos.x, pos.y, 0.0),
        Npc {
            name: props["name"].as_str().unwrap().to_string(),
            role: parse_enum::<NpcRole>(&props["role"]),
            quests: parse_ref_array(&props["quests"]),  // UUIDs → Quest handles
            shop_items: parse_shop_items(&props["shop_items"]),
        },
        Interactable,
    ))
}

/// Custom binder: JSON properties → Enemy component
fn spawn_enemy(pos: Vec2, props: &Value) -> Entity {
    // Handle template instances vs direct entities
    let base_props = if let Some(template_id) = props.get("template_id") {
        // Load template, merge with overrides
        let template = load_template(template_id.as_str().unwrap());
        merge_props(&template.properties, &props["overrides"])
    } else {
        props.clone()
    };

    commands.spawn((
        Transform::from_xyz(pos.x, pos.y, 0.0),
        Enemy {
            name: base_props["name"].as_str().unwrap().to_string(),
            health: base_props["health"].as_i64().unwrap() as i32,
            max_health: base_props["health"].as_i64().unwrap() as i32,
            attack: base_props["attack"].as_i64().unwrap() as i32,
        },
        LootDrops(parse_loot(&base_props["loot"])),
    ))
}
```

### Client-Side Loader (Similar Pattern)

```rust
// crates/eryndor_client/src/level_renderer.rs

/// Client loads levels for rendering (no game logic)
pub fn load_level_visuals(path: &str, commands: &mut Commands, assets: &AssetServer) {
    let json: Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;

    // Load tile layers
    for layer in json["layers"].as_array().unwrap() {
        if layer["type"] == "tiles" {
            spawn_tile_layer(layer, commands, assets);
        }
    }

    // Load entity visuals (sprites, not game logic)
    for entity in json["entities"].as_array().unwrap() {
        spawn_entity_visual(entity, commands, assets);
    }
}
```

### Key Integration Points

| Component | Responsibility |
|-----------|----------------|
| **Schema** | Defines what types/properties exist (shared) |
| **Editor** | Creates/edits data, exports generic JSON |
| **Server Binders** | JSON → game components with logic |
| **Client Binders** | JSON → visual components for rendering |

### Benefits of This Separation

1. **Editor stays generic** - No game code in editor
2. **Type-safe game code** - Your binders convert to strongly-typed Rust structs
3. **Easy iteration** - Change schema, update binders, no editor changes needed
4. **Validation** - Binders can validate data at load time
5. **Hot reload** - Server/client can reload JSON without recompiling

---

## Data Model - Fully Generic

The editor's internal data model has **no game-specific types**. Everything is driven by the schema.

### Core Structures

```rust
/// The entire editor project
#[derive(Serialize, Deserialize)]
pub struct Project {
    pub version: u32,
    pub schema: Schema,                      // Embedded or path to schema.json
    pub tilesets: Vec<Tileset>,
    pub data: DataStore,                     // All data_type instances
    pub levels: Vec<Level>,                  // Maps/zones
}

/// Schema loaded from schema.json
#[derive(Serialize, Deserialize)]
pub struct Schema {
    pub enums: HashMap<String, Vec<String>>,
    pub data_types: HashMap<String, TypeDef>,
    pub entity_types: HashMap<String, TypeDef>,
    pub embedded_types: HashMap<String, TypeDef>,
}

/// Definition of a type (from schema)
#[derive(Serialize, Deserialize)]
pub struct TypeDef {
    pub color: String,
    pub icon: Option<String>,
    pub placeable: Option<bool>,
    pub properties: Vec<PropertyDef>,
}

/// Definition of a property (from schema)
#[derive(Serialize, Deserialize)]
pub struct PropertyDef {
    pub name: String,
    pub prop_type: PropType,
    pub required: bool,
    pub default: Option<Value>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub show_if: Option<String>,           // Conditional visibility
    pub enum_type: Option<String>,         // For enum props
    pub ref_type: Option<String>,          // For ref props
    pub item_type: Option<String>,         // For array props
    pub embedded_type: Option<String>,     // For embedded props
}

#[derive(Serialize, Deserialize)]
pub enum PropType {
    String, Multiline, Int, Float, Bool,
    Enum, Ref, Array, Embedded, Point, Color,
}
```

### Generic Data Storage

```rust
/// Stores all data_type instances (non-placeable things like Items, Quests)
#[derive(Serialize, Deserialize)]
pub struct DataStore {
    /// Key: type name (e.g., "Item", "Quest")
    /// Value: list of instances of that type
    pub instances: HashMap<String, Vec<DataInstance>>,
}

/// A single instance of a data_type
#[derive(Serialize, Deserialize)]
pub struct DataInstance {
    pub id: Uuid,
    pub type_name: String,                   // "Item", "Quest", etc.
    pub properties: HashMap<String, Value>,  // Property values
}

/// A level/map containing tiles and entities
#[derive(Serialize, Deserialize)]
pub struct Level {
    pub id: Uuid,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub layers: Vec<Layer>,
    pub entities: Vec<EntityInstance>,
}

/// A layer (tiles or objects)
#[derive(Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub layer_type: LayerType,
    pub visible: bool,
    pub data: LayerData,
}

#[derive(Serialize, Deserialize)]
pub enum LayerType {
    Tiles,       // Tile data
    Objects,     // Entity instances (alternative to level.entities)
}

#[derive(Serialize, Deserialize)]
pub enum LayerData {
    Tiles {
        tileset_id: Uuid,
        tiles: Vec<Option<u32>>,  // Tile indices, row-major
    },
    Objects {
        entities: Vec<EntityInstance>,
    },
}

/// An entity placed in the world
#[derive(Serialize, Deserialize)]
pub struct EntityInstance {
    pub id: Uuid,
    pub type_name: String,                   // "NPC", "Enemy", "Checkpoint", etc.
    pub position: Vec2,
    pub properties: HashMap<String, Value>,  // Property values
}
```

### Property Values

```rust
/// Generic property value (JSON-like)
#[derive(Serialize, Deserialize, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Ref(Uuid),                              // Reference to another instance
    Array(Vec<Value>),
    Object(HashMap<String, Value>),         // Embedded type
    Point { x: f32, y: f32 },
    Color { r: u8, g: u8, b: u8, a: u8 },
}
```

---

## UI Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│ File   Edit   View   Tools   Help                            [_][□][X] │
├─────────────────────────────────────────────────────────────────────────┤
│ [🖌Paint] [🔲Select] [📍Entity] [🗑Erase] │ Layer: [Ground ▼] Zoom: 100% │
├───────────────┬─────────────────────────────────────┬───────────────────┤
│ TREE VIEW     │                                     │ INSPECTOR         │
│               │                                     │                   │
│ ▼ Database    │                                     │ Selected: Goblin  │
│   ▼ Items     │                                     │ ─────────────────│
│     Sword     │                                     │ Name: [Goblin   ] │
│     Dagger    │         MAP VIEWPORT                │ Type: [Enemy    ] │
│     Potion    │                                     │ Health: [50     ] │
│   ▼ Quests    │    (Tile painting & entities)      │ Attack: [10     ] │
│     First...  │                                     │ Defense: [5     ] │
│   ▼ NPCs      │                                     │ ─────────────────│
│     Elder     │                                     │ Loot Table:       │
│     Smith     │                                     │ [goblin_loot ▼  ] │
│   ▼ Enemies   │                                     │                   │
│     Goblin    │                                     │ Relationships:    │
│     Spider    │                                     │ └─ Loot Table     │
│ ▼ Zones       │                                     │    └─ Iron Sword  │
│   ▼ Starter   │                                     │    └─ Gold (5-10) │
│     Ground    │                                     │                   │
│     Collision │                                     │                   │
│     Entities  │                                     │                   │
├───────────────┴─────────────────────────────────────┴───────────────────┤
│ TILESET PALETTE                                                         │
│ [tile][tile][tile][tile][tile][tile][tile][tile][tile][tile][tile]...  │
└─────────────────────────────────────────────────────────────────────────┘
```

### Panel Descriptions

**Tree View (Left Panel)**
- Expandable tree showing all project data
- Database section: Items, Abilities, Quests, Dialogue, NPCs, Enemies, Loot Tables
- Zones section: Each zone with its layers and entities
- Click to select and edit in Inspector
- Drag-drop to assign relationships (drag Quest onto NPC)

**Map Viewport (Center)**
- 2D tile map view with pan/zoom
- Shows current zone's layers
- Entity gizmos (colored shapes for NPCs, enemies)
- Grid overlay toggle
- Layer visibility toggles

**Inspector (Right Panel)**
- Shows properties of selected item
- Text fields, dropdowns, checkboxes for properties
- Array editors for lists (items for sale, quests, etc.)
- Relationship sub-tree showing what this entity connects to

**Tileset Palette (Bottom)**
- Shows tiles from selected tileset
- Click to select tile for painting
- Multi-tile selection for stamp mode

---

## Implementation Phases

### Phase 0: Cleanup (Day 1)
1. Delete existing `crates/eryndor_editor/` directory
2. Remove ldtk_rust dependency from server (switching to custom JSON)
3. Delete `ldtk_loader.rs`
4. Delete `assets/ldtk/` directory
5. Clean up workspace Cargo.toml

**Deliverable:** Clean slate for new editor

### Phase 1: Schema System & App Shell (Days 2-3)
1. Create new `eryndor_editor` crate with Cargo.toml
2. Setup Bevy app with bevy_egui
3. Create basic UI layout (tree view, inspector, viewport panels)
4. **Implement schema system:**
   - Load `types.json` schema file
   - Parse type definitions and enums
   - Build form generator from schema
5. Implement property window system (pop-up windows)
6. Add File > New/Open/Save with rfd dialogs

**Deliverable:** Editor loads schema and can generate forms dynamically

### Phase 2: Database Editing with Schema Forms (Days 4-5)
1. Implement tree view for database categories
2. **Schema-driven Inspector** that generates forms from types.json:
   - String/multiline text fields
   - Int/float drag values with min/max
   - Enum dropdowns
   - Bool checkboxes
   - Reference dropdowns (with "+ Create New...")
   - Array editors (add/remove items)
   - Embedded object forms
3. Implement conditional property visibility (`showIf`)
4. Add/remove entities from database
5. UUID-based references between entities

**Deliverable:** Can create/edit any entity type defined in schema

### Phase 3: File Browser & Property Windows (Days 6-7)
1. **File Browser Panel:**
   - Directory tree view with expand/collapse
   - File icons by type (template, tileset, image)
   - Quick filter/search box
   - Context menu (New, Rename, Delete)
2. **Drag/Drop Support:**
   - Drag templates to map viewport
   - Drag images to entity properties
   - Drag tilesets to tileset panel
3. Property window workflow:
   - Right-click → "New Item/Quest/NPC/Enemy"
   - Pop-up window with schema form
   - Create button adds to database
4. Tree view relationship sub-trees:
   - Expand NPC to see its Quests, Items
   - Expand Quest to see its Rewards
   - Expand Enemy to see its LootTable
5. Reference dropdowns auto-populate from database

**Deliverable:** File browser with drag/drop, property windows with relationships

### Phase 4: Dialogue Tree Editor (Day 8)
1. Create DialogueTree type in schema
2. Node list editor (add/remove nodes)
3. Response editing with next-node references
4. Condition editing (enum dropdown)
5. Action assignment (OpenShop, AcceptQuest, etc.)
6. Visual preview of dialogue flow

**Deliverable:** Can create dialogue trees and link to NPCs

### Phase 5: Zone & Basic Tile Editing (Days 9-10)
1. Create Zone management (add/remove zones)
2. Implement TileLayer structure
3. Add tileset loading (PNG images)
4. **Basic tile painting** (no autotile yet):
   - Select tile from palette
   - Paint on map
   - Eraser tool
5. Layer management (add, remove, reorder, visibility)
6. Map viewport rendering with Bevy sprites
7. Pan/zoom controls

**Deliverable:** Can paint basic tile maps with multiple layers

### Phase 6: Wang Tile Autotiling (Days 11-12)
1. Implement autotile tileset configuration
2. **47-tile blob algorithm:**
   - Calculate neighbor bitmask
   - Map bitmask to tile index
   - Handle corner optimization
3. Auto-update neighbors when painting
4. Terrain type selection in palette
5. Mixed terrain transitions

**Deliverable:** Smooth terrain edges with Wang blob autotiling

### Phase 7: Rule-Based Automapping (Day 13)
1. Implement automapping rule format
2. **Rule engine:**
   - Pattern matching (center + neighbors)
   - Wildcard and negation support
   - Random chance per rule
3. Rule editor UI
4. "Run Automapping" button
5. Optional auto-run on paint

**Deliverable:** Automatic decoration placement based on rules

### Phase 8: Entity Templates & Placement (Days 14-15)
1. **Template System:**
   - Save entity as template (right-click → "Save as Template")
   - Template storage in `templates/{type}/{name}.template.json`
   - Template instances with override support
2. **Entity Placement:**
   - Drag templates from File Browser to map
   - Click to place entity on map
   - Select and move entities
   - Delete entities
3. **Entity Gizmos:**
   - Colored shapes from schema (color property)
   - Show entity name labels
   - Highlight on hover/select
4. Entity property editing in Inspector
5. Instance override editing (shows inherited + overridden values)

**Deliverable:** Template-based entity placement with drag/drop

### Phase 9: Export & Game Integration (Days 16-17)
1. **Split JSON export:**
   - `data/{TypeName}.json` for each data_type
   - `levels/{name}.json` for each level
   - `templates/{type}/{name}.template.json` for templates
   - Copy `schema.json` to output
2. **Server-side custom binders:**
   - Create `content_loader.rs` with match-based entity spawning
   - Parse templates and merge overrides
   - Convert generic JSON → game components
3. **Client-side loaders (if needed):**
   - Visual-only entity rendering
   - Tile layer rendering
4. Remove old content loading code
5. Test full workflow: Editor → Export → Server → Client

**Deliverable:** Editor exports JSON, game loads with custom binders

### Phase 10: Polish & UX (Day 18)
1. Undo/redo system
2. Keyboard shortcuts
3. Copy/paste entities
4. Validation warnings (missing required fields, broken refs)
5. Auto-save
6. Recent files menu

**Deliverable:** Production-ready editor

---

## Estimated Timeline

| Phase | Days | Cumulative |
|-------|------|------------|
| Phase 0: Cleanup | 1 | 1 |
| Phase 1: Schema + Shell | 2 | 3 |
| Phase 2: Database Forms | 2 | 5 |
| Phase 3: File Browser + Property Windows | 2 | 7 |
| Phase 4: Dialogue Editor | 1 | 8 |
| Phase 5: Basic Tiles | 2 | 10 |
| Phase 6: Autotiling | 2 | 12 |
| Phase 7: Automapping | 1 | 13 |
| Phase 8: Templates + Entity Placement | 2 | 15 |
| Phase 9: Export + Game Binders | 2 | 17 |
| Phase 10: Polish | 1 | 18 |

**Total: ~18 days** (expanded scope: file browser, templates, game integration)

---

## Export Format - Generic & Schema-Driven

The export is **100% driven by your schema**. The editor creates one JSON file per `data_type` and one per `level`.

### Export Structure

```
assets/content/
├── schema.json              # Your schema (copied)
├── data/
│   ├── {DataType1}.json     # One file per data_type
│   ├── {DataType2}.json
│   └── ...
└── levels/
    ├── {level1}.json        # One file per level
    ├── {level2}.json
    └── ...
```

### Example: MMORPG Export

```
assets/content/
├── schema.json
├── data/
│   ├── Item.json
│   ├── Quest.json
│   └── Ability.json
└── levels/
    ├── starter_zone.json
    └── forest.json
```

### Example: Platformer Export

```
assets/content/
├── schema.json
├── data/
│   └── (empty - platformer has no data_types, only entity_types)
└── levels/
    ├── level_1.json
    ├── level_2.json
    └── boss_level.json
```

### Generic Data File Format

Each `data/{TypeName}.json` contains all instances of that data_type:

```json
{
  "$type": "Item",
  "instances": [
    {
      "id": "uuid-here",
      "properties": {
        "name": "Iron Sword",
        "attack_power": 10
      }
    },
    {
      "id": "uuid-here",
      "properties": {
        "name": "Health Potion",
        "heal_amount": 50
      }
    }
  ]
}
```

### Generic Level File Format

Each `levels/{level_name}.json` contains tile layers and entity instances:

```json
{
  "name": "starter_zone",
  "width": 50,
  "height": 50,
  "tile_size": 32,
  "layers": [
    {
      "name": "Ground",
      "type": "tiles",
      "tileset": "terrain",
      "data": [1, 1, 2, 2, 1, 1]
    },
    {
      "name": "Collision",
      "type": "tiles",
      "tileset": "collision",
      "data": [0, 0, 1, 1, 0, 0]
    }
  ],
  "entities": [
    {
      "id": "uuid-here",
      "type": "NPC",
      "position": [400, 300],
      "properties": {
        "name": "Gorrim",
        "role": "Vendor"
      }
    },
    {
      "id": "uuid-here",
      "type": "Enemy",
      "position": [800, 500],
      "properties": {
        "name": "Goblin",
        "health": 50
      }
    },
    {
      "id": "uuid-here",
      "type": "SpawnPoint",
      "position": [100, 100],
      "properties": {
        "spawn_type": "player"
      }
    }
  ]
}
```

### How Your Game Loads It

Your game (client/server) reads the schema and JSON files, then interprets the data:

```rust
// Generic loader - works with ANY schema
fn load_level(path: &str, schema: &Schema) -> Level {
    let json = std::fs::read_to_string(path)?;
    let level: LevelData = serde_json::from_str(&json)?;

    for entity in level.entities {
        // entity.type = "NPC", "Enemy", "SpawnPoint", etc.
        // entity.properties = HashMap of property values
        // Your game decides what to do with each type
        match entity.type_name.as_str() {
            "NPC" => spawn_npc(entity),
            "Enemy" => spawn_enemy(entity),
            "SpawnPoint" => register_spawn(entity),
            _ => warn!("Unknown entity type: {}", entity.type_name),
        }
    }
}
```

### Bevy Integration Example

```rust
// Your game defines how to interpret schema types
fn spawn_npc(entity: &EntityData, commands: &mut Commands) {
    commands.spawn((
        Name::new(entity.get_string("name").unwrap_or("NPC")),
        Transform::from_xyz(entity.position.x, entity.position.y, 0.0),
        Npc {
            role: entity.get_enum("role").unwrap_or(NpcRole::Villager),
            quests: entity.get_ref_array("quests"),
        },
    ));
}
```

---

## Key Implementation Details

### Generic Tree View (Schema-Driven)

```rust
/// Renders tree view dynamically from schema
fn render_tree_view(ui: &mut egui::Ui, project: &Project, schema: &Schema, selection: &mut Selection) {
    // Data types section - auto-generated from schema.data_types
    egui::CollapsingHeader::new("Data")
        .default_open(true)
        .show(ui, |ui| {
            for (type_name, type_def) in &schema.data_types {
                let instances = project.data.instances.get(type_name);
                let count = instances.map(|v| v.len()).unwrap_or(0);

                egui::CollapsingHeader::new(format!("{} ({})", type_name, count))
                    .show(ui, |ui| {
                        if let Some(instances) = instances {
                            for instance in instances {
                                let name = instance.get_display_name();
                                if ui.selectable_label(selection.is_selected(&instance.id), &name).clicked() {
                                    selection.select(instance.id, type_name.clone());
                                }
                            }
                        }
                    });
            }
        });

    // Entity types section - auto-generated from schema.entity_types
    egui::CollapsingHeader::new("Entities")
        .default_open(true)
        .show(ui, |ui| {
            for (type_name, _type_def) in &schema.entity_types {
                // Count entities of this type across all levels
                let count = project.count_entities_of_type(type_name);
                ui.label(format!("{}: {}", type_name, count));
            }
        });
}
```

### Generic Inspector (Schema-Driven Form Generation)

```rust
/// Generates inspector form dynamically from schema
fn render_inspector(ui: &mut egui::Ui, schema: &Schema, instance: &mut DataInstance) {
    let type_def = schema.get_type(&instance.type_name);

    ui.heading(&instance.type_name);
    ui.separator();

    for prop_def in &type_def.properties {
        // Check conditional visibility
        if let Some(condition) = &prop_def.show_if {
            if !evaluate_condition(condition, &instance.properties) {
                continue;
            }
        }

        ui.horizontal(|ui| {
            ui.label(&prop_def.name);

            match prop_def.prop_type {
                PropType::String => {
                    let value = instance.get_string_mut(&prop_def.name);
                    ui.text_edit_singleline(value);
                }
                PropType::Int => {
                    let value = instance.get_int_mut(&prop_def.name);
                    let mut drag = egui::DragValue::new(value);
                    if let Some(min) = prop_def.min { drag = drag.clamp_range(min as i64..); }
                    if let Some(max) = prop_def.max { drag = drag.clamp_range(..=max as i64); }
                    ui.add(drag);
                }
                PropType::Enum => {
                    let enum_values = schema.enums.get(prop_def.enum_type.as_ref().unwrap());
                    let current = instance.get_string(&prop_def.name);
                    egui::ComboBox::from_id_salt(&prop_def.name)
                        .selected_text(current)
                        .show_ui(ui, |ui| {
                            for value in enum_values.unwrap() {
                                ui.selectable_value(
                                    instance.get_string_mut(&prop_def.name),
                                    value.clone(),
                                    value
                                );
                            }
                        });
                }
                PropType::Ref => {
                    // Dropdown of all instances of the referenced type
                    render_ref_dropdown(ui, schema, project, prop_def, instance);
                }
                PropType::Array => {
                    render_array_editor(ui, schema, prop_def, instance);
                }
                // ... other types
            }
        });
    }
}
```

### Tile Painting

```rust
fn handle_paint_tool(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut zone: ResMut<CurrentZone>,
    selected_tile: Res<SelectedTile>,
    selected_layer: Res<SelectedLayer>,
) {
    if mouse.pressed(MouseButton::Left) {
        let window = windows.single();
        let (camera, camera_transform) = camera.single();

        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                // Convert world position to tile coordinates
                let tile_x = (world_pos.x / zone.tile_size as f32) as i32;
                let tile_y = (world_pos.y / zone.tile_size as f32) as i32;

                if tile_x >= 0 && tile_x < zone.width as i32
                   && tile_y >= 0 && tile_y < zone.height as i32 {
                    let index = (tile_y * zone.width as i32 + tile_x) as usize;
                    if let Some(layer) = zone.layers.get_mut(selected_layer.0) {
                        layer.tiles[index] = Some(selected_tile.0);
                    }
                }
            }
        }
    }
}
```

---

## Files to Modify

| File | Changes |
|------|---------|
| `Cargo.toml` (workspace) | Add `eryndor_editor` to members |
| `crates/eryndor_editor/*` | New crate (all files) |
| `crates/eryndor_server/src/world.rs` | Load new JSON format |
| `crates/eryndor_server/src/ldtk_loader.rs` | Remove or repurpose |
| `crates/eryndor_shared/src/lib.rs` | Export shared types for editor |

---

## Summary

**Total Effort:** ~18 days of development

**Key Benefits:**
1. **100% Schema-Driven** - Zero hardcoded game concepts; define your types in JSON
2. **Works with ANY Bevy Game** - MMORPGs, platformers, puzzle games, strategy games
3. **Entity Templates (Prefabs)** - Create reusable entities, drag/drop to place instances
4. **File Browser with Drag/Drop** - Built-in asset browser for templates, tilesets, images
5. **Visual Map Editing** - Paint tiles with Wang autotiling, place entities
6. **Relationship Tree** - See connections between entities (NPC → Quest → Reward)
7. **Flexible Export** - Generic JSON format; your game implements custom binders
8. **Game Integration** - Server/client use custom deserializers to spawn actual entities
9. **Reusable Tool** - Build once, use across multiple game projects
