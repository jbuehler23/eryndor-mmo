//! Editor command trait and implementations
//!
//! Commands encapsulate editor operations for undo/redo support.

use std::collections::HashMap;
use uuid::Uuid;

use crate::map::LayerData;
use crate::project::Project;

/// Trait for editor commands that can be executed and undone
pub trait EditorCommand: Send + Sync {
    /// Execute the command, returning an inverse command for undo
    /// Returns None if the command cannot be executed
    fn execute(&self, project: &mut Project) -> Option<Box<dyn EditorCommand>>;

    /// Human-readable description for UI display
    fn description(&self) -> &str;
}

/// Command to set a single tile
pub struct SetTileCommand {
    pub level_id: Uuid,
    pub layer_idx: usize,
    pub x: u32,
    pub y: u32,
    pub old_tile: Option<u32>,
    pub new_tile: Option<u32>,
}

impl SetTileCommand {
    pub fn new(
        level_id: Uuid,
        layer_idx: usize,
        x: u32,
        y: u32,
        old_tile: Option<u32>,
        new_tile: Option<u32>,
    ) -> Self {
        Self {
            level_id,
            layer_idx,
            x,
            y,
            old_tile,
            new_tile,
        }
    }
}

impl EditorCommand for SetTileCommand {
    fn execute(&self, project: &mut Project) -> Option<Box<dyn EditorCommand>> {
        let level = project.get_level_mut(self.level_id)?;
        level.set_tile(self.layer_idx, self.x, self.y, self.new_tile);
        project.mark_dirty();

        // Return inverse command
        Some(Box::new(SetTileCommand {
            level_id: self.level_id,
            layer_idx: self.layer_idx,
            x: self.x,
            y: self.y,
            old_tile: self.new_tile,
            new_tile: self.old_tile,
        }))
    }

    fn description(&self) -> &str {
        if self.new_tile.is_some() {
            "Paint Tile"
        } else {
            "Erase Tile"
        }
    }
}

/// Command to set multiple tiles at once (for rectangle fill, terrain, etc.)
pub struct BatchTileCommand {
    pub level_id: Uuid,
    pub layer_idx: usize,
    /// Map of (x, y) -> (old_tile, new_tile)
    pub changes: HashMap<(u32, u32), (Option<u32>, Option<u32>)>,
    pub description_text: String,
}

impl BatchTileCommand {
    pub fn new(
        level_id: Uuid,
        layer_idx: usize,
        changes: HashMap<(u32, u32), (Option<u32>, Option<u32>)>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            level_id,
            layer_idx,
            changes,
            description_text: description.into(),
        }
    }

    /// Create a BatchTileCommand from before/after tile state snapshots
    pub fn from_diff(
        level_id: Uuid,
        layer_idx: usize,
        before: HashMap<(u32, u32), Option<u32>>,
        after: HashMap<(u32, u32), Option<u32>>,
        description: impl Into<String>,
    ) -> Self {
        let mut changes = HashMap::new();

        // Collect all positions that changed
        for (pos, new_tile) in &after {
            let old_tile = before.get(pos).copied().flatten();
            if old_tile != *new_tile {
                changes.insert(*pos, (old_tile, *new_tile));
            }
        }

        // Also check for tiles that existed before but not in after
        for (pos, old_tile) in &before {
            if !after.contains_key(pos) {
                changes.insert(*pos, (*old_tile, None));
            }
        }

        Self {
            level_id,
            layer_idx,
            changes,
            description_text: description.into(),
        }
    }
}

impl EditorCommand for BatchTileCommand {
    fn execute(&self, project: &mut Project) -> Option<Box<dyn EditorCommand>> {
        let level = project.get_level_mut(self.level_id)?;

        // Apply all changes
        for ((x, y), (_, new_tile)) in &self.changes {
            level.set_tile(self.layer_idx, *x, *y, *new_tile);
        }
        project.mark_dirty();

        // Create inverse command (swap old and new)
        let inverse_changes: HashMap<(u32, u32), (Option<u32>, Option<u32>)> = self
            .changes
            .iter()
            .map(|(pos, (old, new))| (*pos, (*new, *old)))
            .collect();

        Some(Box::new(BatchTileCommand {
            level_id: self.level_id,
            layer_idx: self.layer_idx,
            changes: inverse_changes,
            description_text: format!("Undo {}", self.description_text),
        }))
    }

    fn description(&self) -> &str {
        &self.description_text
    }
}

/// Command to change layer tileset (for when painting to empty layer)
pub struct SetLayerTilesetCommand {
    pub level_id: Uuid,
    pub layer_idx: usize,
    pub old_tileset: Uuid,
    pub new_tileset: Uuid,
}

impl SetLayerTilesetCommand {
    pub fn new(level_id: Uuid, layer_idx: usize, old_tileset: Uuid, new_tileset: Uuid) -> Self {
        Self {
            level_id,
            layer_idx,
            old_tileset,
            new_tileset,
        }
    }
}

impl EditorCommand for SetLayerTilesetCommand {
    fn execute(&self, project: &mut Project) -> Option<Box<dyn EditorCommand>> {
        let level = project.get_level_mut(self.level_id)?;
        let layer = level.layers.get_mut(self.layer_idx)?;

        if let LayerData::Tiles { tileset_id, .. } = &mut layer.data {
            *tileset_id = self.new_tileset;
            project.mark_dirty();

            return Some(Box::new(SetLayerTilesetCommand {
                level_id: self.level_id,
                layer_idx: self.layer_idx,
                old_tileset: self.new_tileset,
                new_tileset: self.old_tileset,
            }));
        }

        None
    }

    fn description(&self) -> &str {
        "Change Layer Tileset"
    }
}

/// Composite command that groups multiple commands together
pub struct CompositeCommand {
    pub commands: Vec<Box<dyn EditorCommand>>,
    pub description_text: String,
}

impl CompositeCommand {
    pub fn new(commands: Vec<Box<dyn EditorCommand>>, description: impl Into<String>) -> Self {
        Self {
            commands,
            description_text: description.into(),
        }
    }
}

impl EditorCommand for CompositeCommand {
    fn execute(&self, project: &mut Project) -> Option<Box<dyn EditorCommand>> {
        let mut inverse_commands = Vec::new();

        // Execute all commands
        for cmd in &self.commands {
            if let Some(inverse) = cmd.execute(project) {
                inverse_commands.push(inverse);
            }
        }

        if inverse_commands.is_empty() {
            return None;
        }

        // Reverse the order for undo
        inverse_commands.reverse();

        Some(Box::new(CompositeCommand {
            commands: inverse_commands,
            description_text: format!("Undo {}", self.description_text),
        }))
    }

    fn description(&self) -> &str {
        &self.description_text
    }
}

/// Helper to collect tile state in a region for creating diff-based commands
pub fn collect_tiles_in_region(
    project: &Project,
    level_id: Uuid,
    layer_idx: usize,
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
) -> HashMap<(u32, u32), Option<u32>> {
    let mut tiles = HashMap::new();

    if let Some(level) = project.levels.iter().find(|l| l.id == level_id) {
        for y in min_y.max(0)..=max_y.min(level.height as i32 - 1) {
            for x in min_x.max(0)..=max_x.min(level.width as i32 - 1) {
                let tile = level.get_tile(layer_idx, x as u32, y as u32);
                tiles.insert((x as u32, y as u32), tile);
            }
        }
    }

    tiles
}
