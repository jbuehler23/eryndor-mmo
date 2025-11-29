use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// TILEMAP DATA STRUCTURES
// ============================================================================

/// Tilemap data for a zone - stored in zone JSON files
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ZoneTilemap {
    /// Size of each tile in pixels (typically 16)
    pub tile_size: u32,
    /// Number of tiles per chunk (typically 16, so 16x16 = 256 tiles per chunk)
    pub chunk_size: u32,
    /// Map of chunk coordinates to chunk data, keyed by "x_y" string
    pub chunks: HashMap<String, TileChunk>,
}

/// A single chunk of tilemap data
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TileChunk {
    /// Ground layer - 2D array of tile IDs (grass, water, paths, etc.)
    #[serde(default)]
    pub ground: Vec<Vec<u32>>,
    /// Decoration layer - 2D array of decoration IDs (trees, flowers, etc.)
    #[serde(default)]
    pub decorations: Vec<Vec<u32>>,
    /// Collision layer - 2D array of collision flags (0 = passable, 1 = blocked)
    #[serde(default)]
    pub collision: Vec<Vec<u8>>,
}

impl TileChunk {
    /// Create a new empty chunk with the specified size
    pub fn new(size: usize) -> Self {
        Self {
            ground: vec![vec![0; size]; size],
            decorations: vec![vec![0; size]; size],
            collision: vec![vec![0; size]; size],
        }
    }

    /// Get ground tile at local position
    pub fn get_ground(&self, x: usize, y: usize) -> Option<u32> {
        self.ground.get(y).and_then(|row| row.get(x)).copied()
    }

    /// Set ground tile at local position
    pub fn set_ground(&mut self, x: usize, y: usize, tile_id: u32) {
        if let Some(row) = self.ground.get_mut(y) {
            if let Some(cell) = row.get_mut(x) {
                *cell = tile_id;
            }
        }
    }

    /// Get decoration at local position
    pub fn get_decoration(&self, x: usize, y: usize) -> Option<u32> {
        self.decorations.get(y).and_then(|row| row.get(x)).copied()
    }

    /// Set decoration at local position
    pub fn set_decoration(&mut self, x: usize, y: usize, tile_id: u32) {
        if let Some(row) = self.decorations.get_mut(y) {
            if let Some(cell) = row.get_mut(x) {
                *cell = tile_id;
            }
        }
    }

    /// Check if position is blocked
    pub fn is_blocked(&self, x: usize, y: usize) -> bool {
        self.collision
            .get(y)
            .and_then(|row| row.get(x))
            .map(|&v| v != 0)
            .unwrap_or(false)
    }

    /// Set collision at local position
    pub fn set_collision(&mut self, x: usize, y: usize, blocked: bool) {
        if let Some(row) = self.collision.get_mut(y) {
            if let Some(cell) = row.get_mut(x) {
                *cell = if blocked { 1 } else { 0 };
            }
        }
    }
}

impl ZoneTilemap {
    /// Create a new empty tilemap with default settings
    pub fn new() -> Self {
        Self {
            tile_size: 16,
            chunk_size: 16,
            chunks: HashMap::new(),
        }
    }

    /// Get chunk key from chunk coordinates
    pub fn chunk_key(chunk_x: i32, chunk_y: i32) -> String {
        format!("{}_{}", chunk_x, chunk_y)
    }

    /// Parse chunk key into coordinates
    pub fn parse_chunk_key(key: &str) -> Option<(i32, i32)> {
        let parts: Vec<&str> = key.split('_').collect();
        if parts.len() == 2 {
            let x = parts[0].parse().ok()?;
            let y = parts[1].parse().ok()?;
            Some((x, y))
        } else {
            None
        }
    }

    /// Get or create a chunk at the specified coordinates
    pub fn get_or_create_chunk(&mut self, chunk_x: i32, chunk_y: i32) -> &mut TileChunk {
        let key = Self::chunk_key(chunk_x, chunk_y);
        self.chunks
            .entry(key)
            .or_insert_with(|| TileChunk::new(self.chunk_size as usize))
    }

    /// Get chunk at coordinates (if it exists)
    pub fn get_chunk(&self, chunk_x: i32, chunk_y: i32) -> Option<&TileChunk> {
        let key = Self::chunk_key(chunk_x, chunk_y);
        self.chunks.get(&key)
    }

    /// Convert world position to tile coordinates
    pub fn world_to_tile(&self, world_x: f32, world_y: f32) -> (i32, i32) {
        let tile_x = (world_x / self.tile_size as f32).floor() as i32;
        let tile_y = (world_y / self.tile_size as f32).floor() as i32;
        (tile_x, tile_y)
    }

    /// Convert tile coordinates to chunk and local coordinates
    pub fn tile_to_chunk_local(&self, tile_x: i32, tile_y: i32) -> (i32, i32, usize, usize) {
        let chunk_size = self.chunk_size as i32;
        let chunk_x = tile_x.div_euclid(chunk_size);
        let chunk_y = tile_y.div_euclid(chunk_size);
        let local_x = tile_x.rem_euclid(chunk_size) as usize;
        let local_y = tile_y.rem_euclid(chunk_size) as usize;
        (chunk_x, chunk_y, local_x, local_y)
    }
}

// ============================================================================
// TILE PALETTE STRUCTURES
// ============================================================================

/// Tile palette definition - maps tile IDs to sprite assets
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TilePalette {
    pub palette_id: String,
    pub tile_size: u32,
    pub categories: TilePaletteCategories,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TilePaletteCategories {
    #[serde(default)]
    pub ground: HashMap<String, TileEntry>,
    #[serde(default)]
    pub decorations: HashMap<String, TileEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TileEntry {
    /// Unique ID for this tile
    pub id: u32,
    /// Path to the sprite asset (relative to assets/)
    pub path: String,
    /// Size override for multi-tile sprites [width, height] in pixels
    #[serde(default)]
    pub size: Option<[u32; 2]>,
    /// Sprite index within a sprite sheet
    #[serde(default)]
    pub sprite_index: Option<u32>,
    /// Whether this tile blocks movement
    #[serde(default)]
    pub collision: bool,
    /// Whether this tile has animation
    #[serde(default)]
    pub animated: bool,
}

impl Default for TilePalette {
    fn default() -> Self {
        Self {
            palette_id: "default".to_string(),
            tile_size: 16,
            categories: TilePaletteCategories::default(),
        }
    }
}

impl TilePalette {
    /// Get tile entry by ID from any category
    pub fn get_by_id(&self, id: u32) -> Option<&TileEntry> {
        self.categories
            .ground
            .values()
            .find(|e| e.id == id)
            .or_else(|| self.categories.decorations.values().find(|e| e.id == id))
    }

    /// Check if a tile ID represents a ground tile
    pub fn is_ground_tile(&self, id: u32) -> bool {
        self.categories.ground.values().any(|e| e.id == id)
    }

    /// Check if a tile ID represents a decoration
    pub fn is_decoration(&self, id: u32) -> bool {
        self.categories.decorations.values().any(|e| e.id == id)
    }
}

// ============================================================================
// TILE LAYER ENUM
// ============================================================================

/// Tile layer for editor and rendering
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileLayer {
    #[default]
    Ground,
    Decorations,
    Collision,
}
