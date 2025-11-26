use bevy::prelude::*;
use std::collections::HashMap;
use eryndor_shared::{AbilityDefinition, AbilityType, AbilityUnlockRequirement, DebuffType};

#[derive(Resource)]
pub struct ClientAbilityDatabase {
    pub abilities: HashMap<u32, ClientAbilityInfo>,
}

#[derive(Clone, Debug)]
pub struct ClientAbilityInfo {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub damage_multiplier: f32,
    pub cooldown: f32,
    pub range: f32,
    pub mana_cost: f32,
    pub effect_summary: String,
    pub unlock_level: Option<u32>,
}

impl ClientAbilityInfo {
    /// Create a ClientAbilityInfo from an AbilityDefinition
    pub fn from_definition(def: &AbilityDefinition) -> Self {
        let unlock_level = match &def.unlock_requirement {
            AbilityUnlockRequirement::Level(level) => Some(*level),
            AbilityUnlockRequirement::None => None,
            AbilityUnlockRequirement::Quest(_) => None,
            AbilityUnlockRequirement::WeaponProficiency { .. } => None,
        };

        Self {
            id: def.id,
            name: def.name.clone(),
            description: def.description.clone(),
            damage_multiplier: def.damage_multiplier,
            cooldown: def.cooldown,
            range: def.range,
            mana_cost: def.mana_cost,
            effect_summary: generate_effect_summary(&def.ability_types),
            unlock_level,
        }
    }
}

/// Generate a human-readable summary of ability effects
fn generate_effect_summary(ability_types: &[AbilityType]) -> String {
    let mut parts = Vec::new();

    for ability_type in ability_types {
        match ability_type {
            AbilityType::DirectDamage { multiplier } => {
                if *multiplier > 0.0 {
                    parts.push("Direct Damage".to_string());
                }
            }
            AbilityType::AreaOfEffect { radius, max_targets } => {
                parts.push(format!("AoE ({:.1} radius, {} targets)", radius, max_targets));
            }
            AbilityType::DamageOverTime { duration: _, ticks, damage_per_tick } => {
                parts.push(format!("DoT ({} ticks, {:.1} per tick)", ticks, damage_per_tick));
            }
            AbilityType::Heal { amount, is_percent } => {
                if *is_percent {
                    parts.push(format!("Heal ({:.0}% max HP)", amount * 100.0));
                } else {
                    parts.push(format!("Heal ({:.0} HP)", amount));
                }
            }
            AbilityType::Buff { duration, stat_bonuses } => {
                let mut buff_parts = Vec::new();
                if stat_bonuses.attack_power > 0.0 {
                    buff_parts.push(format!("+{:.1} Attack", stat_bonuses.attack_power));
                }
                if stat_bonuses.defense > 0.0 {
                    buff_parts.push(format!("+{:.1} Defense", stat_bonuses.defense));
                }
                if stat_bonuses.move_speed > 0.0 {
                    buff_parts.push(format!("+{:.0}% Speed", stat_bonuses.move_speed * 100.0));
                }
                if !buff_parts.is_empty() {
                    parts.push(format!("Buff ({:.1}s, {})", duration, buff_parts.join(", ")));
                } else {
                    parts.push(format!("Buff ({:.1}s)", duration));
                }
            }
            AbilityType::Debuff { duration, effect } => {
                match effect {
                    DebuffType::Slow { move_speed_reduction } => {
                        parts.push(format!("Slow ({:.1}s, -{:.0}%)", duration, move_speed_reduction * 100.0));
                    }
                    DebuffType::Weaken { attack_reduction } => {
                        parts.push(format!("Weaken ({:.1}s, -{:.0}% attack)", duration, attack_reduction * 100.0));
                    }
                    DebuffType::Stun => {
                        parts.push(format!("Stun ({:.1}s)", duration));
                    }
                    DebuffType::Root => {
                        parts.push(format!("Root ({:.1}s)", duration));
                    }
                }
            }
            AbilityType::Mobility { distance, .. } => {
                parts.push(format!("Dash ({:.1} distance)", distance));
            }
        }
    }

    if parts.is_empty() {
        "No special effects".to_string()
    } else {
        parts.join(" + ")
    }
}

/// Load all abilities from individual JSON files in assets/content/abilities/
/// Works on native builds only - WASM uses fallback hardcoded abilities
#[cfg(not(target_family = "wasm"))]
fn load_abilities_from_content() -> HashMap<u32, ClientAbilityInfo> {
    use std::path::Path;

    let content_path = Path::new("assets/content/abilities");
    let mut abilities = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(content_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    match serde_json::from_str::<AbilityDefinition>(&content) {
                        Ok(ability) => {
                            info!("Client loaded ability: {} (id: {})", ability.name, ability.id);
                            abilities.insert(ability.id, ClientAbilityInfo::from_definition(&ability));
                        }
                        Err(e) => {
                            warn!("Client failed to parse ability file {:?}: {}", path, e);
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

#[cfg(target_family = "wasm")]
fn load_abilities_from_content() -> HashMap<u32, ClientAbilityInfo> {
    // WASM cannot directly read from filesystem - return empty and use fallback
    HashMap::new()
}

impl Default for ClientAbilityDatabase {
    fn default() -> Self {
        // First try to load from JSON files (works on native builds)
        let mut abilities = load_abilities_from_content();

        // If JSON loading found abilities, we're done
        if !abilities.is_empty() {
            info!("ClientAbilityDatabase initialized from JSON with {} abilities", abilities.len());
            return Self { abilities };
        }

        // Fall back to hardcoded abilities for WASM or if JSON files not found
        info!("Using hardcoded ability definitions");

        // ============================================================================
        // KNIGHT ABILITIES (100-199)
        // ============================================================================

        abilities.insert(100, ClientAbilityInfo {
            id: 100,
            name: "Heavy Slash".to_string(),
            description: "A powerful melee attack that deals heavy damage.".to_string(),
            damage_multiplier: 2.0,
            cooldown: 3.0,
            range: 1.5,
            mana_cost: 15.0,
            effect_summary: "Direct Damage".to_string(),
            unlock_level: None,
        });

        abilities.insert(101, ClientAbilityInfo {
            id: 101,
            name: "Shield Bash".to_string(),
            description: "Bash the enemy with your shield, dealing damage and stunning them.".to_string(),
            damage_multiplier: 1.2,
            cooldown: 8.0,
            range: 1.5,
            mana_cost: 20.0,
            effect_summary: "Damage + Stun (2s)".to_string(),
            unlock_level: Some(3),
        });

        abilities.insert(102, ClientAbilityInfo {
            id: 102,
            name: "Taunt".to_string(),
            description: "Taunt nearby enemies, reducing their attack power.".to_string(),
            damage_multiplier: 0.0,
            cooldown: 12.0,
            range: 5.0,
            mana_cost: 25.0,
            effect_summary: "AoE (5.0 radius) + Weaken (6s, -30% attack)".to_string(),
            unlock_level: Some(5),
        });

        abilities.insert(103, ClientAbilityInfo {
            id: 103,
            name: "Cleave".to_string(),
            description: "Swing your weapon in a wide arc, hitting multiple enemies.".to_string(),
            damage_multiplier: 1.5,
            cooldown: 6.0,
            range: 2.0,
            mana_cost: 30.0,
            effect_summary: "Damage + AoE (2.0 radius, 3 targets)".to_string(),
            unlock_level: Some(7),
        });

        abilities.insert(104, ClientAbilityInfo {
            id: 104,
            name: "Second Wind".to_string(),
            description: "Recover health and gain a defensive boost.".to_string(),
            damage_multiplier: 0.0,
            cooldown: 30.0,
            range: 0.0,
            mana_cost: 40.0,
            effect_summary: "Heal (30% max HP) + Defense Buff (+5.0, 10s)".to_string(),
            unlock_level: Some(10),
        });

        abilities.insert(105, ClientAbilityInfo {
            id: 105,
            name: "Charge".to_string(),
            description: "Dash forward, dealing damage to the first enemy hit.".to_string(),
            damage_multiplier: 1.8,
            cooldown: 10.0,
            range: 8.0,
            mana_cost: 25.0,
            effect_summary: "Dash (8.0 distance) + Damage".to_string(),
            unlock_level: Some(12),
        });

        // ============================================================================
        // MAGE ABILITIES (200-299)
        // ============================================================================

        abilities.insert(200, ClientAbilityInfo {
            id: 200,
            name: "Fireball".to_string(),
            description: "Launch a ball of fire at your enemy.".to_string(),
            damage_multiplier: 1.5,
            cooldown: 2.5,
            range: 15.0,
            mana_cost: 25.0,
            effect_summary: "Direct Damage".to_string(),
            unlock_level: None,
        });

        abilities.insert(201, ClientAbilityInfo {
            id: 201,
            name: "Frost Bolt".to_string(),
            description: "Fire a bolt of ice that damages and slows the target.".to_string(),
            damage_multiplier: 1.3,
            cooldown: 3.0,
            range: 15.0,
            mana_cost: 20.0,
            effect_summary: "Damage + Slow (4s, -50% move speed)".to_string(),
            unlock_level: Some(3),
        });

        abilities.insert(202, ClientAbilityInfo {
            id: 202,
            name: "Ignite".to_string(),
            description: "Set the enemy ablaze, dealing damage over time.".to_string(),
            damage_multiplier: 0.5,
            cooldown: 8.0,
            range: 15.0,
            mana_cost: 30.0,
            effect_summary: "Damage + DoT (8 ticks, 8.0 per tick)".to_string(),
            unlock_level: Some(5),
        });

        abilities.insert(203, ClientAbilityInfo {
            id: 203,
            name: "Arcane Explosion".to_string(),
            description: "Release a burst of arcane energy, damaging all nearby enemies.".to_string(),
            damage_multiplier: 1.2,
            cooldown: 10.0,
            range: 5.0,
            mana_cost: 45.0,
            effect_summary: "Damage + AoE (5.0 radius, 8 targets)".to_string(),
            unlock_level: Some(7),
        });

        abilities.insert(204, ClientAbilityInfo {
            id: 204,
            name: "Mana Shield".to_string(),
            description: "Surround yourself with a magical barrier, increasing defense.".to_string(),
            damage_multiplier: 0.0,
            cooldown: 25.0,
            range: 0.0,
            mana_cost: 50.0,
            effect_summary: "Defense Buff (+8.0, 15s)".to_string(),
            unlock_level: Some(10),
        });

        abilities.insert(205, ClientAbilityInfo {
            id: 205,
            name: "Blink".to_string(),
            description: "Teleport a short distance instantly.".to_string(),
            damage_multiplier: 0.0,
            cooldown: 15.0,
            range: 10.0,
            mana_cost: 35.0,
            effect_summary: "Instant Teleport (10.0 distance)".to_string(),
            unlock_level: Some(12),
        });

        // ============================================================================
        // ROGUE ABILITIES (300-399)
        // ============================================================================

        abilities.insert(300, ClientAbilityInfo {
            id: 300,
            name: "Quick Strike".to_string(),
            description: "A fast melee attack.".to_string(),
            damage_multiplier: 1.0,
            cooldown: 1.0,
            range: 1.5,
            mana_cost: 10.0,
            effect_summary: "Direct Damage".to_string(),
            unlock_level: None,
        });

        abilities.insert(301, ClientAbilityInfo {
            id: 301,
            name: "Backstab".to_string(),
            description: "Strike from behind for massive damage.".to_string(),
            damage_multiplier: 2.5,
            cooldown: 5.0,
            range: 1.5,
            mana_cost: 20.0,
            effect_summary: "Direct Damage".to_string(),
            unlock_level: Some(3),
        });

        abilities.insert(302, ClientAbilityInfo {
            id: 302,
            name: "Poison Blade".to_string(),
            description: "Poison your weapon, dealing immediate and ongoing damage.".to_string(),
            damage_multiplier: 1.0,
            cooldown: 8.0,
            range: 1.5,
            mana_cost: 25.0,
            effect_summary: "Damage + DoT (8 ticks, 5.0 per tick)".to_string(),
            unlock_level: Some(5),
        });

        abilities.insert(303, ClientAbilityInfo {
            id: 303,
            name: "Shadow Step".to_string(),
            description: "Dash through shadows, gaining increased critical strike chance.".to_string(),
            damage_multiplier: 0.0,
            cooldown: 12.0,
            range: 8.0,
            mana_cost: 30.0,
            effect_summary: "Dash (8.0 distance) + Attack Buff (+3.0, 5s)".to_string(),
            unlock_level: Some(7),
        });

        abilities.insert(304, ClientAbilityInfo {
            id: 304,
            name: "Smoke Bomb".to_string(),
            description: "Throw a smoke bomb, rooting enemies in place.".to_string(),
            damage_multiplier: 0.5,
            cooldown: 20.0,
            range: 10.0,
            mana_cost: 35.0,
            effect_summary: "Damage + AoE (4.0 radius, 5 targets) + Root (3s)".to_string(),
            unlock_level: Some(10),
        });

        abilities.insert(305, ClientAbilityInfo {
            id: 305,
            name: "Eviscerate".to_string(),
            description: "A devastating finishing move that deals massive damage.".to_string(),
            damage_multiplier: 3.0,
            cooldown: 15.0,
            range: 1.5,
            mana_cost: 40.0,
            effect_summary: "Direct Damage".to_string(),
            unlock_level: Some(12),
        });

        info!("ClientAbilityDatabase initialized with {} hardcoded abilities", abilities.len());
        ClientAbilityDatabase { abilities }
    }
}

impl ClientAbilityDatabase {
    pub fn get_ability_info(&self, ability_id: u32) -> Option<&ClientAbilityInfo> {
        self.abilities.get(&ability_id)
    }

    pub fn get_ability_name(&self, ability_id: u32) -> String {
        self.abilities
            .get(&ability_id)
            .map(|a| a.name.clone())
            .unwrap_or_else(|| format!("Unknown ({})", ability_id))
    }
}
