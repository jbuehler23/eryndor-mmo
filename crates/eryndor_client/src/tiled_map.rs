//! Tiled map rendering using bevy_ecs_tiled
//!
//! This module loads Tiled .tmj maps for rendering only.
//! The server handles entity spawning from object layers - the client
//! only renders tile layers (ground, decorations, etc.)

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::game_state::GameState;

/// Plugin for client-side Tiled map rendering
pub struct ClientTiledMapPlugin;

impl Plugin for ClientTiledMapPlugin {
    fn build(&self, app: &mut App) {
        // Add bevy_ecs_tiled plugin for rendering tile layers
        app.add_plugins(TiledPlugin::default());

        // Load map when entering the game
        app.add_systems(OnEnter(GameState::InGame), load_starter_zone);

        // Cleanup map when leaving the game
        app.add_systems(OnExit(GameState::InGame), cleanup_tiled_map);
    }
}

/// Marker component for the loaded Tiled map entity
#[derive(Component)]
pub struct LoadedTiledMap;

/// System to load the starter zone Tiled map for rendering
fn load_starter_zone(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing_maps: Query<Entity, With<LoadedTiledMap>>,
) {
    // Don't load if already loaded
    if !existing_maps.is_empty() {
        return;
    }

    info!("Loading starter zone Tiled map for rendering...");

    // Load the map with bevy_ecs_tiled
    // bevy_ecs_tiled will render the tile layers automatically
    // Object layers with custom components (TiledEnemy, TiledNpc) will be processed
    // but since the client doesn't register those types, they'll be ignored
    commands.spawn((
        TiledMap(asset_server.load("tiled/maps/starter_zone.tmx")),
        TilemapAnchor::Center,
        LoadedTiledMap,
    ));
}

/// System to cleanup the Tiled map when leaving the game
fn cleanup_tiled_map(
    mut commands: Commands,
    map_query: Query<Entity, With<LoadedTiledMap>>,
) {
    for entity in map_query.iter() {
        commands.entity(entity).despawn();
    }
}
