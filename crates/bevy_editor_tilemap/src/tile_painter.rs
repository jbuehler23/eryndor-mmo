use bevy::prelude::*;
use bevy_editor_formats::TileData;

use crate::layer_manager::LayerManager;
use crate::map_canvas::PaintTileEvent;
use crate::tileset_manager::TilesetManager;

/// Tile painting mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaintMode {
    Single,
    Rectangle,
    BucketFill,
    Line,
}

/// Runtime state for the tile painter backend.
#[derive(Resource)]
pub struct TilePainter {
    pub mode: PaintMode,
    pub flip_x: bool,
    pub flip_y: bool,
    /// Start position for rectangle/line tools.
    pub drag_start: Option<(u32, u32)>,
    /// Current cursor position (for previews rendered by the UI layer).
    pub current_pos: Option<(u32, u32)>,
}

impl Default for TilePainter {
    fn default() -> Self {
        Self {
            mode: PaintMode::Single,
            flip_x: false,
            flip_y: false,
            drag_start: None,
            current_pos: None,
        }
    }
}

/// Apply a single tile modification to the active layer and emit a paint event
/// so renderers can update their tilemap representation.
pub fn paint_single_tile(
    x: u32,
    y: u32,
    tile_id: u32,
    flip_x: bool,
    flip_y: bool,
    layer_manager: &mut LayerManager,
    paint_events: &mut MessageWriter<PaintTileEvent>,
) {
    let tile = TileData {
        x,
        y,
        tile_id,
        flip_x,
        flip_y,
    };
    layer_manager.add_tile(tile);

    paint_events.write(PaintTileEvent {
        layer_id: 0,
        x,
        y,
        tile_id,
    });
}

/// Flood-fill convenience wrapper used by the UI layer.
pub fn bucket_fill(
    start_x: u32,
    start_y: u32,
    tile_id: u32,
    flip_x: bool,
    flip_y: bool,
    layer_manager: &mut LayerManager,
    paint_events: &mut MessageWriter<PaintTileEvent>,
) {
    let target_tile = layer_manager.get_tile_at(start_x, start_y).copied();
    let target_tile_id = target_tile.map(|t| t.tile_id);

    if target_tile_id == Some(tile_id) {
        return;
    }

    let Some(layer) = layer_manager.get_active_layer() else {
        return;
    };
    let width = layer.metadata.width;
    let height = layer.metadata.height;

    let mut stack = vec![(start_x, start_y)];
    let mut visited = std::collections::HashSet::new();

    while let Some((x, y)) = stack.pop() {
        if !visited.insert((x, y)) {
            continue;
        }

        let current_tile = layer_manager.get_tile_at(x, y).copied();
        let current_tile_id = current_tile.map(|t| t.tile_id);

        if current_tile_id != target_tile_id {
            continue;
        }

        paint_single_tile(x, y, tile_id, flip_x, flip_y, layer_manager, paint_events);

        if x > 0 {
            stack.push((x - 1, y));
        }
        if x + 1 < width {
            stack.push((x + 1, y));
        }
        if y > 0 {
            stack.push((x, y - 1));
        }
        if y + 1 < height {
            stack.push((x, y + 1));
        }
    }
}

/// Paint a rectangular area.
pub fn paint_rectangle(
    start_x: u32,
    start_y: u32,
    end_x: u32,
    end_y: u32,
    tile_id: u32,
    flip_x: bool,
    flip_y: bool,
    layer_manager: &mut LayerManager,
    paint_events: &mut MessageWriter<PaintTileEvent>,
) {
    let min_x = start_x.min(end_x);
    let max_x = start_x.max(end_x);
    let min_y = start_y.min(end_y);
    let max_y = start_y.max(end_y);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            paint_single_tile(x, y, tile_id, flip_x, flip_y, layer_manager, paint_events);
        }
    }
}

/// Paint a line using Bresenham's algorithm.
pub fn paint_line(
    start_x: u32,
    start_y: u32,
    end_x: u32,
    end_y: u32,
    tile_id: u32,
    flip_x: bool,
    flip_y: bool,
    layer_manager: &mut LayerManager,
    paint_events: &mut MessageWriter<PaintTileEvent>,
) {
    let dx = (end_x as i32 - start_x as i32).abs();
    let dy = (end_y as i32 - start_y as i32).abs();
    let sx = if start_x < end_x { 1 } else { -1 };
    let sy = if start_y < end_y { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = start_x as i32;
    let mut y = start_y as i32;

    loop {
        paint_single_tile(
            x as u32,
            y as u32,
            tile_id,
            flip_x,
            flip_y,
            layer_manager,
            paint_events,
        );

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
}

/// Paint a multi-tile stamp pattern.
pub fn paint_stamp(
    origin_x: u32,
    origin_y: u32,
    tileset_manager: &TilesetManager,
    flip_x: bool,
    flip_y: bool,
    layer_manager: &mut LayerManager,
    paint_events: &mut MessageWriter<PaintTileEvent>,
) {
    let Some((stamp_width, stamp_height)) = tileset_manager.get_selection_dimensions() else {
        return;
    };

    let Some((_start_col, _start_row)) = tileset_manager.selection_start else {
        return;
    };

    for (index, &tile_id) in tileset_manager.selected_tiles.iter().enumerate() {
        let offset_x = (index as u32) % stamp_width;
        let offset_y = (index as u32) / stamp_width;

        let world_x = origin_x + offset_x;
        let y_invert = stamp_height.saturating_sub(1).saturating_sub(offset_y);
        let world_y = origin_y + y_invert;

        if let Some(layer) = layer_manager.get_active_layer() {
            if world_x >= layer.metadata.width || world_y >= layer.metadata.height {
                continue;
            }
        }

        paint_single_tile(
            world_x,
            world_y,
            tile_id,
            flip_x,
            flip_y,
            layer_manager,
            paint_events,
        );
    }
}
