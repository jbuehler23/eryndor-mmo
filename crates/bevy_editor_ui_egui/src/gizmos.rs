use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use bevy_editor_core::{GizmoMode, GizmoState};
use bevy_editor_foundation::{EditorState, EditorTool};
use bevy_editor_scene::{EditorScene, EditorSceneEntity};
use bevy_editor_tilemap::{PaintMode, TilePainter};

fn gizmo_mode_display_name(mode: GizmoMode) -> &'static str {
    match mode {
        GizmoMode::Move => "Move (Q)",
        GizmoMode::Rotate => "Rotate (W)",
        GizmoMode::Scale => "Scale (E)",
    }
}

fn gizmo_mode_icon(mode: GizmoMode) -> &'static str {
    match mode {
        GizmoMode::Move => "↔",   // Arrows
        GizmoMode::Rotate => "↻", // Rotation arrow
        GizmoMode::Scale => "⤢",  // Diagonal arrows
    }
}

/// Draw grid in the viewport
pub fn draw_grid(
    mut gizmos: Gizmos,
    editor_state: Res<EditorState>,
    camera_q: Query<&Transform, With<Camera2d>>,
) {
    if !editor_state.grid_snap_enabled {
        return;
    }

    let Some(camera_transform) = camera_q.iter().next() else {
        return;
    };

    let grid_size = editor_state.grid_size;
    let camera_pos = camera_transform.translation.xy();

    // Draw grid lines in view
    let grid_extent = 2000.0; // How far to draw grid

    // Vertical lines
    let start_x = ((camera_pos.x - grid_extent) / grid_size).floor() * grid_size;
    let end_x = camera_pos.x + grid_extent;
    let mut x = start_x;
    while x <= end_x {
        gizmos.line_2d(
            Vec2::new(x, camera_pos.y - grid_extent),
            Vec2::new(x, camera_pos.y + grid_extent),
            Color::srgba(1.0, 1.0, 1.0, 0.1),
        );
        x += grid_size;
    }

    // Horizontal lines
    let start_y = ((camera_pos.y - grid_extent) / grid_size).floor() * grid_size;
    let end_y = camera_pos.y + grid_extent;
    let mut y = start_y;
    while y <= end_y {
        gizmos.line_2d(
            Vec2::new(camera_pos.x - grid_extent, y),
            Vec2::new(camera_pos.x + grid_extent, y),
            Color::srgba(1.0, 1.0, 1.0, 0.1),
        );
        y += grid_size;
    }

    // Draw origin lines
    gizmos.line_2d(
        Vec2::new(0.0, camera_pos.y - grid_extent),
        Vec2::new(0.0, camera_pos.y + grid_extent),
        Color::srgba(0.0, 1.0, 0.0, 0.3),
    );
    gizmos.line_2d(
        Vec2::new(camera_pos.x - grid_extent, 0.0),
        Vec2::new(camera_pos.x + grid_extent, 0.0),
        Color::srgba(1.0, 0.0, 0.0, 0.3),
    );
}

/// Draw selection highlights and move handles
pub fn draw_selection_gizmos(
    mut gizmos: Gizmos,
    editor_scene: Res<EditorScene>,
    scene_entities: Query<
        (&Transform, Option<&Sprite>, Option<&Node>, Option<&Text>),
        With<EditorSceneEntity>,
    >,
    images: Res<Assets<Image>>,
    gizmo_state: Res<GizmoState>,
) {
    // Draw gizmos for new scene editor entities
    if let Some(selected_entity) = editor_scene.selected_entity {
        if let Ok((transform, sprite, node, text)) = scene_entities.get(selected_entity) {
            let pos = transform.translation.xy();
            let scale = transform.scale.xy();

            // Calculate bounds based on component type
            let bounds = if let Some(sprite) = sprite {
                // Get sprite size from custom_size or texture dimensions
                let base_size = if let Some(custom_size) = sprite.custom_size {
                    custom_size
                } else if let Some(image) = images.get(&sprite.image) {
                    image.size().as_vec2()
                } else {
                    // Default size for sprites without texture
                    Vec2::new(64.0, 64.0)
                };

                // Apply scale to base size
                base_size * scale
            } else if let Some(_node) = node {
                // UI Node size - use fixed size for now
                Vec2::new(100.0, 100.0) * scale
            } else if let Some(_text) = text {
                // Text size - use fixed size for now
                Vec2::new(100.0, 32.0) * scale
            } else {
                // Default bounds for empty entities
                Vec2::new(32.0, 32.0) * scale
            };

            // Draw bounds rectangle
            let half_size = bounds / 2.0;
            let corners = [
                pos + Vec2::new(-half_size.x, -half_size.y),
                pos + Vec2::new(half_size.x, -half_size.y),
                pos + Vec2::new(half_size.x, half_size.y),
                pos + Vec2::new(-half_size.x, half_size.y),
            ];

            // Draw selection rectangle
            for i in 0..4 {
                gizmos.line_2d(corners[i], corners[(i + 1) % 4], Color::srgb(1.0, 1.0, 0.0));
            }

            // Draw appropriate gizmo handles based on mode
            match gizmo_state.mode {
                GizmoMode::Move => draw_move_handles(&mut gizmos, pos),
                GizmoMode::Rotate => draw_rotation_handles(&mut gizmos, pos, bounds),
                GizmoMode::Scale => draw_scale_handles(&mut gizmos, pos, bounds),
            }
        }
    }
}

fn draw_move_handles(gizmos: &mut Gizmos, position: Vec2) {
    let handle_size = 8.0;

    // Center handle
    gizmos.circle_2d(position, handle_size, Color::srgb(0.0, 1.0, 0.0));

    // Axis handles
    // X-axis (red)
    gizmos.line_2d(
        position,
        position + Vec2::new(30.0, 0.0),
        Color::srgb(1.0, 0.0, 0.0),
    );
    gizmos.circle_2d(
        position + Vec2::new(30.0, 0.0),
        handle_size * 0.7,
        Color::srgb(1.0, 0.0, 0.0),
    );

    // Y-axis (green)
    gizmos.line_2d(
        position,
        position + Vec2::new(0.0, 30.0),
        Color::srgb(0.0, 1.0, 0.0),
    );
    gizmos.circle_2d(
        position + Vec2::new(0.0, 30.0),
        handle_size * 0.7,
        Color::srgb(0.0, 1.0, 0.0),
    );
}

/// Draw rotation handles (circular arc with handle)
fn draw_rotation_handles(gizmos: &mut Gizmos, position: Vec2, bounds: Vec2) {
    let radius = (bounds.length() / 2.0) + 20.0; // Offset from bounds
    let handle_size = 8.0;

    // Draw rotation circle
    gizmos.circle_2d(position, radius, Color::srgba(0.3, 0.7, 1.0, 0.6));

    // Draw rotation handles at cardinal directions
    let handle_positions = [
        position + Vec2::new(radius, 0.0),  // Right
        position + Vec2::new(0.0, radius),  // Top
        position + Vec2::new(-radius, 0.0), // Left
        position + Vec2::new(0.0, -radius), // Bottom
    ];

    for handle_pos in handle_positions {
        gizmos.circle_2d(handle_pos, handle_size, Color::srgb(0.3, 0.7, 1.0));
    }

    // Center handle
    gizmos.circle_2d(position, handle_size * 0.5, Color::srgb(1.0, 1.0, 1.0));
}

/// Draw scale handles (squares at corners and edges)
fn draw_scale_handles(gizmos: &mut Gizmos, position: Vec2, bounds: Vec2) {
    let half_size = bounds / 2.0;
    let handle_size = 6.0;

    // Corner handles (yellow - for uniform and non-uniform scale)
    let corners = [
        position + Vec2::new(-half_size.x, -half_size.y),
        position + Vec2::new(half_size.x, -half_size.y),
        position + Vec2::new(half_size.x, half_size.y),
        position + Vec2::new(-half_size.x, half_size.y),
    ];

    for corner in corners {
        // Draw square handle
        let half_handle = handle_size / 2.0;
        let handle_corners = [
            corner + Vec2::new(-half_handle, -half_handle),
            corner + Vec2::new(half_handle, -half_handle),
            corner + Vec2::new(half_handle, half_handle),
            corner + Vec2::new(-half_handle, half_handle),
        ];

        for i in 0..4 {
            gizmos.line_2d(
                handle_corners[i],
                handle_corners[(i + 1) % 4],
                Color::srgb(1.0, 1.0, 0.0),
            );
        }
    }

    // Edge handles (cyan - for axis-aligned scaling)
    let edges = [
        position + Vec2::new(0.0, -half_size.y), // Bottom
        position + Vec2::new(half_size.x, 0.0),  // Right
        position + Vec2::new(0.0, half_size.y),  // Top
        position + Vec2::new(-half_size.x, 0.0), // Left
    ];

    for edge in edges {
        gizmos.circle_2d(edge, handle_size * 0.7, Color::srgb(0.0, 1.0, 1.0));
    }

    // Center handle
    gizmos.circle_2d(position, handle_size * 0.5, Color::srgb(1.0, 1.0, 1.0));
}

/// Draw preview for rectangle and line tile tools
pub fn draw_tile_tool_preview(
    mut gizmos: Gizmos,
    editor_state: Res<EditorState>,
    tile_painter: Res<TilePainter>,
    tileset_manager: Res<bevy_editor_tilemap::TilesetManager>,
) {
    // Only show preview for tile painting tools
    if editor_state.current_tool != EditorTool::Platform {
        return;
    }

    let grid_size = editor_state.grid_size;

    // Draw stamp preview if in stamp mode (multi-tile selection)
    // Only show when NOT actively dragging (Rectangle/Line tools use drag_start)
    if tile_painter.mode == PaintMode::Single
        && tileset_manager.selected_tiles.len() > 1
        && tile_painter.drag_start.is_none()
    {
        // Prevent conflict with drag tools

        if let Some((cursor_x, cursor_y)) = tile_painter.current_pos {
            if let Some((stamp_width, stamp_height)) = tileset_manager.get_selection_dimensions() {
                let preview_color = Color::srgba(0.0, 1.0, 1.0, 0.5); // Cyan semi-transparent

                // Draw preview for each tile in the stamp
                for offset_y in 0..stamp_height {
                    for offset_x in 0..stamp_width {
                        let world_x = (cursor_x + offset_x) as f32 * grid_size;
                        let world_y = (cursor_y + offset_y) as f32 * grid_size;

                        // Draw tile outline
                        let corners = [
                            Vec2::new(world_x, world_y),
                            Vec2::new(world_x + grid_size, world_y),
                            Vec2::new(world_x + grid_size, world_y + grid_size),
                            Vec2::new(world_x, world_y + grid_size),
                        ];

                        for i in 0..4 {
                            gizmos.line_2d(corners[i], corners[(i + 1) % 4], preview_color);
                        }
                    }
                }
            }
        }
    }

    // Draw rectangle/line preview ONLY when in those modes AND actively dragging
    match tile_painter.mode {
        PaintMode::Rectangle | PaintMode::Line => {
            // Show preview when dragging
            if let (Some((start_x, start_y)), Some((end_x, end_y))) =
                (tile_painter.drag_start, tile_painter.current_pos)
            {
                // Convert tile coords to world coords
                let start_world = Vec2::new(start_x as f32 * grid_size, start_y as f32 * grid_size);
                let end_world = Vec2::new(end_x as f32 * grid_size, end_y as f32 * grid_size);

                let preview_color = Color::srgba(0.0, 1.0, 1.0, 0.5); // Cyan semi-transparent

                match tile_painter.mode {
                    PaintMode::Rectangle => {
                        // Draw rectangle preview
                        let min_x = start_world.x.min(end_world.x);
                        let max_x = start_world.x.max(end_world.x) + grid_size;
                        let min_y = start_world.y.min(end_world.y);
                        let max_y = start_world.y.max(end_world.y) + grid_size;

                        // Draw filled rectangle using lines
                        let corners = [
                            Vec2::new(min_x, min_y),
                            Vec2::new(max_x, min_y),
                            Vec2::new(max_x, max_y),
                            Vec2::new(min_x, max_y),
                        ];

                        // Draw outline
                        for i in 0..4 {
                            gizmos.line_2d(corners[i], corners[(i + 1) % 4], preview_color);
                        }

                        // Draw diagonals to show it's filled
                        gizmos.line_2d(corners[0], corners[2], Color::srgba(0.0, 1.0, 1.0, 0.2));
                        gizmos.line_2d(corners[1], corners[3], Color::srgba(0.0, 1.0, 1.0, 0.2));
                    }
                    PaintMode::Line => {
                        // Draw line preview using Bresenham
                        let tiles = calculate_line_tiles(start_x, start_y, end_x, end_y);

                        for (tile_x, tile_y) in tiles {
                            let world_x = tile_x as f32 * grid_size;
                            let world_y = tile_y as f32 * grid_size;

                            // Draw tile outline
                            let corners = [
                                Vec2::new(world_x, world_y),
                                Vec2::new(world_x + grid_size, world_y),
                                Vec2::new(world_x + grid_size, world_y + grid_size),
                                Vec2::new(world_x, world_y + grid_size),
                            ];

                            for i in 0..4 {
                                gizmos.line_2d(corners[i], corners[(i + 1) % 4], preview_color);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

/// Calculate tiles along a line using Bresenham's algorithm
fn calculate_line_tiles(start_x: u32, start_y: u32, end_x: u32, end_y: u32) -> Vec<(u32, u32)> {
    let mut tiles = Vec::new();

    let dx = (end_x as i32 - start_x as i32).abs();
    let dy = (end_y as i32 - start_y as i32).abs();
    let sx = if start_x < end_x { 1 } else { -1 };
    let sy = if start_y < end_y { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = start_x as i32;
    let mut y = start_y as i32;

    loop {
        tiles.push((x as u32, y as u32));

        if x == end_x as i32 && y == end_y as i32 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }

    tiles
}

/// Draw selection highlights for EditorSceneEntity (new scene editor system)
pub fn draw_scene_entity_gizmos(
    mut gizmos: Gizmos,
    editor_scene: Res<EditorScene>,
    scene_entities: Query<
        (&GlobalTransform, Option<&Sprite>, Option<&Name>),
        With<EditorSceneEntity>,
    >,
) {
    // Draw highlight for selected entity
    if let Some(selected_entity) = editor_scene.selected_entity {
        if let Ok((transform, sprite, name)) = scene_entities.get(selected_entity) {
            let pos = transform.translation().truncate();

            // Calculate bounds based on sprite size or default
            let size = if let Some(sprite_comp) = sprite {
                sprite_comp.custom_size.unwrap_or(Vec2::new(32.0, 32.0))
            } else {
                Vec2::new(32.0, 32.0) // Default size for entities without sprites
            };

            let half_size = size / 2.0;

            // Draw selection rectangle
            let corners = [
                pos + Vec2::new(-half_size.x, -half_size.y),
                pos + Vec2::new(half_size.x, -half_size.y),
                pos + Vec2::new(half_size.x, half_size.y),
                pos + Vec2::new(-half_size.x, half_size.y),
            ];

            // Draw yellow outline
            for i in 0..4 {
                gizmos.line_2d(corners[i], corners[(i + 1) % 4], Color::srgb(1.0, 1.0, 0.0));
            }

            // Draw move gizmo handles
            draw_scene_move_handles(&mut gizmos, pos);

            // Draw entity name above if available
            if let Some(_entity_name) = name {
                // Draw a small indicator where the name would be
                // (egui text rendering would be needed for actual text)
                let name_pos = pos + Vec2::new(0.0, half_size.y + 10.0);
                gizmos.circle_2d(name_pos, 2.0, Color::srgb(1.0, 1.0, 1.0));
            }
        }
    }

    // Draw subtle outlines for all scene entities (not selected)
    for (transform, sprite, _) in scene_entities.iter() {
        let pos = transform.translation().truncate();

        // Skip if this is the selected entity
        if Some(Entity::PLACEHOLDER) != editor_scene.selected_entity {
            // Calculate bounds
            let size = if let Some(sprite_comp) = sprite {
                sprite_comp.custom_size.unwrap_or(Vec2::new(32.0, 32.0))
            } else {
                Vec2::new(32.0, 32.0)
            };

            let half_size = size / 2.0;

            // Draw subtle gray outline for unselected entities
            let corners = [
                pos + Vec2::new(-half_size.x, -half_size.y),
                pos + Vec2::new(half_size.x, -half_size.y),
                pos + Vec2::new(half_size.x, half_size.y),
                pos + Vec2::new(-half_size.x, half_size.y),
            ];

            for i in 0..4 {
                gizmos.line_2d(
                    corners[i],
                    corners[(i + 1) % 4],
                    Color::srgba(0.7, 0.7, 0.7, 0.3),
                );
            }
        }
    }
}

/// Draw move/transform handles for scene entities
fn draw_scene_move_handles(gizmos: &mut Gizmos, position: Vec2) {
    let handle_size = 6.0;
    let axis_length = 40.0;

    // Center handle (white)
    gizmos.circle_2d(position, handle_size, Color::srgb(1.0, 1.0, 1.0));

    // X-axis handle (red)
    gizmos.line_2d(
        position,
        position + Vec2::new(axis_length, 0.0),
        Color::srgb(1.0, 0.0, 0.0),
    );
    gizmos.circle_2d(
        position + Vec2::new(axis_length, 0.0),
        handle_size * 0.8,
        Color::srgb(1.0, 0.0, 0.0),
    );

    // Y-axis handle (green)
    gizmos.line_2d(
        position,
        position + Vec2::new(0.0, axis_length),
        Color::srgb(0.0, 1.0, 0.0),
    );
    gizmos.circle_2d(
        position + Vec2::new(0.0, axis_length),
        handle_size * 0.8,
        Color::srgb(0.0, 1.0, 0.0),
    );
}

/// Draw gizmo mode indicator overlay in viewport
pub fn draw_gizmo_mode_indicator(mut contexts: EguiContexts, gizmo_state: Res<GizmoState>) {
    let Some(ctx) = contexts.ctx_mut().ok() else {
        return;
    };

    // Draw in top-left of viewport, accounting for left panel width
    // Position: after left panel (~250px) + some margin, below toolbar (~45px)
    egui::Area::new(egui::Id::new("gizmo_mode_indicator"))
        .anchor(egui::Align2::LEFT_TOP, [260.0, 45.0])
        .show(ctx, |ui| {
            egui::Frame::default()
                .fill(egui::Color32::from_rgba_unmultiplied(30, 30, 30, 180))
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(8, 6))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 3.0;

                        // Current mode - highlighted
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(gizmo_mode_icon(gizmo_state.mode))
                                    .size(14.0)
                                    .color(egui::Color32::from_rgb(100, 180, 255)),
                            );
                            ui.label(
                                egui::RichText::new(gizmo_mode_display_name(gizmo_state.mode))
                                    .size(11.0)
                                    .strong()
                                    .color(egui::Color32::from_rgb(220, 220, 220)),
                            );
                        });

                        // Show other mode options in smaller, dimmed text
                        ui.add_space(2.0);

                        for mode in [GizmoMode::Move, GizmoMode::Rotate, GizmoMode::Scale] {
                            if mode != gizmo_state.mode {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(gizmo_mode_icon(mode))
                                            .size(10.0)
                                            .color(egui::Color32::from_rgb(120, 120, 120)),
                                    );
                                    ui.label(
                                        egui::RichText::new(gizmo_mode_display_name(mode))
                                            .size(9.0)
                                            .color(egui::Color32::from_rgb(150, 150, 150)),
                                    );
                                });
                            }
                        }
                    });
                });
        });
}
