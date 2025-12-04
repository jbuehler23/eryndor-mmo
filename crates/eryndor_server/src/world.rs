use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use avian2d::prelude::{RigidBody, Collider, CollisionLayers};
use crate::PhysicsPosition;
use crate::game_data::ZoneDatabase;

/// Marker resource indicating the world has been spawned
#[derive(Resource, Default)]
pub struct WorldSpawned;

/// Run condition: returns true when zone data is loaded and world hasn't been spawned
pub fn zone_data_loaded(
    zone_db: Res<ZoneDatabase>,
    world_spawned: Option<Res<WorldSpawned>>,
) -> bool {
    // Only spawn if zone data exists and world hasn't been spawned yet
    world_spawned.is_none() && zone_db.zones.contains_key("starter_zone")
}

/// System to spawn world boundaries at startup (doesn't depend on JSON data)
pub fn spawn_world_boundaries(mut commands: Commands) {
    info!("Spawning world boundaries...");

    let wall_thickness = 20.0;
    let half_width = WORLD_WIDTH / 2.0;
    let half_height = WORLD_HEIGHT / 2.0;
    let segment_size = 50.0;
    let collider_offset = (segment_size / 2.0) - (wall_thickness / 2.0);

    // North wall (top)
    commands.spawn((
        PhysicsPosition(Vec2::new(0.0, half_height - collider_offset)),
        RigidBody::Static,
        Collider::rectangle(WORLD_WIDTH, wall_thickness),
        CollisionLayers::new(GameLayer::Environment, [GameLayer::Player, GameLayer::Enemy]),
    ));
    let num_segments = (WORLD_WIDTH / segment_size).ceil() as i32;
    for i in 0..num_segments {
        let x = -half_width + (i as f32 * segment_size) + (segment_size / 2.0);
        commands.spawn((
            Replicated,
            Position(Vec2::new(x, half_height)),
            VisualShape {
                shape_type: ShapeType::Square,
                color: [1.0, 0.0, 0.0, 1.0],
                size: segment_size,
            },
        ));
    }

    // South wall (bottom)
    commands.spawn((
        PhysicsPosition(Vec2::new(0.0, -half_height + collider_offset)),
        RigidBody::Static,
        Collider::rectangle(WORLD_WIDTH, wall_thickness),
        CollisionLayers::new(GameLayer::Environment, [GameLayer::Player, GameLayer::Enemy]),
    ));
    for i in 0..num_segments {
        let x = -half_width + (i as f32 * segment_size) + (segment_size / 2.0);
        commands.spawn((
            Replicated,
            Position(Vec2::new(x, -half_height)),
            VisualShape {
                shape_type: ShapeType::Square,
                color: [1.0, 0.0, 0.0, 1.0],
                size: segment_size,
            },
        ));
    }

    // East wall (right)
    commands.spawn((
        PhysicsPosition(Vec2::new(half_width - collider_offset, 0.0)),
        RigidBody::Static,
        Collider::rectangle(wall_thickness, WORLD_HEIGHT),
        CollisionLayers::new(GameLayer::Environment, [GameLayer::Player, GameLayer::Enemy]),
    ));
    let num_segments_vertical = (WORLD_HEIGHT / segment_size).ceil() as i32;
    for i in 0..num_segments_vertical {
        let y = -half_height + (i as f32 * segment_size) + (segment_size / 2.0);
        commands.spawn((
            Replicated,
            Position(Vec2::new(half_width, y)),
            VisualShape {
                shape_type: ShapeType::Square,
                color: [1.0, 0.0, 0.0, 1.0],
                size: segment_size,
            },
        ));
    }

    // West wall (left)
    commands.spawn((
        PhysicsPosition(Vec2::new(-half_width + collider_offset, 0.0)),
        RigidBody::Static,
        Collider::rectangle(wall_thickness, WORLD_HEIGHT),
        CollisionLayers::new(GameLayer::Environment, [GameLayer::Player, GameLayer::Enemy]),
    ));
    for i in 0..num_segments_vertical {
        let y = -half_height + (i as f32 * segment_size) + (segment_size / 2.0);
        commands.spawn((
            Replicated,
            Position(Vec2::new(-half_width, y)),
            VisualShape {
                shape_type: ShapeType::Square,
                color: [1.0, 0.0, 0.0, 1.0],
                size: segment_size,
            },
        ));
    }

    info!("World boundaries spawned");
}

/// Marker component for tilemap collision entities
#[derive(Component)]
pub struct TilemapCollider;

/// System to spawn world entities from zone data (runs when zone data is loaded)
/// Note: Enemy and NPC spawning now handled by Tiled map via tiled_spawner module
pub fn spawn_world(
    mut commands: Commands,
    zone_db: Res<ZoneDatabase>,
) {
    info!("Zone data loaded, spawning tilemap collision...");

    // Mark world as spawned
    commands.insert_resource(WorldSpawned);

    // Spawn collision from zone data (enemies/NPCs now come from Tiled map)
    if let Some(zone) = zone_db.zones.get("starter_zone") {
        spawn_tilemap_collision(&mut commands, zone);
    }

    info!("World initialization complete!");
}


/// Spawn tilemap collision entities from zone tilemap data
/// Supports both new TilemapMap format (priority) and legacy ZoneTilemap format
fn spawn_tilemap_collision(commands: &mut Commands, zone: &crate::game_data::ZoneDefinition) {
    // Try new TilemapMap format first
    if let Some(tilemap) = &zone.tilemap_map {
        spawn_tilemapmap_collision(commands, tilemap, &zone.zone_id);
        return;
    }

    // Fall back to legacy ZoneTilemap format
    if let Some(tilemap) = &zone.tilemap {
        spawn_legacy_tilemap_collision(commands, tilemap, &zone.zone_id);
        return;
    }

    info!("No tilemap data for zone {}, skipping collision spawning", zone.zone_id);
}

/// Spawn collision entities from new TilemapMap format
/// Looks for a layer named "Collision" and treats any non-zero tile as a collision tile
fn spawn_tilemapmap_collision(commands: &mut Commands, tilemap: &eryndor_shared::TilemapMap, zone_id: &str) {
    let tile_size = tilemap.tile_width as f32;
    let chunk_size = 16u32; // Default chunk size for infinite maps
    let mut collision_count = 0;

    // Find the collision layer (case-insensitive search)
    let collision_layer = tilemap.layers.iter().find(|l| {
        l.name.to_lowercase() == "collision" && l.is_tile_layer()
    });

    let Some(layer) = collision_layer else {
        info!("No 'Collision' layer found in TilemapMap for zone {}", zone_id);
        return;
    };

    // Get chunk data from the layer
    if let Some(chunks) = &layer.chunks {
        for chunk in chunks {
            let chunk_world_x = chunk.x;
            let chunk_world_y = chunk.y;
            let chunk_width = chunk.width as i32;

            // Iterate through tiles in this chunk
            for (idx, &gid) in chunk.data.iter().enumerate() {
                if gid == 0 {
                    continue; // No collision for empty tiles
                }

                // Calculate tile position within chunk
                let local_x = (idx as i32) % chunk_width;
                let local_y = (idx as i32) / chunk_width;

                // Calculate world position
                let tile_x = chunk_world_x + local_x;
                let tile_y = chunk_world_y + local_y;
                let world_x = tile_x as f32 * tile_size + (tile_size / 2.0);
                let world_y = tile_y as f32 * tile_size + (tile_size / 2.0);

                // Spawn collision entity
                commands.spawn((
                    TilemapCollider,
                    PhysicsPosition(Vec2::new(world_x, world_y)),
                    RigidBody::Static,
                    Collider::rectangle(tile_size, tile_size),
                    CollisionLayers::new(GameLayer::Environment, [GameLayer::Player, GameLayer::Enemy]),
                ));
                collision_count += 1;
            }
        }
    }

    // Also check for finite map data (non-chunked)
    if let Some(data) = &layer.data {
        let map_width = tilemap.width as i32;
        for (idx, &gid) in data.iter().enumerate() {
            if gid == 0 {
                continue;
            }

            let tile_x = (idx as i32) % map_width;
            let tile_y = (idx as i32) / map_width;
            let world_x = tile_x as f32 * tile_size + (tile_size / 2.0);
            let world_y = tile_y as f32 * tile_size + (tile_size / 2.0);

            commands.spawn((
                TilemapCollider,
                PhysicsPosition(Vec2::new(world_x, world_y)),
                RigidBody::Static,
                Collider::rectangle(tile_size, tile_size),
                CollisionLayers::new(GameLayer::Environment, [GameLayer::Player, GameLayer::Enemy]),
            ));
            collision_count += 1;
        }
    }

    if collision_count > 0 {
        info!("Spawned {} TilemapMap collision entities for zone: {}", collision_count, zone_id);
    }
}

/// Spawn collision entities from legacy ZoneTilemap format
fn spawn_legacy_tilemap_collision(commands: &mut Commands, tilemap: &eryndor_shared::ZoneTilemap, zone_id: &str) {
    let tile_size = tilemap.tile_size as f32;
    let chunk_size = tilemap.chunk_size as i32;
    let mut collision_count = 0;

    for (chunk_key, chunk) in &tilemap.chunks {
        let Some((chunk_x, chunk_y)) = eryndor_shared::ZoneTilemap::parse_chunk_key(chunk_key) else {
            continue;
        };

        // Iterate through collision layer
        for (row_idx, row) in chunk.collision.iter().enumerate() {
            for (col_idx, &is_blocked) in row.iter().enumerate() {
                if is_blocked == 0 {
                    continue;
                }

                // Calculate world position for this tile
                let world_x = (chunk_x * chunk_size + col_idx as i32) as f32 * tile_size + (tile_size / 2.0);
                let world_y = (chunk_y * chunk_size + row_idx as i32) as f32 * tile_size + (tile_size / 2.0);

                // Spawn collision entity
                commands.spawn((
                    TilemapCollider,
                    PhysicsPosition(Vec2::new(world_x, world_y)),
                    RigidBody::Static,
                    Collider::rectangle(tile_size, tile_size),
                    CollisionLayers::new(GameLayer::Environment, [GameLayer::Player, GameLayer::Enemy]),
                ));
                collision_count += 1;
            }
        }
    }

    if collision_count > 0 {
        info!("Spawned {} legacy tilemap collision entities for zone: {}", collision_count, zone_id);
    }
}
