//! Editor state management

use bevy::prelude::*;
use serde::{Serialize, Deserialize};

/// The currently active editor tab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorTab {
    #[default]
    World,
    Items,
    Enemies,
    Npcs,
    Quests,
    Abilities,
    Loot,
    Assets,
}

impl EditorTab {
    pub fn label(&self) -> &'static str {
        match self {
            EditorTab::World => "World",
            EditorTab::Items => "Items",
            EditorTab::Enemies => "Enemies",
            EditorTab::Npcs => "NPCs",
            EditorTab::Quests => "Quests",
            EditorTab::Abilities => "Abilities",
            EditorTab::Loot => "Loot",
            EditorTab::Assets => "Assets",
        }
    }

    pub fn all() -> &'static [EditorTab] {
        &[
            EditorTab::World,
            EditorTab::Items,
            EditorTab::Enemies,
            EditorTab::Npcs,
            EditorTab::Quests,
            EditorTab::Abilities,
            EditorTab::Loot,
            EditorTab::Assets,
        ]
    }
}

/// Authentication state
#[derive(Debug, Clone, Default)]
pub enum AuthState {
    #[default]
    NotAuthenticated,
    Authenticating,
    Authenticated {
        token: String,
        username: String,
    },
    Error(String),
}

/// Global editor state resource
#[derive(Resource)]
pub struct EditorState {
    /// Currently active tab
    pub active_tab: EditorTab,

    /// Authentication state
    pub auth: AuthState,

    /// API base URL
    pub api_url: String,

    /// Whether the editor has unsaved changes
    pub has_unsaved_changes: bool,

    /// Status message displayed in status bar
    pub status_message: String,

    /// World editor state
    pub world: WorldEditorState,

    /// Items editor state
    pub items: ItemsEditorState,

    /// Enemies editor state
    pub enemies: EnemiesEditorState,

    /// NPCs editor state
    pub npcs: NpcsEditorState,

    /// Quests editor state
    pub quests: QuestsEditorState,

    /// Abilities editor state
    pub abilities: AbilitiesEditorState,

    /// Loot editor state
    pub loot: LootEditorState,

    /// Assets browser state
    pub assets: AssetsEditorState,

    // === UI Action Flags ===
    // These flags are set by UI code and consumed by systems

    /// Request to load zones list
    pub action_load_zones: bool,

    /// Request to create a new zone
    pub action_create_zone: bool,

    /// Request to load items list
    pub action_load_items: bool,

    /// Request to create a new item
    pub action_create_item: bool,

    /// Request to save current item
    pub action_save_item: bool,

    /// Request to delete current item
    pub action_delete_item: bool,

    /// Request to load enemies list
    pub action_load_enemies: bool,

    /// Request to create a new enemy
    pub action_create_enemy: bool,

    /// Request to save current enemy
    pub action_save_enemy: bool,

    /// Request to delete current enemy
    pub action_delete_enemy: bool,

    // NPC action flags
    pub action_load_npcs: bool,
    pub action_create_npc: bool,
    pub action_save_npc: bool,
    pub action_delete_npc: bool,

    // Quest action flags
    pub action_load_quests: bool,
    pub action_create_quest: bool,
    pub action_save_quest: bool,
    pub action_delete_quest: bool,

    // Ability action flags
    pub action_load_abilities: bool,
    pub action_create_ability: bool,
    pub action_save_ability: bool,
    pub action_delete_ability: bool,

    // Loot table action flags
    pub action_load_loot_tables: bool,
    pub action_create_loot_table: bool,
    pub action_save_loot_table: bool,
    pub action_delete_loot_table: bool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            active_tab: EditorTab::World,
            auth: AuthState::NotAuthenticated,
            api_url: detect_api_url(),
            has_unsaved_changes: false,
            status_message: "Ready".to_string(),
            world: WorldEditorState::new(),
            items: ItemsEditorState::default(),
            enemies: EnemiesEditorState::default(),
            npcs: NpcsEditorState::default(),
            quests: QuestsEditorState::default(),
            abilities: AbilitiesEditorState::default(),
            loot: LootEditorState::default(),
            assets: AssetsEditorState::default(),
            action_load_zones: true, // Load zones on startup
            action_create_zone: false,
            action_load_items: false,
            action_create_item: false,
            action_save_item: false,
            action_delete_item: false,
            action_load_enemies: false,
            action_create_enemy: false,
            action_save_enemy: false,
            action_delete_enemy: false,
            action_load_npcs: false,
            action_create_npc: false,
            action_save_npc: false,
            action_delete_npc: false,
            action_load_quests: false,
            action_create_quest: false,
            action_save_quest: false,
            action_delete_quest: false,
            action_load_abilities: false,
            action_create_ability: false,
            action_save_ability: false,
            action_delete_ability: false,
            action_load_loot_tables: false,
            action_create_loot_table: false,
            action_save_loot_table: false,
            action_delete_loot_table: false,
        }
    }
}

/// World/Zone editor state
#[derive(Debug, Clone, Default)]
pub struct WorldEditorState {
    /// Currently loaded zone ID
    pub current_zone: Option<String>,

    /// List of available zones
    pub zone_list: Vec<ZoneListItem>,

    /// Camera position (pan)
    pub camera_pos: Vec2,

    /// Camera zoom level
    pub zoom: f32,

    /// Show grid overlay
    pub show_grid: bool,

    /// Grid snap enabled
    pub snap_to_grid: bool,

    /// Grid size in pixels
    pub grid_size: f32,

    /// Currently selected tool
    pub active_tool: WorldTool,

    /// Show collision shapes
    pub show_collisions: bool,

    /// Show spawn regions
    pub show_spawn_regions: bool,

    /// Show create new zone dialog
    pub show_create_dialog: bool,

    /// New zone form data
    pub new_zone_name: String,

    /// New zone width
    pub new_zone_width: f32,

    /// New zone height
    pub new_zone_height: f32,
}

impl WorldEditorState {
    pub fn new() -> Self {
        Self {
            current_zone: None,
            zone_list: Vec::new(),
            camera_pos: Vec2::ZERO,
            zoom: 1.0,
            show_grid: true,
            snap_to_grid: true,
            grid_size: 50.0,
            active_tool: WorldTool::Select,
            show_collisions: true,
            show_spawn_regions: true,
            show_create_dialog: false,
            new_zone_name: String::new(),
            new_zone_width: 1920.0,
            new_zone_height: 1080.0,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum WorldTool {
    #[default]
    Select,
    Pan,
    PlaceEntity,
    DrawCollision,
    DrawSpawnRegion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneListItem {
    pub id: String,
    pub name: String,
}

/// Items editor state
#[derive(Debug, Clone, Default)]
pub struct ItemsEditorState {
    /// List of all items
    pub item_list: Vec<ItemListItem>,

    /// Currently selected item ID
    pub selected_item: Option<u32>,

    /// Filter by item type
    pub type_filter: Option<String>,

    /// Search query
    pub search_query: String,

    /// Show create new item dialog
    pub show_create_dialog: bool,

    // New item form
    pub new_item_name: String,
    pub new_item_type: String,

    // Currently editing item data
    pub editing_item: Option<EditingItem>,
}

/// Item being edited
#[derive(Debug, Clone, Default)]
pub struct EditingItem {
    pub id: u32,
    pub name: String,
    pub item_type: String,
    pub grants_ability: Option<u32>,
    pub attack_power: f32,
    pub defense: f32,
    pub max_health: f32,
    pub max_mana: f32,
    pub crit_chance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemListItem {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub item_type: String,
}

/// Enemies editor state
#[derive(Debug, Clone, Default)]
pub struct EnemiesEditorState {
    /// List of all enemies
    pub enemy_list: Vec<EnemyListItem>,

    /// Currently selected enemy ID
    pub selected_enemy: Option<u32>,

    /// Search query
    pub search_query: String,

    /// Show create new enemy dialog
    pub show_create_dialog: bool,

    // New enemy form
    pub new_enemy_name: String,

    // Currently editing enemy data
    pub editing_enemy: Option<EditingEnemy>,
}

/// Enemy being edited
#[derive(Debug, Clone, Default)]
pub struct EditingEnemy {
    pub id: u32,
    pub name: String,
    pub max_health: f32,
    pub attack_power: f32,
    pub defense: f32,
    pub move_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyListItem {
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

/// NPCs editor state
#[derive(Debug, Clone, Default)]
pub struct NpcsEditorState {
    /// List of all NPCs
    pub npc_list: Vec<NpcListItem>,

    /// Currently selected NPC ID
    pub selected_npc: Option<u32>,

    /// Filter by role
    pub role_filter: Option<String>,

    /// Search query
    pub search_query: String,

    /// Show create new NPC dialog
    pub show_create_dialog: bool,

    /// New NPC form - name
    pub new_npc_name: String,

    /// New NPC form - role
    pub new_npc_role: String,

    /// Currently editing NPC data
    pub editing_npc: Option<EditingNpc>,
}

/// NPC being edited
#[derive(Debug, Clone, Default)]
pub struct EditingNpc {
    pub id: u32,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcListItem {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub role: String,
}

/// Quests editor state
#[derive(Debug, Clone, Default)]
pub struct QuestsEditorState {
    /// List of all quests
    pub quest_list: Vec<QuestListItem>,

    /// Currently selected quest ID
    pub selected_quest: Option<u32>,

    /// Filter by type
    pub type_filter: Option<String>,

    /// Search query
    pub search_query: String,

    /// Show create new quest dialog
    pub show_create_dialog: bool,

    /// New quest form - name
    pub new_quest_name: String,

    /// New quest form - type
    pub new_quest_type: String,

    /// Currently editing quest data
    pub editing_quest: Option<EditingQuest>,
}

/// Quest being edited
#[derive(Debug, Clone, Default)]
pub struct EditingQuest {
    pub id: u32,
    pub name: String,
    pub quest_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestListItem {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub quest_type: String,
}

/// Abilities editor state
#[derive(Debug, Clone, Default)]
pub struct AbilitiesEditorState {
    /// List of all abilities
    pub ability_list: Vec<AbilityListItem>,

    /// Currently selected ability ID
    pub selected_ability: Option<u32>,

    /// Filter by type
    pub type_filter: Option<String>,

    /// Search query
    pub search_query: String,

    /// Show create new ability dialog
    pub show_create_dialog: bool,

    /// New ability form - name
    pub new_ability_name: String,

    /// New ability form - type
    pub new_ability_type: String,

    /// Currently editing ability data
    pub editing_ability: Option<EditingAbility>,
}

/// Ability being edited
#[derive(Debug, Clone, Default)]
pub struct EditingAbility {
    pub id: u32,
    pub name: String,
    pub ability_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityListItem {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub ability_type: String,
}

/// Loot tables editor state
#[derive(Debug, Clone, Default)]
pub struct LootEditorState {
    /// List of all loot tables
    pub loot_table_list: Vec<LootTableListItem>,

    /// Currently selected loot table ID
    pub selected_loot_table: Option<String>,

    /// Filter by type
    pub type_filter: Option<String>,

    /// Search query
    pub search_query: String,

    /// Show create new loot table dialog
    pub show_create_dialog: bool,

    /// New loot table form - name
    pub new_loot_table_name: String,

    /// New loot table form - type
    pub new_loot_table_type: String,

    /// Currently editing loot table data
    pub editing_loot_table: Option<EditingLootTable>,
}

/// Loot table being edited
#[derive(Debug, Clone, Default)]
pub struct EditingLootTable {
    pub id: String,
    pub name: String,
    pub table_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootTableListItem {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub table_type: String,
}

/// Assets browser state
#[derive(Debug, Clone, Default)]
pub struct AssetsEditorState {
    /// Current folder path
    pub current_folder: Option<String>,

    /// List of assets in current folder
    pub asset_list: Vec<AssetListItem>,

    /// Currently selected asset
    pub selected_asset: Option<String>,

    /// Search query
    pub search_query: String,

    /// Filter by asset type
    pub type_filter: Option<String>,

    /// View mode (grid or list)
    pub view_mode: AssetViewMode,

    /// Show upload dialog
    pub show_upload_dialog: bool,

    /// Upload type selection
    pub upload_type: Option<String>,

    /// Upload target folder
    pub upload_folder: String,

    /// Show new folder dialog
    pub show_new_folder_dialog: bool,

    /// New folder name input
    pub new_folder_name: String,

    /// Show delete confirmation dialog
    pub show_delete_confirm: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum AssetViewMode {
    #[default]
    Grid,
    List,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetListItem {
    pub id: String,
    pub path: String,
    pub name: String,
    pub asset_type: String,
    pub size_bytes: u64,
    pub dimensions: Option<(u32, u32)>,
}

/// Detect the API URL based on the current environment
fn detect_api_url() -> String {
    #[cfg(target_family = "wasm")]
    {
        // In WASM, the editor runs separately from the server
        // The server API is on port 8080 by default
        if let Some(window) = web_sys::window() {
            if let Ok(hostname) = window.location().hostname() {
                // Use the same host but the server's HTTP port (8080)
                return format!("http://{}:8080/api/editor", hostname);
            }
        }
        "http://localhost:8080/api/editor".to_string()
    }

    #[cfg(not(target_family = "wasm"))]
    {
        "http://localhost:8080/api/editor".to_string()
    }
}
