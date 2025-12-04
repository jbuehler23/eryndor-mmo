//! Tileset & Terrain Editor Window
//!
//! A dedicated window for managing tilesets and terrain sets with Tiled-style UI.
//! Accessible from Tools > Tileset & Terrain Editor

use bevy::prelude::Color;
use bevy_egui::egui;

use crate::autotile::{TerrainSet, TerrainSetType, TileTerrainData};
use crate::project::Project;
use crate::EditorState;
use super::TilesetTextureCache;

/// State for the tileset editor window
pub struct TilesetEditorState {
    /// Selected tileset in the editor
    pub selected_tileset: Option<uuid::Uuid>,

    /// Selected terrain set
    pub selected_terrain_set: Option<uuid::Uuid>,

    /// Selected terrain within the set (for painting)
    pub selected_terrain_idx: Option<usize>,

    // New tileset dialog
    pub new_tileset_name: String,
    pub new_tileset_tile_size: u32,

    // New terrain set dialog
    pub show_new_terrain_set_dialog: bool,
    pub new_terrain_set_name: String,
    pub new_terrain_set_type: TerrainSetType,

    // Add terrain dialog
    pub show_add_terrain_dialog: bool,
    pub new_terrain_name: String,
    pub new_terrain_color: [f32; 3],

    /// Tile display zoom (pixels per tile)
    pub tile_zoom: f32,

    /// Hovered tile index (for highlighting)
    pub hovered_tile: Option<u32>,

    /// Hovered corner/edge position index
    pub hovered_position: Option<usize>,

    /// Whether currently painting markers (mouse held down)
    pub is_painting_markers: bool,

    /// Last painted marker to avoid re-painting same position
    pub last_painted_marker: Option<(u32, usize)>, // (tile_index, position)
}

impl Default for TilesetEditorState {
    fn default() -> Self {
        Self {
            selected_tileset: None,
            selected_terrain_set: None,
            selected_terrain_idx: None,
            new_tileset_name: String::new(),
            new_tileset_tile_size: 32,
            show_new_terrain_set_dialog: false,
            new_terrain_set_name: String::new(),
            new_terrain_set_type: TerrainSetType::Corner,
            show_add_terrain_dialog: false,
            new_terrain_name: String::new(),
            new_terrain_color: [0.0, 1.0, 0.0],
            tile_zoom: 64.0,
            hovered_tile: None,
            hovered_position: None,
            is_painting_markers: false,
            last_painted_marker: None,
        }
    }
}

/// Render the tileset editor window with Tiled-style layout
pub fn render_tileset_editor(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    project: &mut Project,
    tileset_cache: Option<&TilesetTextureCache>,
) {
    if !editor_state.show_tileset_editor {
        return;
    }

    let mut open = true;

    egui::Window::new("Tileset & Terrain Editor")
        .open(&mut open)
        .resizable(true)
        .min_width(600.0)
        .min_height(400.0)
        .default_size([900.0, 650.0])
        .show(ctx, |ui| {
            // Top bar: tileset selector and import buttons
            render_top_bar(ui, editor_state, project);

            ui.separator();

            // Bottom: zoom slider and instructions (rendered first to claim space)
            egui::TopBottomPanel::bottom("tileset_editor_bottom")
                .resizable(false)
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Zoom:");
                        ui.add(egui::Slider::new(&mut editor_state.tileset_editor_state.tile_zoom, 32.0..=128.0).suffix("px"));

                        ui.separator();
                        if editor_state.tileset_editor_state.selected_terrain_idx.is_some() {
                            ui.label("Left-click corners/edges to assign terrain. Right-click to clear.");
                        } else {
                            ui.label("Select a terrain from the left sidebar to start marking tiles.");
                        }
                    });
                });

            // Left sidebar panel
            egui::SidePanel::left("tileset_editor_sidebar")
                .resizable(true)
                .default_width(200.0)
                .min_width(150.0)
                .max_width(300.0)
                .show_inside(ui, |ui| {
                    render_terrain_sidebar(ui, editor_state, project);
                });

            // Main tile grid area fills remaining space
            egui::CentralPanel::default()
                .show_inside(ui, |ui| {
                    egui::ScrollArea::both()
                        .id_salt("tile_grid_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            render_tile_grid(ui, editor_state, project, tileset_cache);
                        });
                });
        });

    if !open {
        editor_state.show_tileset_editor = false;
    }

    // Render dialogs
    render_new_terrain_set_dialog(ctx, editor_state, project);
    render_add_terrain_dialog(ctx, editor_state, project);
}

/// Render the top bar with tileset selector and import buttons
fn render_top_bar(ui: &mut egui::Ui, editor_state: &mut EditorState, project: &mut Project) {
    ui.horizontal(|ui| {
        ui.label("Tileset:");

        // Tileset selector
        let current_name = editor_state.tileset_editor_state.selected_tileset
            .and_then(|id| project.tilesets.iter().find(|t| t.id == id))
            .map(|t| t.name.as_str())
            .unwrap_or("(none)");

        egui::ComboBox::from_id_salt("tileset_editor_selector")
            .selected_text(current_name)
            .show_ui(ui, |ui| {
                for tileset in &project.tilesets {
                    if ui.selectable_value(
                        &mut editor_state.tileset_editor_state.selected_tileset,
                        Some(tileset.id),
                        &tileset.name
                    ).clicked() {
                        // Also update main editor's selected tileset
                        editor_state.selected_tileset = Some(tileset.id);
                        // Clear terrain set selection when changing tileset
                        editor_state.tileset_editor_state.selected_terrain_set = None;
                        editor_state.tileset_editor_state.selected_terrain_idx = None;
                    }
                }
            });

        ui.separator();

        // Import tileset button
        if ui.button("+ Import Tileset").clicked() {
            editor_state.show_new_tileset_dialog = true;
        }

        // Add image to existing tileset
        if editor_state.tileset_editor_state.selected_tileset.is_some() {
            if ui.button("+ Add Image").clicked() {
                editor_state.show_add_tileset_image_dialog = true;
            }
        }
    });
}

/// Render the left sidebar with terrain sets and terrains
fn render_terrain_sidebar(ui: &mut egui::Ui, editor_state: &mut EditorState, project: &mut Project) {
    ui.heading("Terrains");
    ui.separator();

    // Get terrain sets for selected tileset
    let tileset_id = editor_state.tileset_editor_state.selected_tileset;

    // Use available height for the scrollable terrain list
    let available_height = ui.available_height() - 120.0; // Reserve space for buttons below

    egui::ScrollArea::vertical()
        .id_salt("terrain_sidebar_scroll")
        .max_height(available_height.max(100.0))
        .show(ui, |ui| {
            // Filter terrain sets by tileset
            let terrain_sets: Vec<_> = project.autotile_config.terrain_sets.iter()
                .filter(|ts| tileset_id.map_or(true, |id| ts.tileset_id == id))
                .map(|ts| (ts.id, ts.name.clone(), ts.set_type, ts.terrains.len()))
                .collect();

            if terrain_sets.is_empty() {
                ui.label("No terrain sets.");
                ui.label("Create one below.");
            }

            for (set_id, set_name, set_type, terrain_count) in &terrain_sets {
                let type_label = match set_type {
                    TerrainSetType::Corner => "[C]",
                    TerrainSetType::Edge => "[E]",
                    TerrainSetType::Mixed => "[M]",
                };

                let header_selected = editor_state.tileset_editor_state.selected_terrain_set == Some(*set_id);

                egui::CollapsingHeader::new(format!("{} {} ({})", type_label, set_name, terrain_count))
                    .id_salt(set_id)
                    .default_open(header_selected)
                    .show(ui, |ui| {
                        // Get terrains for this set
                        if let Some(terrain_set) = project.autotile_config.get_terrain_set(*set_id) {
                            for (idx, terrain) in terrain_set.terrains.iter().enumerate() {
                                let selected = editor_state.tileset_editor_state.selected_terrain_set == Some(*set_id)
                                    && editor_state.tileset_editor_state.selected_terrain_idx == Some(idx);

                                ui.horizontal(|ui| {
                                    // Color swatch
                                    let srgba = terrain.color.to_srgba();
                                    let color32 = egui::Color32::from_rgba_unmultiplied(
                                        (srgba.red * 255.0) as u8,
                                        (srgba.green * 255.0) as u8,
                                        (srgba.blue * 255.0) as u8,
                                        255,
                                    );
                                    let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                                    ui.painter().rect_filled(rect, 2.0, color32);

                                    // Terrain name (selectable)
                                    if ui.selectable_label(selected, &terrain.name).clicked() {
                                        editor_state.tileset_editor_state.selected_terrain_set = Some(*set_id);
                                        editor_state.tileset_editor_state.selected_terrain_idx = Some(idx);
                                    }
                                });
                            }

                            // Add terrain button under each set
                            if ui.small_button("+ Add Terrain").clicked() {
                                editor_state.tileset_editor_state.selected_terrain_set = Some(*set_id);
                                editor_state.tileset_editor_state.show_add_terrain_dialog = true;
                                editor_state.tileset_editor_state.new_terrain_name = "New Terrain".to_string();
                            }
                        }
                    });
            }
        });

    ui.separator();

    // New terrain set button
    if ui.button("+ New Terrain Set").clicked() {
        editor_state.tileset_editor_state.show_new_terrain_set_dialog = true;
        editor_state.tileset_editor_state.new_terrain_set_name = "New Terrain Set".to_string();
    }

    // Delete terrain set button
    if let Some(set_id) = editor_state.tileset_editor_state.selected_terrain_set {
        if ui.button("Delete Terrain Set").clicked() {
            project.autotile_config.remove_terrain_set(set_id);
            project.mark_dirty();
            editor_state.tileset_editor_state.selected_terrain_set = None;
            editor_state.tileset_editor_state.selected_terrain_idx = None;
        }
    }

    ui.separator();

    // Show selected terrain info
    if let (Some(set_id), Some(terrain_idx)) = (
        editor_state.tileset_editor_state.selected_terrain_set,
        editor_state.tileset_editor_state.selected_terrain_idx
    ) {
        if let Some(terrain_set) = project.autotile_config.get_terrain_set(set_id) {
            if let Some(terrain) = terrain_set.terrains.get(terrain_idx) {
                ui.heading("Selected:");
                ui.horizontal(|ui| {
                    let srgba = terrain.color.to_srgba();
                    let color32 = egui::Color32::from_rgba_unmultiplied(
                        (srgba.red * 255.0) as u8,
                        (srgba.green * 255.0) as u8,
                        (srgba.blue * 255.0) as u8,
                        255,
                    );
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 3.0, color32);
                    ui.label(&terrain.name);
                });

                // Clear selection button
                if ui.small_button("Clear Selection").clicked() {
                    editor_state.tileset_editor_state.selected_terrain_idx = None;
                }
            }
        }
    }
}

/// Terrain assignment action to be applied after rendering
struct TerrainAction {
    tile_index: u32,
    position: usize,
    terrain_idx: Option<usize>,
}

/// Render the tile grid with terrain markers
fn render_tile_grid(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &mut Project,
    tileset_cache: Option<&TilesetTextureCache>,
) {
    let Some(tileset_id) = editor_state.tileset_editor_state.selected_tileset else {
        ui.centered_and_justified(|ui| {
            ui.label("Select a tileset from the dropdown above.");
        });
        return;
    };

    // Get tileset info (tile_size and images)
    let Some(tileset) = project.tilesets.iter().find(|t| t.id == tileset_id) else {
        return;
    };
    let tileset_tile_size = tileset.tile_size;
    let tileset_images = tileset.images.clone();

    if tileset_images.is_empty() {
        return;
    }

    let display_tile_size = editor_state.tileset_editor_state.tile_zoom;
    let spacing = 2.0;

    // Get terrain set info if selected
    let terrain_set_id = editor_state.tileset_editor_state.selected_terrain_set;
    let terrain_set_info = terrain_set_id
        .and_then(|set_id| project.autotile_config.get_terrain_set(set_id))
        .map(|ts| (ts.set_type, ts.terrains.clone(), ts.tile_terrains.clone()));

    let selected_terrain_idx = editor_state.tileset_editor_state.selected_terrain_idx;

    // Check if primary mouse button is held for drag painting
    let pointer = ui.input(|i| i.pointer.clone());
    let is_primary_down = pointer.primary_down();

    // Update painting state
    if is_primary_down {
        editor_state.tileset_editor_state.is_painting_markers = true;
    } else {
        // Mouse released - reset painting state
        editor_state.tileset_editor_state.is_painting_markers = false;
        editor_state.tileset_editor_state.last_painted_marker = None;
    }

    let is_painting = editor_state.tileset_editor_state.is_painting_markers;
    let last_painted = editor_state.tileset_editor_state.last_painted_marker;

    // Collect terrain actions to apply after rendering
    let mut terrain_actions: Vec<TerrainAction> = Vec::new();

    // Render tiles from all images
    let mut virtual_offset = 0u32;

    for image in &tileset_images {
        ui.label(format!("{}:", image.name));

        // Try to get texture info for this image
        let texture_info = tileset_cache
            .and_then(|cache| cache.loaded.get(&image.id))
            .map(|(_, tex_id, width, height)| (*tex_id, *width, *height));

        // Calculate actual columns and rows from texture dimensions
        let (columns, rows) = if let Some((_, tex_width, tex_height)) = texture_info {
            let cols = (tex_width / tileset_tile_size as f32).max(1.0) as u32;
            let rows = (tex_height / tileset_tile_size as f32).max(1.0) as u32;
            (cols, rows)
        } else {
            // Fall back to stored values if texture not loaded
            (image.columns.max(1), image.rows.max(1))
        };

        let texture_id = texture_info.map(|(tex_id, _, _)| tex_id);
        let uv_tile_width = 1.0 / columns as f32;
        let uv_tile_height = 1.0 / rows as f32;

        for row in 0..rows {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);

                for col in 0..columns {
                    let local_index = row * columns + col;
                    let virtual_index = virtual_offset + local_index;

                    // Allocate space for tile
                    let (tile_rect, tile_response) = ui.allocate_exact_size(
                        egui::vec2(display_tile_size, display_tile_size),
                        egui::Sense::hover()
                    );

                    // Draw tile background
                    ui.painter().rect_filled(tile_rect, 0.0, egui::Color32::from_gray(40));

                    // Draw tile texture if available
                    if let Some(tex_id) = texture_id {
                        let uv_min = egui::pos2(
                            col as f32 * uv_tile_width,
                            row as f32 * uv_tile_height,
                        );
                        let uv_max = egui::pos2(
                            (col + 1) as f32 * uv_tile_width,
                            (row + 1) as f32 * uv_tile_height,
                        );

                        ui.painter().image(
                            tex_id,
                            tile_rect,
                            egui::Rect::from_min_max(uv_min, uv_max),
                            egui::Color32::WHITE,
                        );
                    } else {
                        // Show tile index when no texture
                        ui.painter().text(
                            tile_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{}", virtual_index),
                            egui::FontId::default(),
                            egui::Color32::GRAY,
                        );
                    }

                    // Draw terrain markers if a terrain set is selected
                    if let Some((set_type, ref terrains, ref tile_terrains)) = terrain_set_info {
                        if let Some(action) = render_terrain_markers(
                            ui,
                            tile_rect,
                            virtual_index,
                            set_type,
                            terrains,
                            tile_terrains.get(&virtual_index),
                            selected_terrain_idx,
                            is_painting,
                            last_painted,
                        ) {
                            terrain_actions.push(action);
                        }
                    }

                    // Highlight on hover
                    if tile_response.hovered() {
                        ui.painter().rect_stroke(
                            tile_rect,
                            0.0,
                            egui::Stroke::new(2.0, egui::Color32::WHITE),
                            egui::StrokeKind::Inside,
                        );
                    }

                    // Show tooltip
                    tile_response.on_hover_text(format!("Tile {}", virtual_index));
                }
            });
        }

        virtual_offset += columns * rows;
        ui.add_space(8.0);
    }

    // Apply terrain actions after rendering
    if !terrain_actions.is_empty() {
        if let Some(set_id) = terrain_set_id {
            if let Some(terrain_set) = project.autotile_config.get_terrain_set_mut(set_id) {
                for action in &terrain_actions {
                    terrain_set.set_tile_terrain(action.tile_index, action.position, action.terrain_idx);
                }
                project.mark_dirty();

                // Update last painted marker for drag deduplication
                if let Some(last_action) = terrain_actions.last() {
                    editor_state.tileset_editor_state.last_painted_marker = Some((last_action.tile_index, last_action.position));
                }
            }
        }
    }
}

/// Render terrain markers on a tile and handle click/drag interactions.
/// Returns a TerrainAction if the user clicked or dragged to assign/clear terrain.
///
/// `is_painting` - Whether the mouse button is currently held down (for drag painting)
/// `last_painted` - The last painted marker (to avoid re-painting same position while dragging)
fn render_terrain_markers(
    ui: &mut egui::Ui,
    tile_rect: egui::Rect,
    tile_index: u32,
    set_type: TerrainSetType,
    terrains: &[crate::autotile::Terrain],
    tile_terrain_data: Option<&TileTerrainData>,
    selected_terrain_idx: Option<usize>,
    is_painting: bool,
    last_painted: Option<(u32, usize)>,
) -> Option<TerrainAction> {
    let positions = get_marker_positions(tile_rect, set_type);
    let position_names = get_position_names(set_type);
    let mut action = None;

    for (pos_idx, (marker_rect, name)) in positions.iter().zip(position_names.iter()).enumerate() {
        // Get assigned terrain for this position
        let assigned_terrain = tile_terrain_data
            .and_then(|td| td.terrains.get(pos_idx).copied().flatten());

        // Determine marker color
        let marker_color = if let Some(terrain_idx) = assigned_terrain {
            if let Some(terrain) = terrains.get(terrain_idx) {
                let srgba = terrain.color.to_srgba();
                egui::Color32::from_rgba_unmultiplied(
                    (srgba.red * 255.0) as u8,
                    (srgba.green * 255.0) as u8,
                    (srgba.blue * 255.0) as u8,
                    220,
                )
            } else {
                egui::Color32::from_rgba_unmultiplied(100, 100, 100, 100)
            }
        } else {
            egui::Color32::from_rgba_unmultiplied(80, 80, 80, 80)
        };

        // Draw marker
        if assigned_terrain.is_some() {
            ui.painter().rect_filled(*marker_rect, 2.0, marker_color);
        } else {
            ui.painter().rect_stroke(
                *marker_rect,
                2.0,
                egui::Stroke::new(1.0, marker_color),
                egui::StrokeKind::Inside,
            );
        }

        // Handle clicks and drags
        let marker_response = ui.interact(
            *marker_rect,
            egui::Id::new(("terrain_marker", tile_index, pos_idx)),
            egui::Sense::click()
        );

        let is_hovered = marker_response.hovered();

        // Left-click: assign selected terrain
        if marker_response.clicked() {
            if let Some(terrain_idx) = selected_terrain_idx {
                action = Some(TerrainAction {
                    tile_index,
                    position: pos_idx,
                    terrain_idx: Some(terrain_idx),
                });
            }
        }
        // Drag painting: if mouse button held and hovering, paint (but not same marker twice)
        else if is_painting && is_hovered && selected_terrain_idx.is_some() {
            let current_marker = (tile_index, pos_idx);
            if last_painted != Some(current_marker) {
                action = Some(TerrainAction {
                    tile_index,
                    position: pos_idx,
                    terrain_idx: selected_terrain_idx,
                });
            }
        }

        // Right-click: clear terrain
        if marker_response.secondary_clicked() {
            action = Some(TerrainAction {
                tile_index,
                position: pos_idx,
                terrain_idx: None,
            });
        }

        // Highlight on hover (more visible when painting)
        if is_hovered {
            let stroke_width = if is_painting && selected_terrain_idx.is_some() { 3.0 } else { 2.0 };
            let stroke_color = if is_painting && selected_terrain_idx.is_some() {
                egui::Color32::YELLOW
            } else {
                egui::Color32::WHITE
            };
            ui.painter().rect_stroke(
                *marker_rect,
                2.0,
                egui::Stroke::new(stroke_width, stroke_color),
                egui::StrokeKind::Outside,
            );
        }

        // Tooltip
        let tooltip = if let Some(terrain_idx) = assigned_terrain {
            let terrain_name = terrains.get(terrain_idx)
                .map(|t| t.name.as_str())
                .unwrap_or("Unknown");
            format!("{}: {}", name, terrain_name)
        } else {
            format!("{}: (empty)", name)
        };
        marker_response.on_hover_text(tooltip);
    }

    action
}

/// Get marker rectangles for corners/edges based on terrain set type.
/// Marker sizes scale with tile size (20% of tile, clamped 12-24px).
fn get_marker_positions(tile_rect: egui::Rect, set_type: TerrainSetType) -> Vec<egui::Rect> {
    let w = tile_rect.width();
    let h = tile_rect.height();

    // Zoom-responsive marker size: 20% of tile size, clamped between 12-24px
    let base_marker_size = (w * 0.20).clamp(12.0, 24.0);
    let offset = (w * 0.03).max(2.0); // Small offset from edges

    match set_type {
        TerrainSetType::Corner => {
            let marker_size = base_marker_size;
            vec![
                // Top-Left (index 0)
                egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2(offset, offset),
                    egui::vec2(marker_size, marker_size)
                ),
                // Top-Right (index 1)
                egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2(w - marker_size - offset, offset),
                    egui::vec2(marker_size, marker_size)
                ),
                // Bottom-Left (index 2)
                egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2(offset, h - marker_size - offset),
                    egui::vec2(marker_size, marker_size)
                ),
                // Bottom-Right (index 3)
                egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2(w - marker_size - offset, h - marker_size - offset),
                    egui::vec2(marker_size, marker_size)
                ),
            ]
        },
        TerrainSetType::Edge => {
            let edge_length = base_marker_size * 2.0;
            let edge_thickness = base_marker_size * 0.6;
            vec![
                // Top (index 0)
                egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2((w - edge_length) / 2.0, offset),
                    egui::vec2(edge_length, edge_thickness)
                ),
                // Right (index 1)
                egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2(w - edge_thickness - offset, (h - edge_length) / 2.0),
                    egui::vec2(edge_thickness, edge_length)
                ),
                // Bottom (index 2)
                egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2((w - edge_length) / 2.0, h - edge_thickness - offset),
                    egui::vec2(edge_length, edge_thickness)
                ),
                // Left (index 3)
                egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2(offset, (h - edge_length) / 2.0),
                    egui::vec2(edge_thickness, edge_length)
                ),
            ]
        },
        TerrainSetType::Mixed => {
            // For mixed mode, corners are slightly smaller to leave room for edges
            let corner_size = base_marker_size * 0.8;
            let edge_length = base_marker_size * 1.2;
            let edge_thickness = base_marker_size * 0.5;
            // IMPORTANT: Order must match autotile algorithm's clockwise layout:
            // TL(0), Top(1), TR(2), Right(3), BR(4), Bottom(5), BL(6), Left(7)
            vec![
                // Index 0: Top-Left corner
                egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2(offset, offset),
                    egui::vec2(corner_size, corner_size)
                ),
                // Index 1: Top edge
                egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2((w - edge_length) / 2.0, offset),
                    egui::vec2(edge_length, edge_thickness)
                ),
                // Index 2: Top-Right corner
                egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2(w - corner_size - offset, offset),
                    egui::vec2(corner_size, corner_size)
                ),
                // Index 3: Right edge
                egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2(w - edge_thickness - offset, (h - edge_length) / 2.0),
                    egui::vec2(edge_thickness, edge_length)
                ),
                // Index 4: Bottom-Right corner
                egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2(w - corner_size - offset, h - corner_size - offset),
                    egui::vec2(corner_size, corner_size)
                ),
                // Index 5: Bottom edge
                egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2((w - edge_length) / 2.0, h - edge_thickness - offset),
                    egui::vec2(edge_length, edge_thickness)
                ),
                // Index 6: Bottom-Left corner
                egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2(offset, h - corner_size - offset),
                    egui::vec2(corner_size, corner_size)
                ),
                // Index 7: Left edge
                egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2(offset, (h - edge_length) / 2.0),
                    egui::vec2(edge_thickness, edge_length)
                ),
            ]
        },
    }
}

/// Get human-readable names for each position
/// IMPORTANT: Order must match autotile algorithm's index layout
fn get_position_names(set_type: TerrainSetType) -> Vec<&'static str> {
    match set_type {
        TerrainSetType::Corner => vec!["Top-Left", "Top-Right", "Bottom-Left", "Bottom-Right"],
        TerrainSetType::Edge => vec!["Top", "Right", "Bottom", "Left"],
        // Clockwise from TL: TL(0), Top(1), TR(2), Right(3), BR(4), Bottom(5), BL(6), Left(7)
        TerrainSetType::Mixed => vec![
            "Top-Left Corner", "Top Edge", "Top-Right Corner", "Right Edge",
            "Bottom-Right Corner", "Bottom Edge", "Bottom-Left Corner", "Left Edge"
        ],
    }
}

/// Render the new terrain set dialog
fn render_new_terrain_set_dialog(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    project: &mut Project,
) {
    if !editor_state.tileset_editor_state.show_new_terrain_set_dialog {
        return;
    }

    egui::Window::new("New Terrain Set")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut editor_state.tileset_editor_state.new_terrain_set_name);
            });

            ui.horizontal(|ui| {
                ui.label("Tileset:");
                let tileset_name = editor_state.tileset_editor_state.selected_tileset
                    .and_then(|id| project.tilesets.iter().find(|t| t.id == id))
                    .map(|t| t.name.as_str())
                    .unwrap_or("(select tileset first)");
                ui.label(tileset_name);
            });

            ui.horizontal(|ui| {
                ui.label("Type:");
                egui::ComboBox::from_id_salt("new_terrain_set_type")
                    .selected_text(match editor_state.tileset_editor_state.new_terrain_set_type {
                        TerrainSetType::Corner => "Corner (4 corners)",
                        TerrainSetType::Edge => "Edge (4 sides)",
                        TerrainSetType::Mixed => "Mixed (8 positions)",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut editor_state.tileset_editor_state.new_terrain_set_type, TerrainSetType::Corner, "Corner (4 corners)");
                        ui.selectable_value(&mut editor_state.tileset_editor_state.new_terrain_set_type, TerrainSetType::Edge, "Edge (4 sides)");
                        ui.selectable_value(&mut editor_state.tileset_editor_state.new_terrain_set_type, TerrainSetType::Mixed, "Mixed (8 positions)");
                    });
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    editor_state.tileset_editor_state.show_new_terrain_set_dialog = false;
                }

                let can_create = !editor_state.tileset_editor_state.new_terrain_set_name.is_empty()
                    && editor_state.tileset_editor_state.selected_tileset.is_some();

                ui.add_enabled_ui(can_create, |ui| {
                    if ui.button("Create").clicked() {
                        if let Some(tileset_id) = editor_state.tileset_editor_state.selected_tileset {
                            let terrain_set = TerrainSet::new(
                                editor_state.tileset_editor_state.new_terrain_set_name.clone(),
                                tileset_id,
                                editor_state.tileset_editor_state.new_terrain_set_type,
                            );
                            let set_id = terrain_set.id;
                            project.autotile_config.add_terrain_set(terrain_set);
                            project.mark_dirty();
                            editor_state.tileset_editor_state.selected_terrain_set = Some(set_id);
                        }
                        editor_state.tileset_editor_state.show_new_terrain_set_dialog = false;
                    }
                });
            });
        });
}

/// Render the add terrain dialog
fn render_add_terrain_dialog(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    project: &mut Project,
) {
    if !editor_state.tileset_editor_state.show_add_terrain_dialog {
        return;
    }

    egui::Window::new("Add Terrain")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut editor_state.tileset_editor_state.new_terrain_name);
            });

            ui.horizontal(|ui| {
                ui.label("Color:");
                let mut color = [
                    editor_state.tileset_editor_state.new_terrain_color[0],
                    editor_state.tileset_editor_state.new_terrain_color[1],
                    editor_state.tileset_editor_state.new_terrain_color[2],
                ];
                if ui.color_edit_button_rgb(&mut color).changed() {
                    editor_state.tileset_editor_state.new_terrain_color = [color[0], color[1], color[2]];
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    editor_state.tileset_editor_state.show_add_terrain_dialog = false;
                }

                let can_create = !editor_state.tileset_editor_state.new_terrain_name.is_empty()
                    && editor_state.tileset_editor_state.selected_terrain_set.is_some();

                ui.add_enabled_ui(can_create, |ui| {
                    if ui.button("Add").clicked() {
                        if let Some(set_id) = editor_state.tileset_editor_state.selected_terrain_set {
                            if let Some(set) = project.autotile_config.get_terrain_set_mut(set_id) {
                                let color = Color::srgb(
                                    editor_state.tileset_editor_state.new_terrain_color[0],
                                    editor_state.tileset_editor_state.new_terrain_color[1],
                                    editor_state.tileset_editor_state.new_terrain_color[2],
                                );
                                let idx = set.add_terrain(editor_state.tileset_editor_state.new_terrain_name.clone(), color);
                                editor_state.tileset_editor_state.selected_terrain_idx = Some(idx);
                                project.mark_dirty();
                            }
                        }
                        editor_state.tileset_editor_state.show_add_terrain_dialog = false;
                    }
                });
            });
        });
}
