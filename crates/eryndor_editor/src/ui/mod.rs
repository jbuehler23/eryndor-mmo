mod menu_bar;
mod toolbar;
mod inspector;
mod tree_view;
mod tileset;
mod dialogs;
mod schema_editor;
mod animation_editor;
mod dialogue_editor;
mod terrain;
mod tileset_editor;
mod terrain_palette;

pub use menu_bar::*;
pub use toolbar::{render_toolbar, EditorTool, ToolMode};
pub use inspector::{render_inspector, InspectorResult, Selection};
pub use tree_view::{render_tree_view, TreeViewResult};
pub use tileset::{render_tileset_palette, render_tileset_palette_with_cache, open_tileset_dialog};
pub use dialogs::*;
pub use schema_editor::{render_schema_editor, SchemaEditorState};
pub use animation_editor::{render_animation_editor, SpriteEditorState, AnimationEditorResult, open_spritesheet_dialog};
pub use dialogue_editor::{render_dialogue_editor, DialogueEditorState, DialogueEditorResult};
pub use tileset_editor::{render_tileset_editor, TilesetEditorState};
pub use terrain_palette::{render_terrain_palette, TerrainPaintState};

use std::collections::HashMap;
use std::path::PathBuf;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass, EguiTextureHandle};
use uuid::Uuid;

use crate::project::Project;
use crate::EditorState;

/// Resource to track spritesheet texture loading
#[derive(Resource, Default)]
pub struct SpritesheetTextureCache {
    /// Loaded textures: path -> (handle, texture_id, width, height)
    pub loaded: HashMap<String, (Handle<Image>, egui::TextureId, f32, f32)>,
    /// Pending texture loads: path -> handle
    pub pending: HashMap<String, Handle<Image>>,
}

/// Resource to track tileset texture loading
#[derive(Resource, Default)]
pub struct TilesetTextureCache {
    /// Loaded tileset image textures: image_id -> (handle, texture_id, width, height)
    /// Note: keyed by TilesetImage.id, not Tileset.id
    pub loaded: HashMap<Uuid, (Handle<Image>, egui::TextureId, f32, f32)>,
    /// Pending tileset image loads: image_id -> (path, handle)
    pub pending: HashMap<Uuid, (PathBuf, Handle<Image>)>,
    /// Mapping from tileset_id to its first image's id (for backward compat)
    /// This allows code using tileset_id to still find the primary texture
    pub tileset_primary_image: HashMap<Uuid, Uuid>,
}

/// Main UI plugin
pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>()
            .init_resource::<SpritesheetTextureCache>()
            .init_resource::<TilesetTextureCache>()
            .add_systems(Update, (
                update_animation_preview,
                load_spritesheet_textures,
                load_tileset_textures,
            ))
            .add_systems(EguiPrimaryContextPass, render_ui);
    }
}

/// UI state for panel visibility and sizes
#[derive(Resource)]
pub struct UiState {
    pub show_tree_view: bool,
    pub show_inspector: bool,
    pub tree_view_width: f32,
    pub inspector_width: f32,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            show_tree_view: true,
            show_inspector: true,
            tree_view_width: 200.0,
            inspector_width: 250.0,
        }
    }
}

/// System to update animation preview playback
fn update_animation_preview(
    mut editor_state: ResMut<EditorState>,
    time: Res<Time>,
) {
    if !editor_state.show_sprite_editor {
        return;
    }

    let state = &mut editor_state.sprite_editor_state;

    if !state.preview_playing {
        return;
    }

    // Get animation info
    let Some(anim_name) = &state.selected_animation else {
        return;
    };
    let Some(anim) = state.sprite_data.animations.get(anim_name) else {
        return;
    };

    if anim.frames.is_empty() {
        return;
    }

    // Update timer
    state.preview_timer += time.delta_secs() * 1000.0; // Convert to ms

    let frame_duration = anim.frame_duration_ms as f32;
    if state.preview_timer >= frame_duration {
        state.preview_timer -= frame_duration;
        state.preview_frame = (state.preview_frame + 1) % anim.frames.len();
    }
}

/// System to load spritesheet textures and register them with egui
fn load_spritesheet_textures(
    mut editor_state: ResMut<EditorState>,
    mut cache: ResMut<SpritesheetTextureCache>,
    mut contexts: EguiContexts,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
) {
    if !editor_state.show_sprite_editor {
        return;
    }

    let state = &mut editor_state.sprite_editor_state;
    let sheet_path = state.sprite_data.sheet_path.clone();

    // Skip if no path or already loaded
    if sheet_path.is_empty() {
        return;
    }

    // Check if already loaded in cache
    if let Some((_, texture_id, width, height)) = cache.loaded.get(&sheet_path) {
        // Update sprite editor state with cached values
        if state.spritesheet_texture_id.is_none() || state.loaded_sheet_path.as_ref() != Some(&sheet_path) {
            state.set_texture(*texture_id, *width, *height);
        }
        return;
    }

    // Check if load is pending
    if let Some(handle) = cache.pending.get(&sheet_path).cloned() {
        // Check if the image has finished loading
        if let Some(image) = images.get(&handle) {
            let width = image.width() as f32;
            let height = image.height() as f32;

            // Register with egui
            let texture_id = contexts.add_image(EguiTextureHandle::Strong(handle.clone()));

            // Cache the result
            cache.loaded.insert(sheet_path.clone(), (handle.clone(), texture_id, width, height));
            cache.pending.remove(&sheet_path);

            // Update sprite editor state
            state.set_texture(texture_id, width, height);
        }
        return;
    }

    // Start loading the image
    let handle: Handle<Image> = asset_server.load(&sheet_path);
    cache.pending.insert(sheet_path, handle);
}

/// System to load tileset textures and register them with egui
fn load_tileset_textures(
    mut project: ResMut<Project>,
    mut cache: ResMut<TilesetTextureCache>,
    mut contexts: EguiContexts,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
) {
    // Migrate legacy tilesets and collect images to process
    for tileset in project.tilesets.iter_mut() {
        tileset.migrate_to_multi_image();
    }

    // Collect all tileset images to process
    let images_to_process: Vec<_> = project.tilesets.iter()
        .flat_map(|tileset| {
            let tileset_id = tileset.id;
            let tile_size = tileset.tile_size;
            tileset.images.iter().enumerate().map(move |(img_idx, image)| {
                (tileset_id, img_idx, image.id, image.path.clone(), tile_size)
            })
        })
        .filter(|(_, _, img_id, _, _)| !cache.loaded.contains_key(img_id))
        .collect();

    for (tileset_id, img_idx, image_id, image_path, tile_size) in images_to_process {
        // Check if load is pending
        if let Some((_, handle)) = cache.pending.get(&image_id).cloned() {
            // Check if the image has finished loading
            if let Some(image) = images.get(&handle) {
                let width = image.width() as f32;
                let height = image.height() as f32;

                // Register with egui
                let texture_id = contexts.add_image(EguiTextureHandle::Strong(handle.clone()));

                // Cache the result
                cache.loaded.insert(image_id, (handle.clone(), texture_id, width, height));
                cache.pending.remove(&image_id);

                // Track primary image for tileset (first image)
                if img_idx == 0 {
                    cache.tileset_primary_image.insert(tileset_id, image_id);
                }

                // Update image dimensions based on actual size
                if let Some(tileset) = project.tilesets.iter_mut().find(|t| t.id == tileset_id) {
                    if let Some(tileset_image) = tileset.images.iter_mut().find(|i| i.id == image_id) {
                        tileset_image.columns = (width as u32) / tile_size.max(1);
                        tileset_image.rows = (height as u32) / tile_size.max(1);
                    }
                    // Also update legacy columns/rows if this is first image
                    if img_idx == 0 {
                        tileset.columns = (width as u32) / tile_size.max(1);
                        tileset.rows = (height as u32) / tile_size.max(1);
                    }
                }
            }
            continue;
        }

        // Start loading the image
        let path_string = image_path.to_string_lossy().into_owned();
        let handle: Handle<Image> = asset_server.load(path_string);
        cache.pending.insert(image_id, (image_path, handle));

        // Track primary image for tileset
        if img_idx == 0 {
            cache.tileset_primary_image.insert(tileset_id, image_id);
        }
    }
}

/// Main UI rendering system
fn render_ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut editor_state: ResMut<EditorState>,
    mut project: ResMut<Project>,
    tileset_cache: Res<TilesetTextureCache>,
    assets_base_path: Res<crate::AssetsBasePath>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Menu bar
    render_menu_bar(ctx, &mut ui_state, &mut editor_state, &mut project);

    // Toolbar
    render_toolbar(ctx, &mut editor_state);

    // Left panel - Tree View
    let mut tree_view_result = TreeViewResult::default();
    if ui_state.show_tree_view {
        egui::SidePanel::left("tree_view")
            .resizable(true)
            .default_width(ui_state.tree_view_width)
            .min_width(150.0)
            .max_width(400.0)
            .show(ctx, |ui| {
                ui_state.tree_view_width = ui.available_width();
                tree_view_result = render_tree_view(ui, &mut editor_state, &mut project);
            });
    }

    // Right panel - Inspector + Terrain Palette
    let mut inspector_result = InspectorResult::default();
    if ui_state.show_inspector {
        egui::SidePanel::right("inspector")
            .resizable(true)
            .default_width(ui_state.inspector_width)
            .min_width(200.0)
            .max_width(500.0)
            .show(ctx, |ui| {
                ui_state.inspector_width = ui.available_width();

                // Split the panel: Inspector at top, Terrain Palette at bottom
                let available_height = ui.available_height();
                let inspector_height = (available_height * 0.5).max(150.0);

                // Top: Inspector
                egui::TopBottomPanel::top("inspector_top")
                    .resizable(true)
                    .default_height(inspector_height)
                    .min_height(100.0)
                    .show_inside(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("inspector_scroll")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                inspector_result = render_inspector(ui, &mut editor_state, &mut project);
                            });
                    });

                // Bottom: Terrain Palette
                egui::CentralPanel::default()
                    .show_inside(ui, |ui| {
                        ui.heading("Terrain & Tiles");
                        ui.separator();
                        render_terrain_palette(ui, &mut editor_state, &project, Some(&tileset_cache));
                    });
            });
    }

    // Handle inspector actions (deletions, sprite editor)
    if let Some(id) = inspector_result.delete_data_instance {
        project.remove_data_instance(id);
        editor_state.selection = Selection::None;
    }
    if let Some((level_id, entity_id)) = inspector_result.delete_entity {
        if let Some(level) = project.get_level_mut(level_id) {
            level.remove_entity(entity_id);
        }
        editor_state.selection = Selection::None;
    }
    if let Some((prop_name, instance_id)) = inspector_result.open_sprite_editor {
        // Get sprite data from instance and open editor
        if let Some(instance) = project.get_data_instance(instance_id) {
            let sprite_data = instance
                .properties
                .get(&prop_name)
                .and_then(|v| crate::schema::SpriteData::from_value(v))
                .unwrap_or_default();
            editor_state.sprite_editor_state.open(instance_id, prop_name, sprite_data);
            editor_state.show_sprite_editor = true;
        }
    }
    if let Some((prop_name, instance_id)) = inspector_result.open_dialogue_editor {
        // Get dialogue data from instance and open editor
        if let Some(instance) = project.get_data_instance(instance_id) {
            let dialogue_data = instance
                .properties
                .get(&prop_name)
                .and_then(|v| crate::schema::DialogueTree::from_value(v))
                .unwrap_or_else(|| crate::schema::DialogueTree::new());
            editor_state.dialogue_editor_state.open(instance_id, prop_name, dialogue_data);
            editor_state.show_dialogue_editor = true;
        }
    }

    // Handle tree view actions (duplications, deletions)
    if let Some(id) = tree_view_result.duplicate_data {
        if let Some(new_id) = project.duplicate_data_instance(id) {
            editor_state.selection = Selection::DataInstance(new_id);
        }
    }
    if let Some(id) = tree_view_result.delete_data {
        project.remove_data_instance(id);
        editor_state.selection = Selection::None;
    }
    if let Some(id) = tree_view_result.duplicate_level {
        if let Some(new_id) = project.duplicate_level(id) {
            editor_state.selection = Selection::Level(new_id);
        }
    }
    if let Some(id) = tree_view_result.delete_level {
        project.remove_level(id);
        editor_state.selection = Selection::None;
    }
    if let Some((level_id, entity_id)) = tree_view_result.delete_entity {
        if let Some(level) = project.get_level_mut(level_id) {
            level.remove_entity(entity_id);
        }
        editor_state.selection = Selection::None;
    }

    // Handle layer add actions
    if let Some(level_id) = tree_view_result.add_tile_layer {
        // Get tileset_id before mutable borrow of project
        let tileset_id = editor_state.selected_tileset
            .or_else(|| project.tilesets.first().map(|t| t.id))
            .unwrap_or(uuid::Uuid::nil());

        if let Some(level) = project.get_level_mut(level_id) {
            let layer_name = format!("Tile Layer {}", level.layers.len() + 1);
            let layer = crate::map::Layer::new_tile_layer(layer_name, tileset_id, level.width, level.height);
            level.add_layer(layer);
            // Select the new layer
            let new_layer_idx = level.layers.len() - 1;
            editor_state.selection = Selection::Layer(level_id, new_layer_idx);
            editor_state.selected_layer = Some(new_layer_idx);
            editor_state.selected_level = Some(level_id);
        }
    }
    if let Some(level_id) = tree_view_result.add_object_layer {
        if let Some(level) = project.get_level_mut(level_id) {
            let layer_name = format!("Object Layer {}", level.layers.len() + 1);
            let layer = crate::map::Layer::new_object_layer(layer_name);
            level.add_layer(layer);
            // Select the new layer
            let new_layer_idx = level.layers.len() - 1;
            editor_state.selection = Selection::Layer(level_id, new_layer_idx);
            editor_state.selected_layer = Some(new_layer_idx);
            editor_state.selected_level = Some(level_id);
        }
    }

    // Handle layer delete
    if let Some((level_id, layer_index)) = tree_view_result.delete_layer {
        if let Some(level) = project.get_level_mut(level_id) {
            level.remove_layer(layer_index);
            // Clear selection if deleted layer was selected
            if editor_state.selected_layer == Some(layer_index) {
                editor_state.selection = Selection::Level(level_id);
                editor_state.selected_layer = None;
            } else if let Some(sel) = editor_state.selected_layer {
                // Adjust selection index if needed
                if sel > layer_index {
                    editor_state.selected_layer = Some(sel - 1);
                    if let Selection::Layer(_, ref mut idx) = editor_state.selection {
                        *idx = sel - 1;
                    }
                }
            }
            project.mark_dirty();
        }
    }

    // Handle layer move up
    if let Some((level_id, layer_index)) = tree_view_result.move_layer_up {
        if let Some(level) = project.get_level_mut(level_id) {
            if level.move_layer_up(layer_index) {
                // Update selection if needed
                if editor_state.selected_layer == Some(layer_index) {
                    editor_state.selected_layer = Some(layer_index - 1);
                    editor_state.selection = Selection::Layer(level_id, layer_index - 1);
                } else if editor_state.selected_layer == Some(layer_index - 1) {
                    editor_state.selected_layer = Some(layer_index);
                    editor_state.selection = Selection::Layer(level_id, layer_index);
                }
                project.mark_dirty();
            }
        }
    }

    // Handle layer move down
    if let Some((level_id, layer_index)) = tree_view_result.move_layer_down {
        if let Some(level) = project.get_level_mut(level_id) {
            if level.move_layer_down(layer_index) {
                // Update selection if needed
                if editor_state.selected_layer == Some(layer_index) {
                    editor_state.selected_layer = Some(layer_index + 1);
                    editor_state.selection = Selection::Layer(level_id, layer_index + 1);
                } else if editor_state.selected_layer == Some(layer_index + 1) {
                    editor_state.selected_layer = Some(layer_index);
                    editor_state.selection = Selection::Layer(level_id, layer_index);
                }
                project.mark_dirty();
            }
        }
    }

    // Handle layer visibility toggle
    if let Some((level_id, layer_index)) = tree_view_result.toggle_layer_visibility {
        if let Some(level) = project.get_level_mut(level_id) {
            level.toggle_layer_visibility(layer_index);
            project.mark_dirty();
        }
    }

    // Central area - transparent to allow Bevy rendering to show through
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            render_viewport_overlay(ui, &editor_state);
        });

    // Dialogs
    render_dialogs(ctx, &mut editor_state, &mut project, &assets_base_path);

    // Terrain dialogs
    terrain::render_new_terrain_dialog(ctx, &mut editor_state, &mut project);
    terrain::render_new_terrain_set_dialog(ctx, &mut editor_state, &mut project);
    terrain::render_add_terrain_to_set_dialog(ctx, &mut editor_state, &mut project);

    // Sprite/Animation Editor (modal window)
    if editor_state.show_sprite_editor {
        let anim_result = render_animation_editor(ctx, &mut editor_state.sprite_editor_state);

        // Handle file browser request
        if anim_result.browse_spritesheet {
            if let Some(path) = open_spritesheet_dialog() {
                // Convert absolute path to asset path if possible
                let asset_path = convert_to_asset_path(&path);
                editor_state.sprite_editor_state.sheet_path_input = asset_path.clone();
                editor_state.sprite_editor_state.sprite_data.sheet_path = asset_path;
                editor_state.sprite_editor_state.clear_texture();
            }
        }

        // Handle texture reload request
        if anim_result.reload_spritesheet {
            editor_state.sprite_editor_state.clear_texture();
        }

        if anim_result.changed {
            // Save sprite data back to the instance
            if let Some(instance_id) = editor_state.sprite_editor_state.instance_id {
                if let Some(instance) = project.get_data_instance_mut(instance_id) {
                    let prop_name = editor_state.sprite_editor_state.property_name.clone();
                    let sprite_data = editor_state.sprite_editor_state.get_sprite_data();
                    instance.properties.insert(prop_name, sprite_data.to_value());
                }
            }
        }

        if anim_result.close {
            editor_state.show_sprite_editor = false;
        }
    }

    // Dialogue Editor (modal window)
    if editor_state.show_dialogue_editor {
        let dialogue_result = render_dialogue_editor(ctx, &mut editor_state.dialogue_editor_state);

        if dialogue_result.changed {
            // Save dialogue data back to the instance
            if let Some(instance_id) = editor_state.dialogue_editor_state.instance_id {
                if let Some(instance) = project.get_data_instance_mut(instance_id) {
                    let prop_name = editor_state.dialogue_editor_state.property_name.clone();
                    let dialogue_data = editor_state.dialogue_editor_state.get_dialogue_tree();
                    instance.properties.insert(prop_name, dialogue_data.to_value());
                }
            }
        }

        if dialogue_result.close {
            editor_state.show_dialogue_editor = false;
        }
    }

    // Tileset & Terrain Editor (modal window)
    render_tileset_editor(ctx, &mut editor_state, &mut project, Some(&tileset_cache));
}

/// Render viewport overlay with selection info
fn render_viewport_overlay(ui: &mut egui::Ui, editor_state: &EditorState) {
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        ui.horizontal(|ui| {
            ui.label(format!("Tool: {:?}", editor_state.current_tool));
            if let Some(layer) = editor_state.selected_layer {
                ui.separator();
                ui.label(format!("Layer: {}", layer));
            }
            ui.separator();
            ui.label(format!("Zoom: {}%", (editor_state.zoom * 100.0) as i32));
        });
    });
}

/// Convert an absolute file path to an asset path relative to the assets folder
fn convert_to_asset_path(absolute_path: &str) -> String {
    // Try to find "assets" in the path and take everything after it
    let path = absolute_path.replace('\\', "/");

    if let Some(idx) = path.to_lowercase().find("/assets/") {
        return path[idx + 8..].to_string();
    }

    // If no assets folder found, just use the filename
    if let Some(idx) = path.rfind('/') {
        return path[idx + 1..].to_string();
    }

    path
}
