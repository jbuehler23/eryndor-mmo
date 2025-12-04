pub mod schema;
pub mod project;
pub mod map;
pub mod ui;
pub mod tools;
pub mod autotile;
pub mod automap;
pub mod templates;
pub mod render;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use std::path::PathBuf;

use project::Project;
use schema::default_schema;
use ui::{DialogueEditorState, EditorTool, EditorUiPlugin, PendingAction, SchemaEditorState, Selection, SpriteEditorState, TilesetEditorState, TerrainPaintState, ToolMode};
use render::MapRenderPlugin;
use tools::EditorToolsPlugin;

/// Resource storing the base assets path for converting absolute paths to relative
#[derive(Resource, Default)]
pub struct AssetsBasePath(pub PathBuf);

impl AssetsBasePath {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    /// Convert an absolute path to a path relative to the assets folder.
    /// Returns the relative path if the absolute path is within the assets folder,
    /// otherwise returns the original path.
    pub fn to_relative(&self, absolute_path: &std::path::Path) -> PathBuf {
        // Normalize paths for comparison (handle Windows path quirks)
        let assets_path = self.0.canonicalize().unwrap_or_else(|_| self.0.clone());
        let file_path = absolute_path.canonicalize().unwrap_or_else(|_| absolute_path.to_path_buf());

        // Try to strip the assets prefix
        if let Ok(relative) = file_path.strip_prefix(&assets_path) {
            // Convert backslashes to forward slashes for Bevy
            let relative_str = relative.to_string_lossy().replace('\\', "/");
            PathBuf::from(relative_str)
        } else {
            // Not inside assets folder - return original (may fail to load)
            absolute_path.to_path_buf()
        }
    }
}

/// Main editor plugin
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
        .add_plugins(EditorUiPlugin)
        .add_plugins(MapRenderPlugin)
        .add_plugins(EditorToolsPlugin)
        .init_resource::<EditorState>()
        .insert_resource(Project::new(default_schema()));
    }
}

/// Global editor state
#[derive(Resource)]
pub struct EditorState {
    // Selection
    pub selection: Selection,
    pub selected_layer: Option<usize>,
    pub selected_tileset: Option<uuid::Uuid>,
    pub selected_tile: Option<u32>,
    pub selected_level: Option<uuid::Uuid>,

    // Tools
    pub current_tool: EditorTool,
    pub tool_mode: ToolMode,
    pub show_grid: bool,
    pub zoom: f32,
    pub camera_offset: bevy::math::Vec2,

    // Dialogs
    pub show_new_project_dialog: bool,
    pub show_new_level_dialog: bool,
    pub show_new_tileset_dialog: bool,
    pub show_about_dialog: bool,
    pub show_schema_editor: bool,
    pub schema_editor_state: SchemaEditorState,
    pub error_message: Option<String>,

    // New project dialog state
    pub new_project_name: String,
    pub new_project_schema_path: Option<PathBuf>,

    // New level dialog state
    pub new_level_name: String,
    pub new_level_width: u32,
    pub new_level_height: u32,

    // New tileset dialog state
    pub new_tileset_name: String,
    pub new_tileset_path: String,
    pub new_tileset_tile_size: u32,

    // Add image to tileset dialog state
    pub show_add_tileset_image_dialog: bool,
    pub add_image_name: String,
    pub add_image_path: String,

    // Pending actions
    pub pending_action: Option<PendingAction>,
    pub create_new_instance: Option<String>,

    // Sprite editor
    pub show_sprite_editor: bool,
    pub sprite_editor_state: SpriteEditorState,

    // Dialogue editor
    pub show_dialogue_editor: bool,
    pub dialogue_editor_state: DialogueEditorState,

    // Tile painting
    pub is_painting: bool,
    pub last_painted_tile: Option<(u32, u32)>,

    // Autotile / Terrain (Legacy 47-tile blob)
    pub selected_terrain: Option<uuid::Uuid>,
    pub show_new_terrain_dialog: bool,
    pub new_terrain_name: String,
    pub new_terrain_first_tile: u32,

    // Tiled-Style Terrain System
    pub selected_terrain_set: Option<uuid::Uuid>,
    pub selected_terrain_in_set: Option<usize>,
    pub show_new_terrain_set_dialog: bool,
    pub new_terrain_set_type: autotile::TerrainSetType,
    pub show_add_terrain_to_set_dialog: bool,
    pub new_terrain_color: [f32; 3],

    // Tileset & Terrain Editor
    pub show_tileset_editor: bool,
    pub tileset_editor_state: TilesetEditorState,

    // Terrain painting palette
    pub terrain_paint_state: TerrainPaintState,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            selection: Selection::None,
            selected_layer: None,
            selected_tileset: None,
            selected_tile: None,
            selected_level: None,

            current_tool: EditorTool::Select,
            tool_mode: ToolMode::Point,
            show_grid: true,
            zoom: 1.0,
            camera_offset: bevy::math::Vec2::ZERO,

            show_new_project_dialog: false,
            show_new_level_dialog: false,
            show_new_tileset_dialog: false,
            show_about_dialog: false,
            show_schema_editor: false,
            schema_editor_state: SchemaEditorState::default(),
            error_message: None,

            new_project_name: String::new(),
            new_project_schema_path: None,

            new_level_name: "New Level".to_string(),
            new_level_width: 50,
            new_level_height: 50,

            new_tileset_name: "New Tileset".to_string(),
            new_tileset_path: String::new(),
            new_tileset_tile_size: 32,

            show_add_tileset_image_dialog: false,
            add_image_name: String::new(),
            add_image_path: String::new(),

            pending_action: None,
            create_new_instance: None,

            show_sprite_editor: false,
            sprite_editor_state: SpriteEditorState::default(),

            show_dialogue_editor: false,
            dialogue_editor_state: DialogueEditorState::default(),

            is_painting: false,
            last_painted_tile: None,

            selected_terrain: None,
            show_new_terrain_dialog: false,
            new_terrain_name: String::new(),
            new_terrain_first_tile: 0,

            selected_terrain_set: None,
            selected_terrain_in_set: None,
            show_new_terrain_set_dialog: false,
            new_terrain_set_type: autotile::TerrainSetType::Corner,
            show_add_terrain_to_set_dialog: false,
            new_terrain_color: [0.0, 1.0, 0.0], // Default: green

            show_tileset_editor: false,
            tileset_editor_state: TilesetEditorState::default(),

            terrain_paint_state: TerrainPaintState::new(),
        }
    }
}
