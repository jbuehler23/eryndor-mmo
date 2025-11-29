use bevy::prelude::*;
use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use std::collections::HashMap;
use std::path::Path;
use crate::game_data::{ItemDefinition, ItemDatabase, EnemyDefinition, EnemyDatabase, QuestDefinition, QuestDatabase, ZoneDefinition, ZoneDatabase};
use eryndor_shared::AbilityDefinition;
use crate::abilities::AbilityDatabase;

// ============================================================================
// ASSET TYPES
// ============================================================================

/// Wrapper for ItemDefinition as a Bevy Asset
#[derive(Asset, TypePath, Debug)]
pub struct ItemAsset(pub ItemDefinition);

/// Wrapper for EnemyDefinition as a Bevy Asset
#[derive(Asset, TypePath, Debug)]
pub struct EnemyAsset(pub EnemyDefinition);

/// Wrapper for ZoneDefinition as a Bevy Asset
#[derive(Asset, TypePath, Debug)]
pub struct ZoneAsset(pub ZoneDefinition);

/// Wrapper for QuestDefinition as a Bevy Asset
#[derive(Asset, TypePath, Debug)]
pub struct QuestAsset(pub QuestDefinition);

/// Wrapper for AbilityDefinition as a Bevy Asset
#[derive(Asset, TypePath, Debug)]
pub struct AbilityAsset(pub AbilityDefinition);

// ============================================================================
// ASSET LOADERS
// ============================================================================

/// Loader for item JSON files
#[derive(Default)]
pub struct ItemAssetLoader;

impl AssetLoader for ItemAssetLoader {
    type Asset = ItemAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let item: ItemDefinition = serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(ItemAsset(item))
    }

    fn extensions(&self) -> &[&str] {
        &["item.json"]
    }
}

/// Loader for enemy JSON files
#[derive(Default)]
pub struct EnemyAssetLoader;

impl AssetLoader for EnemyAssetLoader {
    type Asset = EnemyAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let enemy: EnemyDefinition = serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(EnemyAsset(enemy))
    }

    fn extensions(&self) -> &[&str] {
        &["enemy.json"]
    }
}

/// Loader for zone JSON files
#[derive(Default)]
pub struct ZoneAssetLoader;

impl AssetLoader for ZoneAssetLoader {
    type Asset = ZoneAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let zone: ZoneDefinition = serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(ZoneAsset(zone))
    }

    fn extensions(&self) -> &[&str] {
        &["zone.json"]
    }
}

/// Loader for quest JSON files
#[derive(Default)]
pub struct QuestAssetLoader;

impl AssetLoader for QuestAssetLoader {
    type Asset = QuestAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let quest: QuestDefinition = serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(QuestAsset(quest))
    }

    fn extensions(&self) -> &[&str] {
        &["quest.json"]
    }
}

/// Loader for ability JSON files
#[derive(Default)]
pub struct AbilityAssetLoader;

impl AssetLoader for AbilityAssetLoader {
    type Asset = AbilityAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let ability: AbilityDefinition = serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(AbilityAsset(ability))
    }

    fn extensions(&self) -> &[&str] {
        &["ability.json"]
    }
}

// ============================================================================
// ASSET HANDLE TRACKING
// ============================================================================

/// Resource to track loaded asset handles for hot reloading
#[derive(Resource, Default)]
pub struct LoadedContentAssets {
    pub items: HashMap<AssetId<ItemAsset>, Handle<ItemAsset>>,
    pub enemies: HashMap<AssetId<EnemyAsset>, Handle<EnemyAsset>>,
    pub zones: HashMap<AssetId<ZoneAsset>, Handle<ZoneAsset>>,
    pub quests: HashMap<AssetId<QuestAsset>, Handle<QuestAsset>>,
    pub abilities: HashMap<AssetId<AbilityAsset>, Handle<AbilityAsset>>,
}

// ============================================================================
// SYSTEMS
// ============================================================================

/// System to initially load all content assets at startup
/// This function is also exported as `setup_content_loading` for use in main.rs
pub fn load_all_content_assets(
    asset_server: Res<AssetServer>,
    mut loaded_assets: ResMut<LoadedContentAssets>,
) {
    info!("Loading game content assets via Bevy AssetServer...");

    // Load all item assets
    if let Ok(entries) = std::fs::read_dir("assets/content/items") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let asset_path = format!("content/items/{}", path.file_name().unwrap().to_str().unwrap());
                let handle: Handle<ItemAsset> = asset_server.load(&asset_path);
                info!("Loading item asset: {}", asset_path);
                loaded_assets.items.insert(handle.id(), handle);
            }
        }
    }

    // Load all enemy assets
    if let Ok(entries) = std::fs::read_dir("assets/content/enemies") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let asset_path = format!("content/enemies/{}", path.file_name().unwrap().to_str().unwrap());
                let handle: Handle<EnemyAsset> = asset_server.load(&asset_path);
                info!("Loading enemy asset: {}", asset_path);
                loaded_assets.enemies.insert(handle.id(), handle);
            }
        }
    }

    // Load all zone assets
    if let Ok(entries) = std::fs::read_dir("assets/content/zones") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let asset_path = format!("content/zones/{}", path.file_name().unwrap().to_str().unwrap());
                let handle: Handle<ZoneAsset> = asset_server.load(&asset_path);
                info!("Loading zone asset: {}", asset_path);
                loaded_assets.zones.insert(handle.id(), handle);
            }
        }
    }

    // Load all quest assets
    if let Ok(entries) = std::fs::read_dir("assets/content/quests") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let asset_path = format!("content/quests/{}", path.file_name().unwrap().to_str().unwrap());
                let handle: Handle<QuestAsset> = asset_server.load(&asset_path);
                info!("Loading quest asset: {}", asset_path);
                loaded_assets.quests.insert(handle.id(), handle);
            }
        }
    }

    // Load all ability assets
    if let Ok(entries) = std::fs::read_dir("assets/content/abilities") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let asset_path = format!("content/abilities/{}", path.file_name().unwrap().to_str().unwrap());
                let handle: Handle<AbilityAsset> = asset_server.load(&asset_path);
                info!("Loading ability asset: {}", asset_path);
                loaded_assets.abilities.insert(handle.id(), handle);
            }
        }
    }

    info!("Content asset loading initiated");
}

/// System to handle item asset events (loaded/modified)
#[allow(deprecated)]
fn handle_item_asset_events(
    mut events: bevy::ecs::event::EventReader<AssetEvent<ItemAsset>>,
    item_assets: Res<Assets<ItemAsset>>,
    mut item_db: ResMut<ItemDatabase>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                if let Some(item_asset) = item_assets.get(*id) {
                    let item = &item_asset.0;
                    info!("Item asset loaded/modified: {} (id: {})", item.name, item.id);
                    item_db.items.insert(item.id, item.clone());
                }
            }
            AssetEvent::Removed { id } => {
                // Find and remove the item from the database
                if let Some(item_asset) = item_assets.get(*id) {
                    let item_id = item_asset.0.id;
                    info!("Item asset removed: {}", item_id);
                    item_db.items.remove(&item_id);
                }
            }
            _ => {}
        }
    }
}

/// System to handle enemy asset events (loaded/modified)
#[allow(deprecated)]
fn handle_enemy_asset_events(
    mut events: bevy::ecs::event::EventReader<AssetEvent<EnemyAsset>>,
    enemy_assets: Res<Assets<EnemyAsset>>,
    mut enemy_db: ResMut<EnemyDatabase>,
    mut enemies_query: Query<(
        &eryndor_shared::EnemyType,
        &mut eryndor_shared::Health,
        &mut eryndor_shared::MoveSpeed,
        &mut eryndor_shared::CombatStats,
        &mut eryndor_shared::BaseStats,
        &mut eryndor_shared::AggroRange,
        &mut eryndor_shared::VisualShape,
    ), With<eryndor_shared::Enemy>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id } => {
                if let Some(enemy_asset) = enemy_assets.get(*id) {
                    let enemy = &enemy_asset.0;
                    info!("Enemy asset loaded: {} (id: {}) - HP: {}", enemy.name, enemy.id, enemy.max_health);
                    enemy_db.enemies.insert(enemy.id, enemy.clone());
                }
            }
            AssetEvent::Modified { id } => {
                if let Some(enemy_asset) = enemy_assets.get(*id) {
                    let def = &enemy_asset.0;
                    info!("Enemy asset modified: {} (id: {}) - HP: {}", def.name, def.id, def.max_health);
                    enemy_db.enemies.insert(def.id, def.clone());

                    // Update all spawned enemies of this type
                    let mut updated_count = 0;
                    for (enemy_type, mut health, mut move_speed, mut combat_stats, mut base_stats, mut aggro_range, mut visual) in &mut enemies_query {
                        if enemy_type.0 == def.id {
                            // Update health (preserve current/max ratio if damaged)
                            let health_ratio = health.current / health.max;
                            health.max = def.max_health;
                            health.current = def.max_health * health_ratio;

                            // Update other stats
                            move_speed.0 = def.move_speed;
                            combat_stats.attack_power = def.attack_power;
                            combat_stats.defense = def.defense;
                            base_stats.attack_power = def.attack_power;
                            base_stats.defense = def.defense;
                            base_stats.move_speed = def.move_speed;
                            aggro_range.aggro = def.aggro_range;
                            aggro_range.leash = def.leash_range;

                            // Update visual
                            visual.color = def.visual.color;
                            visual.size = def.visual.size;
                            visual.shape_type = match def.visual.shape.as_str() {
                                "Square" | "Rectangle" => eryndor_shared::ShapeType::Square,
                                _ => eryndor_shared::ShapeType::Circle,
                            };

                            updated_count += 1;
                        }
                    }
                    if updated_count > 0 {
                        info!("Hot-reloaded {} spawned {} enemies with new stats", updated_count, def.name);
                    }
                }
            }
            AssetEvent::Removed { id } => {
                if let Some(enemy_asset) = enemy_assets.get(*id) {
                    let enemy_id = enemy_asset.0.id;
                    info!("Enemy asset removed: {}", enemy_id);
                    enemy_db.enemies.remove(&enemy_id);
                }
            }
            _ => {}
        }
    }
}

/// System to handle zone asset events (loaded/modified)
#[allow(deprecated)]
fn handle_zone_asset_events(
    mut events: bevy::ecs::event::EventReader<AssetEvent<ZoneAsset>>,
    zone_assets: Res<Assets<ZoneAsset>>,
    mut zone_db: ResMut<ZoneDatabase>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                if let Some(zone_asset) = zone_assets.get(*id) {
                    let zone = &zone_asset.0;
                    info!("Zone asset loaded/modified: {} (id: {})", zone.zone_name, zone.zone_id);
                    zone_db.zones.insert(zone.zone_id.clone(), zone.clone());
                }
            }
            AssetEvent::Removed { id } => {
                if let Some(zone_asset) = zone_assets.get(*id) {
                    let zone_id = zone_asset.0.zone_id.clone();
                    info!("Zone asset removed: {}", zone_id);
                    zone_db.zones.remove(&zone_id);
                }
            }
            _ => {}
        }
    }
}

/// System to handle quest asset events (loaded/modified)
#[allow(deprecated)]
fn handle_quest_asset_events(
    mut events: bevy::ecs::event::EventReader<AssetEvent<QuestAsset>>,
    quest_assets: Res<Assets<QuestAsset>>,
    mut quest_db: ResMut<QuestDatabase>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                if let Some(quest_asset) = quest_assets.get(*id) {
                    let quest = &quest_asset.0;
                    info!("Quest asset loaded/modified: {} (id: {})", quest.name, quest.id);
                    quest_db.quests.insert(quest.id, quest.clone());
                }
            }
            AssetEvent::Removed { id } => {
                if let Some(quest_asset) = quest_assets.get(*id) {
                    let quest_id = quest_asset.0.id;
                    info!("Quest asset removed: {}", quest_id);
                    quest_db.quests.remove(&quest_id);
                }
            }
            _ => {}
        }
    }
}

/// System to handle ability asset events (loaded/modified)
#[allow(deprecated)]
fn handle_ability_asset_events(
    mut events: bevy::ecs::event::EventReader<AssetEvent<AbilityAsset>>,
    ability_assets: Res<Assets<AbilityAsset>>,
    mut ability_db: ResMut<AbilityDatabase>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                if let Some(ability_asset) = ability_assets.get(*id) {
                    let ability = &ability_asset.0;
                    info!("Ability asset loaded/modified: {} (id: {})", ability.name, ability.id);
                    ability_db.abilities.insert(ability.id, ability.clone());
                }
            }
            AssetEvent::Removed { id } => {
                if let Some(ability_asset) = ability_assets.get(*id) {
                    let ability_id = ability_asset.0.id;
                    info!("Ability asset removed: {}", ability_id);
                    ability_db.abilities.remove(&ability_id);
                }
            }
            _ => {}
        }
    }
}

// ============================================================================
// PLUGIN
// ============================================================================

/// Alias for load_all_content_assets to maintain compatibility with main.rs
pub use load_all_content_assets as setup_content_loading;

/// Plugin to register content asset loading with hot reload support
pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register asset types
            .init_asset::<ItemAsset>()
            .init_asset::<EnemyAsset>()
            .init_asset::<ZoneAsset>()
            .init_asset::<QuestAsset>()
            .init_asset::<AbilityAsset>()
            // Register asset loaders
            .init_asset_loader::<ItemAssetLoader>()
            .init_asset_loader::<EnemyAssetLoader>()
            .init_asset_loader::<ZoneAssetLoader>()
            .init_asset_loader::<QuestAssetLoader>()
            .init_asset_loader::<AbilityAssetLoader>()
            // Initialize resources
            .init_resource::<LoadedContentAssets>()
            .init_resource::<ZoneDatabase>()
            // Load all content at startup
            .add_systems(Startup, load_all_content_assets)
            // Handle asset events for hot reloading
            .add_systems(Update, (
                handle_item_asset_events,
                handle_enemy_asset_events,
                handle_zone_asset_events,
                handle_quest_asset_events,
                handle_ability_asset_events,
            ));
    }
}
