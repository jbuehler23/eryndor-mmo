use serde::{Deserialize, Serialize};

/// Stat bonuses that can be applied by buffs
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StatBonuses {
    pub attack_power: f32,
    pub defense: f32,
    pub move_speed: f32,
}

/// Types of weapons for unlock requirements and trainer specialization
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WeaponType {
    Sword,
    Dagger,
    Staff,
    Wand,
    Mace,
    Bow,
    Axe,
}

/// Types of armor for trainer specialization
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ArmorType {
    Cloth,
    Leather,
    Chain,
    Plate,
}

/// Represents different types of ability effects that can be applied
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AbilityType {
    /// Direct damage dealt immediately
    DirectDamage {
        multiplier: f32
    },
    /// Damage dealt over time
    DamageOverTime {
        duration: f32,
        ticks: u32,
        damage_per_tick: f32,
    },
    /// Area of effect that hits multiple targets
    AreaOfEffect {
        radius: f32,
        max_targets: u32,
    },
    /// Temporary stat increase
    Buff {
        duration: f32,
        stat_bonuses: StatBonuses,
    },
    /// Temporary negative effect on target
    Debuff {
        duration: f32,
        effect: DebuffType,
    },
    /// Movement ability (dash, blink, charge)
    Mobility {
        distance: f32,
        dash_speed: f32,
    },
    /// Restore health
    Heal {
        amount: f32,
        is_percent: bool,  // If true, amount is % of max HP
    },
}

/// Types of debuffs that can be applied
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DebuffType {
    /// Reduce movement speed
    Slow {
        move_speed_reduction: f32  // Percent reduction (0.0 - 1.0)
    },
    /// Reduce attack power
    Weaken {
        attack_reduction: f32  // Percent reduction (0.0 - 1.0)
    },
    /// Cannot move or act
    Stun,
    /// Cannot move but can still attack
    Root,
}

/// Requirements for unlocking an ability
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AbilityUnlockRequirement {
    /// Available from the start
    None,
    /// Requires reaching a certain character level
    Level(u32),
    /// Requires completing a specific quest
    Quest(u32),
    /// Requires weapon proficiency level
    WeaponProficiency {
        weapon: WeaponType,
        level: u32,
    },
}

/// Complete definition of an ability
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AbilityDefinition {
    pub id: u32,
    pub name: String,
    pub description: String,
    /// Base damage multiplier (can be 0.0 for support abilities)
    pub damage_multiplier: f32,
    /// Cooldown in seconds
    pub cooldown: f32,
    /// Maximum range in units
    pub range: f32,
    /// Mana cost to use
    pub mana_cost: f32,
    /// List of effects this ability applies (can have multiple)
    pub ability_types: Vec<AbilityType>,
    /// Requirements to learn/use this ability
    pub unlock_requirement: AbilityUnlockRequirement,
}
