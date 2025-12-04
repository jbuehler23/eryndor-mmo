//! Terrain Palette Panel for painting terrains
//!
//! Similar to Tiled's Terrain Sets panel, this shows:
//! - Available terrain sets with expandable terrain list
//! - Erase terrain option
//! - Selected tileset preview with tiles

use bevy_egui::egui;

use crate::autotile::TerrainSetType;
use crate::project::Project;
use crate::EditorState;
use super::TilesetTextureCache;

/// State for terrain painting
#[derive(Default, Clone)]
pub struct TerrainPaintState {
    /// Currently selected terrain set for painting
    pub selected_terrain_set: Option<uuid::Uuid>,
    /// Currently selected terrain index within the set (None = erase mode)
    pub selected_terrain_idx: Option<usize>,
    /// Erase mode active
    pub erase_mode: bool,
    /// Tab selection: 0 = Terrain Sets, 1 = Tiles
    pub active_tab: usize,
    /// Tile zoom level in the palette
    pub tile_zoom: f32,
}

impl TerrainPaintState {
    pub fn new() -> Self {
        Self {
            selected_terrain_set: None,
            selected_terrain_idx: None,
            erase_mode: false,
            active_tab: 0,
            tile_zoom: 32.0,
        }
    }

    /// Returns true if we're in terrain paint mode (terrain selected or erase mode)
    pub fn is_painting_terrain(&self) -> bool {
        self.erase_mode || (self.selected_terrain_set.is_some() && self.selected_terrain_idx.is_some())
    }

    /// Clear terrain selection
    pub fn clear_selection(&mut self) {
        self.selected_terrain_idx = None;
        self.erase_mode = false;
    }
}

/// Render the terrain palette panel (for the right sidebar)
pub fn render_terrain_palette(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &Project,
    tileset_cache: Option<&TilesetTextureCache>,
) {
    let paint_state = &mut editor_state.terrain_paint_state;

    // Tab bar
    ui.horizontal(|ui| {
        if ui.selectable_label(paint_state.active_tab == 0, "Terrain Sets").clicked() {
            paint_state.active_tab = 0;
        }
        if ui.selectable_label(paint_state.active_tab == 1, "Tiles").clicked() {
            paint_state.active_tab = 1;
        }
    });

    ui.separator();

    match paint_state.active_tab {
        0 => render_terrain_sets_panel(ui, editor_state, project),
        1 => render_tileset_preview_panel(ui, editor_state, project, tileset_cache),
        _ => {}
    }
}

/// Render the terrain sets panel with erase option and terrain tree
fn render_terrain_sets_panel(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &Project,
) {
    let paint_state = &mut editor_state.terrain_paint_state;

    // Erase Terrain option at top
    ui.horizontal(|ui| {
        if ui.selectable_label(paint_state.erase_mode, "🗑 Erase Terrain").clicked() {
            paint_state.erase_mode = !paint_state.erase_mode;
            if paint_state.erase_mode {
                paint_state.selected_terrain_idx = None;
                // Also clear root EditorState fields
                editor_state.selected_terrain_in_set = None;
            }
        }
    });

    ui.separator();

    // Get current tileset for filtering terrain sets
    let tileset_id = editor_state.selected_tileset;

    // Terrain sets tree
    egui::ScrollArea::vertical()
        .id_salt("terrain_palette_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Group terrain sets by tileset
            let terrain_sets: Vec<_> = project.autotile_config.terrain_sets.iter()
                .filter(|ts| tileset_id.map_or(true, |id| ts.tileset_id == id))
                .collect();

            if terrain_sets.is_empty() {
                ui.label("No terrain sets.");
                ui.label("Create one in the Tileset Editor");
                ui.label("(Tools > Tileset & Terrain Editor)");
                return;
            }

            for terrain_set in terrain_sets {
                let set_selected = paint_state.selected_terrain_set == Some(terrain_set.id);

                // Get tileset name for display
                let tileset_name = project.tilesets.iter()
                    .find(|t| t.id == terrain_set.tileset_id)
                    .map(|t| t.name.as_str())
                    .unwrap_or("Unknown");

                let type_label = match terrain_set.set_type {
                    TerrainSetType::Corner => "[C]",
                    TerrainSetType::Edge => "[E]",
                    TerrainSetType::Mixed => "[M]",
                };

                let header_text = format!("{} {} ({})", type_label, terrain_set.name, tileset_name);

                egui::CollapsingHeader::new(header_text)
                    .id_salt(terrain_set.id)
                    .default_open(set_selected)
                    .show(ui, |ui| {
                        // Terrains within this set
                        for (idx, terrain) in terrain_set.terrains.iter().enumerate() {
                            let is_selected = set_selected
                                && paint_state.selected_terrain_idx == Some(idx)
                                && !paint_state.erase_mode;

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

                                // Selectable terrain name
                                if ui.selectable_label(is_selected, &terrain.name).clicked() {
                                    paint_state.selected_terrain_set = Some(terrain_set.id);
                                    paint_state.selected_terrain_idx = Some(idx);
                                    paint_state.erase_mode = false;

                                    // IMPORTANT: Also set root EditorState fields that tools/mod.rs reads
                                    editor_state.selected_terrain_set = Some(terrain_set.id);
                                    editor_state.selected_terrain_in_set = Some(idx);

                                    // Also select the tileset for this terrain set
                                    editor_state.selected_tileset = Some(terrain_set.tileset_id);
                                }
                            });
                        }

                        if terrain_set.terrains.is_empty() {
                            ui.label("(no terrains)");
                        }
                    });
            }
        });

    // Show current selection at bottom
    ui.separator();
    render_current_selection(ui, paint_state, project);
}

/// Render current terrain selection info
fn render_current_selection(
    ui: &mut egui::Ui,
    paint_state: &TerrainPaintState,
    project: &Project,
) {
    ui.label("Selected:");
    ui.horizontal(|ui| {
        if paint_state.erase_mode {
            ui.label("🗑 Erase Mode");
        } else if let (Some(set_id), Some(terrain_idx)) = (paint_state.selected_terrain_set, paint_state.selected_terrain_idx) {
            if let Some(terrain_set) = project.autotile_config.get_terrain_set(set_id) {
                if let Some(terrain) = terrain_set.terrains.get(terrain_idx) {
                    // Show color swatch
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
                }
            }
        } else {
            ui.label("(none)");
        }
    });
}

/// Render the tileset preview panel for tile painting
fn render_tileset_preview_panel(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    project: &Project,
    tileset_cache: Option<&TilesetTextureCache>,
) {
    // Tileset selector
    ui.horizontal(|ui| {
        ui.label("Tileset:");

        let current_name = editor_state.selected_tileset
            .and_then(|id| project.tilesets.iter().find(|t| t.id == id))
            .map(|t| t.name.as_str())
            .unwrap_or("(none)");

        egui::ComboBox::from_id_salt("terrain_palette_tileset")
            .selected_text(current_name)
            .show_ui(ui, |ui| {
                for tileset in &project.tilesets {
                    if ui.selectable_value(
                        &mut editor_state.selected_tileset,
                        Some(tileset.id),
                        &tileset.name
                    ).clicked() {
                        // Selection updated
                    }
                }
            });
    });

    // Zoom control
    ui.horizontal(|ui| {
        ui.label("Zoom:");
        ui.add(egui::Slider::new(&mut editor_state.terrain_paint_state.tile_zoom, 16.0..=64.0).suffix("px"));
    });

    ui.separator();

    // Tileset tiles preview
    let Some(tileset_id) = editor_state.selected_tileset else {
        ui.label("Select a tileset");
        return;
    };

    let Some(tileset) = project.tilesets.iter().find(|t| t.id == tileset_id) else {
        return;
    };

    let tile_size = editor_state.terrain_paint_state.tile_zoom;
    let spacing = 1.0;

    egui::ScrollArea::both()
        .id_salt("tileset_preview_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut virtual_offset = 0u32;

            for image in &tileset.images {
                // Get texture info
                let texture_info = tileset_cache
                    .and_then(|cache| cache.loaded.get(&image.id))
                    .map(|(_, tex_id, width, height)| (*tex_id, *width, *height));

                // Calculate columns and rows from texture dimensions
                let (columns, rows) = if let Some((_, tex_width, tex_height)) = texture_info {
                    let cols = (tex_width / tileset.tile_size as f32).max(1.0) as u32;
                    let rows = (tex_height / tileset.tile_size as f32).max(1.0) as u32;
                    (cols, rows)
                } else {
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

                            let (tile_rect, tile_response) = ui.allocate_exact_size(
                                egui::vec2(tile_size, tile_size),
                                egui::Sense::click()
                            );

                            // Draw tile background
                            ui.painter().rect_filled(tile_rect, 0.0, egui::Color32::from_gray(40));

                            // Draw tile texture
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
                            }

                            // Highlight selected tile
                            if editor_state.selected_tile == Some(virtual_index) {
                                ui.painter().rect_stroke(
                                    tile_rect,
                                    0.0,
                                    egui::Stroke::new(2.0, egui::Color32::YELLOW),
                                    egui::StrokeKind::Inside,
                                );
                            }

                            // Handle tile selection
                            if tile_response.clicked() {
                                editor_state.selected_tile = Some(virtual_index);
                                // Switch to tile brush if clicking tiles
                                editor_state.current_tool = crate::ui::EditorTool::Paint;
                            }

                            // Highlight on hover
                            if tile_response.hovered() {
                                ui.painter().rect_stroke(
                                    tile_rect,
                                    0.0,
                                    egui::Stroke::new(1.0, egui::Color32::WHITE),
                                    egui::StrokeKind::Inside,
                                );
                            }

                            tile_response.on_hover_text(format!("Tile {}", virtual_index));
                        }
                    });
                }

                virtual_offset += columns * rows;
            }
        });
}
