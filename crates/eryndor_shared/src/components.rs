use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// PLAYER & CHARACTER COMPONENTS
// ============================================================================

/// Marker for player-controlled entities
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Player;

/// Character data
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Character {
    pub name: String,
    pub class: CharacterClass,
    pub level: u32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CharacterClass {
    #[default]
    Rogue,
    Mage,
    Knight,
}

impl CharacterClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            CharacterClass::Rogue => "Rogue",
            CharacterClass::Mage => "Mage",
            CharacterClass::Knight => "Knight",
        }
    }

    /// Get the starting abilities for this class
    pub fn starting_abilities(&self) -> Vec<u32> {
        match self {
            CharacterClass::Rogue => vec![crate::ABILITY_QUICK_STRIKE],
            CharacterClass::Mage => vec![crate::ABILITY_FIREBALL],
            CharacterClass::Knight => vec![crate::ABILITY_HEAVY_SLASH],
        }
    }

    /// Get the appropriate starting weapon for this class
    pub fn starting_weapon(&self) -> u32 {
        match self {
            CharacterClass::Rogue => crate::ITEM_DAGGER,
            CharacterClass::Mage => crate::ITEM_WAND,
            CharacterClass::Knight => crate::ITEM_SWORD,
        }
    }
}

/// Tracks which client owns this entity
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct OwnedBy(pub Entity);

/// Account ID from database
#[derive(Component, Clone, Copy, Debug)]
pub struct AccountId(pub i64);

// ============================================================================
// SPATIAL COMPONENTS
// ============================================================================

/// World position
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Position(pub Vec2);

impl Default for Position {
    fn default() -> Self {
        Self(Vec2::ZERO)
    }
}

/// Movement velocity
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Velocity(pub Vec2);

impl Default for Velocity {
    fn default() -> Self {
        Self(Vec2::ZERO)
    }
}

/// Movement speed
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct MoveSpeed(pub f32);

impl Default for MoveSpeed {
    fn default() -> Self {
        Self(200.0) // pixels per second
    }
}

// ============================================================================
// COMBAT COMPONENTS
// ============================================================================

/// Health
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn percent(&self) -> f32 {
        if self.max > 0.0 {
            (self.current / self.max).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// Mana/Energy
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Mana {
    pub current: f32,
    pub max: f32,
}

impl Mana {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn percent(&self) -> f32 {
        if self.max > 0.0 {
            (self.current / self.max).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// Combat statistics
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct CombatStats {
    pub attack_power: f32,
    pub defense: f32,
    pub crit_chance: f32,
}

impl Default for CombatStats {
    fn default() -> Self {
        Self {
            attack_power: 10.0,
            defense: 5.0,
            crit_chance: 0.05,
        }
    }
}

/// Current combat target
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct CurrentTarget(pub Option<Entity>);

impl Default for CurrentTarget {
    fn default() -> Self {
        Self(None)
    }
}

/// Marker for entities in combat
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct InCombat(pub bool);

// ============================================================================
// INVENTORY COMPONENTS
// ============================================================================

/// Player inventory
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Inventory {
    pub slots: Vec<Option<ItemStack>>,
    pub max_slots: usize,
}

impl Inventory {
    pub fn new(max_slots: usize) -> Self {
        Self {
            slots: vec![None; max_slots],
            max_slots,
        }
    }

    pub fn add_item(&mut self, item: ItemStack) -> bool {
        // Find first empty slot
        for slot in &mut self.slots {
            if slot.is_none() {
                *slot = Some(item);
                return true;
            }
        }
        false
    }

    pub fn remove_item(&mut self, slot_index: usize) -> Option<ItemStack> {
        if slot_index < self.slots.len() {
            self.slots[slot_index].take()
        } else {
            None
        }
    }

    pub fn has_item(&self, item_id: u32) -> bool {
        self.slots.iter().any(|slot| {
            if let Some(stack) = slot {
                stack.item_id == item_id
            } else {
                false
            }
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ItemStack {
    pub item_id: u32,
    pub quantity: u32,
}

/// Equipment slots
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Equipment {
    pub weapon: Option<u32>, // Item ID
}

impl Default for Equipment {
    fn default() -> Self {
        Self { weapon: None }
    }
}

// ============================================================================
// ABILITY & HOTBAR COMPONENTS
// ============================================================================

/// Player hotbar
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Hotbar {
    pub slots: [Option<HotbarSlot>; 10],
}

impl Default for Hotbar {
    fn default() -> Self {
        Self {
            slots: [None, None, None, None, None, None, None, None, None, None],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum HotbarSlot {
    Ability(u32),
}

/// Learned abilities
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct LearnedAbilities {
    pub abilities: HashSet<u32>,
}

impl Default for LearnedAbilities {
    fn default() -> Self {
        Self {
            abilities: HashSet::new(),
        }
    }
}

impl LearnedAbilities {
    pub fn learn(&mut self, ability_id: u32) {
        self.abilities.insert(ability_id);
    }

    pub fn knows(&self, ability_id: u32) -> bool {
        self.abilities.contains(&ability_id)
    }
}

/// Ability cooldowns (server-side only, not replicated)
#[derive(Component)]
pub struct AbilityCooldowns {
    pub cooldowns: HashMap<u32, Timer>,
}

impl Default for AbilityCooldowns {
    fn default() -> Self {
        Self {
            cooldowns: HashMap::new(),
        }
    }
}

// ============================================================================
// QUEST COMPONENTS
// ============================================================================

/// Player quest log
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct QuestLog {
    pub active_quests: Vec<ActiveQuest>,
    pub completed_quests: HashSet<u32>,
}

impl Default for QuestLog {
    fn default() -> Self {
        Self {
            active_quests: Vec::new(),
            completed_quests: HashSet::new(),
        }
    }
}

impl QuestLog {
    pub fn has_active_quest(&self, quest_id: u32) -> bool {
        self.active_quests.iter().any(|q| q.quest_id == quest_id)
    }

    pub fn has_completed_quest(&self, quest_id: u32) -> bool {
        self.completed_quests.contains(&quest_id)
    }

    pub fn can_accept_quest(&self, quest_id: u32) -> bool {
        !self.has_active_quest(quest_id) && !self.has_completed_quest(quest_id)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActiveQuest {
    pub quest_id: u32,
    pub progress: Vec<u32>,
}

// ============================================================================
// NPC COMPONENTS
// ============================================================================

/// Marker for NPCs
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Npc;

/// NPC that gives quests
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct QuestGiver {
    pub available_quests: Vec<u32>,
}

/// NPC name
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct NpcName(pub String);

// ============================================================================
// ENEMY COMPONENTS
// ============================================================================

/// Marker for enemy entities
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Enemy;

/// Enemy type identifier
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct EnemyType(pub u32);

/// Enemy AI state
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum AiState {
    Idle,
    Chasing(Entity),
    Attacking(Entity),
}

impl Default for AiState {
    fn default() -> Self {
        Self::Idle
    }
}

// ============================================================================
// INTERACTION COMPONENTS
// ============================================================================

/// Marks entities that can be interacted with
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Interactable {
    pub interaction_radius: f32,
    pub interaction_type: InteractionType,
}

impl Interactable {
    pub fn new(interaction_type: InteractionType, interaction_radius: f32) -> Self {
        Self {
            interaction_radius,
            interaction_type,
        }
    }

    pub fn npc() -> Self {
        Self::new(InteractionType::NpcDialogue, 30.0)
    }

    pub fn item() -> Self {
        Self::new(InteractionType::ItemPickup, 30.0)
    }

    pub fn enemy() -> Self {
        Self::new(InteractionType::Enemy, 30.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionType {
    NpcDialogue,
    ItemPickup,
    Enemy,
    Harvest,
    Door,
    LoreObject,
}

// ============================================================================
// WORLD ITEM COMPONENTS
// ============================================================================

/// Marker for items in the world that can be picked up
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct WorldItem {
    pub item_id: u32,
}

/// Visual representation data
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct VisualShape {
    pub shape_type: ShapeType,
    pub color: [f32; 4], // RGBA
    pub size: f32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum ShapeType {
    Circle,
    Triangle,
    Square,
    Diamond,
}
