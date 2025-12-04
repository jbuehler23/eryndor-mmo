use super::Project;
use std::path::Path;

/// Export project to game-ready JSON format
pub fn export_project(project: &Project, output_dir: &Path) -> Result<(), ExportError> {
    // Create output directories
    let data_dir = output_dir.join("data");
    let levels_dir = output_dir.join("levels");

    std::fs::create_dir_all(&data_dir).map_err(|e| ExportError::IoError(e.to_string()))?;
    std::fs::create_dir_all(&levels_dir).map_err(|e| ExportError::IoError(e.to_string()))?;

    // Copy schema
    let schema_json = serde_json::to_string_pretty(&project.schema)
        .map_err(|e| ExportError::SerializeError(e.to_string()))?;
    std::fs::write(output_dir.join("schema.json"), schema_json)
        .map_err(|e| ExportError::IoError(e.to_string()))?;

    // Export each data type to its own file
    for type_name in project.schema.data_type_names() {
        let instances = project.data.get_by_type(type_name);
        if !instances.is_empty() {
            let export = DataTypeExport {
                type_name: type_name.to_string(),
                instances: instances
                    .iter()
                    .map(|i| InstanceExport {
                        id: i.id.to_string(),
                        properties: i.properties.iter()
                            .map(|(k, v)| (k.clone(), v.to_json()))
                            .collect(),
                    })
                    .collect(),
            };

            let json = serde_json::to_string_pretty(&export)
                .map_err(|e| ExportError::SerializeError(e.to_string()))?;
            std::fs::write(data_dir.join(format!("{}.json", type_name)), json)
                .map_err(|e| ExportError::IoError(e.to_string()))?;
        }
    }

    // Export each level to its own file
    for level in &project.levels {
        let export = LevelExport {
            name: level.name.clone(),
            width: level.width,
            height: level.height,
            tile_size: project.schema.project.tile_size,
            layers: level.layers.iter()
                .map(|l| LayerExport {
                    name: l.name.clone(),
                    layer_type: match &l.data {
                        crate::map::LayerData::Tiles { .. } => "tiles".to_string(),
                        crate::map::LayerData::Objects { .. } => "objects".to_string(),
                    },
                    tileset: match &l.data {
                        crate::map::LayerData::Tiles { tileset_id, .. } => {
                            project.tilesets.iter()
                                .find(|t| t.id == *tileset_id)
                                .map(|t| t.name.clone())
                        }
                        _ => None,
                    },
                    data: match &l.data {
                        crate::map::LayerData::Tiles { tiles, .. } => Some(tiles.clone()),
                        _ => None,
                    },
                })
                .collect(),
            entities: level.entities.iter()
                .map(|e| EntityExport {
                    id: e.id.to_string(),
                    entity_type: e.type_name.clone(),
                    position: [e.position.x, e.position.y],
                    template_id: e.template_id.map(|id| id.to_string()),
                    properties: e.properties.iter()
                        .map(|(k, v)| (k.clone(), v.to_json()))
                        .collect(),
                })
                .collect(),
        };

        let json = serde_json::to_string_pretty(&export)
            .map_err(|e| ExportError::SerializeError(e.to_string()))?;
        std::fs::write(levels_dir.join(format!("{}.json", level.name)), json)
            .map_err(|e| ExportError::IoError(e.to_string()))?;
    }

    Ok(())
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct DataTypeExport {
    #[serde(rename = "$type")]
    type_name: String,
    instances: Vec<InstanceExport>,
}

#[derive(Serialize, Deserialize)]
struct InstanceExport {
    id: String,
    properties: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct LevelExport {
    name: String,
    width: u32,
    height: u32,
    tile_size: u32,
    layers: Vec<LayerExport>,
    entities: Vec<EntityExport>,
}

#[derive(Serialize, Deserialize)]
struct LayerExport {
    name: String,
    #[serde(rename = "type")]
    layer_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tileset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Vec<Option<u32>>>,
}

#[derive(Serialize, Deserialize)]
struct EntityExport {
    id: String,
    #[serde(rename = "type")]
    entity_type: String,
    position: [f32; 2],
    #[serde(skip_serializing_if = "Option::is_none")]
    template_id: Option<String>,
    properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
pub enum ExportError {
    IoError(String),
    SerializeError(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::IoError(e) => write!(f, "IO error: {}", e),
            ExportError::SerializeError(e) => write!(f, "Serialize error: {}", e),
        }
    }
}

impl std::error::Error for ExportError {}
