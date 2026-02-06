use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy_editor_formats::TilesetData;
use std::collections::HashMap;

/// Manages loaded tilesets for the editor
#[derive(Resource, Default)]
pub struct TilesetManager {
    /// All loaded tilesets
    pub tilesets: HashMap<u32, TilesetInfo>,
    /// Currently selected tileset ID
    pub selected_tileset_id: Option<u32>,
    /// Currently selected tile within the tileset (for single-tile selection)
    pub selected_tile_id: Option<u32>,
    /// Multi-tile selection for stamp brushes (list of tile IDs)
    pub selected_tiles: Vec<u32>,
    /// Rectangle selection start position (col, row)
    pub selection_start: Option<(u32, u32)>,
    /// Rectangle selection end position (col, row)
    pub selection_end: Option<(u32, u32)>,
    /// Next available tileset ID
    next_id: u32,
}

/// Information about a loaded tileset
pub struct TilesetInfo {
    pub data: TilesetData,
    pub texture_handle: Handle<Image>,
    /// Cached tile count for quick access
    pub tile_count: u32,
}

impl TilesetManager {
    pub fn new() -> Self {
        Self {
            tilesets: HashMap::new(),
            selected_tileset_id: None,
            selected_tile_id: None,
            selected_tiles: Vec::new(),
            selection_start: None,
            selection_end: None,
            next_id: 0,
        }
    }

    /// Add a new tileset
    pub fn add_tileset(&mut self, mut data: TilesetData, texture_handle: Handle<Image>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        data.id = id;

        let tile_count = data.columns * data.rows;

        self.tilesets.insert(
            id,
            TilesetInfo {
                data,
                texture_handle,
                tile_count,
            },
        );

        // Auto-select first tileset
        if self.selected_tileset_id.is_none() {
            self.selected_tileset_id = Some(id);
        }

        id
    }

    /// Remove a tileset
    pub fn remove_tileset(&mut self, id: u32) {
        self.tilesets.remove(&id);
        if self.selected_tileset_id == Some(id) {
            self.selected_tileset_id = self.tilesets.keys().next().copied();
        }
    }

    /// Get tileset by ID
    pub fn get_tileset(&self, id: u32) -> Option<&TilesetInfo> {
        self.tilesets.get(&id)
    }

    /// Get currently selected tileset
    pub fn get_selected_tileset(&self) -> Option<&TilesetInfo> {
        self.selected_tileset_id
            .and_then(|id| self.tilesets.get(&id))
    }

    /// Select a tileset
    pub fn select_tileset(&mut self, id: u32) {
        if self.tilesets.contains_key(&id) {
            self.selected_tileset_id = Some(id);
            // Reset selected tile when changing tilesets
            self.selected_tile_id = None;
        }
    }

    /// Select a tile within the current tileset
    pub fn select_tile(&mut self, tile_id: u32) {
        if let Some(tileset) = self.get_selected_tileset() {
            if tile_id < tileset.tile_count {
                self.selected_tile_id = Some(tile_id);
            }
        }
    }

    /// Get currently selected tile ID
    pub fn get_selected_tile(&self) -> Option<u32> {
        self.selected_tile_id
    }

    /// Check if a tile is currently selected
    pub fn is_tile_selected(&self, tile_id: u32) -> bool {
        self.selected_tile_id == Some(tile_id)
    }

    /// Get tile grid position from tile ID
    pub fn tile_id_to_grid(&self, tile_id: u32) -> Option<(u32, u32)> {
        let tileset = self.get_selected_tileset()?;
        let x = tile_id % tileset.data.columns;
        let y = tile_id / tileset.data.columns;
        Some((x, y))
    }

    /// Get tile ID from grid position
    pub fn grid_to_tile_id(&self, x: u32, y: u32) -> Option<u32> {
        let tileset = self.get_selected_tileset()?;
        if x < tileset.data.columns && y < tileset.data.rows {
            Some(y * tileset.data.columns + x)
        } else {
            None
        }
    }

    /// Clear all tilesets
    pub fn clear(&mut self) {
        self.tilesets.clear();
        self.selected_tileset_id = None;
        self.selected_tile_id = None;
        self.selected_tiles.clear();
        self.selection_start = None;
        self.selection_end = None;
        self.next_id = 0;
    }

    /// Start a rectangular selection at the given grid position
    pub fn start_rect_selection(&mut self, col: u32, row: u32) {
        self.selection_start = Some((col, row));
        self.selection_end = Some((col, row));
    }

    /// Update the end point of rectangular selection
    pub fn update_rect_selection(&mut self, col: u32, row: u32) {
        self.selection_end = Some((col, row));
        self.update_selected_tiles();
    }

    /// Finalize rectangular selection
    pub fn finish_rect_selection(&mut self) {
        self.update_selected_tiles();
    }

    /// Clear multi-tile selection
    pub fn clear_multi_selection(&mut self) {
        self.selected_tiles.clear();
        self.selection_start = None;
        self.selection_end = None;
    }

    /// Update the selected_tiles list based on selection_start and selection_end
    fn update_selected_tiles(&mut self) {
        self.selected_tiles.clear();

        if let (Some((start_col, start_row)), Some((end_col, end_row))) =
            (self.selection_start, self.selection_end)
        {
            // Get tile count and columns without borrowing self
            let (tile_count, columns) = if let Some(tileset_id) = self.selected_tileset_id {
                if let Some(tileset) = self.tilesets.get(&tileset_id) {
                    (tileset.tile_count, tileset.data.columns)
                } else {
                    return;
                }
            } else {
                return;
            };

            let min_col = start_col.min(end_col);
            let max_col = start_col.max(end_col);
            let min_row = start_row.min(end_row);
            let max_row = start_row.max(end_row);

            for row in min_row..=max_row {
                for col in min_col..=max_col {
                    if col < columns {
                        let tile_id = row * columns + col;
                        if tile_id < tile_count {
                            self.selected_tiles.push(tile_id);
                        }
                    }
                }
            }
        }
    }

    /// Get the selection rectangle dimensions (width, height)
    pub fn get_selection_dimensions(&self) -> Option<(u32, u32)> {
        if let (Some((start_col, start_row)), Some((end_col, end_row))) =
            (self.selection_start, self.selection_end)
        {
            let width = (start_col.max(end_col) - start_col.min(end_col)) + 1;
            let height = (start_row.max(end_row) - start_row.min(end_row)) + 1;
            Some((width, height))
        } else {
            None
        }
    }

    /// Clear stamp selection and return to single-tile mode
    pub fn clear_stamp_selection(&mut self) {
        self.selection_start = None;
        self.selection_end = None;
        self.selected_tiles.clear();
    }

    /// Check if currently in stamp mode (multi-tile selection)
    pub fn is_stamp_mode(&self) -> bool {
        self.selected_tiles.len() > 1
    }
}

/// Load a tileset from a file path
pub fn load_tileset(
    path: &str,
    identifier: &str,
    tile_width: u32,
    tile_height: u32,
    asset_server: &AssetServer,
) -> (TilesetData, Handle<Image>) {
    let texture_handle = asset_server.load(path.to_string());

    // Calculate columns and rows from image dimensions
    // Note: In a real implementation, you'd need to load the image first to get dimensions
    // For now, we'll use default values and they can be updated later
    let columns = 16; // Default, should be calculated from actual image
    let rows = 16; // Default, should be calculated from actual image

    let data = TilesetData {
        id: 0, // Will be set by TilesetManager
        identifier: identifier.to_string(),
        texture_path: path.to_string(),
        tile_width,
        tile_height,
        columns,
        rows,
        spacing: 0,
        padding: 0,
        collision_data: std::collections::HashMap::new(),
    };

    (data, texture_handle)
}

/// System to handle tileset loading requests
/// This system waits for the image to load, then calculates the actual grid dimensions
pub fn handle_tileset_load_requests(
    _commands: Commands,
    asset_server: Res<AssetServer>,
    mut tileset_manager: ResMut<TilesetManager>,
    mut images: ResMut<Assets<Image>>,
    mut load_requests: MessageReader<LoadTilesetEvent>,
) {
    for event in load_requests.read() {
        // Load the texture handle
        let texture_handle = asset_server.load(&event.path);

        // Try to get the image immediately (might not be loaded yet)
        if let Some(image) = images.get_mut(&texture_handle) {
            // Image is loaded, calculate dimensions
            let image_width = image.width();
            let image_height = image.height();

            let columns = image_width / event.tile_width;
            let rows = image_height / event.tile_height;

            // Validate dimensions
            if image_width % event.tile_width != 0 {
                warn!(
                    "Tileset '{}': Image width ({}) is not evenly divisible by tile width ({})",
                    event.identifier, image_width, event.tile_width
                );
            }
            if image_height % event.tile_height != 0 {
                warn!(
                    "Tileset '{}': Image height ({}) is not evenly divisible by tile height ({})",
                    event.identifier, image_height, event.tile_height
                );
            }

            // Configure nearest-neighbor sampling for pixel-perfect rendering
            image.sampler = ImageSampler::nearest();

            let data = TilesetData {
                id: 0,
                identifier: event.identifier.clone(),
                texture_path: event.path.clone(),
                tile_width: event.tile_width,
                tile_height: event.tile_height,
                columns,
                rows,
                spacing: 0,
                padding: 0,
                collision_data: std::collections::HashMap::new(),
            };

            let tileset_id = tileset_manager.add_tileset(data, texture_handle.clone());
            info!(
                "Loaded tileset '{}' with ID {} ({}x{} tiles, {}x{} image)",
                event.identifier, tileset_id, columns, rows, image_width, image_height
            );
        } else {
            // Image not loaded yet, create temporary tileset with default dimensions
            // It will be updated when the image loads
            warn!(
                "Tileset '{}': Image not immediately available, using default 16x16 grid. Will update when loaded.",
                event.identifier
            );

            let data = TilesetData {
                id: 0,
                identifier: event.identifier.clone(),
                texture_path: event.path.clone(),
                tile_width: event.tile_width,
                tile_height: event.tile_height,
                columns: 16,
                rows: 16,
                spacing: 0,
                padding: 0,
                collision_data: std::collections::HashMap::new(),
            };

            let tileset_id = tileset_manager.add_tileset(data, texture_handle);
            info!(
                "Queued tileset '{}' for loading with ID {}",
                event.identifier, tileset_id
            );
        }
    }
}

/// System to update tileset dimensions once images are loaded
/// This handles the case where images weren't immediately available
pub fn update_tileset_dimensions(
    mut tileset_manager: ResMut<TilesetManager>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut updates = Vec::new();

    // Check each tileset to see if its image is now loaded
    for (id, tileset_info) in tileset_manager.tilesets.iter() {
        // Check if this tileset is using default dimensions (needs update)
        if tileset_info.data.columns == 16 && tileset_info.data.rows == 16 {
            if let Some(image) = images.get_mut(&tileset_info.texture_handle) {
                let image_width = image.width();
                let image_height = image.height();

                let columns = image_width / tileset_info.data.tile_width;
                let rows = image_height / tileset_info.data.tile_height;

                // Only update if dimensions actually changed
                if columns != tileset_info.data.columns || rows != tileset_info.data.rows {
                    // Configure nearest-neighbor sampling
                    image.sampler = ImageSampler::nearest();

                    updates.push((*id, columns, rows, image_width, image_height));
                }
            }
        }
    }

    // Apply updates
    for (id, columns, rows, image_width, image_height) in updates {
        if let Some(tileset_info) = tileset_manager.tilesets.get_mut(&id) {
            tileset_info.data.columns = columns;
            tileset_info.data.rows = rows;
            tileset_info.tile_count = columns * rows;

            info!(
                "Updated tileset '{}' dimensions: {}x{} tiles ({}x{} image)",
                tileset_info.data.identifier, columns, rows, image_width, image_height
            );
        }
    }
}

/// Event to request loading a tileset
#[derive(Event, Message)]
pub struct LoadTilesetEvent {
    pub path: String,
    pub identifier: String,
    pub tile_width: u32,
    pub tile_height: u32,
}

impl LoadTilesetEvent {
    pub fn new(path: &str, identifier: &str, tile_width: u32, tile_height: u32) -> Self {
        Self {
            path: path.to_string(),
            identifier: identifier.to_string(),
            tile_width,
            tile_height,
        }
    }
}
