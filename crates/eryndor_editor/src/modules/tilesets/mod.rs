//! Tilesets Editor Module
//!
//! A comprehensive Tileset Editor similar to Tiled Map Editor.
//! Allows users to:
//! - Create and configure tilesets (spritesheets + individual images)
//! - Edit Terrain Sets with visual terrain assignment to tiles
//! - Edit per-tile collision shapes (rectangle, polygon, ellipse, point)

use bevy_egui::egui;
use crate::editor_state::{
    EditorState, TilesetEditMode, TileCategory, TilesetDefinition,
};
use std::path::PathBuf;

mod tileset_list;
mod tileset_viewer;
mod tile_properties;

/// Render the tilesets editor module with three-column layout
pub fn render(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    // Left panel: Tileset list and tileset properties
    egui::SidePanel::left("tilesets_list_panel")
        .default_width(200.0)
        .min_width(150.0)
        .show_inside(ui, |ui| {
            tileset_list::render(ui, editor_state);
        });

    // Right panel: Tile properties (only show when a tile is selected)
    if editor_state.tilesets.selected_tile.is_some() {
        egui::SidePanel::right("tile_properties_panel")
            .default_width(220.0)
            .min_width(180.0)
            .show_inside(ui, |ui| {
                tile_properties::render(ui, editor_state);
            });
    }

    // Center panel: Tileset viewer with toolbar
    egui::CentralPanel::default().show_inside(ui, |ui| {
        // Toolbar at top
        render_toolbar(ui, editor_state);

        ui.separator();

        // Tileset grid viewer
        tileset_viewer::render(ui, editor_state);
    });

    // Create new tileset dialog
    render_create_dialog(ui.ctx(), editor_state);

    // Asset browser dialog (WASM only)
    render_asset_browser(ui.ctx(), editor_state);

    // Import dialog (WASM only)
    render_import_dialog(ui.ctx(), editor_state);
}

/// Render the toolbar with edit mode buttons and view options
fn render_toolbar(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    ui.horizontal(|ui| {
        // File operations - platform-specific implementations
        #[cfg(not(target_family = "wasm"))]
        {
            if ui.button("Save").on_hover_text("Save tilesets to JSON file").clicked() {
                handle_save_tilesets(editor_state);
            }
            if ui.button("Load").on_hover_text("Load tilesets from JSON file").clicked() {
                handle_load_tilesets(editor_state);
            }
            ui.separator();
        }

        // WASM: Save via download, Load via import dialog
        #[cfg(target_family = "wasm")]
        {
            if ui.button("Save").on_hover_text("Download tilesets as JSON file").clicked() {
                handle_save_tilesets_wasm(editor_state);
            }
            if ui.button("Load").on_hover_text("Import tilesets from JSON").clicked() {
                editor_state.tilesets.show_import_dialog = true;
            }
            ui.separator();
        }

        // Edit mode buttons
        ui.label("Mode:");
        for mode in TilesetEditMode::all() {
            let is_selected = editor_state.tilesets.edit_mode == *mode;
            if ui.selectable_label(is_selected, mode.label()).clicked() {
                editor_state.tilesets.edit_mode = mode.clone();
            }
        }

        ui.separator();

        // Zoom controls
        ui.label("Zoom:");
        if ui.button("-").clicked() {
            editor_state.tilesets.zoom = (editor_state.tilesets.zoom - 0.5).max(0.5);
        }
        ui.label(format!("{:.0}%", editor_state.tilesets.zoom * 100.0));
        if ui.button("+").clicked() {
            editor_state.tilesets.zoom = (editor_state.tilesets.zoom + 0.5).min(8.0);
        }

        ui.separator();

        // View toggles
        ui.checkbox(&mut editor_state.tilesets.show_grid, "Grid");
        ui.checkbox(&mut editor_state.tilesets.show_terrain_overlay, "Terrain");
        ui.checkbox(&mut editor_state.tilesets.show_collision_shapes, "Collision");
    });
}

/// Render the create new tileset dialog
fn render_create_dialog(ctx: &egui::Context, editor_state: &mut EditorState) {
    if !editor_state.tilesets.show_create_dialog {
        return;
    }

    egui::Window::new("Create New Tileset")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(350.0);

            // Name
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut editor_state.tilesets.new_tileset_name);
            });

            // Image path
            ui.horizontal(|ui| {
                ui.label("Image:");
                ui.add(egui::TextEdit::singleline(&mut editor_state.tilesets.new_tileset_image_path)
                    .desired_width(200.0));

                // Asset browser works on all platforms
                if ui.button("Browse...").clicked() {
                    editor_state.tilesets.show_asset_browser = true;
                }

                // Native-only: also allow browsing filesystem
                #[cfg(not(target_family = "wasm"))]
                if ui.button("File...").on_hover_text("Browse filesystem").clicked() {
                    if let Some(path) = open_image_file_dialog() {
                        editor_state.tilesets.new_tileset_image_path = path;
                    }
                }
            });

            // Help text for paths
            ui.small("Path relative to assets folder (e.g., tilesets/grass.png)");

            ui.separator();

            // Tile configuration
            ui.heading("Tile Configuration");

            ui.horizontal(|ui| {
                ui.label("Tile Width:");
                ui.add(egui::DragValue::new(&mut editor_state.tilesets.new_tileset_tile_width)
                    .range(1..=256)
                    .suffix(" px"));
            });

            ui.horizontal(|ui| {
                ui.label("Tile Height:");
                ui.add(egui::DragValue::new(&mut editor_state.tilesets.new_tileset_tile_height)
                    .range(1..=256)
                    .suffix(" px"));
            });

            ui.horizontal(|ui| {
                ui.label("Margin:");
                ui.add(egui::DragValue::new(&mut editor_state.tilesets.new_tileset_margin)
                    .range(0..=32)
                    .suffix(" px"));
                ui.label("(edge padding)");
            });

            ui.horizontal(|ui| {
                ui.label("Spacing:");
                ui.add(egui::DragValue::new(&mut editor_state.tilesets.new_tileset_spacing)
                    .range(0..=32)
                    .suffix(" px"));
                ui.label("(between tiles)");
            });

            ui.separator();

            // Action buttons
            ui.horizontal(|ui| {
                let can_create = !editor_state.tilesets.new_tileset_name.is_empty()
                    && !editor_state.tilesets.new_tileset_image_path.is_empty();

                if ui.add_enabled(can_create, egui::Button::new("Create")).clicked() {
                    // Create the tileset
                    create_tileset_from_form(editor_state);
                    editor_state.tilesets.show_create_dialog = false;
                }

                if ui.button("Cancel").clicked() {
                    editor_state.tilesets.show_create_dialog = false;
                    clear_create_form(editor_state);
                }
            });
        });
}

/// Create a new tileset from the form data
fn create_tileset_from_form(editor_state: &mut EditorState) {
    use crate::editor_state::{TilesetDefinition, TileSource};
    use std::collections::HashMap;

    let id = editor_state.tilesets.new_tileset_name
        .to_lowercase()
        .replace(' ', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>();

    let new_tileset = TilesetDefinition {
        id: id.clone(),
        name: editor_state.tilesets.new_tileset_name.clone(),
        category: TileCategory::Ground,
        display_tile_size: editor_state.tilesets.new_tileset_tile_width,
        sources: vec![
            TileSource::Spritesheet {
                path: editor_state.tilesets.new_tileset_image_path.clone(),
                tile_width: editor_state.tilesets.new_tileset_tile_width,
                tile_height: editor_state.tilesets.new_tileset_tile_height,
                margin: editor_state.tilesets.new_tileset_margin,
                spacing: editor_state.tilesets.new_tileset_spacing,
                image_width: 0, // Will be populated when loaded
                image_height: 0,
                columns: 0,
                rows: 0,
                first_tile_index: 0,
            }
        ],
        total_tiles: 0,
        tile_metadata: HashMap::new(),
        terrain_sets: Vec::new(),
    };

    // Add to tile palette state
    editor_state.world.tile_palette.tilesets.push(new_tileset);

    // Select the new tileset
    let new_index = editor_state.world.tile_palette.tilesets.len() - 1;
    editor_state.tilesets.selected_tileset = Some(new_index);

    // Trigger texture loading for the new tileset
    // Clear both handles AND egui IDs so they stay in sync
    // (egui IDs point to handles, so clearing one without the other causes stale references)
    editor_state.world.tile_palette.tileset_texture_handles.clear();
    editor_state.world.tile_palette.tileset_egui_ids.clear();
    editor_state.world.tile_palette.tileset_textures_loading = true;

    // Clear the form
    clear_create_form(editor_state);

    editor_state.status_message = format!("Created tileset '{}'", id);
}

/// Clear the create tileset form
fn clear_create_form(editor_state: &mut EditorState) {
    editor_state.tilesets.new_tileset_name.clear();
    editor_state.tilesets.new_tileset_image_path.clear();
    editor_state.tilesets.new_tileset_tile_width = 16;
    editor_state.tilesets.new_tileset_tile_height = 16;
    editor_state.tilesets.new_tileset_margin = 0;
    editor_state.tilesets.new_tileset_spacing = 0;
}

// === File Dialog Functions (Native Only) ===

/// Open a file dialog to select an image file (native only)
#[cfg(not(target_family = "wasm"))]
fn open_image_file_dialog() -> Option<String> {
    use rfd::FileDialog;

    let file = FileDialog::new()
        .add_filter("Image files", &["png", "jpg", "jpeg", "bmp", "gif", "webp"])
        .add_filter("All files", &["*"])
        .set_title("Select Tileset Image")
        .pick_file()?;

    Some(file.to_string_lossy().to_string())
}

/// Open a file dialog to select a JSON file for loading (native only)
#[cfg(not(target_family = "wasm"))]
fn open_json_file_dialog() -> Option<PathBuf> {
    use rfd::FileDialog;

    FileDialog::new()
        .add_filter("JSON files", &["json"])
        .add_filter("All files", &["*"])
        .set_title("Load Tilesets")
        .pick_file()
}

/// Open a file dialog to save a JSON file (native only)
#[cfg(not(target_family = "wasm"))]
fn save_json_file_dialog() -> Option<PathBuf> {
    use rfd::FileDialog;

    FileDialog::new()
        .add_filter("JSON files", &["json"])
        .set_title("Save Tilesets")
        .set_file_name("tilesets.json")
        .save_file()
}

// === Save/Load Functions ===

/// Save all tilesets to a JSON file
pub fn save_tilesets_to_file(editor_state: &EditorState, path: &std::path::Path) -> Result<(), String> {
    let tilesets = &editor_state.world.tile_palette.tilesets;

    let json = serde_json::to_string_pretty(tilesets)
        .map_err(|e| format!("Failed to serialize tilesets: {}", e))?;

    std::fs::write(path, json)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

/// Load tilesets from a JSON file
pub fn load_tilesets_from_file(path: &std::path::Path) -> Result<Vec<TilesetDefinition>, String> {
    let json = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let tilesets: Vec<TilesetDefinition> = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse tilesets: {}", e))?;

    Ok(tilesets)
}

/// Handle save tilesets action
#[cfg(not(target_family = "wasm"))]
pub fn handle_save_tilesets(editor_state: &mut EditorState) {
    if let Some(path) = save_json_file_dialog() {
        match save_tilesets_to_file(editor_state, &path) {
            Ok(()) => {
                editor_state.status_message = format!("Saved tilesets to {}", path.display());
            }
            Err(e) => {
                editor_state.error_popup = Some(e);
            }
        }
    }
}

/// Handle load tilesets action
#[cfg(not(target_family = "wasm"))]
pub fn handle_load_tilesets(editor_state: &mut EditorState) {
    if let Some(path) = open_json_file_dialog() {
        match load_tilesets_from_file(&path) {
            Ok(tilesets) => {
                editor_state.world.tile_palette.tilesets = tilesets;
                editor_state.tilesets.selected_tileset = None;
                editor_state.tilesets.selected_tile = None;
                editor_state.status_message = format!("Loaded tilesets from {}", path.display());
            }
            Err(e) => {
                editor_state.error_popup = Some(e);
            }
        }
    }
}

// === Asset Browser ===

/// Embedded asset manifest (generated at build time)
/// This contains a JSON tree of all PNG files in the tiles directory
const ASSET_MANIFEST: &str = include_str!("../../../../../assets/tile_assets.json");

/// Render the asset browser dialog
fn render_asset_browser(ctx: &egui::Context, editor_state: &mut EditorState) {
    if !editor_state.tilesets.show_asset_browser {
        return;
    }

    // Load manifest if not already loaded
    if editor_state.tilesets.asset_browser_manifest.is_none() {
        if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(ASSET_MANIFEST) {
            editor_state.tilesets.asset_browser_manifest = Some(manifest);
        }
    }

    egui::Window::new("Browse Assets")
        .collapsible(false)
        .resizable(true)
        .default_width(450.0)
        .default_height(400.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("Select Tileset Image");

            ui.separator();

            // Search box
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.add(egui::TextEdit::singleline(&mut editor_state.tilesets.asset_browser_search)
                    .desired_width(200.0)
                    .hint_text("Filter files..."));
                if ui.button("Clear").clicked() {
                    editor_state.tilesets.asset_browser_search.clear();
                }
            });

            ui.separator();

            // Selected path display
            ui.horizontal(|ui| {
                ui.label("Selected:");
                ui.add(egui::TextEdit::singleline(&mut editor_state.tilesets.new_tileset_image_path)
                    .desired_width(300.0)
                    .hint_text("Click a file below to select"));
            });

            ui.separator();

            // Tree view
            egui::ScrollArea::vertical()
                .max_height(250.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if let Some(manifest) = editor_state.tilesets.asset_browser_manifest.clone() {
                        let search = editor_state.tilesets.asset_browser_search.to_lowercase();
                        render_asset_tree(ui, editor_state, &manifest, "", &search);
                    } else {
                        ui.label("Failed to load asset manifest");
                    }
                });

            ui.separator();

            ui.horizontal(|ui| {
                let can_select = !editor_state.tilesets.new_tileset_image_path.is_empty();
                if ui.add_enabled(can_select, egui::Button::new("Select")).clicked() {
                    editor_state.tilesets.show_asset_browser = false;
                }

                if ui.button("Cancel").clicked() {
                    editor_state.tilesets.show_asset_browser = false;
                    editor_state.tilesets.new_tileset_image_path.clear();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Expand All").clicked() {
                        expand_all_folders(editor_state, &editor_state.tilesets.asset_browser_manifest.clone());
                    }
                    if ui.button("Collapse All").clicked() {
                        editor_state.tilesets.asset_browser_expanded.clear();
                    }
                });
            });
        });
}

/// Recursively render the asset tree
fn render_asset_tree(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    node: &serde_json::Value,
    path: &str,
    search: &str,
) {
    if let Some(obj) = node.as_object() {
        // Collect and sort keys
        let mut keys: Vec<_> = obj.keys().collect();
        keys.sort();

        for key in keys {
            let value = &obj[key];
            let full_path = if path.is_empty() {
                key.clone()
            } else {
                format!("{}/{}", path, key)
            };

            if value.is_string() {
                // This is a file
                let file_path = value.as_str().unwrap();

                // Apply search filter
                if !search.is_empty() && !file_path.to_lowercase().contains(search) {
                    continue;
                }

                let is_selected = editor_state.tilesets.new_tileset_image_path == file_path;

                ui.horizontal(|ui| {
                    ui.add_space(16.0); // Indent for files
                    if ui.selectable_label(is_selected, format!("  {}", key)).clicked() {
                        editor_state.tilesets.new_tileset_image_path = file_path.to_string();
                    }
                });
            } else if value.is_object() {
                // This is a folder
                // Check if any children match the search
                let has_matching_children = search.is_empty() || folder_has_matches(value, search);

                if !has_matching_children {
                    continue;
                }

                let is_expanded = editor_state.tilesets.asset_browser_expanded.contains(&full_path);

                // Auto-expand if searching
                let should_show = is_expanded || !search.is_empty();

                let header = egui::CollapsingHeader::new(key)
                    .id_salt(&full_path)
                    .default_open(should_show)
                    .show(ui, |ui| {
                        render_asset_tree(ui, editor_state, value, &full_path, search);
                    });

                // Track expanded state
                if header.fully_open() {
                    editor_state.tilesets.asset_browser_expanded.insert(full_path.clone());
                } else if !search.is_empty() {
                    // Don't collapse during search
                } else {
                    editor_state.tilesets.asset_browser_expanded.remove(&full_path);
                }
            }
        }
    }
}

/// Check if a folder contains any files matching the search
fn folder_has_matches(node: &serde_json::Value, search: &str) -> bool {
    if let Some(obj) = node.as_object() {
        for (_, value) in obj {
            if value.is_string() {
                if let Some(path) = value.as_str() {
                    if path.to_lowercase().contains(search) {
                        return true;
                    }
                }
            } else if value.is_object() {
                if folder_has_matches(value, search) {
                    return true;
                }
            }
        }
    }
    false
}

/// Expand all folders in the tree
fn expand_all_folders(editor_state: &mut EditorState, manifest: &Option<serde_json::Value>) {
    if let Some(node) = manifest {
        collect_folder_paths(node, "", &mut editor_state.tilesets.asset_browser_expanded);
    }
}

/// Recursively collect all folder paths
fn collect_folder_paths(node: &serde_json::Value, path: &str, expanded: &mut std::collections::HashSet<String>) {
    if let Some(obj) = node.as_object() {
        for (key, value) in obj {
            if value.is_object() {
                let full_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}/{}", path, key)
                };
                expanded.insert(full_path.clone());
                collect_folder_paths(value, &full_path, expanded);
            }
        }
    }
}

// === WASM Save/Load Functions ===

/// Handle save tilesets action for WASM (triggers browser download)
#[cfg(target_family = "wasm")]
pub fn handle_save_tilesets_wasm(editor_state: &mut EditorState) {
    let tilesets = &editor_state.world.tile_palette.tilesets;

    match serde_json::to_string_pretty(tilesets) {
        Ok(json) => {
            // Use web_sys to trigger a browser download
            if let Err(e) = trigger_browser_download("tilesets.json", &json) {
                editor_state.error_popup = Some(format!("Failed to download: {:?}", e));
            } else {
                editor_state.status_message = "Downloading tilesets.json...".to_string();
            }
        }
        Err(e) => {
            editor_state.error_popup = Some(format!("Failed to serialize tilesets: {}", e));
        }
    }
}

/// Trigger a browser file download via JavaScript
#[cfg(target_family = "wasm")]
fn trigger_browser_download(filename: &str, content: &str) -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    // Create a Blob from the JSON content
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&wasm_bindgen::JsValue::from_str(content));

    let mut blob_options = BlobPropertyBag::new();
    blob_options.type_("application/json");

    let blob = Blob::new_with_str_sequence_and_options(&blob_parts, &blob_options)?;

    // Create a download URL
    let url = Url::create_object_url_with_blob(&blob)?;

    // Create an anchor element and trigger download
    let anchor: HtmlAnchorElement = document
        .create_element("a")?
        .dyn_into()?;
    anchor.set_href(&url);
    anchor.set_download(filename);

    // Add to document, click, and remove
    document.body().ok_or("No body")?.append_child(&anchor)?;
    anchor.click();
    document.body().ok_or("No body")?.remove_child(&anchor)?;

    // Clean up the object URL
    Url::revoke_object_url(&url)?;

    Ok(())
}

/// Render the import tileset dialog
fn render_import_dialog(ctx: &egui::Context, editor_state: &mut EditorState) {
    if !editor_state.tilesets.show_import_dialog {
        return;
    }

    let mut should_close = false;
    let mut should_import = false;

    egui::Window::new("Import Tilesets")
        .collapsible(false)
        .resizable(true)
        .default_width(500.0)
        .default_height(400.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.heading("Paste Tileset JSON");

            ui.label("Open your saved tilesets.json file in a text editor, copy all the content, and paste it below:");

            ui.add_space(10.0);

            // Large text area for JSON input
            egui::ScrollArea::vertical()
                .max_height(250.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut editor_state.tilesets.import_json_text)
                            .desired_width(f32::INFINITY)
                            .desired_rows(15)
                            .font(egui::TextStyle::Monospace)
                            .hint_text("Paste JSON here...")
                    );
                });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);

            // Preview info
            if !editor_state.tilesets.import_json_text.is_empty() {
                match serde_json::from_str::<Vec<TilesetDefinition>>(&editor_state.tilesets.import_json_text) {
                    Ok(tilesets) => {
                        ui.colored_label(egui::Color32::GREEN, format!("Valid JSON: {} tileset(s) found", tilesets.len()));
                    }
                    Err(e) => {
                        ui.colored_label(egui::Color32::RED, format!("Invalid JSON: {}", e));
                    }
                }
            }

            ui.add_space(10.0);

            // Action buttons
            ui.horizontal(|ui| {
                let can_import = !editor_state.tilesets.import_json_text.is_empty()
                    && serde_json::from_str::<Vec<TilesetDefinition>>(&editor_state.tilesets.import_json_text).is_ok();

                if ui.add_enabled(can_import, egui::Button::new("Import")).clicked() {
                    should_import = true;
                }

                if ui.button("Cancel").clicked() {
                    should_close = true;
                }
            });
        });

    if should_import {
        // Parse and import the tilesets
        match serde_json::from_str::<Vec<TilesetDefinition>>(&editor_state.tilesets.import_json_text) {
            Ok(tilesets) => {
                let count = tilesets.len();
                editor_state.world.tile_palette.tilesets = tilesets;
                editor_state.tilesets.selected_tileset = None;
                editor_state.tilesets.selected_tile = None;
                // Clear texture caches to trigger reload
                editor_state.world.tile_palette.tileset_texture_handles.clear();
                editor_state.world.tile_palette.tileset_egui_ids.clear();
                editor_state.world.tile_palette.tileset_textures_loading = true;
                editor_state.status_message = format!("Imported {} tileset(s)", count);
                should_close = true;
            }
            Err(e) => {
                editor_state.error_popup = Some(format!("Failed to parse tilesets: {}", e));
            }
        }
    }

    if should_close {
        editor_state.tilesets.show_import_dialog = false;
        editor_state.tilesets.import_json_text.clear();
    }
}
