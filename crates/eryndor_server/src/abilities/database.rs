use bevy::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use eryndor_shared::AbilityDefinition;
use super::definitions::*;

/// Load all abilities from individual JSON files in content/abilities/
pub fn load_abilities_from_content() -> HashMap<u32, AbilityDefinition> {
    let content_path = Path::new("assets/content/abilities");
    let mut abilities = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(content_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    match serde_json::from_str::<AbilityDefinition>(&content) {
                        Ok(ability) => {
                            info!("Loaded ability: {} (id: {})", ability.name, ability.id);
                            abilities.insert(ability.id, ability);
                        }
                        Err(e) => {
                            warn!("Failed to parse ability file {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
    } else {
        warn!("Abilities content directory not found: {:?}", content_path);
    }

    abilities
}

/// Central database of all ability definitions
#[derive(Resource)]
pub struct AbilityDatabase {
    pub abilities: HashMap<u32, AbilityDefinition>,
}

impl Default for AbilityDatabase {
    fn default() -> Self {
        // First try to load from JSON files
        let mut abilities = load_abilities_from_content();

        // If no JSON files found, fall back to hardcoded definitions
        if abilities.is_empty() {
            warn!("No ability JSON files found, using hardcoded defaults");

            // Load Knight abilities
            for ability in create_knight_abilities() {
                abilities.insert(ability.id, ability);
            }

            // Load Mage abilities
            for ability in create_mage_abilities() {
                abilities.insert(ability.id, ability);
            }

            // Load Rogue abilities
            for ability in create_rogue_abilities() {
                abilities.insert(ability.id, ability);
            }

            // Load Wizard abilities
            for ability in create_wizard_abilities() {
                abilities.insert(ability.id, ability);
            }

            // Load Cleric abilities
            for ability in create_cleric_abilities() {
                abilities.insert(ability.id, ability);
            }

            // Load Ranger abilities
            for ability in create_ranger_abilities() {
                abilities.insert(ability.id, ability);
            }

            // Load Berserker abilities
            for ability in create_berserker_abilities() {
                abilities.insert(ability.id, ability);
            }
        }

        info!("AbilityDatabase initialized with {} abilities", abilities.len());

        Self { abilities }
    }
}

impl AbilityDatabase {
    /// Get an ability definition by ID
    pub fn get(&self, id: u32) -> Option<&AbilityDefinition> {
        self.abilities.get(&id)
    }

    /// Get all abilities
    pub fn all(&self) -> impl Iterator<Item = &AbilityDefinition> {
        self.abilities.values()
    }

    /// Get abilities that a character can unlock at a given level
    pub fn unlockable_at_level(&self, level: u32) -> Vec<&AbilityDefinition> {
        self.abilities
            .values()
            .filter(|ability| {
                matches!(
                    ability.unlock_requirement,
                    eryndor_shared::AbilityUnlockRequirement::Level(req_level) if req_level == level
                )
            })
            .collect()
    }

    /// Reload all abilities from JSON files
    pub fn reload(&mut self) {
        let new_abilities = load_abilities_from_content();
        if !new_abilities.is_empty() {
            self.abilities = new_abilities;
            info!("AbilityDatabase reloaded with {} abilities", self.abilities.len());
        } else {
            warn!("No ability JSON files found during reload, keeping existing abilities");
        }
    }

    /// Insert or update an ability
    pub fn upsert(&mut self, ability: AbilityDefinition) {
        self.abilities.insert(ability.id, ability);
    }

    /// Remove an ability by ID
    pub fn remove(&mut self, id: u32) -> Option<AbilityDefinition> {
        self.abilities.remove(&id)
    }
}
