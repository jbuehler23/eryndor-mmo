use bevy::prelude::*;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use serde::Deserialize;
use crate::game_data::{ItemDefinition, ItemDatabase, EnemyDefinition, EnemyDatabase, QuestDefinition, QuestDatabase, ZoneDefinition, ZoneDatabase};
use std::collections::HashMap;

/// Asset type for item data loaded from JSON
#[derive(Asset, TypePath, Deserialize)]
pub struct ItemDataAsset {
    #[serde(flatten)]
    pub items: Vec<ItemDefinition>,
}

/// Asset type for enemy data loaded from JSON
#[derive(Asset, TypePath, Deserialize)]
pub struct EnemyDataAsset {
    #[serde(flatten)]
    pub enemies: Vec<EnemyDefinition>,
}

/// Custom asset loader for JSON item files
#[derive(Default)]
pub struct ItemJsonLoader;

impl AssetLoader for ItemJsonLoader {
    type Asset = ItemDataAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let items: Vec<ItemDefinition> = serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        info!("Loaded {} items from JSON", items.len());
        Ok(ItemDataAsset { items })
    }

    fn extensions(&self) -> &[&str] {
        &["items.json"]
    }
}

/// Custom asset loader for JSON enemy files
#[derive(Default)]
pub struct EnemyJsonLoader;

impl AssetLoader for EnemyJsonLoader {
    type Asset = EnemyDataAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let enemies: Vec<EnemyDefinition> = serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        info!("Loaded {} enemies from JSON", enemies.len());
        Ok(EnemyDataAsset { enemies })
    }

    fn extensions(&self) -> &[&str] {
        &["enemies.json"]
    }
}

/// Asset type for quest data loaded from JSON
#[derive(Asset, TypePath, Deserialize)]
pub struct QuestDataAsset {
    #[serde(flatten)]
    pub quests: Vec<QuestDefinition>,
}

/// Custom asset loader for JSON quest files
#[derive(Default)]
pub struct QuestJsonLoader;

impl AssetLoader for QuestJsonLoader {
    type Asset = QuestDataAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let quests: Vec<QuestDefinition> = serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        info!("Loaded {} quests from JSON", quests.len());
        Ok(QuestDataAsset { quests })
    }

    fn extensions(&self) -> &[&str] {
        &["quests.json"]
    }
}

/// Asset type for zone data loaded from JSON
#[derive(Asset, TypePath, Deserialize)]
pub struct ZoneDataAsset {
    #[serde(flatten)]
    pub zone: ZoneDefinition,
}

/// Custom asset loader for JSON zone files
#[derive(Default)]
pub struct ZoneJsonLoader;

impl AssetLoader for ZoneJsonLoader {
    type Asset = ZoneDataAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let zone: ZoneDefinition = serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        info!("Loaded zone '{}' from JSON", zone.zone_name);
        Ok(ZoneDataAsset { zone })
    }

    fn extensions(&self) -> &[&str] {
        &["zone.json"]
    }
}

/// Resource to track loaded asset handles
#[derive(Resource, Default)]
pub struct GameAssetHandles {
    pub items: Option<Handle<ItemDataAsset>>,
    pub enemies: Option<Handle<EnemyDataAsset>>,
    pub quests: Option<Handle<QuestDataAsset>>,
    pub zones: Option<Handle<ZoneDataAsset>>,
}

/// System to initialize asset loading on startup
pub fn setup_asset_loading(
    mut handles: ResMut<GameAssetHandles>,
    asset_server: Res<AssetServer>,
) {
    info!("Loading game assets from JSON files...");

    // Load items
    handles.items = Some(asset_server.load("items/weapons.json"));

    // Load enemies
    handles.enemies = Some(asset_server.load("enemies/enemy_types.json"));

    // Load quests
    handles.quests = Some(asset_server.load("quests/main_story.json"));

    // Load zones
    handles.zones = Some(asset_server.load("zones/starter_zone.zone.json"));

    info!("Asset loading initiated");
}

/// System to process loaded item assets and populate ItemDatabase
pub fn process_item_assets(
    mut item_db: ResMut<ItemDatabase>,
    mut asset_events: MessageReader<AssetEvent<ItemDataAsset>>,
    item_assets: Res<Assets<ItemDataAsset>>,
) {
    for event in asset_events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                if let Some(item_data) = item_assets.get(*id) {
                    info!("Reloading item database with {} items", item_data.items.len());

                    // Build new HashMap atomically to avoid empty database state during reload
                    let mut new_items = HashMap::new();
                    for item in &item_data.items {
                        new_items.insert(item.id, item.clone());
                    }

                    // Atomic swap - database is never empty
                    item_db.items = new_items;

                    info!("Item database updated successfully");
                }
            }
            AssetEvent::Removed { .. } => {
                warn!("Item asset removed - this shouldn't happen during gameplay");
            }
            _ => {}
        }
    }
}

/// System to process loaded enemy assets and populate EnemyDatabase
pub fn process_enemy_assets(
    mut enemy_db: ResMut<EnemyDatabase>,
    mut asset_events: MessageReader<AssetEvent<EnemyDataAsset>>,
    enemy_assets: Res<Assets<EnemyDataAsset>>,
) {
    for event in asset_events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                if let Some(enemy_data) = enemy_assets.get(*id) {
                    info!("Reloading enemy database with {} enemies", enemy_data.enemies.len());

                    // Build new HashMap atomically to avoid empty database state during reload
                    let mut new_enemies = HashMap::new();
                    for enemy in &enemy_data.enemies {
                        new_enemies.insert(enemy.id, enemy.clone());
                    }

                    // Atomic swap - database is never empty
                    enemy_db.enemies = new_enemies;

                    info!("Enemy database updated successfully");
                }
            }
            AssetEvent::Removed { .. } => {
                warn!("Enemy asset removed - this shouldn't happen during gameplay");
            }
            _ => {}
        }
    }
}

/// System to process loaded quest assets and populate QuestDatabase
pub fn process_quest_assets(
    mut quest_db: ResMut<QuestDatabase>,
    mut asset_events: MessageReader<AssetEvent<QuestDataAsset>>,
    quest_assets: Res<Assets<QuestDataAsset>>,
) {
    for event in asset_events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                if let Some(quest_data) = quest_assets.get(*id) {
                    info!("Reloading quest database with {} quests", quest_data.quests.len());

                    // Build new HashMap atomically to avoid empty database state during reload
                    let mut new_quests = HashMap::new();
                    for quest in &quest_data.quests {
                        new_quests.insert(quest.id, quest.clone());
                    }

                    // Atomic swap - database is never empty
                    quest_db.quests = new_quests;

                    info!("Quest database updated successfully");
                }
            }
            AssetEvent::Removed { .. } => {
                warn!("Quest asset removed - this shouldn't happen during gameplay");
            }
            _ => {}
        }
    }
}

/// System to process loaded zone assets and populate ZoneDatabase
pub fn process_zone_assets(
    mut zone_db: ResMut<ZoneDatabase>,
    mut asset_events: MessageReader<AssetEvent<ZoneDataAsset>>,
    zone_assets: Res<Assets<ZoneDataAsset>>,
) {
    for event in asset_events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                if let Some(zone_data) = zone_assets.get(*id) {
                    let zone = &zone_data.zone;
                    info!("Reloading zone: {}", zone.zone_name);

                    // Add or update zone in database
                    zone_db.zones.insert(zone.zone_id.clone(), zone.clone());

                    info!("Zone '{}' updated successfully", zone.zone_name);
                }
            }
            AssetEvent::Removed { .. } => {
                warn!("Zone asset removed - this shouldn't happen during gameplay");
            }
            _ => {}
        }
    }
}

/// Plugin to register asset loaders and systems
pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GameAssetHandles>()
            .init_resource::<ZoneDatabase>()
            .init_asset::<ItemDataAsset>()
            .init_asset::<EnemyDataAsset>()
            .init_asset::<QuestDataAsset>()
            .init_asset::<ZoneDataAsset>()
            .register_asset_loader(ItemJsonLoader)
            .register_asset_loader(EnemyJsonLoader)
            .register_asset_loader(QuestJsonLoader)
            .register_asset_loader(ZoneJsonLoader)
            .add_systems(Startup, setup_asset_loading)
            .add_systems(Update, (
                process_item_assets,
                process_enemy_assets,
                process_quest_assets,
                process_zone_assets,
            ));
    }
}
