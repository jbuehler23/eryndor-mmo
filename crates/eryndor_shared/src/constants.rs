use bevy::prelude::*;
use std::env;

// ============================================================================
// NETWORK CONSTANTS
// ============================================================================

// Environment-configurable network settings with sensible defaults

/// Get server bind address from environment or use default
/// Production: 0.0.0.0 (accepts connections from any IP)
/// Development: 127.0.0.1 (localhost only)
pub fn server_addr() -> String {
    env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string())
}

/// Get UDP port from environment or use default 5001
pub fn server_port() -> u16 {
    env::var("SERVER_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5001)
}

/// Get WebSocket port from environment or use default 5003
pub fn server_port_websocket() -> u16 {
    env::var("SERVER_PORT_WEBSOCKET")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5003)
}

/// Get WebTransport port from environment or use default 5002
pub fn server_port_webtransport() -> u16 {
    env::var("SERVER_PORT_WEBTRANSPORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5002)
}

/// Get HTTP certificate server port from environment or use default 8080
pub fn server_cert_port() -> u16 {
    env::var("SERVER_CERT_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080)
}

// Legacy constants for backwards compatibility (deprecated - use functions above)
pub const SERVER_PORT: u16 = 5001;
pub const SERVER_PORT_WEBTRANSPORT: u16 = 5002;
pub const SERVER_PORT_WEBSOCKET: u16 = 5003;
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

// Rogue abilities (Dagger) - IDs 300-399
pub const ABILITY_QUICK_STRIKE: u32 = 300;

// Mage abilities (Wand) - IDs 200-299
pub const ABILITY_FIREBALL: u32 = 200;

// Knight abilities (Sword) - IDs 100-199
pub const ABILITY_HEAVY_SLASH: u32 = 100;

// Wizard abilities (Staff) - IDs 400-499
pub const ABILITY_ARCANE_BLAST: u32 = 400;

// Cleric abilities (Mace) - IDs 500-599
pub const ABILITY_SMITE: u32 = 500;

// Ranger abilities (Bow) - IDs 600-699
pub const ABILITY_AIMED_SHOT: u32 = 600;

// Berserker abilities (Axe) - IDs 700-799
pub const ABILITY_RENDING_STRIKE: u32 = 700;

// Test abilities - IDs 900-999
pub const ABILITY_TEST_AOE: u32 = 900;
pub const ABILITY_TEST_HEAL: u32 = 901;
pub const ABILITY_TEST_MOBILITY: u32 = 902;
pub const ABILITY_TEST_BUFF: u32 = 903;

// ============================================================================
// ITEM IDS
// ============================================================================

// Weapons
pub const ITEM_DAGGER: u32 = 1;
pub const ITEM_WAND: u32 = 2;
pub const ITEM_SWORD: u32 = 3;
pub const ITEM_STAFF: u32 = 4;
pub const ITEM_MACE: u32 = 5;
pub const ITEM_BOW: u32 = 6;
pub const ITEM_AXE: u32 = 7;

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
pub const ENEMY_TYPE_GOBLIN: u32 = 2;
pub const ENEMY_TYPE_WOLF: u32 = 3;
pub const ENEMY_TYPE_SKELETON: u32 = 4;
pub const ENEMY_TYPE_ORC: u32 = 5;
pub const ENEMY_TYPE_SPIDER: u32 = 6;

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
pub const COLOR_GOLD: [f32; 4] = [1.0, 0.84, 0.0, 1.0]; // Gold
pub const COLOR_ITEM: [f32; 4] = [0.6, 0.4, 0.2, 1.0]; // Brown/Bronze
pub const COLOR_ITEM_DAGGER: [f32; 4] = [0.8, 0.8, 0.2, 1.0]; // Yellow
pub const COLOR_ITEM_WAND: [f32; 4] = [0.8, 0.2, 0.8, 1.0]; // Purple
pub const COLOR_ITEM_SWORD: [f32; 4] = [0.7, 0.7, 0.7, 1.0]; // Silver
pub const COLOR_LOOT_CONTAINER: [f32; 4] = [0.6, 0.4, 0.2, 1.0]; // Brown (like a chest/bag)

// ============================================================================
// LOOT CONSTANTS
// ============================================================================

pub const LOOT_CONTAINER_SIZE: f32 = 20.0;
