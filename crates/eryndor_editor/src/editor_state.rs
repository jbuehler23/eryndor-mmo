//! Editor state management

use bevy::prelude::*;
use bevy_egui::egui;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// The currently active editor tab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorTab {
    #[default]
    World,
    Tilesets,
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
            EditorTab::Tilesets => "Tilesets",
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
            EditorTab::Tilesets,
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

    /// Error popup - displayed as a modal window until dismissed
    pub error_popup: Option<String>,

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

    /// Tilesets editor state
    pub tilesets: TilesetsEditorState,

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

    // Tilemap action flags
    pub action_save_tilemap: bool,
    pub action_load_tilemap: bool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            active_tab: EditorTab::World,
            auth: AuthState::NotAuthenticated,
            api_url: detect_api_url(),
            has_unsaved_changes: false,
            status_message: "Ready".to_string(),
            error_popup: None,
            world: WorldEditorState::new(),
            items: ItemsEditorState::default(),
            enemies: EnemiesEditorState::default(),
            npcs: NpcsEditorState::default(),
            quests: QuestsEditorState::default(),
            abilities: AbilitiesEditorState::default(),
            loot: LootEditorState::default(),
            assets: AssetsEditorState::default(),
            tilesets: TilesetsEditorState::new(),
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
            action_save_tilemap: false,
            action_load_tilemap: false,
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

    // === Tile Painting State ===

    /// Tile palette containing available tiles
    pub tile_palette: TilePaletteState,

    /// Currently selected tile ID for painting
    pub selected_tile: Option<u32>,

    /// Tile brush size (1 = single tile, 2 = 2x2, etc.)
    pub brush_size: u32,

    /// Show ground tiles layer (legacy - use layer visibility)
    pub show_ground_layer: bool,

    /// Show decoration layer (legacy - use layer visibility)
    pub show_decoration_layer: bool,

    /// Show tilemap collision layer (legacy - use layer visibility)
    pub show_tile_collision_layer: bool,

    /// Current zone's tilemap data for editing (legacy format)
    pub editing_tilemap: Option<eryndor_shared::ZoneTilemap>,

    // === New Tiled-Compatible Layer System ===

    /// Current zone's tilemap in new Tiled-compatible format
    pub editing_tilemap_new: Option<eryndor_shared::TilemapMap>,

    /// Currently selected layer ID for painting/editing
    pub selected_layer_id: Option<u32>,

    /// Whether to use the new layer system (false = use legacy system)
    pub use_new_layer_system: bool,

    /// Chunk size for the tilemap (default 16)
    pub chunk_size: u32,

    /// Show layer panel in the UI
    pub show_layer_panel: bool,

    /// Layer panel width
    pub layer_panel_width: f32,

    /// Currently renaming layer ID (for inline rename UI)
    pub renaming_layer: Option<u32>,

    /// Rename text buffer
    pub rename_buffer: String,

    /// Show add layer menu
    pub show_add_layer_menu: bool,

    // === End New Layer System ===

    /// Currently selected entity in the world editor
    pub selected_entity: SelectedEntity,

    /// Undo/redo history for tilemap operations
    pub undo_history: UndoHistory,

    /// Terrain sets for auto-tiling
    pub terrain_sets: TerrainSetState,

    /// Height of the tile palette panel (stored to persist across frames)
    pub tile_palette_height: f32,
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
            grid_size: 16.0, // Match tile size for pixel-perfect editing
            active_tool: WorldTool::Select,
            show_collisions: true,
            show_spawn_regions: true,
            show_create_dialog: false,
            new_zone_name: String::new(),
            new_zone_width: 1920.0,
            new_zone_height: 1080.0,
            // Tile painting defaults
            tile_palette: TilePaletteState::default(),
            selected_tile: None,
            brush_size: 1,
            show_ground_layer: true,
            show_decoration_layer: true,
            show_tile_collision_layer: false,
            editing_tilemap: None,
            // New Tiled-compatible layer system
            editing_tilemap_new: None,
            selected_layer_id: None,
            use_new_layer_system: true, // Enable new system by default
            chunk_size: 16,
            show_layer_panel: true,
            layer_panel_width: 200.0,
            renaming_layer: None,
            rename_buffer: String::new(),
            show_add_layer_menu: false,
            // Other state
            selected_entity: SelectedEntity::None,
            undo_history: UndoHistory::default(),
            terrain_sets: TerrainSetState::default(),
            tile_palette_height: 120.0,
        }
    }

    /// Get the currently selected layer (if any)
    pub fn get_selected_layer(&self) -> Option<&eryndor_shared::MapLayer> {
        let tilemap = self.editing_tilemap_new.as_ref()?;
        let layer_id = self.selected_layer_id?;
        tilemap.get_layer(layer_id)
    }

    /// Get the currently selected layer mutably (if any)
    pub fn get_selected_layer_mut(&mut self) -> Option<&mut eryndor_shared::MapLayer> {
        let layer_id = self.selected_layer_id?;
        self.editing_tilemap_new.as_mut()?.get_layer_mut(layer_id)
    }

    /// Create a new empty tilemap for the zone
    pub fn create_new_tilemap(&mut self, tile_size: u32) {
        let mut tilemap = eryndor_shared::TilemapMap::new(tile_size, tile_size);

        // Add default layers (like Tiled's default setup)
        let ground_id = tilemap.add_tile_layer("Ground");
        tilemap.add_tile_layer("Decorations");
        tilemap.add_tile_layer("Collision");

        // Select the ground layer by default
        self.selected_layer_id = Some(ground_id);
        self.editing_tilemap_new = Some(tilemap);
    }

    /// Add a new tile layer to the current tilemap
    pub fn add_tile_layer(&mut self, name: &str) -> Option<u32> {
        let tilemap = self.editing_tilemap_new.as_mut()?;
        let id = tilemap.add_tile_layer(name);
        self.selected_layer_id = Some(id);
        Some(id)
    }

    /// Add a new object layer to the current tilemap
    pub fn add_object_layer(&mut self, name: &str) -> Option<u32> {
        let tilemap = self.editing_tilemap_new.as_mut()?;
        let id = tilemap.add_object_layer(name);
        self.selected_layer_id = Some(id);
        Some(id)
    }

    /// Delete the currently selected layer
    pub fn delete_selected_layer(&mut self) -> bool {
        if let (Some(tilemap), Some(layer_id)) = (&mut self.editing_tilemap_new, self.selected_layer_id) {
            if tilemap.layers.len() > 1 {
                tilemap.remove_layer(layer_id);
                // Select the first remaining layer
                self.selected_layer_id = tilemap.layers.first().map(|l| l.id);
                return true;
            }
        }
        false
    }

    /// Move the selected layer up (toward front/top)
    pub fn move_selected_layer_up(&mut self) -> bool {
        if let (Some(tilemap), Some(layer_id)) = (&mut self.editing_tilemap_new, self.selected_layer_id) {
            return tilemap.move_layer_up(layer_id);
        }
        false
    }

    /// Move the selected layer down (toward back/bottom)
    pub fn move_selected_layer_down(&mut self) -> bool {
        if let (Some(tilemap), Some(layer_id)) = (&mut self.editing_tilemap_new, self.selected_layer_id) {
            return tilemap.move_layer_down(layer_id);
        }
        false
    }

    /// Toggle visibility of the selected layer
    pub fn toggle_selected_layer_visibility(&mut self) {
        if let Some(layer) = self.get_selected_layer_mut() {
            layer.visible = !layer.visible;
        }
    }

    /// Toggle lock of the selected layer
    pub fn toggle_selected_layer_lock(&mut self) {
        if let Some(layer) = self.get_selected_layer_mut() {
            layer.locked = !layer.locked;
        }
    }
}

/// State for the tile palette in the editor
#[derive(Clone, Default, Debug)]
pub struct TilePaletteState {
    /// Whether the palette has been loaded
    pub loaded: bool,

    // === New Hybrid Tileset System ===

    /// Loaded tileset definitions
    pub tilesets: Vec<TilesetDefinition>,

    /// Currently selected tileset index
    pub selected_tileset: Option<usize>,

    /// Currently selected tile within the tileset (global tile index)
    pub selected_tile_index: Option<u32>,

    /// Multi-tile selection start (for rectangular brush) - (col, row in tileset grid)
    pub selection_start: Option<(u32, u32)>,

    /// Multi-tile selection end (for rectangular brush)
    pub selection_end: Option<(u32, u32)>,

    // === Legacy Single-Tile System (for backwards compatibility) ===

    /// Ground tile entries (legacy)
    pub ground_tiles: Vec<TilePaletteEntry>,

    /// Decoration tile entries (legacy)
    pub decoration_tiles: Vec<TilePaletteEntry>,

    /// Currently selected category tab
    pub selected_category: TileCategory,

    // === Texture Loading for Visual Previews ===

    /// Bevy image handles keyed by image path (for both legacy and new system)
    pub texture_handles: HashMap<u32, Handle<Image>>,

    /// Bevy image handles keyed by image path string (for tileset sources)
    pub tileset_texture_handles: HashMap<String, Handle<Image>>,

    /// egui texture IDs keyed by image path (for tileset display)
    pub tileset_egui_ids: HashMap<String, egui::TextureId>,

    /// egui texture IDs for display in the palette (registered after loading) - legacy
    pub egui_texture_ids: HashMap<u32, egui::TextureId>,

    /// Whether texture loading is in progress
    pub textures_loading: bool,

    /// Whether textures have been registered with egui
    pub textures_registered: bool,

    /// Whether tileset textures are loading
    pub tileset_textures_loading: bool,
}

/// A single entry in the tile palette
#[derive(Debug, Clone)]
pub struct TilePaletteEntry {
    /// Unique tile ID
    pub id: u32,

    /// Display name
    pub name: String,

    /// Asset path for preview
    pub path: String,

    /// Whether this tile has collision
    pub has_collision: bool,
}

/// Tile category for palette filtering
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileCategory {
    #[default]
    Ground,
    Decorations,
    /// Terrain sets for auto-tiling
    Terrain,
    /// New hybrid tileset system (spritesheets + individual images)
    Tilesets,
}

// === Hybrid Tileset System (Spritesheets + Individual Images) ===

/// Source of tiles - either a spritesheet or individual image
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TileSource {
    /// A spritesheet containing a grid of tiles
    #[serde(rename = "spritesheet")]
    Spritesheet {
        /// Path to the spritesheet image
        path: String,
        /// Width of each tile in pixels
        tile_width: u32,
        /// Height of each tile in pixels
        tile_height: u32,
        /// Margin around the spritesheet edge (pixels)
        #[serde(default)]
        margin: u32,
        /// Spacing between tiles (pixels)
        #[serde(default)]
        spacing: u32,
        /// Image dimensions (populated on load)
        #[serde(default)]
        image_width: u32,
        #[serde(default)]
        image_height: u32,
        /// Grid dimensions (calculated from image)
        #[serde(default)]
        columns: u32,
        #[serde(default)]
        rows: u32,
        /// First tile index in this tileset (for continuous indexing)
        #[serde(default)]
        first_tile_index: u32,
    },
    /// A single image as one tile
    #[serde(rename = "single_image")]
    SingleImage {
        /// Path to the image
        path: String,
        /// Tile index in this tileset
        #[serde(default)]
        tile_index: u32,
        /// Display name for this tile
        name: String,
        /// Whether this tile has collision
        #[serde(default)]
        has_collision: bool,
    },
}

impl TileSource {
    /// Get the number of tiles in this source
    pub fn tile_count(&self) -> u32 {
        match self {
            TileSource::Spritesheet { columns, rows, .. } => columns * rows,
            TileSource::SingleImage { .. } => 1,
        }
    }

    /// Get the UV rect for a tile at given index within this source
    /// Returns (u_min, v_min, u_max, v_max)
    pub fn get_tile_uv(&self, local_index: u32) -> (f32, f32, f32, f32) {
        match self {
            TileSource::Spritesheet {
                tile_width, tile_height, margin, spacing,
                image_width, image_height, columns, ..
            } => {
                if *image_width == 0 || *image_height == 0 {
                    return (0.0, 0.0, 1.0, 1.0); // Not loaded yet
                }
                let col = local_index % columns;
                let row = local_index / columns;
                let x = margin + col * (tile_width + spacing);
                let y = margin + row * (tile_height + spacing);

                let u_min = x as f32 / *image_width as f32;
                let v_min = y as f32 / *image_height as f32;
                let u_max = (x + tile_width) as f32 / *image_width as f32;
                let v_max = (y + tile_height) as f32 / *image_height as f32;
                (u_min, v_min, u_max, v_max)
            }
            TileSource::SingleImage { .. } => (0.0, 0.0, 1.0, 1.0), // Full image
        }
    }

    /// Get the image path for this source
    pub fn image_path(&self) -> &str {
        match self {
            TileSource::Spritesheet { path, .. } => path,
            TileSource::SingleImage { path, .. } => path,
        }
    }

    /// Get the first tile index for this source
    pub fn first_index(&self) -> u32 {
        match self {
            TileSource::Spritesheet { first_tile_index, .. } => *first_tile_index,
            TileSource::SingleImage { tile_index, .. } => *tile_index,
        }
    }
}

/// A tileset containing multiple tile sources (spritesheets and/or individual images)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilesetDefinition {
    /// Unique tileset ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Category for organization
    pub category: TileCategory,
    /// Display tile size (for rendering in palette - may differ from source)
    #[serde(default = "default_display_tile_size")]
    pub display_tile_size: u32,
    /// All tile sources in this tileset
    pub sources: Vec<TileSource>,
    /// Total number of tiles across all sources (calculated)
    #[serde(default)]
    pub total_tiles: u32,
    /// Per-tile metadata (collision shapes, terrain assignments, properties)
    #[serde(default)]
    pub tile_metadata: HashMap<u32, TileMetadata>,
    /// Terrain sets defined for this tileset
    #[serde(default)]
    pub terrain_sets: Vec<TilesetTerrainSet>,
}

fn default_display_tile_size() -> u32 { 32 }

impl TilesetDefinition {
    /// Recalculate tile indices after modifying sources
    pub fn recalculate_indices(&mut self) {
        let mut next_index = 0u32;
        for source in &mut self.sources {
            match source {
                TileSource::Spritesheet { first_tile_index, columns, rows, .. } => {
                    *first_tile_index = next_index;
                    next_index += *columns * *rows;
                }
                TileSource::SingleImage { tile_index, .. } => {
                    *tile_index = next_index;
                    next_index += 1;
                }
            }
        }
        self.total_tiles = next_index;
    }

    /// Find the source and local index for a global tile index
    pub fn find_tile(&self, global_index: u32) -> Option<(&TileSource, u32)> {
        for source in &self.sources {
            match source {
                TileSource::Spritesheet { first_tile_index, columns, rows, .. } => {
                    let count = columns * rows;
                    if global_index >= *first_tile_index && global_index < *first_tile_index + count {
                        return Some((source, global_index - first_tile_index));
                    }
                }
                TileSource::SingleImage { tile_index, .. } => {
                    if global_index == *tile_index {
                        return Some((source, 0));
                    }
                }
            }
        }
        None
    }
}

/// Reference to a specific tile (replaces simple u32 tile ID)
/// Packs tileset_id and tile_index into a single u32 for efficient storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct TileRef {
    /// Tileset ID (index into tilesets array)
    pub tileset_id: u16,
    /// Tile index within the tileset
    pub tile_index: u16,
}

impl TileRef {
    pub const EMPTY: TileRef = TileRef { tileset_id: 0, tile_index: 0 };

    pub fn new(tileset_id: u16, tile_index: u16) -> Self {
        Self { tileset_id, tile_index }
    }

    /// Pack into a u32 for storage (tileset in high 16 bits, index in low 16 bits)
    pub fn to_u32(&self) -> u32 {
        ((self.tileset_id as u32) << 16) | (self.tile_index as u32)
    }

    /// Unpack from u32
    pub fn from_u32(packed: u32) -> Self {
        Self {
            tileset_id: (packed >> 16) as u16,
            tile_index: (packed & 0xFFFF) as u16,
        }
    }

    /// Check if this is an empty/null tile reference
    pub fn is_empty(&self) -> bool {
        self.tileset_id == 0 && self.tile_index == 0
    }
}

/// Container for all loaded tilesets
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TilesetsConfig {
    pub tilesets: Vec<TilesetDefinition>,
}

// === Tilesets Editor Tab Types ===

/// State for the Tilesets editor tab
#[derive(Debug, Clone, Default)]
pub struct TilesetsEditorState {
    /// Currently selected tileset index
    pub selected_tileset: Option<usize>,

    /// Currently selected tile index within tileset
    pub selected_tile: Option<u32>,

    /// Current editing mode
    pub edit_mode: TilesetEditMode,

    /// Currently selected terrain set (for terrain mode)
    pub selected_terrain_set: Option<usize>,

    /// Currently selected terrain within set
    pub selected_terrain: Option<usize>,

    /// Collision editor state
    pub collision_editor: CollisionEditorState,

    /// Zoom level for tileset view
    pub zoom: f32,

    /// Pan offset
    pub pan_offset: (f32, f32),

    /// Show grid overlay
    pub show_grid: bool,

    /// Show terrain overlay
    pub show_terrain_overlay: bool,

    /// Show collision shapes
    pub show_collision_shapes: bool,

    /// Show create new tileset dialog
    pub show_create_dialog: bool,

    /// Show asset browser dialog (WASM only)
    pub show_asset_browser: bool,

    /// Asset browser state
    pub asset_browser_manifest: Option<serde_json::Value>,
    pub asset_browser_expanded: std::collections::HashSet<String>,
    pub asset_browser_search: String,

    /// New tileset form data
    pub new_tileset_name: String,
    pub new_tileset_image_path: String,
    pub new_tileset_tile_width: u32,
    pub new_tileset_tile_height: u32,
    pub new_tileset_margin: u32,
    pub new_tileset_spacing: u32,

    /// Import dialog state
    pub show_import_dialog: bool,
    pub import_json_text: String,
}

impl TilesetsEditorState {
    pub fn new() -> Self {
        Self {
            selected_tileset: None,
            selected_tile: None,
            edit_mode: TilesetEditMode::Select,
            selected_terrain_set: None,
            selected_terrain: None,
            collision_editor: CollisionEditorState::default(),
            zoom: 2.0,
            pan_offset: (0.0, 0.0),
            show_grid: true,
            show_terrain_overlay: true,
            show_collision_shapes: true,
            show_create_dialog: false,
            show_asset_browser: false,
            asset_browser_manifest: None,
            asset_browser_expanded: std::collections::HashSet::new(),
            asset_browser_search: String::new(),
            new_tileset_name: String::new(),
            new_tileset_image_path: String::new(),
            new_tileset_tile_width: 16,
            new_tileset_tile_height: 16,
            new_tileset_margin: 0,
            new_tileset_spacing: 0,
            show_import_dialog: false,
            import_json_text: String::new(),
        }
    }
}

/// Edit mode for the tileset editor
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum TilesetEditMode {
    #[default]
    Select,
    Terrain,
    Collision,
}

impl TilesetEditMode {
    pub fn label(&self) -> &'static str {
        match self {
            TilesetEditMode::Select => "Select",
            TilesetEditMode::Terrain => "Terrain",
            TilesetEditMode::Collision => "Collision",
        }
    }

    pub fn all() -> &'static [TilesetEditMode] {
        &[
            TilesetEditMode::Select,
            TilesetEditMode::Terrain,
            TilesetEditMode::Collision,
        ]
    }
}

/// State for the collision shape editor
#[derive(Debug, Clone, Default)]
pub struct CollisionEditorState {
    /// Current shape being drawn
    pub drawing_shape: Option<CollisionShapeType>,

    /// Points collected for polygon
    pub polygon_points: Vec<(f32, f32)>,

    /// Selected shape index for editing
    pub selected_shape: Option<usize>,

    /// Drag start position for rectangle/ellipse
    pub drag_start: Option<(f32, f32)>,
}

/// Type of collision shape to draw
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionShapeType {
    Rectangle,
    Polygon,
    Ellipse,
    Point,
}

impl CollisionShapeType {
    pub fn label(&self) -> &'static str {
        match self {
            CollisionShapeType::Rectangle => "Rectangle",
            CollisionShapeType::Polygon => "Polygon",
            CollisionShapeType::Ellipse => "Ellipse",
            CollisionShapeType::Point => "Point",
        }
    }

    pub fn all() -> &'static [CollisionShapeType] {
        &[
            CollisionShapeType::Rectangle,
            CollisionShapeType::Polygon,
            CollisionShapeType::Ellipse,
            CollisionShapeType::Point,
        ]
    }
}

/// Collision shape types stored per-tile (like Tiled)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CollisionShape {
    #[serde(rename = "rectangle")]
    Rectangle {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    #[serde(rename = "polygon")]
    Polygon {
        points: Vec<(f32, f32)>,
    },
    #[serde(rename = "ellipse")]
    Ellipse {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    #[serde(rename = "point")]
    Point {
        x: f32,
        y: f32,
        name: String,
    },
}

/// Per-tile metadata (collision, terrain, properties)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TileMetadata {
    /// Collision shapes for this tile
    #[serde(default)]
    pub collision_shapes: Vec<CollisionShape>,

    /// Terrain assignments for corners (NW, NE, SW, SE)
    /// Each value is an index into the terrain set's terrains array
    #[serde(default)]
    pub terrain_corners: Option<[Option<usize>; 4]>,

    /// Terrain assignments for edges (N, E, S, W) - for Edge/Mixed modes
    #[serde(default)]
    pub terrain_edges: Option<[Option<usize>; 4]>,

    /// Custom properties
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

/// Terrain set defined within a tileset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilesetTerrainSet {
    pub id: String,
    pub name: String,
    pub set_type: TerrainMatchMode,
    pub terrains: Vec<TerrainDefinition>,
}

impl Default for TilesetTerrainSet {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "New Terrain Set".to_string(),
            set_type: TerrainMatchMode::Corner,
            terrains: Vec::new(),
        }
    }
}

/// A single terrain type within a set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainDefinition {
    pub name: String,
    pub color: [u8; 4],  // RGBA for visual display
    #[serde(default = "default_probability")]
    pub probability: f32,  // For random selection (1.0 = normal)
}

fn default_probability() -> f32 { 1.0 }

impl Default for TerrainDefinition {
    fn default() -> Self {
        Self {
            name: "Terrain".to_string(),
            color: [100, 200, 100, 255],
            probability: 1.0,
        }
    }
}

/// Terrain position for assignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainPosition {
    // Corners
    NW,
    NE,
    SW,
    SE,
    // Edges
    N,
    E,
    S,
    W,
}

// === Auto-Tiling / Terrain Set System ===

/// Terrain matching mode (determines how neighbors affect tile selection)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TerrainMatchMode {
    /// 4 corners (NW, NE, SE, SW) - 16 tile variants (2^4)
    /// Best for: grass-to-dirt edges, simple terrain transitions
    #[default]
    Corner,
    /// 4 sides (N, E, S, W) - 16 tile variants (2^4)
    /// Best for: walls, fences, paths
    Edge,
    /// 8 neighbors (4 corners + 4 sides) - 47-256 tile variants
    /// Best for: complex terrain with full corner/edge handling
    Mixed,
}

impl TerrainMatchMode {
    pub fn label(&self) -> &'static str {
        match self {
            TerrainMatchMode::Corner => "Corner (4-bit)",
            TerrainMatchMode::Edge => "Edge (4-bit)",
            TerrainMatchMode::Mixed => "Mixed (8-bit)",
        }
    }

    pub fn all() -> &'static [TerrainMatchMode] {
        &[
            TerrainMatchMode::Corner,
            TerrainMatchMode::Edge,
            TerrainMatchMode::Mixed,
        ]
    }

    /// Number of neighbor positions checked
    pub fn neighbor_count(&self) -> usize {
        match self {
            TerrainMatchMode::Corner | TerrainMatchMode::Edge => 4,
            TerrainMatchMode::Mixed => 8,
        }
    }

    /// Maximum possible bitmask value
    pub fn max_bitmask(&self) -> u8 {
        match self {
            TerrainMatchMode::Corner | TerrainMatchMode::Edge => 15, // 2^4 - 1
            TerrainMatchMode::Mixed => 255, // 2^8 - 1
        }
    }
}

/// A single tile in a terrain set, associated with a bitmask
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TerrainTile {
    /// The tile ID from the tileset
    pub tile_id: u32,
    /// Bitmask value for neighbor matching
    /// For Corner mode: bits 0-3 = NW, NE, SE, SW
    /// For Edge mode: bits 0-3 = N, E, S, W
    /// For Mixed mode: bits 0-7 = N, NE, E, SE, S, SW, W, NW
    pub bitmask: u8,
}

/// A terrain set defines a group of tiles that auto-tile together
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TerrainSet {
    /// Unique name for this terrain set
    pub name: String,
    /// Matching mode (determines neighbor check algorithm)
    pub mode: TerrainMatchMode,
    /// The tile to use when completely surrounded (bitmask = all 1s)
    pub inner_tile: u32,
    /// The tile to use when no neighbors match (bitmask = 0)
    pub outer_tile: u32,
    /// All tile variants in this set (keyed by bitmask)
    pub tiles: Vec<TerrainTile>,
}

impl Default for TerrainSet {
    fn default() -> Self {
        Self {
            name: "New Terrain".to_string(),
            mode: TerrainMatchMode::Corner,
            inner_tile: 0,
            outer_tile: 0,
            tiles: Vec::new(),
        }
    }
}

impl TerrainSet {
    /// Get the tile ID for a given bitmask, falling back to inner/outer tiles
    pub fn get_tile_for_bitmask(&self, bitmask: u8) -> u32 {
        // Check for exact bitmask match
        if let Some(tile) = self.tiles.iter().find(|t| t.bitmask == bitmask) {
            return tile.tile_id;
        }

        // Fallback to inner tile (all neighbors match) or outer tile (no neighbors match)
        let max = self.mode.max_bitmask();
        if bitmask == max {
            self.inner_tile
        } else if bitmask == 0 {
            self.outer_tile
        } else {
            // For partial matches, try to find closest match
            // This is a simple fallback - could be improved with smarter matching
            self.inner_tile
        }
    }
}

/// State for terrain sets in the editor
#[derive(Debug, Clone, Default)]
pub struct TerrainSetState {
    /// All defined terrain sets
    pub terrain_sets: Vec<TerrainSet>,
    /// Currently selected terrain set index
    pub selected_set: Option<usize>,
    /// Whether to use auto-tiling when painting
    pub auto_tile_enabled: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum WorldTool {
    #[default]
    Select,
    Pan,
    PlaceEntity,
    DrawCollision,
    DrawSpawnRegion,
    // === Legacy Layer-specific Tools (for backward compatibility) ===
    /// Paint ground tiles (legacy - use PaintTile with layer selection)
    PaintGround,
    /// Paint decoration tiles (legacy - use PaintTile with layer selection)
    PaintDecoration,
    /// Paint collision layer (legacy - use PaintTile with layer selection)
    PaintTileCollision,
    // === New Generic Layer Tools ===
    /// Paint tiles to the currently selected layer
    PaintTile,
    /// Place/edit objects on the currently selected object layer
    PlaceObject,
    /// Erase tiles/decorations/collision from selected layer
    Erase,
    /// Fill tool (bucket fill) - floods connected area with selected tile
    Fill,
    /// Rectangle select/fill tool
    RectangleTool,
}

impl WorldTool {
    /// Get the display name for this tool
    pub fn label(&self) -> &'static str {
        match self {
            WorldTool::Select => "Select",
            WorldTool::Pan => "Pan",
            WorldTool::PlaceEntity => "Place Entity",
            WorldTool::DrawCollision => "Draw Collision",
            WorldTool::DrawSpawnRegion => "Draw Spawn",
            WorldTool::PaintGround => "Paint Ground",
            WorldTool::PaintDecoration => "Paint Decor",
            WorldTool::PaintTileCollision => "Paint Collision",
            WorldTool::PaintTile => "Paint Tile",
            WorldTool::PlaceObject => "Place Object",
            WorldTool::Erase => "Erase",
            WorldTool::Fill => "Fill",
            WorldTool::RectangleTool => "Rectangle",
        }
    }

    /// Get all available tools for the new layer system
    pub fn layer_tools() -> &'static [WorldTool] {
        &[
            WorldTool::Select,
            WorldTool::Pan,
            WorldTool::PaintTile,
            WorldTool::PlaceObject,
            WorldTool::Erase,
            WorldTool::Fill,
            WorldTool::RectangleTool,
        ]
    }

    /// Get all available tools for the legacy system
    pub fn legacy_tools() -> &'static [WorldTool] {
        &[
            WorldTool::Select,
            WorldTool::Pan,
            WorldTool::PaintGround,
            WorldTool::PaintDecoration,
            WorldTool::PaintTileCollision,
            WorldTool::Erase,
            WorldTool::Fill,
        ]
    }

    /// Check if this tool can paint/edit the given layer type
    pub fn can_edit_layer(&self, layer_type: &str) -> bool {
        match self {
            WorldTool::PaintTile | WorldTool::Fill | WorldTool::RectangleTool => {
                layer_type == "tilelayer"
            }
            WorldTool::PlaceObject => layer_type == "objectgroup",
            WorldTool::Erase => layer_type == "tilelayer" || layer_type == "objectgroup",
            WorldTool::Select => true,
            _ => false,
        }
    }
}

// === Undo/Redo System ===

/// Layer type for tile operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileLayer {
    Ground,
    Decoration,
    Collision,
}

/// A single tile operation that can be undone/redone
#[derive(Debug, Clone)]
pub struct TileOperation {
    pub tile_x: i32,
    pub tile_y: i32,
    pub layer: TileLayer,
    pub old_value: u32,
    pub new_value: u32,
}

/// A batch of operations grouped together (e.g., a single brush stroke or fill)
#[derive(Debug, Clone, Default)]
pub struct UndoEntry {
    pub operations: Vec<TileOperation>,
}

/// Undo/redo history for tilemap operations
#[derive(Debug, Clone)]
pub struct UndoHistory {
    /// Stack of undoable operations
    pub undo_stack: Vec<UndoEntry>,
    /// Stack of redoable operations
    pub redo_stack: Vec<UndoEntry>,
    /// Current batch being recorded (during drag operations)
    pub current_batch: Option<Vec<TileOperation>>,
    /// Max entries to keep (memory limit)
    pub max_entries: usize,
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_batch: None,
            max_entries: 100, // Keep last 100 operations
        }
    }
}

impl UndoHistory {
    /// Start recording a batch of operations (call at start of drag/fill)
    pub fn begin_batch(&mut self) {
        self.current_batch = Some(Vec::new());
    }

    /// Record a single tile operation
    pub fn record(&mut self, op: TileOperation) {
        // Skip no-op changes
        if op.old_value == op.new_value {
            return;
        }

        if let Some(ref mut batch) = self.current_batch {
            batch.push(op);
        } else {
            // No batch active, create single-operation entry
            self.push_entry(UndoEntry { operations: vec![op] });
        }
    }

    /// End the current batch and push it to the undo stack
    pub fn end_batch(&mut self) {
        if let Some(batch) = self.current_batch.take() {
            if !batch.is_empty() {
                self.push_entry(UndoEntry { operations: batch });
            }
        }
    }

    /// Push an entry to the undo stack
    fn push_entry(&mut self, entry: UndoEntry) {
        self.undo_stack.push(entry);
        // Clear redo stack when new action is performed
        self.redo_stack.clear();
        // Trim if over max
        while self.undo_stack.len() > self.max_entries {
            self.undo_stack.remove(0);
        }
    }

    /// Pop the last entry for undo (returns operations to reverse)
    pub fn pop_undo(&mut self) -> Option<UndoEntry> {
        self.undo_stack.pop()
    }

    /// Push an entry to the redo stack
    pub fn push_redo(&mut self, entry: UndoEntry) {
        self.redo_stack.push(entry);
    }

    /// Pop from redo stack
    pub fn pop_redo(&mut self) -> Option<UndoEntry> {
        self.redo_stack.pop()
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

/// What is currently selected in the world editor
#[derive(Debug, Clone, Default)]
pub enum SelectedEntity {
    #[default]
    None,
    /// A tile is selected
    Tile {
        tile_x: i32,
        tile_y: i32,
        ground_id: u32,
        decoration_id: u32,
        has_collision: bool,
    },
    /// An NPC is selected (by index in zone spawns)
    Npc { index: usize, name: String },
    /// An enemy spawn region is selected
    EnemyRegion { region_id: String },
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
    // Combat/AI
    pub aggro_range: f32,
    pub leash_range: f32,
    pub respawn_delay: f32,
    // Visual
    pub visual_shape: String,  // "Circle", "Square"
    pub visual_color: [f32; 4],  // RGBA
    pub visual_size: f32,
    // Loot
    pub gold_min: u32,
    pub gold_max: u32,
    pub loot_items: Vec<EditingLootItem>,
}

/// Loot item in enemy's loot table
#[derive(Debug, Clone, Default)]
pub struct EditingLootItem {
    pub item_id: u32,
    pub drop_chance: f32,  // 0.0 to 1.0
    pub quantity_min: u32,
    pub quantity_max: u32,
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
    #[serde(default)]
    pub aggro_range: f32,
    #[serde(default)]
    pub leash_range: f32,
    #[serde(default)]
    pub respawn_delay: f32,
    #[serde(default)]
    pub visual: VisualDataJson,
    #[serde(default)]
    pub loot_table: LootTableJson,
}

/// Visual data from JSON
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VisualDataJson {
    #[serde(default)]
    pub shape: String,
    #[serde(default)]
    pub color: [f32; 4],
    #[serde(default = "default_visual_size")]
    pub size: f32,
}

fn default_visual_size() -> f32 { 16.0 }

/// Loot table data from JSON
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LootTableJson {
    #[serde(default)]
    pub gold_min: u32,
    #[serde(default)]
    pub gold_max: u32,
    #[serde(default)]
    pub items: Vec<LootItemData>,
}

/// Loot item data from server
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LootItemData {
    pub item_id: u32,
    #[serde(default = "default_drop_chance")]
    pub drop_chance: f32,
    #[serde(default = "default_quantity")]
    pub quantity_min: u32,
    #[serde(default = "default_quantity")]
    pub quantity_max: u32,
}

fn default_drop_chance() -> f32 { 1.0 }
fn default_quantity() -> u32 { 1 }

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

/// NPC being edited - mirrors NpcSpawnDef from server
#[derive(Debug, Clone, Default)]
pub struct EditingNpc {
    pub id: u32,
    pub name: String,
    pub npc_type: String,  // "QuestGiver" or "Trainer"
    pub position_x: f32,
    pub position_y: f32,
    pub quests: Vec<u32>,  // Quest IDs for quest givers
    pub trainer_items: Vec<EditingTrainerItem>,  // Items for sale by trainers
    pub visual_shape: String,  // "Circle", "Rectangle"
    pub visual_color: [f32; 4],  // RGBA
    pub visual_size: f32,
}

/// Item sold by a trainer NPC
#[derive(Debug, Clone, Default)]
pub struct EditingTrainerItem {
    pub item_id: u32,
    pub cost: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcListItem {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub npc_type: String,
    #[serde(default)]
    pub quests: Vec<u32>,
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

/// Quest being edited - mirrors QuestDefinition from server
#[derive(Debug, Clone, Default)]
pub struct EditingQuest {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub objectives: Vec<EditingQuestObjective>,
    pub reward_exp: u32,
    pub proficiency_requirements: Vec<EditingProficiencyRequirement>,
    pub reward_abilities: Vec<u32>,
}

/// Quest objective types
#[derive(Debug, Clone)]
pub enum EditingQuestObjective {
    ObtainItem { item_id: u32, count: u32 },
    KillEnemy { enemy_type: u32, count: u32 },
    TalkToNpc { npc_id: u32 },
}

impl Default for EditingQuestObjective {
    fn default() -> Self {
        Self::TalkToNpc { npc_id: 1 }
    }
}

/// Proficiency requirement for quests
#[derive(Debug, Clone, Default)]
pub struct EditingProficiencyRequirement {
    pub weapon_type: String,  // "Sword", "Dagger", "Wand", etc.
    pub level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestListItem {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub reward_exp: u32,
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

/// Ability being edited - mirrors AbilityDefinition from eryndor_shared
#[derive(Debug, Clone, Default)]
pub struct EditingAbility {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub damage_multiplier: f32,
    pub cooldown: f32,
    pub range: f32,
    pub mana_cost: f32,
    pub ability_effects: Vec<EditingAbilityEffect>,
    pub unlock_requirement: EditingUnlockRequirement,
}

/// Represents an ability effect in the editor
#[derive(Debug, Clone)]
pub enum EditingAbilityEffect {
    DirectDamage { multiplier: f32 },
    DamageOverTime { duration: f32, ticks: u32, damage_per_tick: f32 },
    AreaOfEffect { radius: f32, max_targets: u32 },
    Buff { duration: f32, attack_power: f32, defense: f32, move_speed: f32 },
    Debuff { duration: f32, debuff_type: EditingDebuffType },
    Mobility { distance: f32, dash_speed: f32 },
    Heal { amount: f32, is_percent: bool },
}

impl Default for EditingAbilityEffect {
    fn default() -> Self {
        Self::DirectDamage { multiplier: 1.0 }
    }
}

/// Debuff types for the editor
#[derive(Debug, Clone, Default)]
pub enum EditingDebuffType {
    #[default]
    Stun,
    Root,
    Slow { move_speed_reduction: f32 },
    Weaken { attack_reduction: f32 },
}

/// Unlock requirement for abilities
#[derive(Debug, Clone, Default)]
pub enum EditingUnlockRequirement {
    #[default]
    None,
    Level(u32),
    Quest(u32),
    WeaponProficiency { weapon: String, level: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityListItem {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub damage_multiplier: f32,
    #[serde(default)]
    pub cooldown: f32,
    #[serde(default)]
    pub mana_cost: f32,
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
