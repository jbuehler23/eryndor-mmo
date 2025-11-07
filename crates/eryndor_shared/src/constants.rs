use bevy::prelude::*;

// ============================================================================
// NETWORK CONSTANTS
// ============================================================================

pub const SERVER_PORT: u16 = 5000;
pub const SERVER_ADDR: &str = "127.0.0.1";

// ============================================================================
// GAME CONSTANTS
// ============================================================================

pub const WORLD_WIDTH: f32 = 2000.0;
pub const WORLD_HEIGHT: f32 = 2000.0;

pub const PLAYER_SIZE: f32 = 20.0;
pub const NPC_SIZE: f32 = 25.0;
pub const ENEMY_SIZE: f32 = 18.0;
pub const ITEM_SIZE: f32 = 12.0;

pub const DEFAULT_MOVE_SPEED: f32 = 200.0;
pub const SPRINT_MOVE_SPEED: f32 = 350.0;

pub const INTERACTION_RANGE: f32 = 50.0;
pub const PICKUP_RANGE: f32 = 40.0;

pub const MAX_INVENTORY_SLOTS: usize = 20;

// ============================================================================
// COMBAT CONSTANTS
// ============================================================================

pub const MELEE_RANGE: f32 = 30.0;
pub const RANGED_RANGE: f32 = 200.0;

pub const AGGRO_RANGE: f32 = 150.0;
pub const LEASH_RANGE: f32 = 300.0;

// ============================================================================
// ABILITY IDS
// ============================================================================

// Rogue abilities (Dagger)
pub const ABILITY_QUICK_STRIKE: u32 = 1;

// Mage abilities (Wand)
pub const ABILITY_FIREBALL: u32 = 2;

// Knight abilities (Sword)
pub const ABILITY_HEAVY_SLASH: u32 = 3;

// ============================================================================
// ITEM IDS
// ============================================================================

// Weapons
pub const ITEM_DAGGER: u32 = 1;
pub const ITEM_WAND: u32 = 2;
pub const ITEM_SWORD: u32 = 3;

// Armor - Helmets
pub const ITEM_LEATHER_CAP: u32 = 10;
pub const ITEM_CLOTH_HAT: u32 = 11;
pub const ITEM_IRON_HELM: u32 = 12;

// Armor - Chest
pub const ITEM_LEATHER_TUNIC: u32 = 20;
pub const ITEM_CLOTH_ROBE: u32 = 21;
pub const ITEM_IRON_CHESTPLATE: u32 = 22;

// Armor - Legs
pub const ITEM_LEATHER_PANTS: u32 = 30;
pub const ITEM_CLOTH_PANTS: u32 = 31;
pub const ITEM_IRON_GREAVES: u32 = 32;

// Armor - Boots
pub const ITEM_LEATHER_BOOTS: u32 = 40;
pub const ITEM_CLOTH_SHOES: u32 = 41;
pub const ITEM_IRON_BOOTS: u32 = 42;

// ============================================================================
// QUEST IDS
// ============================================================================

pub const QUEST_FIRST_WEAPON: u32 = 1;

// ============================================================================
// ENEMY TYPES
// ============================================================================

pub const ENEMY_TYPE_SLIME: u32 = 1;

// ============================================================================
// SPAWN LOCATIONS
// ============================================================================

pub const SPAWN_POINT: Vec2 = Vec2::new(0.0, 0.0);
pub const NPC_POSITION: Vec2 = Vec2::new(0.0, -20.0);  // 20 pixels below spawn point

// Item spawn positions
pub const ITEM_DAGGER_POS: Vec2 = Vec2::new(-80.0, -80.0);
pub const ITEM_WAND_POS: Vec2 = Vec2::new(0.0, -80.0);
pub const ITEM_SWORD_POS: Vec2 = Vec2::new(80.0, -80.0);

// Enemy spawn positions
pub const ENEMY_SPAWN_1: Vec2 = Vec2::new(150.0, 150.0);
pub const ENEMY_SPAWN_2: Vec2 = Vec2::new(-150.0, 150.0);
pub const ENEMY_SPAWN_3: Vec2 = Vec2::new(0.0, 200.0);

// ============================================================================
// UI CONSTANTS
// ============================================================================

pub const UI_HEALTH_BAR_WIDTH: f32 = 100.0;
pub const UI_HEALTH_BAR_HEIGHT: f32 = 12.0;

pub const UI_HOTBAR_SLOT_SIZE: f32 = 40.0;
pub const UI_HOTBAR_SPACING: f32 = 5.0;

// ============================================================================
// COLORS
// ============================================================================

pub const COLOR_PLAYER: [f32; 4] = [0.2, 0.6, 1.0, 1.0]; // Blue
pub const COLOR_NPC: [f32; 4] = [0.2, 1.0, 0.2, 1.0]; // Green
pub const COLOR_ENEMY: [f32; 4] = [1.0, 0.2, 0.2, 1.0]; // Red
pub const COLOR_ITEM_DAGGER: [f32; 4] = [0.8, 0.8, 0.2, 1.0]; // Yellow
pub const COLOR_ITEM_WAND: [f32; 4] = [0.8, 0.2, 0.8, 1.0]; // Purple
pub const COLOR_ITEM_SWORD: [f32; 4] = [0.7, 0.7, 0.7, 1.0]; // Silver
