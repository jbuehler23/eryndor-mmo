use crate::schema::Value;
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// An entity placed in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInstance {
    pub id: Uuid,
    pub type_name: String,
    pub position: Vec2,
    /// If this is an instance of a template, the template ID
    pub template_id: Option<Uuid>,
    /// Property overrides (for template instances) or direct properties
    pub properties: HashMap<String, Value>,
}

impl EntityInstance {
    pub fn new(type_name: String, position: Vec2) -> Self {
        Self {
            id: Uuid::new_v4(),
            type_name,
            position,
            template_id: None,
            properties: HashMap::new(),
        }
    }

    pub fn from_template(template_id: Uuid, type_name: String, position: Vec2) -> Self {
        Self {
            id: Uuid::new_v4(),
            type_name,
            position,
            template_id: Some(template_id),
            properties: HashMap::new(),
        }
    }

    pub fn get_display_name(&self) -> String {
        self.properties
            .get("name")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{} ({})", self.type_name, &self.id.to_string()[..8]))
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.properties.get(key).and_then(|v| v.as_string())
    }

    pub fn set_string(&mut self, key: &str, value: String) {
        self.properties.insert(key.to_string(), Value::String(value));
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.properties.get(key).and_then(|v| v.as_int())
    }

    pub fn set_int(&mut self, key: &str, value: i64) {
        self.properties.insert(key.to_string(), Value::Int(value));
    }

    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.properties.get(key).and_then(|v| v.as_float())
    }

    pub fn set_float(&mut self, key: &str, value: f64) {
        self.properties.insert(key.to_string(), Value::Float(value));
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.properties.get(key).and_then(|v| v.as_bool())
    }

    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.properties.insert(key.to_string(), Value::Bool(value));
    }
}
