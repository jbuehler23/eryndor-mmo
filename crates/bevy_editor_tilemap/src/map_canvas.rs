use crate::{LayerManager, TilesetManager};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

/// Component to mark the tilemap entity for the map canvas
#[derive(Component)]
pub struct MapCanvas {
    pub layer_id: u32,
}

/// Resource to track the current map dimensions
#[derive(Resource)]
pub struct MapDimensions {
    pub width: u32,
    pub height: u32,
}

impl Default for MapDimensions {
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
        }
    }
}

/// Setup the map canvas - creates a simple tilemap for testing
/// This is a simplified version that creates one tilemap if a tileset is loaded
pub fn setup_map_canvas(
    mut commands: Commands,
    tileset_manager: Res<TilesetManager>,
    _layer_manager: Res<LayerManager>,
    map_dimensions: Res<MapDimensions>,
    existing_canvas: Query<Entity, With<MapCanvas>>,
) {
    // Only create if we don't already have a canvas and we have a tileset
    if !existing_canvas.is_empty() {
        return;
    }

    if let Some(tileset_id) = tileset_manager.selected_tileset_id {
        if let Some(tileset_info) = tileset_manager.tilesets.get(&tileset_id) {
            info!(
                "Creating map canvas with tileset '{}'",
                tileset_info.data.identifier
            );

            let map_size = TilemapSize {
                x: map_dimensions.width,
                y: map_dimensions.height,
            };

            let tile_size = TilemapTileSize {
                x: tileset_info.data.tile_width as f32,
                y: tileset_info.data.tile_height as f32,
            };

            let grid_size = TilemapGridSize {
                x: tileset_info.data.tile_width as f32,
                y: tileset_info.data.tile_height as f32,
            };

            let tilemap_entity = commands.spawn_empty().id();
            let mut tile_storage = TileStorage::empty(map_size);

            // Spawn tiles (initially hidden)
            for x in 0..map_size.x {
                for y in 0..map_size.y {
                    let tile_pos = TilePos { x, y };
                    let tile_entity = commands
                        .spawn(TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(tilemap_entity),
                            texture_index: TileTextureIndex(0),
                            visible: TileVisible(false),
                            ..Default::default()
                        })
                        .id();
                    tile_storage.set(&tile_pos, tile_entity);
                }
            }

            // Create tilemap with proper type and positioning
            // Offset by half a tile so tiles are centered on grid intersections
            let half_tile_x = tileset_info.data.tile_width as f32 / 2.0;
            let half_tile_y = tileset_info.data.tile_height as f32 / 2.0;

            commands.entity(tilemap_entity).insert((
                TilemapBundle {
                    grid_size,
                    size: map_size,
                    storage: tile_storage,
                    texture: TilemapTexture::Single(tileset_info.texture_handle.clone()),
                    tile_size,
                    transform: Transform::from_xyz(half_tile_x, half_tile_y, 0.0),
                    map_type: TilemapType::Square,
                    ..Default::default()
                },
                MapCanvas { layer_id: 0 },
            ));

            info!(
                "Created map canvas tilemap: {}x{} tiles",
                map_size.x, map_size.y
            );
        }
    }
}

/// System to update the tilemap when layers change
pub fn update_map_canvas_on_layer_changes(layer_manager: Res<LayerManager>) {
    // Placeholder - would rebuild tilemap when layers change
    if layer_manager.is_changed() {
        // TODO: Implement layer change handling
    }
}

/// Event to paint a tile at a specific position
#[derive(Event, Message)]
pub struct PaintTileEvent {
    pub layer_id: u32,
    pub x: u32,
    pub y: u32,
    pub tile_id: u32,
}

/// System to handle tile painting events
pub fn handle_paint_tile_events(
    mut paint_events: MessageReader<PaintTileEvent>,
    tilemap_query: Query<&TileStorage, With<MapCanvas>>,
    mut tile_query: Query<(&mut TileTextureIndex, &mut TileVisible)>,
) {
    for event in paint_events.read() {
        // Find the tilemap
        for tile_storage in &tilemap_query {
            let tile_pos = TilePos {
                x: event.x,
                y: event.y,
            };

            // Get the tile entity and update it
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                if let Ok((mut texture_index, mut visible)) = tile_query.get_mut(tile_entity) {
                    texture_index.0 = event.tile_id;
                    visible.0 = true;
                    info!(
                        "Painted tile {} at ({}, {})",
                        event.tile_id, event.x, event.y
                    );
                }
            }
        }
    }
}

/// System to handle mouse clicks on the map canvas for painting
pub fn handle_canvas_click_painting(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    tileset_manager: Res<TilesetManager>,
    tilemap_query: Query<
        (&TileStorage, &TilemapSize, &TilemapGridSize, &Transform),
        With<MapCanvas>,
    >,
    mut paint_events: MessageWriter<PaintTileEvent>,
) {
    // Only paint on left click
    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    // Need a selected tile
    let Some(selected_tile) = tileset_manager.selected_tile_id else {
        return;
    };

    // Get cursor position in world space
    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    // Find which tile was clicked
    for (_storage, map_size, grid_size, transform) in &tilemap_query {
        // Convert world position to tile position
        // The tilemap is offset by half a tile, so we need to account for that
        let local_pos = world_pos - transform.translation.truncate();

        // Add half tile offset back to properly center the coordinate system
        let adjusted_x = local_pos.x + (grid_size.x / 2.0);
        let adjusted_y = local_pos.y + (grid_size.y / 2.0);

        let tile_x = (adjusted_x / grid_size.x).floor() as i32;
        let tile_y = (adjusted_y / grid_size.y).floor() as i32;

        // Check bounds
        if tile_x < 0 || tile_y < 0 || tile_x >= map_size.x as i32 || tile_y >= map_size.y as i32 {
            continue;
        }

        // Send paint event
        paint_events.write(PaintTileEvent {
            layer_id: 0,
            x: tile_x as u32,
            y: tile_y as u32,
            tile_id: selected_tile,
        });

        break; // Only paint on first tilemap
    }
}
