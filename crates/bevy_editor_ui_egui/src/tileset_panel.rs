use bevy::prelude::*;
use bevy_editor_tilemap::{LoadTilesetEvent, TilesetManager};
use bevy_egui::egui::TextureId;
use bevy_egui::{egui, EguiContexts};

/// Resource to track tileset panel zoom level
#[derive(Resource)]
pub struct TilesetZoom {
    pub zoom: f32,
}

impl Default for TilesetZoom {
    fn default() -> Self {
        Self { zoom: 1.0 }
    }
}

/// Tileset panel system - shows tilesets and allows tile selection
pub fn tileset_panel_ui(
    mut contexts: EguiContexts,
    mut tileset_manager: ResMut<TilesetManager>,
    mut tileset_zoom: ResMut<TilesetZoom>,
    mut load_events: EventWriter<LoadTilesetEvent>,
    mut select_tile_events: EventWriter<SelectTileEvent>,
    mut select_tileset_events: EventWriter<SelectTilesetEvent>,
    images: Res<Assets<Image>>,
) {
    // Pre-register all tileset textures with egui
    let mut egui_textures = std::collections::HashMap::new();
    for (id, tileset_info) in tileset_manager.tilesets.iter() {
        if images.get(&tileset_info.texture_handle).is_some() {
            let egui_tex = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(tileset_info.texture_handle.clone()));
            egui_textures.insert(*id, egui_tex);
        }
    }

    let Some(ctx) = contexts.ctx_mut().ok() else {
        return;
    };

    let (tileset_to_select, tile_to_select) = egui::SidePanel::right("tileset_panel")
        .default_width(300.0)
        .min_width(200.0)
        .show(ctx, |ui| {
            let mut tileset_to_select = None;
            let mut tile_to_select = None;

            ui.heading("Tilesets");
            ui.separator();

            // Load Tileset button
            if ui.button("📁 Load Tileset...").clicked() {
                // Open file dialog to select tileset image
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Image Files", &["png", "jpg", "jpeg", "bmp"])
                    .set_title("Select Tileset Image")
                    .pick_file()
                {
                    // Extract filename for identifier
                    let identifier = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Untitled Tileset")
                        .to_string();

                    // TODO: In a real implementation, load image to detect dimensions
                    // For now, use default 16x16 tiles
                    load_events.write(LoadTilesetEvent {
                        path: path.to_string_lossy().to_string(),
                        identifier,
                        tile_width: 16,
                        tile_height: 16,
                    });
                }
            }

            ui.separator();

            // Show loaded tilesets
            if tileset_manager.tilesets.is_empty() {
                ui.label("No tilesets loaded");
                ui.label("Click 'Load Tileset' to begin");
            } else {
                // Collect tileset IDs first to avoid borrow checker issues
                let tileset_ids: Vec<u32> = tileset_manager.tilesets.keys().copied().collect();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for id in tileset_ids {
                        let Some(tileset_info) = tileset_manager.tilesets.get(&id) else {
                            continue;
                        };
                        let is_selected = tileset_manager.selected_tileset_id == Some(id);

                        // Clone data needed for the closure to avoid borrow issues
                        let identifier = tileset_info.data.identifier.clone();
                        let tile_width = tileset_info.data.tile_width;
                        let tile_height = tileset_info.data.tile_height;
                        let columns = tileset_info.data.columns;
                        let rows = tileset_info.data.rows;
                        let tile_count = tileset_info.tile_count;
                        let texture_handle = tileset_info.texture_handle.clone();

                        ui.push_id(id, |ui| {
                            if ui.selectable_label(is_selected, &identifier).clicked() {
                                tileset_to_select = Some(id);
                            }

                            if is_selected {
                                ui.separator();

                                // Show tileset properties
                                ui.label(format!("Tile Size: {}x{}px", tile_width, tile_height));
                                ui.label(format!("Grid: {}x{} tiles", columns, rows));
                                ui.label(format!("Total Tiles: {}", tile_count));

                                // Show image dimensions if loaded
                                if let Some(image) = images.get(&texture_handle) {
                                    ui.label(format!(
                                        "Image: {}x{}px",
                                        image.width(),
                                        image.height()
                                    ));

                                    // Validation warnings
                                    let image_width = image.width();
                                    let image_height = image.height();
                                    if image_width % tile_width != 0 {
                                        ui.colored_label(
                                            egui::Color32::YELLOW,
                                            "⚠ Width not evenly divisible",
                                        );
                                    }
                                    if image_height % tile_height != 0 {
                                        ui.colored_label(
                                            egui::Color32::YELLOW,
                                            "⚠ Height not evenly divisible",
                                        );
                                    }
                                } else {
                                    ui.colored_label(egui::Color32::GRAY, "Loading texture...");
                                }

                                ui.separator();

                                // Show stamp brush status
                                if tileset_manager.selected_tiles.len() > 1 {
                                    if let Some((stamp_width, stamp_height)) =
                                        tileset_manager.get_selection_dimensions()
                                    {
                                        ui.colored_label(
                                            egui::Color32::LIGHT_BLUE,
                                            format!(
                                                "🖌 Stamp Brush: {}x{} ({} tiles)",
                                                stamp_width,
                                                stamp_height,
                                                tileset_manager.selected_tiles.len()
                                            ),
                                        );
                                    }
                                } else {
                                    ui.label("🖌 Single Tile Brush");
                                }

                                ui.separator();

                                // Zoom controls
                                ui.horizontal(|ui| {
                                    ui.label("Zoom:");
                                    if ui.button("-").clicked() {
                                        tileset_zoom.zoom = (tileset_zoom.zoom - 0.25).max(0.25);
                                    }
                                    ui.label(format!("{:.0}%", tileset_zoom.zoom * 100.0));
                                    if ui.button("+").clicked() {
                                        tileset_zoom.zoom = (tileset_zoom.zoom + 0.25).min(4.0);
                                    }
                                    if ui.button("Reset").clicked() {
                                        tileset_zoom.zoom = 1.0;
                                    }
                                });

                                ui.separator();

                                // Tile grid visualization with scroll area
                                egui::ScrollArea::both().auto_shrink([false, false]).show(
                                    ui,
                                    |ui| {
                                        if let Some(&egui_texture) = egui_textures.get(&id) {
                                            if let Some(selected_tile) = render_tileset_grid(
                                                ui,
                                                &mut tileset_manager,
                                                id,
                                                &images,
                                                egui_texture,
                                                tileset_zoom.zoom,
                                            ) {
                                                tile_to_select = Some(selected_tile);
                                            }
                                        }
                                    },
                                );
                            }
                        });
                    }
                });
            }

            (tileset_to_select, tile_to_select)
        })
        .inner;

    // Send selection events
    if let Some(tileset_id) = tileset_to_select {
        select_tileset_events.write(SelectTilesetEvent { tileset_id });
    }
    if let Some(tile_id) = tile_to_select {
        select_tile_events.write(SelectTileEvent { tile_id });
    }
}

/// Renders the tileset as a clickable grid with actual texture tiles
/// Returns the tile ID if a tile was clicked
fn render_tileset_grid(
    ui: &mut egui::Ui,
    tileset_manager: &mut TilesetManager,
    tileset_id: u32,
    images: &Assets<Image>,
    egui_texture: TextureId,
    zoom: f32,
) -> Option<u32> {
    let Some(tileset_info) = tileset_manager.tilesets.get(&tileset_id) else {
        return None;
    };

    // Check if texture is loaded
    let Some(image) = images.get(&tileset_info.texture_handle) else {
        ui.label("Loading texture...");
        return None;
    };

    ui.label("Click a tile to select:");

    // Get actual texture dimensions
    let _texture_width = image.width() as f32;
    let _texture_height = image.height() as f32;

    let tile_width = tileset_info.data.tile_width as f32;
    let tile_height = tileset_info.data.tile_height as f32;
    let columns = tileset_info.data.columns;
    let rows = tileset_info.data.rows;

    // Apply zoom to tile dimensions
    let display_tile_width = tile_width * zoom;
    let display_tile_height = tile_height * zoom;

    // Calculate full image display size
    let display_width = display_tile_width * columns as f32;
    let display_height = display_tile_height * rows as f32;

    // Allocate space for the full tileset image (enable drag for multi-select)
    let (image_rect, response) = ui.allocate_exact_size(
        egui::vec2(display_width, display_height),
        egui::Sense::click_and_drag(),
    );

    // Draw the full tileset image
    ui.painter().image(
        egui_texture,
        image_rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );

    // Draw grid overlay and handle selection
    let mut clicked_tile = None;
    let mut hovered_tile = None;

    for row in 0..rows {
        for col in 0..columns {
            let tile_id = row * columns + col;

            // Calculate tile rectangle in screen space
            let tile_rect = egui::Rect::from_min_size(
                egui::pos2(
                    image_rect.min.x + col as f32 * display_tile_width,
                    image_rect.min.y + row as f32 * display_tile_height,
                ),
                egui::vec2(display_tile_width, display_tile_height),
            );

            // Check if this tile is hovered
            if tile_rect.contains(response.hover_pos().unwrap_or(egui::pos2(-1000.0, -1000.0))) {
                hovered_tile = Some((tile_id, col, row));
            }

            // Check if tile is in multi-selection
            let is_in_selection = if let (Some((start_col, start_row)), Some((end_col, end_row))) = (
                tileset_manager.selection_start,
                tileset_manager.selection_end,
            ) {
                let min_col = start_col.min(end_col);
                let max_col = start_col.max(end_col);
                let min_row = start_row.min(end_row);
                let max_row = start_row.max(end_row);
                col >= min_col && col <= max_col && row >= min_row && row <= max_row
            } else {
                false
            };

            let is_selected = tileset_manager.selected_tile_id == Some(tile_id);
            let is_hovered = hovered_tile.map(|(id, _, _)| id) == Some(tile_id);

            // Draw grid lines
            let grid_color = egui::Color32::from_rgba_premultiplied(100, 100, 100, 200);
            ui.painter().rect_stroke(
                tile_rect,
                egui::CornerRadius::ZERO,
                (1.0, grid_color),
                egui::epaint::StrokeKind::Outside,
            );

            // Draw selection/hover highlight
            if is_in_selection {
                // Multi-selection highlight (cyan)
                ui.painter().rect_filled(
                    tile_rect,
                    egui::CornerRadius::ZERO,
                    egui::Color32::from_rgba_premultiplied(0, 255, 255, 60),
                );
                ui.painter().rect_stroke(
                    tile_rect,
                    egui::CornerRadius::ZERO,
                    (2.0, egui::Color32::CYAN),
                    egui::epaint::StrokeKind::Outside,
                );
            } else if is_selected {
                ui.painter().rect_stroke(
                    tile_rect,
                    egui::CornerRadius::ZERO,
                    (2.0, egui::Color32::YELLOW),
                    egui::epaint::StrokeKind::Outside,
                );
            } else if is_hovered {
                ui.painter().rect_stroke(
                    tile_rect,
                    egui::CornerRadius::ZERO,
                    (2.0, egui::Color32::LIGHT_BLUE),
                    egui::epaint::StrokeKind::Outside,
                );
            }
        }
    }

    // Track if we just completed a drag to prevent clearing selection
    let just_finished_drag = response.drag_stopped();

    // Handle drag selection for multi-tile stamps
    if response.drag_started() {
        if let Some((_, col, row)) = hovered_tile {
            // Start drag selection
            tileset_manager.selection_start = Some((col, row));
            tileset_manager.selection_end = Some((col, row));
        }
    } else if response.dragged() {
        if let Some((_, col, row)) = hovered_tile {
            // Update drag selection end
            tileset_manager.selection_end = Some((col, row));
        }
    } else if just_finished_drag {
        // Finalize multi-selection - populate selected_tiles
        if let (Some((start_col, start_row)), Some((end_col, end_row))) = (
            tileset_manager.selection_start,
            tileset_manager.selection_end,
        ) {
            let min_col = start_col.min(end_col);
            let max_col = start_col.max(end_col);
            let min_row = start_row.min(end_row);
            let max_row = start_row.max(end_row);

            tileset_manager.selected_tiles.clear();
            for row in min_row..=max_row {
                for col in min_col..=max_col {
                    let tile_id = row * columns + col;
                    tileset_manager.selected_tiles.push(tile_id);
                }
            }
            let tile_count = tileset_manager.selected_tiles.len();
            info!("Selected {} tile(s) for stamp brush", tile_count);
        }
    }

    // Handle single click (clear multi-selection and select single tile)
    // IMPORTANT: Don't process clicks if we just finished dragging!
    if response.clicked() && !just_finished_drag {
        if let Some((tile_id, _, _)) = hovered_tile {
            // Clear stamp selection and return to single-tile mode
            tileset_manager.clear_stamp_selection();
            // Single tile selection
            clicked_tile = Some(tile_id);
            info!("Selected single tile {}, cleared stamp selection", tile_id);
        }
    }

    // Show tooltip
    if let Some((tile_id, col, row)) = hovered_tile {
        let tooltip_text = if tileset_manager.selected_tiles.len() > 1 {
            format!(
                "Tile ID: {}\nGrid: ({}, {})\nStamp: {} tiles selected",
                tile_id,
                col,
                row,
                tileset_manager.selected_tiles.len()
            )
        } else {
            format!(
                "Tile ID: {}\nGrid: ({}, {})\nDrag to select multiple",
                tile_id, col, row
            )
        };
        response.on_hover_text(tooltip_text);
    }

    clicked_tile
}

/// Event to select a specific tile in a tileset
#[derive(Event, Message)]
pub struct SelectTileEvent {
    pub tile_id: u32,
}

/// Event to select a tileset
#[derive(Event, Message)]
pub struct SelectTilesetEvent {
    pub tileset_id: u32,
}

/// System to handle tile selection events
pub fn handle_tile_selection_events(
    mut tileset_manager: ResMut<TilesetManager>,
    mut tile_events: EventReader<SelectTileEvent>,
    mut tileset_events: EventReader<SelectTilesetEvent>,
) {
    for event in tileset_events.read() {
        tileset_manager.select_tileset(event.tileset_id);
    }

    for event in tile_events.read() {
        tileset_manager.select_tile(event.tile_id);
    }
}
