//! World/Zone Editor Module
//! Visual zone/level design with canvas, collision shapes, spawn regions, and tilemap painting.

use bevy_egui::egui;
use crate::editor_state::{EditorState, WorldTool, TileCategory, SelectedEntity, TileLayer, TileOperation, UndoEntry, TerrainMatchMode, TerrainSet, TileSource};

/// Render the world editor module
pub fn render(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    // Render side panels first so they claim their space
    render_left_sidebar(ui, editor_state);
    render_properties_panel(ui, editor_state);
    // Note: Bottom palette panel is rendered at context level via render_bottom_panel()
    // Central panel fills remaining space
    render_canvas(ui, editor_state);
    // Render dialogs on top
    render_create_zone_dialog(ui, editor_state);
}

/// Render the bottom tile palette panel at context level (must be called before CentralPanel)
/// This enables proper resizing of the panel.
pub fn render_bottom_panel(ctx: &egui::Context, editor_state: &mut EditorState) {
    render_bottom_palette_panel_ctx(ctx, editor_state);
}

fn render_left_sidebar(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    egui::SidePanel::left("world_left_panel")
        .default_width(200.0)
        .show_inside(ui, |ui| {
            ui.heading("Zones");

            ui.horizontal(|ui| {
                // New zone button
                if ui.button("+ New Zone").clicked() {
                    editor_state.world.show_create_dialog = true;
                }
                // Refresh button
                if ui.button("Refresh").clicked() {
                    editor_state.action_load_zones = true;
                }
            });

            ui.separator();

            // Zone list
            egui::ScrollArea::vertical().id_salt("zone_list_scroll").show(ui, |ui| {
                if editor_state.world.zone_list.is_empty() {
                    ui.label("No zones loaded");
                    ui.label("Click 'Refresh' to load zones");
                } else {
                    for zone in &editor_state.world.zone_list {
                        let is_selected = editor_state.world.current_zone.as_ref() == Some(&zone.id);
                        if ui.selectable_label(is_selected, &zone.name).clicked() {
                            editor_state.world.current_zone = Some(zone.id.clone());
                            editor_state.status_message = format!("Selected zone: {}", zone.name);
                        }
                    }
                }
            });

            ui.separator();

            // Tools
            ui.heading("Tools");

            // General tools
            ui.label("General:");
            let general_tools = [
                (WorldTool::Select, "Select", "Select and move entities"),
                (WorldTool::Pan, "Pan", "Pan the camera"),
                (WorldTool::PlaceEntity, "Place", "Place entities from palette"),
            ];

            for (tool, label, tooltip) in general_tools {
                let is_selected = editor_state.world.active_tool == tool;
                if ui.selectable_label(is_selected, label).on_hover_text(tooltip).clicked() {
                    editor_state.world.active_tool = tool;
                }
            }

            ui.add_space(4.0);

            // Tile painting tools
            ui.label("Tile Painting:");
            let tile_tools = [
                (WorldTool::PaintGround, "Ground", "Paint ground tiles"),
                (WorldTool::PaintDecoration, "Decor", "Paint decoration tiles"),
                (WorldTool::PaintTileCollision, "Collision", "Paint tile collision"),
                (WorldTool::Erase, "Erase", "Erase tiles"),
                (WorldTool::Fill, "Fill", "Bucket fill (flood fill connected area)"),
            ];

            for (tool, label, tooltip) in tile_tools {
                let is_selected = editor_state.world.active_tool == tool;
                if ui.selectable_label(is_selected, label).on_hover_text(tooltip).clicked() {
                    editor_state.world.active_tool = tool;
                }
            }

            ui.add_space(4.0);

            // Region tools
            ui.label("Regions:");
            let region_tools = [
                (WorldTool::DrawCollision, "Box Coll.", "Draw box collision shapes"),
                (WorldTool::DrawSpawnRegion, "Spawn", "Draw spawn regions"),
            ];

            for (tool, label, tooltip) in region_tools {
                let is_selected = editor_state.world.active_tool == tool;
                if ui.selectable_label(is_selected, label).on_hover_text(tooltip).clicked() {
                    editor_state.world.active_tool = tool;
                }
            }

            ui.separator();

            // Brush settings (shown when tile tools are active)
            let is_tile_tool = matches!(
                editor_state.world.active_tool,
                WorldTool::PaintGround | WorldTool::PaintDecoration | WorldTool::PaintTileCollision | WorldTool::Erase
            );

            if is_tile_tool {
                ui.heading("Brush");
                ui.horizontal(|ui| {
                    ui.label("Size:");
                    ui.add(egui::DragValue::new(&mut editor_state.world.brush_size)
                        .speed(0.1)
                        .range(1..=5));
                });
                ui.separator();
            }

            // Entity Palette
            ui.heading("Entity Palette");
            ui.label("NPCs");
            // TODO: List available NPCs to place

            ui.label("Enemies");
            // TODO: List available enemies to place
        });
}

fn render_canvas(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    // Note: We're already inside a CentralPanel, so don't nest another one.
    // This function renders directly into the passed ui to avoid layout conflicts
    // with the bottom palette panel.

    // Canvas toolbar - row 1: general options
    ui.horizontal(|ui| {
            ui.checkbox(&mut editor_state.world.show_grid, "Grid");
            ui.checkbox(&mut editor_state.world.snap_to_grid, "Snap");

            ui.separator();

            ui.label("Zoom:");
            if ui.button("-").clicked() {
                editor_state.world.zoom = (editor_state.world.zoom - 0.1).max(0.1);
            }
            ui.label(format!("{:.0}%", editor_state.world.zoom * 100.0));
            if ui.button("+").clicked() {
                editor_state.world.zoom = (editor_state.world.zoom + 0.1).min(5.0);
            }
            if ui.button("Reset").clicked() {
                editor_state.world.zoom = 1.0;
                editor_state.world.camera_pos = bevy::prelude::Vec2::ZERO;
            }

            ui.separator();

            // Initialize tilemap button
            if editor_state.world.editing_tilemap.is_none() {
                if ui.button("New Tilemap").clicked() {
                    editor_state.world.editing_tilemap = Some(eryndor_shared::ZoneTilemap::new());
                    editor_state.status_message = "Created new tilemap".to_string();
                }
            } else {
                // Show unsaved indicator
                let label = if editor_state.has_unsaved_changes {
                    "Tilemap [*]"
                } else {
                    "Tilemap"
                };
                ui.label(label);

                // Save button - enabled when we have a zone selected
                // (allows saving even if no changes, as a "force save")
                let save_enabled = editor_state.world.current_zone.is_some();
                let save_label = if editor_state.has_unsaved_changes { "Save*" } else { "Save" };
                if ui.add_enabled(save_enabled, egui::Button::new(save_label)).clicked() {
                    editor_state.action_save_tilemap = true;
                }

                if ui.button("Clear").clicked() {
                    editor_state.world.editing_tilemap = Some(eryndor_shared::ZoneTilemap::new());
                    editor_state.status_message = "Cleared tilemap".to_string();
                    editor_state.has_unsaved_changes = true;
                }
            }
        });

        // Canvas toolbar - row 2: layer visibility + undo/redo
        ui.horizontal(|ui| {
            ui.label("Layers:");
            ui.checkbox(&mut editor_state.world.show_ground_layer, "Ground");
            ui.checkbox(&mut editor_state.world.show_decoration_layer, "Decor");
            ui.checkbox(&mut editor_state.world.show_tile_collision_layer, "Tile Coll.");

            ui.separator();

            ui.checkbox(&mut editor_state.world.show_collisions, "Box Coll.");
            ui.checkbox(&mut editor_state.world.show_spawn_regions, "Spawns");

            ui.separator();

            // Undo/Redo buttons
            let can_undo = editor_state.world.undo_history.can_undo();
            let can_redo = editor_state.world.undo_history.can_redo();

            if ui.add_enabled(can_undo, egui::Button::new("Undo"))
                .on_hover_text("Ctrl+Z")
                .clicked()
            {
                perform_undo(editor_state);
            }
            if ui.add_enabled(can_redo, egui::Button::new("Redo"))
                .on_hover_text("Ctrl+Y")
                .clicked()
            {
                perform_redo(editor_state);
            }
        });

        // Handle keyboard shortcuts (Ctrl+Z for undo, Ctrl+Y for redo)
        let input = ui.ctx().input(|i| {
            (i.modifiers.ctrl && i.key_pressed(egui::Key::Z) && !i.modifiers.shift,
             i.modifiers.ctrl && (i.key_pressed(egui::Key::Y) || (i.key_pressed(egui::Key::Z) && i.modifiers.shift)))
        });
        if input.0 {
            perform_undo(editor_state);
        }
        if input.1 {
            perform_redo(editor_state);
        }

        ui.separator();

        // Main canvas
        let available_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click_and_drag());

        let canvas_rect = response.rect;
        let tile_size = editor_state.world.grid_size;
        let zoom = editor_state.world.zoom;
        let camera_pos = editor_state.world.camera_pos;

        // Draw tilemap layers
        if let Some(tilemap) = &editor_state.world.editing_tilemap {
            draw_tilemap(&painter, &canvas_rect, tilemap, editor_state);
        }

        // Draw grid
        if editor_state.world.show_grid {
            draw_grid(&painter, &canvas_rect, editor_state);
        }

        // Draw origin marker
        let origin_screen = world_to_screen(0.0, 0.0, &canvas_rect, camera_pos, zoom);
        painter.circle_filled(origin_screen, 4.0, egui::Color32::WHITE);

        // Show hover tile indicator when using tile tools
        let is_tile_tool = matches!(
            editor_state.world.active_tool,
            WorldTool::PaintGround | WorldTool::PaintDecoration | WorldTool::PaintTileCollision | WorldTool::Erase
        );

        if is_tile_tool {
            if let Some(hover_pos) = response.hover_pos() {
                let (world_x, world_y) = screen_to_world(hover_pos, &canvas_rect, camera_pos, zoom);
                let (tile_x, tile_y) = world_to_tile(world_x, world_y, tile_size);

                // Draw tile cursor
                let tile_world_x = tile_x as f32 * tile_size;
                let tile_world_y = tile_y as f32 * tile_size;
                let tile_screen_min = world_to_screen(tile_world_x, tile_world_y, &canvas_rect, camera_pos, zoom);
                let tile_screen_max = world_to_screen(
                    tile_world_x + tile_size,
                    tile_world_y + tile_size,
                    &canvas_rect, camera_pos, zoom
                );

                let cursor_color = match editor_state.world.active_tool {
                    WorldTool::PaintGround => egui::Color32::from_rgba_unmultiplied(0, 255, 0, 100),
                    WorldTool::PaintDecoration => egui::Color32::from_rgba_unmultiplied(0, 200, 255, 100),
                    WorldTool::PaintTileCollision => egui::Color32::from_rgba_unmultiplied(255, 0, 0, 100),
                    WorldTool::Erase => egui::Color32::from_rgba_unmultiplied(255, 255, 0, 100),
                    _ => egui::Color32::TRANSPARENT,
                };

                painter.rect_filled(
                    egui::Rect::from_min_max(tile_screen_min, tile_screen_max),
                    0.0,
                    cursor_color,
                );

                // Show tile coordinates in status
                editor_state.status_message = format!("Tile: ({}, {})", tile_x, tile_y);
            }
        }

        // Handle canvas interactions
        if response.dragged() || response.clicked() {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let (world_x, world_y) = screen_to_world(pointer_pos, &canvas_rect, camera_pos, zoom);

                // Tile Picker (eyedropper) - Alt+click to sample tile regardless of current tool
                let alt_held = ui.ctx().input(|i| i.modifiers.alt);
                if alt_held && response.clicked() {
                    if let Some(ref tilemap) = editor_state.world.editing_tilemap {
                        let (tile_x, tile_y) = world_to_tile(world_x, world_y, tile_size);
                        let sampled_tile = sample_ground_tile(tilemap, tile_x, tile_y);
                        if sampled_tile > 0 {
                            editor_state.world.selected_tile = Some(sampled_tile);
                            editor_state.status_message = format!("Picked tile #{} at ({}, {})", sampled_tile, tile_x, tile_y);
                        } else {
                            editor_state.status_message = format!("No tile at ({}, {})", tile_x, tile_y);
                        }
                    }
                    // Skip normal tool handling when using eyedropper
                    return;
                }

                match editor_state.world.active_tool {
                    WorldTool::Pan => {
                        if response.dragged() {
                            let delta = response.drag_delta();
                            editor_state.world.camera_pos.x -= delta.x / zoom;
                            editor_state.world.camera_pos.y -= delta.y / zoom;
                        }
                    }
                    WorldTool::Select => {
                        // Select tile on click
                        if response.clicked() {
                            if let Some(ref tilemap) = editor_state.world.editing_tilemap {
                                let (tile_x, tile_y) = world_to_tile(world_x, world_y, tile_size);
                                let chunk_size = tilemap.chunk_size as i32;
                                let chunk_x = if tile_x >= 0 { tile_x / chunk_size } else { (tile_x - chunk_size + 1) / chunk_size };
                                let chunk_y = if tile_y >= 0 { tile_y / chunk_size } else { (tile_y - chunk_size + 1) / chunk_size };
                                let local_x = ((tile_x % chunk_size) + chunk_size) % chunk_size;
                                let local_y = ((tile_y % chunk_size) + chunk_size) % chunk_size;

                                let (ground_id, decoration_id, has_collision) =
                                    if let Some(chunk) = tilemap.get_chunk(chunk_x, chunk_y) {
                                        (
                                            chunk.get_ground(local_x as usize, local_y as usize).unwrap_or(0),
                                            chunk.get_decoration(local_x as usize, local_y as usize).unwrap_or(0),
                                            chunk.is_blocked(local_x as usize, local_y as usize),
                                        )
                                    } else {
                                        (0, 0, false)
                                    };

                                editor_state.world.selected_entity = SelectedEntity::Tile {
                                    tile_x,
                                    tile_y,
                                    ground_id,
                                    decoration_id,
                                    has_collision,
                                };
                                editor_state.status_message = format!("Selected tile ({}, {})", tile_x, tile_y);
                            }
                        }
                    }
                    WorldTool::PaintGround | WorldTool::PaintDecoration | WorldTool::PaintTileCollision | WorldTool::Erase => {
                        // Start batch on drag start
                        if response.drag_started() {
                            editor_state.world.undo_history.begin_batch();
                        }

                        // Paint tile at position
                        if let Some(ref mut tilemap) = editor_state.world.editing_tilemap {
                            let (tile_x, tile_y) = world_to_tile(world_x, world_y, tile_size);
                            let brush_size = editor_state.world.brush_size as i32;
                            let selected_tile = editor_state.world.selected_tile;
                            let active_tool = editor_state.world.active_tool.clone();

                            // Check decoration collision
                            let decoration_has_collision = |tile_id: u32| -> bool {
                                editor_state.world.tile_palette.decoration_tiles
                                    .iter()
                                    .find(|t| t.id == tile_id)
                                    .is_some_and(|t| t.has_collision)
                            };

                            // Paint in a square brush
                            for dy in 0..brush_size {
                                for dx in 0..brush_size {
                                    let tx = tile_x + dx;
                                    let ty = tile_y + dy;

                                    // Get chunk coordinates
                                    let chunk_size = tilemap.chunk_size as i32;
                                    let chunk_x = if tx >= 0 { tx / chunk_size } else { (tx - chunk_size + 1) / chunk_size };
                                    let chunk_y = if ty >= 0 { ty / chunk_size } else { (ty - chunk_size + 1) / chunk_size };
                                    let local_x = ((tx % chunk_size) + chunk_size) % chunk_size;
                                    let local_y = ((ty % chunk_size) + chunk_size) % chunk_size;

                                    let chunk = tilemap.get_or_create_chunk(chunk_x, chunk_y);

                                    match active_tool {
                                        WorldTool::PaintGround => {
                                            if let Some(tile_id) = selected_tile {
                                                let old_value = chunk.get_ground(local_x as usize, local_y as usize).unwrap_or(0);
                                                if old_value != tile_id {
                                                    chunk.set_ground(local_x as usize, local_y as usize, tile_id);
                                                    editor_state.world.undo_history.record(TileOperation {
                                                        tile_x: tx,
                                                        tile_y: ty,
                                                        layer: TileLayer::Ground,
                                                        old_value,
                                                        new_value: tile_id,
                                                    });
                                                }
                                            }
                                        }
                                        WorldTool::PaintDecoration => {
                                            if let Some(tile_id) = selected_tile {
                                                let old_value = chunk.get_decoration(local_x as usize, local_y as usize).unwrap_or(0);
                                                if old_value != tile_id {
                                                    chunk.set_decoration(local_x as usize, local_y as usize, tile_id);
                                                    editor_state.world.undo_history.record(TileOperation {
                                                        tile_x: tx,
                                                        tile_y: ty,
                                                        layer: TileLayer::Decoration,
                                                        old_value,
                                                        new_value: tile_id,
                                                    });
                                                    // Auto-set collision for tiles that have it
                                                    if decoration_has_collision(tile_id) {
                                                        let old_coll = if chunk.is_blocked(local_x as usize, local_y as usize) { 1 } else { 0 };
                                                        if old_coll != 1 {
                                                            chunk.set_collision(local_x as usize, local_y as usize, true);
                                                            editor_state.world.undo_history.record(TileOperation {
                                                                tile_x: tx,
                                                                tile_y: ty,
                                                                layer: TileLayer::Collision,
                                                                old_value: old_coll,
                                                                new_value: 1,
                                                            });
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        WorldTool::PaintTileCollision => {
                                            let old_value = if chunk.is_blocked(local_x as usize, local_y as usize) { 1 } else { 0 };
                                            if old_value != 1 {
                                                chunk.set_collision(local_x as usize, local_y as usize, true);
                                                editor_state.world.undo_history.record(TileOperation {
                                                    tile_x: tx,
                                                    tile_y: ty,
                                                    layer: TileLayer::Collision,
                                                    old_value,
                                                    new_value: 1,
                                                });
                                            }
                                        }
                                        WorldTool::Erase => {
                                            // Erase ground
                                            let old_ground = chunk.get_ground(local_x as usize, local_y as usize).unwrap_or(0);
                                            if old_ground != 0 {
                                                chunk.set_ground(local_x as usize, local_y as usize, 0);
                                                editor_state.world.undo_history.record(TileOperation {
                                                    tile_x: tx,
                                                    tile_y: ty,
                                                    layer: TileLayer::Ground,
                                                    old_value: old_ground,
                                                    new_value: 0,
                                                });
                                            }
                                            // Erase decoration
                                            let old_decor = chunk.get_decoration(local_x as usize, local_y as usize).unwrap_or(0);
                                            if old_decor != 0 {
                                                chunk.set_decoration(local_x as usize, local_y as usize, 0);
                                                editor_state.world.undo_history.record(TileOperation {
                                                    tile_x: tx,
                                                    tile_y: ty,
                                                    layer: TileLayer::Decoration,
                                                    old_value: old_decor,
                                                    new_value: 0,
                                                });
                                            }
                                            // Erase collision
                                            let old_coll = if chunk.is_blocked(local_x as usize, local_y as usize) { 1 } else { 0 };
                                            if old_coll != 0 {
                                                chunk.set_collision(local_x as usize, local_y as usize, false);
                                                editor_state.world.undo_history.record(TileOperation {
                                                    tile_x: tx,
                                                    tile_y: ty,
                                                    layer: TileLayer::Collision,
                                                    old_value: old_coll,
                                                    new_value: 0,
                                                });
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            editor_state.has_unsaved_changes = true;
                        }

                        // End batch on drag release or click (single click = start + immediate end)
                        if response.drag_stopped() || (response.clicked() && !response.dragged()) {
                            editor_state.world.undo_history.end_batch();
                        }
                    }
                    WorldTool::Fill => {
                        // Bucket fill - only on click, not drag
                        if response.clicked() {
                            if let Some(tile_id) = editor_state.world.selected_tile {
                                if let Some(ref mut tilemap) = editor_state.world.editing_tilemap {
                                    let (start_x, start_y) = world_to_tile(world_x, world_y, tile_size);
                                    // Start a new batch for this fill operation
                                    editor_state.world.undo_history.begin_batch();
                                    let filled_ops = flood_fill_ground_with_undo(tilemap, start_x, start_y, tile_id);
                                    if !filled_ops.is_empty() {
                                        let count = filled_ops.len();
                                        // Record all operations
                                        for op in filled_ops {
                                            editor_state.world.undo_history.record(op);
                                        }
                                        editor_state.has_unsaved_changes = true;
                                        editor_state.status_message = format!("Filled {} tiles", count);
                                    }
                                    // End the batch
                                    editor_state.world.undo_history.end_batch();
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Draw help text if no tilemap
        if editor_state.world.editing_tilemap.is_none() {
            painter.text(
                canvas_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Click 'New Tilemap' to start editing tiles",
                egui::FontId::default(),
                egui::Color32::GRAY,
            );
        }
}

/// Convert world coordinates to screen coordinates
fn world_to_screen(world_x: f32, world_y: f32, canvas_rect: &egui::Rect, camera_pos: bevy::prelude::Vec2, zoom: f32) -> egui::Pos2 {
    let screen_x = canvas_rect.center().x + (world_x - camera_pos.x) * zoom;
    let screen_y = canvas_rect.center().y - (world_y - camera_pos.y) * zoom; // Y is inverted
    egui::pos2(screen_x, screen_y)
}

/// Convert screen coordinates to world coordinates
fn screen_to_world(screen_pos: egui::Pos2, canvas_rect: &egui::Rect, camera_pos: bevy::prelude::Vec2, zoom: f32) -> (f32, f32) {
    let world_x = camera_pos.x + (screen_pos.x - canvas_rect.center().x) / zoom;
    let world_y = camera_pos.y - (screen_pos.y - canvas_rect.center().y) / zoom; // Y is inverted
    (world_x, world_y)
}

/// Convert world coordinates to tile coordinates
fn world_to_tile(world_x: f32, world_y: f32, tile_size: f32) -> (i32, i32) {
    let tile_x = (world_x / tile_size).floor() as i32;
    let tile_y = (world_y / tile_size).floor() as i32;
    (tile_x, tile_y)
}

/// Sample the ground tile at a position (for eyedropper/tile picker)
/// Returns the tile ID at the position, or 0 if no tile
fn sample_ground_tile(tilemap: &eryndor_shared::ZoneTilemap, tile_x: i32, tile_y: i32) -> u32 {
    let chunk_size = tilemap.chunk_size as i32;
    let chunk_x = if tile_x >= 0 { tile_x / chunk_size } else { (tile_x - chunk_size + 1) / chunk_size };
    let chunk_y = if tile_y >= 0 { tile_y / chunk_size } else { (tile_y - chunk_size + 1) / chunk_size };
    let local_x = ((tile_x % chunk_size) + chunk_size) % chunk_size;
    let local_y = ((tile_y % chunk_size) + chunk_size) % chunk_size;

    tilemap.get_chunk(chunk_x, chunk_y)
        .and_then(|c| c.get_ground(local_x as usize, local_y as usize))
        .unwrap_or(0)
}

/// Flood fill ground layer tiles starting from a position
/// Returns a vector of TileOperations for undo support
fn flood_fill_ground_with_undo(tilemap: &mut eryndor_shared::ZoneTilemap, start_x: i32, start_y: i32, fill_tile_id: u32) -> Vec<TileOperation> {
    use std::collections::{HashSet, VecDeque};

    let chunk_size = tilemap.chunk_size as i32;

    // Helper to get tile at position
    let get_ground_tile = |tm: &eryndor_shared::ZoneTilemap, tx: i32, ty: i32| -> u32 {
        let chunk_x = if tx >= 0 { tx / chunk_size } else { (tx - chunk_size + 1) / chunk_size };
        let chunk_y = if ty >= 0 { ty / chunk_size } else { (ty - chunk_size + 1) / chunk_size };
        let local_x = ((tx % chunk_size) + chunk_size) % chunk_size;
        let local_y = ((ty % chunk_size) + chunk_size) % chunk_size;

        tm.get_chunk(chunk_x, chunk_y)
            .and_then(|c| c.get_ground(local_x as usize, local_y as usize))
            .unwrap_or(0)
    };

    // Get the target tile ID (what we're replacing)
    let target_tile_id = get_ground_tile(tilemap, start_x, start_y);

    // Don't fill if target is same as fill tile
    if target_tile_id == fill_tile_id {
        return Vec::new();
    }

    // BFS flood fill
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
    let mut operations: Vec<TileOperation> = Vec::new();

    // Limit to prevent infinite fills (max 10000 tiles)
    const MAX_FILL: usize = 10000;

    queue.push_back((start_x, start_y));
    visited.insert((start_x, start_y));

    while let Some((tx, ty)) = queue.pop_front() {
        if operations.len() >= MAX_FILL {
            break;
        }

        // Check if this tile matches the target
        let current_tile = get_ground_tile(tilemap, tx, ty);
        if current_tile != target_tile_id {
            continue;
        }

        // Fill this tile
        let chunk_x = if tx >= 0 { tx / chunk_size } else { (tx - chunk_size + 1) / chunk_size };
        let chunk_y = if ty >= 0 { ty / chunk_size } else { (ty - chunk_size + 1) / chunk_size };
        let local_x = ((tx % chunk_size) + chunk_size) % chunk_size;
        let local_y = ((ty % chunk_size) + chunk_size) % chunk_size;

        let chunk = tilemap.get_or_create_chunk(chunk_x, chunk_y);
        chunk.set_ground(local_x as usize, local_y as usize, fill_tile_id);

        // Record the operation for undo
        operations.push(TileOperation {
            tile_x: tx,
            tile_y: ty,
            layer: TileLayer::Ground,
            old_value: target_tile_id,
            new_value: fill_tile_id,
        });

        // Add neighbors (4-connected)
        for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
            let nx = tx + dx;
            let ny = ty + dy;
            if !visited.contains(&(nx, ny)) {
                visited.insert((nx, ny));
                queue.push_back((nx, ny));
            }
        }
    }

    operations
}

/// Draw the tilemap layers
fn draw_tilemap(painter: &egui::Painter, canvas_rect: &egui::Rect, tilemap: &eryndor_shared::ZoneTilemap, editor_state: &EditorState) {
    let tile_size = editor_state.world.grid_size;
    let zoom = editor_state.world.zoom;
    let camera_pos = editor_state.world.camera_pos;
    let chunk_size = tilemap.chunk_size as i32;

    // Calculate visible area in world coordinates
    let (min_world_x, max_world_y) = screen_to_world(canvas_rect.min, canvas_rect, camera_pos, zoom);
    let (max_world_x, min_world_y) = screen_to_world(canvas_rect.max, canvas_rect, camera_pos, zoom);

    // Calculate visible chunk range
    let min_chunk_x = ((min_world_x / tile_size).floor() as i32 / chunk_size) - 1;
    let max_chunk_x = ((max_world_x / tile_size).ceil() as i32 / chunk_size) + 1;
    let min_chunk_y = ((min_world_y / tile_size).floor() as i32 / chunk_size) - 1;
    let max_chunk_y = ((max_world_y / tile_size).ceil() as i32 / chunk_size) + 1;

    // Draw tiles for visible chunks
    for (chunk_key, chunk) in &tilemap.chunks {
        let Some((chunk_x, chunk_y)) = eryndor_shared::ZoneTilemap::parse_chunk_key(chunk_key) else {
            continue;
        };

        // Skip chunks outside visible range
        if chunk_x < min_chunk_x || chunk_x > max_chunk_x || chunk_y < min_chunk_y || chunk_y > max_chunk_y {
            continue;
        }

        // Draw ground layer
        if editor_state.world.show_ground_layer {
            for (row_idx, row) in chunk.ground.iter().enumerate() {
                for (col_idx, &tile_id) in row.iter().enumerate() {
                    if tile_id == 0 {
                        continue;
                    }

                    let world_x = (chunk_x * chunk_size + col_idx as i32) as f32 * tile_size;
                    let world_y = (chunk_y * chunk_size + row_idx as i32) as f32 * tile_size;

                    let screen_min = world_to_screen(world_x, world_y + tile_size, canvas_rect, camera_pos, zoom);
                    let screen_max = world_to_screen(world_x + tile_size, world_y, canvas_rect, camera_pos, zoom);

                    // Color based on tile ID (simple visualization)
                    let color = tile_id_to_color(tile_id, false);
                    painter.rect_filled(egui::Rect::from_min_max(screen_min, screen_max), 0.0, color);
                }
            }
        }

        // Draw decoration layer
        if editor_state.world.show_decoration_layer {
            for (row_idx, row) in chunk.decorations.iter().enumerate() {
                for (col_idx, &tile_id) in row.iter().enumerate() {
                    if tile_id == 0 {
                        continue;
                    }

                    let world_x = (chunk_x * chunk_size + col_idx as i32) as f32 * tile_size;
                    let world_y = (chunk_y * chunk_size + row_idx as i32) as f32 * tile_size;

                    let screen_min = world_to_screen(world_x, world_y + tile_size, canvas_rect, camera_pos, zoom);
                    let screen_max = world_to_screen(world_x + tile_size, world_y, canvas_rect, camera_pos, zoom);

                    let color = tile_id_to_color(tile_id, true);
                    painter.rect_filled(egui::Rect::from_min_max(screen_min, screen_max), 0.0, color);
                }
            }
        }

        // Draw collision layer
        if editor_state.world.show_tile_collision_layer {
            for (row_idx, row) in chunk.collision.iter().enumerate() {
                for (col_idx, &is_blocked) in row.iter().enumerate() {
                    if is_blocked == 0 {
                        continue;
                    }

                    let world_x = (chunk_x * chunk_size + col_idx as i32) as f32 * tile_size;
                    let world_y = (chunk_y * chunk_size + row_idx as i32) as f32 * tile_size;

                    let screen_min = world_to_screen(world_x, world_y + tile_size, canvas_rect, camera_pos, zoom);
                    let screen_max = world_to_screen(world_x + tile_size, world_y, canvas_rect, camera_pos, zoom);

                    // Red overlay for collision
                    painter.rect_filled(
                        egui::Rect::from_min_max(screen_min, screen_max),
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(255, 0, 0, 80),
                    );
                    painter.rect_stroke(
                        egui::Rect::from_min_max(screen_min, screen_max),
                        0.0,
                        egui::Stroke::new(1.0, egui::Color32::RED),
                        egui::StrokeKind::Outside,
                    );
                }
            }
        }
    }
}

/// Convert tile ID to a display color (simple visualization until actual textures are implemented)
fn tile_id_to_color(tile_id: u32, is_decoration: bool) -> egui::Color32 {
    if is_decoration {
        // Decorations - browns and greens
        match tile_id {
            100..=123 => egui::Color32::from_rgb(34, 139, 34),  // Trees - forest green
            150 => egui::Color32::from_rgb(255, 182, 193),      // Flowers - pink
            151..=152 => egui::Color32::from_rgb(100, 149, 237), // Fountain/Well - blue
            153..=162 => egui::Color32::from_rgb(139, 90, 43),  // Props - brown
            _ => egui::Color32::from_rgb(128, 128, 128),        // Unknown - gray
        }
    } else {
        // Ground tiles
        match tile_id {
            1..=7 => egui::Color32::from_rgb(86, 125, 70),   // Grass - green
            10..=11 => egui::Color32::from_rgb(194, 178, 128), // Path - tan
            20..=21 => egui::Color32::from_rgb(128, 128, 128), // Cobble - gray
            30 => egui::Color32::from_rgb(160, 160, 160),    // Pavement - light gray
            40..=44 => egui::Color32::from_rgb(65, 105, 225), // Water - blue
            50 => egui::Color32::from_rgb(238, 214, 175),    // Beach - sand
            _ => egui::Color32::from_rgb(100, 100, 100),     // Unknown - dark gray
        }
    }
}

fn render_properties_panel(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    egui::SidePanel::right("world_right_panel")
        .default_width(250.0)
        .show_inside(ui, |ui| {
            ui.heading("Properties");

            if let Some(zone_id) = &editor_state.world.current_zone {
                ui.label(format!("Zone: {}", zone_id));

                ui.separator();

                // Zone properties
                ui.label("Zone Settings:");
                // TODO: Zone name, bounds, etc.

                ui.separator();

                // Selected entity properties
                ui.label("Selected Entity:");
                match &editor_state.world.selected_entity {
                    SelectedEntity::None => {
                        ui.label("(No entity selected)");
                        ui.label("Use Select tool and click on the canvas");
                    }
                    SelectedEntity::Tile { tile_x, tile_y, ground_id, decoration_id, has_collision } => {
                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            ui.strong("Tile");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Position:");
                            ui.label(format!("({}, {})", tile_x, tile_y));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Ground ID:");
                            ui.label(format!("{}", ground_id));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Decoration ID:");
                            ui.label(format!("{}", decoration_id));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Collision:");
                            ui.label(if *has_collision { "Yes" } else { "No" });
                        });
                    }
                    SelectedEntity::Npc { index, name } => {
                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            ui.strong("NPC");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Index:");
                            ui.label(format!("{}", index));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.label(name.as_str());
                        });
                    }
                    SelectedEntity::EnemyRegion { region_id } => {
                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            ui.strong("Enemy Region");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Region ID:");
                            ui.label(region_id.as_str());
                        });
                    }
                }
            } else {
                ui.label("No zone selected");
            }
        });
}

fn draw_grid(painter: &egui::Painter, rect: &egui::Rect, editor_state: &EditorState) {
    let tile_size = editor_state.world.grid_size;
    let zoom = editor_state.world.zoom;
    let camera_pos = editor_state.world.camera_pos;
    let grid_size_screen = tile_size * zoom;

    // Safety guard: prevent infinite loop if grid size is too small
    if grid_size_screen < 1.0 {
        return;
    }

    let grid_color = egui::Color32::from_rgba_unmultiplied(80, 80, 80, 60);

    // Calculate the world coordinates at the screen edges
    let (min_world_x, _) = screen_to_world(rect.min, rect, camera_pos, zoom);
    let (max_world_x, _) = screen_to_world(rect.max, rect, camera_pos, zoom);
    let (_, max_world_y) = screen_to_world(rect.min, rect, camera_pos, zoom);
    let (_, min_world_y) = screen_to_world(rect.max, rect, camera_pos, zoom);

    // Find the first grid line in world coordinates (aligned to tile boundaries)
    let first_grid_x = (min_world_x / tile_size).floor() * tile_size;
    let first_grid_y = (min_world_y / tile_size).floor() * tile_size;

    // Draw vertical lines (aligned to world X coordinates)
    let mut world_x = first_grid_x;
    while world_x <= max_world_x + tile_size {
        // Convert world X to screen X (using y=0 for simplicity, we just need X)
        let screen_pos = world_to_screen(world_x, 0.0, rect, camera_pos, zoom);
        painter.line_segment(
            [egui::pos2(screen_pos.x, rect.top()), egui::pos2(screen_pos.x, rect.bottom())],
            egui::Stroke::new(1.0, grid_color),
        );
        world_x += tile_size;
    }

    // Draw horizontal lines (aligned to world Y coordinates)
    let mut world_y = first_grid_y;
    while world_y <= max_world_y + tile_size {
        // Convert world Y to screen Y (using x=0 for simplicity, we just need Y)
        let screen_pos = world_to_screen(0.0, world_y, rect, camera_pos, zoom);
        painter.line_segment(
            [egui::pos2(rect.left(), screen_pos.y), egui::pos2(rect.right(), screen_pos.y)],
            egui::Stroke::new(1.0, grid_color),
        );
        world_y += tile_size;
    }
}

fn render_create_zone_dialog(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    if !editor_state.world.show_create_dialog {
        return;
    }

    egui::Window::new("Create New Zone")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.set_min_width(300.0);

            ui.horizontal(|ui| {
                ui.label("Zone Name:");
                ui.text_edit_singleline(&mut editor_state.world.new_zone_name);
            });

            ui.horizontal(|ui| {
                ui.label("Width:");
                ui.add(egui::DragValue::new(&mut editor_state.world.new_zone_width)
                    .speed(10.0)
                    .range(100.0..=10000.0)
                    .suffix(" px"));
            });

            ui.horizontal(|ui| {
                ui.label("Height:");
                ui.add(egui::DragValue::new(&mut editor_state.world.new_zone_height)
                    .speed(10.0)
                    .range(100.0..=10000.0)
                    .suffix(" px"));
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Create").clicked() {
                    editor_state.action_create_zone = true;
                }
                if ui.button("Cancel").clicked() {
                    editor_state.world.show_create_dialog = false;
                    editor_state.world.new_zone_name.clear();
                }
            });
        });
}

/// Render the bottom-docked tile palette panel at context level (enables proper resizing)
fn render_bottom_palette_panel_ctx(ctx: &egui::Context, editor_state: &mut EditorState) {
    // Let egui persist the panel height via its ID-based memory system
    // Using a constant default_height (not dynamic) so egui can remember resizes
    let response = egui::TopBottomPanel::bottom("tile_palette_panel")
        .resizable(true)
        .default_height(120.0) // Initial default, egui will remember user's resize
        .min_height(80.0)
        .max_height(400.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Tile Palette");
                ui.separator();

                // Category tabs
                if ui.selectable_label(
                    editor_state.world.tile_palette.selected_category == TileCategory::Ground,
                    "Ground"
                ).clicked() {
                    editor_state.world.tile_palette.selected_category = TileCategory::Ground;
                }
                if ui.selectable_label(
                    editor_state.world.tile_palette.selected_category == TileCategory::Decorations,
                    "Decor"
                ).clicked() {
                    editor_state.world.tile_palette.selected_category = TileCategory::Decorations;
                }
                if ui.selectable_label(
                    editor_state.world.tile_palette.selected_category == TileCategory::Terrain,
                    "Terrain"
                ).clicked() {
                    editor_state.world.tile_palette.selected_category = TileCategory::Terrain;
                }
                if ui.selectable_label(
                    editor_state.world.tile_palette.selected_category == TileCategory::Tilesets,
                    "Tilesets"
                ).clicked() {
                    editor_state.world.tile_palette.selected_category = TileCategory::Tilesets;
                }

                ui.separator();

                // Auto-tile toggle (when terrain tab is selected)
                if editor_state.world.tile_palette.selected_category == TileCategory::Terrain {
                    ui.checkbox(&mut editor_state.world.terrain_sets.auto_tile_enabled, "Auto-tile");
                    ui.separator();
                }

                // Selected tile/terrain info
                if editor_state.world.tile_palette.selected_category == TileCategory::Terrain {
                    if let Some(idx) = editor_state.world.terrain_sets.selected_set {
                        if let Some(set) = editor_state.world.terrain_sets.terrain_sets.get(idx) {
                            ui.label(format!("Selected: {}", set.name));
                        }
                    }
                } else if editor_state.world.tile_palette.selected_category == TileCategory::Tilesets {
                    // Show selected tileset and tile
                    if let Some(tileset_idx) = editor_state.world.tile_palette.selected_tileset {
                        if let Some(tileset) = editor_state.world.tile_palette.tilesets.get(tileset_idx) {
                            let tile_info = editor_state.world.tile_palette.selected_tile_index
                                .map(|idx| format!(" (tile #{})", idx))
                                .unwrap_or_default();
                            ui.label(format!("{}{}", tileset.name, tile_info));
                        }
                    }
                } else if let Some(tile_id) = editor_state.world.selected_tile {
                    ui.label(format!("Selected: #{}", tile_id));
                }
            });

            ui.separator();

            // Content depends on selected category
            if editor_state.world.tile_palette.selected_category == TileCategory::Terrain {
                // Terrain sets UI
                render_terrain_palette(ui, editor_state);
            } else if editor_state.world.tile_palette.selected_category == TileCategory::Tilesets {
                // New hybrid tileset system UI
                render_tileset_palette(ui, editor_state);
            } else {
                // Horizontal scrolling tile grid with visual previews
                egui::ScrollArea::horizontal()
                    .id_salt("tile_palette_scroll")
                    .show(ui, |ui| {
                        let tiles = match editor_state.world.tile_palette.selected_category {
                            TileCategory::Ground => &editor_state.world.tile_palette.ground_tiles,
                            TileCategory::Decorations => &editor_state.world.tile_palette.decoration_tiles,
                            TileCategory::Terrain | TileCategory::Tilesets => return, // Handled above
                        };

                    if tiles.is_empty() {
                        if !editor_state.world.tile_palette.loaded {
                            ui.label("Loading palette...");
                        } else {
                            ui.label("No tiles in this category");
                        }
                    } else {
                        ui.horizontal(|ui| {
                            let tile_size = 48.0;
                            for tile in tiles.iter() {
                                let is_selected = editor_state.world.selected_tile == Some(tile.id);

                                // Create a visual tile cell
                                let (rect, response) = ui.allocate_exact_size(
                                    egui::vec2(tile_size, tile_size),
                                    egui::Sense::click()
                                );

                                if response.clicked() {
                                    editor_state.world.selected_tile = Some(tile.id);
                                }

                                // Draw tile preview background
                                let fill_color = if is_selected {
                                    egui::Color32::from_rgb(80, 120, 200)
                                } else if response.hovered() {
                                    egui::Color32::from_rgb(60, 60, 80)
                                } else {
                                    egui::Color32::from_rgb(40, 40, 50)
                                };

                                ui.painter().rect_filled(rect, 4.0, fill_color);

                                // Draw tile preview - use texture if available, otherwise fallback to color
                                let preview_rect = rect.shrink(4.0);
                                if let Some(&texture_id) = editor_state.world.tile_palette.egui_texture_ids.get(&tile.id) {
                                    // Draw actual texture
                                    ui.painter().image(
                                        texture_id,
                                        preview_rect,
                                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                        egui::Color32::WHITE,
                                    );
                                } else {
                                    // Fallback to color preview while loading
                                    let preview_color = tile_preview_color(tile.id);
                                    ui.painter().rect_filled(preview_rect, 2.0, preview_color);
                                }

                                // Draw collision indicator
                                if tile.has_collision {
                                    let indicator_rect = egui::Rect::from_min_size(
                                        egui::pos2(rect.right() - 12.0, rect.top() + 2.0),
                                        egui::vec2(10.0, 10.0)
                                    );
                                    ui.painter().rect_filled(indicator_rect, 2.0, egui::Color32::RED);
                                }

                                // Draw selection border
                                if is_selected {
                                    ui.painter().rect_stroke(
                                        rect,
                                        4.0,
                                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                                        egui::StrokeKind::Outside
                                    );
                                }

                                // Tooltip with tile info
                                response.on_hover_ui(|ui| {
                                    ui.label(&tile.name);
                                    ui.label(format!("ID: {}", tile.id));
                                    if tile.has_collision {
                                        ui.colored_label(egui::Color32::RED, "Has Collision");
                                    }
                                });
                            }
                        });
                    }
                });
            } // end else (non-terrain categories)
        });

    // Update stored height from the actual panel rect
    let actual_height = response.response.rect.height();
    if (actual_height - editor_state.world.tile_palette_height).abs() > 1.0 {
        editor_state.world.tile_palette_height = actual_height;
    }
}

/// Render the terrain palette (terrain sets for auto-tiling)
fn render_terrain_palette(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    ui.horizontal(|ui| {
        // Terrain set list on the left
        egui::ScrollArea::horizontal()
            .id_salt("terrain_sets_scroll")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // "New" button
                    if ui.button("+ New Set").clicked() {
                        let new_set = crate::editor_state::TerrainSet::default();
                        editor_state.world.terrain_sets.terrain_sets.push(new_set);
                        editor_state.world.terrain_sets.selected_set = Some(
                            editor_state.world.terrain_sets.terrain_sets.len() - 1
                        );
                    }

                    ui.separator();

                    // Existing terrain sets
                    let selected = editor_state.world.terrain_sets.selected_set;
                    for (idx, set) in editor_state.world.terrain_sets.terrain_sets.iter().enumerate() {
                        let is_selected = selected == Some(idx);
                        let label = format!("{} ({})", set.name, set.mode.label());

                        if ui.selectable_label(is_selected, &label).clicked() {
                            editor_state.world.terrain_sets.selected_set = Some(idx);
                        }
                    }

                    if editor_state.world.terrain_sets.terrain_sets.is_empty() {
                        ui.label("No terrain sets. Click '+ New Set' to create one.");
                    }
                });
            });
    });

    // Show selected terrain set properties
    if let Some(idx) = editor_state.world.terrain_sets.selected_set {
        if idx < editor_state.world.terrain_sets.terrain_sets.len() {
            ui.separator();
            ui.horizontal(|ui| {
                let set = &mut editor_state.world.terrain_sets.terrain_sets[idx];

                // Name input
                ui.label("Name:");
                ui.add(egui::TextEdit::singleline(&mut set.name).desired_width(100.0));

                ui.separator();

                // Mode selector
                ui.label("Mode:");
                egui::ComboBox::from_id_salt("terrain_mode")
                    .selected_text(set.mode.label())
                    .show_ui(ui, |ui| {
                        for mode in crate::editor_state::TerrainMatchMode::all() {
                            ui.selectable_value(&mut set.mode, *mode, mode.label());
                        }
                    });

                ui.separator();

                // Inner/outer tile IDs
                ui.label("Inner:");
                ui.add(egui::DragValue::new(&mut set.inner_tile).speed(1));
                ui.label("Outer:");
                ui.add(egui::DragValue::new(&mut set.outer_tile).speed(1));

                ui.separator();

                // Delete button
                if ui.button("Delete Set").clicked() {
                    editor_state.world.terrain_sets.terrain_sets.remove(idx);
                    editor_state.world.terrain_sets.selected_set = None;
                }
            });

            // Show tile mappings hint
            ui.horizontal(|ui| {
                let set = &editor_state.world.terrain_sets.terrain_sets.get(idx);
                if let Some(set) = set {
                    ui.label(format!(
                        "Tiles: {} defined (max {} for {} mode)",
                        set.tiles.len(),
                        set.mode.max_bitmask() + 1,
                        set.mode.label()
                    ));
                    ui.label(" | Paint with ground tiles to use auto-tiling.");
                }
            });
        }
    }
}

/// Render the tileset palette (hybrid system with spritesheets + individual images)
fn render_tileset_palette(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    // Tileset selector row
    ui.horizontal(|ui| {
        ui.label("Tileset:");

        // Tileset dropdown
        let current_tileset_name = editor_state.world.tile_palette.selected_tileset
            .and_then(|idx| editor_state.world.tile_palette.tilesets.get(idx))
            .map(|t| t.name.as_str())
            .unwrap_or("Select...");

        egui::ComboBox::from_id_salt("tileset_selector")
            .selected_text(current_tileset_name)
            .show_ui(ui, |ui| {
                for (idx, tileset) in editor_state.world.tile_palette.tilesets.iter().enumerate() {
                    let is_selected = editor_state.world.tile_palette.selected_tileset == Some(idx);
                    if ui.selectable_label(is_selected, &tileset.name).clicked() {
                        editor_state.world.tile_palette.selected_tileset = Some(idx);
                        editor_state.world.tile_palette.selected_tile_index = None;
                    }
                }
            });

        if editor_state.world.tile_palette.tilesets.is_empty() {
            ui.label("(No tilesets loaded)");
        }

        // Show tileset info
        if let Some(tileset_idx) = editor_state.world.tile_palette.selected_tileset {
            if let Some(tileset) = editor_state.world.tile_palette.tilesets.get(tileset_idx) {
                ui.separator();
                ui.label(format!("{} sources, {} tiles", tileset.sources.len(), tileset.total_tiles));
            }
        }
    });

    ui.separator();

    // Tile grid for selected tileset
    if let Some(tileset_idx) = editor_state.world.tile_palette.selected_tileset {
        if let Some(tileset) = editor_state.world.tile_palette.tilesets.get(tileset_idx).cloned() {
            egui::ScrollArea::both()
                .id_salt("tileset_tiles_scroll")
                .show(ui, |ui| {
                    // Calculate display size
                    let display_tile_size = tileset.display_tile_size as f32;

                    // For each source in the tileset, render its tiles
                    for source in &tileset.sources {
                        match source {
                            TileSource::Spritesheet {
                                path, columns, rows, first_tile_index, ..
                            } => {
                                render_spritesheet_grid(
                                    ui,
                                    editor_state,
                                    path,
                                    *columns,
                                    *rows,
                                    display_tile_size,
                                    *first_tile_index,
                                );
                            }
                            TileSource::SingleImage {
                                path, tile_index, name, has_collision
                            } => {
                                render_single_tile(
                                    ui,
                                    editor_state,
                                    path,
                                    *tile_index,
                                    name,
                                    *has_collision,
                                    display_tile_size,
                                );
                            }
                        }
                    }
                });
        }
    } else {
        ui.label("Select a tileset from the dropdown above");
    }
}

/// Render a spritesheet as a grid of selectable tiles
fn render_spritesheet_grid(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    path: &str,
    columns: u32,
    rows: u32,
    display_tile_size: f32,
    first_tile_index: u32,
) {
    if columns == 0 || rows == 0 {
        return;
    }

    // Get texture for this spritesheet
    let texture_id = editor_state.world.tile_palette.tileset_egui_ids.get(path).copied();

    ui.vertical(|ui| {
        // Show spritesheet label
        let filename = path.rsplit('/').next().unwrap_or(path);
        ui.label(format!("{}x{} ({})", columns, rows, filename));

        // Render grid of tiles
        for row in 0..rows {
            ui.horizontal(|ui| {
                for col in 0..columns {
                    let tile_index = first_tile_index + row * columns + col;
                    let is_selected = editor_state.world.tile_palette.selected_tile_index == Some(tile_index);

                    // Allocate space for this tile
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(display_tile_size, display_tile_size),
                        egui::Sense::click()
                    );

                    if response.clicked() {
                        editor_state.world.tile_palette.selected_tile_index = Some(tile_index);
                        // Also update the legacy selected_tile for compatibility
                        editor_state.world.selected_tile = Some(tile_index);
                    }

                    // Background
                    let fill_color = if is_selected {
                        egui::Color32::from_rgb(80, 120, 200)
                    } else if response.hovered() {
                        egui::Color32::from_rgb(60, 60, 80)
                    } else {
                        egui::Color32::from_rgb(40, 40, 50)
                    };
                    ui.painter().rect_filled(rect, 2.0, fill_color);

                    // Draw tile from spritesheet using UV coordinates
                    let preview_rect = rect.shrink(2.0);
                    if let Some(tex_id) = texture_id {
                        // Calculate UV coordinates for this tile
                        let u_min = col as f32 / columns as f32;
                        let v_min = row as f32 / rows as f32;
                        let u_max = (col + 1) as f32 / columns as f32;
                        let v_max = (row + 1) as f32 / rows as f32;

                        ui.painter().image(
                            tex_id,
                            preview_rect,
                            egui::Rect::from_min_max(
                                egui::pos2(u_min, v_min),
                                egui::pos2(u_max, v_max)
                            ),
                            egui::Color32::WHITE,
                        );
                    } else {
                        // Fallback: checkerboard pattern while loading
                        let checker_color = if (col + row) % 2 == 0 {
                            egui::Color32::from_rgb(100, 100, 100)
                        } else {
                            egui::Color32::from_rgb(80, 80, 80)
                        };
                        ui.painter().rect_filled(preview_rect, 1.0, checker_color);
                    }

                    // Selection border
                    if is_selected {
                        ui.painter().rect_stroke(
                            rect,
                            2.0,
                            egui::Stroke::new(2.0, egui::Color32::WHITE),
                            egui::StrokeKind::Outside
                        );
                    }

                    // Tooltip
                    response.on_hover_ui(|ui| {
                        ui.label(format!("Tile #{}", tile_index));
                        ui.label(format!("Grid: ({}, {})", col, row));
                    });
                }
            });
        }

        ui.add_space(8.0);
    });
}

/// Render a single image tile
fn render_single_tile(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    path: &str,
    tile_index: u32,
    name: &str,
    has_collision: bool,
    display_tile_size: f32,
) {
    let is_selected = editor_state.world.tile_palette.selected_tile_index == Some(tile_index);
    let texture_id = editor_state.world.tile_palette.tileset_egui_ids.get(path).copied();

    ui.horizontal(|ui| {
        // Allocate space for this tile (potentially larger than grid tiles)
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(display_tile_size, display_tile_size),
            egui::Sense::click()
        );

        if response.clicked() {
            editor_state.world.tile_palette.selected_tile_index = Some(tile_index);
            // Also update the legacy selected_tile for compatibility
            editor_state.world.selected_tile = Some(tile_index);
        }

        // Background
        let fill_color = if is_selected {
            egui::Color32::from_rgb(80, 120, 200)
        } else if response.hovered() {
            egui::Color32::from_rgb(60, 60, 80)
        } else {
            egui::Color32::from_rgb(40, 40, 50)
        };
        ui.painter().rect_filled(rect, 4.0, fill_color);

        // Draw tile image
        let preview_rect = rect.shrink(4.0);
        if let Some(tex_id) = texture_id {
            ui.painter().image(
                tex_id,
                preview_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        } else {
            // Fallback color while loading
            ui.painter().rect_filled(preview_rect, 2.0, egui::Color32::from_rgb(80, 80, 80));
        }

        // Collision indicator
        if has_collision {
            let indicator_rect = egui::Rect::from_min_size(
                egui::pos2(rect.right() - 12.0, rect.top() + 2.0),
                egui::vec2(10.0, 10.0)
            );
            ui.painter().rect_filled(indicator_rect, 2.0, egui::Color32::RED);
        }

        // Selection border
        if is_selected {
            ui.painter().rect_stroke(
                rect,
                4.0,
                egui::Stroke::new(2.0, egui::Color32::WHITE),
                egui::StrokeKind::Outside
            );
        }

        // Label next to the tile
        ui.label(name);

        // Tooltip
        response.on_hover_ui(|ui| {
            ui.label(name);
            ui.label(format!("Tile #{}", tile_index));
            if has_collision {
                ui.colored_label(egui::Color32::RED, "Has Collision");
            }
        });
    });
}

/// Perform undo operation
fn perform_undo(editor_state: &mut EditorState) {
    if let Some(entry) = editor_state.world.undo_history.pop_undo() {
        if let Some(ref mut tilemap) = editor_state.world.editing_tilemap {
            let chunk_size = tilemap.chunk_size as i32;

            // Create reverse entry for redo
            let mut redo_ops = Vec::new();

            // Apply operations in reverse
            for op in entry.operations.iter().rev() {
                let chunk_x = if op.tile_x >= 0 { op.tile_x / chunk_size } else { (op.tile_x - chunk_size + 1) / chunk_size };
                let chunk_y = if op.tile_y >= 0 { op.tile_y / chunk_size } else { (op.tile_y - chunk_size + 1) / chunk_size };
                let local_x = ((op.tile_x % chunk_size) + chunk_size) % chunk_size;
                let local_y = ((op.tile_y % chunk_size) + chunk_size) % chunk_size;

                let chunk = tilemap.get_or_create_chunk(chunk_x, chunk_y);

                // Get current value for redo
                let current_value = match op.layer {
                    TileLayer::Ground => chunk.get_ground(local_x as usize, local_y as usize).unwrap_or(0),
                    TileLayer::Decoration => chunk.get_decoration(local_x as usize, local_y as usize).unwrap_or(0),
                    TileLayer::Collision => if chunk.is_blocked(local_x as usize, local_y as usize) { 1 } else { 0 },
                };

                // Apply old value
                match op.layer {
                    TileLayer::Ground => chunk.set_ground(local_x as usize, local_y as usize, op.old_value),
                    TileLayer::Decoration => chunk.set_decoration(local_x as usize, local_y as usize, op.old_value),
                    TileLayer::Collision => chunk.set_collision(local_x as usize, local_y as usize, op.old_value != 0),
                }

                // Record for redo (reversed operation)
                redo_ops.push(TileOperation {
                    tile_x: op.tile_x,
                    tile_y: op.tile_y,
                    layer: op.layer,
                    old_value: current_value,
                    new_value: op.old_value,
                });
            }

            editor_state.world.undo_history.push_redo(UndoEntry { operations: redo_ops });
            editor_state.has_unsaved_changes = true;
            editor_state.status_message = format!("Undo: {} tiles", entry.operations.len());
        }
    }
}

/// Perform redo operation
fn perform_redo(editor_state: &mut EditorState) {
    if let Some(entry) = editor_state.world.undo_history.pop_redo() {
        if let Some(ref mut tilemap) = editor_state.world.editing_tilemap {
            let chunk_size = tilemap.chunk_size as i32;

            // Create reverse entry for undo
            let mut undo_ops = Vec::new();

            // Apply operations
            for op in entry.operations.iter() {
                let chunk_x = if op.tile_x >= 0 { op.tile_x / chunk_size } else { (op.tile_x - chunk_size + 1) / chunk_size };
                let chunk_y = if op.tile_y >= 0 { op.tile_y / chunk_size } else { (op.tile_y - chunk_size + 1) / chunk_size };
                let local_x = ((op.tile_x % chunk_size) + chunk_size) % chunk_size;
                let local_y = ((op.tile_y % chunk_size) + chunk_size) % chunk_size;

                let chunk = tilemap.get_or_create_chunk(chunk_x, chunk_y);

                // Get current value for undo
                let current_value = match op.layer {
                    TileLayer::Ground => chunk.get_ground(local_x as usize, local_y as usize).unwrap_or(0),
                    TileLayer::Decoration => chunk.get_decoration(local_x as usize, local_y as usize).unwrap_or(0),
                    TileLayer::Collision => if chunk.is_blocked(local_x as usize, local_y as usize) { 1 } else { 0 },
                };

                // Apply new value
                match op.layer {
                    TileLayer::Ground => chunk.set_ground(local_x as usize, local_y as usize, op.new_value),
                    TileLayer::Decoration => chunk.set_decoration(local_x as usize, local_y as usize, op.new_value),
                    TileLayer::Collision => chunk.set_collision(local_x as usize, local_y as usize, op.new_value != 0),
                }

                // Record for undo (reversed operation)
                undo_ops.push(TileOperation {
                    tile_x: op.tile_x,
                    tile_y: op.tile_y,
                    layer: op.layer,
                    old_value: current_value,
                    new_value: op.new_value,
                });
            }

            // Push to undo stack (don't clear redo - that's handled by push_entry)
            editor_state.world.undo_history.undo_stack.push(UndoEntry { operations: undo_ops });
            editor_state.has_unsaved_changes = true;
            editor_state.status_message = format!("Redo: {} tiles", entry.operations.len());
        }
    }
}

// === Auto-Tiling Algorithm ===

/// Neighbor offsets for different matching modes
/// Corner mode: NW, NE, SE, SW (bits 0-3)
const CORNER_OFFSETS: [(i32, i32); 4] = [(-1, 1), (1, 1), (1, -1), (-1, -1)];
/// Edge mode: N, E, S, W (bits 0-3)
const EDGE_OFFSETS: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];
/// Mixed mode: N, NE, E, SE, S, SW, W, NW (bits 0-7)
const MIXED_OFFSETS: [(i32, i32); 8] = [
    (0, 1), (1, 1), (1, 0), (1, -1),
    (0, -1), (-1, -1), (-1, 0), (-1, 1)
];

/// Get the ground tile at a position (helper for auto-tiling)
fn get_tile_at(tilemap: &eryndor_shared::ZoneTilemap, tx: i32, ty: i32) -> u32 {
    let chunk_size = tilemap.chunk_size as i32;
    let chunk_x = if tx >= 0 { tx / chunk_size } else { (tx - chunk_size + 1) / chunk_size };
    let chunk_y = if ty >= 0 { ty / chunk_size } else { (ty - chunk_size + 1) / chunk_size };
    let local_x = ((tx % chunk_size) + chunk_size) % chunk_size;
    let local_y = ((ty % chunk_size) + chunk_size) % chunk_size;

    tilemap.get_chunk(chunk_x, chunk_y)
        .and_then(|c| c.get_ground(local_x as usize, local_y as usize))
        .unwrap_or(0)
}

/// Check if a tile is part of a terrain set (i.e., one of its tile IDs)
fn is_terrain_tile(terrain_set: &TerrainSet, tile_id: u32) -> bool {
    tile_id == terrain_set.inner_tile
        || tile_id == terrain_set.outer_tile
        || terrain_set.tiles.iter().any(|t| t.tile_id == tile_id)
}

/// Calculate the bitmask for a position based on neighboring terrain tiles
fn calculate_bitmask(
    tilemap: &eryndor_shared::ZoneTilemap,
    tx: i32,
    ty: i32,
    terrain_set: &TerrainSet,
) -> u8 {
    let offsets: &[(i32, i32)] = match terrain_set.mode {
        TerrainMatchMode::Corner => &CORNER_OFFSETS,
        TerrainMatchMode::Edge => &EDGE_OFFSETS,
        TerrainMatchMode::Mixed => &MIXED_OFFSETS,
    };

    let mut bitmask: u8 = 0;
    for (bit, (dx, dy)) in offsets.iter().enumerate() {
        let neighbor_tile = get_tile_at(tilemap, tx + dx, ty + dy);
        // Set bit if neighbor is part of this terrain set
        if is_terrain_tile(terrain_set, neighbor_tile) {
            bitmask |= 1 << bit;
        }
    }
    bitmask
}

/// Apply auto-tiling to a single tile position
/// Returns the TileOperation if the tile was changed, None otherwise
fn apply_auto_tile(
    tilemap: &mut eryndor_shared::ZoneTilemap,
    tx: i32,
    ty: i32,
    terrain_set: &TerrainSet,
) -> Option<TileOperation> {
    let chunk_size = tilemap.chunk_size as i32;
    let chunk_x = if tx >= 0 { tx / chunk_size } else { (tx - chunk_size + 1) / chunk_size };
    let chunk_y = if ty >= 0 { ty / chunk_size } else { (ty - chunk_size + 1) / chunk_size };
    let local_x = ((tx % chunk_size) + chunk_size) % chunk_size;
    let local_y = ((ty % chunk_size) + chunk_size) % chunk_size;

    // Get current tile
    let current_tile = tilemap.get_chunk(chunk_x, chunk_y)
        .and_then(|c| c.get_ground(local_x as usize, local_y as usize))
        .unwrap_or(0);

    // Only auto-tile if current tile is part of this terrain set
    if !is_terrain_tile(terrain_set, current_tile) {
        return None;
    }

    // Calculate what the tile should be based on neighbors
    let bitmask = calculate_bitmask(tilemap, tx, ty, terrain_set);
    let new_tile = terrain_set.get_tile_for_bitmask(bitmask);

    // Only update if different
    if new_tile != current_tile {
        let chunk = tilemap.get_or_create_chunk(chunk_x, chunk_y);
        chunk.set_ground(local_x as usize, local_y as usize, new_tile);
        Some(TileOperation {
            tile_x: tx,
            tile_y: ty,
            layer: TileLayer::Ground,
            old_value: current_tile,
            new_value: new_tile,
        })
    } else {
        None
    }
}

/// Update auto-tiling for a tile and all its neighbors
/// Call this after placing a terrain tile
/// Returns a vector of TileOperations for undo support
pub fn update_auto_tile_region(
    tilemap: &mut eryndor_shared::ZoneTilemap,
    center_x: i32,
    center_y: i32,
    terrain_set: &TerrainSet,
) -> Vec<TileOperation> {
    let mut operations = Vec::new();

    // Update the center tile first
    if let Some(op) = apply_auto_tile(tilemap, center_x, center_y, terrain_set) {
        operations.push(op);
    }

    // Get offsets for this mode to update neighbors
    let neighbor_offsets: &[(i32, i32)] = match terrain_set.mode {
        TerrainMatchMode::Corner => &CORNER_OFFSETS,
        TerrainMatchMode::Edge => &EDGE_OFFSETS,
        TerrainMatchMode::Mixed => &MIXED_OFFSETS,
    };

    // Update all neighbors
    for (dx, dy) in neighbor_offsets {
        let nx = center_x + dx;
        let ny = center_y + dy;
        if let Some(op) = apply_auto_tile(tilemap, nx, ny, terrain_set) {
            operations.push(op);
        }
    }

    // For mixed mode, we may need to update extended neighbors
    // (tiles that share a corner but not in the immediate 8 neighbors)
    if terrain_set.mode == TerrainMatchMode::Mixed {
        // Additional corner-adjacent tiles for complete coverage
        for (dx, dy) in [(-1, 2), (0, 2), (1, 2), (2, 1), (2, 0), (2, -1),
                         (1, -2), (0, -2), (-1, -2), (-2, -1), (-2, 0), (-2, 1)] {
            if let Some(op) = apply_auto_tile(tilemap, center_x + dx, center_y + dy, terrain_set) {
                operations.push(op);
            }
        }
    }

    operations
}

/// Generate a preview color for a tile based on its ID
fn tile_preview_color(tile_id: u32) -> egui::Color32 {
    // Generate distinct colors based on tile ID
    let hue = ((tile_id * 137) % 360) as f32;
    let saturation = 0.5 + ((tile_id * 53) % 50) as f32 / 100.0;
    let value = 0.4 + ((tile_id * 31) % 40) as f32 / 100.0;

    // HSV to RGB conversion
    let c = value * saturation;
    let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m = value - c;

    let (r1, g1, b1) = if hue < 60.0 {
        (c, x, 0.0)
    } else if hue < 120.0 {
        (x, c, 0.0)
    } else if hue < 180.0 {
        (0.0, c, x)
    } else if hue < 240.0 {
        (0.0, x, c)
    } else if hue < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    egui::Color32::from_rgb(
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    )
}
