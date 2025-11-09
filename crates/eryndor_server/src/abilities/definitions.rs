use eryndor_shared::*;

// ============================================================================
// ABILITY ID CONSTANTS
// ============================================================================

// Knight abilities (IDs 100-199)
pub const ABILITY_HEAVY_SLASH: u32 = 100;
pub const ABILITY_SHIELD_BASH: u32 = 101;
pub const ABILITY_TAUNT: u32 = 102;
pub const ABILITY_CLEAVE: u32 = 103;
pub const ABILITY_SECOND_WIND: u32 = 104;
pub const ABILITY_CHARGE: u32 = 105;

// Mage abilities (IDs 200-299)
pub const ABILITY_FIREBALL: u32 = 200;
pub const ABILITY_FROST_BOLT: u32 = 201;
pub const ABILITY_FLAME_DOT: u32 = 202;
pub const ABILITY_ARCANE_EXPLOSION: u32 = 203;
pub const ABILITY_MANA_SHIELD: u32 = 204;
pub const ABILITY_BLINK: u32 = 205;

// Rogue abilities (IDs 300-399)
pub const ABILITY_QUICK_STRIKE: u32 = 300;
pub const ABILITY_BACKSTAB: u32 = 301;
pub const ABILITY_POISON_BLADE: u32 = 302;
pub const ABILITY_SHADOW_STEP: u32 = 303;
pub const ABILITY_SMOKE_BOMB: u32 = 304;
pub const ABILITY_EVISCERATE: u32 = 305;

// ============================================================================
// KNIGHT ABILITIES
// ============================================================================

pub fn create_knight_abilities() -> Vec<AbilityDefinition> {
    vec![
        // Heavy Slash - Starting ability
        AbilityDefinition {
            id: ABILITY_HEAVY_SLASH,
            name: "Heavy Slash".to_string(),
            description: "A powerful melee attack that deals heavy damage.".to_string(),
            damage_multiplier: 2.0,
            cooldown: 3.0,
            range: 1.5,
            mana_cost: 15.0,
            ability_types: vec![
                AbilityType::DirectDamage { multiplier: 2.0 },
            ],
            unlock_requirement: AbilityUnlockRequirement::None,
        },

        // Shield Bash - Level 3
        AbilityDefinition {
            id: ABILITY_SHIELD_BASH,
            name: "Shield Bash".to_string(),
            description: "Bash the enemy with your shield, dealing damage and stunning them.".to_string(),
            damage_multiplier: 1.2,
            cooldown: 8.0,
            range: 1.5,
            mana_cost: 20.0,
            ability_types: vec![
                AbilityType::DirectDamage { multiplier: 1.2 },
                AbilityType::Debuff {
                    duration: 2.0,
                    effect: DebuffType::Stun,
                },
            ],
            unlock_requirement: AbilityUnlockRequirement::Level(3),
        },

        // Taunt - Level 5
        AbilityDefinition {
            id: ABILITY_TAUNT,
            name: "Taunt".to_string(),
            description: "Taunt nearby enemies, reducing their attack power.".to_string(),
            damage_multiplier: 0.0,
            cooldown: 12.0,
            range: 5.0,
            mana_cost: 25.0,
            ability_types: vec![
                AbilityType::AreaOfEffect {
                    radius: 5.0,
                    max_targets: 5,
                },
                AbilityType::Debuff {
                    duration: 6.0,
                    effect: DebuffType::Weaken { attack_reduction: 0.3 },
                },
            ],
            unlock_requirement: AbilityUnlockRequirement::Level(5),
        },

        // Cleave - Level 7
        AbilityDefinition {
            id: ABILITY_CLEAVE,
            name: "Cleave".to_string(),
            description: "Swing your weapon in a wide arc, hitting multiple enemies.".to_string(),
            damage_multiplier: 1.5,
            cooldown: 6.0,
            range: 2.0,
            mana_cost: 30.0,
            ability_types: vec![
                AbilityType::DirectDamage { multiplier: 1.5 },
                AbilityType::AreaOfEffect {
                    radius: 2.0,
                    max_targets: 3,
                },
            ],
            unlock_requirement: AbilityUnlockRequirement::Level(7),
        },

        // Second Wind - Level 10
        AbilityDefinition {
            id: ABILITY_SECOND_WIND,
            name: "Second Wind".to_string(),
            description: "Recover health and gain a defensive boost.".to_string(),
            damage_multiplier: 0.0,
            cooldown: 30.0,
            range: 0.0,
            mana_cost: 40.0,
            ability_types: vec![
                AbilityType::Heal {
                    amount: 0.3,
                    is_percent: true,
                },
                AbilityType::Buff {
                    duration: 10.0,
                    stat_bonuses: StatBonuses {
                        attack_power: 0.0,
                        defense: 5.0,
                        move_speed: 0.0,
                    },
                },
            ],
            unlock_requirement: AbilityUnlockRequirement::Level(10),
        },

        // Charge - Level 12
        AbilityDefinition {
            id: ABILITY_CHARGE,
            name: "Charge".to_string(),
            description: "Dash forward, dealing damage to the first enemy hit.".to_string(),
            damage_multiplier: 1.8,
            cooldown: 10.0,
            range: 8.0,
            mana_cost: 25.0,
            ability_types: vec![
                AbilityType::Mobility {
                    distance: 8.0,
                    dash_speed: 20.0,
                },
                AbilityType::DirectDamage { multiplier: 1.8 },
            ],
            unlock_requirement: AbilityUnlockRequirement::Level(12),
        },
    ]
}

// ============================================================================
// MAGE ABILITIES
// ============================================================================

pub fn create_mage_abilities() -> Vec<AbilityDefinition> {
    vec![
        // Fireball - Starting ability
        AbilityDefinition {
            id: ABILITY_FIREBALL,
            name: "Fireball".to_string(),
            description: "Launch a ball of fire at your enemy.".to_string(),
            damage_multiplier: 1.5,
            cooldown: 2.5,
            range: 15.0,
            mana_cost: 25.0,
            ability_types: vec![
                AbilityType::DirectDamage { multiplier: 1.5 },
            ],
            unlock_requirement: AbilityUnlockRequirement::None,
        },

        // Frost Bolt - Level 3
        AbilityDefinition {
            id: ABILITY_FROST_BOLT,
            name: "Frost Bolt".to_string(),
            description: "Fire a bolt of ice that damages and slows the target.".to_string(),
            damage_multiplier: 1.3,
            cooldown: 3.0,
            range: 15.0,
            mana_cost: 20.0,
            ability_types: vec![
                AbilityType::DirectDamage { multiplier: 1.3 },
                AbilityType::Debuff {
                    duration: 4.0,
                    effect: DebuffType::Slow { move_speed_reduction: 0.5 },
                },
            ],
            unlock_requirement: AbilityUnlockRequirement::Level(3),
        },

        // Flame DoT - Level 5
        AbilityDefinition {
            id: ABILITY_FLAME_DOT,
            name: "Ignite".to_string(),
            description: "Set the enemy ablaze, dealing damage over time.".to_string(),
            damage_multiplier: 0.5,
            cooldown: 8.0,
            range: 15.0,
            mana_cost: 30.0,
            ability_types: vec![
                AbilityType::DirectDamage { multiplier: 0.5 },
                AbilityType::DamageOverTime {
                    duration: 6.0,
                    ticks: 6,
                    damage_per_tick: 8.0,
                },
            ],
            unlock_requirement: AbilityUnlockRequirement::Level(5),
        },

        // Arcane Explosion - Level 7
        AbilityDefinition {
            id: ABILITY_ARCANE_EXPLOSION,
            name: "Arcane Explosion".to_string(),
            description: "Release a burst of arcane energy, damaging all nearby enemies.".to_string(),
            damage_multiplier: 1.2,
            cooldown: 10.0,
            range: 5.0,
            mana_cost: 45.0,
            ability_types: vec![
                AbilityType::DirectDamage { multiplier: 1.2 },
                AbilityType::AreaOfEffect {
                    radius: 5.0,
                    max_targets: 8,
                },
            ],
            unlock_requirement: AbilityUnlockRequirement::Level(7),
        },

        // Mana Shield - Level 10
        AbilityDefinition {
            id: ABILITY_MANA_SHIELD,
            name: "Mana Shield".to_string(),
            description: "Surround yourself with a magical barrier, increasing defense.".to_string(),
            damage_multiplier: 0.0,
            cooldown: 25.0,
            range: 0.0,
            mana_cost: 50.0,
            ability_types: vec![
                AbilityType::Buff {
                    duration: 15.0,
                    stat_bonuses: StatBonuses {
                        attack_power: 0.0,
                        defense: 8.0,
                        move_speed: 0.0,
                    },
                },
            ],
            unlock_requirement: AbilityUnlockRequirement::Level(10),
        },

        // Blink - Level 12
        AbilityDefinition {
            id: ABILITY_BLINK,
            name: "Blink".to_string(),
            description: "Teleport a short distance instantly.".to_string(),
            damage_multiplier: 0.0,
            cooldown: 15.0,
            range: 10.0,
            mana_cost: 35.0,
            ability_types: vec![
                AbilityType::Mobility {
                    distance: 10.0,
                    dash_speed: 100.0,  // Instant teleport
                },
            ],
            unlock_requirement: AbilityUnlockRequirement::Level(12),
        },
    ]
}

// ============================================================================
// ROGUE ABILITIES
// ============================================================================

pub fn create_rogue_abilities() -> Vec<AbilityDefinition> {
    vec![
        // Quick Strike - Starting ability
        AbilityDefinition {
            id: ABILITY_QUICK_STRIKE,
            name: "Quick Strike".to_string(),
            description: "A fast melee attack.".to_string(),
            damage_multiplier: 1.0,
            cooldown: 1.0,
            range: 1.5,
            mana_cost: 10.0,
            ability_types: vec![
                AbilityType::DirectDamage { multiplier: 1.0 },
            ],
            unlock_requirement: AbilityUnlockRequirement::None,
        },

        // Backstab - Level 3
        AbilityDefinition {
            id: ABILITY_BACKSTAB,
            name: "Backstab".to_string(),
            description: "Strike from behind for massive damage.".to_string(),
            damage_multiplier: 2.5,
            cooldown: 5.0,
            range: 1.5,
            mana_cost: 20.0,
            ability_types: vec![
                AbilityType::DirectDamage { multiplier: 2.5 },
            ],
            unlock_requirement: AbilityUnlockRequirement::Level(3),
        },

        // Poison Blade - Level 5
        AbilityDefinition {
            id: ABILITY_POISON_BLADE,
            name: "Poison Blade".to_string(),
            description: "Poison your weapon, dealing immediate and ongoing damage.".to_string(),
            damage_multiplier: 1.0,
            cooldown: 8.0,
            range: 1.5,
            mana_cost: 25.0,
            ability_types: vec![
                AbilityType::DirectDamage { multiplier: 1.0 },
                AbilityType::DamageOverTime {
                    duration: 8.0,
                    ticks: 8,
                    damage_per_tick: 5.0,
                },
            ],
            unlock_requirement: AbilityUnlockRequirement::Level(5),
        },

        // Shadow Step - Level 7
        AbilityDefinition {
            id: ABILITY_SHADOW_STEP,
            name: "Shadow Step".to_string(),
            description: "Dash through shadows, gaining increased critical strike chance.".to_string(),
            damage_multiplier: 0.0,
            cooldown: 12.0,
            range: 8.0,
            mana_cost: 30.0,
            ability_types: vec![
                AbilityType::Mobility {
                    distance: 8.0,
                    dash_speed: 25.0,
                },
                AbilityType::Buff {
                    duration: 5.0,
                    stat_bonuses: StatBonuses {
                        attack_power: 3.0,
                        defense: 0.0,
                        move_speed: 0.0,
                    },
                },
            ],
            unlock_requirement: AbilityUnlockRequirement::Level(7),
        },

        // Smoke Bomb - Level 10
        AbilityDefinition {
            id: ABILITY_SMOKE_BOMB,
            name: "Smoke Bomb".to_string(),
            description: "Throw a smoke bomb, rooting enemies in place.".to_string(),
            damage_multiplier: 0.5,
            cooldown: 20.0,
            range: 10.0,
            mana_cost: 35.0,
            ability_types: vec![
                AbilityType::DirectDamage { multiplier: 0.5 },
                AbilityType::AreaOfEffect {
                    radius: 4.0,
                    max_targets: 5,
                },
                AbilityType::Debuff {
                    duration: 3.0,
                    effect: DebuffType::Root,
                },
            ],
            unlock_requirement: AbilityUnlockRequirement::Level(10),
        },

        // Eviscerate - Level 12
        AbilityDefinition {
            id: ABILITY_EVISCERATE,
            name: "Eviscerate".to_string(),
            description: "A devastating finishing move that deals massive damage.".to_string(),
            damage_multiplier: 3.0,
            cooldown: 15.0,
            range: 1.5,
            mana_cost: 40.0,
            ability_types: vec![
                AbilityType::DirectDamage { multiplier: 3.0 },
            ],
            unlock_requirement: AbilityUnlockRequirement::Level(12),
        },
    ]
}
