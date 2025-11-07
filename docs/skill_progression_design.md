# Eryndor MMO: Skill Progression & Tutorial System Design

---

## Table of Contents

1. [Summary](#summary)
2. [Design Principles](#design-principles)
3. [Architecture Overview](#architecture-overview)
4. [Core Systems](#core-systems)
5. [Data Structures](#data-structures)
6. [Extensibility Patterns](#extensibility-patterns)
7. [Content Templates](#content-templates)
8. [Implementation Phases](#implementation-phases)
9. [Database Schema](#database-schema)
10. [UI Framework](#ui-framework)
11. [Future Expansion](#future-expansion)

---

## Summary

This document outlines the design for a comprehensive skill progression and tutorial system for Eryndor MMO. The system is built on **data-driven principles** that allow easy addition of new content (classes, abilities, quests, skills) without requiring core code changes.

### Key Features

- **Experience & Leveling**: Traditional XP-based progression with milestone rewards
- **Weapon Proficiency**: Skill-up system that rewards using specific weapon types
- **Armor Proficiency**: Passive skill unlocks through taking damage while wearing armor
- **Quest-Unlocked Abilities**: Immersive ability learning through class trainer quests
- **Tutorial System**: Interactive NPCs guide players through systems naturally
- **Fully Extensible**: All content defined in data structures, not hardcoded logic

---

## Design Principles

### 1. Data-Driven Architecture

**All game content is defined in data structures, not code.**

- Adding a new ability = adding an entry to `AbilityDatabase`
- Adding a new class = adding a variant to `CharacterClass` enum + data entries
- Adding a quest = adding an entry to `QuestDatabase`
- Adding a trainer NPC = spawning with `Trainer` component + data
- NO hardcoded logic tied to specific IDs/classes/abilities

### 2. Composition Over Inheritance

**Use ECS components to build flexible, reusable systems.**

- Components define behavior (e.g., `Trainer`, `QuestGiver`, `Experience`)
- Systems operate on component combinations
- Easy to add new component types without breaking existing systems

###  3. Separation of Concerns

**Clear boundaries between systems:**

- **Data Layer**: Databases (abilities, quests, items, armor passives)
- **Logic Layer**: Systems (combat, progression, quests)
- **Presentation Layer**: UI components (spellbook, character sheet, tooltips)
- **Persistence Layer**: Database (save/load character state)

### 4. Progressive Disclosure

**Don't overwhelm new players:**

- Tutorial introduces one system at a time
- UI shows simple views first, detailed stats later
- Tooltips and hints appear contextually
- Complexity grows with player knowledge

### 5. Immersive World-Building

**Systems feel natural, not gamey:**

- NPCs have personalities and purpose
- Quest text explains "why" not just "what"
- Trainers exist in the world, not just menus
- Progression feels earned, not arbitrary

---

## Architecture Overview

### System Interaction Diagram

```
┌──────────────────┐
│  Player Action   │ (e.g., kills enemy)
└────────┬─────────┘
         │
         ▼
┌──────────────────────────────────────┐
│      Event Systems                   │
│  - Combat System                     │
│  - Quest Objective System            │
│  - Experience Gain System            │
└────────┬─────────────────────────────┘
         │
         ▼
┌──────────────────────────────────────┐
│    Progression Components            │
│  - Experience (XP tracking)          │
│  - WeaponProficiencyExp              │
│  - ArmorProficiencyExp               │
│  - LearnedAbilities                  │
│  - UnlockedArmorPassives             │
└────────┬─────────────────────────────┘
         │
         ▼
┌──────────────────────────────────────┐
│    Level-Up / Unlock Events          │
│  - Character level up                │
│  - Weapon proficiency level up       │
│  - Armor passive unlock              │
│  - Ability available to learn        │
└────────┬─────────────────────────────┘
         │
         ▼
┌──────────────────────────────────────┐
│    Notification System               │
│  - Send events to client             │
│  - Show UI notifications             │
│  - Update character sheet            │
│  - Trigger tutorial hints            │
└──────────────────────────────────────┘
```

### Data Flow: Learning a New Ability

```
1. Player levels up to 5
   ↓
2. Server checks AbilityDatabase for level 5 unlocks for player's class
   ↓
3. Server finds "Ability X requires level 5 + quest Y"
   ↓
4. Server sends notification: "Visit your trainer to learn a new ability!"
   ↓
5. Player talks to Trainer NPC
   ↓
6. Server checks Trainer component, sees quest Y available
   ↓
7. Server sends QuestDialogueEvent with quest details
   ↓
8. Player accepts quest, completes objectives
   ↓
9. Player returns to Trainer, completes quest
   ↓
10. Server grants ability via LearnedAbilities component
    ↓
11. Client UI updates: spellbook shows new ability, can drag to hotbar
```

---

## Core Systems

### 1. Experience & Leveling System

#### Components

```rust
// components.rs
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Experience {
    pub current_xp: u32,
    pub xp_to_next_level: u32,
}

// Character component (already exists, extend usage)
pub struct Character {
    pub name: String,
    pub class: CharacterClass,
    pub level: u32,  // Currently unused, will be active
}
```

#### XP Calculation Formula

```rust
// Exponential curve: XP needed = 100 * level^1.5
pub fn xp_for_level(level: u32) -> u32 {
    (100.0 * (level as f32).powf(1.5)) as u32
}

// Example progression:
// Level 1 → 2: 100 XP
// Level 2 → 3: 283 XP
// Level 5 → 6: 1,118 XP
// Level 10 → 11: 3,162 XP
```

#### XP Sources

| Source | XP Amount | Notes |
|--------|-----------|-------|
| Kill Slime (level 1 enemy) | 50 XP | Base enemy XP |
| Kill Boss Enemy | 500+ XP | Scaled to difficulty |
| Complete Quest | Variable | Defined in quest data |
| First time actions | Bonus | Equipment set, proficiency milestones |

#### Level-Up Rewards

**Automatic stat scaling:**
```rust
// Every level up:
max_health += 10 + (class_modifier * level)
max_mana += 5 + (class_modifier * level)
attack += 0.5
defense += 0.3
```

**Milestone notifications:**
- Levels 3, 5, 7, 10, 12, 15, 18, 20: "Visit your class trainer!"

---

### 2. Weapon Proficiency System

#### Components

```rust
// components.rs
#[derive(Component, Clone)]
pub struct WeaponProficiencyExp {
    pub sword_xp: u32,
    pub dagger_xp: u32,
    pub staff_xp: u32,
    pub mace_xp: u32,
    pub bow_xp: u32,
    pub axe_xp: u32,
}

// WeaponProficiency already exists, will be actively used
pub struct WeaponProficiency {
    pub sword: u32,
    pub dagger: u32,
    pub staff: u32,
    pub mace: u32,
    pub bow: u32,
    pub axe: u32,
}
```

#### XP Gain Rules

**On successful weapon hit (auto-attack or ability):**
- Grant 5 XP to equipped weapon type's proficiency

**On enemy kill with weapon:**
- Grant bonus 50 XP to equipped weapon type

**Proficiency leveling:**
- 100 XP per proficiency level
- Max proficiency level: 100 (can be increased later)

#### Proficiency Bonuses

```rust
// Applied in damage calculation
let proficiency_bonus = proficiency_level * 0.02;  // 2% per level
let damage = base_damage * (1.0 + proficiency_bonus);

// Example: Level 10 sword proficiency = +20% damage with swords
```

#### Proficiency Requirements

**Items can require minimum proficiency:**

```rust
pub struct ItemDefinition {
    // ...existing fields...
    pub required_proficiency: Option<(WeaponType, u32)>,
    // Example: Some((WeaponType::Sword, 15)) = requires level 15 sword proficiency
}
```

**Benefits:**
- Creates clear progression path
- Prevents new players from using endgame weapons ineffectively
- Encourages specialization and build diversity

---

### 3. Armor Proficiency & Passive Skills

#### Armor Types

```rust
// components.rs
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArmorType {
    Light,   // Cloth - Mage armor
    Medium,  // Leather - Rogue armor
    Heavy,   // Iron - Knight armor
}
```

#### Components

```rust
#[derive(Component, Clone)]
pub struct ArmorProficiencyExp {
    pub light_xp: u32,
    pub medium_xp: u32,
    pub heavy_xp: u32,
}

#[derive(Component, Clone)]
pub struct ArmorProficiency {
    pub light: u32,
    pub medium: u32,
    pub heavy: u32,
}

#[derive(Component, Clone)]
pub struct UnlockedArmorPassives {
    pub passives: HashSet<u32>,  // Set of passive skill IDs
}
```

#### XP Gain Rules

**When player takes damage:**
```rust
// System: gain_armor_proficiency_from_damage()
let armor_xp = (damage_taken / 10.0) as u32;  // 10 damage = 1 XP

// Distribute XP to all equipped armor types
for armor_piece in equipped_armor {
    add_xp_to_armor_type(armor_piece.armor_type, armor_xp);
}
```

**Proficiency leveling:**
- 100 XP per proficiency level
- Unlock passive skills at levels 5, 10, 15, 20

#### Armor Passive Skills

```rust
// game_data.rs
pub struct ArmorPassiveSkill {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub armor_type: ArmorType,
    pub required_proficiency: u32,
    pub passive_type: PassiveType,
    pub value: f32,
}

pub enum PassiveType {
    DamageReduction(f32),        // Reduce incoming damage by %
    HealthRegeneration(f32),     // HP/second
    ManaRegeneration(f32),       // Mana/second
    MovementSpeed(f32),          // % increase
    CooldownReduction(f32),      // % reduction
    MaxHealthBonus(f32),         // Flat bonus
    MaxManaBonus(f32),           // Flat bonus
    CritChanceBonus(f32),        // % bonus
}
```

#### Example Passive Skills

**Heavy Armor (Iron) Passives:**
- Level 5: "Iron Skin" - 5% damage reduction
- Level 10: "Stalwart" - +50 max health
- Level 15: "Unbreakable" - 10% damage reduction
- Level 20: "Fortress" - 2 HP/second regeneration

**Medium Armor (Leather) Passives:**
- Level 5: "Agility" - +10% movement speed
- Level 10: "Evasion" - +5% crit chance
- Level 15: "Swift Reflexes" - 10% cooldown reduction
- Level 20: "Cat's Grace" - +15% movement speed

**Light Armor (Cloth) Passives:**
- Level 5: "Arcane Focus" - +50 max mana
- Level 10: "Meditation" - 2 mana/second regeneration
- Level 15: "Spell Weaving" - 15% cooldown reduction
- Level 20: "Mana Surge" - +100 max mana

#### Set Bonuses

```rust
// game_data.rs
pub fn calculate_armor_set_bonus(equipment: &Equipment, item_db: &ItemDatabase) -> StatBonuses {
    let armor_pieces = count_armor_pieces_by_type(equipment, item_db);

    for (armor_type, count) in armor_pieces {
        if count >= 2 {
            // 2-piece: +5% to primary stat
        }
        if count >= 3 {
            // 3-piece: +10% to primary stat + minor passive
        }
        if count >= 4 {
            // 4-piece: +15% to primary stat + major passive
        }
    }
}
```

---

### 4. Ability System (Expanded)

#### Ability Database Structure

```rust
// game_data.rs
pub struct AbilityDefinition {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub icon: String,  // For future sprite system
    pub damage_multiplier: f32,
    pub cooldown: f32,
    pub range: f32,
    pub mana_cost: f32,
    pub ability_type: AbilityType,
    pub unlock_requirement: AbilityUnlockRequirement,
}

pub enum AbilityType {
    DirectDamage,
    DamageOverTime { duration: f32, ticks: u32 },
    AreaOfEffect { radius: f32 },
    Buff { duration: f32, stat_bonus: StatBonuses },
    Debuff { duration: f32, effect: DebuffType },
    Mobility { distance: f32 },
    Heal { amount: f32 },
}

pub struct AbilityUnlockRequirement {
    pub required_level: u32,
    pub required_class: CharacterClass,
    pub prerequisite_quest_id: Option<u32>,
    pub trainer_npc_id: u32,  // Which NPC teaches this
}
```

#### Example Ability Progression (Knight)

| Level | Ability | Type | Description |
|-------|---------|------|-------------|
| 1 | Heavy Slash | Direct | 2.0x damage, 3s cooldown |
| 3 | Shield Bash | Debuff | 1.0x damage, stuns 2s, 8s cooldown |
| 5 | Taunt | Buff | Force enemies to target you, 12s cooldown |
| 7 | Whirlwind | AoE | 1.5x damage in radius, 10s cooldown |
| 10 | Defensive Stance | Buff | +20% defense, -10% damage, 30s duration |
| 12 | Execute | Direct | 3.0x damage to enemies <20% HP, 20s cooldown |
| 15 | Charge | Mobility | Dash to target, 1.5x damage, 15s cooldown |
| 18 | Last Stand | Buff | Survive fatal blow with 1 HP, 5min cooldown |

#### Learning New Abilities

**Flow:**
1. Player reaches level milestone
2. Server sends notification: "A new ability awaits you at your class trainer!"
3. Player visits class trainer NPC
4. Trainer offers quest: "Prove your mastery of [current abilities]"
5. Quest objective: Kill X enemies using your abilities
6. Return to trainer → learn new ability
7. Ability added to LearnedAbilities, appears in spellbook

---

## Data Structures

### Complete Component Reference

```rust
// components.rs (Shared)

// === CHARACTER PROGRESSION ===
pub struct Experience {
    pub current_xp: u32,
    pub xp_to_next_level: u32,
}

pub struct WeaponProficiencyExp {
    pub sword_xp: u32,
    pub dagger_xp: u32,
    pub staff_xp: u32,
    pub mace_xp: u32,
    pub bow_xp: u32,
    pub axe_xp: u32,
}

pub struct ArmorProficiencyExp {
    pub light_xp: u32,
    pub medium_xp: u32,
    pub heavy_xp: u32,
}

pub struct ArmorProficiency {
    pub light: u32,
    pub medium: u32,
    pub heavy: u32,
}

pub struct UnlockedArmorPassives {
    pub passives: HashSet<u32>,
}

// === NPC / WORLD ===
pub struct Trainer {
    pub trainer_type: TrainerType,
    pub class_specialization: Option<CharacterClass>,
    pub dialogue_lines: Vec<String>,
    pub available_ability_quests: Vec<u32>,  // Quest IDs
}

pub enum TrainerType {
    ClassTrainer,      // Teaches class abilities
    WeaponMaster,      // Explains weapon proficiency
    ArmorInstructor,   // Explains armor proficiency
    GeneralTutor,      // Basic tutorial
}
```

### Database Resources

```rust
// game_data.rs (Server)

pub struct AbilityDatabase {
    pub abilities: HashMap<u32, AbilityDefinition>,
}

pub struct ArmorPassiveDatabase {
    pub passives: HashMap<u32, ArmorPassiveSkill>,
}

pub struct QuestDatabase {
    pub quests: HashMap<u32, QuestDefinition>,
}

pub struct ItemDatabase {
    pub items: HashMap<u32, ItemDefinition>,
}
```

### Network Messages

```rust
// protocol.rs (Shared)

// === PROGRESSION EVENTS ===
#[derive(Event, Message)]
pub struct LevelUpEvent {
    pub new_level: u32,
    pub stat_increases: StatBonuses,
    pub abilities_available: bool,
}

#[derive(Event, Message)]
pub struct ProficiencyLevelUpEvent {
    pub proficiency_type: ProficiencyType,
    pub weapon_or_armor: String,
    pub new_level: u32,
}

pub enum ProficiencyType {
    Weapon,
    Armor,
}

#[derive(Event, Message)]
pub struct PassiveUnlockedEvent {
    pub passive_id: u32,
    pub passive_name: String,
    pub description: String,
}

// === ABILITY LEARNING ===
#[derive(Event, Message)]
pub struct LearnAbilityRequest {
    pub ability_id: u32,
    pub trainer_entity: Entity,
}

#[derive(Event, Message)]
pub struct LearnAbilityResponse {
    pub success: bool,
    pub message: String,
    pub ability: Option<AbilityDefinition>,
}

// === TRAINER INTERACTION ===
#[derive(Event, Message)]
pub struct TrainerDialogueEvent {
    pub trainer_name: String,
    pub dialogue_text: String,
    pub available_quests: Vec<u32>,
    pub available_abilities: Vec<u32>,
    pub hints: Vec<String>,
}
```

---

## Extensibility Patterns

### Pattern 1: Adding a New Class

**No core code changes required** - only data additions:

```rust
// 1. Add enum variant (components.rs)
pub enum CharacterClass {
    Knight,
    Mage,
    Rogue,
    Ranger,  // NEW CLASS
}

// 2. Add starting stats (character.rs - data only)
CharacterClass::Ranger => (
    10.0, // attack
    4.0,  // defense
    12.0, // crit
    90.0, // hp
    110.0,// mana
),

// 3. Add starting weapon (character.rs - data only)
CharacterClass::Ranger => ITEM_BOW,

// 4. Add starting proficiencies (weapon.rs - data only)
CharacterClass::Ranger => WeaponProficiency {
    bow: 10,
    dagger: 5,
    sword: 3,
    ..Default::default()
},

// 5. Add abilities to AbilityDatabase (game_data.rs - data only)
abilities.insert(ABILITY_RANGER_SHOT, AbilityDefinition {
    id: ABILITY_RANGER_SHOT,
    name: "Rapid Shot".to_string(),
    // ... rest of definition
});

// 6. Spawn trainer NPC (world.rs - data only)
commands.spawn((
    Trainer {
        trainer_type: TrainerType::ClassTrainer,
        class_specialization: Some(CharacterClass::Ranger),
        // ...
    },
    // ... position, visual, etc.
));

// 7. Add quests to QuestDatabase (game_data.rs - data only)
```

**Result:** New class fully functional with abilities, progression, and tutorials!

### Pattern 2: Adding a New Ability

```rust
// game_data.rs
impl Default for AbilityDatabase {
    fn default() -> Self {
        let mut abilities = HashMap::new();

        // Just add new entry - no code changes elsewhere!
        abilities.insert(ABILITY_TELEPORT, AbilityDefinition {
            id: ABILITY_TELEPORT,
            name: "Teleport".to_string(),
            description: "Instantly teleport to target location".to_string(),
            icon: "teleport.png".to_string(),
            damage_multiplier: 0.0,  // No damage
            cooldown: 15.0,
            range: 200.0,
            mana_cost: 50.0,
            ability_type: AbilityType::Mobility { distance: 200.0 },
            unlock_requirement: AbilityUnlockRequirement {
                required_level: 12,
                required_class: CharacterClass::Mage,
                prerequisite_quest_id: Some(QUEST_TELEPORT_TRAINING),
                trainer_npc_id: NPC_MAGE_TRAINER,
            },
        });

        Self { abilities }
    }
}
```

**Ability automatically:**
- Shows in spellbook when level requirement met
- Requires quest completion to learn
- Learned from correct trainer
- Can be placed on hotbar
- All handled by existing systems!

### Pattern 3: Adding Tutorial Content

```rust
// game_data.rs - QuestDatabase
quests.insert(QUEST_ARMOR_BASICS, QuestDefinition {
    id: QUEST_ARMOR_BASICS,
    name: "Understanding Armor".to_string(),
    description: "Learn about armor types and set bonuses.".to_string(),
    quest_giver: NPC_ARMOR_INSTRUCTOR,
    objectives: vec![
        QuestObjective::ObtainItem { item_id: ITEM_LEATHER_TUNIC, quantity: 1 },
        QuestObjective::ObtainItem { item_id: ITEM_LEATHER_PANTS, quantity: 1 },
        QuestObjective::TalkToNpc { npc_id: NPC_ARMOR_INSTRUCTOR },
    ],
    reward_exp: 150,
    reward_items: vec![
        (ITEM_LEATHER_BOOTS, 1),
    ],
});
```

**Quest automatically:**
- Shows in quest log when accepted
- Tracks objectives
- Grants rewards on completion
- Sends notifications
- No new code required!

### Pattern 4: Adding Armor Passive Skills

```rust
// game_data.rs - ArmorPassiveDatabase
passives.insert(PASSIVE_SHADOW_STEP, ArmorPassiveSkill {
    id: PASSIVE_SHADOW_STEP,
    name: "Shadow Step".to_string(),
    description: "Increase movement speed by 20% when wearing medium armor.".to_string(),
    armor_type: ArmorType::Medium,
    required_proficiency: 20,
    passive_type: PassiveType::MovementSpeed(0.20),
    value: 0.20,
});
```

**Passive automatically:**
- Unlocks when proficiency reached
- Shows in character sheet
- Applied in movement system
- Persisted to database
- Zero code changes!

---

## Content Templates

### Template: Class Ability Progression

```rust
// Copy this template for each new class

// === KNIGHT ABILITIES ===
pub const ABILITY_KNIGHT_HEAVY_SLASH: u32 = 1000;
pub const ABILITY_KNIGHT_SHIELD_BASH: u32 = 1001;
pub const ABILITY_KNIGHT_TAUNT: u32 = 1002;
pub const ABILITY_KNIGHT_WHIRLWIND: u32 = 1003;
pub const ABILITY_KNIGHT_DEFENSIVE_STANCE: u32 = 1004;
pub const ABILITY_KNIGHT_EXECUTE: u32 = 1005;
pub const ABILITY_KNIGHT_CHARGE: u32 = 1006;
pub const ABILITY_KNIGHT_LAST_STAND: u32 = 1007;

fn init_knight_abilities(abilities: &mut HashMap<u32, AbilityDefinition>) {
    // Level 1 - Starter
    abilities.insert(ABILITY_KNIGHT_HEAVY_SLASH, AbilityDefinition {
        id: ABILITY_KNIGHT_HEAVY_SLASH,
        name: "Heavy Slash".to_string(),
        description: "A powerful sword strike.".to_string(),
        icon: "heavy_slash.png".to_string(),
        damage_multiplier: 2.0,
        cooldown: 3.0,
        range: 30.0,
        mana_cost: 15.0,
        ability_type: AbilityType::DirectDamage,
        unlock_requirement: AbilityUnlockRequirement {
            required_level: 1,
            required_class: CharacterClass::Knight,
            prerequisite_quest_id: None,  // Starter ability
            trainer_npc_id: NPC_KNIGHT_TRAINER,
        },
    });

    // Level 3
    abilities.insert(ABILITY_KNIGHT_SHIELD_BASH, AbilityDefinition {
        id: ABILITY_KNIGHT_SHIELD_BASH,
        name: "Shield Bash".to_string(),
        description: "Bash your enemy, stunning them briefly.".to_string(),
        icon: "shield_bash.png".to_string(),
        damage_multiplier: 1.0,
        cooldown: 8.0,
        range: 30.0,
        mana_cost: 20.0,
        ability_type: AbilityType::Debuff {
            duration: 2.0,
            effect: DebuffType::Stun,
        },
        unlock_requirement: AbilityUnlockRequirement {
            required_level: 3,
            required_class: CharacterClass::Knight,
            prerequisite_quest_id: Some(QUEST_KNIGHT_SHIELD_BASH),
            trainer_npc_id: NPC_KNIGHT_TRAINER,
        },
    });

    // ... repeat for each ability level 5, 7, 10, 12, 15, 18
}
```

### Template: Ability Training Quest

```rust
// One quest per ability (except starter)

pub const QUEST_KNIGHT_SHIELD_BASH: u32 = 2001;

quests.insert(QUEST_KNIGHT_SHIELD_BASH, QuestDefinition {
    id: QUEST_KNIGHT_SHIELD_BASH,
    name: "The Shield is Mightier".to_string(),
    description: "Your trainer wants you to demonstrate proficiency with Heavy Slash before learning a defensive technique.".to_string(),
    quest_giver: NPC_KNIGHT_TRAINER,
    objectives: vec![
        QuestObjective::KillEnemy {
            enemy_type: ENEMY_TYPE_SLIME,
            count: 10,
            current: 0,
        },
    ],
    reward_exp: 200,
    reward_items: vec![],
    completion_grants_ability: Some(ABILITY_KNIGHT_SHIELD_BASH),
});
```

### Template: Trainer NPC

```rust
// world.rs - spawn_world() function

// Knight Trainer
commands.spawn((
    Replicated,
    Npc,
    Trainer {
        trainer_type: TrainerType::ClassTrainer,
        class_specialization: Some(CharacterClass::Knight),
        dialogue_lines: vec![
            "Greetings, warrior. Ready to hone your skills?".to_string(),
            "The path of the Knight requires discipline and strength.".to_string(),
            "I have new techniques to teach you, if you prove yourself worthy.".to_string(),
        ],
        available_ability_quests: vec![
            QUEST_KNIGHT_SHIELD_BASH,
            QUEST_KNIGHT_TAUNT,
            QUEST_KNIGHT_WHIRLWIND,
            // ... all knight ability quests
        ],
    },
    NpcName("Sir Aldric".to_string()),
    Interactable,
    Position(Vec2::new(100.0, 50.0)),  // Near spawn
    VisualShape {
        shape_type: ShapeType::Square,
        color: [0.8, 0.6, 0.2, 1.0],  // Golden knight
        size: 40.0,
    },
));
```

---

## Implementation Phases

### Phase 1: Foundation - Experience & Leveling

**Goal:** Get basic XP system working with level-ups

**Tasks:**
1. Add `Experience` component to shared components
2. Create `gain_experience()` system in combat.rs
3. Create `check_level_up()` system
4. Implement XP formula and level-up stat scaling
5. Add `LevelUpEvent` message and handler
6. Create basic XP bar in UI
7. Test: Kill enemies → gain XP → level up → stats increase

**Success Criteria:**
- Killing enemies grants XP
- XP bar fills correctly
- Level-ups increase stats
- Notifications show on level-up

---

### Phase 2: Content Creation - Expanded Abilities

**Goal:** Create full ability progressions for all 3 classes

**Tasks:**
1. Design 6-8 abilities per class (Knight, Mage, Rogue)
2. Add abilities to AbilityDatabase with unlock requirements
3. Create ability unlock quests (one per ability)
4. Add quests to QuestDatabase
5. Test: Abilities show correct unlock level, require quests

**Success Criteria:**
- Each class has 6-8 unique abilities
- Abilities have varied mechanics (damage, buff, debuff, mobility)
- Each ability has associated unlock quest
- Unlock requirements properly defined

---

### Phase 3: Trainers & World Building

**Goal:** Create immersive trainer NPC system

**Tasks:**
1. Add `Trainer` component to shared components
2. Spawn trainer NPCs in world (one per class + general tutor)
3. Modify `handle_interact_npc()` to detect Trainer component
4. Create `TrainerDialogueEvent` message and handler
5. Build trainer dialogue UI window
6. Implement ability learning flow (talk to trainer → accept quest → learn)
7. Test: Talk to trainer → see available quests → complete → learn ability

**Success Criteria:**
- Trainer NPCs exist in world
- Dialogue window shows on interaction
- Available quests listed correctly
- Learning flow works end-to-end

---

### Phase 4: Spellbook UI

**Goal:** Create UI for managing abilities and hotbar

**Tasks:**
1. Design spellbook window showing learned + locked abilities
2. Implement drag-and-drop from spellbook to hotbar
3. Show ability tooltips (description, cooldown, mana cost, range)
4. Add spellbook toggle button to action menu
5. Display unlock requirements for locked abilities
6. Test: Drag abilities to hotbar → use from hotbar

**Success Criteria:**
- Spellbook shows all class abilities
- Locked abilities greyed out with requirements shown
- Drag-and-drop works smoothly
- Tooltips informative and accurate

---

### Phase 5: Weapon Proficiency

**Goal:** Implement weapon skill-up system

**Tasks:**
1. Add `WeaponProficiencyExp` component
2. Create `gain_weapon_proficiency()` system in combat.rs
3. Modify damage calculation to include proficiency bonus
4. Add proficiency leveling logic
5. Create `ProficiencyLevelUpEvent` message and handler
6. Add proficiency display to character sheet UI
7. Implement weapon proficiency requirements on items
8. Test: Use weapons → gain proficiency → see damage increase

**Success Criteria:**
- Hitting enemies with weapon grants proficiency XP
- Killing enemies grants bonus proficiency XP
- Proficiency levels up at 100 XP
- Proficiency bonus applies to damage
- UI shows current proficiency levels

---

### Phase 6: Armor Proficiency & Passives

**Goal:** Implement armor skill-up and passive unlock system

**Tasks:**
1. Add `ArmorType` enum to components
2. Add armor types to all armor items in ItemDatabase
3. Add `ArmorProficiencyExp` and `ArmorProficiency` components
4. Add `UnlockedArmorPassives` component
5. Create `ArmorPassiveDatabase` resource
6. Define 3-4 passives per armor type
7. Create `gain_armor_proficiency_from_damage()` system
8. Create `check_armor_passive_unlocks()` system
9. Apply passive effects in relevant systems (combat, movement, etc.)
10. Add passive display to character sheet UI
11. Test: Take damage → gain proficiency → unlock passives → see effects

**Success Criteria:**
- Taking damage grants armor proficiency
- Proficiency unlocks passive skills at milestones
- Passive effects applied correctly
- UI shows unlocked passives
- Set bonuses calculated and applied

---

### Phase 7: Tutorial Quests

**Goal:** Create guided tutorial experience

**Tasks:**
1. Create General Tutor NPC
2. Write tutorial quest chain (5-7 quests)
   - First Steps (movement, inventory)
   - Combat Training (auto-attack, targeting)
   - Meet Your Trainer (introduce class trainer)
   - (Class-specific ability quests created in Phase 2)
3. Add Weapon Master NPC with proficiency tutorial quest
4. Add Armor Instructor NPC with armor tutorial quest
5. Implement tutorial hint system (contextual notifications)
6. Test: New character follows tutorial naturally

**Success Criteria:**
- Tutorial quests guide player through all systems
- NPCs have personality and purpose
- Quest flow feels natural, not forced
- Hints appear at appropriate times

---

### Phase 8: UI Polish

**Goal:** Make all progression systems visible and intuitive

**Tasks:**
1. Add XP bar with tooltips
2. Enhance character sheet with proficiency displays
3. Add ability tooltips throughout UI
4. Create tutorial hint notification system
5. Add level-up animation/effect
6. Add proficiency level-up notifications
7. Add passive unlock notifications
8. Polish spellbook UI
9. Test: All progression feels satisfying and clear

**Success Criteria:**
- Players always know current XP/level progress
- Proficiency levels easy to check
- Unlocks feel rewarding (visual feedback)
- UI is polished and professional

---

### Phase 9: Database Persistence

**Goal:** Save all progression data

**Tasks:**
1. Extend character database schema
2. Add save functions for Experience, Proficiencies, UnlockedPassives
3. Add load functions to restore progression on character spawn
4. Test save/load cycle thoroughly
5. Add migration for existing characters

**Database Schema Changes:**
```sql
ALTER TABLE characters ADD COLUMN experience_xp INTEGER DEFAULT 0;
ALTER TABLE characters ADD COLUMN experience_to_next INTEGER DEFAULT 100;

CREATE TABLE character_weapon_proficiency (
    character_id INTEGER PRIMARY KEY,
    sword_level INTEGER DEFAULT 1,
    sword_xp INTEGER DEFAULT 0,
    dagger_level INTEGER DEFAULT 1,
    dagger_xp INTEGER DEFAULT 0,
    -- ... etc for all weapon types
    FOREIGN KEY (character_id) REFERENCES characters(id)
);

CREATE TABLE character_armor_proficiency (
    character_id INTEGER PRIMARY KEY,
    light_level INTEGER DEFAULT 1,
    light_xp INTEGER DEFAULT 0,
    medium_level INTEGER DEFAULT 1,
    medium_xp INTEGER DEFAULT 0,
    heavy_level INTEGER DEFAULT 1,
    heavy_xp INTEGER DEFAULT 0,
    FOREIGN KEY (character_id) REFERENCES characters(id)
);

CREATE TABLE character_unlocked_passives (
    character_id INTEGER,
    passive_id INTEGER,
    PRIMARY KEY (character_id, passive_id),
    FOREIGN KEY (character_id) REFERENCES characters(id)
);
```

**Success Criteria:**
- All progression saves correctly
- All progression loads correctly
- No data loss on logout/login
- Existing characters migrated successfully

---

### Phase 10: Testing & Balance

**Goal:** Ensure systems are fun and balanced

**Tasks:**
1. Playtest full progression 1-20
2. Balance XP requirements
3. Balance proficiency gain rates
4. Balance ability power levels
5. Balance passive skill effects
6. Tune quest rewards
7. Adjust tutorial pacing
8. Fix any bugs discovered

**Success Criteria:**
- Progression feels rewarding, not grindy
- All systems work together smoothly
- Tutorial is clear and helpful
- No major bugs or exploits

---

## Database Schema

### Complete Schema for Progression Systems

```sql
-- === CHARACTER PROGRESSION ===

-- Experience and level tracking
ALTER TABLE characters ADD COLUMN level INTEGER DEFAULT 1;
ALTER TABLE characters ADD COLUMN experience_xp INTEGER DEFAULT 0;
ALTER TABLE characters ADD COLUMN experience_to_next INTEGER DEFAULT 100;

-- Weapon proficiency
CREATE TABLE character_weapon_proficiency (
    character_id INTEGER PRIMARY KEY,
    sword_level INTEGER DEFAULT 1,
    sword_xp INTEGER DEFAULT 0,
    dagger_level INTEGER DEFAULT 1,
    dagger_xp INTEGER DEFAULT 0,
    staff_level INTEGER DEFAULT 1,
    staff_xp INTEGER DEFAULT 0,
    mace_level INTEGER DEFAULT 1,
    mace_xp INTEGER DEFAULT 0,
    bow_level INTEGER DEFAULT 1,
    bow_xp INTEGER DEFAULT 0,
    axe_level INTEGER DEFAULT 1,
    axe_xp INTEGER DEFAULT 0,
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
);

-- Armor proficiency
CREATE TABLE character_armor_proficiency (
    character_id INTEGER PRIMARY KEY,
    light_level INTEGER DEFAULT 1,
    light_xp INTEGER DEFAULT 0,
    medium_level INTEGER DEFAULT 1,
    medium_xp INTEGER DEFAULT 0,
    heavy_level INTEGER DEFAULT 1,
    heavy_xp INTEGER DEFAULT 0,
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
);

-- Unlocked armor passive skills
CREATE TABLE character_unlocked_passives (
    character_id INTEGER,
    passive_id INTEGER,
    unlocked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (character_id, passive_id),
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
);

-- Learned abilities (already exists, but documented here)
CREATE TABLE character_learned_abilities (
    character_id INTEGER,
    ability_id INTEGER,
    learned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (character_id, ability_id),
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
);

-- Tutorial progress tracking
CREATE TABLE character_tutorial_hints (
    character_id INTEGER,
    hint_id STRING,
    shown_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (character_id, hint_id),
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
);
```

### Save/Load Functions

```rust
// database.rs

pub fn save_character_progression(
    conn: &Connection,
    character_id: i64,
    experience: &Experience,
    weapon_prof: &WeaponProficiency,
    weapon_prof_exp: &WeaponProficiencyExp,
    armor_prof: &ArmorProficiency,
    armor_prof_exp: &ArmorProficiencyExp,
    unlocked_passives: &UnlockedArmorPassives,
) -> Result<()> {
    // Update experience
    conn.execute(
        "UPDATE characters SET experience_xp = ?1, experience_to_next = ?2 WHERE id = ?3",
        params![experience.current_xp, experience.xp_to_next_level, character_id],
    )?;

    // Save weapon proficiency
    conn.execute(
        "INSERT OR REPLACE INTO character_weapon_proficiency
         (character_id, sword_level, sword_xp, dagger_level, dagger_xp, ...)
         VALUES (?1, ?2, ?3, ?4, ?5, ...)",
        params![
            character_id,
            weapon_prof.sword, weapon_prof_exp.sword_xp,
            weapon_prof.dagger, weapon_prof_exp.dagger_xp,
            // ... etc
        ],
    )?;

    // Save armor proficiency
    // ... similar pattern

    // Save unlocked passives
    conn.execute("DELETE FROM character_unlocked_passives WHERE character_id = ?1", params![character_id])?;
    for passive_id in &unlocked_passives.passives {
        conn.execute(
            "INSERT INTO character_unlocked_passives (character_id, passive_id) VALUES (?1, ?2)",
            params![character_id, passive_id],
        )?;
    }

    Ok(())
}

pub fn load_character_progression(
    conn: &Connection,
    character_id: i64,
) -> Result<CharacterProgressionData> {
    // Load all progression data
    // Return struct containing all components
}
```

---

## UI Framework

### Reusable UI Components

#### 1. Progress Bar Component

```rust
// ui.rs
pub fn draw_progress_bar(
    ui: &mut egui::Ui,
    current: u32,
    max: u32,
    label: &str,
    color: egui::Color32,
) {
    let percentage = current as f32 / max as f32;

    ui.horizontal(|ui| {
        ui.label(label);
        let progress_bar = egui::ProgressBar::new(percentage)
            .fill(color)
            .text(format!("{}/{}", current, max));
        ui.add(progress_bar);
    });
}

// Usage:
draw_progress_bar(ui, experience.current_xp, experience.xp_to_next_level, "XP", egui::Color32::GOLD);
draw_progress_bar(ui, sword_xp, 100, "Sword", egui::Color32::LIGHT_BLUE);
```

#### 2. Ability Tooltip Component

```rust
pub fn show_ability_tooltip(
    ui: &mut egui::Ui,
    ability: &AbilityDefinition,
    learned: bool,
) {
    ui.vertical(|ui| {
        // Name
        ui.heading(&ability.name);
        ui.separator();

        // Description
        ui.label(&ability.description);
        ui.add_space(5.0);

        // Stats
        ui.label(format!("Damage: {}x", ability.damage_multiplier));
        ui.label(format!("Cooldown: {:.1}s", ability.cooldown));
        ui.label(format!("Mana Cost: {}", ability.mana_cost));
        ui.label(format!("Range: {}px", ability.range));

        // Unlock requirement (if not learned)
        if !learned {
            ui.add_space(5.0);
            ui.separator();
            ui.colored_label(egui::Color32::YELLOW, "Requirements:");
            ui.label(format!("Level {}", ability.unlock_requirement.required_level));
            if let Some(quest_id) = ability.unlock_requirement.prerequisite_quest_id {
                ui.label(format!("Complete quest #{}", quest_id));
            }
        }
    });
}
```

#### 3. Notification System

```rust
// ui.rs
pub struct NotificationQueue {
    notifications: VecDeque<Notification>,
}

pub struct Notification {
    message: String,
    notification_type: NotificationType,
    timestamp: Instant,
    duration: f32,
}

pub fn show_notifications(ui_state: &mut UiState, ctx: &egui::Context) {
    let current_time = Instant::now();

    egui::Area::new("notifications")
        .fixed_pos([800.0, 50.0])
        .show(ctx, |ui| {
            for notification in &ui_state.notifications {
                if current_time.duration_since(notification.timestamp).as_secs_f32() < notification.duration {
                    let color = match notification.notification_type {
                        NotificationType::LevelUp => egui::Color32::GOLD,
                        NotificationType::AbilityUnlocked => egui::Color32::LIGHT_BLUE,
                        NotificationType::PassiveUnlocked => egui::Color32::GREEN,
                        NotificationType::Hint => egui::Color32::WHITE,
                    };

                    ui.colored_label(color, &notification.message);
                }
            }
        });
}
```

#### 4. Character Sheet Window

```rust
pub fn character_sheet_window(
    ctx: &egui::Context,
    character: &Character,
    experience: &Experience,
    stats: &CombatStats,
    health: &Health,
    mana: &Mana,
    weapon_prof: &WeaponProficiency,
    armor_prof: &ArmorProficiency,
    unlocked_passives: &UnlockedArmorPassives,
    passive_db: &ArmorPassiveDatabase,
) {
    egui::Window::new("Character Sheet")
        .show(ctx, |ui| {
            // Basic info
            ui.heading(&character.name);
            ui.label(format!("Level {} {}", character.level, character.class.as_str()));
            ui.separator();

            // Experience
            draw_progress_bar(ui, experience.current_xp, experience.xp_to_next_level, "Experience", egui::Color32::GOLD);
            ui.add_space(10.0);

            // Stats
            ui.label(format!("Health: {}/{}", health.current, health.max));
            ui.label(format!("Mana: {}/{}", mana.current, mana.max));
            ui.label(format!("Attack: {:.1}", stats.attack_power));
            ui.label(format!("Defense: {:.1}", stats.defense));
            ui.label(format!("Crit Chance: {:.1}%", stats.crit_chance * 100.0));
            ui.add_space(10.0);

            // Weapon proficiency
            ui.heading("Weapon Proficiency");
            draw_progress_bar(ui, 0, 100, "Sword", egui::Color32::LIGHT_BLUE);  // Would need XP tracking
            ui.label(format!("  Level {} (+{}% damage)", weapon_prof.sword, weapon_prof.sword * 2));
            // ... repeat for all weapons
            ui.add_space(10.0);

            // Armor proficiency & passives
            ui.heading("Armor Proficiency & Passives");
            draw_progress_bar(ui, 0, 100, "Heavy Armor", egui::Color32::LIGHT_GRAY);
            ui.label(format!("  Level {}", armor_prof.heavy));

            // Show unlocked passives
            for passive_id in &unlocked_passives.passives {
                if let Some(passive) = passive_db.passives.get(passive_id) {
                    ui.label(format!("  ✓ {}: {}", passive.name, passive.description));
                }
            }
        });
}
```

---

## Future Enhancements: Content Pipeline Modularity

While the current data-driven architecture significantly reduces code changes for new content, there's still room for improvement. The following enhancements would make content creation even more streamlined and accessible.

### Problem Statement

Currently, adding new content requires:
1. Editing Rust source files (game_data.rs, components.rs, world.rs)
2. Adding enum variants for new classes/weapon types/armor types
3. Inserting data into HashMaps in Rust code
4. Recompiling the entire project
5. Manual testing to catch data errors

This creates friction for rapid content iteration and increases the risk of typos, missing references, and invalid values.

### Solution: External Content Definition Files

### 1. Content Definition Files (TOML/RON/JSON)

**Goal:** Move all game content out of Rust code and into human-readable data files.

**Architecture:**
```
game_data/
├── classes/
│   ├── knight.toml
│   ├── mage.toml
│   └── rogue.toml
├── abilities/
│   ├── knight_abilities.toml
│   ├── mage_abilities.toml
│   └── rogue_abilities.toml
├── items/
│   ├── weapons.toml
│   └── armor.toml
├── quests/
│   ├── tutorial_quests.toml
│   └── class_quests.toml
├── npcs/
│   └── trainers.toml
└── passives/
    └── armor_passives.toml
```

**Example: Class Definition (TOML)**
```toml
# game_data/classes/knight.toml
[class]
id = "knight"
display_name = "Knight"
description = "A heavily armored warrior specializing in melee combat and defense"

[starting_stats]
health = 120.0
mana = 80.0
attack = 12.0
defense = 8.0
crit_chance = 5.0

[starting_equipment]
weapon = "item_sword_basic"
armor_pieces = ["item_iron_chest", "item_iron_legs"]

[weapon_proficiencies]
sword = 10
mace = 5
axe = 3

[armor_proficiencies]
heavy = 10
medium = 3
light = 1
```

**Example: Ability Definition (TOML)**
```toml
# game_data/abilities/knight_abilities.toml

[[ability]]
id = "ability_heavy_slash"
name = "Heavy Slash"
description = "A powerful sword strike dealing 2x damage"
icon = "heavy_slash.png"

[ability.stats]
damage_multiplier = 2.0
cooldown = 3.0
range = 30.0
mana_cost = 15.0
ability_type = "DirectDamage"

[ability.unlock]
required_level = 1
required_class = "knight"
prerequisite_quest = null  # Starter ability
trainer_npc = "npc_knight_trainer"

[[ability]]
id = "ability_shield_bash"
name = "Shield Bash"
description = "Bash your enemy, stunning them briefly"
icon = "shield_bash.png"

[ability.stats]
damage_multiplier = 1.0
cooldown = 8.0
range = 30.0
mana_cost = 20.0
ability_type = { Debuff = { duration = 2.0, effect = "Stun" } }

[ability.unlock]
required_level = 3
required_class = "knight"
prerequisite_quest = "quest_shield_bash_training"
trainer_npc = "npc_knight_trainer"
```

**Example: Quest Definition (TOML)**
```toml
# game_data/quests/class_quests.toml

[[quest]]
id = "quest_shield_bash_training"
name = "The Shield is Mightier"
description = "Your trainer wants you to demonstrate proficiency with Heavy Slash"
quest_giver = "npc_knight_trainer"

[[quest.objectives]]
type = "KillEnemy"
enemy_type = "enemy_slime"
count = 10

[quest.rewards]
experience = 200
items = []
grants_ability = "ability_shield_bash"
```

**Implementation Approach:**
```rust
// game_data.rs - loader system

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct ClassDefinitionFile {
    class: ClassInfo,
    starting_stats: StartingStats,
    starting_equipment: StartingEquipment,
    weapon_proficiencies: HashMap<String, u32>,
    armor_proficiencies: HashMap<String, u32>,
}

impl AbilityDatabase {
    pub fn load_from_files() -> Result<Self, ContentLoadError> {
        let mut abilities = HashMap::new();

        // Load all TOML files from game_data/abilities/
        for entry in std::fs::read_dir("game_data/abilities")? {
            let path = entry?.path();
            if path.extension() == Some("toml".as_ref()) {
                let content = std::fs::read_to_string(&path)?;
                let file_data: AbilityFile = toml::from_str(&content)?;

                for ability in file_data.abilities {
                    abilities.insert(ability.id, ability.into());
                }
            }
        }

        Ok(Self { abilities })
    }
}
```

**Benefits:**
- No recompilation needed for content changes
- Non-programmers can add/edit content
- Version control shows clear content diffs
- Easier to spot errors in structured data
- Can be edited with any text editor

---

### 2. Content Validation System

**Goal:** Catch content errors before runtime with automated validation.

**Validation Checks:**
```rust
// validation.rs

pub struct ContentValidator {
    errors: Vec<ValidationError>,
    warnings: Vec<ValidationWarning>,
}

impl ContentValidator {
    pub fn validate_all_content() -> ValidationReport {
        let mut validator = Self::new();

        validator.validate_class_files();
        validator.validate_ability_files();
        validator.validate_quest_files();
        validator.validate_item_files();
        validator.validate_npc_files();
        validator.check_references();

        ValidationReport {
            errors: validator.errors,
            warnings: validator.warnings,
        }
    }

    fn check_references(&mut self) {
        // Check that all IDs referenced actually exist
        for ability in &self.abilities {
            // Check trainer NPC exists
            if !self.npcs.contains_key(&ability.unlock.trainer_npc) {
                self.errors.push(ValidationError::MissingReference {
                    source: format!("ability: {}", ability.id),
                    reference_type: "trainer_npc",
                    missing_id: ability.unlock.trainer_npc.clone(),
                });
            }

            // Check quest exists if specified
            if let Some(quest_id) = &ability.unlock.prerequisite_quest {
                if !self.quests.contains_key(quest_id) {
                    self.errors.push(ValidationError::MissingReference {
                        source: format!("ability: {}", ability.id),
                        reference_type: "quest",
                        missing_id: quest_id.clone(),
                    });
                }
            }
        }

        // Check quest objectives reference valid items/enemies
        for quest in &self.quests {
            for objective in &quest.objectives {
                match objective {
                    QuestObjective::ObtainItem { item_id, .. } => {
                        if !self.items.contains_key(item_id) {
                            self.errors.push(ValidationError::MissingReference {
                                source: format!("quest: {}", quest.id),
                                reference_type: "item",
                                missing_id: item_id.clone(),
                            });
                        }
                    }
                    QuestObjective::KillEnemy { enemy_type, .. } => {
                        if !self.enemies.contains_key(enemy_type) {
                            self.errors.push(ValidationError::MissingReference {
                                source: format!("quest: {}", quest.id),
                                reference_type: "enemy",
                                missing_id: enemy_type.clone(),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn validate_ability_files(&mut self) {
        for ability in &self.abilities {
            // Check required fields
            if ability.name.is_empty() {
                self.errors.push(ValidationError::EmptyField {
                    content_type: "ability",
                    id: ability.id.clone(),
                    field: "name",
                });
            }

            // Check value ranges
            if ability.stats.cooldown < 0.0 {
                self.errors.push(ValidationError::InvalidValue {
                    content_type: "ability",
                    id: ability.id.clone(),
                    field: "cooldown",
                    reason: "cannot be negative",
                });
            }

            if ability.stats.mana_cost < 0.0 {
                self.errors.push(ValidationError::InvalidValue {
                    content_type: "ability",
                    id: ability.id.clone(),
                    field: "mana_cost",
                    reason: "cannot be negative",
                });
            }

            // Warnings for potentially unbalanced values
            if ability.stats.damage_multiplier > 5.0 {
                self.warnings.push(ValidationWarning::UnusualValue {
                    content_type: "ability",
                    id: ability.id.clone(),
                    field: "damage_multiplier",
                    value: ability.stats.damage_multiplier.to_string(),
                    reason: "very high damage multiplier (>5x)",
                });
            }
        }
    }
}
```

**CLI Validation Tool:**
```bash
# Run validation before starting server
$ cargo run --bin validate-content

Validating game content...

[ERROR] ability_teleport: Missing reference to trainer_npc "npc_mage_trainer_typo"
[ERROR] quest_armor_basics: Missing reference to item "item_leather_tunic_typo"
[ERROR] ability_fireball: Invalid value for "cooldown": cannot be negative (-1.0)
[WARNING] ability_mega_slash: Unusual damage_multiplier (10.0) - very high damage multiplier (>5x)

Found 3 errors, 1 warning
Content validation FAILED - fix errors before running server
```

**Benefits:**
- Catch typos immediately
- Prevent broken references
- Warn about balance issues
- Fast feedback loop
- Prevents runtime crashes

---

### 3. Hot-Reload Support

**Goal:** Reload content files without restarting the server (development mode only).

**Implementation:**
```rust
// hot_reload.rs (server only, development builds)

#[cfg(debug_assertions)]
pub fn setup_hot_reload(mut commands: Commands) {
    use notify::{Watcher, RecursiveMode, watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();

    watcher.watch("game_data/", RecursiveMode::Recursive).unwrap();

    commands.insert_resource(ContentWatcher { watcher, rx });
}

#[cfg(debug_assertions)]
pub fn check_content_changes(
    watcher: Res<ContentWatcher>,
    mut ability_db: ResMut<AbilityDatabase>,
    mut quest_db: ResMut<QuestDatabase>,
    mut item_db: ResMut<ItemDatabase>,
) {
    while let Ok(event) = watcher.rx.try_recv() {
        match event {
            DebouncedEvent::Write(path) => {
                info!("Content file changed: {:?}", path);

                // Reload specific database based on file path
                if path.starts_with("game_data/abilities") {
                    match AbilityDatabase::load_from_files() {
                        Ok(new_db) => {
                            *ability_db = new_db;
                            info!("Reloaded ability database");
                        }
                        Err(e) => error!("Failed to reload abilities: {}", e),
                    }
                }
                // ... similar for other content types
            }
            _ => {}
        }
    }
}
```

**Benefits:**
- Instant iteration on content
- No server restarts needed
- Faster balancing and testing
- Development-only feature (disabled in release builds)

---

### 4. Future: Visual Content Editor

**Long-term Goal:** GUI tool for editing game content (much later stage).

**Concept:**
- Electron or Tauri-based desktop application
- Forms for editing classes, abilities, quests
- Dropdown selectors for references (prevents typos)
- Real-time validation as you type
- Visual ability/quest flow editor
- Runs content validation before saving
- Generates TOML files from GUI

**This is a much later enhancement** - only after content definition files are established and the content pipeline is mature.

---

### Explicit Exclusions

**What we are NOT doing:**

1. **Community Mods/Plugins**: Not supported due to balance concerns in an MMO environment. All content must be curated by the development team.

2. **Code Generation**: Avoided for safety and security reasons. Content files are loaded and parsed at runtime, not used to generate Rust code.

3. **Scripting System**: Not in scope for initial implementation. Game logic remains in Rust for performance and safety.

---

### Implementation Priority

If these enhancements are pursued, recommended order:

1. **Phase 1**: Content Definition Files (TOML)
   - Start with abilities and quests (most frequently changed)
   - Keep existing Rust-based databases as fallback
   - Gradual migration

2. **Phase 2**: Content Validation System
   - Build CLI validation tool
   - Integrate into development workflow
   - Add pre-commit hooks

3. **Phase 3**: Hot-Reload Support
   - Development builds only
   - Significantly speeds up content iteration

4. **Phase 4** (Much Later): Visual Content Editor
   - Only after content system is mature and stable
   - Optional convenience tool, not required

---

### Benefits Summary

With content definition files and validation:
- **Adding a new class**: Edit 1 TOML file, run validation, restart server
- **Adding a new ability**: Add entry to TOML, run validation, hot-reload in dev
- **Adding a new quest**: Edit TOML file, automatic reference checking
- **Balancing changes**: Edit numbers in TOML, hot-reload to test immediately
- **No compilation**: Content changes don't require rebuilding Rust code
- **Error prevention**: Validation catches issues before they reach runtime
- **Accessibility**: Non-programmers can contribute content safely

---

## Conclusion

This design provides a **solid, extensible foundation** for skill progression in Eryndor MMO. The data-driven architecture means:

- **Easy Content Addition**: New abilities, quests, classes added via data, not code
- **Minimal Maintenance**: Systems work with any data structure conforming to types
- **Clear Patterns**: Templates make adding content straightforward
- **Scalable**: Architecture supports growth from 3 classes to 10+ without refactoring
- **Immersive**: NPCs, quests, and progression feel integrated into the world

The phased implementation approach ensures each system is fully tested before moving on, reducing the risk of major refactoring later.

The Future Enhancements section outlines a path toward even greater content modularity through external content definition files and validation systems, making content creation accessible and safe without requiring code changes.

**Next Step:** Review this document, then proceed with Phase 1 implementation (Experience & Leveling system).
