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

/// Tile layer for editor and rendering (legacy - use MapLayer instead)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileLayer {
    #[default]
    Ground,
    Decorations,
    Collision,
}

// ============================================================================
// TILED-COMPATIBLE MAP STRUCTURES
// ============================================================================

// Serde default helpers
fn default_true() -> bool {
    true
}
fn default_one() -> f32 {
    1.0
}
fn default_orientation() -> String {
    "orthogonal".to_string()
}
fn default_render_order() -> String {
    "right-down".to_string()
}
fn default_chunk_size() -> u32 {
    16
}

/// A complete tilemap following Tiled JSON format
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TilemapMap {
    /// Map width in tiles (0 for infinite maps)
    #[serde(default)]
    pub width: u32,
    /// Map height in tiles (0 for infinite maps)
    #[serde(default)]
    pub height: u32,
    /// Width of each tile in pixels
    #[serde(rename = "tilewidth")]
    pub tile_width: u32,
    /// Height of each tile in pixels
    #[serde(rename = "tileheight")]
    pub tile_height: u32,
    /// Whether this is an infinite (chunk-based) map
    #[serde(default)]
    pub infinite: bool,
    /// Map orientation: "orthogonal", "isometric", "staggered", "hexagonal"
    #[serde(default = "default_orientation")]
    pub orientation: String,
    /// Tile render order: "right-down", "right-up", "left-down", "left-up"
    #[serde(default = "default_render_order")]
    pub renderorder: String,
    /// All layers in the map (rendered bottom to top)
    #[serde(default)]
    pub layers: Vec<MapLayer>,
    /// Tileset references used by this map
    #[serde(default)]
    pub tilesets: Vec<TilesetRef>,
    /// Next available layer ID
    #[serde(default)]
    pub nextlayerid: u32,
    /// Next available object ID
    #[serde(default)]
    pub nextobjectid: u32,
    /// Custom properties
    #[serde(default)]
    pub properties: Vec<CustomProperty>,
}

/// A layer in the map (can be tile layer, object group, image layer, or group)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MapLayer {
    /// Unique ID for this layer
    pub id: u32,
    /// Layer name (user-defined)
    pub name: String,
    /// Layer type: "tilelayer", "objectgroup", "imagelayer", "group"
    #[serde(rename = "type")]
    pub layer_type: String,
    /// Whether layer is visible
    #[serde(default = "default_true")]
    pub visible: bool,
    /// Layer opacity (0.0 to 1.0)
    #[serde(default = "default_one")]
    pub opacity: f32,
    /// Horizontal offset in pixels
    #[serde(default)]
    pub offsetx: f32,
    /// Vertical offset in pixels
    #[serde(default)]
    pub offsety: f32,
    /// Horizontal parallax factor (1.0 = normal scrolling)
    #[serde(default = "default_one")]
    pub parallaxx: f32,
    /// Vertical parallax factor (1.0 = normal scrolling)
    #[serde(default = "default_one")]
    pub parallaxy: f32,
    /// Tint color in "#AARRGGBB" or "#RRGGBB" format
    #[serde(default)]
    pub tintcolor: Option<String>,
    /// Whether layer is locked for editing
    #[serde(default)]
    pub locked: bool,

    // === Tile Layer specific fields ===
    /// Tile data chunks (for infinite maps)
    #[serde(default)]
    pub chunks: Option<Vec<LayerTileChunk>>,
    /// Flat tile data array (for fixed-size maps)
    #[serde(default)]
    pub data: Option<Vec<u32>>,
    /// Layer width in tiles (for fixed-size tile layers)
    #[serde(default)]
    pub width: Option<u32>,
    /// Layer height in tiles (for fixed-size tile layers)
    #[serde(default)]
    pub height: Option<u32>,

    // === Object Layer specific fields ===
    /// Objects in this layer (for objectgroup type)
    #[serde(default)]
    pub objects: Option<Vec<MapObject>>,
    /// Draw order for objects: "topdown" or "index"
    #[serde(default)]
    pub draworder: Option<String>,

    // === Image Layer specific fields ===
    /// Image path (for imagelayer type)
    #[serde(default)]
    pub image: Option<String>,
    /// Whether image repeats horizontally
    #[serde(default)]
    pub repeatx: bool,
    /// Whether image repeats vertically
    #[serde(default)]
    pub repeaty: bool,

    // === Group Layer specific fields ===
    /// Child layers (for group type)
    #[serde(rename = "layers", default)]
    pub sublayers: Option<Vec<MapLayer>>,

    /// Custom properties
    #[serde(default)]
    pub properties: Vec<CustomProperty>,
}

/// A chunk of tile data for infinite maps
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LayerTileChunk {
    /// X position of chunk in tile coordinates
    pub x: i32,
    /// Y position of chunk in tile coordinates
    pub y: i32,
    /// Chunk width in tiles
    #[serde(default = "default_chunk_size")]
    pub width: u32,
    /// Chunk height in tiles
    #[serde(default = "default_chunk_size")]
    pub height: u32,
    /// Tile GIDs in row-major order (left-to-right, top-to-bottom)
    pub data: Vec<u32>,
}

/// An object in an object layer
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MapObject {
    /// Unique object ID
    pub id: u32,
    /// Object name
    #[serde(default)]
    pub name: String,
    /// Object type/class
    #[serde(rename = "type", default)]
    pub obj_type: String,
    /// X position in pixels
    pub x: f32,
    /// Y position in pixels
    pub y: f32,
    /// Width in pixels (for rectangle/ellipse)
    #[serde(default)]
    pub width: f32,
    /// Height in pixels (for rectangle/ellipse)
    #[serde(default)]
    pub height: f32,
    /// Rotation in degrees clockwise
    #[serde(default)]
    pub rotation: f32,
    /// Whether object is visible
    #[serde(default = "default_true")]
    pub visible: bool,
    /// Global tile ID (for tile objects)
    #[serde(default)]
    pub gid: Option<u32>,
    /// True if this is an ellipse
    #[serde(default)]
    pub ellipse: bool,
    /// True if this is a point
    #[serde(default)]
    pub point: bool,
    /// Polygon points (relative to x,y)
    #[serde(default)]
    pub polygon: Option<Vec<PolyPoint>>,
    /// Polyline points (relative to x,y)
    #[serde(default)]
    pub polyline: Option<Vec<PolyPoint>>,
    /// Text content (for text objects)
    #[serde(default)]
    pub text: Option<TextData>,
    /// Custom properties
    #[serde(default)]
    pub properties: Vec<CustomProperty>,
}

/// A point in a polygon or polyline
#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct PolyPoint {
    pub x: f32,
    pub y: f32,
}

/// Text object data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextData {
    pub text: String,
    #[serde(default)]
    pub fontfamily: Option<String>,
    #[serde(default)]
    pub pixelsize: Option<u32>,
    #[serde(default)]
    pub wrap: bool,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub halign: Option<String>,
    #[serde(default)]
    pub valign: Option<String>,
}

/// Reference to a tileset in the map
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TilesetRef {
    /// First global tile ID for this tileset
    pub firstgid: u32,
    /// Path to tileset JSON file (external tileset)
    #[serde(default)]
    pub source: Option<String>,
    // Embedded tileset data (if not external)
    #[serde(flatten)]
    pub embedded: Option<EmbeddedTileset>,
}

/// Embedded tileset data (when not using external file)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmbeddedTileset {
    pub name: String,
    #[serde(rename = "tilewidth")]
    pub tile_width: u32,
    #[serde(rename = "tileheight")]
    pub tile_height: u32,
    #[serde(rename = "tilecount")]
    pub tile_count: u32,
    pub columns: u32,
    #[serde(default)]
    pub margin: u32,
    #[serde(default)]
    pub spacing: u32,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(rename = "imagewidth", default)]
    pub image_width: Option<u32>,
    #[serde(rename = "imageheight", default)]
    pub image_height: Option<u32>,
}

/// Custom property (Tiled-compatible)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomProperty {
    /// Property name
    pub name: String,
    /// Property type: "string", "int", "float", "bool", "color", "file", "object"
    #[serde(rename = "type", default = "default_string_type")]
    pub prop_type: String,
    /// Property value
    pub value: serde_json::Value,
}

fn default_string_type() -> String {
    "string".to_string()
}

// ============================================================================
// TILEMAPMAP IMPLEMENTATION
// ============================================================================

impl Default for TilemapMap {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            tile_width: 16,
            tile_height: 16,
            infinite: true,
            orientation: default_orientation(),
            renderorder: default_render_order(),
            layers: Vec::new(),
            tilesets: Vec::new(),
            nextlayerid: 1,
            nextobjectid: 1,
            properties: Vec::new(),
        }
    }
}

impl TilemapMap {
    /// Create a new empty tilemap
    pub fn new(tile_width: u32, tile_height: u32) -> Self {
        Self {
            tile_width,
            tile_height,
            ..Default::default()
        }
    }

    /// Get tileset index and local tile ID from a global tile ID (GID)
    /// Returns None for GID 0 (empty tile) or invalid GIDs
    pub fn gid_to_tileset(&self, gid: u32) -> Option<(usize, u32)> {
        // GID 0 is always empty
        if gid == 0 {
            return None;
        }

        // Strip flip flags (highest 3 bits in Tiled format)
        let raw_gid = gid & 0x1FFFFFFF;

        // Find tileset with highest firstgid <= raw_gid
        let mut best_match: Option<(usize, u32)> = None;
        for (idx, ts_ref) in self.tilesets.iter().enumerate() {
            if ts_ref.firstgid <= raw_gid {
                match best_match {
                    None => best_match = Some((idx, ts_ref.firstgid)),
                    Some((_, prev_gid)) if ts_ref.firstgid > prev_gid => {
                        best_match = Some((idx, ts_ref.firstgid));
                    }
                    _ => {}
                }
            }
        }

        best_match.map(|(idx, firstgid)| (idx, raw_gid - firstgid))
    }

    /// Create a new tile layer and add it to the map
    pub fn add_tile_layer(&mut self, name: &str) -> u32 {
        let id = self.nextlayerid;
        self.nextlayerid += 1;

        let layer = MapLayer::new_tile_layer(id, name);
        self.layers.push(layer);
        id
    }

    /// Create a new object layer and add it to the map
    pub fn add_object_layer(&mut self, name: &str) -> u32 {
        let id = self.nextlayerid;
        self.nextlayerid += 1;

        let layer = MapLayer::new_object_layer(id, name);
        self.layers.push(layer);
        id
    }

    /// Get a layer by ID
    pub fn get_layer(&self, id: u32) -> Option<&MapLayer> {
        self.layers.iter().find(|l| l.id == id)
    }

    /// Get a mutable layer by ID
    pub fn get_layer_mut(&mut self, id: u32) -> Option<&mut MapLayer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    /// Get layer by name
    pub fn get_layer_by_name(&self, name: &str) -> Option<&MapLayer> {
        self.layers.iter().find(|l| l.name == name)
    }

    /// Remove a layer by ID
    pub fn remove_layer(&mut self, id: u32) -> Option<MapLayer> {
        if let Some(pos) = self.layers.iter().position(|l| l.id == id) {
            Some(self.layers.remove(pos))
        } else {
            None
        }
    }

    /// Move a layer up in the stack (toward front/top)
    pub fn move_layer_up(&mut self, id: u32) -> bool {
        if let Some(pos) = self.layers.iter().position(|l| l.id == id) {
            if pos + 1 < self.layers.len() {
                self.layers.swap(pos, pos + 1);
                return true;
            }
        }
        false
    }

    /// Move a layer down in the stack (toward back/bottom)
    pub fn move_layer_down(&mut self, id: u32) -> bool {
        if let Some(pos) = self.layers.iter().position(|l| l.id == id) {
            if pos > 0 {
                self.layers.swap(pos, pos - 1);
                return true;
            }
        }
        false
    }

    /// Convert world position to tile coordinates
    pub fn world_to_tile(&self, world_x: f32, world_y: f32) -> (i32, i32) {
        let tile_x = (world_x / self.tile_width as f32).floor() as i32;
        let tile_y = (world_y / self.tile_height as f32).floor() as i32;
        (tile_x, tile_y)
    }

    /// Convert tile coordinates to chunk and local coordinates
    pub fn tile_to_chunk_local(&self, tile_x: i32, tile_y: i32, chunk_size: u32) -> (i32, i32, usize, usize) {
        let chunk_size_i32 = chunk_size as i32;
        let chunk_x = tile_x.div_euclid(chunk_size_i32);
        let chunk_y = tile_y.div_euclid(chunk_size_i32);
        let local_x = tile_x.rem_euclid(chunk_size_i32) as usize;
        let local_y = tile_y.rem_euclid(chunk_size_i32) as usize;
        (chunk_x, chunk_y, local_x, local_y)
    }

    /// Get chunk key from chunk coordinates
    pub fn chunk_key(chunk_x: i32, chunk_y: i32) -> String {
        format!("{}_{}", chunk_x, chunk_y)
    }
}

// ============================================================================
// MAPLAYER IMPLEMENTATION
// ============================================================================

impl MapLayer {
    /// Create a new tile layer
    pub fn new_tile_layer(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            layer_type: "tilelayer".to_string(),
            visible: true,
            opacity: 1.0,
            offsetx: 0.0,
            offsety: 0.0,
            parallaxx: 1.0,
            parallaxy: 1.0,
            tintcolor: None,
            locked: false,
            chunks: Some(Vec::new()),
            data: None,
            width: None,
            height: None,
            objects: None,
            draworder: None,
            image: None,
            repeatx: false,
            repeaty: false,
            sublayers: None,
            properties: Vec::new(),
        }
    }

    /// Create a new object layer
    pub fn new_object_layer(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            layer_type: "objectgroup".to_string(),
            visible: true,
            opacity: 1.0,
            offsetx: 0.0,
            offsety: 0.0,
            parallaxx: 1.0,
            parallaxy: 1.0,
            tintcolor: None,
            locked: false,
            chunks: None,
            data: None,
            width: None,
            height: None,
            objects: Some(Vec::new()),
            draworder: Some("topdown".to_string()),
            image: None,
            repeatx: false,
            repeaty: false,
            sublayers: None,
            properties: Vec::new(),
        }
    }

    /// Create a new image layer
    pub fn new_image_layer(id: u32, name: &str, image_path: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            layer_type: "imagelayer".to_string(),
            visible: true,
            opacity: 1.0,
            offsetx: 0.0,
            offsety: 0.0,
            parallaxx: 1.0,
            parallaxy: 1.0,
            tintcolor: None,
            locked: false,
            chunks: None,
            data: None,
            width: None,
            height: None,
            objects: None,
            draworder: None,
            image: Some(image_path.to_string()),
            repeatx: false,
            repeaty: false,
            sublayers: None,
            properties: Vec::new(),
        }
    }

    /// Create a new group layer
    pub fn new_group_layer(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            layer_type: "group".to_string(),
            visible: true,
            opacity: 1.0,
            offsetx: 0.0,
            offsety: 0.0,
            parallaxx: 1.0,
            parallaxy: 1.0,
            tintcolor: None,
            locked: false,
            chunks: None,
            data: None,
            width: None,
            height: None,
            objects: None,
            draworder: None,
            image: None,
            repeatx: false,
            repeaty: false,
            sublayers: Some(Vec::new()),
            properties: Vec::new(),
        }
    }

    /// Check if this is a tile layer
    pub fn is_tile_layer(&self) -> bool {
        self.layer_type == "tilelayer"
    }

    /// Check if this is an object layer
    pub fn is_object_layer(&self) -> bool {
        self.layer_type == "objectgroup"
    }

    /// Check if this is an image layer
    pub fn is_image_layer(&self) -> bool {
        self.layer_type == "imagelayer"
    }

    /// Check if this is a group layer
    pub fn is_group_layer(&self) -> bool {
        self.layer_type == "group"
    }

    /// Get tile at position in a tile layer (for infinite maps with chunks)
    pub fn get_tile(&self, tile_x: i32, tile_y: i32, chunk_size: u32) -> Option<u32> {
        if !self.is_tile_layer() {
            return None;
        }

        let chunks = self.chunks.as_ref()?;
        let chunk_size_i32 = chunk_size as i32;
        let chunk_x = tile_x.div_euclid(chunk_size_i32);
        let chunk_y = tile_y.div_euclid(chunk_size_i32);

        // Find the chunk
        let chunk = chunks.iter().find(|c| c.x == chunk_x * chunk_size_i32 && c.y == chunk_y * chunk_size_i32)?;

        let local_x = tile_x.rem_euclid(chunk_size_i32) as usize;
        let local_y = tile_y.rem_euclid(chunk_size_i32) as usize;
        let index = local_y * chunk.width as usize + local_x;

        chunk.data.get(index).copied()
    }

    /// Set tile at position in a tile layer (creates chunk if needed)
    pub fn set_tile(&mut self, tile_x: i32, tile_y: i32, gid: u32, chunk_size: u32) {
        if !self.is_tile_layer() {
            return;
        }

        let chunks = self.chunks.get_or_insert_with(Vec::new);
        let chunk_size_i32 = chunk_size as i32;
        let chunk_x = tile_x.div_euclid(chunk_size_i32) * chunk_size_i32;
        let chunk_y = tile_y.div_euclid(chunk_size_i32) * chunk_size_i32;

        // Find or create the chunk
        let chunk = if let Some(chunk) = chunks.iter_mut().find(|c| c.x == chunk_x && c.y == chunk_y) {
            chunk
        } else {
            // Create new chunk
            let new_chunk = LayerTileChunk {
                x: chunk_x,
                y: chunk_y,
                width: chunk_size,
                height: chunk_size,
                data: vec![0; (chunk_size * chunk_size) as usize],
            };
            chunks.push(new_chunk);
            chunks.last_mut().unwrap()
        };

        let local_x = tile_x.rem_euclid(chunk_size_i32) as usize;
        let local_y = tile_y.rem_euclid(chunk_size_i32) as usize;
        let index = local_y * chunk.width as usize + local_x;

        if index < chunk.data.len() {
            chunk.data[index] = gid;
        }
    }

    /// Add an object to an object layer
    pub fn add_object(&mut self, object: MapObject) {
        if let Some(objects) = &mut self.objects {
            objects.push(object);
        }
    }
}

// ============================================================================
// MAPOBJECT IMPLEMENTATION
// ============================================================================

impl MapObject {
    /// Create a new rectangle object
    pub fn new_rectangle(id: u32, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id,
            name: String::new(),
            obj_type: String::new(),
            x,
            y,
            width,
            height,
            rotation: 0.0,
            visible: true,
            gid: None,
            ellipse: false,
            point: false,
            polygon: None,
            polyline: None,
            text: None,
            properties: Vec::new(),
        }
    }

    /// Create a new point object
    pub fn new_point(id: u32, x: f32, y: f32) -> Self {
        Self {
            id,
            name: String::new(),
            obj_type: String::new(),
            x,
            y,
            width: 0.0,
            height: 0.0,
            rotation: 0.0,
            visible: true,
            gid: None,
            ellipse: false,
            point: true,
            polygon: None,
            polyline: None,
            text: None,
            properties: Vec::new(),
        }
    }

    /// Create a new ellipse object
    pub fn new_ellipse(id: u32, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id,
            name: String::new(),
            obj_type: String::new(),
            x,
            y,
            width,
            height,
            rotation: 0.0,
            visible: true,
            gid: None,
            ellipse: true,
            point: false,
            polygon: None,
            polyline: None,
            text: None,
            properties: Vec::new(),
        }
    }

    /// Create a new polygon object
    pub fn new_polygon(id: u32, x: f32, y: f32, points: Vec<PolyPoint>) -> Self {
        Self {
            id,
            name: String::new(),
            obj_type: String::new(),
            x,
            y,
            width: 0.0,
            height: 0.0,
            rotation: 0.0,
            visible: true,
            gid: None,
            ellipse: false,
            point: false,
            polygon: Some(points),
            polyline: None,
            text: None,
            properties: Vec::new(),
        }
    }

    /// Create a new tile object
    pub fn new_tile_object(id: u32, x: f32, y: f32, gid: u32) -> Self {
        Self {
            id,
            name: String::new(),
            obj_type: String::new(),
            x,
            y,
            width: 0.0,
            height: 0.0,
            rotation: 0.0,
            visible: true,
            gid: Some(gid),
            ellipse: false,
            point: false,
            polygon: None,
            polyline: None,
            text: None,
            properties: Vec::new(),
        }
    }
}

// ============================================================================
// LEGACY MIGRATION
// ============================================================================

impl TilemapMap {
    /// Convert a legacy ZoneTilemap to the new TilemapMap format
    pub fn from_legacy(old: &ZoneTilemap) -> Self {
        let mut map = TilemapMap::new(old.tile_size, old.tile_size);
        map.infinite = true;

        // Create layers for the old hardcoded layers
        let ground_layer_id = map.add_tile_layer("Ground");
        let decorations_layer_id = map.add_tile_layer("Decorations");
        let collision_layer_id = map.add_tile_layer("Collision");

        // Convert chunks from old format to new format
        for (key, old_chunk) in &old.chunks {
            if let Some((chunk_x, chunk_y)) = ZoneTilemap::parse_chunk_key(key) {
                let chunk_size = old.chunk_size;
                let base_tile_x = chunk_x * chunk_size as i32;
                let base_tile_y = chunk_y * chunk_size as i32;

                // Convert ground layer
                if let Some(ground_layer) = map.get_layer_mut(ground_layer_id) {
                    let mut ground_data = vec![0u32; (chunk_size * chunk_size) as usize];
                    for (y, row) in old_chunk.ground.iter().enumerate() {
                        for (x, &tile_id) in row.iter().enumerate() {
                            if tile_id != 0 {
                                let idx = y * chunk_size as usize + x;
                                // Offset by 1 since GID 0 is empty in Tiled
                                ground_data[idx] = tile_id + 1;
                            }
                        }
                    }
                    let chunk = LayerTileChunk {
                        x: base_tile_x,
                        y: base_tile_y,
                        width: chunk_size,
                        height: chunk_size,
                        data: ground_data,
                    };
                    if let Some(chunks) = &mut ground_layer.chunks {
                        chunks.push(chunk);
                    }
                }

                // Convert decorations layer
                if let Some(decorations_layer) = map.get_layer_mut(decorations_layer_id) {
                    let mut decor_data = vec![0u32; (chunk_size * chunk_size) as usize];
                    for (y, row) in old_chunk.decorations.iter().enumerate() {
                        for (x, &tile_id) in row.iter().enumerate() {
                            if tile_id != 0 {
                                let idx = y * chunk_size as usize + x;
                                // Offset for decorations tileset (assuming ground tileset has <1000 tiles)
                                decor_data[idx] = tile_id + 1000;
                            }
                        }
                    }
                    let chunk = LayerTileChunk {
                        x: base_tile_x,
                        y: base_tile_y,
                        width: chunk_size,
                        height: chunk_size,
                        data: decor_data,
                    };
                    if let Some(chunks) = &mut decorations_layer.chunks {
                        chunks.push(chunk);
                    }
                }

                // Convert collision layer (collision uses GID 1 for blocked)
                if let Some(collision_layer) = map.get_layer_mut(collision_layer_id) {
                    let mut collision_data = vec![0u32; (chunk_size * chunk_size) as usize];
                    for (y, row) in old_chunk.collision.iter().enumerate() {
                        for (x, &blocked) in row.iter().enumerate() {
                            if blocked != 0 {
                                let idx = y * chunk_size as usize + x;
                                // Use a special GID for collision (e.g., 2000)
                                collision_data[idx] = 2000;
                            }
                        }
                    }
                    let chunk = LayerTileChunk {
                        x: base_tile_x,
                        y: base_tile_y,
                        width: chunk_size,
                        height: chunk_size,
                        data: collision_data,
                    };
                    if let Some(chunks) = &mut collision_layer.chunks {
                        chunks.push(chunk);
                    }
                }
            }
        }

        // Set up tileset references (these would need to be properly configured)
        map.tilesets.push(TilesetRef {
            firstgid: 1,
            source: Some("ground.json".to_string()),
            embedded: None,
        });
        map.tilesets.push(TilesetRef {
            firstgid: 1000,
            source: Some("decorations.json".to_string()),
            embedded: None,
        });
        map.tilesets.push(TilesetRef {
            firstgid: 2000,
            source: Some("collision.json".to_string()),
            embedded: None,
        });

        map
    }
}
