//! Editor tools - painting, selection, pan/zoom
//!
//! Handles viewport input for various editing operations.

use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use bevy_egui::EguiContexts;
use std::collections::HashMap;

use crate::autotile;
use crate::commands::{CommandHistory, BatchTileCommand, collect_tiles_in_region};
use crate::map::{EntityInstance, LayerData};
use crate::project::Project;
use crate::EditorState;
use crate::ui::{EditorTool, Selection, ToolMode};
use crate::render::RenderState;

/// Plugin for editor tools and viewport input
pub struct EditorToolsPlugin;

impl Plugin for EditorToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewportInputState>()
            .init_resource::<PaintStrokeTracker>()
            .add_systems(Update, (
                handle_viewport_input,
                handle_zoom_input,
                finalize_paint_stroke,
            ));
    }
}

/// State for viewport input handling
#[derive(Resource, Default)]
pub struct ViewportInputState {
    /// Last mouse position in world coordinates
    pub last_world_pos: Option<Vec2>,
    /// Whether we're currently panning
    pub is_panning: bool,
    /// Last cursor position for panning
    pub pan_start_pos: Option<Vec2>,
    /// Start tile position for rectangle tool
    pub rect_start_tile: Option<(i32, i32)>,
    /// Whether we're currently drawing a rectangle
    pub is_drawing_rect: bool,
    /// Last terrain paint target (for deduplication)
    pub last_paint_target: Option<autotile::PaintTarget>,
}

/// Tracks tile changes during a painting stroke for undo support
#[derive(Resource, Default)]
pub struct PaintStrokeTracker {
    /// Whether we're currently in a paint stroke
    pub active: bool,
    /// The level being painted
    pub level_id: Option<uuid::Uuid>,
    /// The layer being painted
    pub layer_idx: Option<usize>,
    /// Changes made during this stroke: (x, y) -> (old_tile, new_tile)
    pub changes: HashMap<(u32, u32), (Option<u32>, Option<u32>)>,
    /// Description of the operation
    pub description: String,
}

/// Capture a snapshot of tiles in a region around a center point for undo tracking
/// Returns a map of (x, y) -> tile_value for tiles within the specified radius
fn capture_tile_region(
    tiles: &[Option<u32>],
    width: u32,
    height: u32,
    center_x: i32,
    center_y: i32,
    radius: i32,
) -> HashMap<(u32, u32), Option<u32>> {
    let mut snapshot = HashMap::new();

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = center_x + dx;
            let y = center_y + dy;

            if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
                let x = x as u32;
                let y = y as u32;
                let idx = (y * width + x) as usize;
                let tile = tiles.get(idx).copied().flatten();
                snapshot.insert((x, y), tile);
            }
        }
    }

    snapshot
}

/// System to handle viewport input (painting, panning)
fn handle_viewport_input(
    mut contexts: EguiContexts,
    mut editor_state: ResMut<EditorState>,
    mut project: ResMut<Project>,
    mut render_state: ResMut<RenderState>,
    mut input_state: ResMut<ViewportInputState>,
    mut stroke_tracker: ResMut<PaintStrokeTracker>,
    mut history: ResMut<CommandHistory>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let Some(window) = windows.iter().next() else { return };
    let Some((camera, camera_transform)) = camera_q.iter().next() else { return };

    let Some(cursor_position) = window.cursor_position() else {
        input_state.is_panning = false;
        editor_state.is_painting = false;
        return;
    };

    // Convert cursor position to world coordinates
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    // Always update world position for preview rendering
    input_state.last_world_pos = Some(world_pos);

    // Check if egui is actively using the pointer (e.g., clicking a button, dragging a slider)
    // We use is_using_pointer() instead of wants_pointer_input() because the transparent
    // CentralPanel triggers wants_pointer_input() but shouldn't block viewport input.
    // However, if we're already drawing a rectangle, we must allow completion.
    let egui_using_pointer = ctx.is_using_pointer();

    // If egui is actively using the pointer and we're not in the middle of a rectangle draw,
    // block input. But always allow rectangle operations to complete.
    if egui_using_pointer && !input_state.is_drawing_rect {
        input_state.is_panning = false;
        editor_state.is_painting = false;
        return;
    }

    // Handle panning (middle mouse or right mouse)
    if mouse_buttons.pressed(MouseButton::Middle) || mouse_buttons.pressed(MouseButton::Right) {
        if !input_state.is_panning {
            input_state.is_panning = true;
            input_state.pan_start_pos = Some(cursor_position);
        } else if let Some(start_pos) = input_state.pan_start_pos {
            let delta = cursor_position - start_pos;
            editor_state.camera_offset.x -= delta.x / editor_state.zoom;
            editor_state.camera_offset.y -= delta.y / editor_state.zoom;
            input_state.pan_start_pos = Some(cursor_position);
        }
    } else {
        input_state.is_panning = false;
        input_state.pan_start_pos = None;
    }

    // Get tile size for coordinate conversion
    let tile_size = get_tile_size(&editor_state, &project);

    // Determine if we're in rectangle mode for this tool
    let is_rectangle_mode = editor_state.tool_mode == ToolMode::Rectangle
        && editor_state.current_tool.supports_modes();

    // Handle painting/erasing/entity placement/selection with left mouse
    if mouse_buttons.just_pressed(MouseButton::Left) && !input_state.is_panning {
        match editor_state.current_tool {
            EditorTool::Entity => {
                place_entity(&mut editor_state, &mut project, world_pos);
            }
            EditorTool::Fill => {
                fill_area(&mut editor_state, &mut project, &mut render_state, world_pos);
            }
            // Select tool - start marquee selection
            EditorTool::Select => {
                let tile_x = (world_pos.x / tile_size).floor() as i32;
                let tile_y = (-world_pos.y / tile_size).floor() as i32;
                input_state.rect_start_tile = Some((tile_x, tile_y));
                input_state.is_drawing_rect = true;
                editor_state.tile_selection.is_selecting = true;
                editor_state.tile_selection.drag_start = Some((tile_x, tile_y));
            }
            // For tools that support modes, start rectangle drawing if in Rectangle mode
            EditorTool::Paint | EditorTool::Erase | EditorTool::Terrain if is_rectangle_mode => {
                let tile_x = (world_pos.x / tile_size).floor() as i32;
                let tile_y = (-world_pos.y / tile_size).floor() as i32;
                input_state.rect_start_tile = Some((tile_x, tile_y));
                input_state.is_drawing_rect = true;
            }
            _ => {}
        }
    }

    // Handle rectangle mode release
    if mouse_buttons.just_released(MouseButton::Left) && input_state.is_drawing_rect {
        if let Some((start_x, start_y)) = input_state.rect_start_tile {
            let end_x = (world_pos.x / tile_size).floor() as i32;
            let end_y = (-world_pos.y / tile_size).floor() as i32;

            // Fill based on the current tool
            match editor_state.current_tool {
                EditorTool::Terrain => {
                    fill_terrain_rectangle(&mut editor_state, &mut project, &mut render_state, &mut history, start_x, start_y, end_x, end_y);
                }
                EditorTool::Paint | EditorTool::Erase => {
                    fill_rectangle(&mut editor_state, &mut project, &mut render_state, start_x, start_y, end_x, end_y);
                }
                EditorTool::Select => {
                    // Finalize marquee selection
                    if let (Some(level_id), Some(layer_idx)) = (editor_state.selected_level, editor_state.selected_layer) {
                        // Additive selection not yet implemented (would need keyboard input param)
                        let additive = false;

                        // Normalize rectangle bounds
                        let min_x = start_x.min(end_x).max(0) as u32;
                        let max_x = start_x.max(end_x).max(0) as u32;
                        let min_y = start_y.min(end_y).max(0) as u32;
                        let max_y = start_y.max(end_y).max(0) as u32;

                        editor_state.tile_selection.select_rectangle(
                            level_id,
                            layer_idx,
                            min_x,
                            min_y,
                            max_x,
                            max_y,
                            additive,
                        );
                    }
                    editor_state.tile_selection.is_selecting = false;
                    editor_state.tile_selection.drag_start = None;
                }
                _ => {}
            }
        }
        input_state.rect_start_tile = None;
        input_state.is_drawing_rect = false;
    }

    // Point mode painting (continuous while dragging)
    if mouse_buttons.pressed(MouseButton::Left) && !input_state.is_panning && !is_rectangle_mode {
        match editor_state.current_tool {
            EditorTool::Paint => {
                paint_tile(&mut editor_state, &mut project, &mut render_state, &mut stroke_tracker, world_pos);
            }
            EditorTool::Terrain => {
                paint_terrain_tile(&mut editor_state, &mut project, &mut render_state, &mut input_state, &mut stroke_tracker, world_pos);
            }
            EditorTool::Erase => {
                erase_tile(&mut editor_state, &mut project, &mut render_state, &mut stroke_tracker, world_pos);
            }
            _ => {}
        }
    } else if !input_state.is_drawing_rect {
        editor_state.is_painting = false;
        editor_state.last_painted_tile = None;
        input_state.last_paint_target = None;
    }
}

/// Get the tile size for the current level/layer/tileset
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

/// System to handle zoom input
fn handle_zoom_input(
    mut contexts: EguiContexts,
    mut editor_state: ResMut<EditorState>,
    mut scroll_events: EventReader<MouseWheel>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Check if we're hovering over an egui widget that wants pointer input (including scroll)
    // wants_pointer_input() returns true if egui wants to handle scroll events (e.g., in scroll areas, windows)
    let egui_wants_pointer = ctx.wants_pointer_input();

    for event in scroll_events.read() {
        // Skip zoom if egui wants pointer input (e.g., scrolling in a window or scroll area)
        if egui_wants_pointer {
            continue;
        }
        let zoom_delta = event.y * 0.1;
        editor_state.zoom = (editor_state.zoom * (1.0 + zoom_delta)).clamp(0.25, 4.0);
    }
}

/// Check if a layer has any non-empty tiles
fn layer_has_tiles(layer: &crate::map::Layer) -> bool {
    if let LayerData::Tiles { tiles, .. } = &layer.data {
        tiles.iter().any(|t| t.is_some())
    } else {
        false
    }
}

/// Get the tileset_id for a layer
fn get_layer_tileset_id(layer: &crate::map::Layer) -> Option<uuid::Uuid> {
    if let LayerData::Tiles { tileset_id, .. } = &layer.data {
        Some(*tileset_id)
    } else {
        None
    }
}

/// Paint a tile at the given world position
fn paint_tile(
    editor_state: &mut EditorState,
    project: &mut Project,
    render_state: &mut RenderState,
    stroke_tracker: &mut PaintStrokeTracker,
    world_pos: Vec2,
) {
    // Need a selected level, layer, tile, and tileset
    let Some(level_id) = editor_state.selected_level else { return };
    let Some(layer_idx) = editor_state.selected_layer else { return };
    let Some(tile_index) = editor_state.selected_tile else { return };
    let Some(selected_tileset) = editor_state.selected_tileset else { return };

    // Get tile size from the selected tileset
    let tile_size = project.tilesets.iter()
        .find(|t| t.id == selected_tileset)
        .map(|t| t.tile_size as f32)
        .unwrap_or(32.0);

    // Convert world position to tile coordinates
    let tile_x = (world_pos.x / tile_size).floor() as i32;
    let tile_y = (-world_pos.y / tile_size).floor() as i32;

    // Don't repaint the same tile
    if editor_state.last_painted_tile == Some((tile_x as u32, tile_y as u32)) {
        return;
    }

    // Validate coordinates
    let Some(level) = project.get_level_mut(level_id) else { return };
    if tile_x < 0 || tile_y < 0 || tile_x >= level.width as i32 || tile_y >= level.height as i32 {
        return;
    }

    let tile_x = tile_x as u32;
    let tile_y = tile_y as u32;

    // Check tileset compatibility - only update if layer is empty, otherwise require matching tileset
    // First, check if layer has tiles and get its tileset_id (immutable borrow)
    let (has_tiles, layer_tileset) = level.layers.get(layer_idx)
        .map(|layer| (layer_has_tiles(layer), get_layer_tileset_id(layer)))
        .unwrap_or((false, None));

    // Now do the tileset logic
    if has_tiles {
        // Layer has tiles - only paint if tileset matches
        if layer_tileset != Some(selected_tileset) {
            return; // Don't paint - tileset mismatch
        }
    } else {
        // Layer is empty - update to use selected tileset
        if let Some(layer) = level.layers.get_mut(layer_idx) {
            if let LayerData::Tiles { tileset_id, .. } = &mut layer.data {
                *tileset_id = selected_tileset;
            }
        }
    }

    // Get old tile for undo tracking
    let old_tile = level.get_tile(layer_idx, tile_x, tile_y);

    // Set the tile
    level.set_tile(layer_idx, tile_x, tile_y, Some(tile_index));
    project.mark_dirty();

    // Track changes for undo
    if !stroke_tracker.active {
        stroke_tracker.active = true;
        stroke_tracker.level_id = Some(level_id);
        stroke_tracker.layer_idx = Some(layer_idx);
        stroke_tracker.changes.clear();
        stroke_tracker.description = "Paint Tiles".to_string();
    }

    // Only track if this is a new change for this position
    if !stroke_tracker.changes.contains_key(&(tile_x, tile_y)) {
        stroke_tracker.changes.insert((tile_x, tile_y), (old_tile, Some(tile_index)));
    } else {
        // Update the new_tile value but keep the original old_tile
        if let Some(change) = stroke_tracker.changes.get_mut(&(tile_x, tile_y)) {
            change.1 = Some(tile_index);
        }
    }

    // Mark for re-render
    render_state.needs_rebuild = true;

    editor_state.is_painting = true;
    editor_state.last_painted_tile = Some((tile_x, tile_y));
}

/// Erase a tile at the given world position
fn erase_tile(
    editor_state: &mut EditorState,
    project: &mut Project,
    render_state: &mut RenderState,
    stroke_tracker: &mut PaintStrokeTracker,
    world_pos: Vec2,
) {
    // Need a selected level and layer
    let Some(level_id) = editor_state.selected_level else { return };
    let Some(layer_idx) = editor_state.selected_layer else { return };

    // Get tile size from the layer's tileset, or selected tileset, default to 32 if not found
    let tile_size = {
        let level = project.levels.iter().find(|l| l.id == level_id);
        let layer_tileset_id = level.and_then(|l| {
            l.layers.get(layer_idx).and_then(|layer| {
                if let crate::map::LayerData::Tiles { tileset_id, .. } = &layer.data {
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
    };

    // Convert world position to tile coordinates
    let tile_x = (world_pos.x / tile_size).floor() as i32;
    let tile_y = (-world_pos.y / tile_size).floor() as i32; // Y is inverted

    // Don't erase the same tile repeatedly
    if editor_state.last_painted_tile == Some((tile_x as u32, tile_y as u32)) {
        return;
    }

    // Validate coordinates
    let Some(level) = project.get_level_mut(level_id) else { return };
    if tile_x < 0 || tile_y < 0 || tile_x >= level.width as i32 || tile_y >= level.height as i32 {
        return;
    }

    let tile_x = tile_x as u32;
    let tile_y = tile_y as u32;

    // Get old tile for undo tracking
    let old_tile = level.get_tile(layer_idx, tile_x, tile_y);

    // Erase the tile
    level.set_tile(layer_idx, tile_x, tile_y, None);
    project.mark_dirty();

    // Track changes for undo
    if !stroke_tracker.active {
        stroke_tracker.active = true;
        stroke_tracker.level_id = Some(level_id);
        stroke_tracker.layer_idx = Some(layer_idx);
        stroke_tracker.changes.clear();
        stroke_tracker.description = "Erase Tiles".to_string();
    }

    // Only track if this is a new change for this position
    if !stroke_tracker.changes.contains_key(&(tile_x, tile_y)) {
        stroke_tracker.changes.insert((tile_x, tile_y), (old_tile, None));
    } else {
        // Update the new_tile value but keep the original old_tile
        if let Some(change) = stroke_tracker.changes.get_mut(&(tile_x, tile_y)) {
            change.1 = None;
        }
    }

    // Mark for re-render
    render_state.needs_rebuild = true;

    editor_state.is_painting = true;
    editor_state.last_painted_tile = Some((tile_x, tile_y));
}

/// Place an entity at the given world position
fn place_entity(
    editor_state: &mut EditorState,
    project: &mut Project,
    world_pos: Vec2,
) {
    // Need a selected level
    let Some(level_id) = editor_state.selected_level else { return };

    // Convert world position to level coordinates (Y is inverted in world space)
    let position = Vec2::new(world_pos.x, -world_pos.y);

    // Create a new entity - default type is "Entity", can be changed in inspector
    let entity = EntityInstance::new("Entity".to_string(), position);
    let entity_id = entity.id;

    // Add to level
    let Some(level) = project.get_level_mut(level_id) else { return };
    level.add_entity(entity);
    project.mark_dirty();

    // Select the new entity
    editor_state.selection = Selection::EntityInstance(level_id, entity_id);
}

/// Fill a rectangular area with the selected tile (or erase if no tile selected)
fn fill_rectangle(
    editor_state: &mut EditorState,
    project: &mut Project,
    render_state: &mut RenderState,
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
) {
    let Some(level_id) = editor_state.selected_level else { return };
    let Some(layer_idx) = editor_state.selected_layer else { return };

    // Get tile and tileset - if no tile selected, we'll erase
    let tile_index = editor_state.selected_tile;
    let selected_tileset = editor_state.selected_tileset;

    // Get level dimensions
    let Some(level) = project.get_level_mut(level_id) else { return };
    let level_width = level.width as i32;
    let level_height = level.height as i32;

    // Normalize rectangle bounds
    let min_x = start_x.min(end_x).max(0);
    let max_x = start_x.max(end_x).min(level_width - 1);
    let min_y = start_y.min(end_y).max(0);
    let max_y = start_y.max(end_y).min(level_height - 1);

    // For painting (not erasing), check tileset compatibility
    if let (Some(tile_idx), Some(sel_tileset)) = (tile_index, selected_tileset) {
        // First, check if layer has tiles and get its tileset_id (immutable borrow)
        let (has_tiles, layer_tileset) = level.layers.get(layer_idx)
            .map(|layer| (layer_has_tiles(layer), get_layer_tileset_id(layer)))
            .unwrap_or((false, None));

        if has_tiles {
            // Layer has tiles - only paint if tileset matches
            if layer_tileset != Some(sel_tileset) {
                return; // Don't paint - tileset mismatch
            }
        } else {
            // Layer is empty - update to use selected tileset
            if let Some(layer) = level.layers.get_mut(layer_idx) {
                if let LayerData::Tiles { tileset_id, .. } = &mut layer.data {
                    *tileset_id = sel_tileset;
                }
            }
        }

        // Fill the rectangle with tiles
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                level.set_tile(layer_idx, x as u32, y as u32, Some(tile_idx));
            }
        }
    } else {
        // No tile selected - erase the rectangle
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                level.set_tile(layer_idx, x as u32, y as u32, None);
            }
        }
    }

    project.mark_dirty();
    render_state.needs_rebuild = true;
}

/// Flood fill an area with the selected tile (bucket fill)
fn fill_area(
    editor_state: &mut EditorState,
    project: &mut Project,
    render_state: &mut RenderState,
    world_pos: Vec2,
) {
    let Some(level_id) = editor_state.selected_level else { return };
    let Some(layer_idx) = editor_state.selected_layer else { return };
    let Some(tile_index) = editor_state.selected_tile else { return };
    let Some(selected_tileset) = editor_state.selected_tileset else { return };

    let tile_size = get_tile_size(editor_state, project);

    // Convert world position to tile coordinates
    let start_x = (world_pos.x / tile_size).floor() as i32;
    let start_y = (-world_pos.y / tile_size).floor() as i32;

    let Some(level) = project.get_level_mut(level_id) else { return };

    // Validate starting position
    if start_x < 0 || start_y < 0 || start_x >= level.width as i32 || start_y >= level.height as i32 {
        return;
    }

    // Get the tile we're replacing
    let target_tile = level.get_tile(layer_idx, start_x as u32, start_y as u32);

    // Don't fill if clicking on the same tile type
    if target_tile == Some(tile_index) {
        return;
    }

    // Check tileset compatibility - only update if layer is empty, otherwise require matching tileset
    // First, check if layer has tiles and get its tileset_id (immutable borrow)
    let (has_tiles, layer_tileset) = level.layers.get(layer_idx)
        .map(|layer| (layer_has_tiles(layer), get_layer_tileset_id(layer)))
        .unwrap_or((false, None));

    if has_tiles {
        // Layer has tiles - only fill if tileset matches
        if layer_tileset != Some(selected_tileset) {
            return; // Don't fill - tileset mismatch
        }
    } else {
        // Layer is empty - update to use selected tileset
        if let Some(layer) = level.layers.get_mut(layer_idx) {
            if let LayerData::Tiles { tileset_id, .. } = &mut layer.data {
                *tileset_id = selected_tileset;
            }
        }
    }

    let level_width = level.width;
    let level_height = level.height;

    // Flood fill using a stack-based approach
    let mut stack = vec![(start_x as u32, start_y as u32)];
    let mut visited = std::collections::HashSet::new();

    while let Some((x, y)) = stack.pop() {
        if visited.contains(&(x, y)) {
            continue;
        }
        visited.insert((x, y));

        // Check if this tile matches the target
        if level.get_tile(layer_idx, x, y) != target_tile {
            continue;
        }

        // Fill this tile
        level.set_tile(layer_idx, x, y, Some(tile_index));

        // Add neighbors
        if x > 0 {
            stack.push((x - 1, y));
        }
        if x < level_width - 1 {
            stack.push((x + 1, y));
        }
        if y > 0 {
            stack.push((x, y - 1));
        }
        if y < level_height - 1 {
            stack.push((x, y + 1));
        }
    }

    project.mark_dirty();
    render_state.needs_rebuild = true;
}

/// Paint a terrain tile with autotiling at the given world position
fn paint_terrain_tile(
    editor_state: &mut EditorState,
    project: &mut Project,
    render_state: &mut RenderState,
    input_state: &mut ViewportInputState,
    stroke_tracker: &mut PaintStrokeTracker,
    world_pos: Vec2,
) {
    // Need a selected level and layer
    let Some(level_id) = editor_state.selected_level else { return };
    let Some(layer_idx) = editor_state.selected_layer else { return };

    // Check if we're using new terrain sets or legacy terrains
    if let Some(terrain_set_id) = editor_state.selected_terrain_set {
        // New Tiled-style terrain system
        paint_terrain_set_tile(editor_state, project, render_state, input_state, stroke_tracker, world_pos, level_id, layer_idx, terrain_set_id);
    } else if let Some(terrain_id) = editor_state.selected_terrain {
        // Legacy 47-tile blob terrain system
        paint_legacy_terrain_tile(editor_state, project, render_state, stroke_tracker, world_pos, level_id, layer_idx, terrain_id);
    }
}

/// Paint using the new Tiled-style terrain set system
fn paint_terrain_set_tile(
    editor_state: &mut EditorState,
    project: &mut Project,
    render_state: &mut RenderState,
    input_state: &mut ViewportInputState,
    stroke_tracker: &mut PaintStrokeTracker,
    world_pos: Vec2,
    level_id: uuid::Uuid,
    layer_idx: usize,
    terrain_set_id: uuid::Uuid,
) {
    // Need a terrain selected within the set
    let Some(terrain_idx) = editor_state.selected_terrain_in_set else { return };

    // Get terrain set info (clone to avoid borrow issues)
    let terrain_set = match project.autotile_config.get_terrain_set(terrain_set_id) {
        Some(ts) => ts.clone(),
        None => return,
    };
    let selected_tileset = terrain_set.tileset_id;

    // Get tile size from the selected tileset
    let tile_size = project.tilesets.iter()
        .find(|t| t.id == selected_tileset)
        .map(|t| t.tile_size as f32)
        .unwrap_or(32.0);

    // Determine paint target based on mouse position within tile
    let paint_target = autotile::get_paint_target(
        world_pos.x,
        world_pos.y,
        tile_size,
        terrain_set.set_type,
    );

    // Don't repaint the same corner/edge
    if input_state.last_paint_target == Some(paint_target) {
        return;
    }

    // Validate coordinates for the paint target
    let Some(level) = project.get_level_mut(level_id) else { return };

    // Check tileset compatibility
    let (has_tiles, layer_tileset) = level.layers.get(layer_idx)
        .map(|layer| (layer_has_tiles(layer), get_layer_tileset_id(layer)))
        .unwrap_or((false, None));

    if has_tiles {
        if layer_tileset != Some(selected_tileset) {
            return; // Don't paint - tileset mismatch
        }
    } else {
        if let Some(layer) = level.layers.get_mut(layer_idx) {
            if let LayerData::Tiles { tileset_id, .. } = &mut layer.data {
                *tileset_id = selected_tileset;
            }
        }
    }

    let level_width = level.width;
    let level_height = level.height;

    // Get the tiles array for this layer
    let tiles = if let Some(layer) = level.layers.get_mut(layer_idx) {
        if let LayerData::Tiles { tiles, .. } = &mut layer.data {
            tiles
        } else {
            return;
        }
    } else {
        return;
    };

    // Snapshot tiles in the affected region before painting (for undo tracking)
    // Terrain painting affects a region around the paint target, typically 3x3 or larger
    let (center_x, center_y): (i32, i32) = match paint_target {
        autotile::PaintTarget::Corner { corner_x, corner_y } => (corner_x as i32, corner_y as i32),
        autotile::PaintTarget::HorizontalEdge { tile_x, edge_y } => (tile_x as i32, edge_y as i32),
        autotile::PaintTarget::VerticalEdge { edge_x, tile_y } => (edge_x as i32, tile_y as i32),
    };

    // Capture a 5x5 region around the center for undo (terrain painting can affect neighbors)
    let snapshot_region = capture_tile_region(tiles, level_width, level_height, center_x, center_y, 2);

    // Use the autotile module to paint at the determined target (corner or edge)
    autotile::paint_terrain_at_target(
        tiles,
        level_width,
        level_height,
        paint_target,
        &terrain_set,
        terrain_idx,
    );

    // Track changes for undo by comparing before/after
    if !stroke_tracker.active {
        stroke_tracker.active = true;
        stroke_tracker.level_id = Some(level_id);
        stroke_tracker.layer_idx = Some(layer_idx);
        stroke_tracker.changes.clear();
        stroke_tracker.description = "Paint Terrain".to_string();
    }

    // Compare snapshot to current state and record changes
    for ((x, y), old_tile) in snapshot_region {
        let idx = (y * level_width + x) as usize;
        let new_tile = tiles.get(idx).copied().flatten();
        if old_tile != new_tile {
            // Only track if this position hasn't been tracked yet in this stroke
            if !stroke_tracker.changes.contains_key(&(x, y)) {
                stroke_tracker.changes.insert((x, y), (old_tile, new_tile));
            } else {
                // Update the new_tile value but keep the original old_tile
                if let Some(change) = stroke_tracker.changes.get_mut(&(x, y)) {
                    change.1 = new_tile;
                }
            }
        }
    }

    project.mark_dirty();
    render_state.needs_rebuild = true;

    editor_state.is_painting = true;
    input_state.last_paint_target = Some(paint_target);
}

/// Paint using the legacy 47-tile blob terrain system
fn paint_legacy_terrain_tile(
    editor_state: &mut EditorState,
    project: &mut Project,
    render_state: &mut RenderState,
    stroke_tracker: &mut PaintStrokeTracker,
    world_pos: Vec2,
    level_id: uuid::Uuid,
    layer_idx: usize,
    terrain_id: uuid::Uuid,
) {
    // Get the terrain configuration
    let terrain = match project.autotile_config.get_terrain(terrain_id) {
        Some(t) => t.clone(),
        None => return,
    };

    let selected_tileset = terrain.tileset_id;

    // Get tile size from the selected tileset
    let tile_size = project.tilesets.iter()
        .find(|t| t.id == selected_tileset)
        .map(|t| t.tile_size as f32)
        .unwrap_or(32.0);

    // Convert world position to tile coordinates
    let tile_x = (world_pos.x / tile_size).floor() as i32;
    let tile_y = (-world_pos.y / tile_size).floor() as i32;

    // Don't repaint the same tile
    if editor_state.last_painted_tile == Some((tile_x as u32, tile_y as u32)) {
        return;
    }

    // Validate coordinates
    let Some(level) = project.get_level_mut(level_id) else { return };
    if tile_x < 0 || tile_y < 0 || tile_x >= level.width as i32 || tile_y >= level.height as i32 {
        return;
    }

    let tile_x_u32 = tile_x as u32;
    let tile_y_u32 = tile_y as u32;

    // Check tileset compatibility - only update if layer is empty, otherwise require matching tileset
    let (has_tiles, layer_tileset) = level.layers.get(layer_idx)
        .map(|layer| (layer_has_tiles(layer), get_layer_tileset_id(layer)))
        .unwrap_or((false, None));

    if has_tiles {
        // Layer has tiles - only paint if tileset matches
        if layer_tileset != Some(selected_tileset) {
            return; // Don't paint - tileset mismatch
        }
    } else {
        // Layer is empty - update to use selected tileset
        if let Some(layer) = level.layers.get_mut(layer_idx) {
            if let LayerData::Tiles { tileset_id, .. } = &mut layer.data {
                *tileset_id = selected_tileset;
            }
        }
    }

    let level_width = level.width;
    let level_height = level.height;

    // Get the tiles array for this layer and apply autotiling
    if let Some(layer) = level.layers.get_mut(layer_idx) {
        if let LayerData::Tiles { tiles, .. } = &mut layer.data {
            // Snapshot tiles around the paint area for undo tracking (3x3 region)
            let snapshot_region = capture_tile_region(tiles, level_width, level_height, tile_x, tile_y, 1);

            // Create a closure that checks if a tile belongs to this terrain
            // A tile belongs to the terrain if it's within the terrain's tile range
            let first_tile = terrain.base_tile.saturating_sub(46);
            let last_tile = terrain.base_tile;
            let is_terrain_tile = |tile: Option<u32>| -> bool {
                match tile {
                    Some(t) => t >= first_tile && t <= last_tile,
                    None => false,
                }
            };

            // Use the autotile module to paint with proper neighbor updates
            autotile::paint_autotile(
                tiles,
                level_width,
                level_height,
                tile_x_u32,
                tile_y_u32,
                &terrain,
                is_terrain_tile,
            );

            // Track changes for undo by comparing before/after
            if !stroke_tracker.active {
                stroke_tracker.active = true;
                stroke_tracker.level_id = Some(level_id);
                stroke_tracker.layer_idx = Some(layer_idx);
                stroke_tracker.changes.clear();
                stroke_tracker.description = "Paint Terrain".to_string();
            }

            // Compare snapshot to current state and record changes
            for ((x, y), old_tile) in snapshot_region {
                let idx = (y * level_width + x) as usize;
                let new_tile = tiles.get(idx).copied().flatten();
                if old_tile != new_tile {
                    if !stroke_tracker.changes.contains_key(&(x, y)) {
                        stroke_tracker.changes.insert((x, y), (old_tile, new_tile));
                    } else {
                        if let Some(change) = stroke_tracker.changes.get_mut(&(x, y)) {
                            change.1 = new_tile;
                        }
                    }
                }
            }
        }
    }

    project.mark_dirty();
    render_state.needs_rebuild = true;

    editor_state.is_painting = true;
    editor_state.last_painted_tile = Some((tile_x_u32, tile_y_u32));
}

/// Fill a rectangular area with terrain tiles using the autotile system
fn fill_terrain_rectangle(
    editor_state: &mut EditorState,
    project: &mut Project,
    render_state: &mut RenderState,
    history: &mut CommandHistory,
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
) {
    let Some(level_id) = editor_state.selected_level else { return };
    let Some(layer_idx) = editor_state.selected_layer else { return };
    let Some(terrain_set_id) = editor_state.selected_terrain_set else { return };
    let Some(terrain_idx) = editor_state.selected_terrain_in_set else { return };

    // Get terrain set info (clone to avoid borrow issues)
    let terrain_set = match project.autotile_config.get_terrain_set(terrain_set_id) {
        Some(ts) => ts.clone(),
        None => return,
    };
    let selected_tileset = terrain_set.tileset_id;

    // Get level dimensions
    let Some(level) = project.get_level_mut(level_id) else { return };
    let level_width = level.width as i32;
    let level_height = level.height as i32;

    // Normalize rectangle bounds
    let min_x = start_x.min(end_x).max(0);
    let max_x = start_x.max(end_x).min(level_width - 1);
    let min_y = start_y.min(end_y).max(0);
    let max_y = start_y.max(end_y).min(level_height - 1);

    // Calculate the affected region (includes neighbors for autotiling)
    let update_min_x = (min_x - 1).max(0);
    let update_max_x = (max_x + 1).min(level_width - 1);
    let update_min_y = (min_y - 1).max(0);
    let update_max_y = (max_y + 1).min(level_height - 1);

    // Capture tile state BEFORE modifications for undo
    let before_tiles = collect_tiles_in_region(
        project, level_id, layer_idx,
        update_min_x, update_max_x, update_min_y, update_max_y,
    );

    // Re-get mutable level reference after immutable borrow
    let Some(level) = project.get_level_mut(level_id) else { return };

    // Check tileset compatibility
    let (has_tiles, layer_tileset) = level.layers.get(layer_idx)
        .map(|layer| (layer_has_tiles(layer), get_layer_tileset_id(layer)))
        .unwrap_or((false, None));

    if has_tiles {
        if layer_tileset != Some(selected_tileset) {
            return; // Don't paint - tileset mismatch
        }
    } else {
        if let Some(layer) = level.layers.get_mut(layer_idx) {
            if let LayerData::Tiles { tileset_id, .. } = &mut layer.data {
                *tileset_id = selected_tileset;
            }
        }
    }

    let level_width = level.width;
    let level_height = level.height;

    // Get the tiles array for this layer
    let tiles = if let Some(layer) = level.layers.get_mut(layer_idx) {
        if let LayerData::Tiles { tiles, .. } = &mut layer.data {
            tiles
        } else {
            return;
        }
    } else {
        return;
    };

    // First pass: Fill the entire rectangle with uniform terrain tiles
    let uniform_tiles = terrain_set.find_uniform_tiles(terrain_idx);
    let uniform_tile = uniform_tiles.first().copied();

    if let Some(tile_index) = uniform_tile {
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let idx = (y as u32 * level_width + x as u32) as usize;
                if idx < tiles.len() {
                    tiles[idx] = Some(tile_index);
                }
            }
        }
    } else {
        // No uniform tile available - can't fill
        return;
    }

    // Second pass: Update edge dirt tiles FIRST (while grass neighbors are still uniform)
    // This ensures edge tiles find proper transitions based on uniform grass terrain data
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let is_at_edge = x == min_x || x == max_x || y == min_y || y == max_y;
            if is_at_edge {
                autotile::update_tile_with_neighbors(
                    tiles, level_width, level_height,
                    x, y, &terrain_set, terrain_idx,
                );
            }
        }
    }

    // Third pass: Update outside neighbor tiles (they now see updated edge tiles)
    let update_min_x = (min_x - 1).max(0);
    let update_max_x = (max_x + 1).min(level_width as i32 - 1);
    let update_min_y = (min_y - 1).max(0);
    let update_max_y = (max_y + 1).min(level_height as i32 - 1);

    for y in update_min_y..=update_max_y {
        for x in update_min_x..=update_max_x {
            // Skip tiles inside the filled rectangle
            let is_inside = x >= min_x && x <= max_x && y >= min_y && y <= max_y;
            if is_inside {
                continue;
            }

            let idx = (y as u32 * level_width + x as u32) as usize;
            let current_tile = tiles.get(idx).copied().flatten();

            if let Some(tile) = current_tile {
                if let Some(tile_data) = terrain_set.get_tile_terrain(tile) {
                    if let Some(primary_terrain) = tile_data.terrains.iter().find_map(|t| *t) {
                        autotile::update_tile_with_neighbors(
                            tiles, level_width, level_height,
                            x, y, &terrain_set, primary_terrain,
                        );
                    }
                }
            }
        }
    }

    // Capture tile state AFTER modifications for undo command
    let after_tiles = collect_tiles_in_region(
        project, level_id, layer_idx,
        update_min_x, update_max_x, update_min_y, update_max_y,
    );

    // Create undo command from diff and push to history
    let command = BatchTileCommand::from_diff(
        level_id,
        layer_idx,
        before_tiles,
        after_tiles,
        "Fill Terrain Rectangle",
    );

    // Only add to history if there were actual changes
    if !command.changes.is_empty() {
        // Create inverse for undo (swap old and new)
        let mut inverse_changes = std::collections::HashMap::new();
        for ((x, y), (old_tile, new_tile)) in &command.changes {
            inverse_changes.insert((*x, *y), (*new_tile, *old_tile));
        }

        let inverse_command = BatchTileCommand::new(
            level_id,
            layer_idx,
            inverse_changes,
            "Undo Fill Terrain Rectangle",
        );
        history.push_undo(Box::new(inverse_command));
    }

    project.mark_dirty();
    render_state.needs_rebuild = true;
}

/// System to finalize paint strokes and create undo commands
fn finalize_paint_stroke(
    mut stroke_tracker: ResMut<PaintStrokeTracker>,
    mut history: ResMut<CommandHistory>,
    editor_state: Res<EditorState>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    // Only finalize when left mouse button is released and we have an active stroke
    if !stroke_tracker.active {
        return;
    }

    // Check if painting has stopped
    if !editor_state.is_painting && !mouse_buttons.pressed(MouseButton::Left) {
        // Finalize the stroke - create a command for undo
        if !stroke_tracker.changes.is_empty() {
            if let (Some(level_id), Some(layer_idx)) = (stroke_tracker.level_id, stroke_tracker.layer_idx) {
                // We need to create the inverse command manually since we already applied changes
                let mut inverse_changes = HashMap::new();
                for ((x, y), (old_tile, new_tile)) in &stroke_tracker.changes {
                    // Swap old and new for inverse
                    inverse_changes.insert((*x, *y), (*new_tile, *old_tile));
                }

                let inverse_command = BatchTileCommand::new(
                    level_id,
                    layer_idx,
                    inverse_changes,
                    stroke_tracker.description.clone(),
                );

                history.push_undo(Box::new(inverse_command));
            }
        }

        // Reset the stroke tracker
        stroke_tracker.active = false;
        stroke_tracker.level_id = None;
        stroke_tracker.layer_idx = None;
        stroke_tracker.changes.clear();
        stroke_tracker.description.clear();
    }
}
