use bevy::prelude::*;
use std::collections::HashMap;
use eryndor_shared::AbilityDefinition;
use super::definitions::*;

/// Central database of all ability definitions
#[derive(Resource)]
pub struct AbilityDatabase {
    abilities: HashMap<u32, AbilityDefinition>,
}

impl Default for AbilityDatabase {
    fn default() -> Self {
        let mut abilities = HashMap::new();

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
}
