mod layer;
mod entity;

pub use layer::*;
pub use entity::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A level/map containing tiles and entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub id: Uuid,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub layers: Vec<Layer>,
    pub entities: Vec<EntityInstance>,
}

impl Level {
    pub fn new(name: String, width: u32, height: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            width,
            height,
            layers: Vec::new(),
            entities: Vec::new(),
        }
    }

    /// Add a new layer
    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    /// Add an entity to the level
    pub fn add_entity(&mut self, entity: EntityInstance) {
        self.entities.push(entity);
    }

    /// Remove an entity by ID
    pub fn remove_entity(&mut self, id: Uuid) -> Option<EntityInstance> {
        self.entities
            .iter()
            .position(|e| e.id == id)
            .map(|pos| self.entities.remove(pos))
    }

    /// Get entity by ID
    pub fn get_entity(&self, id: Uuid) -> Option<&EntityInstance> {
        self.entities.iter().find(|e| e.id == id)
    }

    /// Get mutable entity by ID
    pub fn get_entity_mut(&mut self, id: Uuid) -> Option<&mut EntityInstance> {
        self.entities.iter_mut().find(|e| e.id == id)
    }

    /// Get tile at position for a specific layer
    pub fn get_tile(&self, layer_index: usize, x: u32, y: u32) -> Option<u32> {
        if let Some(layer) = self.layers.get(layer_index) {
            if let LayerData::Tiles { tiles, .. } = &layer.data {
                let index = (y * self.width + x) as usize;
                return tiles.get(index).copied().flatten();
            }
        }
        None
    }

    /// Set tile at position for a specific layer
    pub fn set_tile(&mut self, layer_index: usize, x: u32, y: u32, tile: Option<u32>) {
        if let Some(layer) = self.layers.get_mut(layer_index) {
            if let LayerData::Tiles { tiles, .. } = &mut layer.data {
                let index = (y * self.width + x) as usize;
                if index < tiles.len() {
                    tiles[index] = tile;
                }
            }
        }
    }

    /// Remove a layer by index
    pub fn remove_layer(&mut self, index: usize) -> Option<Layer> {
        if index < self.layers.len() {
            Some(self.layers.remove(index))
        } else {
            None
        }
    }

    /// Move a layer up (toward index 0)
    pub fn move_layer_up(&mut self, index: usize) -> bool {
        if index > 0 && index < self.layers.len() {
            self.layers.swap(index, index - 1);
            true
        } else {
            false
        }
    }

    /// Move a layer down (toward higher index)
    pub fn move_layer_down(&mut self, index: usize) -> bool {
        if index < self.layers.len().saturating_sub(1) {
            self.layers.swap(index, index + 1);
            true
        } else {
            false
        }
    }

    /// Toggle layer visibility
    pub fn toggle_layer_visibility(&mut self, index: usize) -> bool {
        if let Some(layer) = self.layers.get_mut(index) {
            layer.visible = !layer.visible;
            true
        } else {
            false
        }
    }

    /// Get layer by index
    pub fn get_layer(&self, index: usize) -> Option<&Layer> {
        self.layers.get(index)
    }

    /// Get mutable layer by index
    pub fn get_layer_mut(&mut self, index: usize) -> Option<&mut Layer> {
        self.layers.get_mut(index)
    }
}
