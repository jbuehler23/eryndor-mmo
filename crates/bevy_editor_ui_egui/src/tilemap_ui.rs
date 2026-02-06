use bevy::prelude::*;
use bevy_editor_foundation::EditorTool;
use bevy_editor_tilemap::PaintTileEvent;
use bevy_editor_tilemap::{
    bucket_fill, paint_line, paint_rectangle, paint_single_tile, paint_stamp, LayerManager,
    PaintMode, TilePainter, TilesetManager,
};
use bevy_egui::EguiContexts;

/// System to handle tile painting. This remains in the UI crate because it
/// depends on egui to determine when the cursor is over UI elements.
pub fn handle_tile_painting(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    tileset_manager: Res<TilesetManager>,
    mut layer_manager: ResMut<LayerManager>,
    mut tile_painter: ResMut<TilePainter>,
    editor_state: Res<bevy_editor_foundation::EditorState>,
    mut contexts: EguiContexts,
    mut paint_events: EventWriter<PaintTileEvent>,
) {
    if editor_state.current_tool != EditorTool::Platform {
        tile_painter.current_pos = None;
        return;
    }

    let Some(ctx) = contexts.ctx_mut().ok() else {
        tile_painter.current_pos = None;
        return;
    };
    if ctx.is_pointer_over_area() {
        tile_painter.current_pos = None;
        return;
    }

    let Some(mouse_world_pos) = get_mouse_world_position(&windows, &camera_q) else {
        tile_painter.current_pos = None;
        return;
    };

    let Some(selected_tile_id) = tileset_manager.get_selected_tile() else {
        tile_painter.current_pos = None;
        return;
    };

    let Some(active_layer) = layer_manager.get_active_layer() else {
        tile_painter.current_pos = None;
        return;
    };

    let grid_size = active_layer.metadata.grid_size as f32;

    let tile_x = (mouse_world_pos.x / grid_size).floor() as i32;
    let tile_y = (mouse_world_pos.y / grid_size).floor() as i32;

    if tile_x < 0
        || tile_y < 0
        || tile_x >= active_layer.metadata.width as i32
        || tile_y >= active_layer.metadata.height as i32
    {
        tile_painter.current_pos = None;
        return;
    }

    let tile_x = tile_x as u32;
    let tile_y = tile_y as u32;

    tile_painter.current_pos = Some((tile_x, tile_y));

    if mouse_button.pressed(MouseButton::Left) {
        match tile_painter.mode {
            PaintMode::Single => {
                if tileset_manager.selected_tiles.len() > 1 {
                    paint_stamp(
                        tile_x,
                        tile_y,
                        &tileset_manager,
                        tile_painter.flip_x,
                        tile_painter.flip_y,
                        &mut layer_manager,
                        &mut paint_events,
                    );
                } else {
                    paint_single_tile(
                        tile_x,
                        tile_y,
                        selected_tile_id,
                        tile_painter.flip_x,
                        tile_painter.flip_y,
                        &mut layer_manager,
                        &mut paint_events,
                    );
                }
            }
            PaintMode::Rectangle => {
                if mouse_button.just_pressed(MouseButton::Left) {
                    tile_painter.drag_start = Some((tile_x, tile_y));
                }
            }
            PaintMode::Line => {
                if mouse_button.just_pressed(MouseButton::Left) {
                    tile_painter.drag_start = Some((tile_x, tile_y));
                }
            }
            PaintMode::BucketFill => {
                if mouse_button.just_pressed(MouseButton::Left) {
                    bucket_fill(
                        tile_x,
                        tile_y,
                        selected_tile_id,
                        tile_painter.flip_x,
                        tile_painter.flip_y,
                        &mut layer_manager,
                        &mut paint_events,
                    );
                }
            }
        }
    }

    if mouse_button.just_released(MouseButton::Left) {
        if let Some((start_x, start_y)) = tile_painter.drag_start {
            match tile_painter.mode {
                PaintMode::Rectangle => {
                    paint_rectangle(
                        start_x,
                        start_y,
                        tile_x,
                        tile_y,
                        selected_tile_id,
                        tile_painter.flip_x,
                        tile_painter.flip_y,
                        &mut layer_manager,
                        &mut paint_events,
                    );
                }
                PaintMode::Line => {
                    paint_line(
                        start_x,
                        start_y,
                        tile_x,
                        tile_y,
                        selected_tile_id,
                        tile_painter.flip_x,
                        tile_painter.flip_y,
                        &mut layer_manager,
                        &mut paint_events,
                    );
                }
                _ => {}
            }

            tile_painter.drag_start = None;
        } else {
            tile_painter.drag_start = None;
        }
    }

    if mouse_button.pressed(MouseButton::Right) {
        layer_manager.remove_tile(tile_x, tile_y);
    }

    if keyboard.just_pressed(KeyCode::KeyX) {
        tile_painter.flip_x = !tile_painter.flip_x;
    }

    if keyboard.just_pressed(KeyCode::KeyY) {
        tile_painter.flip_y = !tile_painter.flip_y;
    }
}

/// System to handle the temporary eyedropper tool.
pub fn handle_eyedropper(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut tileset_manager: ResMut<TilesetManager>,
    layer_manager: Res<LayerManager>,
    mut editor_state: ResMut<bevy_editor_foundation::EditorState>,
    mut tile_painter: ResMut<TilePainter>,
    mut contexts: EguiContexts,
) {
    let is_alt_held = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);
    let is_eyedropper_active = editor_state.current_tool == EditorTool::Eyedropper || is_alt_held;

    if !is_eyedropper_active {
        return;
    }

    let Some(ctx) = contexts.ctx_mut().ok() else {
        return;
    };
    if ctx.is_pointer_over_area() {
        return;
    }

    let Some(mouse_world_pos) = get_mouse_world_position(&windows, &camera_q) else {
        return;
    };

    let Some(active_layer) = layer_manager.get_active_layer() else {
        return;
    };

    let grid_size = active_layer.metadata.grid_size as f32;

    let tile_x = (mouse_world_pos.x / grid_size).floor() as i32;
    let tile_y = (mouse_world_pos.y / grid_size).floor() as i32;

    if tile_x < 0
        || tile_y < 0
        || tile_x >= active_layer.metadata.width as i32
        || tile_y >= active_layer.metadata.height as i32
    {
        return;
    }

    let tile_x = tile_x as u32;
    let tile_y = tile_y as u32;

    if mouse_button.just_pressed(MouseButton::Left) {
        if let Some(tile_data) = layer_manager.get_tile_at(tile_x, tile_y) {
            tileset_manager.selected_tile_id = Some(tile_data.tile_id);

            if is_alt_held && editor_state.current_tool != EditorTool::Eyedropper {
                // temporary eyedropper, keep current tool
            } else {
                editor_state.current_tool = EditorTool::Platform;
                tile_painter.mode = PaintMode::Single;
            }
        }
    }
}

fn get_mouse_world_position(
    windows: &Query<&Window>,
    camera_q: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.iter().next()?;
    let (camera, camera_transform) = camera_q.iter().next()?;
    window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
}
