use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::ability_types::{DebuffType, StatBonuses};

// ============================================================================
// EFFECT TRACKING COMPONENTS
// ============================================================================

/// Tracks active buffs on an entity
#[derive(Component, Serialize, Deserialize, Clone, Default)]
pub struct ActiveBuffs {
    pub buffs: Vec<ActiveBuff>,
}

/// A single active buff with expiration time
#[derive(Serialize, Deserialize, Clone)]
pub struct ActiveBuff {
    pub ability_id: u32,
    pub stat_bonuses: StatBonuses,
    pub expires_at: f32,  // Game time in seconds
}

/// Tracks active debuffs on an entity
#[derive(Component, Serialize, Deserialize, Clone, Default)]
pub struct ActiveDebuffs {
    pub debuffs: Vec<ActiveDebuff>,
}

/// A single active debuff with expiration time
#[derive(Serialize, Deserialize, Clone)]
pub struct ActiveDebuff {
    pub ability_id: u32,
    pub effect: DebuffType,
    pub expires_at: f32,  // Game time in seconds
}

/// Tracks active damage-over-time effects
#[derive(Component, Serialize, Deserialize, Clone, Default)]
pub struct ActiveDoTs {
    pub dots: Vec<ActiveDoT>,
}

/// A single damage-over-time effect
#[derive(Serialize, Deserialize, Clone)]
pub struct ActiveDoT {
    pub ability_id: u32,
    pub caster: Entity,
    pub damage_per_tick: f32,
    pub ticks_remaining: u32,
    pub next_tick_at: f32,  // Game time in seconds
}
