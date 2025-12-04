//! Map rendering for the editor viewport
//!
//! Renders tile layers using Bevy sprites, syncing with the project data.

use bevy::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::map::LayerData;
use crate::project::Project;
use crate::tools::ViewportInputState;
use crate::EditorState;
use crate::ui::{TilesetTextureCache, ToolMode};

/// Plugin for map rendering
pub struct MapRenderPlugin;

impl Plugin for MapRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderState>()
            .init_resource::<SelectionRenderState>()
            .add_systems(Update, sync_level_rendering)
            .add_systems(Update, sync_grid_rendering)
            .add_systems(Update, sync_selection_preview)
            .add_systems(Update, sync_tile_selection_highlights)
            .add_systems(Update, update_camera_from_editor_state);
    }
}

/// Tracks the current render state
#[derive(Resource, Default)]
pub struct RenderState {
    /// Currently rendered level ID
    pub rendered_level: Option<Uuid>,
    /// Spawned tile entities per layer: (level_id, layer_index, x, y) -> entity
    pub tile_entities: HashMap<(Uuid, usize, u32, u32), Entity>,
    /// Grid line entities
    pub grid_entities: Vec<Entity>,
    /// Whether we need to rebuild the map
    pub needs_rebuild: bool,
    /// Last known layer visibility states for change detection
    pub layer_visibility: HashMap<(Uuid, usize), bool>,
    /// Last known grid visibility state
    pub last_grid_visible: bool,
    /// Last rendered level dimensions for grid
    pub last_grid_dimensions: Option<(u32, u32, u32)>, // (width, height, tile_size)
}

/// Marker component for tile sprites
#[derive(Component)]
pub struct TileSprite {
    pub level_id: Uuid,
    pub layer_index: usize,
    pub x: u32,
    pub y: u32,
}

/// Marker component for the grid overlay
#[derive(Component)]
pub struct GridLine;

/// Marker component for the selection rectangle preview
#[derive(Component)]
pub struct SelectionPreview;

/// System to sync level rendering with the project data
fn sync_level_rendering(
    mut commands: Commands,
    mut render_state: ResMut<RenderState>,
    editor_state: Res<EditorState>,
    project: Res<Project>,
    tileset_cache: Res<TilesetTextureCache>,
    mut tile_sprites: Query<(Entity, &TileSprite, &mut Visibility)>,
    asset_server: Res<AssetServer>,
) {
    let current_level_id = editor_state.selected_level;

    // Check if we need to switch levels
    if render_state.rendered_level != current_level_id {
        // Despawn all existing tile sprites
        for (entity, _, _) in tile_sprites.iter() {
            commands.entity(entity).despawn();
        }
        render_state.tile_entities.clear();
        render_state.layer_visibility.clear();
        render_state.rendered_level = current_level_id;
        render_state.needs_rebuild = true;
    }

    // Get the current level
    let Some(level_id) = current_level_id else {
        return;
    };

    let Some(level) = project.levels.iter().find(|l| l.id == level_id) else {
        return;
    };

    // Check for layer visibility changes
    for (layer_index, layer) in level.layers.iter().enumerate() {
        let key = (level_id, layer_index);
        let old_vis = render_state.layer_visibility.get(&key).copied();
        if old_vis != Some(layer.visible) {
            render_state.layer_visibility.insert(key, layer.visible);
            // Update visibility of existing tiles
            for ((lid, li, _, _), entity) in render_state.tile_entities.iter() {
                if *lid == level_id && *li == layer_index {
                    if let Ok((_, _, mut visibility)) = tile_sprites.get_mut(*entity) {
                        *visibility = if layer.visible { Visibility::Inherited } else { Visibility::Hidden };
                    }
                }
            }
        }
    }

    // Rebuild if needed - despawn old tiles first
    if render_state.needs_rebuild {
        // Despawn existing tiles for this level
        let entities_to_despawn: Vec<_> = render_state.tile_entities
            .iter()
            .filter(|((lid, _, _, _), _)| *lid == level_id)
            .map(|(_, e)| *e)
            .collect();

        for entity in entities_to_despawn {
            commands.entity(entity).despawn();
        }

        // Remove from hashmap
        render_state.tile_entities.retain(|(lid, _, _, _), _| *lid != level_id);

        spawn_level_tiles(
            &mut commands,
            &mut render_state,
            level,
            &project,
            &tileset_cache,
            &asset_server,
        );
        render_state.needs_rebuild = false;
    }
}

/// System to render grid overlay
fn sync_grid_rendering(
    mut commands: Commands,
    mut render_state: ResMut<RenderState>,
    editor_state: Res<EditorState>,
    project: Res<Project>,
) {
    let show_grid = editor_state.show_grid;

    // Get current level info
    let level_info = editor_state.selected_level.and_then(|level_id| {
        project.levels.iter().find(|l| l.id == level_id).map(|level| {
            // Get tile size from first tile layer's tileset, or default
            let tile_size = level.layers.iter()
                .find_map(|layer| {
                    if let LayerData::Tiles { tileset_id, .. } = &layer.data {
                        project.tilesets.iter()
                            .find(|t| t.id == *tileset_id)
                            .map(|t| t.tile_size)
                    } else {
                        None
                    }
                })
                .unwrap_or(32);
            (level.width, level.height, tile_size)
        })
    });

    // Check if we need to update grid
    let needs_update = show_grid != render_state.last_grid_visible
        || (show_grid && level_info != render_state.last_grid_dimensions);

    if !needs_update {
        return;
    }

    // Despawn existing grid
    for entity in render_state.grid_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    render_state.last_grid_visible = show_grid;
    render_state.last_grid_dimensions = level_info;

    if !show_grid {
        return;
    }

    let Some((width, height, tile_size)) = level_info else {
        return;
    };

    let tile_size_f32 = tile_size as f32;
    let grid_color = Color::srgba(0.5, 0.5, 0.5, 0.5);
    let line_thickness = 1.0;
    let grid_width = width as f32 * tile_size_f32;
    let grid_height = height as f32 * tile_size_f32;

    // Grid covers tiles from (0,0) to (width-1, height-1)
    // Tiles are rendered with their CENTER at (x*tile_size + tile_size/2, -(y*tile_size + tile_size/2))
    // So grid lines should be at the edges of tiles:
    // Vertical lines at x=0, x=tile_size, x=2*tile_size, etc.
    // Horizontal lines at y=0, y=-tile_size, y=-2*tile_size, etc.

    // Spawn vertical lines
    for x in 0..=width {
        let world_x = x as f32 * tile_size_f32;
        // Line spans from y=0 to y=-grid_height, so center at y=-grid_height/2
        let center_y = -grid_height / 2.0;
        let entity = commands.spawn((
            Sprite {
                color: grid_color,
                custom_size: Some(Vec2::new(line_thickness, grid_height)),
                ..default()
            },
            Transform::from_xyz(world_x, center_y, 100.0),
            GridLine,
        )).id();
        render_state.grid_entities.push(entity);
    }

    // Spawn horizontal lines
    for y in 0..=height {
        let world_y = -(y as f32 * tile_size_f32);
        // Line spans from x=0 to x=grid_width, so center at x=grid_width/2
        let center_x = grid_width / 2.0;
        let entity = commands.spawn((
            Sprite {
                color: grid_color,
                custom_size: Some(Vec2::new(grid_width, line_thickness)),
                ..default()
            },
            Transform::from_xyz(center_x, world_y, 100.0),
            GridLine,
        )).id();
        render_state.grid_entities.push(entity);
    }
}

/// Spawn tile sprites for a level
fn spawn_level_tiles(
    commands: &mut Commands,
    render_state: &mut RenderState,
    level: &crate::map::Level,
    project: &Project,
    tileset_cache: &TilesetTextureCache,
    asset_server: &AssetServer,
) {
    for (layer_index, layer) in level.layers.iter().enumerate() {
        // Skip non-tile layers
        let LayerData::Tiles { tileset_id, tiles } = &layer.data else {
            continue;
        };

        // Get tileset info
        let Some(tileset) = project.tilesets.iter().find(|t| t.id == *tileset_id) else {
            continue;
        };

        let tile_size = tileset.tile_size;
        let tile_size_f32 = tile_size as f32;

        // Spawn tiles
        for y in 0..level.height {
            for x in 0..level.width {
                let index = (y * level.width + x) as usize;
                if let Some(Some(tile_index)) = tiles.get(index) {
                    let key = (level.id, layer_index, x, y);

                    // Convert virtual tile index to (image, local_index)
                    // This properly handles multi-image tilesets
                    let (texture_handle, columns, local_tile_index) = if let Some((image, local_idx)) = tileset.get_tile_image_info(*tile_index) {
                        // Get texture for this specific image from cache
                        if let Some((handle, _, _, _)) = tileset_cache.loaded.get(&image.id) {
                            (handle.clone(), image.columns, local_idx)
                        } else {
                            // Image not in cache, try to load it
                            let handle = asset_server.load(image.path.to_string_lossy().to_string());
                            (handle, image.columns, local_idx)
                        }
                    } else if !tileset.images.is_empty() {
                        // Tile index out of bounds for images - use first image as fallback
                        let image = &tileset.images[0];
                        if let Some((handle, _, _, _)) = tileset_cache.loaded.get(&image.id) {
                            (handle.clone(), image.columns, *tile_index)
                        } else {
                            let handle = asset_server.load(image.path.to_string_lossy().to_string());
                            (handle, image.columns, *tile_index)
                        }
                    } else {
                        // Legacy: no images array, use tileset-level path
                        if let Some(path) = tileset.primary_path() {
                            (asset_server.load(path.to_string_lossy().to_string()), tileset.columns, *tile_index)
                        } else {
                            continue;
                        }
                    };

                    // Calculate tile position in world space
                    let world_x = x as f32 * tile_size_f32 + tile_size_f32 / 2.0;
                    let world_y = -(y as f32 * tile_size_f32) - tile_size_f32 / 2.0;

                    // Calculate texture rect using the correct image's columns
                    let tile_col = local_tile_index % columns;
                    let tile_row = local_tile_index / columns;
                    let rect = URect::new(
                        tile_col * tile_size,
                        tile_row * tile_size,
                        (tile_col + 1) * tile_size,
                        (tile_row + 1) * tile_size,
                    );

                    let entity = commands.spawn((
                        Sprite {
                            image: texture_handle,
                            custom_size: Some(Vec2::new(tile_size_f32, tile_size_f32)),
                            rect: Some(Rect::new(
                                rect.min.x as f32,
                                rect.min.y as f32,
                                rect.max.x as f32,
                                rect.max.y as f32,
                            )),
                            ..default()
                        },
                        Transform::from_xyz(world_x, world_y, layer_index as f32),
                        if layer.visible { Visibility::Inherited } else { Visibility::Hidden },
                        TileSprite {
                            level_id: level.id,
                            layer_index,
                            x,
                            y,
                        },
                    )).id();

                    render_state.tile_entities.insert(key, entity);
                }
            }
        }
    }

    // Store layer visibility
    for (layer_index, layer) in level.layers.iter().enumerate() {
        render_state.layer_visibility.insert((level.id, layer_index), layer.visible);
    }
}

/// System to update camera based on editor state
fn update_camera_from_editor_state(
    editor_state: Res<EditorState>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    mut projection_query: Query<&mut Projection, With<Camera2d>>,
) {
    // Update camera position
    for mut transform in camera_query.iter_mut() {
        transform.translation.x = editor_state.camera_offset.x;
        transform.translation.y = -editor_state.camera_offset.y; // Flip Y for screen coords
    }

    // Update zoom via projection
    for mut projection in projection_query.iter_mut() {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale = 1.0 / editor_state.zoom;
        }
    }
}

/// System to render the selection rectangle preview
fn sync_selection_preview(
    mut commands: Commands,
    editor_state: Res<EditorState>,
    input_state: Res<ViewportInputState>,
    project: Res<Project>,
    existing_preview: Query<Entity, With<SelectionPreview>>,
) {
    // Always despawn existing preview first
    for entity in existing_preview.iter() {
        commands.entity(entity).despawn();
    }

    // Only show preview when actively drawing in Rectangle mode
    let is_rectangle_mode = editor_state.tool_mode == ToolMode::Rectangle
        && editor_state.current_tool.supports_modes();
    if !is_rectangle_mode || !input_state.is_drawing_rect {
        return;
    }

    let Some((start_x, start_y)) = input_state.rect_start_tile else {
        return;
    };

    let Some(current_pos) = input_state.last_world_pos else {
        return;
    };

    // Get tile size
    let tile_size = get_tile_size(&editor_state, &project);

    // Calculate end tile position
    let end_x = (current_pos.x / tile_size).floor() as i32;
    let end_y = (-current_pos.y / tile_size).floor() as i32;

    // Normalize bounds
    let min_x = start_x.min(end_x);
    let max_x = start_x.max(end_x);
    let min_y = start_y.min(end_y);
    let max_y = start_y.max(end_y);

    // Calculate world coordinates for the rectangle
    // The rectangle covers tiles from (min_x, min_y) to (max_x, max_y)
    // World position of tile top-left corner is (x * tile_size, -y * tile_size)
    let world_min_x = min_x as f32 * tile_size;
    let world_max_x = (max_x + 1) as f32 * tile_size;
    let world_min_y = -(max_y + 1) as f32 * tile_size;
    let world_max_y = -(min_y as f32 * tile_size);

    let width = world_max_x - world_min_x;
    let height = world_max_y - world_min_y;
    let center_x = world_min_x + width / 2.0;
    let center_y = world_min_y + height / 2.0;

    // Choose color based on whether we're filling or erasing
    let color = if editor_state.selected_tile.is_some() {
        Color::srgba(0.2, 0.4, 0.8, 0.4) // Blue for fill
    } else {
        Color::srgba(0.8, 0.2, 0.2, 0.4) // Red for erase
    };

    // Spawn the preview rectangle
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(width, height)),
            ..default()
        },
        Transform::from_xyz(center_x, center_y, 200.0), // High Z to render on top
        SelectionPreview,
    ));
}

/// Get the tile size for the current level/layer/tileset (for preview rendering)
fn get_tile_size(editor_state: &EditorState, project: &Project) -> f32 {
    let level_id = editor_state.selected_level;
    let layer_idx = editor_state.selected_layer;

    let level = level_id.and_then(|id| project.levels.iter().find(|l| l.id == id));
    let layer_tileset_id = level.and_then(|l| {
        layer_idx.and_then(|idx| l.layers.get(idx)).and_then(|layer| {
            if let LayerData::Tiles { tileset_id, .. } = &layer.data {
                Some(*tileset_id)
            } else {
                None
            }
        })
    });

    layer_tileset_id
        .or(editor_state.selected_tileset)
        .and_then(|id| project.tilesets.iter().find(|t| t.id == id))
        .map(|t| t.tile_size as f32)
        .unwrap_or(32.0)
}

/// Resource tracking the current selection highlight state for change detection
#[derive(Resource, Default)]
pub struct SelectionRenderState {
    /// Set of currently highlighted tiles (level_id, layer_idx, x, y)
    pub highlighted_tiles: std::collections::HashSet<(Uuid, usize, u32, u32)>,
    /// Entities for each highlight sprite
    pub highlight_entities: HashMap<(Uuid, usize, u32, u32), Entity>,
}

/// Marker component for tile selection highlight sprites
#[derive(Component)]
pub struct TileSelectionHighlight;

/// System to render tile selection highlights
fn sync_tile_selection_highlights(
    mut commands: Commands,
    editor_state: Res<EditorState>,
    project: Res<Project>,
    mut selection_state: ResMut<SelectionRenderState>,
) {
    let current_selection = &editor_state.tile_selection.tiles;

    // Check if selection has changed
    if *current_selection == selection_state.highlighted_tiles {
        return;
    }

    // Find tiles to remove (in old selection but not in new)
    let to_remove: Vec<_> = selection_state.highlighted_tiles
        .difference(current_selection)
        .cloned()
        .collect();

    // Find tiles to add (in new selection but not in old)
    let to_add: Vec<_> = current_selection
        .difference(&selection_state.highlighted_tiles)
        .cloned()
        .collect();

    // Remove old highlights
    for key in to_remove {
        if let Some(entity) = selection_state.highlight_entities.remove(&key) {
            commands.entity(entity).despawn();
        }
    }

    // Get tile size for positioning
    let tile_size = get_tile_size(&editor_state, &project);
    let highlight_color = Color::srgba(0.2, 0.6, 1.0, 0.4); // Blue highlight

    // Add new highlights
    for (level_id, layer_idx, x, y) in to_add {
        // Only render highlights for the currently selected level
        if Some(level_id) != editor_state.selected_level {
            continue;
        }

        // Calculate world position
        let world_x = x as f32 * tile_size + tile_size / 2.0;
        let world_y = -(y as f32 * tile_size) - tile_size / 2.0;

        let entity = commands.spawn((
            Sprite {
                color: highlight_color,
                custom_size: Some(Vec2::new(tile_size, tile_size)),
                ..default()
            },
            Transform::from_xyz(world_x, world_y, 150.0), // Above tiles, below grid
            TileSelectionHighlight,
        )).id();

        selection_state.highlight_entities.insert((level_id, layer_idx, x, y), entity);
    }

    // Update tracked state
    selection_state.highlighted_tiles = current_selection.clone();
}
