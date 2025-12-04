//! Tile clipboard for copy/paste operations
//!
//! Stores copied tiles with relative positions for pasting elsewhere.

use std::collections::{HashMap, HashSet};
use bevy::prelude::*;
use uuid::Uuid;

use crate::project::Project;
use crate::EditorState;

/// Resource storing copied tiles for paste operations
#[derive(Resource, Default)]
pub struct TileClipboard {
    /// Copied tiles: relative position (x, y) -> (tileset_id, tile_index)
    pub tiles: HashMap<(i32, i32), (Uuid, u32)>,
    /// Bounding box of copied tiles: (min_x, min_y, max_x, max_y)
    pub bounds: Option<(i32, i32, i32, i32)>,
    /// Source layer index (for reference)
    pub source_layer: Option<usize>,
}

impl TileClipboard {
    /// Clear the clipboard
    pub fn clear(&mut self) {
        self.tiles.clear();
        self.bounds = None;
        self.source_layer = None;
    }

    /// Check if clipboard has content
    pub fn has_content(&self) -> bool {
        !self.tiles.is_empty()
    }

    /// Copy tiles from the current selection
    pub fn copy_selection(
        &mut self,
        selection: &TileSelection,
        project: &Project,
        editor_state: &EditorState,
    ) {
        self.clear();

        let Some(level_id) = editor_state.selected_level else {
            return;
        };

        let Some(level) = project.levels.iter().find(|l| l.id == level_id) else {
            return;
        };

        if selection.tiles.is_empty() {
            return;
        }

        // Find the origin (top-left) of the selection
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;

        for (_, _, x, y) in &selection.tiles {
            let x = *x as i32;
            let y = *y as i32;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }

        // Copy tiles with positions relative to origin
        for (lid, layer_idx, x, y) in &selection.tiles {
            if *lid != level_id {
                continue;
            }

            if let Some(layer) = level.layers.get(*layer_idx) {
                if let crate::map::LayerData::Tiles { tileset_id, tiles } = &layer.data {
                    let idx = (*y * level.width + *x) as usize;
                    if let Some(Some(tile_index)) = tiles.get(idx) {
                        let relative_x = *x as i32 - min_x;
                        let relative_y = *y as i32 - min_y;
                        self.tiles.insert((relative_x, relative_y), (*tileset_id, *tile_index));
                    }
                }
            }
        }

        self.bounds = Some((0, 0, max_x - min_x, max_y - min_y));
        self.source_layer = editor_state.selected_layer;
    }

    /// Get the width and height of the clipboard content
    pub fn dimensions(&self) -> Option<(u32, u32)> {
        self.bounds.map(|(min_x, min_y, max_x, max_y)| {
            ((max_x - min_x + 1) as u32, (max_y - min_y + 1) as u32)
        })
    }
}

/// Tile selection state
#[derive(Default, Clone)]
pub struct TileSelection {
    /// Selected tiles: (level_id, layer_index, x, y)
    pub tiles: HashSet<(Uuid, usize, u32, u32)>,
    /// Marquee drag start position (tile coordinates)
    pub drag_start: Option<(i32, i32)>,
    /// Whether we're currently in a selection drag
    pub is_selecting: bool,
}

impl TileSelection {
    /// Clear all selected tiles
    pub fn clear(&mut self) {
        self.tiles.clear();
        self.drag_start = None;
        self.is_selecting = false;
    }

    /// Check if there are selected tiles
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    /// Select a single tile (clears previous selection unless shift is held)
    pub fn select_tile(&mut self, level_id: Uuid, layer_idx: usize, x: u32, y: u32, add_to_selection: bool) {
        if !add_to_selection {
            self.tiles.clear();
        }
        self.tiles.insert((level_id, layer_idx, x, y));
    }

    /// Toggle selection of a tile
    pub fn toggle_tile(&mut self, level_id: Uuid, layer_idx: usize, x: u32, y: u32) {
        let key = (level_id, layer_idx, x, y);
        if self.tiles.contains(&key) {
            self.tiles.remove(&key);
        } else {
            self.tiles.insert(key);
        }
    }

    /// Select all tiles in a rectangle
    pub fn select_rectangle(
        &mut self,
        level_id: Uuid,
        layer_idx: usize,
        min_x: u32,
        min_y: u32,
        max_x: u32,
        max_y: u32,
        add_to_selection: bool,
    ) {
        if !add_to_selection {
            self.tiles.clear();
        }

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                self.tiles.insert((level_id, layer_idx, x, y));
            }
        }
    }

    /// Check if a tile is selected
    pub fn is_selected(&self, level_id: Uuid, layer_idx: usize, x: u32, y: u32) -> bool {
        self.tiles.contains(&(level_id, layer_idx, x, y))
    }

    /// Get bounding box of selection: (min_x, min_y, max_x, max_y)
    pub fn bounds(&self) -> Option<(u32, u32, u32, u32)> {
        if self.tiles.is_empty() {
            return None;
        }

        let mut min_x = u32::MAX;
        let mut min_y = u32::MAX;
        let mut max_x = 0u32;
        let mut max_y = 0u32;

        for (_, _, x, y) in &self.tiles {
            min_x = min_x.min(*x);
            min_y = min_y.min(*y);
            max_x = max_x.max(*x);
            max_y = max_y.max(*y);
        }

        Some((min_x, min_y, max_x, max_y))
    }

    /// Remove all tiles from selection that don't match the given level and layer
    pub fn filter_to_layer(&mut self, level_id: Uuid, layer_idx: usize) {
        self.tiles.retain(|(lid, li, _, _)| *lid == level_id && *li == layer_idx);
    }
}
