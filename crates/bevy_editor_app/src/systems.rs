use bevy::prelude::*;
use bevy::scene::{DynamicScene, DynamicSceneRoot};
use bevy_editor_foundation::EditorState;
use bevy_editor_project::CurrentProject;

/// Resource to track pending tilemap restoration
#[derive(Resource, Default)]
pub struct PendingTilemapRestore {
    pub should_restore: bool,
}

/// Handle saving and loading levels
pub fn handle_save_load(world: &mut World) {
    // Access resources through world
    let keyboard = world.resource::<ButtonInput<KeyCode>>().clone();

    // Ctrl+S to save (new .scn.ron format)
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyS) {
        // Clone the data we need before mutable borrow
        let (file_path, has_scene) = {
            let open_scenes = world.resource::<bevy_editor_scene::OpenScenes>();
            let file_path = open_scenes.active_scene().and_then(|s| s.file_path.clone());
            (file_path, open_scenes.active_scene().is_some())
        };

        if has_scene {
            if let Some(path) = file_path {
                // Save to .scn.ron using new DynamicScene system
                match bevy_editor_scene::save_editor_scene_to_file(world, &path) {
                    Ok(_) => {
                        // Update scene state
                        let mut open_scenes = world.resource_mut::<bevy_editor_scene::OpenScenes>();
                        if let Some(scene) = open_scenes.active_scene_mut() {
                            scene.is_modified = false;
                        }

                        let mut editor_scene =
                            world.resource_mut::<bevy_editor_scene::EditorScene>();
                        editor_scene.mark_saved();

                        info!("Scene saved to: {}", path);

                        // Update last opened scene in project config
                        if let Some(mut project) = world.get_resource_mut::<CurrentProject>() {
                            update_last_opened_scene(&mut project, &path);
                        }
                    }
                    Err(e) => {
                        error!("Failed to save scene: {}", e);
                    }
                }
            } else {
                // No path set, show Save As dialog
                save_as_scene_dialog_world(world);
            }
        }
    }

    // Ctrl+Shift+S for Save As
    if keyboard.pressed(KeyCode::ControlLeft)
        && keyboard.pressed(KeyCode::ShiftLeft)
        && keyboard.just_pressed(KeyCode::KeyS)
    {
        save_as_scene_dialog_world(world);
    }

    // Ctrl+O to open
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::KeyO) {
        open_scene_dialog_world(world);
    }
}

/// Save As dialog for .scn.ron files (World-based version)
fn save_as_scene_dialog_world(world: &mut World) {
    use rfd::FileDialog;

    // Get current scene name, strip .scn.ron if already present
    let scene_name = {
        let open_scenes = world.resource::<bevy_editor_scene::OpenScenes>();
        open_scenes
            .active_scene()
            .map(|s| {
                let name = s.name.clone();
                // Strip .scn.ron suffix if present to avoid double extension
                if name.ends_with(".scn.ron") {
                    name.trim_end_matches(".scn.ron").to_string()
                } else {
                    name
                }
            })
            .unwrap_or_else(|| "Untitled".to_string())
    };

    if let Some(path) = FileDialog::new()
        .add_filter("Scene", &["scn.ron"])
        .set_file_name(format!("{}.scn.ron", scene_name))
        .save_file()
    {
        let path_str = path.to_string_lossy().to_string();

        match bevy_editor_scene::save_editor_scene_to_file(world, &path_str) {
            Ok(_) => {
                let new_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
                    .to_string();

                // Update scene state
                let mut open_scenes = world.resource_mut::<bevy_editor_scene::OpenScenes>();
                if let Some(scene) = open_scenes.active_scene_mut() {
                    scene.file_path = Some(path_str.clone());
                    scene.name = new_name;
                    scene.is_modified = false;
                }

                let mut editor_scene = world.resource_mut::<bevy_editor_scene::EditorScene>();
                editor_scene.mark_saved();

                info!("Scene saved to: {}", path_str);
            }
            Err(e) => {
                error!("Failed to save scene: {}", e);
            }
        }
    }
}

/// Open dialog for .scn.ron files (World-based version)
fn open_scene_dialog_world(world: &mut World) {
    use rfd::FileDialog;

    if let Some(path) = FileDialog::new()
        .add_filter("Scene", &["scn.ron"])
        .pick_file()
    {
        let path_str = path.to_string_lossy().to_string();
        let scene_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        // Clear existing scene entities before loading new scene
        let mut entities_to_despawn = Vec::new();
        {
            let mut query =
                world.query_filtered::<Entity, With<bevy_editor_scene::EditorSceneEntity>>();
            for entity in query.iter(world) {
                entities_to_despawn.push(entity);
            }
        }
        for entity in entities_to_despawn {
            world.despawn(entity);
        }

        // Load scene using asset server
        // Clone AssetServer path loading before taking mutable borrow
        let scene_handle = {
            let asset_server = world.resource::<AssetServer>();
            asset_server.load::<DynamicScene>(path_str.clone())
        };

        // Spawn scene
        world.commands().spawn((
            DynamicSceneRoot(scene_handle.clone()),
            bevy_editor_scene::EditorSceneEntity,
            bevy_editor_scene::LoadingSceneRoot,
        ));

        // Add new scene tab
        let mut open_scenes = world.resource_mut::<bevy_editor_scene::OpenScenes>();
        let new_scene = bevy_editor_scene::OpenScene {
            name: scene_name.clone(),
            file_path: Some(path_str.clone()),
            level_data: bevy_editor_formats::LevelData::new(scene_name, 2000.0, 1000.0), // Deprecated, for compat
            is_modified: false,
            runtime_scene: None,
        };
        open_scenes.add_scene(new_scene);
        drop(open_scenes); // Release borrow

        let mut editor_scene = world.resource_mut::<bevy_editor_scene::EditorScene>();
        editor_scene.is_modified = false;

        info!("Scene loaded from: {}", path_str);
    }
}

// ==============================================================================
// OLD TILEMAP-BASED SAVE/LOAD FUNCTIONS - DEPRECATED
// These are kept temporarily for backward compatibility with .bscene files
// TODO: Remove after migration period
// ==============================================================================

#[allow(dead_code)]
fn save_level_with_tilemap(
    scene: &mut bevy_editor_scene::OpenScene, // Changed from CurrentLevel
    path: &str,
    editor_state: &EditorState,
    tileset_manager: &bevy_editor_tilemap::TilesetManager,
    map_dimensions: &bevy_editor_tilemap::MapDimensions,
    tilemap_query: &Query<
        &bevy_ecs_tilemap::prelude::TileStorage,
        With<bevy_editor_tilemap::MapCanvas>,
    >,
    tile_query: &Query<(
        &bevy_ecs_tilemap::prelude::TileTextureIndex,
        &bevy_ecs_tilemap::prelude::TileVisible,
        &bevy_ecs_tilemap::prelude::TilePos,
    )>,
) {
    use bevy_editor_formats::{
        LevelLayerData, LevelTileInstance, LevelTilemapData, LevelTilesetData,
    };
    use std::collections::HashSet;

    // Build tilemap data
    let mut tilemap_data = LevelTilemapData {
        grid_size: editor_state.grid_size,
        map_width: map_dimensions.width,
        map_height: map_dimensions.height,
        tilesets: Vec::new(),
        selected_tileset_id: tileset_manager.selected_tileset_id,
        layers: Vec::new(),
    };

    // Save tilesets - deduplicate by texture_path and convert to relative paths
    let mut seen_paths = HashSet::new();
    for (id, tileset_info) in tileset_manager.tilesets.iter() {
        if seen_paths.insert(tileset_info.data.texture_path.clone()) {
            // Convert absolute path to relative path for portability
            let relative_path = convert_to_relative_asset_path(&tileset_info.data.texture_path);

            tilemap_data.tilesets.push(LevelTilesetData {
                id: *id,
                identifier: tileset_info.data.identifier.clone(),
                texture_path: relative_path,
                tile_width: tileset_info.data.tile_width,
                tile_height: tileset_info.data.tile_height,
            });
        }
    }

    // Save tile data
    let mut layer_tiles = Vec::new();
    if let Some(tile_storage) = tilemap_query.iter().next() {
        for tile_entity in tile_storage.iter().flatten() {
            if let Ok((texture_index, visible, tile_pos)) = tile_query.get(*tile_entity) {
                if visible.0 {
                    layer_tiles.push(LevelTileInstance {
                        x: tile_pos.x,
                        y: tile_pos.y,
                        tile_id: texture_index.0,
                    });
                }
            }
        }
        info!("Saving {} tiles to level", layer_tiles.len());
    } else {
        warn!("No tilemap found when trying to save!");
    }

    tilemap_data.layers.push(LevelLayerData {
        id: 0,
        name: "Layer 0".to_string(),
        visible: true,
        tiles: layer_tiles,
    });

    // Update level data with tilemap
    scene.level_data.tilemap = Some(tilemap_data);

    // Create BevyScene and save to .bscene file
    let bevy_scene = bevy_editor_formats::BevyScene::new(scene.level_data.clone());
    if let Err(e) = bevy_scene.save_to_file(path) {
        error!("Failed to save scene: {}", e);
    } else {
        scene.file_path = Some(path.to_string());
        scene.is_modified = false;

        // Update scene name from filename
        if let Some(filename) = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
        {
            scene.name = filename.to_string();
        }

        info!("Scene saved to {}", path);
    }
}

/// System to restore tilemap data when a level is loaded
pub fn restore_tilemap_from_level(
    mut pending_restore: ResMut<PendingTilemapRestore>,
    open_scenes: Res<bevy_editor_scene::OpenScenes>, // Changed from CurrentLevel
    mut editor_state: ResMut<EditorState>,
    mut map_dimensions: ResMut<bevy_editor_tilemap::MapDimensions>,
    mut load_tileset_events: MessageWriter<bevy_editor_tilemap::LoadTilesetEvent>,
    tilemap_query: Query<
        &bevy_ecs_tilemap::prelude::TileStorage,
        (
            With<bevy_editor_tilemap::MapCanvas>,
            Added<bevy_editor_tilemap::MapCanvas>,
        ),
    >,
    mut tile_query: Query<(
        &mut bevy_ecs_tilemap::prelude::TileTextureIndex,
        &mut bevy_ecs_tilemap::prelude::TileVisible,
    )>,
) {
    // Only run if we have a pending restore
    if !pending_restore.should_restore {
        return;
    }

    // Get active scene's tilemap data
    let tilemap_data = open_scenes
        .active_scene()
        .and_then(|scene| scene.level_data.tilemap.as_ref());

    if let Some(tilemap_data) = tilemap_data {
        // Check if the tilemap canvas has been created
        if let Some(tile_storage) = tilemap_query.iter().next() {
            info!("Restoring tilemap from level data...");

            // Update editor state
            editor_state.grid_size = tilemap_data.grid_size;
            map_dimensions.width = tilemap_data.map_width;
            map_dimensions.height = tilemap_data.map_height;

            // Load tilesets
            for tileset in &tilemap_data.tilesets {
                load_tileset_events.write(bevy_editor_tilemap::LoadTilesetEvent {
                    path: tileset.texture_path.clone(),
                    identifier: tileset.identifier.clone(),
                    tile_width: tileset.tile_width,
                    tile_height: tileset.tile_height,
                });
            }

            // Restore tiles
            if !tilemap_data.layers.is_empty() {
                let layer = &tilemap_data.layers[0];
                info!("Restoring {} tiles", layer.tiles.len());

                for tile_instance in &layer.tiles {
                    let tile_pos = bevy_ecs_tilemap::prelude::TilePos {
                        x: tile_instance.x,
                        y: tile_instance.y,
                    };

                    if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                        if let Ok((mut texture_index, mut visible)) =
                            tile_query.get_mut(tile_entity)
                        {
                            texture_index.0 = tile_instance.tile_id;
                            visible.0 = true;
                        }
                    }
                }
            }

            // Clear the pending restore flag
            pending_restore.should_restore = false;
        } else {
            // Tilemap canvas not yet created, we need to initialize it
            // Update settings so the canvas will be created with correct dimensions
            editor_state.grid_size = tilemap_data.grid_size;
            map_dimensions.width = tilemap_data.map_width;
            map_dimensions.height = tilemap_data.map_height;

            // Load tilesets - this will trigger tilemap canvas creation
            for tileset in &tilemap_data.tilesets {
                load_tileset_events.write(bevy_editor_tilemap::LoadTilesetEvent {
                    path: tileset.texture_path.clone(),
                    identifier: tileset.identifier.clone(),
                    tile_width: tileset.tile_width,
                    tile_height: tileset.tile_height,
                });
            }
            // Don't clear pending_restore - we'll restore tiles next frame when canvas exists
        }
    }
}

/// Convert absolute asset path to relative path from assets folder
fn convert_to_relative_asset_path(absolute_path: &str) -> String {
    // Find "assets" in the path and return everything after it
    if let Some(idx) = absolute_path.find("assets") {
        // Skip "assets/" or "assets\" to get the relative path
        let relative = &absolute_path[idx + 7..]; // "assets/" is 7 chars
        return relative.replace('\\', "/"); // Normalize to forward slashes
    }

    // Fallback: return the original path if "assets" not found
    absolute_path.to_string()
}

/// Update the last opened scene in the project config
fn update_last_opened_scene(project: &mut CurrentProject, scene_path: &str) {
    use std::path::Path;

    // Convert absolute scene path to relative path (relative to assets/world/)
    let path = Path::new(scene_path);

    // Try to extract filename from path
    if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
        // Update last opened scene
        let _ = project.update_config(|config| {
            config.last_opened_scene = Some(filename.to_string());
            info!("Updated last opened scene to: {}", filename);
        });
    }
}

#[derive(Resource, Default)]
pub struct PreviousSceneIndex(pub Option<usize>);

/// Cache the active scene into the corresponding [`OpenScene`] before switching tabs.
pub fn cache_runtime_scene_on_scene_switch(world: &mut World) {
    let current_index = {
        let open_scenes = world.resource::<bevy_editor_scene::OpenScenes>();
        open_scenes.active_index
    };

    let mut previous = world.resource_mut::<PreviousSceneIndex>();
    if previous.0 == Some(current_index) {
        return;
    }

    let previous_index = previous.0;
    previous.0 = Some(current_index);
    drop(previous);

    let Some(prev_idx) = previous_index else {
        return;
    };

    let scene_count = {
        let open_scenes = world.resource::<bevy_editor_scene::OpenScenes>();
        open_scenes.scenes.len()
    };
    if prev_idx >= scene_count {
        return;
    }

    world.resource_scope::<bevy_editor_scene::OpenScenes, _>(|world, mut open_scenes| {
        if let Some(scene) = open_scenes.scenes.get_mut(prev_idx) {
            let dynamic_scene = bevy_editor_scene::capture_editor_scene_runtime(world);
            let is_modified = world
                .get_resource::<bevy_editor_scene::EditorScene>()
                .map(|editor_scene| editor_scene.is_modified)
                .unwrap_or(false);

            world.resource_scope::<Assets<DynamicScene>, _>(|_world, mut assets| {
                if let Some(old_handle) = scene.runtime_scene.take() {
                    assets.remove(old_handle.id());
                }
                let handle = assets.add(dynamic_scene);
                scene.runtime_scene = Some(handle);
                scene.is_modified = is_modified;
            });
        }
    });
}

/// System to sync tilemap when switching scenes
/// This captures the current tilemap state, saves it to the old scene,
/// and loads the new scene's tilemap data
pub fn sync_tilemap_on_scene_switch(
    mut open_scenes: ResMut<bevy_editor_scene::OpenScenes>,
    mut previous_scene: Local<Option<usize>>,
    tilemap_query: Query<
        &bevy_ecs_tilemap::prelude::TileStorage,
        With<bevy_editor_tilemap::MapCanvas>,
    >,
    mut tile_query: Query<(
        &mut bevy_ecs_tilemap::prelude::TileTextureIndex,
        &mut bevy_ecs_tilemap::prelude::TileVisible,
    )>,
    editor_state: Res<EditorState>,
    map_dimensions: Res<bevy_editor_tilemap::MapDimensions>,
    tileset_manager: Res<bevy_editor_tilemap::TilesetManager>,
) {
    let current_index = open_scenes.active_index;

    // Check if scene actually changed
    if *previous_scene == Some(current_index) {
        return;
    }

    // PHASE 1: Save current tilemap to previous scene (if any)
    if let Some(prev_idx) = *previous_scene {
        if let Some(prev_scene) = open_scenes.scenes.get_mut(prev_idx) {
            if let Some(tile_storage) = tilemap_query.iter().next() {
                // Capture current tilemap state
                let tilemap_data = capture_tilemap_state(
                    tile_storage,
                    &tile_query,
                    &editor_state,
                    &map_dimensions,
                    &tileset_manager,
                );
                prev_scene.level_data.tilemap = Some(tilemap_data);
                info!("Saved tilemap state for scene '{}'", prev_scene.name);
            }
        }
    }

    // PHASE 2: Clear tilemap (set all tiles invisible AND reset texture)
    if let Some(tile_storage) = tilemap_query.iter().next() {
        for y in 0..map_dimensions.height {
            for x in 0..map_dimensions.width {
                let tile_pos = bevy_ecs_tilemap::prelude::TilePos { x, y };
                if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                    if let Ok((mut tex, mut visible)) = tile_query.get_mut(tile_entity) {
                        tex.0 = 0; // Reset texture index to 0
                        visible.0 = false; // Make invisible
                    }
                }
            }
        }
    }

    // PHASE 3: Load active scene's tilemap
    if let Some(active_scene) = open_scenes.active_scene() {
        if let Some(tilemap_data) = &active_scene.level_data.tilemap {
            if let Some(tile_storage) = tilemap_query.iter().next() {
                // Restore tiles from data
                if !tilemap_data.layers.is_empty() {
                    let layer = &tilemap_data.layers[0];
                    info!(
                        "Loading {} tiles for scene '{}'",
                        layer.tiles.len(),
                        active_scene.name
                    );

                    for tile_instance in &layer.tiles {
                        let tile_pos = bevy_ecs_tilemap::prelude::TilePos {
                            x: tile_instance.x,
                            y: tile_instance.y,
                        };
                        if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                            if let Ok((mut tex, mut vis)) = tile_query.get_mut(tile_entity) {
                                tex.0 = tile_instance.tile_id;
                                vis.0 = true;
                            }
                        }
                    }
                }
            }
        } else {
            info!("Switched to scene '{}' (empty tilemap)", active_scene.name);
        }
    }

    // Update previous scene tracker
    *previous_scene = Some(current_index);
}

/// Helper function to capture current tilemap state into LevelTilemapData
fn capture_tilemap_state(
    tile_storage: &bevy_ecs_tilemap::prelude::TileStorage,
    tile_query: &Query<(
        &mut bevy_ecs_tilemap::prelude::TileTextureIndex,
        &mut bevy_ecs_tilemap::prelude::TileVisible,
    )>,
    editor_state: &EditorState,
    map_dimensions: &bevy_editor_tilemap::MapDimensions,
    tileset_manager: &bevy_editor_tilemap::TilesetManager,
) -> bevy_editor_formats::LevelTilemapData {
    use bevy_editor_formats::{
        LevelLayerData, LevelTileInstance, LevelTilemapData, LevelTilesetData,
    };
    use std::collections::HashSet;

    let mut tiles = Vec::new();
    for y in 0..map_dimensions.height {
        for x in 0..map_dimensions.width {
            let tile_pos = bevy_ecs_tilemap::prelude::TilePos { x, y };
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                if let Ok((tex, vis)) = tile_query.get(tile_entity) {
                    if vis.0 {
                        // Only save visible tiles
                        tiles.push(LevelTileInstance {
                            x,
                            y,
                            tile_id: tex.0,
                        });
                    }
                }
            }
        }
    }

    // Build tileset data
    let mut tilesets = Vec::new();
    let mut seen_paths = HashSet::new();
    for (id, tileset_info) in tileset_manager.tilesets.iter() {
        if seen_paths.insert(tileset_info.data.texture_path.clone()) {
            let relative_path = convert_to_relative_asset_path(&tileset_info.data.texture_path);
            tilesets.push(LevelTilesetData {
                id: *id,
                identifier: tileset_info.data.identifier.clone(),
                texture_path: relative_path,
                tile_width: tileset_info.data.tile_width,
                tile_height: tileset_info.data.tile_height,
            });
        }
    }

    LevelTilemapData {
        grid_size: editor_state.grid_size,
        map_width: map_dimensions.width,
        map_height: map_dimensions.height,
        tilesets,
        selected_tileset_id: tileset_manager.selected_tileset_id,
        layers: vec![LevelLayerData {
            id: 0,
            name: "Layer 0".to_string(),
            visible: true,
            tiles,
        }],
    }
}
