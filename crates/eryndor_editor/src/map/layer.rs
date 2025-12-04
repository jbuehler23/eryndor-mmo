use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A layer (tiles or objects)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub data: LayerData,
}

impl Layer {
    pub fn new_tile_layer(name: String, tileset_id: Uuid, width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            name,
            visible: true,
            data: LayerData::Tiles {
                tileset_id,
                tiles: vec![None; size],
            },
        }
    }

    pub fn new_object_layer(name: String) -> Self {
        Self {
            name,
            visible: true,
            data: LayerData::Objects {
                entities: Vec::new(),
            },
        }
    }

    pub fn layer_type(&self) -> LayerType {
        match &self.data {
            LayerData::Tiles { .. } => LayerType::Tiles,
            LayerData::Objects { .. } => LayerType::Objects,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerType {
    Tiles,
    Objects,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerData {
    Tiles {
        tileset_id: Uuid,
        tiles: Vec<Option<u32>>,
    },
    Objects {
        entities: Vec<Uuid>,
    },
}
