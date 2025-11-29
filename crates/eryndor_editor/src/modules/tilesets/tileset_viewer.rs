//! Tileset viewer panel - center area showing the tileset grid

use bevy_egui::egui;
use crate::editor_state::{
    EditorState, TileSource, TilesetEditMode, TileMetadata,
    CollisionShape, CollisionShapeType,
};

/// Render the tileset viewer with grid
pub fn render(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    // Get the selected tileset
    let Some(tileset_index) = editor_state.tilesets.selected_tileset else {
        ui.centered_and_justified(|ui| {
            ui.label("Select a tileset from the list to view and edit");
        });
        return;
    };

    // Check if tileset exists and has sources
    let tileset_info = editor_state.world.tile_palette.tilesets.get(tileset_index).map(|t| {
        (t.sources.clone(), t.sources.is_empty())
    });

    let Some((sources, sources_empty)) = tileset_info else {
        ui.centered_and_justified(|ui| {
            ui.label("Tileset not found");
        });
        return;
    };

    if sources_empty {
        ui.centered_and_justified(|ui| {
            ui.label("Tileset has no sources");
        });
        return;
    }

    let zoom = editor_state.tilesets.zoom;
    let show_grid = editor_state.tilesets.show_grid;
    let show_terrain_overlay = editor_state.tilesets.show_terrain_overlay;
    let show_collision_shapes = editor_state.tilesets.show_collision_shapes;
    let selected_tile = editor_state.tilesets.selected_tile;
    let edit_mode = editor_state.tilesets.edit_mode.clone();

    // Scrollable area for the tileset
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // For each source in the tileset, render its tiles
            for source in &sources {
                match source {
                    TileSource::Spritesheet {
                        path,
                        columns,
                        rows,
                        tile_width,
                        tile_height,
                        image_width,
                        image_height,
                        first_tile_index,
                        ..
                    } => {
                        // Skip if not loaded yet
                        if *columns == 0 || *rows == 0 {
                            ui.label(format!("Loading spritesheet: {}", path));
                            continue;
                        }

                        // Debug: Show texture status
                        let has_texture = editor_state.world.tile_palette.tileset_egui_ids.contains_key(path);
                        let has_handle = editor_state.world.tile_palette.tileset_texture_handles.contains_key(path);
                        ui.label(format!(
                            "Spritesheet: {} ({}x{} = {} tiles) | Handle: {} | Texture: {}",
                            path, columns, rows, columns * rows, has_handle, has_texture
                        ));

                        render_spritesheet_grid(
                            ui,
                            editor_state,
                            path,
                            *columns,
                            *rows,
                            *tile_width,
                            *tile_height,
                            *image_width,
                            *image_height,
                            *first_tile_index,
                            zoom,
                            show_grid,
                            show_terrain_overlay,
                            show_collision_shapes,
                            selected_tile,
                            &edit_mode,
                        );
                    }
                    TileSource::SingleImage { path, tile_index, name, .. } => {
                        // Render single image as a single tile
                        render_single_image_tile(
                            ui,
                            editor_state,
                            path,
                            name,
                            *tile_index,
                            zoom,
                            show_grid,
                            selected_tile,
                        );
                    }
                }
            }
        });
}

/// Render a spritesheet as a grid of tiles
fn render_spritesheet_grid(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    path: &str,
    columns: u32,
    rows: u32,
    tile_width: u32,
    tile_height: u32,
    _image_width: u32,
    _image_height: u32,
    first_tile_index: u32,
    zoom: f32,
    show_grid: bool,
    show_terrain_overlay: bool,
    show_collision_shapes: bool,
    selected_tile: Option<u32>,
    edit_mode: &TilesetEditMode,
) {
    let tile_size = egui::vec2(tile_width as f32 * zoom, tile_height as f32 * zoom);
    let grid_width = columns as f32 * tile_size.x;
    let grid_height = rows as f32 * tile_size.y;

    // Check if we have the texture loaded
    let texture_id = editor_state.world.tile_palette.tileset_egui_ids.get(path).copied();

    // Reserve space for the grid
    let (response, painter) = ui.allocate_painter(
        egui::vec2(grid_width, grid_height),
        egui::Sense::click_and_drag(),
    );

    let rect = response.rect;

    // Draw background
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(30, 30, 40));

    // Debug: Show rect dimensions and texture status
    ui.label(format!(
        "DEBUG: rect=({:.0},{:.0})-({:.0},{:.0}) size={:.0}x{:.0} | tex_id={:?}",
        rect.min.x, rect.min.y, rect.max.x, rect.max.y,
        rect.width(), rect.height(),
        texture_id.map(|id| format!("{:?}", id)).unwrap_or_else(|| "None".to_string())
    ));

    // If we have the texture, draw the tileset image
    if let Some(tex_id) = texture_id {
        // Draw the full tileset image scaled to fit
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.image(tex_id, rect, uv, egui::Color32::WHITE);

        // Debug: Draw a bright red border to confirm image call was made
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(3.0, egui::Color32::RED),
            egui::StrokeKind::Inside,
        );
    } else {
        // Draw placeholder tiles
        for row in 0..rows {
            for col in 0..columns {
                let tile_index = first_tile_index + row * columns + col;
                let tile_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(col as f32 * tile_size.x, row as f32 * tile_size.y),
                    tile_size,
                );

                // Alternating colors for visibility
                let color = if (row + col) % 2 == 0 {
                    egui::Color32::from_rgb(60, 60, 80)
                } else {
                    egui::Color32::from_rgb(50, 50, 70)
                };

                painter.rect_filled(tile_rect, 0.0, color);

                // Draw tile index in center (guard against zero font size)
                let font_size = (10.0 * zoom.min(2.0)).max(1.0);
                painter.text(
                    tile_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{}", tile_index),
                    egui::FontId::proportional(font_size),
                    egui::Color32::from_rgb(120, 120, 140),
                );
            }
        }
    }

    // Draw grid overlay
    if show_grid {
        let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, 100));

        // Vertical lines
        for col in 0..=columns {
            let x = rect.min.x + col as f32 * tile_size.x;
            painter.line_segment(
                [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                stroke,
            );
        }

        // Horizontal lines
        for row in 0..=rows {
            let y = rect.min.y + row as f32 * tile_size.y;
            painter.line_segment(
                [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                stroke,
            );
        }
    }

    // Get terrain and collision data for overlays
    let tileset_index = editor_state.tilesets.selected_tileset;
    let selected_terrain_set = editor_state.tilesets.selected_terrain_set;

    // Draw terrain overlay for each tile
    if show_terrain_overlay {
        if let (Some(tileset_idx), Some(terrain_set_idx)) = (tileset_index, selected_terrain_set) {
            // Get terrain colors
            let terrain_colors: Vec<[u8; 4]> = editor_state
                .world
                .tile_palette
                .tilesets
                .get(tileset_idx)
                .and_then(|ts| ts.terrain_sets.get(terrain_set_idx))
                .map(|ts| ts.terrains.iter().map(|t| t.color).collect())
                .unwrap_or_default();

            // Get tile metadata
            let tile_metadata: std::collections::HashMap<u32, [Option<usize>; 4]> = editor_state
                .world
                .tile_palette
                .tilesets
                .get(tileset_idx)
                .map(|ts| {
                    ts.tile_metadata
                        .iter()
                        .filter_map(|(k, v)| v.terrain_corners.map(|c| (*k, c)))
                        .collect()
                })
                .unwrap_or_default();

            // Draw terrain overlays
            for row in 0..rows {
                for col in 0..columns {
                    let tile_index = first_tile_index + row * columns + col;

                    if let Some(corners) = tile_metadata.get(&tile_index) {
                        let tile_rect = egui::Rect::from_min_size(
                            rect.min + egui::vec2(col as f32 * tile_size.x, row as f32 * tile_size.y),
                            tile_size,
                        );

                        draw_terrain_corners(&painter, tile_rect, corners, &terrain_colors);
                    }
                }
            }
        }
    }

    // Draw collision shape overlays
    if show_collision_shapes {
        if let Some(tileset_idx) = tileset_index {
            let collision_data: Vec<(u32, Vec<crate::editor_state::CollisionShape>)> = editor_state
                .world
                .tile_palette
                .tilesets
                .get(tileset_idx)
                .map(|ts| {
                    ts.tile_metadata
                        .iter()
                        .filter(|(_, v)| !v.collision_shapes.is_empty())
                        .map(|(k, v)| (*k, v.collision_shapes.clone()))
                        .collect()
                })
                .unwrap_or_default();

            for (tile_index, shapes) in collision_data {
                if tile_index >= first_tile_index && tile_index < first_tile_index + rows * columns {
                    let local_index = tile_index - first_tile_index;
                    let tile_row = local_index / columns;
                    let tile_col = local_index % columns;

                    let tile_rect = egui::Rect::from_min_size(
                        rect.min + egui::vec2(tile_col as f32 * tile_size.x, tile_row as f32 * tile_size.y),
                        tile_size,
                    );

                    draw_collision_shapes(&painter, tile_rect, &shapes, zoom);
                }
            }
        }
    }

    // Draw selection highlight
    if let Some(sel_tile) = selected_tile {
        if sel_tile >= first_tile_index && sel_tile < first_tile_index + rows * columns {
            let local_index = sel_tile - first_tile_index;
            let row = local_index / columns;
            let col = local_index % columns;

            let sel_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(col as f32 * tile_size.x, row as f32 * tile_size.y),
                tile_size,
            );

            painter.rect_stroke(
                sel_rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::YELLOW),
                egui::StrokeKind::Outside,
            );
        }
    }

    // In Collision mode, draw preview of shape being drawn
    if *edit_mode == TilesetEditMode::Collision {
        if let Some(sel_tile) = selected_tile {
            if sel_tile >= first_tile_index && sel_tile < first_tile_index + rows * columns {
                let local_index = sel_tile - first_tile_index;
                let tile_row = local_index / columns;
                let tile_col = local_index % columns;
                let tile_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(tile_col as f32 * tile_size.x, tile_row as f32 * tile_size.y),
                    tile_size,
                );

                // Draw polygon points if in polygon mode
                let polygon_points = editor_state.tilesets.collision_editor.polygon_points.clone();
                if !polygon_points.is_empty() {
                    draw_polygon_preview(&painter, tile_rect, &polygon_points, zoom);
                }

                // Draw drag preview for rectangle/ellipse
                if let Some(drag_start) = editor_state.tilesets.collision_editor.drag_start {
                    if let Some(current_pos) = ui.ctx().pointer_hover_pos() {
                        if tile_rect.contains(current_pos) {
                            let current_local = (
                                (current_pos.x - tile_rect.min.x) / zoom,
                                (current_pos.y - tile_rect.min.y) / zoom,
                            );
                            draw_shape_preview(
                                &painter,
                                tile_rect,
                                drag_start,
                                current_local,
                                editor_state.tilesets.collision_editor.drawing_shape,
                                zoom,
                            );
                        }
                    }
                }
            }
        }
    }

    // Handle clicks and drags based on edit mode
    let drawing_shape = editor_state.tilesets.collision_editor.drawing_shape;

    // Handle drag start
    if response.drag_started() && *edit_mode == TilesetEditMode::Collision {
        if let Some(pos) = response.interact_pointer_pos() {
            if let Some(sel_tile) = selected_tile {
                if sel_tile >= first_tile_index && sel_tile < first_tile_index + rows * columns {
                    let local_index = sel_tile - first_tile_index;
                    let tile_row = local_index / columns;
                    let tile_col = local_index % columns;
                    let tile_rect = egui::Rect::from_min_size(
                        rect.min + egui::vec2(tile_col as f32 * tile_size.x, tile_row as f32 * tile_size.y),
                        tile_size,
                    );

                    if tile_rect.contains(pos) {
                        // Start drag for rectangle/ellipse
                        if matches!(drawing_shape, Some(CollisionShapeType::Rectangle) | Some(CollisionShapeType::Ellipse)) {
                            let local_pos = (
                                (pos.x - tile_rect.min.x) / zoom,
                                (pos.y - tile_rect.min.y) / zoom,
                            );
                            editor_state.tilesets.collision_editor.drag_start = Some(local_pos);
                        }
                    }
                }
            }
        }
    }

    // Handle drag end - create the shape
    if response.drag_stopped() && *edit_mode == TilesetEditMode::Collision {
        if let Some(drag_start) = editor_state.tilesets.collision_editor.drag_start.take() {
            if let Some(pos) = response.interact_pointer_pos() {
                if let Some(sel_tile) = selected_tile {
                    if sel_tile >= first_tile_index && sel_tile < first_tile_index + rows * columns {
                        let local_index = sel_tile - first_tile_index;
                        let tile_row = local_index / columns;
                        let tile_col = local_index % columns;
                        let tile_rect = egui::Rect::from_min_size(
                            rect.min + egui::vec2(tile_col as f32 * tile_size.x, tile_row as f32 * tile_size.y),
                            tile_size,
                        );

                        let end_pos = (
                            ((pos.x - tile_rect.min.x) / zoom).clamp(0.0, tile_width as f32),
                            ((pos.y - tile_rect.min.y) / zoom).clamp(0.0, tile_height as f32),
                        );

                        // Create the shape
                        create_collision_shape(
                            editor_state,
                            sel_tile,
                            drawing_shape,
                            drag_start,
                            end_pos,
                            tile_width as f32,
                            tile_height as f32,
                        );
                    }
                }
            }
        }
    }

    // Handle regular clicks
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let local_pos = pos - rect.min;
            let col = (local_pos.x / tile_size.x) as u32;
            let row = (local_pos.y / tile_size.y) as u32;

            if col < columns && row < rows {
                let tile_index = first_tile_index + row * columns + col;

                match edit_mode {
                    TilesetEditMode::Terrain => {
                        // In terrain mode, detect which corner was clicked
                        let tile_rect = egui::Rect::from_min_size(
                            rect.min + egui::vec2(col as f32 * tile_size.x, row as f32 * tile_size.y),
                            tile_size,
                        );

                        if let Some(corner_idx) = detect_corner_click(pos, tile_rect) {
                            // Assign terrain to the clicked corner
                            assign_terrain_to_corner(
                                editor_state,
                                tile_index,
                                corner_idx,
                            );
                        }

                        // Also select the tile
                        editor_state.tilesets.selected_tile = Some(tile_index);
                    }
                    TilesetEditMode::Collision => {
                        // First, select the tile if clicking on a different tile
                        if selected_tile != Some(tile_index) {
                            editor_state.tilesets.selected_tile = Some(tile_index);
                            // Clear polygon points when selecting a new tile
                            editor_state.tilesets.collision_editor.polygon_points.clear();
                        } else {
                            // Clicking on the selected tile - handle shape creation
                            let tile_rect = egui::Rect::from_min_size(
                                rect.min + egui::vec2(col as f32 * tile_size.x, row as f32 * tile_size.y),
                                tile_size,
                            );

                            let click_local = (
                                (pos.x - tile_rect.min.x) / zoom,
                                (pos.y - tile_rect.min.y) / zoom,
                            );

                            match drawing_shape {
                                Some(CollisionShapeType::Point) => {
                                    // Create point immediately
                                    add_point_shape(editor_state, tile_index, click_local);
                                }
                                Some(CollisionShapeType::Polygon) => {
                                    // Add point to polygon
                                    editor_state.tilesets.collision_editor.polygon_points.push(click_local);
                                }
                                _ => {
                                    // Try to select existing shape
                                    select_shape_at_pos(editor_state, tile_index, click_local, zoom);
                                }
                            }
                        }
                    }
                    _ => {
                        // In other modes, just select the tile
                        editor_state.tilesets.selected_tile = Some(tile_index);
                    }
                }
            }
        }
    }

    // Handle double-click to finish polygon
    if response.double_clicked() && *edit_mode == TilesetEditMode::Collision {
        if drawing_shape == Some(CollisionShapeType::Polygon) {
            if let Some(sel_tile) = selected_tile {
                finish_polygon(editor_state, sel_tile);
            }
        }
    }

    // Handle right-click to finish polygon or cancel
    if response.secondary_clicked() && *edit_mode == TilesetEditMode::Collision {
        if drawing_shape == Some(CollisionShapeType::Polygon) {
            if let Some(sel_tile) = selected_tile {
                if editor_state.tilesets.collision_editor.polygon_points.len() >= 3 {
                    finish_polygon(editor_state, sel_tile);
                } else {
                    // Cancel - not enough points
                    editor_state.tilesets.collision_editor.polygon_points.clear();
                }
            }
        }
    }

    // Show tooltip on hover
    if response.hovered() {
        if let Some(pos) = ui.ctx().pointer_hover_pos() {
            if rect.contains(pos) {
                let local_pos = pos - rect.min;
                let col = (local_pos.x / tile_size.x) as u32;
                let row = (local_pos.y / tile_size.y) as u32;

                if col < columns && row < rows {
                    let tile_index = first_tile_index + row * columns + col;

                    // In terrain mode, show corner info in tooltip
                    let extra_info = if *edit_mode == TilesetEditMode::Terrain {
                        let tile_rect = egui::Rect::from_min_size(
                            rect.min + egui::vec2(col as f32 * tile_size.x, row as f32 * tile_size.y),
                            tile_size,
                        );
                        detect_corner_click(pos, tile_rect)
                            .map(|c| format!("\nCorner: {}", corner_name(c)))
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };

                    #[allow(deprecated)]
                    egui::show_tooltip(ui.ctx(), ui.layer_id(), egui::Id::new("tile_tooltip"), |ui| {
                        ui.label(format!("Tile #{}", tile_index));
                        ui.label(format!("Position: ({}, {}){}", col, row, extra_info));
                    });
                }
            }
        }
    }
}

/// Draw terrain corner indicators on a tile
fn draw_terrain_corners(
    painter: &egui::Painter,
    tile_rect: egui::Rect,
    corners: &[Option<usize>; 4],
    terrain_colors: &[[u8; 4]],
) {
    let corner_size = (tile_rect.width() * 0.3).min(tile_rect.height() * 0.3);

    // Draw corner triangles: NW, NE, SW, SE
    let corner_positions = [
        (tile_rect.left_top(), egui::vec2(corner_size, 0.0), egui::vec2(0.0, corner_size)),        // NW
        (tile_rect.right_top(), egui::vec2(-corner_size, 0.0), egui::vec2(0.0, corner_size)),     // NE
        (tile_rect.left_bottom(), egui::vec2(corner_size, 0.0), egui::vec2(0.0, -corner_size)),   // SW
        (tile_rect.right_bottom(), egui::vec2(-corner_size, 0.0), egui::vec2(0.0, -corner_size)), // SE
    ];

    for (i, &corner_terrain) in corners.iter().enumerate() {
        if let Some(terrain_idx) = corner_terrain {
            if let Some(color) = terrain_colors.get(terrain_idx) {
                let (corner_pos, dx, dy) = corner_positions[i];
                let fill_color = egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], 180);

                // Draw a filled triangle at the corner
                painter.add(egui::Shape::convex_polygon(
                    vec![
                        corner_pos,
                        corner_pos + dx,
                        corner_pos + dy,
                    ],
                    fill_color,
                    egui::Stroke::NONE,
                ));
            }
        }
    }
}

/// Draw collision shapes on a tile
fn draw_collision_shapes(
    painter: &egui::Painter,
    tile_rect: egui::Rect,
    shapes: &[crate::editor_state::CollisionShape],
    zoom: f32,
) {
    let stroke = egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(255, 100, 100, 200));
    let fill = egui::Color32::from_rgba_unmultiplied(255, 100, 100, 50);

    for shape in shapes {
        match shape {
            crate::editor_state::CollisionShape::Rectangle { x, y, width, height } => {
                let shape_rect = egui::Rect::from_min_size(
                    tile_rect.min + egui::vec2(*x * zoom, *y * zoom),
                    egui::vec2(*width * zoom, *height * zoom),
                );
                painter.rect_filled(shape_rect, 0.0, fill);
                painter.rect_stroke(shape_rect, 0.0, stroke, egui::StrokeKind::Inside);
            }
            crate::editor_state::CollisionShape::Polygon { points } => {
                if points.len() >= 3 {
                    let scaled_points: Vec<egui::Pos2> = points
                        .iter()
                        .map(|(px, py)| tile_rect.min + egui::vec2(*px * zoom, *py * zoom))
                        .collect();
                    painter.add(egui::Shape::convex_polygon(scaled_points.clone(), fill, stroke));
                }
            }
            crate::editor_state::CollisionShape::Ellipse { x, y, width, height } => {
                let center = tile_rect.min + egui::vec2(*x * zoom + *width * zoom / 2.0, *y * zoom + *height * zoom / 2.0);
                let radius = egui::vec2(*width * zoom / 2.0, *height * zoom / 2.0);
                // Draw ellipse using Shape (egui doesn't have ellipse methods on Painter)
                painter.add(egui::Shape::ellipse_filled(center, radius, fill));
                painter.add(egui::Shape::ellipse_stroke(center, radius, stroke));
            }
            crate::editor_state::CollisionShape::Point { x, y, .. } => {
                let pos = tile_rect.min + egui::vec2(*x * zoom, *y * zoom);
                painter.circle_filled(pos, 3.0, egui::Color32::from_rgb(255, 100, 100));
            }
        }
    }
}

/// Detect which corner of a tile was clicked (NW=0, NE=1, SW=2, SE=3)
fn detect_corner_click(click_pos: egui::Pos2, tile_rect: egui::Rect) -> Option<usize> {
    let center = tile_rect.center();
    let in_left = click_pos.x < center.x;
    let in_top = click_pos.y < center.y;

    Some(match (in_left, in_top) {
        (true, true) => 0,   // NW
        (false, true) => 1,  // NE
        (true, false) => 2,  // SW
        (false, false) => 3, // SE
    })
}

/// Get corner name from index
fn corner_name(corner_idx: usize) -> &'static str {
    match corner_idx {
        0 => "NW",
        1 => "NE",
        2 => "SW",
        3 => "SE",
        _ => "??",
    }
}

/// Assign the selected terrain to a tile corner
fn assign_terrain_to_corner(
    editor_state: &mut EditorState,
    tile_index: u32,
    corner_idx: usize,
) {
    let Some(tileset_idx) = editor_state.tilesets.selected_tileset else {
        return;
    };

    let selected_terrain = editor_state.tilesets.selected_terrain;

    if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_idx) {
        let metadata = tileset.tile_metadata.entry(tile_index).or_insert_with(TileMetadata::default);
        let corners = metadata.terrain_corners.get_or_insert([None; 4]);
        corners[corner_idx] = selected_terrain;
    }
}

/// Render a single image as a tile
fn render_single_image_tile(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    path: &str,
    name: &str,
    tile_index: u32,
    zoom: f32,
    show_grid: bool,
    selected_tile: Option<u32>,
) {
    let tile_size = egui::vec2(32.0 * zoom, 32.0 * zoom);

    let (response, painter) = ui.allocate_painter(tile_size, egui::Sense::click());

    let rect = response.rect;

    // Check if we have the texture
    let texture_id = editor_state.world.tile_palette.tileset_egui_ids.get(path).copied();

    if let Some(tex_id) = texture_id {
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.image(tex_id, rect, uv, egui::Color32::WHITE);
    } else {
        // Draw placeholder
        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(60, 80, 60));
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            name,
            egui::FontId::proportional(10.0),
            egui::Color32::WHITE,
        );
    }

    // Grid
    if show_grid {
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, 100)),
            egui::StrokeKind::Outside,
        );
    }

    // Selection
    if selected_tile == Some(tile_index) {
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(2.0, egui::Color32::YELLOW),
            egui::StrokeKind::Outside,
        );
    }

    // Click to select
    if response.clicked() {
        editor_state.tilesets.selected_tile = Some(tile_index);
    }

    // Tooltip
    response.on_hover_text(format!("{} (Tile #{})", name, tile_index));
}

// === Collision Shape Drawing Helper Functions ===

/// Draw a preview of polygon points being placed
fn draw_polygon_preview(
    painter: &egui::Painter,
    tile_rect: egui::Rect,
    points: &[(f32, f32)],
    zoom: f32,
) {
    let stroke = egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(100, 255, 100, 200));
    let point_color = egui::Color32::from_rgb(100, 255, 100);

    // Draw lines between points
    let screen_points: Vec<egui::Pos2> = points
        .iter()
        .map(|(x, y)| tile_rect.min + egui::vec2(*x * zoom, *y * zoom))
        .collect();

    for i in 0..screen_points.len() {
        let next = (i + 1) % screen_points.len();
        if next != 0 || screen_points.len() >= 3 {
            // Only close the polygon if we have 3+ points
            if next != 0 {
                painter.line_segment([screen_points[i], screen_points[next]], stroke);
            }
        }
    }

    // Draw points
    for pt in &screen_points {
        painter.circle_filled(*pt, 4.0, point_color);
    }
}

/// Draw a preview of the shape being dragged (rectangle or ellipse)
fn draw_shape_preview(
    painter: &egui::Painter,
    tile_rect: egui::Rect,
    start: (f32, f32),
    end: (f32, f32),
    shape_type: Option<CollisionShapeType>,
    zoom: f32,
) {
    let stroke = egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(100, 255, 100, 200));
    let fill = egui::Color32::from_rgba_unmultiplied(100, 255, 100, 50);

    let min_x = start.0.min(end.0);
    let min_y = start.1.min(end.1);
    let max_x = start.0.max(end.0);
    let max_y = start.1.max(end.1);

    let screen_min = tile_rect.min + egui::vec2(min_x * zoom, min_y * zoom);
    let screen_max = tile_rect.min + egui::vec2(max_x * zoom, max_y * zoom);
    let preview_rect = egui::Rect::from_min_max(screen_min, screen_max);

    match shape_type {
        Some(CollisionShapeType::Rectangle) => {
            painter.rect_filled(preview_rect, 0.0, fill);
            painter.rect_stroke(preview_rect, 0.0, stroke, egui::StrokeKind::Inside);
        }
        Some(CollisionShapeType::Ellipse) => {
            let center = preview_rect.center();
            let radius = egui::vec2(preview_rect.width() / 2.0, preview_rect.height() / 2.0);
            painter.add(egui::Shape::ellipse_filled(center, radius, fill));
            painter.add(egui::Shape::ellipse_stroke(center, radius, stroke));
        }
        _ => {}
    }
}

/// Create a collision shape from drag coordinates
fn create_collision_shape(
    editor_state: &mut EditorState,
    tile_index: u32,
    shape_type: Option<CollisionShapeType>,
    start: (f32, f32),
    end: (f32, f32),
    max_width: f32,
    max_height: f32,
) {
    let Some(tileset_idx) = editor_state.tilesets.selected_tileset else {
        return;
    };

    let min_x = start.0.min(end.0).clamp(0.0, max_width);
    let min_y = start.1.min(end.1).clamp(0.0, max_height);
    let max_x = start.0.max(end.0).clamp(0.0, max_width);
    let max_y = start.1.max(end.1).clamp(0.0, max_height);
    let width = max_x - min_x;
    let height = max_y - min_y;

    // Don't create tiny shapes
    if width < 2.0 || height < 2.0 {
        return;
    }

    let shape = match shape_type {
        Some(CollisionShapeType::Rectangle) => {
            CollisionShape::Rectangle {
                x: min_x,
                y: min_y,
                width,
                height,
            }
        }
        Some(CollisionShapeType::Ellipse) => {
            CollisionShape::Ellipse {
                x: min_x,
                y: min_y,
                width,
                height,
            }
        }
        _ => return,
    };

    if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_idx) {
        let metadata = tileset.tile_metadata.entry(tile_index).or_insert_with(TileMetadata::default);
        metadata.collision_shapes.push(shape);
        editor_state.status_message = format!("Added {:?} shape to tile #{}", shape_type.unwrap(), tile_index);
    }
}

/// Add a point shape at the given position
fn add_point_shape(
    editor_state: &mut EditorState,
    tile_index: u32,
    pos: (f32, f32),
) {
    let Some(tileset_idx) = editor_state.tilesets.selected_tileset else {
        return;
    };

    let shape = CollisionShape::Point {
        x: pos.0,
        y: pos.1,
        name: format!("point_{}", tile_index),
    };

    if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_idx) {
        let metadata = tileset.tile_metadata.entry(tile_index).or_insert_with(TileMetadata::default);
        metadata.collision_shapes.push(shape);
        editor_state.status_message = format!("Added point to tile #{}", tile_index);
    }
}

/// Finish polygon and add it as a collision shape
fn finish_polygon(
    editor_state: &mut EditorState,
    tile_index: u32,
) {
    let Some(tileset_idx) = editor_state.tilesets.selected_tileset else {
        return;
    };

    let points: Vec<(f32, f32)> = editor_state.tilesets.collision_editor.polygon_points.drain(..).collect();

    if points.len() < 3 {
        return;
    }

    let shape = CollisionShape::Polygon { points };

    if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_idx) {
        let metadata = tileset.tile_metadata.entry(tile_index).or_insert_with(TileMetadata::default);
        metadata.collision_shapes.push(shape);
        editor_state.status_message = format!("Added polygon to tile #{}", tile_index);
    }
}

/// Try to select an existing shape at the given position
fn select_shape_at_pos(
    editor_state: &mut EditorState,
    tile_index: u32,
    pos: (f32, f32),
    _zoom: f32,
) {
    let Some(tileset_idx) = editor_state.tilesets.selected_tileset else {
        return;
    };

    let shapes = editor_state
        .world
        .tile_palette
        .tilesets
        .get(tileset_idx)
        .and_then(|ts| ts.tile_metadata.get(&tile_index))
        .map(|m| m.collision_shapes.clone())
        .unwrap_or_default();

    // Find which shape contains the click position
    for (i, shape) in shapes.iter().enumerate() {
        if shape_contains_point(shape, pos) {
            editor_state.tilesets.collision_editor.selected_shape = Some(i);
            return;
        }
    }

    // No shape found, deselect
    editor_state.tilesets.collision_editor.selected_shape = None;
}

/// Check if a shape contains the given point
fn shape_contains_point(shape: &CollisionShape, pos: (f32, f32)) -> bool {
    match shape {
        CollisionShape::Rectangle { x, y, width, height } => {
            pos.0 >= *x && pos.0 <= *x + *width && pos.1 >= *y && pos.1 <= *y + *height
        }
        CollisionShape::Ellipse { x, y, width, height } => {
            // Ellipse equation: ((px - cx) / rx)^2 + ((py - cy) / ry)^2 <= 1
            let cx = *x + *width / 2.0;
            let cy = *y + *height / 2.0;
            let rx = *width / 2.0;
            let ry = *height / 2.0;
            if rx <= 0.0 || ry <= 0.0 {
                return false;
            }
            let dx = (pos.0 - cx) / rx;
            let dy = (pos.1 - cy) / ry;
            dx * dx + dy * dy <= 1.0
        }
        CollisionShape::Polygon { points } => {
            // Point-in-polygon test using ray casting
            if points.len() < 3 {
                return false;
            }
            let mut inside = false;
            let mut j = points.len() - 1;
            for i in 0..points.len() {
                let (xi, yi) = points[i];
                let (xj, yj) = points[j];
                if ((yi > pos.1) != (yj > pos.1))
                    && (pos.0 < (xj - xi) * (pos.1 - yi) / (yj - yi) + xi)
                {
                    inside = !inside;
                }
                j = i;
            }
            inside
        }
        CollisionShape::Point { x, y, .. } => {
            // Points have a small hit radius
            let dx = pos.0 - *x;
            let dy = pos.1 - *y;
            dx * dx + dy * dy <= 16.0 // 4 pixel radius
        }
    }
}
