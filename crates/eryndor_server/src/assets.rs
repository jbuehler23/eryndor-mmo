use bevy::prelude::*;
use crate::game_data::{ItemDefinition, ItemDatabase, EnemyDefinition, EnemyDatabase, QuestDefinition, QuestDatabase, ZoneDefinition, ZoneDatabase};
use std::collections::HashMap;
use std::path::Path;

/// Load all items from individual JSON files in content/items/
pub fn load_items_from_content() -> HashMap<u32, ItemDefinition> {
    let content_path = Path::new("assets/content/items");
    let mut items = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(content_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    match serde_json::from_str::<ItemDefinition>(&content) {
                        Ok(item) => {
                            info!("Loaded item: {} (id: {})", item.name, item.id);
                            items.insert(item.id, item);
                        }
                        Err(e) => {
                            warn!("Failed to parse item file {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
    } else {
        warn!("Items content directory not found: {:?}", content_path);
    }

    items
}

/// Load all enemies from individual JSON files in content/enemies/
pub fn load_enemies_from_content() -> HashMap<u32, EnemyDefinition> {
    let content_path = Path::new("assets/content/enemies");
    let mut enemies = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(content_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    match serde_json::from_str::<EnemyDefinition>(&content) {
                        Ok(enemy) => {
                            info!("Loaded enemy: {} (id: {})", enemy.name, enemy.id);
                            enemies.insert(enemy.id, enemy);
                        }
                        Err(e) => {
                            warn!("Failed to parse enemy file {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
    } else {
        warn!("Enemies content directory not found: {:?}", content_path);
    }

    enemies
}

/// Load all zones from individual JSON files in content/zones/
pub fn load_zones_from_content() -> HashMap<String, ZoneDefinition> {
    let content_path = Path::new("assets/content/zones");
    let mut zones = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(content_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    match serde_json::from_str::<ZoneDefinition>(&content) {
                        Ok(zone) => {
                            info!("Loaded zone: {} (id: {})", zone.zone_name, zone.zone_id);
                            zones.insert(zone.zone_id.clone(), zone);
                        }
                        Err(e) => {
                            warn!("Failed to parse zone file {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
    } else {
        warn!("Zones content directory not found: {:?}", content_path);
    }

    zones
}

/// Load all quests from individual JSON files in content/quests/
pub fn load_quests_from_content() -> HashMap<u32, QuestDefinition> {
    let content_path = Path::new("assets/content/quests");
    let mut quests = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(content_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    match serde_json::from_str::<QuestDefinition>(&content) {
                        Ok(quest) => {
                            info!("Loaded quest: {} (id: {})", quest.name, quest.id);
                            quests.insert(quest.id, quest);
                        }
                        Err(e) => {
                            warn!("Failed to parse quest file {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
    } else {
        // Quests directory may not exist yet - that's okay
        info!("Quests content directory not found: {:?}", content_path);
    }

    quests
}

/// System to load all game content from individual JSON files at startup
pub fn setup_content_loading(
    mut item_db: ResMut<ItemDatabase>,
    mut enemy_db: ResMut<EnemyDatabase>,
    mut quest_db: ResMut<QuestDatabase>,
    mut zone_db: ResMut<ZoneDatabase>,
) {
    info!("Loading game content from individual JSON files...");

    // Load items
    let items = load_items_from_content();
    info!("Loaded {} items from content/items/", items.len());
    item_db.items = items;

    // Load enemies
    let enemies = load_enemies_from_content();
    info!("Loaded {} enemies from content/enemies/", enemies.len());
    enemy_db.enemies = enemies;

    // Load zones
    let zones = load_zones_from_content();
    info!("Loaded {} zones from content/zones/", zones.len());
    zone_db.zones = zones;

    // Load quests
    let quests = load_quests_from_content();
    info!("Loaded {} quests from content/quests/", quests.len());
    quest_db.quests = quests;

    info!("Content loading complete!");
}

/// Plugin to register content loading
pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ZoneDatabase>()
            .add_systems(Startup, setup_content_loading);
    }
}
