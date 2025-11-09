use bevy::prelude::*;
use std::collections::HashMap;

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

impl Default for ClientAbilityDatabase {
    fn default() -> Self {
        let mut abilities = HashMap::new();

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
