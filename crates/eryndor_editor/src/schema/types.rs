use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The schema loaded from schema.json - defines all types and enums
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Schema {
    pub version: u32,
    pub project: ProjectConfig,
    #[serde(default)]
    pub enums: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub data_types: HashMap<String, TypeDef>,
    #[serde(default)]
    pub embedded_types: HashMap<String, TypeDef>,
}

impl Schema {
    /// Get a type definition by name (checks data_types and embedded_types)
    pub fn get_type(&self, name: &str) -> Option<&TypeDef> {
        self.data_types
            .get(name)
            .or_else(|| self.embedded_types.get(name))
    }

    /// Get enum values by name
    pub fn get_enum(&self, name: &str) -> Option<&Vec<String>> {
        self.enums.get(name)
    }

    /// Get all type names sorted alphabetically
    pub fn all_type_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .data_types
            .keys()
            .map(|s| s.as_str())
            .collect();
        names.sort();
        names
    }

    /// Get all data type names sorted alphabetically
    pub fn data_type_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.data_types.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Get all placeable type names (types that can be placed in levels)
    pub fn placeable_type_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.data_types
            .iter()
            .filter(|(_, def)| def.placeable)
            .map(|(name, _)| name.as_str())
            .collect();
        names.sort();
        names
    }
}

/// Project-level configuration from schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default = "default_tile_size")]
    pub tile_size: u32,
    #[serde(default)]
    pub default_layer_types: Vec<String>,
}

fn default_tile_size() -> u32 {
    32
}

/// Definition of a type (from schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDef {
    #[serde(default = "default_color")]
    pub color: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub placeable: bool,
    #[serde(default)]
    pub properties: Vec<PropertyDef>,
}

fn default_color() -> String {
    "#808080".to_string()
}

impl Default for TypeDef {
    fn default() -> Self {
        Self {
            color: default_color(),
            icon: None,
            placeable: false,
            properties: Vec::new(),
        }
    }
}

/// Definition of a property (from schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDef {
    pub name: String,
    #[serde(rename = "type")]
    pub prop_type: PropType,
    #[serde(default)]
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    #[serde(rename = "showIf")]
    pub show_if: Option<String>,
    #[serde(rename = "enumType")]
    pub enum_type: Option<String>,
    #[serde(rename = "refType")]
    pub ref_type: Option<String>,
    #[serde(rename = "itemType")]
    pub item_type: Option<String>,
    #[serde(rename = "embeddedType")]
    pub embedded_type: Option<String>,
}

/// Property types supported by the schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PropType {
    String,
    Multiline,
    Int,
    Float,
    Bool,
    Enum,
    Ref,
    Array,
    Embedded,
    Point,
    Color,
    Sprite,
    Dialogue,
}

impl PropType {
    pub fn display_name(&self) -> &'static str {
        match self {
            PropType::String => "String",
            PropType::Multiline => "Multiline",
            PropType::Int => "Integer",
            PropType::Float => "Float",
            PropType::Bool => "Boolean",
            PropType::Enum => "Enum",
            PropType::Ref => "Reference",
            PropType::Array => "Array",
            PropType::Embedded => "Embedded",
            PropType::Point => "Point",
            PropType::Color => "Color",
            PropType::Sprite => "Sprite",
            PropType::Dialogue => "Dialogue Tree",
        }
    }
}

/// Animation loop mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LoopMode {
    #[default]
    Loop,
    Once,
    PingPong,
}

impl LoopMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            LoopMode::Loop => "Loop",
            LoopMode::Once => "Once",
            LoopMode::PingPong => "Ping-Pong",
        }
    }

    pub fn all() -> &'static [LoopMode] {
        &[LoopMode::Loop, LoopMode::Once, LoopMode::PingPong]
    }
}

/// A single animation definition
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnimationDef {
    /// Frame indices into the spritesheet grid (left-to-right, top-to-bottom)
    pub frames: Vec<usize>,
    /// Duration of each frame in milliseconds
    #[serde(default = "default_frame_duration")]
    pub frame_duration_ms: u32,
    /// How the animation loops
    #[serde(default)]
    pub loop_mode: LoopMode,
}

fn default_frame_duration() -> u32 {
    100
}

/// Sprite data with spritesheet reference and animations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpriteData {
    /// Path to the spritesheet image (relative to assets)
    pub sheet_path: String,
    /// Width of each frame in pixels
    pub frame_width: u32,
    /// Height of each frame in pixels
    pub frame_height: u32,
    /// Number of columns in the spritesheet (calculated from image width / frame_width)
    #[serde(default)]
    pub columns: u32,
    /// Number of rows in the spritesheet (calculated from image height / frame_height)
    #[serde(default)]
    pub rows: u32,
    /// Pivot point (0.0-1.0, where 0.5,0.5 is center)
    #[serde(default = "default_pivot")]
    pub pivot_x: f32,
    #[serde(default = "default_pivot")]
    pub pivot_y: f32,
    /// Named animations (user-defined names like "idle", "attack", "death", etc.)
    #[serde(default)]
    pub animations: HashMap<String, AnimationDef>,
}

fn default_pivot() -> f32 {
    0.5
}

impl SpriteData {
    /// Get total frame count based on grid
    pub fn total_frames(&self) -> usize {
        (self.columns * self.rows) as usize
    }

    /// Convert frame index to grid position (col, row)
    pub fn frame_to_grid(&self, frame: usize) -> (u32, u32) {
        if self.columns == 0 {
            return (0, 0);
        }
        let col = (frame as u32) % self.columns;
        let row = (frame as u32) / self.columns;
        (col, row)
    }

    /// Convert grid position to frame index
    pub fn grid_to_frame(&self, col: u32, row: u32) -> usize {
        (row * self.columns + col) as usize
    }

    /// Get pixel rect for a frame
    pub fn frame_rect(&self, frame: usize) -> (u32, u32, u32, u32) {
        let (col, row) = self.frame_to_grid(frame);
        (
            col * self.frame_width,
            row * self.frame_height,
            self.frame_width,
            self.frame_height,
        )
    }

    /// Convert to Value for storage
    pub fn to_value(&self) -> Value {
        let json = serde_json::to_value(self).unwrap_or(serde_json::Value::Null);
        Value::from_json(json)
    }

    /// Convert from Value
    pub fn from_value(value: &Value) -> Option<Self> {
        let json = value.to_json();
        serde_json::from_value(json).ok()
    }
}

/// Generic property value (JSON-like but typed)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl Value {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            Value::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Convert from serde_json::Value
    pub fn from_json(json: serde_json::Value) -> Self {
        match json {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Int(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::Null
                }
            }
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Array(arr) => {
                Value::Array(arr.into_iter().map(Value::from_json).collect())
            }
            serde_json::Value::Object(obj) => Value::Object(
                obj.into_iter()
                    .map(|(k, v)| (k, Value::from_json(v)))
                    .collect(),
            ),
        }
    }

    /// Convert to serde_json::Value
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Value::Null => serde_json::Value::Null,
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::Int(i) => serde_json::json!(*i),
            Value::Float(f) => serde_json::json!(*f),
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(|v| v.to_json()).collect())
            }
            Value::Object(obj) => serde_json::Value::Object(
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.to_json()))
                    .collect(),
            ),
        }
    }
}

// ============================================================================
// Dialogue Tree Types
// ============================================================================

/// A complete dialogue tree with nodes and connections
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DialogueTree {
    /// Unique identifier for the dialogue tree
    #[serde(default = "default_dialogue_id")]
    pub id: String,
    /// Display name for the dialogue
    #[serde(default)]
    pub name: String,
    /// The ID of the starting node
    #[serde(default)]
    pub start_node: String,
    /// All nodes in the dialogue tree
    #[serde(default)]
    pub nodes: HashMap<String, DialogueNode>,
}

fn default_dialogue_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

impl DialogueTree {
    /// Create a new empty dialogue tree
    pub fn new() -> Self {
        let start_id = uuid::Uuid::new_v4().to_string();
        let mut nodes = HashMap::new();
        nodes.insert(
            start_id.clone(),
            DialogueNode {
                id: start_id.clone(),
                node_type: DialogueNodeType::Text,
                speaker: String::new(),
                text: "Hello!".to_string(),
                choices: Vec::new(),
                next_node: None,
                condition: None,
                action: None,
                position: (100.0, 100.0),
            },
        );
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "New Dialogue".to_string(),
            start_node: start_id,
            nodes,
        }
    }

    /// Add a new node to the tree
    pub fn add_node(&mut self, node: DialogueNode) -> String {
        let id = node.id.clone();
        self.nodes.insert(id.clone(), node);
        id
    }

    /// Remove a node from the tree
    pub fn remove_node(&mut self, id: &str) {
        self.nodes.remove(id);
        // Clean up references to this node
        for node in self.nodes.values_mut() {
            if node.next_node.as_deref() == Some(id) {
                node.next_node = None;
            }
            node.choices.retain(|c| c.next_node.as_deref() != Some(id));
        }
        if self.start_node == id {
            self.start_node = String::new();
        }
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&DialogueNode> {
        self.nodes.get(id)
    }

    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut DialogueNode> {
        self.nodes.get_mut(id)
    }

    /// Convert to Value for storage
    pub fn to_value(&self) -> Value {
        let json = serde_json::to_value(self).unwrap_or(serde_json::Value::Null);
        Value::from_json(json)
    }

    /// Convert from Value
    pub fn from_value(value: &Value) -> Option<Self> {
        let json = value.to_json();
        serde_json::from_value(json).ok()
    }
}

/// A single node in the dialogue tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    /// Unique identifier for this node
    pub id: String,
    /// Type of this node
    #[serde(default)]
    pub node_type: DialogueNodeType,
    /// Speaker name (for text nodes)
    #[serde(default)]
    pub speaker: String,
    /// The text content of this node
    #[serde(default)]
    pub text: String,
    /// Choices available to the player (for choice nodes)
    #[serde(default)]
    pub choices: Vec<DialogueChoice>,
    /// Next node to go to (for linear flow)
    pub next_node: Option<String>,
    /// Condition to check before showing this node
    pub condition: Option<String>,
    /// Action to execute when entering this node
    pub action: Option<String>,
    /// Position in the editor (x, y)
    #[serde(default = "default_position")]
    pub position: (f32, f32),
}

fn default_position() -> (f32, f32) {
    (0.0, 0.0)
}

impl Default for DialogueNode {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            node_type: DialogueNodeType::Text,
            speaker: String::new(),
            text: String::new(),
            choices: Vec::new(),
            next_node: None,
            condition: None,
            action: None,
            position: (0.0, 0.0),
        }
    }
}

impl DialogueNode {
    /// Create a new text node
    pub fn new_text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    /// Create a new choice node
    pub fn new_choice() -> Self {
        Self {
            node_type: DialogueNodeType::Choice,
            ..Default::default()
        }
    }
}

/// Type of dialogue node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DialogueNodeType {
    /// NPC speaks text, then continues to next node
    #[default]
    Text,
    /// Player chooses from multiple options
    Choice,
    /// Check a condition and branch
    Condition,
    /// Execute an action (give item, start quest, etc.)
    Action,
    /// End of dialogue
    End,
}

impl DialogueNodeType {
    pub fn display_name(&self) -> &'static str {
        match self {
            DialogueNodeType::Text => "Text",
            DialogueNodeType::Choice => "Choice",
            DialogueNodeType::Condition => "Condition",
            DialogueNodeType::Action => "Action",
            DialogueNodeType::End => "End",
        }
    }

    pub fn all() -> &'static [DialogueNodeType] {
        &[
            DialogueNodeType::Text,
            DialogueNodeType::Choice,
            DialogueNodeType::Condition,
            DialogueNodeType::Action,
            DialogueNodeType::End,
        ]
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            DialogueNodeType::Text => (100, 149, 237),     // Cornflower blue
            DialogueNodeType::Choice => (255, 165, 0),    // Orange
            DialogueNodeType::Condition => (147, 112, 219), // Medium purple
            DialogueNodeType::Action => (50, 205, 50),    // Lime green
            DialogueNodeType::End => (220, 20, 60),       // Crimson
        }
    }
}

/// A player choice option in a dialogue
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DialogueChoice {
    /// Display text for this choice
    pub text: String,
    /// Node to go to when this choice is selected
    pub next_node: Option<String>,
    /// Condition required to show this choice
    pub condition: Option<String>,
}
