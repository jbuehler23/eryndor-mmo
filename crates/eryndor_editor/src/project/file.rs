use crate::autotile::AutotileConfig;
use crate::map::Level;
use crate::schema::{DataInstance, Schema};
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// The entire editor project
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct Project {
    pub version: u32,
    #[serde(skip)]
    pub path: Option<PathBuf>,
    #[serde(skip)]
    pub schema_path: Option<PathBuf>,
    pub schema: Schema,
    pub tilesets: Vec<Tileset>,
    pub data: DataStore,
    pub levels: Vec<Level>,
    /// Autotile terrain configuration
    #[serde(default)]
    pub autotile_config: AutotileConfig,
    #[serde(skip)]
    pub dirty: bool,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            version: 1,
            path: None,
            schema_path: None,
            schema: crate::schema::default_schema(),
            tilesets: Vec::new(),
            data: DataStore::default(),
            levels: Vec::new(),
            autotile_config: AutotileConfig::default(),
            dirty: false,
        }
    }
}

impl Project {
    pub fn new(schema: Schema) -> Self {
        Self {
            version: 1,
            path: None,
            schema_path: None,
            schema,
            tilesets: Vec::new(),
            data: DataStore::default(),
            levels: Vec::new(),
            autotile_config: AutotileConfig::default(),
            dirty: false,
        }
    }

    /// Load project from file
    pub fn load(path: &Path) -> Result<Self, ProjectError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ProjectError::IoError(e.to_string()))?;

        let mut project: Project =
            serde_json::from_str(&content).map_err(|e| ProjectError::ParseError(e.to_string()))?;

        project.path = Some(path.to_path_buf());
        project.dirty = false;

        Ok(project)
    }

    /// Save project to file
    pub fn save(&mut self, path: &Path) -> Result<(), ProjectError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ProjectError::SerializeError(e.to_string()))?;

        std::fs::write(path, content).map_err(|e| ProjectError::IoError(e.to_string()))?;

        self.path = Some(path.to_path_buf());
        self.dirty = false;

        Ok(())
    }

    /// Save to current path if set
    pub fn save_current(&mut self) -> Result<(), ProjectError> {
        if let Some(path) = self.path.clone() {
            self.save(&path)
        } else {
            Err(ProjectError::NoPath)
        }
    }

    /// Mark project as modified
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if project has unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Get project name (from path or schema)
    pub fn name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.schema.project.name.clone())
    }

    /// Add a new data instance
    pub fn add_data_instance(&mut self, instance: DataInstance) {
        self.data.add(instance);
        self.dirty = true;
    }

    /// Remove a data instance by ID
    pub fn remove_data_instance(&mut self, id: Uuid) -> Option<DataInstance> {
        let result = self.data.remove(id);
        if result.is_some() {
            self.dirty = true;
        }
        result
    }

    /// Get data instance by ID
    pub fn get_data_instance(&self, id: Uuid) -> Option<&DataInstance> {
        self.data.get(id)
    }

    /// Get mutable data instance by ID
    pub fn get_data_instance_mut(&mut self, id: Uuid) -> Option<&mut DataInstance> {
        let result = self.data.get_mut(id);
        if result.is_some() {
            self.dirty = true;
        }
        result
    }

    /// Count entities of a given type across all levels
    pub fn count_entities_of_type(&self, type_name: &str) -> usize {
        self.levels
            .iter()
            .map(|level| {
                level
                    .entities
                    .iter()
                    .filter(|e| e.type_name == type_name)
                    .count()
            })
            .sum()
    }

    /// Add a new level
    pub fn add_level(&mut self, level: Level) {
        self.levels.push(level);
        self.dirty = true;
    }

    /// Get level by ID
    pub fn get_level(&self, id: Uuid) -> Option<&Level> {
        self.levels.iter().find(|l| l.id == id)
    }

    /// Get mutable level by ID
    pub fn get_level_mut(&mut self, id: Uuid) -> Option<&mut Level> {
        self.dirty = true;
        self.levels.iter_mut().find(|l| l.id == id)
    }

    /// Duplicate a data instance by ID, returns the new instance's ID
    pub fn duplicate_data_instance(&mut self, id: Uuid) -> Option<Uuid> {
        let original = self.data.get(id)?.clone();
        let mut duplicate = original;
        duplicate.id = Uuid::new_v4();

        // Append " (Copy)" to the name if there's a name property
        if let Some(crate::schema::Value::String(name)) = duplicate.properties.get_mut("name") {
            name.push_str(" (Copy)");
        }

        let new_id = duplicate.id;
        self.data.add(duplicate);
        self.dirty = true;
        Some(new_id)
    }

    /// Remove a level by ID
    pub fn remove_level(&mut self, id: Uuid) -> Option<Level> {
        if let Some(pos) = self.levels.iter().position(|l| l.id == id) {
            self.dirty = true;
            Some(self.levels.remove(pos))
        } else {
            None
        }
    }

    /// Duplicate a level by ID, returns the new level's ID
    pub fn duplicate_level(&mut self, id: Uuid) -> Option<Uuid> {
        let original = self.get_level(id)?.clone();
        let mut duplicate = original;
        duplicate.id = Uuid::new_v4();
        duplicate.name = format!("{} (Copy)", duplicate.name);

        // Also assign new IDs to all entities and layers
        for entity in &mut duplicate.entities {
            entity.id = Uuid::new_v4();
        }

        let new_id = duplicate.id;
        self.levels.push(duplicate);
        self.dirty = true;
        Some(new_id)
    }
}

/// Stores all data_type instances (non-placeable things like Items, Quests)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataStore {
    /// Key: type name (e.g., "Item", "Quest")
    /// Value: list of instances of that type
    pub instances: HashMap<String, Vec<DataInstance>>,
}

impl DataStore {
    pub fn add(&mut self, instance: DataInstance) {
        self.instances
            .entry(instance.type_name.clone())
            .or_default()
            .push(instance);
    }

    pub fn remove(&mut self, id: Uuid) -> Option<DataInstance> {
        for instances in self.instances.values_mut() {
            if let Some(pos) = instances.iter().position(|i| i.id == id) {
                return Some(instances.remove(pos));
            }
        }
        None
    }

    pub fn get(&self, id: Uuid) -> Option<&DataInstance> {
        for instances in self.instances.values() {
            if let Some(instance) = instances.iter().find(|i| i.id == id) {
                return Some(instance);
            }
        }
        None
    }

    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut DataInstance> {
        for instances in self.instances.values_mut() {
            if let Some(instance) = instances.iter_mut().find(|i| i.id == id) {
                return Some(instance);
            }
        }
        None
    }

    pub fn get_by_type(&self, type_name: &str) -> &[DataInstance] {
        self.instances.get(type_name).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn all_instances(&self) -> impl Iterator<Item = &DataInstance> {
        self.instances.values().flatten()
    }
}

/// A single image source within a tileset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilesetImage {
    pub id: Uuid,
    pub name: String,
    pub path: PathBuf,
    pub columns: u32,
    pub rows: u32,
}

impl TilesetImage {
    pub fn new(name: String, path: PathBuf, columns: u32, rows: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            path,
            columns,
            rows,
        }
    }

    /// Total number of tiles in this image
    pub fn tile_count(&self) -> u32 {
        self.columns * self.rows
    }
}

/// Tileset configuration - can contain multiple images
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tileset {
    pub id: Uuid,
    pub name: String,
    /// Legacy single-image path (for backward compatibility)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    pub tile_size: u32,
    /// Legacy columns (for backward compatibility)
    #[serde(default)]
    pub columns: u32,
    /// Legacy rows (for backward compatibility)
    #[serde(default)]
    pub rows: u32,
    /// Multiple image sources (Godot-style)
    #[serde(default)]
    pub images: Vec<TilesetImage>,
    #[serde(default)]
    pub autotile_config: Option<AutotileConfig>,
}

impl Tileset {
    pub fn new(name: String, path: PathBuf, tile_size: u32, columns: u32, rows: u32) -> Self {
        // Create with single image for backward compatibility
        let image = TilesetImage::new(
            "Main".to_string(),
            path.clone(),
            columns,
            rows,
        );
        Self {
            id: Uuid::new_v4(),
            name,
            path: Some(path),
            tile_size,
            columns,
            rows,
            images: vec![image],
            autotile_config: None,
        }
    }

    /// Create a new empty tileset without an image
    pub fn new_empty(name: String, tile_size: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            path: None,
            tile_size,
            columns: 0,
            rows: 0,
            images: Vec::new(),
            autotile_config: None,
        }
    }

    /// Migrate legacy single-image format to multi-image format
    pub fn migrate_to_multi_image(&mut self) {
        if self.images.is_empty() {
            if let Some(path) = &self.path {
                let image = TilesetImage::new(
                    "Main".to_string(),
                    path.clone(),
                    self.columns,
                    self.rows,
                );
                self.images.push(image);
            }
        }
    }

    /// Add a new image to this tileset
    pub fn add_image(&mut self, name: String, path: PathBuf, columns: u32, rows: u32) -> Uuid {
        let image = TilesetImage::new(name, path, columns, rows);
        let id = image.id;
        self.images.push(image);
        id
    }

    /// Remove an image by ID
    pub fn remove_image(&mut self, id: Uuid) -> bool {
        if let Some(pos) = self.images.iter().position(|img| img.id == id) {
            self.images.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get total tile count across all images
    pub fn total_tile_count(&self) -> u32 {
        if self.images.is_empty() {
            // Legacy mode
            self.columns * self.rows
        } else {
            self.images.iter().map(|img| img.tile_count()).sum()
        }
    }

    /// Convert virtual tile index to (image_index, local_tile_index)
    /// Returns None if the index is out of bounds
    pub fn virtual_to_local(&self, virtual_index: u32) -> Option<(usize, u32)> {
        if self.images.is_empty() {
            // Legacy mode - single image
            if virtual_index < self.columns * self.rows {
                return Some((0, virtual_index));
            }
            return None;
        }

        let mut offset = 0u32;
        for (img_idx, image) in self.images.iter().enumerate() {
            let tile_count = image.tile_count();
            if virtual_index < offset + tile_count {
                return Some((img_idx, virtual_index - offset));
            }
            offset += tile_count;
        }
        None
    }

    /// Convert (image_index, local_tile_index) to virtual tile index
    pub fn local_to_virtual(&self, image_index: usize, local_index: u32) -> Option<u32> {
        if self.images.is_empty() {
            // Legacy mode
            if image_index == 0 && local_index < self.columns * self.rows {
                return Some(local_index);
            }
            return None;
        }

        if image_index >= self.images.len() {
            return None;
        }

        let image = &self.images[image_index];
        if local_index >= image.tile_count() {
            return None;
        }

        let offset: u32 = self.images[..image_index]
            .iter()
            .map(|img| img.tile_count())
            .sum();
        Some(offset + local_index)
    }

    /// Get image info for a virtual tile index
    pub fn get_tile_image_info(&self, virtual_index: u32) -> Option<(&TilesetImage, u32)> {
        let (img_idx, local_idx) = self.virtual_to_local(virtual_index)?;
        if self.images.is_empty() {
            // Legacy mode - return a temporary image-like struct info
            None
        } else {
            Some((&self.images[img_idx], local_idx))
        }
    }

    /// Get the first image path (for legacy compatibility)
    pub fn primary_path(&self) -> Option<&PathBuf> {
        if !self.images.is_empty() {
            Some(&self.images[0].path)
        } else {
            self.path.as_ref()
        }
    }

    /// Get image at index
    pub fn get_image(&self, index: usize) -> Option<&TilesetImage> {
        self.images.get(index)
    }

    /// Get mutable image at index
    pub fn get_image_mut(&mut self, index: usize) -> Option<&mut TilesetImage> {
        self.images.get_mut(index)
    }
}


#[derive(Debug)]
pub enum ProjectError {
    IoError(String),
    ParseError(String),
    SerializeError(String),
    NoPath,
}

impl std::fmt::Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectError::IoError(e) => write!(f, "IO error: {}", e),
            ProjectError::ParseError(e) => write!(f, "Parse error: {}", e),
            ProjectError::SerializeError(e) => write!(f, "Serialize error: {}", e),
            ProjectError::NoPath => write!(f, "No file path set"),
        }
    }
}

impl std::error::Error for ProjectError {}
