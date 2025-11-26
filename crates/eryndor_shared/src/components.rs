use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use avian2d::prelude::*;

// ============================================================================
// PHYSICS & COLLISION LAYERS
// ============================================================================

/// Defines collision layers for physics interactions
/// This controls which entities can collide with each other
#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    #[default]
    Player,      // Player characters
    Enemy,       // Enemy NPCs
    Npc,         // Friendly NPCs (non-combat)
    WorldItem,   // Items that can be picked up
    Environment, // Walls, obstacles, boundaries
}

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

/// Health regeneration rate
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct HealthRegen {
    pub base_regen: f32,  // HP per second
    pub in_combat_multiplier: f32,  // Multiplier when in combat (0.0 = no regen in combat, 0.3 = 30%)
}

impl Default for HealthRegen {
    fn default() -> Self {
        Self {
            base_regen: 5.0,  // 5 HP/second out of combat
            in_combat_multiplier: 0.0,  // No regen in combat by default
        }
    }
}

/// Mana regeneration rate
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct ManaRegen {
    pub base_regen: f32,  // Mana per second
    pub in_combat_multiplier: f32,  // Multiplier when in combat
}

impl Default for ManaRegen {
    fn default() -> Self {
        Self {
            base_regen: 2.0,  // 2 mana/second
            in_combat_multiplier: 0.5,  // 50% regen in combat (slower but not stopped)
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
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, Default)]
#[component(map_entities)]
pub struct CurrentTarget(pub Option<Entity>);

impl bevy::ecs::entity::MapEntities for CurrentTarget {
    fn map_entities<M: bevy::ecs::entity::EntityMapper>(&mut self, entity_mapper: &mut M) {
        if let Some(entity) = &mut self.0 {
            *entity = entity_mapper.get_mapped(*entity);
        }
    }
}

/// Marker for entities in combat
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct InCombat(pub bool);

/// Auto-attack state - enabled/disabled and cooldown timer
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct AutoAttack {
    pub enabled: bool,
    pub cooldown_timer: f32,  // Time until next auto-attack (in seconds)
}

impl Default for AutoAttack {
    fn default() -> Self {
        Self {
            enabled: false,
            cooldown_timer: 0.0,
        }
    }
}

/// Weapon proficiency levels (1-99 like RuneScape)
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct WeaponProficiency {
    pub sword: u32,
    pub dagger: u32,
    pub staff: u32,
    pub wand: u32,
    pub mace: u32,
    pub bow: u32,
    pub axe: u32,
}

impl Default for WeaponProficiency {
    fn default() -> Self {
        Self {
            sword: 1,
            dagger: 1,
            staff: 1,
            wand: 1,
            mace: 1,
            bow: 1,
            axe: 1,
        }
    }
}

// ============================================================================
// INVENTORY COMPONENTS
// ============================================================================

/// Player gold currency
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct Gold(pub u32);

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
#[derive(Component, Serialize, Deserialize, Clone, Debug, Default)]
pub struct Equipment {
    pub weapon: Option<u32>,  // Item ID
    pub helmet: Option<u32>,  // Item ID
    pub chest: Option<u32>,   // Item ID
    pub legs: Option<u32>,    // Item ID
    pub boots: Option<u32>,   // Item ID
}

// ============================================================================
// ABILITY & HOTBAR COMPONENTS
// ============================================================================

/// Player hotbar
#[derive(Component, Serialize, Deserialize, Clone, Debug, Default)]
pub struct Hotbar {
    pub slots: [Option<HotbarSlot>; 10],
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum HotbarSlot {
    Ability(u32),
}

/// Learned abilities
#[derive(Component, Serialize, Deserialize, Clone, Debug, Default)]
pub struct LearnedAbilities {
    pub abilities: HashSet<u32>,
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
#[derive(Component, Default)]
pub struct AbilityCooldowns {
    pub cooldowns: HashMap<u32, Timer>,
}

// ============================================================================
// QUEST COMPONENTS
// ============================================================================

/// Player quest log
#[derive(Component, Serialize, Deserialize, Clone, Debug, Default)]
pub struct QuestLog {
    pub active_quests: Vec<ActiveQuest>,
    pub completed_quests: HashSet<u32>,
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

/// NPC that sells items (weapon trainer, merchant, etc.)
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Trainer {
    pub items_for_sale: Vec<TrainerItem>,
}

/// An item available for purchase from a trainer
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrainerItem {
    pub item_id: u32,
    pub cost: u32, // Gold cost
}

// ============================================================================
// ENEMY COMPONENTS
// ============================================================================

/// Marker for enemy entities
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Enemy;

/// Enemy name (replicated to clients for display)
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct EnemyName(pub String);

/// Enemy type identifier
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct EnemyType(pub u32);

/// Enemy AI state
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, Default)]
#[component(map_entities)]
pub enum AiState {
    #[default]
    Idle,
    Chasing(Entity),
    Attacking(Entity),
}

/// Loot table for enemies - defines what they drop on death
#[derive(Component, Serialize, Deserialize, Clone, Debug, Default)]
pub struct LootTable {
    pub gold_min: u32,
    pub gold_max: u32,
    pub items: Vec<LootItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LootItem {
    pub item_id: u32,
    #[serde(default = "default_drop_chance")]
    pub drop_chance: f32, // 0.0 to 1.0
    #[serde(default = "default_quantity_min")]
    pub quantity_min: u32,
    #[serde(default = "default_quantity_max")]
    pub quantity_max: u32,
}

fn default_drop_chance() -> f32 { 1.0 }
fn default_quantity_min() -> u32 { 1 }
fn default_quantity_max() -> u32 { 1 }

impl bevy::ecs::entity::MapEntities for AiState {
    fn map_entities<M: bevy::ecs::entity::EntityMapper>(&mut self, entity_mapper: &mut M) {
        match self {
            AiState::Chasing(entity) => *entity = entity_mapper.get_mapped(*entity),
            AiState::Attacking(entity) => *entity = entity_mapper.get_mapped(*entity),
            AiState::Idle => {},
        }
    }
}

/// Tracks delay before AI activates (for newly spawned enemies)
/// This prevents combat events from being sent before entity replication completes
#[derive(Component)]
pub struct AiActivationDelay {
    pub timer: Timer,
}

impl Default for AiActivationDelay {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Once), // 500ms delay ensures entity replication completes
        }
    }
}

/// Per-enemy aggro and leash range configuration
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct AggroRange {
    pub aggro: f32,
    pub leash: f32,
}

impl Default for AggroRange {
    fn default() -> Self {
        Self {
            aggro: 150.0,
            leash: 300.0,
        }
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

    pub fn loot_container() -> Self {
        Self::new(InteractionType::LootContainer, 40.0)
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
    LootContainer,
}

// ============================================================================
// PROGRESSION COMPONENTS
// ============================================================================

/// Experience and leveling
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Experience {
    pub current_xp: u32,
    pub xp_to_next_level: u32,
}

impl Experience {
    pub fn new(level: u32) -> Self {
        Self {
            current_xp: 0,
            xp_to_next_level: Self::xp_for_level(level + 1),
        }
    }

    /// Calculate XP needed for a given level
    /// Formula: 100 * level^1.5
    pub fn xp_for_level(level: u32) -> u32 {
        (100.0 * (level as f32).powf(1.5)) as u32
    }

    /// Add XP and return true if leveled up
    pub fn add_xp(&mut self, amount: u32, current_level: u32) -> bool {
        self.current_xp += amount;
        if self.current_xp >= self.xp_to_next_level {
            self.current_xp -= self.xp_to_next_level;
            self.xp_to_next_level = Self::xp_for_level(current_level + 2);
            true
        } else {
            false
        }
    }
}

/// Weapon proficiency experience tracking
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct WeaponProficiencyExp {
    pub sword_xp: u32,
    pub dagger_xp: u32,
    pub staff_xp: u32,
    pub wand_xp: u32,
    pub mace_xp: u32,
    pub bow_xp: u32,
    pub axe_xp: u32,
}

impl WeaponProficiencyExp {
    /// Calculate XP required for a given weapon proficiency level
    /// Similar scaling to character XP but faster progression
    pub fn xp_for_level(level: u32) -> u32 {
        if level <= 1 {
            0
        } else {
            // Weapon proficiency levels up faster than character levels
            50 * level * level
        }
    }
}

/// Armor proficiency levels
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct ArmorProficiency {
    pub light: u32,
    pub medium: u32,
    pub heavy: u32,
}

impl Default for ArmorProficiency {
    fn default() -> Self {
        Self {
            light: 1,
            medium: 1,
            heavy: 1,
        }
    }
}

/// Armor proficiency experience tracking
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct ArmorProficiencyExp {
    pub light_xp: u32,
    pub medium_xp: u32,
    pub heavy_xp: u32,
}

/// Unlocked armor passive skills
#[derive(Component, Serialize, Deserialize, Clone, Debug, Default)]
pub struct UnlockedArmorPassives {
    pub passives: HashSet<u32>,
}

// ============================================================================
// WORLD ITEM COMPONENTS
// ============================================================================

/// Marker for items in the world that can be picked up
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct WorldItem {
    pub item_id: u32,
}

/// Gold drop marker - indicates a world item is gold currency
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct GoldDrop(pub u32);

/// Loot container - aggregates all drops from an enemy into a single entity
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct LootContainer {
    pub contents: Vec<LootContents>,
    pub source_name: String, // e.g., "Goblin", "Troll Warrior"
}

/// Individual items within a loot container
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LootContents {
    Gold(u32),
    Item(ItemStack),
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
