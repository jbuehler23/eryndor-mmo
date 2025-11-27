use bevy::prelude::*;
use std::collections::HashMap;
use eryndor_shared::*;
use serde::{Serialize, Deserialize};

// ============================================================================
// ITEM DEFINITIONS
// ============================================================================

#[derive(Resource)]
pub struct ItemDatabase {
    pub items: HashMap<u32, ItemDefinition>,
}

impl Default for ItemDatabase {
    fn default() -> Self {
        // Start with empty database - items will be loaded from JSON files
        Self { items: HashMap::new() }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ItemDefinition {
    pub id: u32,
    pub name: String,
    pub item_type: ItemType,
    pub grants_ability: Option<u32>,
    pub stat_bonuses: ItemStatBonuses,
}

/// Stat bonuses provided by an item when equipped
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct ItemStatBonuses {
    pub attack_power: f32,
    pub defense: f32,
    pub max_health: f32,
    pub max_mana: f32,
    pub crit_chance: f32,
}

impl ItemStatBonuses {
    /// Add another set of bonuses to this one
    pub fn add(&mut self, other: &ItemStatBonuses) {
        self.attack_power += other.attack_power;
        self.defense += other.defense;
        self.max_health += other.max_health;
        self.max_mana += other.max_mana;
        self.crit_chance += other.crit_chance;
    }
}

impl ItemDatabase {
    /// Calculate total stat bonuses from equipped items
    pub fn calculate_equipment_bonuses(&self, equipment: &Equipment) -> ItemStatBonuses {
        let mut total = ItemStatBonuses::default();

        // Add bonuses from each equipped item
        if let Some(weapon_id) = equipment.weapon {
            if let Some(item) = self.items.get(&weapon_id) {
                total.add(&item.stat_bonuses);
            }
        }
        if let Some(helmet_id) = equipment.helmet {
            if let Some(item) = self.items.get(&helmet_id) {
                total.add(&item.stat_bonuses);
            }
        }
        if let Some(chest_id) = equipment.chest {
            if let Some(item) = self.items.get(&chest_id) {
                total.add(&item.stat_bonuses);
            }
        }
        if let Some(legs_id) = equipment.legs {
            if let Some(item) = self.items.get(&legs_id) {
                total.add(&item.stat_bonuses);
            }
        }
        if let Some(boots_id) = equipment.boots {
            if let Some(item) = self.items.get(&boots_id) {
                total.add(&item.stat_bonuses);
            }
        }

        total
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum ItemType {
    Weapon,
    Helmet,
    Chest,
    Legs,
    Boots,
    Consumable,
    QuestItem,
}

// ============================================================================
// QUEST DEFINITIONS
// ============================================================================

#[derive(Resource)]
pub struct QuestDatabase {
    pub quests: HashMap<u32, QuestDefinition>,
}

impl Default for QuestDatabase {
    fn default() -> Self {
        let mut quests = HashMap::new();

        // First weapon quest
        quests.insert(QUEST_FIRST_WEAPON, QuestDefinition {
            id: QUEST_FIRST_WEAPON,
            name: "Choose Your Path".to_string(),
            description: "Return to me when you're ready to receive your class weapon and begin your training. As a member of your class, you will wield the weapon that best suits your abilities.".to_string(),
            objectives: vec![QuestObjective::TalkToNpc {
                npc_id: 1, // Elder
            }],
            reward_exp: 100,
            proficiency_requirements: vec![], // No requirements for starter quest
            reward_abilities: vec![], // Weapon grants starter ability
        });

        Self { quests }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuestDefinition {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub objectives: Vec<QuestObjective>,
    pub reward_exp: u32,
    pub proficiency_requirements: Vec<(crate::weapon::WeaponType, u32)>,
    pub reward_abilities: Vec<u32>,
}

#[derive(Serialize, Deserialize, Clone)]
#[derive(Debug)]
#[serde(tag = "type")]
pub enum QuestObjective {
    ObtainItem { item_id: u32, count: u32 },
    KillEnemy { enemy_type: u32, count: u32 },
    TalkToNpc { npc_id: u32 },
}

// ============================================================================
// ENEMY DEFINITIONS
// ============================================================================

#[derive(Resource)]
pub struct EnemyDatabase {
    pub enemies: HashMap<u32, EnemyDefinition>,
}

impl Default for EnemyDatabase {
    fn default() -> Self {
        // Start with empty database - enemies will be loaded from JSON files
        Self { enemies: HashMap::new() }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EnemyDefinition {
    pub id: u32,
    pub name: String,
    pub max_health: f32,
    pub attack_power: f32,
    pub defense: f32,
    pub move_speed: f32,
    #[serde(default = "default_aggro_range")]
    pub aggro_range: f32,
    #[serde(default = "default_leash_range")]
    pub leash_range: f32,
    #[serde(default = "default_respawn_delay")]
    pub respawn_delay: f32,
    #[serde(default)]
    pub loot_table: LootTable,
    #[serde(default)]
    pub visual: VisualData,
}

fn default_aggro_range() -> f32 { 150.0 }
fn default_leash_range() -> f32 { 300.0 }
fn default_respawn_delay() -> f32 { 10.0 }

// ============================================================================
// ZONE & SPAWN DEFINITIONS
// ============================================================================

#[derive(Resource, Default)]
pub struct ZoneDatabase {
    pub zones: HashMap<String, ZoneDefinition>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ZoneDefinition {
    pub zone_id: String,
    pub zone_name: String,
    pub enemy_spawns: Vec<EnemySpawnRegion>,
    pub npc_spawns: Vec<NpcSpawnDef>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EnemySpawnRegion {
    pub region_id: String,
    pub enemy_type: u32,
    pub spawn_points: Vec<Vec2Data>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NpcSpawnDef {
    pub npc_id: u32,
    pub name: String,
    pub npc_type: String,  // "QuestGiver" or "Trainer"
    pub position: Vec2Data,
    pub quests: Vec<u32>,
    pub trainer_items: Vec<TrainerItem>,
    pub visual: VisualData,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Vec2Data {
    pub x: f32,
    pub y: f32,
}

impl From<Vec2Data> for Vec2 {
    fn from(v: Vec2Data) -> Self {
        Vec2::new(v.x, v.y)
    }
}

// LootItem and LootTable are imported from eryndor_shared

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct VisualData {
    pub shape: String,  // "Circle", "Rectangle", etc.
    pub color: [f32; 4],  // RGBA
    pub size: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrainerItem {
    pub item_id: u32,
    pub cost: u32,
}

// ============================================================================
// TRAINER DEFINITIONS
// ============================================================================

#[derive(Resource)]
pub struct TrainerDatabase {
    pub trainers: HashMap<String, TrainerDefinition>,
}

impl Default for TrainerDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl TrainerDatabase {
    pub fn new() -> Self {
        let mut trainers = HashMap::new();

        // Weapon Trainer - Sells all weapons
        trainers.insert("Weapon Master".to_string(), TrainerDefinition {
            name: "Weapon Master".to_string(),
            items: vec![
                TrainerItem { item_id: ITEM_DAGGER, cost: 50 },
                TrainerItem { item_id: ITEM_SWORD, cost: 75 },
                TrainerItem { item_id: ITEM_WAND, cost: 100 },
                TrainerItem { item_id: ITEM_STAFF, cost: 150 },
                TrainerItem { item_id: ITEM_MACE, cost: 125 },
                TrainerItem { item_id: ITEM_BOW, cost: 100 },
                TrainerItem { item_id: ITEM_AXE, cost: 125 },
            ],
        });

        Self { trainers }
    }
}

pub struct TrainerDefinition {
    pub name: String,
    pub items: Vec<TrainerItem>,
}
