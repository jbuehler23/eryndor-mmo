use bevy::prelude::*;
use std::collections::HashMap;
use eryndor_shared::*;

// ============================================================================
// ABILITY DEFINITIONS
// ============================================================================

#[derive(Resource)]
pub struct AbilityDatabase {
    pub abilities: HashMap<u32, AbilityDefinition>,
}

impl Default for AbilityDatabase {
    fn default() -> Self {
        let mut abilities = HashMap::new();

        // Rogue - Quick Strike (Dagger)
        abilities.insert(ABILITY_QUICK_STRIKE, AbilityDefinition {
            id: ABILITY_QUICK_STRIKE,
            name: "Quick Strike".to_string(),
            damage_multiplier: 1.0,
            cooldown: 1.0, // Fast attack
            range: MELEE_RANGE,
            mana_cost: 10.0,
        });

        // Mage - Fireball (Wand)
        abilities.insert(ABILITY_FIREBALL, AbilityDefinition {
            id: ABILITY_FIREBALL,
            name: "Fireball".to_string(),
            damage_multiplier: 1.5,
            cooldown: 2.5,
            range: RANGED_RANGE,
            mana_cost: 25.0,
        });

        // Knight - Heavy Slash (Sword)
        abilities.insert(ABILITY_HEAVY_SLASH, AbilityDefinition {
            id: ABILITY_HEAVY_SLASH,
            name: "Heavy Slash".to_string(),
            damage_multiplier: 2.0,
            cooldown: 3.0, // Slow but powerful
            range: MELEE_RANGE,
            mana_cost: 15.0,
        });

        Self { abilities }
    }
}

pub struct AbilityDefinition {
    pub id: u32,
    pub name: String,
    pub damage_multiplier: f32,
    pub cooldown: f32,
    pub range: f32,
    pub mana_cost: f32,
}

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

        // Dagger
        items.insert(ITEM_DAGGER, ItemDefinition {
            id: ITEM_DAGGER,
            name: "Dagger".to_string(),
            item_type: ItemType::Weapon,
            grants_ability: Some(ABILITY_QUICK_STRIKE),
        });

        // Wand
        items.insert(ITEM_WAND, ItemDefinition {
            id: ITEM_WAND,
            name: "Wand".to_string(),
            item_type: ItemType::Weapon,
            grants_ability: Some(ABILITY_FIREBALL),
        });

        // Sword
        items.insert(ITEM_SWORD, ItemDefinition {
            id: ITEM_SWORD,
            name: "Sword".to_string(),
            item_type: ItemType::Weapon,
            grants_ability: Some(ABILITY_HEAVY_SLASH),
        });

        Self { items }
    }
}

pub struct ItemDefinition {
    pub id: u32,
    pub name: String,
    pub item_type: ItemType,
    pub grants_ability: Option<u32>,
}

pub enum ItemType {
    Weapon,
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
        });

        Self { quests }
    }
}

pub struct QuestDefinition {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub objectives: Vec<QuestObjective>,
    pub reward_exp: u32,
}

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

        enemies.insert(ENEMY_TYPE_SLIME, EnemyDefinition {
            id: ENEMY_TYPE_SLIME,
            name: "Slime".to_string(),
            max_health: 50.0,
            attack_power: 5.0,
            defense: 2.0,
            move_speed: 80.0,
        });

        Self { enemies }
    }
}

pub struct EnemyDefinition {
    pub id: u32,
    pub name: String,
    pub max_health: f32,
    pub attack_power: f32,
    pub defense: f32,
    pub move_speed: f32,
}
