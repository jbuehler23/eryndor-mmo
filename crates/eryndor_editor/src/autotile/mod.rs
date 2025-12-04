//! Tiled-Style Terrain System
//!
//! This module implements automatic tile selection based on Tiled's terrain approach,
//! where users manually mark tile corners/edges with terrain types, and the system
//! finds matching tiles based on neighbor constraints.
//!
//! ## Terrain Set Types
//!
//! - **Corner Sets**: Match at tile corners (4 corners per tile, 16 tiles for 2 terrains)
//! - **Edge Sets**: Match at tile edges (4 edges per tile, 16 tiles for 2 terrains)
//! - **Mixed Sets**: Both corners AND edges (256 tiles for 2 terrains)
//!
//! ## Position Layout
//!
//! ```text
//! Corner positions:        Edge positions:         Mixed positions:
//!   0───1                    ─0─                    0─1─2
//!   │   │                  3     1                  7   3
//!   2───3                    ─2─                    6─5─4
//! ```

use bevy::prelude::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Type of terrain set - determines how tiles are matched
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TerrainSetType {
    /// 4 corners per tile (TL, TR, BL, BR)
    /// Good for basic terrain transitions
    #[default]
    Corner,
    /// 4 edges per tile (Top, Right, Bottom, Left)
    /// Good for roads, platforms, paths
    Edge,
    /// 4 corners + 4 edges per tile
    /// Most flexible, requires more tiles
    Mixed,
}

impl TerrainSetType {
    /// Get the number of positions used by this terrain set type
    pub fn position_count(&self) -> usize {
        match self {
            TerrainSetType::Corner => 4,
            TerrainSetType::Edge => 4,
            TerrainSetType::Mixed => 8,
        }
    }

    /// Get the name for a position index
    pub fn position_name(&self, index: usize) -> &'static str {
        match self {
            TerrainSetType::Corner => {
                match index {
                    0 => "Top-Left",
                    1 => "Top-Right",
                    2 => "Bottom-Left",
                    3 => "Bottom-Right",
                    _ => "Unknown",
                }
            }
            TerrainSetType::Edge => {
                match index {
                    0 => "Top",
                    1 => "Right",
                    2 => "Bottom",
                    3 => "Left",
                    _ => "Unknown",
                }
            }
            TerrainSetType::Mixed => {
                match index {
                    0 => "Top-Left Corner",
                    1 => "Top Edge",
                    2 => "Top-Right Corner",
                    3 => "Right Edge",
                    4 => "Bottom-Right Corner",
                    5 => "Bottom Edge",
                    6 => "Bottom-Left Corner",
                    7 => "Left Edge",
                    _ => "Unknown",
                }
            }
        }
    }
}

/// A terrain type within a set (e.g., "Grass", "Dirt", "Water")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Terrain {
    pub id: Uuid,
    pub name: String,
    /// Display color for UI visualization
    #[serde(with = "color_serde")]
    pub color: Color,
    /// Representative tile for this terrain (shown in UI)
    pub icon_tile: Option<u32>,
}

impl Terrain {
    pub fn new(name: String, color: Color) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            color,
            icon_tile: None,
        }
    }
}

/// Custom serialization for Color since bevy's Color doesn't implement Serialize
mod color_serde {
    use bevy::prelude::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct ColorRgba {
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    }

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let srgba = color.to_srgba();
        let rgba = ColorRgba {
            r: srgba.red,
            g: srgba.green,
            b: srgba.blue,
            a: srgba.alpha,
        };
        rgba.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let rgba = ColorRgba::deserialize(deserializer)?;
        Ok(Color::srgba(rgba.r, rgba.g, rgba.b, rgba.a))
    }
}

/// Terrain assignments for a single tile's corners/edges
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TileTerrainData {
    /// Terrain index at each position (None = no terrain assigned)
    /// For Corner: indices 0-3 (TL, TR, BL, BR)
    /// For Edge: indices 0-3 (Top, Right, Bottom, Left)
    /// For Mixed: indices 0-7 (TL, T, TR, R, BR, B, BL, L - clockwise from top-left)
    pub terrains: [Option<usize>; 8],
}

impl TileTerrainData {
    pub fn new() -> Self {
        Self {
            terrains: [None; 8],
        }
    }

    /// Set terrain at a specific position
    pub fn set(&mut self, position: usize, terrain_index: Option<usize>) {
        if position < 8 {
            self.terrains[position] = terrain_index;
        }
    }

    /// Get terrain at a specific position
    pub fn get(&self, position: usize) -> Option<usize> {
        self.terrains.get(position).copied().flatten()
    }

    /// Check if this tile has any terrain assigned
    pub fn has_any_terrain(&self) -> bool {
        self.terrains.iter().any(|t| t.is_some())
    }

    /// Check if all positions have the same terrain (useful for fill tiles)
    pub fn is_uniform(&self, position_count: usize) -> Option<usize> {
        let first = self.terrains[0]?;
        for i in 1..position_count {
            if self.terrains[i] != Some(first) {
                return None;
            }
        }
        Some(first)
    }
}

/// A terrain set attached to a tileset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainSet {
    pub id: Uuid,
    pub name: String,
    /// Which tileset this terrain set belongs to
    pub tileset_id: Uuid,
    /// Type of terrain matching (Corner, Edge, or Mixed)
    pub set_type: TerrainSetType,
    /// List of terrains in this set (e.g., ["Grass", "Dirt", "Water"])
    pub terrains: Vec<Terrain>,
    /// Terrain assignments for each tile (tile_index -> TileTerrainData)
    pub tile_terrains: HashMap<u32, TileTerrainData>,
}

impl TerrainSet {
    pub fn new(name: String, tileset_id: Uuid, set_type: TerrainSetType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            tileset_id,
            set_type,
            terrains: Vec::new(),
            tile_terrains: HashMap::new(),
        }
    }

    /// Add a new terrain to this set
    pub fn add_terrain(&mut self, name: String, color: Color) -> usize {
        let terrain = Terrain::new(name, color);
        self.terrains.push(terrain);
        self.terrains.len() - 1
    }

    /// Remove a terrain by index
    pub fn remove_terrain(&mut self, index: usize) -> Option<Terrain> {
        if index < self.terrains.len() {
            // Update all tile terrain data to remove references to this terrain
            for tile_data in self.tile_terrains.values_mut() {
                for pos in tile_data.terrains.iter_mut() {
                    if let Some(terrain_idx) = pos {
                        if *terrain_idx == index {
                            *pos = None;
                        } else if *terrain_idx > index {
                            *terrain_idx -= 1;
                        }
                    }
                }
            }
            Some(self.terrains.remove(index))
        } else {
            None
        }
    }

    /// Get terrain index by name
    pub fn get_terrain_index(&self, name: &str) -> Option<usize> {
        self.terrains.iter().position(|t| t.name == name)
    }

    /// Set terrain for a tile position
    pub fn set_tile_terrain(&mut self, tile_index: u32, position: usize, terrain_index: Option<usize>) {
        let data = self.tile_terrains.entry(tile_index).or_default();
        data.set(position, terrain_index);
    }

    /// Get tile terrain data
    pub fn get_tile_terrain(&self, tile_index: u32) -> Option<&TileTerrainData> {
        self.tile_terrains.get(&tile_index)
    }

    /// Find a tile that matches the given constraints (Tiled-style penalty scoring)
    /// Returns the best matching tile, even if not perfect
    pub fn find_matching_tile(&self, constraints: &TileConstraints) -> Option<u32> {
        self.find_best_tile(constraints).map(|(tile, _score)| tile)
    }

    /// Find the best tile match using Tiled-style penalty scoring
    /// Returns (tile_index, penalty_score) where lower score = better match
    /// Returns None only if no tiles have terrain data
    pub fn find_best_tile(&self, constraints: &TileConstraints) -> Option<(u32, i32)> {
        let position_count = self.set_type.position_count();
        let mut best_tile: Option<(u32, i32)> = None;

        for (&tile_index, tile_data) in &self.tile_terrains {
            if !tile_data.has_any_terrain() {
                continue;
            }

            let mut penalty = 0i32;
            let mut impossible = false;

            for i in 0..position_count {
                let desired = constraints.desired[i];
                let actual = tile_data.terrains[i];
                let is_constrained = constraints.mask[i];

                match (desired, actual, is_constrained) {
                    // Constrained position: must match exactly
                    (Some(d), Some(a), true) if d != a => {
                        // Hard constraint violation - reject this tile
                        impossible = true;
                        break;
                    }
                    // Constrained position: matches
                    (Some(_), Some(_), true) => {
                        // Perfect match, no penalty
                    }
                    // Constrained but tile has no terrain here
                    (Some(_), None, true) => {
                        impossible = true;
                        break;
                    }
                    // Unconstrained position with preference: score by match
                    (Some(d), Some(a), false) if d != a => {
                        // Soft mismatch - add transition penalty
                        penalty += self.transition_penalty(d, a);
                    }
                    // Unconstrained, no preference or matches
                    _ => {
                        // No penalty
                    }
                }
            }

            if impossible {
                continue;
            }

            // Track the best (lowest penalty) tile
            match best_tile {
                None => best_tile = Some((tile_index, penalty)),
                Some((_, best_penalty)) if penalty < best_penalty => {
                    best_tile = Some((tile_index, penalty));
                }
                _ => {}
            }
        }

        best_tile
    }

    /// Calculate transition penalty between two terrain types
    /// Returns 0 for same terrain, positive for different terrains
    /// Used for soft constraints (unconstrained positions)
    fn transition_penalty(&self, from: usize, to: usize) -> i32 {
        if from == to {
            0
        } else {
            // Simple penalty: 1 per mismatch
            // Could be extended to use a terrain compatibility matrix
            1
        }
    }

    /// Find all tiles that have a specific terrain (useful for finding "fill" tiles)
    pub fn find_uniform_tiles(&self, terrain_index: usize) -> Vec<u32> {
        let position_count = self.set_type.position_count();

        self.tile_terrains
            .iter()
            .filter_map(|(&tile_index, tile_data)| {
                if tile_data.is_uniform(position_count) == Some(terrain_index) {
                    Some(tile_index)
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Constraints for finding a matching tile (Tiled-style with masks)
#[derive(Debug, Clone, Default)]
pub struct TileConstraints {
    /// Desired terrain at each position
    pub desired: [Option<usize>; 8],
    /// Mask indicating which positions are constrained (true = must match)
    pub mask: [bool; 8],
}

impl TileConstraints {
    pub fn new() -> Self {
        Self {
            desired: [None; 8],
            mask: [false; 8],
        }
    }

    /// Set a constrained terrain at a position
    pub fn set(&mut self, position: usize, terrain_index: usize) {
        if position < 8 {
            self.desired[position] = Some(terrain_index);
            self.mask[position] = true;
        }
    }

    /// Set desired terrain without constraining (soft preference)
    pub fn set_desired(&mut self, position: usize, terrain_index: usize) {
        if position < 8 {
            self.desired[position] = Some(terrain_index);
        }
    }

    /// Check if a position is constrained
    pub fn is_constrained(&self, position: usize) -> bool {
        position < 8 && self.mask[position]
    }

    // Legacy compatibility
    #[allow(dead_code)]
    pub fn required(&self) -> &[Option<usize>; 8] {
        &self.desired
    }
}

// ============================================================================
// Tiled-Style WangFiller Algorithm (New Implementation)
// ============================================================================

/// Wang ID representing terrain colors at all 8 positions
/// Uses Tiled's position indexing:
///   7|0|1
///   6|X|2
///   5|4|3
/// - Even indices (0,2,4,6) = Edges (Top, Right, Bottom, Left)
/// - Odd indices (1,3,5,7) = Corners (TopRight, BottomRight, BottomLeft, TopLeft)
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct WangId {
    /// 8 positions: Top=0, TopRight=1, Right=2, BottomRight=3, Bottom=4, BottomLeft=5, Left=6, TopLeft=7
    /// None = wildcard (any terrain matches)
    pub colors: [Option<usize>; 8],
}

impl WangId {
    pub const WILDCARD: Self = WangId { colors: [None; 8] };

    /// Create a WangId with all positions set to one terrain
    pub fn filled(terrain: usize) -> Self {
        WangId { colors: [Some(terrain); 8] }
    }

    /// Get opposite index (position on neighbor that faces us)
    pub fn opposite_index(i: usize) -> usize {
        (i + 4) % 8
    }

    /// Check if index is a corner (odd indices: 1,3,5,7)
    pub fn is_corner(i: usize) -> bool {
        i % 2 == 1
    }

    /// Get next index clockwise
    pub fn next_index(i: usize) -> usize {
        (i + 1) % 8
    }

    /// Get previous index counter-clockwise
    pub fn prev_index(i: usize) -> usize {
        (i + 7) % 8
    }
}

/// Information about constraints for a single cell
#[derive(Clone, Default, Debug)]
pub struct CellInfo {
    /// Desired terrain colors at each position
    pub desired: WangId,
    /// Which positions are hard-constrained (must match exactly)
    pub mask: [bool; 8],
}

/// Fills a region with Wang tiles based on constraints
/// This is a port of Tiled's WangFiller algorithm
pub struct WangFiller<'a> {
    terrain_set: &'a TerrainSet,
    /// Grid of cell constraints for the fill region
    cells: HashMap<(i32, i32), CellInfo>,
    /// Corrections queue for tiles that need re-evaluation
    corrections: Vec<(i32, i32)>,
    /// Whether corrections are enabled (starts false, enabled after initial pass)
    corrections_enabled: bool,
}

impl<'a> WangFiller<'a> {
    pub fn new(terrain_set: &'a TerrainSet) -> Self {
        Self {
            terrain_set,
            cells: HashMap::new(),
            corrections: Vec::new(),
            corrections_enabled: false,
        }
    }

    /// Get or create cell info at position
    pub fn get_cell_mut(&mut self, x: i32, y: i32) -> &mut CellInfo {
        self.cells.entry((x, y)).or_default()
    }

    /// Build constraints from the 8 surrounding tiles
    fn wang_id_from_surroundings(
        &self,
        tiles: &[Option<u32>],
        width: u32,
        height: u32,
        x: i32,
        y: i32,
    ) -> WangId {
        let mut result = WangId::WILDCARD;

        // Neighbor offsets in clockwise order matching WangId positions
        // Top, TopRight, Right, BottomRight, Bottom, BottomLeft, Left, TopLeft
        let offsets: [(i32, i32); 8] = [
            (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)
        ];

        let mut neighbor_wangids: [WangId; 8] = [WangId::WILDCARD; 8];

        // Get WangId of each neighbor
        for (i, (dx, dy)) in offsets.iter().enumerate() {
            let nx = x + dx;
            let ny = y + dy;

            if nx >= 0 && ny >= 0 && nx < width as i32 && ny < height as i32 {
                let nidx = (ny as u32 * width + nx as u32) as usize;
                if let Some(tile) = tiles.get(nidx).copied().flatten() {
                    if let Some(terrain_data) = self.terrain_set.get_tile_terrain(tile) {
                        // Convert from our internal format to WangId
                        neighbor_wangids[i] = self.tile_terrain_to_wang_id(terrain_data);
                    }
                }
            }
        }

        // Get edge colors from opposite sides of neighbors
        for i in [0, 2, 4, 6] { // Top, Right, Bottom, Left edges
            let opp = WangId::opposite_index(i);
            result.colors[i] = neighbor_wangids[i].colors[opp];
        }

        // Get corner colors with fallback logic (like Tiled)
        for i in [1, 3, 5, 7] { // TopRight, BottomRight, BottomLeft, TopLeft corners
            let opp = WangId::opposite_index(i);
            let mut color = neighbor_wangids[i].colors[opp];

            // Fallback 1: Get from left neighbor's corner
            if color.is_none() {
                let left_idx = WangId::prev_index(i);
                let left_corner = (i + 2) % 8;
                color = neighbor_wangids[left_idx].colors[left_corner];
            }

            // Fallback 2: Get from right neighbor's corner
            if color.is_none() {
                let right_idx = WangId::next_index(i);
                let right_corner = (i + 6) % 8;
                color = neighbor_wangids[right_idx].colors[right_corner];
            }

            result.colors[i] = color;
        }

        result
    }

    /// Convert TileTerrainData to WangId using Tiled's position mapping
    fn tile_terrain_to_wang_id(&self, data: &TileTerrainData) -> WangId {
        let mut wang_id = WangId::WILDCARD;

        match self.terrain_set.set_type {
            TerrainSetType::Corner => {
                // Our Corner: 0=TL, 1=TR, 2=BL, 3=BR
                // Tiled corners: 7=TopLeft, 1=TopRight, 5=BottomLeft, 3=BottomRight
                wang_id.colors[7] = data.get(0); // TL
                wang_id.colors[1] = data.get(1); // TR
                wang_id.colors[5] = data.get(2); // BL
                wang_id.colors[3] = data.get(3); // BR
            }
            TerrainSetType::Edge => {
                // Our Edge: 0=Top, 1=Right, 2=Bottom, 3=Left
                // Tiled edges: 0=Top, 2=Right, 4=Bottom, 6=Left
                wang_id.colors[0] = data.get(0); // Top
                wang_id.colors[2] = data.get(1); // Right
                wang_id.colors[4] = data.get(2); // Bottom
                wang_id.colors[6] = data.get(3); // Left
            }
            TerrainSetType::Mixed => {
                // Our Mixed: 0=TL, 1=Top, 2=TR, 3=Right, 4=BR, 5=Bottom, 6=BL, 7=Left
                // Tiled: 0=Top, 1=TR, 2=Right, 3=BR, 4=Bottom, 5=BL, 6=Left, 7=TL
                wang_id.colors[7] = data.get(0); // TL corner
                wang_id.colors[0] = data.get(1); // Top edge
                wang_id.colors[1] = data.get(2); // TR corner
                wang_id.colors[2] = data.get(3); // Right edge
                wang_id.colors[3] = data.get(4); // BR corner
                wang_id.colors[4] = data.get(5); // Bottom edge
                wang_id.colors[5] = data.get(6); // BL corner
                wang_id.colors[6] = data.get(7); // Left edge
            }
        }

        wang_id
    }

    /// Find the best tile matching constraints using penalty scoring
    fn find_best_match(&self, info: &CellInfo) -> Option<u32> {
        let mut best_tile = None;
        let mut lowest_penalty = i32::MAX;
        let mut tiles_checked = 0;
        let mut tiles_rejected_hard = 0;

        for (&tile_id, tile_terrain) in &self.terrain_set.tile_terrains {
            if !tile_terrain.has_any_terrain() {
                continue;
            }
            tiles_checked += 1;

            let tile_wang_id = self.tile_terrain_to_wang_id(tile_terrain);

            // Check hard constraints first
            let mut matches_hard = true;
            let mut failed_pos = None;
            for i in 0..8 {
                if info.mask[i] {
                    let desired = info.desired.colors[i];
                    let actual = tile_wang_id.colors[i];
                    if desired.is_some() && desired != actual {
                        matches_hard = false;
                        failed_pos = Some((i, desired, actual));
                        break;
                    }
                }
            }

            if !matches_hard {
                tiles_rejected_hard += 1;
                if tiles_rejected_hard <= 3 {
                    bevy::log::debug!(
                        "find_best_match: tile {} failed hard constraint at pos {:?}",
                        tile_id, failed_pos
                    );
                }
                continue;
            }

            // Calculate penalty for soft preferences
            let mut penalty = 0i32;
            for i in 0..8 {
                if !info.mask[i] {
                    if let Some(desired) = info.desired.colors[i] {
                        let actual = tile_wang_id.colors[i];
                        if Some(desired) != actual {
                            penalty += 1;
                        }
                    }
                }
            }

            if penalty < lowest_penalty {
                lowest_penalty = penalty;
                best_tile = Some(tile_id);
            }
        }

        bevy::log::info!(
            "find_best_match: checked {} tiles, {} rejected hard, best={:?} penalty={}",
            tiles_checked, tiles_rejected_hard, best_tile, lowest_penalty
        );

        best_tile
    }

    /// Update adjacent cell's constraints based on a placed tile
    fn update_adjacent(&mut self, wang_id: &WangId, adj_x: i32, adj_y: i32, direction_index: usize) {
        let cell = self.get_cell_mut(adj_x, adj_y);
        let opp = WangId::opposite_index(direction_index);

        // Set the opposite position on the neighbor
        cell.desired.colors[opp] = wang_id.colors[direction_index];
        cell.mask[opp] = true;

        // If this is an EDGE (not corner), also update adjacent corners
        if !WangId::is_corner(opp) {
            let corner_a = WangId::next_index(opp);
            let corner_b = WangId::prev_index(opp);

            let adj_corner_a = WangId::prev_index(direction_index);
            let adj_corner_b = WangId::next_index(direction_index);

            cell.desired.colors[corner_a] = wang_id.colors[adj_corner_a];
            cell.mask[corner_a] = true;

            cell.desired.colors[corner_b] = wang_id.colors[adj_corner_b];
            cell.mask[corner_b] = true;
        }
    }

    /// Apply the filler to a tile layer
    pub fn apply(
        &mut self,
        tiles: &mut [Option<u32>],
        width: u32,
        height: u32,
        region: &[(i32, i32)], // Positions to fill
    ) {
        // Neighbor offsets matching WangId positions
        let offsets: [(i32, i32); 8] = [
            (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)
        ];

        // Phase 1: Set border constraints from outside tiles AND preserve current tile's terrain
        for &(x, y) in region {
            // Track which positions we preserve from the current tile (soft preferences)
            let mut preserved_positions = [false; 8];

            // First, get the current tile's terrain (if any) to preserve non-painted corners
            let idx = (y as u32 * width + x as u32) as usize;
            if let Some(current_tile) = tiles.get(idx).copied().flatten() {
                if let Some(current_terrain) = self.terrain_set.get_tile_terrain(current_tile) {
                    let current_wang_id = self.tile_terrain_to_wang_id(current_terrain);
                    let cell = self.get_cell_mut(x, y);

                    bevy::log::debug!(
                        "Phase1: tile ({},{}) current={} wang={:?} mask={:?}",
                        x, y, current_tile, current_wang_id.colors, cell.mask
                    );

                    // Preserve current tile's terrain for positions not explicitly set
                    // Keep as SOFT preferences (don't set mask) but track them
                    let mut preserved = Vec::new();
                    for i in 0..8 {
                        if !cell.mask[i] && current_wang_id.colors[i].is_some() {
                            cell.desired.colors[i] = current_wang_id.colors[i];
                            preserved_positions[i] = true; // Track for surroundings merge
                            preserved.push((i, current_wang_id.colors[i]));
                        }
                    }
                    if !preserved.is_empty() {
                        bevy::log::debug!(
                            "Phase1: tile ({},{}) preserved corners {:?} as soft preferences",
                            x, y, preserved
                        );
                    }
                }
            }

            // Then get constraints from neighboring tiles
            let surroundings = self.wang_id_from_surroundings(tiles, width, height, x, y);
            let cell = self.get_cell_mut(x, y);

            // Merge surroundings but don't override hard constraints OR preserved values
            for i in 0..8 {
                if !cell.mask[i] && !preserved_positions[i] && surroundings.colors[i].is_some() {
                    cell.desired.colors[i] = surroundings.colors[i];
                }
            }
        }

        // Phase 2: Resolve tiles
        for &(x, y) in region {
            let cell = self.cells.get(&(x, y)).cloned().unwrap_or_default();

            if let Some(new_tile) = self.find_best_match(&cell) {
                let idx = (y as u32 * width + x as u32) as usize;
                tiles[idx] = Some(new_tile);

                // Get WangId of placed tile
                let placed_wang_id = self.terrain_set
                    .get_tile_terrain(new_tile)
                    .map(|t| self.tile_terrain_to_wang_id(t))
                    .unwrap_or(WangId::WILDCARD);

                // Update neighbors
                for (i, (dx, dy)) in offsets.iter().enumerate() {
                    let nx = x + dx;
                    let ny = y + dy;

                    if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                        continue;
                    }

                    // KEY FIX: Only update neighbors that ALREADY have terrain tiles
                    // This prevents painting tiles on empty cells
                    let nidx = (ny as u32 * width + nx as u32) as usize;
                    if tiles.get(nidx).copied().flatten().is_none() {
                        continue;
                    }

                    self.update_adjacent(&placed_wang_id, nx, ny, i);

                    // Queue for corrections if needed (only edge neighbors, not corners)
                    if self.corrections_enabled && !WangId::is_corner(i) {
                        let region_contains = region.contains(&(nx, ny));
                        if !region_contains {
                            self.corrections.push((nx, ny));
                        }
                    }
                }
            }
        }

        // Phase 3: Apply corrections
        self.corrections_enabled = true;
        while !self.corrections.is_empty() {
            let to_process: Vec<_> = self.corrections.drain(..).collect();
            for (x, y) in to_process {
                let cell = self.cells.get(&(x, y)).cloned().unwrap_or_default();

                if let Some(new_tile) = self.find_best_match(&cell) {
                    let idx = (y as u32 * width + x as u32) as usize;
                    let old_tile = tiles[idx];

                    if old_tile != Some(new_tile) {
                        tiles[idx] = Some(new_tile);
                    }
                }
            }
        }
    }
}

/// Configuration for autotiling in a project
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutotileConfig {
    /// All terrain sets defined in the project
    pub terrain_sets: Vec<TerrainSet>,
    /// Legacy terrain types (for backward compatibility - will be migrated)
    #[serde(default)]
    pub terrains: Vec<LegacyTerrainType>,
}

impl AutotileConfig {
    pub fn new() -> Self {
        Self {
            terrain_sets: Vec::new(),
            terrains: Vec::new(),
        }
    }

    /// Add a terrain set
    pub fn add_terrain_set(&mut self, terrain_set: TerrainSet) {
        self.terrain_sets.push(terrain_set);
    }

    /// Get terrain set by ID
    pub fn get_terrain_set(&self, id: Uuid) -> Option<&TerrainSet> {
        self.terrain_sets.iter().find(|ts| ts.id == id)
    }

    /// Get mutable terrain set by ID
    pub fn get_terrain_set_mut(&mut self, id: Uuid) -> Option<&mut TerrainSet> {
        self.terrain_sets.iter_mut().find(|ts| ts.id == id)
    }

    /// Remove terrain set by ID
    pub fn remove_terrain_set(&mut self, id: Uuid) -> Option<TerrainSet> {
        if let Some(pos) = self.terrain_sets.iter().position(|ts| ts.id == id) {
            Some(self.terrain_sets.remove(pos))
        } else {
            None
        }
    }

    /// Get all terrain sets for a specific tileset
    pub fn get_terrain_sets_for_tileset(&self, tileset_id: Uuid) -> Vec<&TerrainSet> {
        self.terrain_sets
            .iter()
            .filter(|ts| ts.tileset_id == tileset_id)
            .collect()
    }

    // Legacy compatibility methods

    /// Add a legacy terrain type (for backward compatibility)
    pub fn add_terrain(&mut self, terrain: LegacyTerrainType) {
        self.terrains.push(terrain);
    }

    /// Get legacy terrain by ID
    pub fn get_terrain(&self, id: Uuid) -> Option<&LegacyTerrainType> {
        self.terrains.iter().find(|t| t.id == id)
    }

    /// Remove legacy terrain by ID
    pub fn remove_terrain(&mut self, id: Uuid) -> Option<LegacyTerrainType> {
        if let Some(pos) = self.terrains.iter().position(|t| t.id == id) {
            Some(self.terrains.remove(pos))
        } else {
            None
        }
    }
}

/// Legacy terrain type for backward compatibility with old 47-tile blob format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyTerrainType {
    pub id: Uuid,
    pub name: String,
    pub base_tile: u32,
    pub tileset_id: Uuid,
    #[serde(default)]
    pub tile_mapping: HashMap<u8, u32>,
}

// Keep the old TerrainType as an alias for migration
pub type TerrainType = LegacyTerrainType;

impl LegacyTerrainType {
    /// Create a new legacy terrain type with standard 47-tile blob mapping
    pub fn new(name: String, tileset_id: Uuid, first_tile_index: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            base_tile: first_tile_index + 46,
            tileset_id,
            tile_mapping: Self::create_47_tile_mapping(first_tile_index),
        }
    }

    /// Create the standard 47-tile blob mapping (for backward compatibility)
    fn create_47_tile_mapping(first_tile: u32) -> HashMap<u8, u32> {
        // Simplified 47-tile mapping - keeping for legacy support
        let mut mapping = HashMap::new();
        for i in 0..47 {
            mapping.insert(i as u8, first_tile + i);
        }
        mapping
    }

    /// Get the tile index for a given neighbor bitmask
    pub fn get_tile(&self, bitmask: u8) -> u32 {
        self.tile_mapping.get(&bitmask).copied().unwrap_or(self.base_tile)
    }
}

// ============================================================================
// Legacy Autotile Functions (for backward compatibility with 47-tile blob)
// ============================================================================

/// Legacy neighbor direction flags for bitmask calculation
pub mod neighbors {
    pub const N: u8  = 0b0000_0001;  // North
    pub const NE: u8 = 0b0000_0010;  // Northeast (corner)
    pub const E: u8  = 0b0000_0100;  // East
    pub const SE: u8 = 0b0000_1000;  // Southeast (corner)
    pub const S: u8  = 0b0001_0000;  // South
    pub const SW: u8 = 0b0010_0000;  // Southwest (corner)
    pub const W: u8  = 0b0100_0000;  // West
    pub const NW: u8 = 0b1000_0000;  // Northwest (corner)
}

/// Apply corner optimization to a bitmask (legacy)
pub fn optimize_bitmask(bitmask: u8) -> u8 {
    use neighbors::*;

    let mut result = bitmask;

    // NW corner requires N and W
    if (bitmask & (N | W)) != (N | W) {
        result &= !NW;
    }
    // NE corner requires N and E
    if (bitmask & (N | E)) != (N | E) {
        result &= !NE;
    }
    // SE corner requires S and E
    if (bitmask & (S | E)) != (S | E) {
        result &= !SE;
    }
    // SW corner requires S and W
    if (bitmask & (S | W)) != (S | W) {
        result &= !SW;
    }

    result
}

/// Calculate the neighbor bitmask for a tile (legacy)
pub fn calculate_bitmask<F>(x: i32, y: i32, is_same_terrain: F) -> u8
where
    F: Fn(i32, i32) -> bool
{
    use neighbors::*;

    let mut bitmask = 0u8;

    if is_same_terrain(x, y - 1) { bitmask |= N; }
    if is_same_terrain(x + 1, y - 1) { bitmask |= NE; }
    if is_same_terrain(x + 1, y) { bitmask |= E; }
    if is_same_terrain(x + 1, y + 1) { bitmask |= SE; }
    if is_same_terrain(x, y + 1) { bitmask |= S; }
    if is_same_terrain(x - 1, y + 1) { bitmask |= SW; }
    if is_same_terrain(x - 1, y) { bitmask |= W; }
    if is_same_terrain(x - 1, y - 1) { bitmask |= NW; }

    optimize_bitmask(bitmask)
}

/// Apply autotiling to a region of tiles (legacy)
pub fn apply_autotile_to_region<F>(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    region_x: i32,
    region_y: i32,
    region_w: i32,
    region_h: i32,
    terrain: &LegacyTerrainType,
    is_terrain_tile: F,
) where
    F: Fn(Option<u32>) -> bool
{
    let min_x = (region_x - 1).max(0) as u32;
    let min_y = (region_y - 1).max(0) as u32;
    let max_x = ((region_x + region_w + 1) as u32).min(width);
    let max_y = ((region_y + region_h + 1) as u32).min(height);

    let mut updates: Vec<(usize, u32)> = Vec::new();

    for y in min_y..max_y {
        for x in min_x..max_x {
            let idx = (y * width + x) as usize;
            if let Some(Some(_)) = tiles.get(idx) {
                if is_terrain_tile(tiles[idx]) {
                    let bitmask = calculate_bitmask(x as i32, y as i32, |nx, ny| {
                        if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                            return false;
                        }
                        let nidx = (ny as u32 * width + nx as u32) as usize;
                        is_terrain_tile(tiles.get(nidx).copied().flatten())
                    });
                    updates.push((idx, terrain.get_tile(bitmask)));
                }
            }
        }
    }

    for (idx, new_tile) in updates {
        tiles[idx] = Some(new_tile);
    }
}

/// Paint a single tile with autotiling and update neighbors (legacy)
pub fn paint_autotile<F>(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    terrain: &LegacyTerrainType,
    is_terrain_tile: F,
) where
    F: Fn(Option<u32>) -> bool + Copy
{
    let idx = (y * width + x) as usize;
    if idx < tiles.len() {
        tiles[idx] = Some(terrain.base_tile);
    }

    apply_autotile_to_region(
        tiles,
        width,
        height,
        x as i32,
        y as i32,
        1,
        1,
        terrain,
        is_terrain_tile,
    );
}

/// Erase a tile and update autotiling for neighbors (legacy)
pub fn erase_autotile<F>(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    terrain: &LegacyTerrainType,
    is_terrain_tile: F,
) where
    F: Fn(Option<u32>) -> bool + Copy
{
    let idx = (y * width + x) as usize;
    if idx < tiles.len() {
        tiles[idx] = None;
    }

    apply_autotile_to_region(
        tiles,
        width,
        height,
        x as i32,
        y as i32,
        1,
        1,
        terrain,
        is_terrain_tile,
    );
}

/// Terrain brush state for painting with automatic tile selection
#[derive(Debug, Clone, Default)]
pub struct TerrainBrush {
    /// Currently selected terrain set ID
    pub selected_terrain_set: Option<Uuid>,
    /// Currently selected terrain index within the set
    pub selected_terrain_index: Option<usize>,
    /// Whether terrain painting mode is active
    pub active: bool,
}

impl TerrainBrush {
    pub fn new() -> Self {
        Self {
            selected_terrain_set: None,
            selected_terrain_index: None,
            active: false,
        }
    }

    pub fn select(&mut self, terrain_set_id: Uuid, terrain_index: usize) {
        self.selected_terrain_set = Some(terrain_set_id);
        self.selected_terrain_index = Some(terrain_index);
        self.active = true;
    }

    pub fn deselect(&mut self) {
        self.selected_terrain_set = None;
        self.selected_terrain_index = None;
        self.active = false;
    }
}

/// Build constraints for a tile based on its neighbors' terrain data
pub fn build_constraints_from_neighbors(
    terrain_set: &TerrainSet,
    terrain_index: usize,
    get_neighbor_tile: impl Fn(i32, i32) -> Option<u32>,
) -> TileConstraints {
    let mut constraints = TileConstraints::new();

    match terrain_set.set_type {
        TerrainSetType::Corner => {
            // For corner sets, we need to match corners with adjacent tiles
            // Position 0 (TL) is shared with tile above-left, above, and left
            // Position 1 (TR) is shared with tile above, above-right, and right
            // Position 2 (BL) is shared with tile left, below-left, and below
            // Position 3 (BR) is shared with tile right, below-right, and below

            // By default, set all corners to the selected terrain
            for i in 0..4 {
                constraints.set(i, terrain_index);
            }

            // Check neighbors and adjust constraints
            // Top-left corner (position 0) - check neighbor above (their BR) and left (their TR)
            if let Some(above_tile) = get_neighbor_tile(0, -1) {
                if let Some(above_data) = terrain_set.get_tile_terrain(above_tile) {
                    if let Some(t) = above_data.get(2) { // BR of tile above
                        constraints.set(0, t);
                    }
                    if let Some(t) = above_data.get(3) { // BR of tile above
                        constraints.set(1, t);
                    }
                }
            }

            if let Some(left_tile) = get_neighbor_tile(-1, 0) {
                if let Some(left_data) = terrain_set.get_tile_terrain(left_tile) {
                    if let Some(t) = left_data.get(1) { // TR of tile left
                        constraints.set(0, t);
                    }
                    if let Some(t) = left_data.get(3) { // BR of tile left
                        constraints.set(2, t);
                    }
                }
            }

            if let Some(right_tile) = get_neighbor_tile(1, 0) {
                if let Some(right_data) = terrain_set.get_tile_terrain(right_tile) {
                    if let Some(t) = right_data.get(0) { // TL of tile right
                        constraints.set(1, t);
                    }
                    if let Some(t) = right_data.get(2) { // BL of tile right
                        constraints.set(3, t);
                    }
                }
            }

            if let Some(below_tile) = get_neighbor_tile(0, 1) {
                if let Some(below_data) = terrain_set.get_tile_terrain(below_tile) {
                    if let Some(t) = below_data.get(0) { // TL of tile below
                        constraints.set(2, t);
                    }
                    if let Some(t) = below_data.get(1) { // TR of tile below
                        constraints.set(3, t);
                    }
                }
            }
        }

        TerrainSetType::Edge => {
            // For edge sets, we match edges with adjacent tiles
            // Position 0 (Top) matches with tile above's Bottom
            // Position 1 (Right) matches with tile right's Left
            // Position 2 (Bottom) matches with tile below's Top
            // Position 3 (Left) matches with tile left's Right

            for i in 0..4 {
                constraints.set(i, terrain_index);
            }

            if let Some(above_tile) = get_neighbor_tile(0, -1) {
                if let Some(above_data) = terrain_set.get_tile_terrain(above_tile) {
                    if let Some(t) = above_data.get(2) { // Bottom of tile above
                        constraints.set(0, t);
                    }
                }
            }

            if let Some(right_tile) = get_neighbor_tile(1, 0) {
                if let Some(right_data) = terrain_set.get_tile_terrain(right_tile) {
                    if let Some(t) = right_data.get(3) { // Left of tile right
                        constraints.set(1, t);
                    }
                }
            }

            if let Some(below_tile) = get_neighbor_tile(0, 1) {
                if let Some(below_data) = terrain_set.get_tile_terrain(below_tile) {
                    if let Some(t) = below_data.get(0) { // Top of tile below
                        constraints.set(2, t);
                    }
                }
            }

            if let Some(left_tile) = get_neighbor_tile(-1, 0) {
                if let Some(left_data) = terrain_set.get_tile_terrain(left_tile) {
                    if let Some(t) = left_data.get(1) { // Right of tile left
                        constraints.set(3, t);
                    }
                }
            }
        }

        TerrainSetType::Mixed => {
            // For mixed sets, corners and edges are separate
            // Positions 0,2,4,6 are corners, 1,3,5,7 are edges
            for i in 0..8 {
                constraints.set(i, terrain_index);
            }

            // Similar logic but with 8 positions
            // This is more complex - corners share with diagonal + orthogonal neighbors
            // Edges share with orthogonal neighbors only

            if let Some(above_tile) = get_neighbor_tile(0, -1) {
                if let Some(above_data) = terrain_set.get_tile_terrain(above_tile) {
                    // Top edge (1) matches above's bottom edge (5)
                    if let Some(t) = above_data.get(5) {
                        constraints.set(1, t);
                    }
                    // Corners 0 and 2 match above's 6 and 4
                    if let Some(t) = above_data.get(6) {
                        constraints.set(0, t);
                    }
                    if let Some(t) = above_data.get(4) {
                        constraints.set(2, t);
                    }
                }
            }

            if let Some(right_tile) = get_neighbor_tile(1, 0) {
                if let Some(right_data) = terrain_set.get_tile_terrain(right_tile) {
                    // Right edge (3) matches right's left edge (7)
                    if let Some(t) = right_data.get(7) {
                        constraints.set(3, t);
                    }
                    // Corners 2 and 4 match right's 0 and 6
                    if let Some(t) = right_data.get(0) {
                        constraints.set(2, t);
                    }
                    if let Some(t) = right_data.get(6) {
                        constraints.set(4, t);
                    }
                }
            }

            if let Some(below_tile) = get_neighbor_tile(0, 1) {
                if let Some(below_data) = terrain_set.get_tile_terrain(below_tile) {
                    // Bottom edge (5) matches below's top edge (1)
                    if let Some(t) = below_data.get(1) {
                        constraints.set(5, t);
                    }
                    // Corners 4 and 6 match below's 2 and 0
                    if let Some(t) = below_data.get(2) {
                        constraints.set(4, t);
                    }
                    if let Some(t) = below_data.get(0) {
                        constraints.set(6, t);
                    }
                }
            }

            if let Some(left_tile) = get_neighbor_tile(-1, 0) {
                if let Some(left_data) = terrain_set.get_tile_terrain(left_tile) {
                    // Left edge (7) matches left's right edge (3)
                    if let Some(t) = left_data.get(3) {
                        constraints.set(7, t);
                    }
                    // Corners 0 and 6 match left's 2 and 4
                    if let Some(t) = left_data.get(2) {
                        constraints.set(0, t);
                    }
                    if let Some(t) = left_data.get(4) {
                        constraints.set(6, t);
                    }
                }
            }
        }
    }

    constraints
}

/// Get the positions of a tile that face a neighbor in the given direction
/// Returns positions that should match between the current tile and its neighbor
fn get_facing_positions(set_type: TerrainSetType, dx: i32, dy: i32) -> Vec<usize> {
    match set_type {
        TerrainSetType::Corner => {
            // Corner positions: 0=TL, 1=TR, 2=BL, 3=BR
            match (dx, dy) {
                (0, -1) => vec![0, 1],    // Neighbor above: our top corners
                (1, 0) => vec![1, 3],     // Neighbor right: our right corners
                (0, 1) => vec![2, 3],     // Neighbor below: our bottom corners
                (-1, 0) => vec![0, 2],    // Neighbor left: our left corners
                (1, -1) => vec![1],       // Diagonal: top-right
                (1, 1) => vec![3],        // Diagonal: bottom-right
                (-1, 1) => vec![2],       // Diagonal: bottom-left
                (-1, -1) => vec![0],      // Diagonal: top-left
                _ => vec![],
            }
        }
        TerrainSetType::Edge => {
            // Edge positions: 0=Top, 1=Right, 2=Bottom, 3=Left
            match (dx, dy) {
                (0, -1) => vec![0],   // Neighbor above: our top edge
                (1, 0) => vec![1],    // Neighbor right: our right edge
                (0, 1) => vec![2],    // Neighbor below: our bottom edge
                (-1, 0) => vec![3],   // Neighbor left: our left edge
                _ => vec![],          // Diagonals don't share edges
            }
        }
        TerrainSetType::Mixed => {
            // Mixed positions: 0=TL, 1=Top, 2=TR, 3=Right, 4=BR, 5=Bottom, 6=BL, 7=Left
            match (dx, dy) {
                (0, -1) => vec![0, 1, 2],     // Neighbor above: top corners + top edge
                (1, 0) => vec![2, 3, 4],      // Neighbor right: right corners + right edge
                (0, 1) => vec![4, 5, 6],      // Neighbor below: bottom corners + bottom edge
                (-1, 0) => vec![6, 7, 0],     // Neighbor left: left corners + left edge
                (1, -1) => vec![2],           // Diagonal top-right: TR corner only
                (1, 1) => vec![4],            // Diagonal bottom-right: BR corner only
                (-1, 1) => vec![6],           // Diagonal bottom-left: BL corner only
                (-1, -1) => vec![0],          // Diagonal top-left: TL corner only
                _ => vec![],
            }
        }
    }
}

/// Get the opposite positions on a neighbor tile that face us
fn get_neighbor_facing_positions(set_type: TerrainSetType, dx: i32, dy: i32) -> Vec<usize> {
    // The positions on the neighbor that face our tile (opposite direction)
    get_facing_positions(set_type, -dx, -dy)
}

/// Paint terrain at a corner intersection using Tiled-style approach
///
/// In Tiled, terrain painting works at CORNER INTERSECTIONS, not tile centers.
/// Clicking at a point sets the terrain at that corner, affecting up to 4 tiles
/// that share that corner. Each tile gets ONE corner updated.
///
/// For Corner terrain sets:
/// - Corner (x, y) is shared by tiles at (x-1,y-1), (x,y-1), (x-1,y), (x,y)
/// - Each tile gets its adjacent corner position set to the painted terrain
///
/// This creates proper transition tiles instead of solid filled tiles.
pub fn paint_terrain(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    terrain_set: &TerrainSet,
    terrain_index: usize,
) {
    bevy::log::info!(
        "paint_terrain: corner=({},{}) terrain_idx={} set_type={:?} tiles_in_set={}",
        x, y, terrain_index, terrain_set.set_type, terrain_set.tile_terrains.len()
    );

    // The click position (x, y) represents a corner intersection
    // This corner is shared by up to 4 tiles:
    // - Tile (x-1, y-1): corner is at its BottomRight (position 3)
    // - Tile (x, y-1): corner is at its BottomLeft (position 5)
    // - Tile (x-1, y): corner is at its TopRight (position 1)
    // - Tile (x, y): corner is at its TopLeft (position 7)

    let cx = x as i32;
    let cy = y as i32;

    // Define affected tiles and which corner position to set on each
    // WangId positions: 0=Top, 1=TR, 2=Right, 3=BR, 4=Bottom, 5=BL, 6=Left, 7=TL
    let affected_tiles: [(i32, i32, usize); 4] = [
        (cx - 1, cy - 1, 3),  // Tile above-left: set BR corner
        (cx,     cy - 1, 5),  // Tile above-right: set BL corner
        (cx - 1, cy,     1),  // Tile below-left: set TR corner
        (cx,     cy,     7),  // Tile below-right: set TL corner
    ];

    let mut filler = WangFiller::new(terrain_set);
    let mut region = Vec::new();

    // Set constraints for each affected tile
    for &(tx, ty, corner_pos) in &affected_tiles {
        if tx >= 0 && ty >= 0 && tx < width as i32 && ty < height as i32 {
            let cell = filler.get_cell_mut(tx, ty);
            cell.desired.colors[corner_pos] = Some(terrain_index);
            cell.mask[corner_pos] = true;
            region.push((tx, ty));

            bevy::log::debug!(
                "paint_terrain: tile ({},{}) corner {} set to terrain {}",
                tx, ty, corner_pos, terrain_index
            );
        }
    }

    bevy::log::info!(
        "paint_terrain: affecting {} tiles at corner ({},{})",
        region.len(), x, y
    );

    // Apply to all affected tiles
    filler.apply(tiles, width, height, &region);

    bevy::log::info!(
        "paint_terrain: completed corner paint at ({},{})",
        x, y
    );
}

/// Represents what the terrain brush is painting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaintTarget {
    /// Paint at a corner intersection (affects 4 tiles)
    /// Coordinates are corner indices (between tiles)
    Corner { corner_x: u32, corner_y: u32 },
    /// Paint at an edge (horizontal edge between rows of tiles)
    /// tile_x is the tile column, edge_y is the edge row (between tile rows)
    HorizontalEdge { tile_x: u32, edge_y: u32 },
    /// Paint at an edge (vertical edge between columns of tiles)
    /// edge_x is the edge column (between tile columns), tile_y is the tile row
    VerticalEdge { edge_x: u32, tile_y: u32 },
}

/// Determine the paint target based on mouse position within a tile
/// For Mixed terrain sets, we divide the tile into a 3x3 grid:
/// - Corners: the four corner regions
/// - Edges: the four edge regions (between corners)
/// - Center: the middle (ignored, or could paint all corners/edges)
///
/// Returns the appropriate paint target for the mouse position.
///
/// Parameters:
/// - world_pos: Mouse position in world coordinates
/// - tile_size: Size of each tile in world units
/// - set_type: The type of terrain set being painted
pub fn get_paint_target(
    world_x: f32,
    world_y: f32,
    tile_size: f32,
    set_type: TerrainSetType,
) -> PaintTarget {
    // Calculate tile coordinates
    let tile_x = (world_x / tile_size).floor() as i32;
    let tile_y = ((-world_y) / tile_size).floor() as i32;

    // Calculate position within the tile (0.0 to 1.0)
    let local_x = (world_x / tile_size).fract();
    let local_y = ((-world_y) / tile_size).fract();

    // Handle negative fractional parts
    let local_x = if local_x < 0.0 { local_x + 1.0 } else { local_x };
    let local_y = if local_y < 0.0 { local_y + 1.0 } else { local_y };

    // For Corner-only sets, always paint corners
    if set_type == TerrainSetType::Corner {
        // Determine which corner based on position within tile
        let corner_x = if local_x < 0.5 { tile_x } else { tile_x + 1 };
        let corner_y = if local_y < 0.5 { tile_y } else { tile_y + 1 };
        return PaintTarget::Corner {
            corner_x: corner_x.max(0) as u32,
            corner_y: corner_y.max(0) as u32,
        };
    }

    // For Edge-only sets, always paint edges
    if set_type == TerrainSetType::Edge {
        // Determine if closer to horizontal or vertical edge
        let dist_to_horizontal = (local_y - 0.5).abs();
        let dist_to_vertical = (local_x - 0.5).abs();

        if dist_to_horizontal < dist_to_vertical {
            // Closer to horizontal (top/bottom)
            let edge_y = if local_y < 0.5 { tile_y } else { tile_y + 1 };
            return PaintTarget::HorizontalEdge {
                tile_x: tile_x.max(0) as u32,
                edge_y: edge_y.max(0) as u32,
            };
        } else {
            // Closer to vertical (left/right)
            let edge_x = if local_x < 0.5 { tile_x } else { tile_x + 1 };
            return PaintTarget::VerticalEdge {
                edge_x: edge_x.max(0) as u32,
                tile_y: tile_y.max(0) as u32,
            };
        }
    }

    // For Mixed sets, divide tile into 3x3 grid
    // Corners: (0,0), (2,0), (0,2), (2,2)
    // Edges: (1,0) top, (2,1) right, (1,2) bottom, (0,1) left
    // Center: (1,1) - treat as corner for now

    // Determine 3x3 zone (0, 1, or 2 in each axis)
    let zone_x = if local_x < 0.33 { 0 } else if local_x < 0.67 { 1 } else { 2 };
    let zone_y = if local_y < 0.33 { 0 } else if local_y < 0.67 { 1 } else { 2 };

    match (zone_x, zone_y) {
        // Corners
        (0, 0) => PaintTarget::Corner {
            corner_x: tile_x.max(0) as u32,
            corner_y: tile_y.max(0) as u32,
        },
        (2, 0) => PaintTarget::Corner {
            corner_x: (tile_x + 1).max(0) as u32,
            corner_y: tile_y.max(0) as u32,
        },
        (0, 2) => PaintTarget::Corner {
            corner_x: tile_x.max(0) as u32,
            corner_y: (tile_y + 1).max(0) as u32,
        },
        (2, 2) => PaintTarget::Corner {
            corner_x: (tile_x + 1).max(0) as u32,
            corner_y: (tile_y + 1).max(0) as u32,
        },
        // Edges
        (1, 0) => PaintTarget::HorizontalEdge {
            tile_x: tile_x.max(0) as u32,
            edge_y: tile_y.max(0) as u32,
        },
        (1, 2) => PaintTarget::HorizontalEdge {
            tile_x: tile_x.max(0) as u32,
            edge_y: (tile_y + 1).max(0) as u32,
        },
        (0, 1) => PaintTarget::VerticalEdge {
            edge_x: tile_x.max(0) as u32,
            tile_y: tile_y.max(0) as u32,
        },
        (2, 1) => PaintTarget::VerticalEdge {
            edge_x: (tile_x + 1).max(0) as u32,
            tile_y: tile_y.max(0) as u32,
        },
        // Center - default to painting all adjacent corners
        (1, 1) => PaintTarget::Corner {
            corner_x: tile_x.max(0) as u32,
            corner_y: tile_y.max(0) as u32,
        },
        _ => unreachable!(),
    }
}

/// Paint terrain at a horizontal edge (between tiles vertically)
/// A horizontal edge at (tile_x, edge_y) is shared by:
/// - Tile (tile_x, edge_y - 1): at its Bottom edge (position 4)
/// - Tile (tile_x, edge_y): at its Top edge (position 0)
///
/// To create continuous strokes, we also paint the corners at both ends of the edge.
/// The horizontal edge spans from corner (tile_x, edge_y) to corner (tile_x+1, edge_y).
pub fn paint_terrain_horizontal_edge(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    tile_x: u32,
    edge_y: u32,
    terrain_set: &TerrainSet,
    terrain_index: usize,
) {
    bevy::log::info!(
        "paint_terrain_horizontal_edge: tile_x={} edge_y={} terrain_idx={}",
        tile_x, edge_y, terrain_index
    );

    let tx = tile_x as i32;
    let ey = edge_y as i32;

    let mut filler = WangFiller::new(terrain_set);
    let mut region = Vec::new();

    // Edge positions: WangId 0=Top, 4=Bottom
    // For the tile above (ey-1): set Bottom edge (4)
    // For the tile below (ey): set Top edge (0)

    // Also set corners at both ends of the edge to create continuous strokes:
    // Left corner (tx, ey):
    //   - Tile (tx-1, ey-1): BR corner (3)
    //   - Tile (tx, ey-1): BL corner (5)
    //   - Tile (tx-1, ey): TR corner (1)
    //   - Tile (tx, ey): TL corner (7)
    // Right corner (tx+1, ey):
    //   - Tile (tx, ey-1): BR corner (3)
    //   - Tile (tx+1, ey-1): BL corner (5)
    //   - Tile (tx, ey): TR corner (1)
    //   - Tile (tx+1, ey): TL corner (7)

    // Affected tiles: tile position, list of (position, terrain_index)
    let affected: Vec<(i32, i32, Vec<usize>)> = vec![
        // Main edge tiles
        (tx, ey - 1, vec![4, 3, 5]),     // Tile above: Bottom edge + BL/BR corners
        (tx, ey, vec![0, 1, 7]),         // Tile below: Top edge + TL/TR corners
        // Left corner tiles (tx-1)
        (tx - 1, ey - 1, vec![3]),       // Above-left: BR corner
        (tx - 1, ey, vec![1]),           // Below-left: TR corner
        // Right corner tiles (tx+1)
        (tx + 1, ey - 1, vec![5]),       // Above-right: BL corner
        (tx + 1, ey, vec![7]),           // Below-right: TL corner
    ];

    for (tile_x, tile_y, positions) in &affected {
        if *tile_x >= 0 && *tile_y >= 0 && *tile_x < width as i32 && *tile_y < height as i32 {
            let cell = filler.get_cell_mut(*tile_x, *tile_y);
            for &pos in positions {
                cell.desired.colors[pos] = Some(terrain_index);
                cell.mask[pos] = true;
            }
            if !region.contains(&(*tile_x, *tile_y)) {
                region.push((*tile_x, *tile_y));
            }

            bevy::log::debug!(
                "paint_terrain_horizontal_edge: tile ({},{}) positions {:?} set to terrain {}",
                tile_x, tile_y, positions, terrain_index
            );
        }
    }

    filler.apply(tiles, width, height, &region);
}

/// Paint terrain at a vertical edge (between tiles horizontally)
/// A vertical edge at (edge_x, tile_y) is shared by:
/// - Tile (edge_x - 1, tile_y): at its Right edge (position 2)
/// - Tile (edge_x, tile_y): at its Left edge (position 6)
///
/// To create continuous strokes, we also paint the corners at both ends of the edge.
/// The vertical edge spans from corner (edge_x, tile_y) to corner (edge_x, tile_y+1).
pub fn paint_terrain_vertical_edge(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    edge_x: u32,
    tile_y: u32,
    terrain_set: &TerrainSet,
    terrain_index: usize,
) {
    bevy::log::info!(
        "paint_terrain_vertical_edge: edge_x={} tile_y={} terrain_idx={}",
        edge_x, tile_y, terrain_index
    );

    let ex = edge_x as i32;
    let ty = tile_y as i32;

    let mut filler = WangFiller::new(terrain_set);
    let mut region = Vec::new();

    // Edge positions: WangId 2=Right, 6=Left
    // For the tile to the left (ex-1): set Right edge (2)
    // For the tile to the right (ex): set Left edge (6)

    // Also set corners at both ends of the edge to create continuous strokes:
    // Top corner (ex, ty):
    //   - Tile (ex-1, ty-1): BR corner (3)
    //   - Tile (ex, ty-1): BL corner (5)
    //   - Tile (ex-1, ty): TR corner (1)
    //   - Tile (ex, ty): TL corner (7)
    // Bottom corner (ex, ty+1):
    //   - Tile (ex-1, ty): BR corner (3)
    //   - Tile (ex, ty): BL corner (5)
    //   - Tile (ex-1, ty+1): TR corner (1)
    //   - Tile (ex, ty+1): TL corner (7)

    // Affected tiles: tile position, list of positions to set
    let affected: Vec<(i32, i32, Vec<usize>)> = vec![
        // Main edge tiles
        (ex - 1, ty, vec![2, 1, 3]),     // Tile left: Right edge + TR/BR corners
        (ex, ty, vec![6, 5, 7]),         // Tile right: Left edge + TL/BL corners
        // Top corner tiles (ty-1)
        (ex - 1, ty - 1, vec![3]),       // Above-left: BR corner
        (ex, ty - 1, vec![5]),           // Above-right: BL corner
        // Bottom corner tiles (ty+1)
        (ex - 1, ty + 1, vec![1]),       // Below-left: TR corner
        (ex, ty + 1, vec![7]),           // Below-right: TL corner
    ];

    for (tile_x, tile_y, positions) in &affected {
        if *tile_x >= 0 && *tile_y >= 0 && *tile_x < width as i32 && *tile_y < height as i32 {
            let cell = filler.get_cell_mut(*tile_x, *tile_y);
            for &pos in positions {
                cell.desired.colors[pos] = Some(terrain_index);
                cell.mask[pos] = true;
            }
            if !region.contains(&(*tile_x, *tile_y)) {
                region.push((*tile_x, *tile_y));
            }

            bevy::log::debug!(
                "paint_terrain_vertical_edge: tile ({},{}) positions {:?} set to terrain {}",
                tile_x, tile_y, positions, terrain_index
            );
        }
    }

    filler.apply(tiles, width, height, &region);
}

/// Unified terrain painting function that handles corners and edges based on PaintTarget
pub fn paint_terrain_at_target(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    target: PaintTarget,
    terrain_set: &TerrainSet,
    terrain_index: usize,
) {
    match target {
        PaintTarget::Corner { corner_x, corner_y } => {
            paint_terrain(tiles, width, height, corner_x, corner_y, terrain_set, terrain_index);
        }
        PaintTarget::HorizontalEdge { tile_x, edge_y } => {
            paint_terrain_horizontal_edge(tiles, width, height, tile_x, edge_y, terrain_set, terrain_index);
        }
        PaintTarget::VerticalEdge { edge_x, tile_y } => {
            paint_terrain_vertical_edge(tiles, width, height, edge_x, tile_y, terrain_set, terrain_index);
        }
    }
}

/// Get the WangId positions used by each terrain set type
/// Corner sets: corners (1,3,5,7)
/// Edge sets: edges (0,2,4,6)
/// Mixed sets: all (0-7)
fn get_used_wang_positions(set_type: TerrainSetType) -> Vec<usize> {
    match set_type {
        TerrainSetType::Corner => vec![1, 3, 5, 7], // TL=7, TR=1, BR=3, BL=5 in Tiled
        TerrainSetType::Edge => vec![0, 2, 4, 6],   // Top=0, Right=2, Bottom=4, Left=6
        TerrainSetType::Mixed => (0..8).collect(),
    }
}

/// Update neighbors with a Tiled-style corrections queue (DEPRECATED - use WangFiller)
/// This ensures proper transitions by iteratively refining edge tiles
#[allow(dead_code)]
fn update_neighbors_with_corrections(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    center_x: u32,
    center_y: u32,
    terrain_set: &TerrainSet,
    painted_terrain: usize,
) {
    use std::collections::VecDeque;

    let position_count = terrain_set.set_type.position_count();

    // Corrections queue: tiles that need reconsideration
    let mut corrections: VecDeque<(u32, u32)> = VecDeque::new();

    // First pass: update all immediate neighbors
    let neighbor_offsets: [(i32, i32); 8] = [
        (-1, -1), (0, -1), (1, -1),
        (-1, 0),          (1, 0),
        (-1, 1),  (0, 1),  (1, 1),
    ];

    for &(dx, dy) in &neighbor_offsets {
        let nx = center_x as i32 + dx;
        let ny = center_y as i32 + dy;

        if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
            continue;
        }

        let nx = nx as u32;
        let ny = ny as u32;
        let nidx = (ny * width + nx) as usize;

        // Get current tile and its terrain data (if any)
        let current_tile = tiles.get(nidx).copied().flatten();
        let tile_data = current_tile.and_then(|t| terrain_set.get_tile_terrain(t));

        // Build constraints for this neighbor
        let mut constraints = TileConstraints::new();

        // Start with the tile's current terrain as SOFT preferences (if it has terrain data)
        if let Some(data) = &tile_data {
            for i in 0..position_count {
                if let Some(t) = data.get(i) {
                    constraints.set_desired(i, t);
                }
            }
        }

        // Constrain positions that face the painted center tile
        // These MUST be the painted terrain (hard constraint)
        let facing_center = get_facing_positions(terrain_set.set_type, -(dx), -(dy));
        for pos in &facing_center {
            constraints.set(*pos, painted_terrain);
        }

        // For positions NOT facing the center, prefer terrain index 0 (background/empty)
        // if there's no existing terrain data - this creates proper transition tiles
        if tile_data.is_none() {
            for i in 0..position_count {
                if !constraints.is_constrained(i) && constraints.desired[i].is_none() {
                    // Use terrain 0 (typically background/empty) for non-facing positions
                    constraints.set_desired(i, 0);
                }
            }
        }

        // Also check other neighbors of this tile and constrain accordingly
        for &(ddx, ddy) in &neighbor_offsets {
            let nnx = nx as i32 + ddx;
            let nny = ny as i32 + ddy;

            if nnx < 0 || nny < 0 || nnx >= width as i32 || nny >= height as i32 {
                continue;
            }

            // Skip the center tile we just painted
            if nnx == center_x as i32 && nny == center_y as i32 {
                continue;
            }

            let nnidx = (nny as u32 * width + nnx as u32) as usize;
            if let Some(other_tile) = tiles.get(nnidx).copied().flatten() {
                if let Some(other_data) = terrain_set.get_tile_terrain(other_tile) {
                    // Get positions on THIS neighbor that face the OTHER neighbor
                    let our_facing = get_facing_positions(terrain_set.set_type, ddx, ddy);
                    let their_facing = get_neighbor_facing_positions(terrain_set.set_type, ddx, ddy);

                    for (our_pos, their_pos) in our_facing.iter().zip(their_facing.iter()) {
                        // Don't overwrite positions that are already hard-constrained
                        // (facing the center tile)
                        if constraints.is_constrained(*our_pos) {
                            continue;
                        }
                        if let Some(terrain) = other_data.get(*their_pos) {
                            // Soft constraint - prefer to match
                            constraints.set_desired(*our_pos, terrain);
                        }
                    }
                }
            }
        }

        // Find best tile for this neighbor
        if let Some(new_tile) = terrain_set.find_matching_tile(&constraints) {
            let should_update = match current_tile {
                Some(ct) => new_tile != ct,
                None => true, // Always place if neighbor was empty
            };
            if should_update {
                tiles[nidx] = Some(new_tile);
                // Queue edge neighbors for potential correction
                corrections.push_back((nx, ny));
            }
        }
    }

    // Second pass: re-evaluate immediate neighbors only (no cascading)
    // The corrections queue contains tiles that changed, but we only want to
    // re-check them once with updated neighbor information, not cascade further.
    // We limit this to only tiles within 1 cell of the original center.
    let tiles_to_recheck: Vec<_> = corrections.drain(..).collect();

    for (tx, ty) in tiles_to_recheck {
        // Only process tiles that are immediate neighbors of the original center
        let dist_x = (tx as i32 - center_x as i32).abs();
        let dist_y = (ty as i32 - center_y as i32).abs();
        if dist_x > 1 || dist_y > 1 {
            continue;
        }

        let tidx = (ty * width + tx) as usize;
        let Some(current_tile) = tiles.get(tidx).copied().flatten() else { continue };
        let Some(tile_data) = terrain_set.get_tile_terrain(current_tile) else { continue };

        // Build constraints from current tile's terrain data (soft preference)
        let mut constraints = TileConstraints::new();
        for i in 0..position_count {
            if let Some(t) = tile_data.get(i) {
                constraints.set_desired(i, t);
            }
        }

        // Check all neighbors and constrain facing positions
        for &(dx, dy) in &neighbor_offsets {
            let nx = tx as i32 + dx;
            let ny = ty as i32 + dy;

            if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                continue;
            }

            let nidx = (ny as u32 * width + nx as u32) as usize;
            if let Some(other_tile) = tiles.get(nidx).copied().flatten() {
                if let Some(other_data) = terrain_set.get_tile_terrain(other_tile) {
                    let our_facing = get_facing_positions(terrain_set.set_type, dx, dy);
                    let their_facing = get_neighbor_facing_positions(terrain_set.set_type, dx, dy);

                    for (our_pos, their_pos) in our_facing.iter().zip(their_facing.iter()) {
                        if constraints.is_constrained(*our_pos) {
                            continue;
                        }
                        if let Some(terrain) = other_data.get(*their_pos) {
                            // Use hard constraints for positions facing the center tile
                            // since we know the center tile's terrain is definitive
                            if nx == center_x as i32 && ny == center_y as i32 {
                                constraints.set(*our_pos, terrain);
                            } else {
                                constraints.set_desired(*our_pos, terrain);
                            }
                        }
                    }
                }
            }
        }

        if let Some(new_tile) = terrain_set.find_matching_tile(&constraints) {
            if new_tile != current_tile {
                tiles[tidx] = Some(new_tile);
            }
        }
    }
}

/// Update a single tile to find the best match based on its neighbors
fn update_single_tile(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    terrain_set: &TerrainSet,
    primary_terrain: usize,
) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }

    let idx = (y as u32 * width + x as u32) as usize;
    let position_count = terrain_set.set_type.position_count();

    // Build constraints: start with the primary terrain for all positions (soft preference)
    let mut constraints = TileConstraints::new();
    for i in 0..position_count {
        constraints.set_desired(i, primary_terrain);
    }

    // Check each neighbor and CONSTRAIN facing positions based on their terrain
    let neighbor_offsets = [
        (-1, -1), (0, -1), (1, -1),
        (-1, 0),          (1, 0),
        (-1, 1),  (0, 1),  (1, 1),
    ];

    for (dx, dy) in neighbor_offsets {
        let nx = x + dx;
        let ny = y + dy;

        if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
            continue;
        }

        let nidx = (ny as u32 * width + nx as u32) as usize;
        let neighbor_tile = tiles.get(nidx).copied().flatten();

        if let Some(tile) = neighbor_tile {
            if let Some(neighbor_data) = terrain_set.get_tile_terrain(tile) {
                // Get positions on THIS tile that face the neighbor
                let our_facing = get_facing_positions(terrain_set.set_type, dx, dy);
                // Get positions on the NEIGHBOR that face us
                let their_facing = get_neighbor_facing_positions(terrain_set.set_type, dx, dy);

                // HARD CONSTRAIN our facing positions to match what neighbor exposes
                // Don't overwrite positions already constrained by earlier neighbors
                for (our_pos, their_pos) in our_facing.iter().zip(their_facing.iter()) {
                    if constraints.is_constrained(*our_pos) {
                        continue;
                    }
                    if let Some(terrain) = neighbor_data.get(*their_pos) {
                        constraints.set(*our_pos, terrain); // Hard constraint
                    }
                }
            }
        }
    }

    // Find best tile that matches constraints
    if let Some(new_tile) = terrain_set.find_matching_tile(&constraints) {
        tiles[idx] = Some(new_tile);
    }
    // If no match found, keep the existing tile
}

/// Public wrapper to update a single tile based on its neighbors
/// This is used by the rectangle fill tool
pub fn update_tile_with_neighbors(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    terrain_set: &TerrainSet,
    primary_terrain: usize,
) {
    update_single_tile(tiles, width, height, x, y, terrain_set, primary_terrain);
}

/// Update neighboring tiles after a terrain change (legacy version, deprecated)
/// Use update_neighbors_with_corrections for better results
#[allow(dead_code)]
fn update_neighbors(
    tiles: &mut [Option<u32>],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    terrain_set: &TerrainSet,
    painted_terrain: usize,
) {
    let neighbors = [
        (-1, -1), (0, -1), (1, -1),
        (-1, 0),          (1, 0),
        (-1, 1),  (0, 1),  (1, 1),
    ];

    let position_count = terrain_set.set_type.position_count();

    for (dx, dy) in neighbors {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;

        if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
            continue;
        }

        let nidx = (ny as u32 * width + nx as u32) as usize;
        let current_tile = tiles.get(nidx).copied().flatten();

        // Only update neighbors that already have tiles with terrain data
        let Some(tile) = current_tile else { continue };
        let Some(tile_data) = terrain_set.get_tile_terrain(tile) else { continue };

        // Build constraints for the neighbor:
        // - Start with the neighbor's current terrain data (soft preferences)
        // - Override positions that face the painted tile with the painted terrain (hard)
        let mut constraints = TileConstraints::new();

        // Copy the neighbor's current terrain data as soft preferences
        for i in 0..position_count {
            if let Some(terrain) = tile_data.get(i) {
                constraints.set_desired(i, terrain);
            }
        }

        // Hard constrain positions that face the painted tile
        // dx, dy point FROM the painted tile TO the neighbor
        // So positions on neighbor that face painted tile are in direction (-dx, -dy)
        let facing_positions = get_facing_positions(terrain_set.set_type, -dx, -dy);
        for pos in facing_positions {
            constraints.set(pos, painted_terrain);
        }

        // Find a matching tile for these constraints
        if let Some(new_tile) = terrain_set.find_matching_tile(&constraints) {
            tiles[nidx] = Some(new_tile);
        }
        // If no match found, leave the tile unchanged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_set_type_position_count() {
        assert_eq!(TerrainSetType::Corner.position_count(), 4);
        assert_eq!(TerrainSetType::Edge.position_count(), 4);
        assert_eq!(TerrainSetType::Mixed.position_count(), 8);
    }

    #[test]
    fn test_tile_terrain_data() {
        let mut data = TileTerrainData::new();
        assert!(!data.has_any_terrain());

        data.set(0, Some(0));
        assert!(data.has_any_terrain());
        assert_eq!(data.get(0), Some(0));
        assert_eq!(data.get(1), None);
    }

    #[test]
    fn test_terrain_set_find_uniform() {
        let mut set = TerrainSet::new(
            "Test".to_string(),
            Uuid::new_v4(),
            TerrainSetType::Corner,
        );

        set.add_terrain("Grass".to_string(), Color::srgb(0.0, 1.0, 0.0));

        // Add a uniform tile (all corners = grass)
        let mut tile_data = TileTerrainData::new();
        for i in 0..4 {
            tile_data.set(i, Some(0));
        }
        set.tile_terrains.insert(42, tile_data);

        let uniform_tiles = set.find_uniform_tiles(0);
        assert!(uniform_tiles.contains(&42));
    }
}
