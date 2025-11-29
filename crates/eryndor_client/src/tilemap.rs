//! Tilemap rendering system for the client
//!
//! Loads zone tilemap data and renders tile sprites for ground and decoration layers.

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use eryndor_shared::{TilePalette, ZoneTilemap, TileChunk};

/// Z-index values for different layers
pub const Z_GROUND: f32 = -10.0;
pub const Z_DECORATIONS: f32 = -5.0;
pub const Z_ENTITIES: f32 = 0.0;

/// Resource to store loaded tile palette
#[derive(Resource, Default)]
pub struct TilePaletteResource {
    pub palette: Option<TilePalette>,
    pub tile_textures: HashMap<u32, Handle<Image>>,
    pub loaded: bool,
}

/// Resource to store the current zone's tilemap
#[derive(Resource, Default)]
pub struct CurrentZoneTilemap {
    pub tilemap: Option<ZoneTilemap>,
    pub zone_id: Option<String>,
}

/// Resource to track which chunks are currently rendered
#[derive(Resource, Default)]
pub struct LoadedTileChunks {
    pub chunks: HashSet<(i32, i32)>,
}

/// Marker component for ground tile sprites
#[derive(Component)]
pub struct GroundTile {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub local_x: usize,
    pub local_y: usize,
}

/// Marker component for decoration sprites
#[derive(Component)]
pub struct DecorationTile {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub local_x: usize,
    pub local_y: usize,
    pub tile_id: u32,
}

/// Marker component for all tilemap entities (for cleanup)
#[derive(Component)]
pub struct TilemapEntity;

/// System to load the tile palette from JSON
pub fn load_tile_palette(
    asset_server: Res<AssetServer>,
    mut palette_res: ResMut<TilePaletteResource>,
) {
    if palette_res.loaded {
        return;
    }

    // For now, we'll hardcode a simple palette until we implement proper JSON loading
    // In a full implementation, you'd load this from the JSON file
    let palette = TilePalette {
        tile_size: 16,
        ..Default::default()
    };

    // Load textures for common tiles
    let tile_paths = [
        (1, "tiles/Tiles/Grass/Grass_1_Middle.png"),
        (2, "tiles/Tiles/Grass/Grass_2_Middle.png"),
        (3, "tiles/Tiles/Grass/Grass_3_Middle.png"),
        (4, "tiles/Tiles/Grass/Grass_4_Middle.png"),
        (5, "tiles/Tiles/Grass/Grass_Tiles_2.png"),
        (20, "tiles/Tiles/Cobble_Road/Cobble_Road_1.png"),
        (21, "tiles/Tiles/Cobble_Road/Cobble_Road_2.png"),
        (30, "tiles/Tiles/Pavement_Tiles.png"),
        (40, "tiles/Tiles/Water/Water_Middle.png"),
        // Trees
        (100, "tiles/Trees/Big_Oak_Tree.png"),
        (101, "tiles/Trees/Big_Birch_Tree.png"),
        (110, "tiles/Trees/Medium_Oak_Tree.png"),
        (120, "tiles/Trees/Small_Oak_Tree.png"),
        // Decorations
        (150, "tiles/Outdoor decoration/Flowers.png"),
        (151, "tiles/Outdoor decoration/Fountain.png"),
        (152, "tiles/Outdoor decoration/Well.png"),
        (153, "tiles/Outdoor decoration/Benches.png"),
        (154, "tiles/Outdoor decoration/Fences.png"),
        (156, "tiles/Outdoor decoration/barrels.png"),
    ];

    for (id, path) in tile_paths {
        let handle: Handle<Image> = asset_server.load(path);
        palette_res.tile_textures.insert(id, handle);
    }

    palette_res.palette = Some(palette);
    palette_res.loaded = true;
}

/// System to spawn tile sprites for the current zone's tilemap
pub fn spawn_tilemap_sprites(
    mut commands: Commands,
    tilemap_res: Res<CurrentZoneTilemap>,
    palette_res: Res<TilePaletteResource>,
    mut loaded_chunks: ResMut<LoadedTileChunks>,
    existing_tiles: Query<Entity, With<TilemapEntity>>,
) {
    let Some(tilemap) = &tilemap_res.tilemap else {
        return;
    };

    if !palette_res.loaded {
        return;
    }

    let tile_size = tilemap.tile_size as f32;
    let chunk_size = tilemap.chunk_size as i32;

    for (chunk_key, chunk) in &tilemap.chunks {
        let Some((chunk_x, chunk_y)) = ZoneTilemap::parse_chunk_key(chunk_key) else {
            continue;
        };

        // Skip if already loaded
        if loaded_chunks.chunks.contains(&(chunk_x, chunk_y)) {
            continue;
        }

        // Spawn ground tiles
        spawn_chunk_ground(&mut commands, chunk, chunk_x, chunk_y, chunk_size, tile_size, &palette_res);

        // Spawn decoration tiles
        spawn_chunk_decorations(&mut commands, chunk, chunk_x, chunk_y, chunk_size, tile_size, &palette_res);

        loaded_chunks.chunks.insert((chunk_x, chunk_y));
    }
}

fn spawn_chunk_ground(
    commands: &mut Commands,
    chunk: &TileChunk,
    chunk_x: i32,
    chunk_y: i32,
    chunk_size: i32,
    tile_size: f32,
    palette_res: &TilePaletteResource,
) {
    for (row_idx, row) in chunk.ground.iter().enumerate() {
        for (col_idx, &tile_id) in row.iter().enumerate() {
            if tile_id == 0 {
                continue;
            }

            let Some(texture) = palette_res.tile_textures.get(&tile_id) else {
                continue;
            };

            let world_x = (chunk_x * chunk_size + col_idx as i32) as f32 * tile_size;
            let world_y = (chunk_y * chunk_size + row_idx as i32) as f32 * tile_size;

            commands.spawn((
                Sprite {
                    image: texture.clone(),
                    ..default()
                },
                Transform::from_xyz(world_x, world_y, Z_GROUND),
                GroundTile {
                    chunk_x,
                    chunk_y,
                    local_x: col_idx,
                    local_y: row_idx,
                },
                TilemapEntity,
            ));
        }
    }
}

fn spawn_chunk_decorations(
    commands: &mut Commands,
    chunk: &TileChunk,
    chunk_x: i32,
    chunk_y: i32,
    chunk_size: i32,
    tile_size: f32,
    palette_res: &TilePaletteResource,
) {
    for (row_idx, row) in chunk.decorations.iter().enumerate() {
        for (col_idx, &tile_id) in row.iter().enumerate() {
            if tile_id == 0 {
                continue;
            }

            let Some(texture) = palette_res.tile_textures.get(&tile_id) else {
                continue;
            };

            let world_x = (chunk_x * chunk_size + col_idx as i32) as f32 * tile_size;
            let world_y = (chunk_y * chunk_size + row_idx as i32) as f32 * tile_size;

            // Decorations are rendered above ground
            commands.spawn((
                Sprite {
                    image: texture.clone(),
                    ..default()
                },
                Transform::from_xyz(world_x, world_y, Z_DECORATIONS),
                DecorationTile {
                    chunk_x,
                    chunk_y,
                    local_x: col_idx,
                    local_y: row_idx,
                    tile_id,
                },
                TilemapEntity,
            ));
        }
    }
}

/// System to cleanup tilemap entities when zone changes
pub fn cleanup_tilemap(
    mut commands: Commands,
    tilemap_entities: Query<Entity, With<TilemapEntity>>,
    mut loaded_chunks: ResMut<LoadedTileChunks>,
) {
    for entity in tilemap_entities.iter() {
        commands.entity(entity).despawn();
    }
    loaded_chunks.chunks.clear();
}

/// System to load zone tilemap from JSON (placeholder - will be extended)
/// In the full implementation, this would fetch tilemap data from server or load from assets
pub fn update_zone_tilemap(
    mut tilemap_res: ResMut<CurrentZoneTilemap>,
) {
    // For now, this is a placeholder
    // The tilemap will be set when the zone is loaded
    // This could be done via:
    // 1. Loading from local assets (zone JSON files)
    // 2. Receiving from server via event
}

/// Debug system to create a simple test tilemap
#[allow(dead_code)]
pub fn create_test_tilemap(
    mut tilemap_res: ResMut<CurrentZoneTilemap>,
) {
    if tilemap_res.tilemap.is_some() {
        return;
    }

    let mut tilemap = ZoneTilemap::new();

    // Create a 3x3 grid of chunks around origin
    for chunk_y in -1..=1 {
        for chunk_x in -1..=1 {
            let chunk = tilemap.get_or_create_chunk(chunk_x, chunk_y);

            // Fill ground with grass
            for y in 0..16 {
                for x in 0..16 {
                    // Vary grass types for visual interest
                    let tile_id = match (x + y) % 4 {
                        0 => 1, // grass_1
                        1 => 2, // grass_2
                        2 => 3, // grass_3
                        _ => 4, // grass_4
                    };
                    chunk.set_ground(x, y, tile_id);
                }
            }

            // Add some trees as decorations
            if chunk_x == 0 && chunk_y == 0 {
                // Skip center chunk (where players spawn)
                continue;
            }

            // Random-ish tree placement
            let seed = (chunk_x.abs() + chunk_y.abs() * 7) as usize;
            if seed.is_multiple_of(2) {
                chunk.set_decoration(4, 4, 100); // Big oak
                chunk.set_collision(4, 4, true);
            }
            if seed.is_multiple_of(3) {
                chunk.set_decoration(12, 8, 110); // Medium oak
                chunk.set_collision(12, 8, true);
            }
        }
    }

    tilemap_res.tilemap = Some(tilemap);
    tilemap_res.zone_id = Some("test".to_string());
}
