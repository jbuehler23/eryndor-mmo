use super::Schema;
use std::path::Path;

/// Load a schema from a JSON file
pub fn load_schema(path: &Path) -> Result<Schema, SchemaError> {
    let content = std::fs::read_to_string(path).map_err(|e| SchemaError::IoError(e.to_string()))?;

    let schema: Schema =
        serde_json::from_str(&content).map_err(|e| SchemaError::ParseError(e.to_string()))?;

    validate_schema(&schema)?;

    Ok(schema)
}

/// Save a schema to a JSON file
pub fn save_schema(schema: &Schema, path: &Path) -> Result<(), SchemaError> {
    let content = serde_json::to_string_pretty(schema)
        .map_err(|e| SchemaError::ParseError(e.to_string()))?;

    std::fs::write(path, content).map_err(|e| SchemaError::IoError(e.to_string()))?;

    Ok(())
}

/// Load a schema from a JSON string
pub fn parse_schema(json: &str) -> Result<Schema, SchemaError> {
    let schema: Schema =
        serde_json::from_str(json).map_err(|e| SchemaError::ParseError(e.to_string()))?;

    validate_schema(&schema)?;

    Ok(schema)
}

/// Validate that the schema is internally consistent
fn validate_schema(schema: &Schema) -> Result<(), SchemaError> {
    // Check that all enum references point to valid enums
    for (type_name, type_def) in schema
        .data_types
        .iter()
        .chain(schema.embedded_types.iter())
    {
        for prop in &type_def.properties {
            if let Some(enum_type) = &prop.enum_type {
                if !schema.enums.contains_key(enum_type) {
                    return Err(SchemaError::ValidationError(format!(
                        "Type '{}' property '{}' references unknown enum '{}'",
                        type_name, prop.name, enum_type
                    )));
                }
            }

            if let Some(ref_type) = &prop.ref_type {
                if !schema.data_types.contains_key(ref_type) {
                    return Err(SchemaError::ValidationError(format!(
                        "Type '{}' property '{}' references unknown type '{}'",
                        type_name, prop.name, ref_type
                    )));
                }
            }

            if let Some(embedded_type) = &prop.embedded_type {
                if !schema.embedded_types.contains_key(embedded_type) {
                    return Err(SchemaError::ValidationError(format!(
                        "Type '{}' property '{}' references unknown embedded type '{}'",
                        type_name, prop.name, embedded_type
                    )));
                }
            }
        }
    }

    Ok(())
}

/// Create a default schema with Eryndor game types
pub fn default_schema() -> Schema {
    let json = r##"
{
  "version": 1,
  "project": {
    "name": "Eryndor Project",
    "tile_size": 32,
    "default_layer_types": ["Ground", "Collision", "Objects"]
  },
  "enums": {
    "ItemType": ["Weapon", "Armor", "Consumable", "Quest", "Material"],
    "Rarity": ["Common", "Uncommon", "Rare", "Epic", "Legendary"],
    "DamageType": ["Physical", "Fire", "Ice", "Lightning", "Poison", "Holy", "Dark"],
    "TargetType": ["Self", "SingleEnemy", "SingleAlly", "AllEnemies", "AllAllies", "Area"],
    "AbilityType": ["Attack", "Heal", "Buff", "Debuff", "Utility"],
    "QuestType": ["Main", "Side", "Daily", "Repeatable"],
    "NpcRole": ["Vendor", "QuestGiver", "Trainer", "Guard", "Innkeeper"]
  },
  "data_types": {
    "Item": {
      "color": "#4CAF50",
      "placeable": false,
      "properties": [
        { "name": "name", "type": "string", "required": true },
        { "name": "description", "type": "multiline" },
        { "name": "itemType", "type": "enum", "enumType": "ItemType", "required": true },
        { "name": "rarity", "type": "enum", "enumType": "Rarity" },
        { "name": "value", "type": "int", "min": 0 },
        { "name": "stackable", "type": "bool" },
        { "name": "maxStack", "type": "int", "min": 1, "showIf": "stackable == true" }
      ]
    },
    "Ability": {
      "color": "#2196F3",
      "placeable": false,
      "properties": [
        { "name": "name", "type": "string", "required": true },
        { "name": "description", "type": "multiline" },
        { "name": "abilityType", "type": "enum", "enumType": "AbilityType", "required": true },
        { "name": "targetType", "type": "enum", "enumType": "TargetType", "required": true },
        { "name": "damageType", "type": "enum", "enumType": "DamageType" },
        { "name": "baseDamage", "type": "int", "min": 0 },
        { "name": "manaCost", "type": "int", "min": 0 },
        { "name": "cooldown", "type": "float", "min": 0 },
        { "name": "range", "type": "float", "min": 0 },
        { "name": "aoeRadius", "type": "float", "min": 0, "showIf": "targetType == Area" }
      ]
    },
    "Quest": {
      "color": "#FF9800",
      "placeable": false,
      "properties": [
        { "name": "name", "type": "string", "required": true },
        { "name": "description", "type": "multiline" },
        { "name": "questType", "type": "enum", "enumType": "QuestType", "required": true },
        { "name": "level", "type": "int", "min": 1 },
        { "name": "xpReward", "type": "int", "min": 0 },
        { "name": "goldReward", "type": "int", "min": 0 },
        { "name": "prerequisiteQuest", "type": "ref", "refType": "Quest" }
      ]
    },
    "Enemy": {
      "color": "#F44336",
      "placeable": false,
      "properties": [
        { "name": "name", "type": "string", "required": true },
        { "name": "level", "type": "int", "min": 1, "required": true },
        { "name": "health", "type": "int", "min": 1 },
        { "name": "damage", "type": "int", "min": 0 },
        { "name": "xpReward", "type": "int", "min": 0 },
        { "name": "lootTable", "type": "array", "itemType": "ref" }
      ]
    },
    "NPC": {
      "color": "#9C27B0",
      "placeable": true,
      "properties": [
        { "name": "name", "type": "string", "required": true },
        { "name": "role", "type": "enum", "enumType": "NpcRole", "required": true },
        { "name": "dialogue", "type": "dialogue" },
        { "name": "questsOffered", "type": "array", "itemType": "ref", "refType": "Quest" }
      ]
    },
    "SpawnPoint": {
      "color": "#E91E63",
      "placeable": true,
      "properties": [
        { "name": "enemy", "type": "ref", "refType": "Enemy", "required": true },
        { "name": "respawnTime", "type": "float", "min": 0 },
        { "name": "maxCount", "type": "int", "min": 1 }
      ]
    },
    "Trigger": {
      "color": "#00BCD4",
      "placeable": true,
      "properties": [
        { "name": "name", "type": "string" },
        { "name": "width", "type": "int", "min": 1 },
        { "name": "height", "type": "int", "min": 1 },
        { "name": "onEnter", "type": "string" },
        { "name": "onExit", "type": "string" }
      ]
    }
  },
  "embedded_types": {}
}
"##;
    parse_schema(json).expect("Default schema should be valid")
}

#[derive(Debug)]
pub enum SchemaError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaError::IoError(e) => write!(f, "IO error: {}", e),
            SchemaError::ParseError(e) => write!(f, "Parse error: {}", e),
            SchemaError::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for SchemaError {}
