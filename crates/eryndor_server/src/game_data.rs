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
        let mut items = HashMap::new();

        // Dagger - High crit, moderate attack
        items.insert(ITEM_DAGGER, ItemDefinition {
            id: ITEM_DAGGER,
            name: "Dagger".to_string(),
            item_type: ItemType::Weapon,
            grants_ability: Some(crate::abilities::ABILITY_QUICK_STRIKE),
            stat_bonuses: ItemStatBonuses {
                attack_power: 8.0,
                crit_chance: 0.10,
                ..Default::default()
            },
        });

        // Wand - High attack, mana bonus
        items.insert(ITEM_WAND, ItemDefinition {
            id: ITEM_WAND,
            name: "Wand".to_string(),
            item_type: ItemType::Weapon,
            grants_ability: Some(crate::abilities::ABILITY_FIREBALL),
            stat_bonuses: ItemStatBonuses {
                attack_power: 12.0,
                max_mana: 20.0,
                ..Default::default()
            },
        });

        // Sword - Balanced attack and defense
        items.insert(ITEM_SWORD, ItemDefinition {
            id: ITEM_SWORD,
            name: "Sword".to_string(),
            item_type: ItemType::Weapon,
            grants_ability: Some(crate::abilities::ABILITY_HEAVY_SLASH),
            stat_bonuses: ItemStatBonuses {
                attack_power: 10.0,
                defense: 2.0,
                ..Default::default()
            },
        });

        // Staff - Highest magic attack, large mana pool
        items.insert(ITEM_STAFF, ItemDefinition {
            id: ITEM_STAFF,
            name: "Staff".to_string(),
            item_type: ItemType::Weapon,
            grants_ability: Some(crate::abilities::ABILITY_ARCANE_BLAST),
            stat_bonuses: ItemStatBonuses {
                attack_power: 14.0,
                max_mana: 30.0,
                ..Default::default()
            },
        });

        // Mace - Moderate attack, good defense
        items.insert(ITEM_MACE, ItemDefinition {
            id: ITEM_MACE,
            name: "Mace".to_string(),
            item_type: ItemType::Weapon,
            grants_ability: Some(crate::abilities::ABILITY_SMITE),
            stat_bonuses: ItemStatBonuses {
                attack_power: 11.0,
                defense: 3.0,
                max_health: 15.0,
                ..Default::default()
            },
        });

        // Bow - Moderate attack, high crit
        items.insert(ITEM_BOW, ItemDefinition {
            id: ITEM_BOW,
            name: "Bow".to_string(),
            item_type: ItemType::Weapon,
            grants_ability: Some(crate::abilities::ABILITY_AIMED_SHOT),
            stat_bonuses: ItemStatBonuses {
                attack_power: 9.0,
                crit_chance: 0.15,
                ..Default::default()
            },
        });

        // Axe - High attack, low crit
        items.insert(ITEM_AXE, ItemDefinition {
            id: ITEM_AXE,
            name: "Axe".to_string(),
            item_type: ItemType::Weapon,
            grants_ability: Some(crate::abilities::ABILITY_RENDING_STRIKE),
            stat_bonuses: ItemStatBonuses {
                attack_power: 13.0,
                crit_chance: 0.05,
                max_health: 10.0,
                ..Default::default()
            },
        });

        // ========== ARMOR - HELMETS ==========

        items.insert(ITEM_LEATHER_CAP, ItemDefinition {
            id: ITEM_LEATHER_CAP,
            name: "Leather Cap".to_string(),
            item_type: ItemType::Helmet,
            grants_ability: None,
            stat_bonuses: ItemStatBonuses {
                defense: 2.0,
                max_health: 5.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_CLOTH_HAT, ItemDefinition {
            id: ITEM_CLOTH_HAT,
            name: "Cloth Hat".to_string(),
            item_type: ItemType::Helmet,
            grants_ability: None,
            stat_bonuses: ItemStatBonuses {
                defense: 1.0,
                max_mana: 10.0,
                attack_power: 2.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_IRON_HELM, ItemDefinition {
            id: ITEM_IRON_HELM,
            name: "Iron Helm".to_string(),
            item_type: ItemType::Helmet,
            grants_ability: None,
            stat_bonuses: ItemStatBonuses {
                defense: 4.0,
                max_health: 10.0,
                ..Default::default()
            },
        });

        // ========== ARMOR - CHEST ==========

        items.insert(ITEM_LEATHER_TUNIC, ItemDefinition {
            id: ITEM_LEATHER_TUNIC,
            name: "Leather Tunic".to_string(),
            item_type: ItemType::Chest,
            grants_ability: None,
            stat_bonuses: ItemStatBonuses {
                defense: 4.0,
                max_health: 10.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_CLOTH_ROBE, ItemDefinition {
            id: ITEM_CLOTH_ROBE,
            name: "Cloth Robe".to_string(),
            item_type: ItemType::Chest,
            grants_ability: None,
            stat_bonuses: ItemStatBonuses {
                defense: 2.0,
                max_mana: 20.0,
                attack_power: 4.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_IRON_CHESTPLATE, ItemDefinition {
            id: ITEM_IRON_CHESTPLATE,
            name: "Iron Chestplate".to_string(),
            item_type: ItemType::Chest,
            grants_ability: None,
            stat_bonuses: ItemStatBonuses {
                defense: 8.0,
                max_health: 20.0,
                ..Default::default()
            },
        });

        // ========== ARMOR - LEGS ==========

        items.insert(ITEM_LEATHER_PANTS, ItemDefinition {
            id: ITEM_LEATHER_PANTS,
            name: "Leather Pants".to_string(),
            item_type: ItemType::Legs,
            grants_ability: None,
            stat_bonuses: ItemStatBonuses {
                defense: 3.0,
                max_health: 8.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_CLOTH_PANTS, ItemDefinition {
            id: ITEM_CLOTH_PANTS,
            name: "Cloth Pants".to_string(),
            item_type: ItemType::Legs,
            grants_ability: None,
            stat_bonuses: ItemStatBonuses {
                defense: 1.5,
                max_mana: 15.0,
                attack_power: 3.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_IRON_GREAVES, ItemDefinition {
            id: ITEM_IRON_GREAVES,
            name: "Iron Greaves".to_string(),
            item_type: ItemType::Legs,
            grants_ability: None,
            stat_bonuses: ItemStatBonuses {
                defense: 6.0,
                max_health: 15.0,
                ..Default::default()
            },
        });

        // ========== ARMOR - BOOTS ==========

        items.insert(ITEM_LEATHER_BOOTS, ItemDefinition {
            id: ITEM_LEATHER_BOOTS,
            name: "Leather Boots".to_string(),
            item_type: ItemType::Boots,
            grants_ability: None,
            stat_bonuses: ItemStatBonuses {
                defense: 1.5,
                max_health: 5.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_CLOTH_SHOES, ItemDefinition {
            id: ITEM_CLOTH_SHOES,
            name: "Cloth Shoes".to_string(),
            item_type: ItemType::Boots,
            grants_ability: None,
            stat_bonuses: ItemStatBonuses {
                defense: 0.5,
                max_mana: 10.0,
                attack_power: 1.0,
                ..Default::default()
            },
        });

        items.insert(ITEM_IRON_BOOTS, ItemDefinition {
            id: ITEM_IRON_BOOTS,
            name: "Iron Boots".to_string(),
            item_type: ItemType::Boots,
            grants_ability: None,
            stat_bonuses: ItemStatBonuses {
                defense: 3.0,
                max_health: 10.0,
                ..Default::default()
            },
        });

        Self { items }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ItemDefinition {
    pub id: u32,
    pub name: String,
    pub item_type: ItemType,
    pub grants_ability: Option<u32>,
    pub stat_bonuses: ItemStatBonuses,
}

/// Stat bonuses provided by an item when equipped
#[derive(Serialize, Deserialize, Clone, Default)]
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

#[derive(Serialize, Deserialize, Clone, Copy)]
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

#[derive(Serialize, Deserialize, Clone)]
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
        let mut enemies = HashMap::new();

        // Level 1: Slime - Weak starter enemy
        enemies.insert(ENEMY_TYPE_SLIME, EnemyDefinition {
            id: ENEMY_TYPE_SLIME,
            name: "Slime".to_string(),
            max_health: 50.0,
            attack_power: 5.0,
            defense: 2.0,
            move_speed: 80.0,
            aggro_range: 150.0,
            leash_range: 300.0,
            respawn_delay: 10.0,
            loot_table: LootTable { gold_min: 3, gold_max: 8, items: vec![] },
            visual: VisualData { shape: "Circle".to_string(), color: [0.2, 0.8, 0.2, 1.0], size: 16.0 },
        });

        // Level 2: Goblin - Weak humanoid
        enemies.insert(ENEMY_TYPE_GOBLIN, EnemyDefinition {
            id: ENEMY_TYPE_GOBLIN,
            name: "Goblin".to_string(),
            max_health: 80.0,
            attack_power: 8.0,
            defense: 3.0,
            move_speed: 90.0,
            aggro_range: 180.0,
            leash_range: 350.0,
            respawn_delay: 10.0,
            loot_table: LootTable { gold_min: 8, gold_max: 15, items: vec![] },
            visual: VisualData { shape: "Square".to_string(), color: [0.4, 0.6, 0.2, 1.0], size: 16.0 },
        });

        // Level 3: Wolf - Fast predator
        enemies.insert(ENEMY_TYPE_WOLF, EnemyDefinition {
            id: ENEMY_TYPE_WOLF,
            name: "Wolf".to_string(),
            max_health: 100.0,
            attack_power: 12.0,
            defense: 4.0,
            move_speed: 120.0,
            aggro_range: 200.0,
            leash_range: 400.0,
            respawn_delay: 10.0,
            loot_table: LootTable { gold_min: 12, gold_max: 20, items: vec![] },
            visual: VisualData { shape: "Circle".to_string(), color: [0.6, 0.5, 0.3, 1.0], size: 16.0 },
        });

        // Level 4: Skeleton - Undead warrior
        enemies.insert(ENEMY_TYPE_SKELETON, EnemyDefinition {
            id: ENEMY_TYPE_SKELETON,
            name: "Skeleton".to_string(),
            max_health: 120.0,
            attack_power: 15.0,
            defense: 5.0,
            move_speed: 85.0,
            aggro_range: 160.0,
            leash_range: 320.0,
            respawn_delay: 10.0,
            loot_table: LootTable { gold_min: 15, gold_max: 25, items: vec![] },
            visual: VisualData { shape: "Square".to_string(), color: [0.9, 0.9, 0.8, 1.0], size: 16.0 },
        });

        // Level 5: Orc - Strong bruiser
        enemies.insert(ENEMY_TYPE_ORC, EnemyDefinition {
            id: ENEMY_TYPE_ORC,
            name: "Orc".to_string(),
            max_health: 150.0,
            attack_power: 18.0,
            defense: 6.0,
            move_speed: 75.0,
            aggro_range: 200.0,
            leash_range: 400.0,
            respawn_delay: 10.0,
            loot_table: LootTable { gold_min: 20, gold_max: 35, items: vec![] },
            visual: VisualData { shape: "Square".to_string(), color: [0.3, 0.5, 0.3, 1.0], size: 16.0 },
        });

        // Level 3: Spider - Fast but fragile
        enemies.insert(ENEMY_TYPE_SPIDER, EnemyDefinition {
            id: ENEMY_TYPE_SPIDER,
            name: "Spider".to_string(),
            max_health: 90.0,
            attack_power: 10.0,
            defense: 3.0,
            move_speed: 110.0,
            aggro_range: 120.0,
            leash_range: 250.0,
            respawn_delay: 10.0,
            loot_table: LootTable { gold_min: 10, gold_max: 18, items: vec![] },
            visual: VisualData { shape: "Circle".to_string(), color: [0.3, 0.1, 0.3, 1.0], size: 16.0 },
        });

        Self { enemies }
    }
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct ZoneDefinition {
    pub zone_id: String,
    pub zone_name: String,
    pub enemy_spawns: Vec<EnemySpawnRegion>,
    pub npc_spawns: Vec<NpcSpawnDef>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EnemySpawnRegion {
    pub region_id: String,
    pub enemy_type: u32,
    pub spawn_points: Vec<Vec2Data>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NpcSpawnDef {
    pub npc_id: u32,
    pub name: String,
    pub npc_type: String,  // "QuestGiver" or "Trainer"
    pub position: Vec2Data,
    pub quests: Vec<u32>,
    pub trainer_items: Vec<TrainerItem>,
    pub visual: VisualData,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
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

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct VisualData {
    pub shape: String,  // "Circle", "Rectangle", etc.
    pub color: [f32; 4],  // RGBA
    pub size: f32,
}

#[derive(Serialize, Deserialize, Clone)]
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
