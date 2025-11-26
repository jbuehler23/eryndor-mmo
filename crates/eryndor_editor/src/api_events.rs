//! API Events and Async Task Handling
//! Provides an event-based system for async API operations in Bevy.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::api_client::{ApiClient, ApiConfig, ContentType};
use crate::editor_state::{ZoneListItem, ItemListItem, EnemyListItem, NpcListItem, QuestListItem, AbilityListItem, LootTableListItem, EditorState};

/// Pending API operations queue
#[derive(Resource, Default)]
pub struct ApiTaskQueue {
    /// Pending zone list results
    pub zone_list_result: Arc<Mutex<Option<Result<Vec<ZoneListItem>, String>>>>,
    /// Pending zone creation result
    pub zone_create_result: Arc<Mutex<Option<Result<ZoneData, String>>>>,
    /// Pending zone load result
    pub zone_load_result: Arc<Mutex<Option<Result<ZoneData, String>>>>,
    /// Is currently loading zones
    pub loading_zones: bool,
    /// Is currently creating a zone
    pub creating_zone: bool,

    // Items
    /// Pending item list results
    pub item_list_result: Arc<Mutex<Option<Result<Vec<ItemListItem>, String>>>>,
    /// Pending item creation result
    pub item_create_result: Arc<Mutex<Option<Result<ItemData, String>>>>,
    /// Pending item update result
    pub item_update_result: Arc<Mutex<Option<Result<ItemData, String>>>>,
    /// Pending item delete result
    pub item_delete_result: Arc<Mutex<Option<Result<(), String>>>>,
    /// Is currently loading items
    pub loading_items: bool,
    /// Is currently creating an item
    pub creating_item: bool,
    /// Is currently updating an item
    pub updating_item: bool,
    /// Is currently deleting an item
    pub deleting_item: bool,

    // Enemies
    /// Pending enemy list results
    pub enemy_list_result: Arc<Mutex<Option<Result<Vec<EnemyListItem>, String>>>>,
    /// Pending enemy creation result
    pub enemy_create_result: Arc<Mutex<Option<Result<EnemyData, String>>>>,
    /// Pending enemy update result
    pub enemy_update_result: Arc<Mutex<Option<Result<EnemyData, String>>>>,
    /// Pending enemy delete result
    pub enemy_delete_result: Arc<Mutex<Option<Result<(), String>>>>,
    /// Is currently loading enemies
    pub loading_enemies: bool,
    /// Is currently creating an enemy
    pub creating_enemy: bool,
    /// Is currently updating an enemy
    pub updating_enemy: bool,
    /// Is currently deleting an enemy
    pub deleting_enemy: bool,

    // NPCs
    pub npc_list_result: Arc<Mutex<Option<Result<Vec<NpcListItem>, String>>>>,
    pub npc_create_result: Arc<Mutex<Option<Result<NpcData, String>>>>,
    pub npc_update_result: Arc<Mutex<Option<Result<NpcData, String>>>>,
    pub npc_delete_result: Arc<Mutex<Option<Result<(), String>>>>,
    pub loading_npcs: bool,
    pub creating_npc: bool,
    pub updating_npc: bool,
    pub deleting_npc: bool,

    // Quests
    pub quest_list_result: Arc<Mutex<Option<Result<Vec<QuestListItem>, String>>>>,
    pub quest_create_result: Arc<Mutex<Option<Result<QuestData, String>>>>,
    pub quest_update_result: Arc<Mutex<Option<Result<QuestData, String>>>>,
    pub quest_delete_result: Arc<Mutex<Option<Result<(), String>>>>,
    pub loading_quests: bool,
    pub creating_quest: bool,
    pub updating_quest: bool,
    pub deleting_quest: bool,

    // Abilities
    pub ability_list_result: Arc<Mutex<Option<Result<Vec<AbilityListItem>, String>>>>,
    pub ability_create_result: Arc<Mutex<Option<Result<AbilityData, String>>>>,
    pub ability_update_result: Arc<Mutex<Option<Result<AbilityData, String>>>>,
    pub ability_delete_result: Arc<Mutex<Option<Result<(), String>>>>,
    pub loading_abilities: bool,
    pub creating_ability: bool,
    pub updating_ability: bool,
    pub deleting_ability: bool,

    // Loot Tables
    pub loot_table_list_result: Arc<Mutex<Option<Result<Vec<LootTableListItem>, String>>>>,
    pub loot_table_create_result: Arc<Mutex<Option<Result<LootTableData, String>>>>,
    pub loot_table_update_result: Arc<Mutex<Option<Result<LootTableData, String>>>>,
    pub loot_table_delete_result: Arc<Mutex<Option<Result<(), String>>>>,
    pub loading_loot_tables: bool,
    pub creating_loot_table: bool,
    pub updating_loot_table: bool,
    pub deleting_loot_table: bool,
}

/// Zone data structure matching the server API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneData {
    pub id: String,
    pub name: String,
    pub width: f32,
    pub height: f32,
    pub spawn_point: [f32; 2],
    #[serde(default)]
    pub background_color: Option<[f32; 4]>,
    #[serde(default)]
    pub entities: Vec<serde_json::Value>,
    #[serde(default)]
    pub collision_shapes: Vec<serde_json::Value>,
    #[serde(default)]
    pub spawn_regions: Vec<serde_json::Value>,
}

impl Default for ZoneData {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            width: 1920.0,
            height: 1080.0,
            spawn_point: [100.0, 100.0],
            background_color: Some([0.1, 0.1, 0.12, 1.0]),
            entities: Vec::new(),
            collision_shapes: Vec::new(),
            spawn_regions: Vec::new(),
        }
    }
}

/// Message to request loading the zone list
#[derive(Message)]
pub struct LoadZoneListEvent;

/// Message to request creating a new zone
#[derive(Message)]
pub struct CreateZoneEvent {
    pub zone: ZoneData,
}

/// Message to request loading a specific zone
#[derive(Message)]
pub struct LoadZoneEvent {
    pub zone_id: String,
}

/// Message to request saving a zone
#[derive(Message)]
pub struct SaveZoneEvent {
    pub zone: ZoneData,
}

/// Message to request deleting a zone
#[derive(Message)]
pub struct DeleteZoneEvent {
    pub zone_id: String,
}

// =============================================================================
// Items
// =============================================================================

/// Item data structure matching the server API
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ItemData {
    pub id: u32,
    pub name: String,
    pub item_type: String,
    #[serde(default)]
    pub grants_ability: Option<u32>,
    #[serde(default)]
    pub stat_bonuses: ItemStatBonuses,
}

/// Item stat bonuses
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ItemStatBonuses {
    #[serde(default)]
    pub attack_power: f32,
    #[serde(default)]
    pub defense: f32,
    #[serde(default)]
    pub max_health: f32,
    #[serde(default)]
    pub max_mana: f32,
    #[serde(default)]
    pub crit_chance: f32,
}

/// Message to request loading the item list
#[derive(Message)]
pub struct LoadItemListEvent;

/// Message to request creating a new item
#[derive(Message)]
pub struct CreateItemEvent {
    pub item: ItemData,
}

/// Message to request updating an item
#[derive(Message)]
pub struct UpdateItemEvent {
    pub item: ItemData,
}

/// Message to request deleting an item
#[derive(Message)]
pub struct DeleteItemEvent {
    pub item_id: u32,
}

// =============================================================================
// Enemies
// =============================================================================

/// Enemy data structure matching the server API
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnemyData {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub max_health: f32,
    #[serde(default)]
    pub attack_power: f32,
    #[serde(default)]
    pub defense: f32,
    #[serde(default)]
    pub move_speed: f32,
}

/// Message to request loading the enemy list
#[derive(Message)]
pub struct LoadEnemyListEvent;

/// Message to request creating a new enemy
#[derive(Message)]
pub struct CreateEnemyEvent {
    pub enemy: EnemyData,
}

/// Message to request updating an enemy
#[derive(Message)]
pub struct UpdateEnemyEvent {
    pub enemy: EnemyData,
}

/// Message to request deleting an enemy
#[derive(Message)]
pub struct DeleteEnemyEvent {
    pub enemy_id: u32,
}

// =============================================================================
// NPCs
// =============================================================================

/// NPC data structure matching the server API
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NpcData {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub role: String,
}

/// Message to request loading the NPC list
#[derive(Message)]
pub struct LoadNpcListEvent;

/// Message to request creating a new NPC
#[derive(Message)]
pub struct CreateNpcEvent {
    pub npc: NpcData,
}

/// Message to request updating an NPC
#[derive(Message)]
pub struct UpdateNpcEvent {
    pub npc: NpcData,
}

/// Message to request deleting an NPC
#[derive(Message)]
pub struct DeleteNpcEvent {
    pub npc_id: u32,
}

// =============================================================================
// Quests
// =============================================================================

/// Quest data structure matching the server API
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuestData {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub quest_type: String,
}

/// Message to request loading the quest list
#[derive(Message)]
pub struct LoadQuestListEvent;

/// Message to request creating a new quest
#[derive(Message)]
pub struct CreateQuestEvent {
    pub quest: QuestData,
}

/// Message to request updating a quest
#[derive(Message)]
pub struct UpdateQuestEvent {
    pub quest: QuestData,
}

/// Message to request deleting a quest
#[derive(Message)]
pub struct DeleteQuestEvent {
    pub quest_id: u32,
}

// =============================================================================
// Abilities
// =============================================================================

/// Ability data structure matching the server API
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AbilityData {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub ability_type: String,
}

/// Message to request loading the ability list
#[derive(Message)]
pub struct LoadAbilityListEvent;

/// Message to request creating a new ability
#[derive(Message)]
pub struct CreateAbilityEvent {
    pub ability: AbilityData,
}

/// Message to request updating an ability
#[derive(Message)]
pub struct UpdateAbilityEvent {
    pub ability: AbilityData,
}

/// Message to request deleting an ability
#[derive(Message)]
pub struct DeleteAbilityEvent {
    pub ability_id: u32,
}

// =============================================================================
// Loot Tables
// =============================================================================

/// Loot table data structure matching the server API
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LootTableData {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub table_type: String,
}

/// Message to request loading the loot table list
#[derive(Message)]
pub struct LoadLootTableListEvent;

/// Message to request creating a new loot table
#[derive(Message)]
pub struct CreateLootTableEvent {
    pub loot_table: LootTableData,
}

/// Message to request updating a loot table
#[derive(Message)]
pub struct UpdateLootTableEvent {
    pub loot_table: LootTableData,
}

/// Message to request deleting a loot table
#[derive(Message)]
pub struct DeleteLootTableEvent {
    pub loot_table_id: String,
}

/// API Plugin - adds all necessary systems and resources
pub struct ApiEventsPlugin;

impl Plugin for ApiEventsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ApiTaskQueue>()
            // Zone messages
            .add_message::<LoadZoneListEvent>()
            .add_message::<CreateZoneEvent>()
            .add_message::<LoadZoneEvent>()
            .add_message::<SaveZoneEvent>()
            .add_message::<DeleteZoneEvent>()
            // Item messages
            .add_message::<LoadItemListEvent>()
            .add_message::<CreateItemEvent>()
            .add_message::<UpdateItemEvent>()
            .add_message::<DeleteItemEvent>()
            // Enemy messages
            .add_message::<LoadEnemyListEvent>()
            .add_message::<CreateEnemyEvent>()
            .add_message::<UpdateEnemyEvent>()
            .add_message::<DeleteEnemyEvent>()
            // NPC messages
            .add_message::<LoadNpcListEvent>()
            .add_message::<CreateNpcEvent>()
            .add_message::<UpdateNpcEvent>()
            .add_message::<DeleteNpcEvent>()
            // Quest messages
            .add_message::<LoadQuestListEvent>()
            .add_message::<CreateQuestEvent>()
            .add_message::<UpdateQuestEvent>()
            .add_message::<DeleteQuestEvent>()
            // Ability messages
            .add_message::<LoadAbilityListEvent>()
            .add_message::<CreateAbilityEvent>()
            .add_message::<UpdateAbilityEvent>()
            .add_message::<DeleteAbilityEvent>()
            // Loot table messages
            .add_message::<LoadLootTableListEvent>()
            .add_message::<CreateLootTableEvent>()
            .add_message::<UpdateLootTableEvent>()
            .add_message::<DeleteLootTableEvent>()
            // Split systems into smaller tuples to avoid Bevy's tuple size limit
            .add_systems(Update, (
                // Zone handlers
                handle_load_zone_list,
                handle_create_zone,
                // Item handlers
                handle_load_item_list,
                handle_create_item,
                handle_update_item,
                handle_delete_item,
                // Enemy handlers
                handle_load_enemy_list,
                handle_create_enemy,
                handle_update_enemy,
                handle_delete_enemy,
            ))
            .add_systems(Update, (
                // NPC handlers
                handle_load_npc_list,
                handle_create_npc,
                handle_update_npc,
                handle_delete_npc,
                // Quest handlers
                handle_load_quest_list,
                handle_create_quest,
                handle_update_quest,
                handle_delete_quest,
            ))
            .add_systems(Update, (
                // Ability handlers
                handle_load_ability_list,
                handle_create_ability,
                handle_update_ability,
                handle_delete_ability,
                // Loot table handlers
                handle_load_loot_table_list,
                handle_create_loot_table,
                handle_update_loot_table,
                handle_delete_loot_table,
                // Polling
                poll_api_results,
            ));
    }
}

/// System to handle zone list loading requests
fn handle_load_zone_list(
    mut events: MessageReader<LoadZoneListEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for _ in events.read() {
        if task_queue.loading_zones {
            continue; // Already loading
        }

        task_queue.loading_zones = true;
        let result_holder = task_queue.zone_list_result.clone();
        let api_url = editor_state.api_url.clone();

        // Spawn async task
        #[cfg(target_family = "wasm")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.list::<ZoneListItem>(ContentType::Zones, None).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(data) = response.data {
                                Ok(data.items)
                            } else {
                                Ok(Vec::new())
                            }
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            });
        }

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;

            IoTaskPool::get().spawn(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.list::<ZoneListItem>(ContentType::Zones, None).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(data) = response.data {
                                Ok(data.items)
                            } else {
                                Ok(Vec::new())
                            }
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

/// System to handle zone creation requests
fn handle_create_zone(
    mut events: MessageReader<CreateZoneEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.creating_zone {
            continue; // Already creating
        }

        task_queue.creating_zone = true;
        let result_holder = task_queue.zone_create_result.clone();
        let api_url = editor_state.api_url.clone();
        let zone_data = event.zone.clone();

        #[cfg(target_family = "wasm")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.create::<ZoneData, ZoneData>(ContentType::Zones, &zone_data).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(data) = response.data {
                                Ok(data)
                            } else {
                                Err("No data returned".to_string())
                            }
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            });
        }

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;

            IoTaskPool::get().spawn(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.create::<ZoneData, ZoneData>(ContentType::Zones, &zone_data).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(data) = response.data {
                                Ok(data)
                            } else {
                                Err("No data returned".to_string())
                            }
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

// =============================================================================
// Item Handlers
// =============================================================================

/// System to handle item list loading requests
fn handle_load_item_list(
    mut events: MessageReader<LoadItemListEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for _ in events.read() {
        if task_queue.loading_items {
            continue;
        }

        task_queue.loading_items = true;
        let result_holder = task_queue.item_list_result.clone();
        let api_url = editor_state.api_url.clone();

        #[cfg(target_family = "wasm")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.list::<ItemListItem>(ContentType::Items, None).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(data) = response.data {
                                Ok(data.items)
                            } else {
                                Ok(Vec::new())
                            }
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            });
        }

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;

            IoTaskPool::get().spawn(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.list::<ItemListItem>(ContentType::Items, None).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(data) = response.data {
                                Ok(data.items)
                            } else {
                                Ok(Vec::new())
                            }
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

/// System to handle item creation requests
fn handle_create_item(
    mut events: MessageReader<CreateItemEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.creating_item {
            continue;
        }

        task_queue.creating_item = true;
        let result_holder = task_queue.item_create_result.clone();
        let api_url = editor_state.api_url.clone();
        let item_data = event.item.clone();

        #[cfg(target_family = "wasm")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.create::<ItemData, ItemData>(ContentType::Items, &item_data).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(data) = response.data {
                                Ok(data)
                            } else {
                                Err("No data returned".to_string())
                            }
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            });
        }

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;

            IoTaskPool::get().spawn(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.create::<ItemData, ItemData>(ContentType::Items, &item_data).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(data) = response.data {
                                Ok(data)
                            } else {
                                Err("No data returned".to_string())
                            }
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

/// System to handle item update requests
fn handle_update_item(
    mut events: MessageReader<UpdateItemEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.updating_item {
            continue;
        }

        task_queue.updating_item = true;
        let result_holder = task_queue.item_update_result.clone();
        let api_url = editor_state.api_url.clone();
        let item_data = event.item.clone();

        #[cfg(target_family = "wasm")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.update::<ItemData, ItemData>(ContentType::Items, &item_data.id.to_string(), &item_data).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(data) = response.data {
                                Ok(data)
                            } else {
                                Err("No data returned".to_string())
                            }
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            });
        }

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;

            IoTaskPool::get().spawn(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.update::<ItemData, ItemData>(ContentType::Items, &item_data.id.to_string(), &item_data).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(data) = response.data {
                                Ok(data)
                            } else {
                                Err("No data returned".to_string())
                            }
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

/// System to handle item deletion requests
fn handle_delete_item(
    mut events: MessageReader<DeleteItemEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.deleting_item {
            continue;
        }

        task_queue.deleting_item = true;
        let result_holder = task_queue.item_delete_result.clone();
        let api_url = editor_state.api_url.clone();
        let item_id = event.item_id;

        #[cfg(target_family = "wasm")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.delete(ContentType::Items, &item_id.to_string()).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            Ok(())
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            });
        }

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;

            IoTaskPool::get().spawn(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.delete(ContentType::Items, &item_id.to_string()).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            Ok(())
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

// =============================================================================
// Enemy Handlers
// =============================================================================

/// System to handle enemy list loading requests
fn handle_load_enemy_list(
    mut events: MessageReader<LoadEnemyListEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for _ in events.read() {
        if task_queue.loading_enemies {
            continue;
        }

        task_queue.loading_enemies = true;
        let result_holder = task_queue.enemy_list_result.clone();
        let api_url = editor_state.api_url.clone();

        #[cfg(target_family = "wasm")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.list::<EnemyListItem>(ContentType::Enemies, None).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(data) = response.data {
                                Ok(data.items)
                            } else {
                                Ok(Vec::new())
                            }
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            });
        }

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;

            IoTaskPool::get().spawn(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.list::<EnemyListItem>(ContentType::Enemies, None).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(data) = response.data {
                                Ok(data.items)
                            } else {
                                Ok(Vec::new())
                            }
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

/// System to handle enemy creation requests
fn handle_create_enemy(
    mut events: MessageReader<CreateEnemyEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.creating_enemy {
            continue;
        }

        task_queue.creating_enemy = true;
        let result_holder = task_queue.enemy_create_result.clone();
        let api_url = editor_state.api_url.clone();
        let enemy_data = event.enemy.clone();

        #[cfg(target_family = "wasm")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.create::<EnemyData, EnemyData>(ContentType::Enemies, &enemy_data).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(data) = response.data {
                                Ok(data)
                            } else {
                                Err("No data returned".to_string())
                            }
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            });
        }

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;

            IoTaskPool::get().spawn(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.create::<EnemyData, EnemyData>(ContentType::Enemies, &enemy_data).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(data) = response.data {
                                Ok(data)
                            } else {
                                Err("No data returned".to_string())
                            }
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

/// System to handle enemy update requests
fn handle_update_enemy(
    mut events: MessageReader<UpdateEnemyEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.updating_enemy {
            continue;
        }

        task_queue.updating_enemy = true;
        let result_holder = task_queue.enemy_update_result.clone();
        let api_url = editor_state.api_url.clone();
        let enemy_data = event.enemy.clone();

        #[cfg(target_family = "wasm")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.update::<EnemyData, EnemyData>(ContentType::Enemies, &enemy_data.id.to_string(), &enemy_data).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(data) = response.data {
                                Ok(data)
                            } else {
                                Err("No data returned".to_string())
                            }
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            });
        }

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;

            IoTaskPool::get().spawn(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.update::<EnemyData, EnemyData>(ContentType::Enemies, &enemy_data.id.to_string(), &enemy_data).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            if let Some(data) = response.data {
                                Ok(data)
                            } else {
                                Err("No data returned".to_string())
                            }
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

/// System to handle enemy deletion requests
fn handle_delete_enemy(
    mut events: MessageReader<DeleteEnemyEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.deleting_enemy {
            continue;
        }

        task_queue.deleting_enemy = true;
        let result_holder = task_queue.enemy_delete_result.clone();
        let api_url = editor_state.api_url.clone();
        let enemy_id = event.enemy_id;

        #[cfg(target_family = "wasm")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.delete(ContentType::Enemies, &enemy_id.to_string()).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            Ok(())
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            });
        }

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;

            IoTaskPool::get().spawn(async move {
                let config = ApiConfig {
                    base_url: api_url,
                    auth_token: None,
                };
                let client = ApiClient::new(config);

                let result = client.delete(ContentType::Enemies, &enemy_id.to_string()).await;

                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => {
                        if response.success {
                            Ok(())
                        } else {
                            Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

// =============================================================================
// NPC Handlers
// =============================================================================

fn handle_load_npc_list(
    mut events: MessageReader<LoadNpcListEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for _ in events.read() {
        if task_queue.loading_npcs { continue; }
        task_queue.loading_npcs = true;
        let result_holder = task_queue.npc_list_result.clone();
        let api_url = editor_state.api_url.clone();

        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let config = ApiConfig { base_url: api_url, auth_token: None };
            let client = ApiClient::new(config);
            let result = client.list::<NpcListItem>(ContentType::Npcs, None).await;
            let mut holder = result_holder.lock().unwrap();
            *holder = Some(match result {
                Ok(response) => if response.success { Ok(response.data.map(|d| d.items).unwrap_or_default()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                Err(e) => Err(e),
            });
        });

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;
            IoTaskPool::get().spawn(async move {
                let config = ApiConfig { base_url: api_url, auth_token: None };
                let client = ApiClient::new(config);
                let result = client.list::<NpcListItem>(ContentType::Npcs, None).await;
                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => if response.success { Ok(response.data.map(|d| d.items).unwrap_or_default()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

fn handle_create_npc(
    mut events: MessageReader<CreateNpcEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.creating_npc { continue; }
        task_queue.creating_npc = true;
        let result_holder = task_queue.npc_create_result.clone();
        let api_url = editor_state.api_url.clone();
        let data = event.npc.clone();

        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let config = ApiConfig { base_url: api_url, auth_token: None };
            let client = ApiClient::new(config);
            let result = client.create::<NpcData, NpcData>(ContentType::Npcs, &data).await;
            let mut holder = result_holder.lock().unwrap();
            *holder = Some(match result {
                Ok(response) => if response.success { response.data.ok_or_else(|| "No data returned".to_string()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                Err(e) => Err(e),
            });
        });

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;
            IoTaskPool::get().spawn(async move {
                let config = ApiConfig { base_url: api_url, auth_token: None };
                let client = ApiClient::new(config);
                let result = client.create::<NpcData, NpcData>(ContentType::Npcs, &data).await;
                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => if response.success { response.data.ok_or_else(|| "No data returned".to_string()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

fn handle_update_npc(
    mut events: MessageReader<UpdateNpcEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.updating_npc { continue; }
        task_queue.updating_npc = true;
        let result_holder = task_queue.npc_update_result.clone();
        let api_url = editor_state.api_url.clone();
        let data = event.npc.clone();

        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let config = ApiConfig { base_url: api_url, auth_token: None };
            let client = ApiClient::new(config);
            let result = client.update::<NpcData, NpcData>(ContentType::Npcs, &data.id.to_string(), &data).await;
            let mut holder = result_holder.lock().unwrap();
            *holder = Some(match result {
                Ok(response) => if response.success { response.data.ok_or_else(|| "No data returned".to_string()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                Err(e) => Err(e),
            });
        });

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;
            IoTaskPool::get().spawn(async move {
                let config = ApiConfig { base_url: api_url, auth_token: None };
                let client = ApiClient::new(config);
                let result = client.update::<NpcData, NpcData>(ContentType::Npcs, &data.id.to_string(), &data).await;
                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => if response.success { response.data.ok_or_else(|| "No data returned".to_string()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

fn handle_delete_npc(
    mut events: MessageReader<DeleteNpcEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.deleting_npc { continue; }
        task_queue.deleting_npc = true;
        let result_holder = task_queue.npc_delete_result.clone();
        let api_url = editor_state.api_url.clone();
        let id = event.npc_id;

        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let config = ApiConfig { base_url: api_url, auth_token: None };
            let client = ApiClient::new(config);
            let result = client.delete(ContentType::Npcs, &id.to_string()).await;
            let mut holder = result_holder.lock().unwrap();
            *holder = Some(match result {
                Ok(response) => if response.success { Ok(()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                Err(e) => Err(e),
            });
        });

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;
            IoTaskPool::get().spawn(async move {
                let config = ApiConfig { base_url: api_url, auth_token: None };
                let client = ApiClient::new(config);
                let result = client.delete(ContentType::Npcs, &id.to_string()).await;
                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => if response.success { Ok(()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

// =============================================================================
// Quest Handlers
// =============================================================================

fn handle_load_quest_list(
    mut events: MessageReader<LoadQuestListEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for _ in events.read() {
        if task_queue.loading_quests { continue; }
        task_queue.loading_quests = true;
        let result_holder = task_queue.quest_list_result.clone();
        let api_url = editor_state.api_url.clone();

        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let config = ApiConfig { base_url: api_url, auth_token: None };
            let client = ApiClient::new(config);
            let result = client.list::<QuestListItem>(ContentType::Quests, None).await;
            let mut holder = result_holder.lock().unwrap();
            *holder = Some(match result {
                Ok(response) => if response.success { Ok(response.data.map(|d| d.items).unwrap_or_default()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                Err(e) => Err(e),
            });
        });

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;
            IoTaskPool::get().spawn(async move {
                let config = ApiConfig { base_url: api_url, auth_token: None };
                let client = ApiClient::new(config);
                let result = client.list::<QuestListItem>(ContentType::Quests, None).await;
                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => if response.success { Ok(response.data.map(|d| d.items).unwrap_or_default()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

fn handle_create_quest(
    mut events: MessageReader<CreateQuestEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.creating_quest { continue; }
        task_queue.creating_quest = true;
        let result_holder = task_queue.quest_create_result.clone();
        let api_url = editor_state.api_url.clone();
        let data = event.quest.clone();

        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let config = ApiConfig { base_url: api_url, auth_token: None };
            let client = ApiClient::new(config);
            let result = client.create::<QuestData, QuestData>(ContentType::Quests, &data).await;
            let mut holder = result_holder.lock().unwrap();
            *holder = Some(match result {
                Ok(response) => if response.success { response.data.ok_or_else(|| "No data returned".to_string()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                Err(e) => Err(e),
            });
        });

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;
            IoTaskPool::get().spawn(async move {
                let config = ApiConfig { base_url: api_url, auth_token: None };
                let client = ApiClient::new(config);
                let result = client.create::<QuestData, QuestData>(ContentType::Quests, &data).await;
                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => if response.success { response.data.ok_or_else(|| "No data returned".to_string()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

fn handle_update_quest(
    mut events: MessageReader<UpdateQuestEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.updating_quest { continue; }
        task_queue.updating_quest = true;
        let result_holder = task_queue.quest_update_result.clone();
        let api_url = editor_state.api_url.clone();
        let data = event.quest.clone();

        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let config = ApiConfig { base_url: api_url, auth_token: None };
            let client = ApiClient::new(config);
            let result = client.update::<QuestData, QuestData>(ContentType::Quests, &data.id.to_string(), &data).await;
            let mut holder = result_holder.lock().unwrap();
            *holder = Some(match result {
                Ok(response) => if response.success { response.data.ok_or_else(|| "No data returned".to_string()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                Err(e) => Err(e),
            });
        });

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;
            IoTaskPool::get().spawn(async move {
                let config = ApiConfig { base_url: api_url, auth_token: None };
                let client = ApiClient::new(config);
                let result = client.update::<QuestData, QuestData>(ContentType::Quests, &data.id.to_string(), &data).await;
                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => if response.success { response.data.ok_or_else(|| "No data returned".to_string()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

fn handle_delete_quest(
    mut events: MessageReader<DeleteQuestEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.deleting_quest { continue; }
        task_queue.deleting_quest = true;
        let result_holder = task_queue.quest_delete_result.clone();
        let api_url = editor_state.api_url.clone();
        let id = event.quest_id;

        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let config = ApiConfig { base_url: api_url, auth_token: None };
            let client = ApiClient::new(config);
            let result = client.delete(ContentType::Quests, &id.to_string()).await;
            let mut holder = result_holder.lock().unwrap();
            *holder = Some(match result {
                Ok(response) => if response.success { Ok(()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                Err(e) => Err(e),
            });
        });

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;
            IoTaskPool::get().spawn(async move {
                let config = ApiConfig { base_url: api_url, auth_token: None };
                let client = ApiClient::new(config);
                let result = client.delete(ContentType::Quests, &id.to_string()).await;
                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => if response.success { Ok(()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

// =============================================================================
// Ability Handlers
// =============================================================================

fn handle_load_ability_list(
    mut events: MessageReader<LoadAbilityListEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for _ in events.read() {
        if task_queue.loading_abilities { continue; }
        task_queue.loading_abilities = true;
        let result_holder = task_queue.ability_list_result.clone();
        let api_url = editor_state.api_url.clone();

        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let config = ApiConfig { base_url: api_url, auth_token: None };
            let client = ApiClient::new(config);
            let result = client.list::<AbilityListItem>(ContentType::Abilities, None).await;
            let mut holder = result_holder.lock().unwrap();
            *holder = Some(match result {
                Ok(response) => if response.success { Ok(response.data.map(|d| d.items).unwrap_or_default()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                Err(e) => Err(e),
            });
        });

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;
            IoTaskPool::get().spawn(async move {
                let config = ApiConfig { base_url: api_url, auth_token: None };
                let client = ApiClient::new(config);
                let result = client.list::<AbilityListItem>(ContentType::Abilities, None).await;
                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => if response.success { Ok(response.data.map(|d| d.items).unwrap_or_default()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

fn handle_create_ability(
    mut events: MessageReader<CreateAbilityEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.creating_ability { continue; }
        task_queue.creating_ability = true;
        let result_holder = task_queue.ability_create_result.clone();
        let api_url = editor_state.api_url.clone();
        let data = event.ability.clone();

        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let config = ApiConfig { base_url: api_url, auth_token: None };
            let client = ApiClient::new(config);
            let result = client.create::<AbilityData, AbilityData>(ContentType::Abilities, &data).await;
            let mut holder = result_holder.lock().unwrap();
            *holder = Some(match result {
                Ok(response) => if response.success { response.data.ok_or_else(|| "No data returned".to_string()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                Err(e) => Err(e),
            });
        });

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;
            IoTaskPool::get().spawn(async move {
                let config = ApiConfig { base_url: api_url, auth_token: None };
                let client = ApiClient::new(config);
                let result = client.create::<AbilityData, AbilityData>(ContentType::Abilities, &data).await;
                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => if response.success { response.data.ok_or_else(|| "No data returned".to_string()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

fn handle_update_ability(
    mut events: MessageReader<UpdateAbilityEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.updating_ability { continue; }
        task_queue.updating_ability = true;
        let result_holder = task_queue.ability_update_result.clone();
        let api_url = editor_state.api_url.clone();
        let data = event.ability.clone();

        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let config = ApiConfig { base_url: api_url, auth_token: None };
            let client = ApiClient::new(config);
            let result = client.update::<AbilityData, AbilityData>(ContentType::Abilities, &data.id.to_string(), &data).await;
            let mut holder = result_holder.lock().unwrap();
            *holder = Some(match result {
                Ok(response) => if response.success { response.data.ok_or_else(|| "No data returned".to_string()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                Err(e) => Err(e),
            });
        });

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;
            IoTaskPool::get().spawn(async move {
                let config = ApiConfig { base_url: api_url, auth_token: None };
                let client = ApiClient::new(config);
                let result = client.update::<AbilityData, AbilityData>(ContentType::Abilities, &data.id.to_string(), &data).await;
                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => if response.success { response.data.ok_or_else(|| "No data returned".to_string()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

fn handle_delete_ability(
    mut events: MessageReader<DeleteAbilityEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.deleting_ability { continue; }
        task_queue.deleting_ability = true;
        let result_holder = task_queue.ability_delete_result.clone();
        let api_url = editor_state.api_url.clone();
        let id = event.ability_id;

        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let config = ApiConfig { base_url: api_url, auth_token: None };
            let client = ApiClient::new(config);
            let result = client.delete(ContentType::Abilities, &id.to_string()).await;
            let mut holder = result_holder.lock().unwrap();
            *holder = Some(match result {
                Ok(response) => if response.success { Ok(()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                Err(e) => Err(e),
            });
        });

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;
            IoTaskPool::get().spawn(async move {
                let config = ApiConfig { base_url: api_url, auth_token: None };
                let client = ApiClient::new(config);
                let result = client.delete(ContentType::Abilities, &id.to_string()).await;
                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => if response.success { Ok(()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

// =============================================================================
// Loot Table Handlers
// =============================================================================

fn handle_load_loot_table_list(
    mut events: MessageReader<LoadLootTableListEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for _ in events.read() {
        if task_queue.loading_loot_tables { continue; }
        task_queue.loading_loot_tables = true;
        let result_holder = task_queue.loot_table_list_result.clone();
        let api_url = editor_state.api_url.clone();

        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let config = ApiConfig { base_url: api_url, auth_token: None };
            let client = ApiClient::new(config);
            let result = client.list::<LootTableListItem>(ContentType::LootTables, None).await;
            let mut holder = result_holder.lock().unwrap();
            *holder = Some(match result {
                Ok(response) => if response.success { Ok(response.data.map(|d| d.items).unwrap_or_default()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                Err(e) => Err(e),
            });
        });

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;
            IoTaskPool::get().spawn(async move {
                let config = ApiConfig { base_url: api_url, auth_token: None };
                let client = ApiClient::new(config);
                let result = client.list::<LootTableListItem>(ContentType::LootTables, None).await;
                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => if response.success { Ok(response.data.map(|d| d.items).unwrap_or_default()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

fn handle_create_loot_table(
    mut events: MessageReader<CreateLootTableEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.creating_loot_table { continue; }
        task_queue.creating_loot_table = true;
        let result_holder = task_queue.loot_table_create_result.clone();
        let api_url = editor_state.api_url.clone();
        let data = event.loot_table.clone();

        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let config = ApiConfig { base_url: api_url, auth_token: None };
            let client = ApiClient::new(config);
            let result = client.create::<LootTableData, LootTableData>(ContentType::LootTables, &data).await;
            let mut holder = result_holder.lock().unwrap();
            *holder = Some(match result {
                Ok(response) => if response.success { response.data.ok_or_else(|| "No data returned".to_string()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                Err(e) => Err(e),
            });
        });

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;
            IoTaskPool::get().spawn(async move {
                let config = ApiConfig { base_url: api_url, auth_token: None };
                let client = ApiClient::new(config);
                let result = client.create::<LootTableData, LootTableData>(ContentType::LootTables, &data).await;
                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => if response.success { response.data.ok_or_else(|| "No data returned".to_string()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

fn handle_update_loot_table(
    mut events: MessageReader<UpdateLootTableEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.updating_loot_table { continue; }
        task_queue.updating_loot_table = true;
        let result_holder = task_queue.loot_table_update_result.clone();
        let api_url = editor_state.api_url.clone();
        let data = event.loot_table.clone();

        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let config = ApiConfig { base_url: api_url, auth_token: None };
            let client = ApiClient::new(config);
            let result = client.update::<LootTableData, LootTableData>(ContentType::LootTables, &data.id, &data).await;
            let mut holder = result_holder.lock().unwrap();
            *holder = Some(match result {
                Ok(response) => if response.success { response.data.ok_or_else(|| "No data returned".to_string()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                Err(e) => Err(e),
            });
        });

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;
            IoTaskPool::get().spawn(async move {
                let config = ApiConfig { base_url: api_url, auth_token: None };
                let client = ApiClient::new(config);
                let result = client.update::<LootTableData, LootTableData>(ContentType::LootTables, &data.id, &data).await;
                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => if response.success { response.data.ok_or_else(|| "No data returned".to_string()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

fn handle_delete_loot_table(
    mut events: MessageReader<DeleteLootTableEvent>,
    mut task_queue: ResMut<ApiTaskQueue>,
    editor_state: Res<EditorState>,
) {
    for event in events.read() {
        if task_queue.deleting_loot_table { continue; }
        task_queue.deleting_loot_table = true;
        let result_holder = task_queue.loot_table_delete_result.clone();
        let api_url = editor_state.api_url.clone();
        let id = event.loot_table_id.clone();

        #[cfg(target_family = "wasm")]
        wasm_bindgen_futures::spawn_local(async move {
            let config = ApiConfig { base_url: api_url, auth_token: None };
            let client = ApiClient::new(config);
            let result = client.delete(ContentType::LootTables, &id).await;
            let mut holder = result_holder.lock().unwrap();
            *holder = Some(match result {
                Ok(response) => if response.success { Ok(()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                Err(e) => Err(e),
            });
        });

        #[cfg(not(target_family = "wasm"))]
        {
            use bevy::tasks::IoTaskPool;
            IoTaskPool::get().spawn(async move {
                let config = ApiConfig { base_url: api_url, auth_token: None };
                let client = ApiClient::new(config);
                let result = client.delete(ContentType::LootTables, &id).await;
                let mut holder = result_holder.lock().unwrap();
                *holder = Some(match result {
                    Ok(response) => if response.success { Ok(()) } else { Err(response.error.unwrap_or_else(|| "Unknown error".to_string())) },
                    Err(e) => Err(e),
                });
            }).detach();
        }
    }
}

/// System to poll for completed API results and update editor state
fn poll_api_results(
    mut task_queue: ResMut<ApiTaskQueue>,
    mut editor_state: ResMut<EditorState>,
    mut load_zone_list_events: MessageWriter<LoadZoneListEvent>,
    mut load_item_list_events: MessageWriter<LoadItemListEvent>,
    mut load_enemy_list_events: MessageWriter<LoadEnemyListEvent>,
    mut load_npc_list_events: MessageWriter<LoadNpcListEvent>,
    mut load_quest_list_events: MessageWriter<LoadQuestListEvent>,
    mut load_ability_list_events: MessageWriter<LoadAbilityListEvent>,
    mut load_loot_table_list_events: MessageWriter<LoadLootTableListEvent>,
) {
    // Check zone list result
    if task_queue.loading_zones {
        let result = {
            let mut holder = task_queue.zone_list_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.loading_zones = false;
            match result {
                Ok(zones) => {
                    editor_state.world.zone_list = zones;
                    editor_state.status_message = format!("Loaded {} zones", editor_state.world.zone_list.len());
                }
                Err(e) => {
                    editor_state.status_message = format!("Failed to load zones: {}", e);
                }
            }
        }
    }

    // Check zone creation result
    if task_queue.creating_zone {
        let result = {
            let mut holder = task_queue.zone_create_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.creating_zone = false;
            match result {
                Ok(zone) => {
                    editor_state.status_message = format!("Created zone: {}", zone.name);
                    editor_state.world.current_zone = Some(zone.id.clone());
                    // Refresh zone list
                    load_zone_list_events.write(LoadZoneListEvent);
                }
                Err(e) => {
                    editor_state.status_message = format!("Failed to create zone: {}", e);
                }
            }
        }
    }

    // Check item list result
    if task_queue.loading_items {
        let result = {
            let mut holder = task_queue.item_list_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.loading_items = false;
            match result {
                Ok(items) => {
                    editor_state.items.item_list = items;
                    editor_state.status_message = format!("Loaded {} items", editor_state.items.item_list.len());
                }
                Err(e) => {
                    editor_state.status_message = format!("Failed to load items: {}", e);
                }
            }
        }
    }

    // Check item creation result
    if task_queue.creating_item {
        let result = {
            let mut holder = task_queue.item_create_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.creating_item = false;
            match result {
                Ok(item) => {
                    editor_state.status_message = format!("Created item: {}", item.name);
                    editor_state.items.selected_item = Some(item.id);
                    // Refresh item list
                    load_item_list_events.write(LoadItemListEvent);
                }
                Err(e) => {
                    editor_state.status_message = format!("Failed to create item: {}", e);
                }
            }
        }
    }

    // Check item update result
    if task_queue.updating_item {
        let result = {
            let mut holder = task_queue.item_update_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.updating_item = false;
            match result {
                Ok(item) => {
                    editor_state.status_message = format!("Saved item: {}", item.name);
                    // Refresh item list
                    load_item_list_events.write(LoadItemListEvent);
                }
                Err(e) => {
                    editor_state.status_message = format!("Failed to save item: {}", e);
                }
            }
        }
    }

    // Check item delete result
    if task_queue.deleting_item {
        let result = {
            let mut holder = task_queue.item_delete_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.deleting_item = false;
            match result {
                Ok(()) => {
                    editor_state.status_message = "Item deleted".to_string();
                    editor_state.items.selected_item = None;
                    editor_state.items.editing_item = None;
                    // Refresh item list
                    load_item_list_events.write(LoadItemListEvent);
                }
                Err(e) => {
                    editor_state.status_message = format!("Failed to delete item: {}", e);
                }
            }
        }
    }

    // Check enemy list result
    if task_queue.loading_enemies {
        let result = {
            let mut holder = task_queue.enemy_list_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.loading_enemies = false;
            match result {
                Ok(enemies) => {
                    editor_state.enemies.enemy_list = enemies;
                    editor_state.status_message = format!("Loaded {} enemies", editor_state.enemies.enemy_list.len());
                }
                Err(e) => {
                    editor_state.status_message = format!("Failed to load enemies: {}", e);
                }
            }
        }
    }

    // Check enemy creation result
    if task_queue.creating_enemy {
        let result = {
            let mut holder = task_queue.enemy_create_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.creating_enemy = false;
            match result {
                Ok(enemy) => {
                    editor_state.status_message = format!("Created enemy: {}", enemy.name);
                    editor_state.enemies.selected_enemy = Some(enemy.id);
                    // Refresh enemy list
                    load_enemy_list_events.write(LoadEnemyListEvent);
                }
                Err(e) => {
                    editor_state.status_message = format!("Failed to create enemy: {}", e);
                }
            }
        }
    }

    // Check enemy update result
    if task_queue.updating_enemy {
        let result = {
            let mut holder = task_queue.enemy_update_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.updating_enemy = false;
            match result {
                Ok(enemy) => {
                    editor_state.status_message = format!("Saved enemy: {}", enemy.name);
                    // Refresh enemy list
                    load_enemy_list_events.write(LoadEnemyListEvent);
                }
                Err(e) => {
                    editor_state.status_message = format!("Failed to save enemy: {}", e);
                }
            }
        }
    }

    // Check enemy delete result
    if task_queue.deleting_enemy {
        let result = {
            let mut holder = task_queue.enemy_delete_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.deleting_enemy = false;
            match result {
                Ok(()) => {
                    editor_state.status_message = "Enemy deleted".to_string();
                    editor_state.enemies.selected_enemy = None;
                    editor_state.enemies.editing_enemy = None;
                    // Refresh enemy list
                    load_enemy_list_events.write(LoadEnemyListEvent);
                }
                Err(e) => {
                    editor_state.status_message = format!("Failed to delete enemy: {}", e);
                }
            }
        }
    }

    // =============================================================================
    // NPC Polling
    // =============================================================================

    if task_queue.loading_npcs {
        let result = {
            let mut holder = task_queue.npc_list_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.loading_npcs = false;
            match result {
                Ok(npcs) => {
                    editor_state.npcs.npc_list = npcs;
                    editor_state.status_message = format!("Loaded {} NPCs", editor_state.npcs.npc_list.len());
                }
                Err(e) => editor_state.status_message = format!("Failed to load NPCs: {}", e),
            }
        }
    }

    if task_queue.creating_npc {
        let result = {
            let mut holder = task_queue.npc_create_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.creating_npc = false;
            match result {
                Ok(npc) => {
                    editor_state.status_message = format!("Created NPC: {}", npc.name);
                    editor_state.npcs.selected_npc = Some(npc.id);
                    load_npc_list_events.write(LoadNpcListEvent);
                }
                Err(e) => editor_state.status_message = format!("Failed to create NPC: {}", e),
            }
        }
    }

    if task_queue.updating_npc {
        let result = {
            let mut holder = task_queue.npc_update_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.updating_npc = false;
            match result {
                Ok(npc) => {
                    editor_state.status_message = format!("Saved NPC: {}", npc.name);
                    load_npc_list_events.write(LoadNpcListEvent);
                }
                Err(e) => editor_state.status_message = format!("Failed to save NPC: {}", e),
            }
        }
    }

    if task_queue.deleting_npc {
        let result = {
            let mut holder = task_queue.npc_delete_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.deleting_npc = false;
            match result {
                Ok(()) => {
                    editor_state.status_message = "NPC deleted".to_string();
                    editor_state.npcs.selected_npc = None;
                    load_npc_list_events.write(LoadNpcListEvent);
                }
                Err(e) => editor_state.status_message = format!("Failed to delete NPC: {}", e),
            }
        }
    }

    // =============================================================================
    // Quest Polling
    // =============================================================================

    if task_queue.loading_quests {
        let result = {
            let mut holder = task_queue.quest_list_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.loading_quests = false;
            match result {
                Ok(quests) => {
                    editor_state.quests.quest_list = quests;
                    editor_state.status_message = format!("Loaded {} quests", editor_state.quests.quest_list.len());
                }
                Err(e) => editor_state.status_message = format!("Failed to load quests: {}", e),
            }
        }
    }

    if task_queue.creating_quest {
        let result = {
            let mut holder = task_queue.quest_create_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.creating_quest = false;
            match result {
                Ok(quest) => {
                    editor_state.status_message = format!("Created quest: {}", quest.name);
                    editor_state.quests.selected_quest = Some(quest.id);
                    load_quest_list_events.write(LoadQuestListEvent);
                }
                Err(e) => editor_state.status_message = format!("Failed to create quest: {}", e),
            }
        }
    }

    if task_queue.updating_quest {
        let result = {
            let mut holder = task_queue.quest_update_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.updating_quest = false;
            match result {
                Ok(quest) => {
                    editor_state.status_message = format!("Saved quest: {}", quest.name);
                    load_quest_list_events.write(LoadQuestListEvent);
                }
                Err(e) => editor_state.status_message = format!("Failed to save quest: {}", e),
            }
        }
    }

    if task_queue.deleting_quest {
        let result = {
            let mut holder = task_queue.quest_delete_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.deleting_quest = false;
            match result {
                Ok(()) => {
                    editor_state.status_message = "Quest deleted".to_string();
                    editor_state.quests.selected_quest = None;
                    load_quest_list_events.write(LoadQuestListEvent);
                }
                Err(e) => editor_state.status_message = format!("Failed to delete quest: {}", e),
            }
        }
    }

    // =============================================================================
    // Ability Polling
    // =============================================================================

    if task_queue.loading_abilities {
        let result = {
            let mut holder = task_queue.ability_list_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.loading_abilities = false;
            match result {
                Ok(abilities) => {
                    editor_state.abilities.ability_list = abilities;
                    editor_state.status_message = format!("Loaded {} abilities", editor_state.abilities.ability_list.len());
                }
                Err(e) => editor_state.status_message = format!("Failed to load abilities: {}", e),
            }
        }
    }

    if task_queue.creating_ability {
        let result = {
            let mut holder = task_queue.ability_create_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.creating_ability = false;
            match result {
                Ok(ability) => {
                    editor_state.status_message = format!("Created ability: {}", ability.name);
                    editor_state.abilities.selected_ability = Some(ability.id);
                    load_ability_list_events.write(LoadAbilityListEvent);
                }
                Err(e) => editor_state.status_message = format!("Failed to create ability: {}", e),
            }
        }
    }

    if task_queue.updating_ability {
        let result = {
            let mut holder = task_queue.ability_update_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.updating_ability = false;
            match result {
                Ok(ability) => {
                    editor_state.status_message = format!("Saved ability: {}", ability.name);
                    load_ability_list_events.write(LoadAbilityListEvent);
                }
                Err(e) => editor_state.status_message = format!("Failed to save ability: {}", e),
            }
        }
    }

    if task_queue.deleting_ability {
        let result = {
            let mut holder = task_queue.ability_delete_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.deleting_ability = false;
            match result {
                Ok(()) => {
                    editor_state.status_message = "Ability deleted".to_string();
                    editor_state.abilities.selected_ability = None;
                    load_ability_list_events.write(LoadAbilityListEvent);
                }
                Err(e) => editor_state.status_message = format!("Failed to delete ability: {}", e),
            }
        }
    }

    // =============================================================================
    // Loot Table Polling
    // =============================================================================

    if task_queue.loading_loot_tables {
        let result = {
            let mut holder = task_queue.loot_table_list_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.loading_loot_tables = false;
            match result {
                Ok(loot_tables) => {
                    editor_state.loot.loot_table_list = loot_tables;
                    editor_state.status_message = format!("Loaded {} loot tables", editor_state.loot.loot_table_list.len());
                }
                Err(e) => editor_state.status_message = format!("Failed to load loot tables: {}", e),
            }
        }
    }

    if task_queue.creating_loot_table {
        let result = {
            let mut holder = task_queue.loot_table_create_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.creating_loot_table = false;
            match result {
                Ok(loot_table) => {
                    editor_state.status_message = format!("Created loot table: {}", loot_table.name);
                    editor_state.loot.selected_loot_table = Some(loot_table.id.clone());
                    load_loot_table_list_events.write(LoadLootTableListEvent);
                }
                Err(e) => editor_state.status_message = format!("Failed to create loot table: {}", e),
            }
        }
    }

    if task_queue.updating_loot_table {
        let result = {
            let mut holder = task_queue.loot_table_update_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.updating_loot_table = false;
            match result {
                Ok(loot_table) => {
                    editor_state.status_message = format!("Saved loot table: {}", loot_table.name);
                    load_loot_table_list_events.write(LoadLootTableListEvent);
                }
                Err(e) => editor_state.status_message = format!("Failed to save loot table: {}", e),
            }
        }
    }

    if task_queue.deleting_loot_table {
        let result = {
            let mut holder = task_queue.loot_table_delete_result.lock().unwrap();
            holder.take()
        };
        if let Some(result) = result {
            task_queue.deleting_loot_table = false;
            match result {
                Ok(()) => {
                    editor_state.status_message = "Loot table deleted".to_string();
                    editor_state.loot.selected_loot_table = None;
                    load_loot_table_list_events.write(LoadLootTableListEvent);
                }
                Err(e) => editor_state.status_message = format!("Failed to delete loot table: {}", e),
            }
        }
    }
}
