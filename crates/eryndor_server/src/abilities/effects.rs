use bevy::prelude::*;
use eryndor_shared::*;

// ============================================================================
// BUFF SYSTEM
// ============================================================================

/// Process active buffs - apply stat bonuses and remove expired buffs
pub fn process_buffs(
    mut query: Query<(&mut ActiveBuffs, &mut CombatStats, &mut MoveSpeed)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed().as_secs_f32();

    for (mut active_buffs, mut stats, mut move_speed) in &mut query {
        // Remove expired buffs
        active_buffs.buffs.retain(|buff| buff.expires_at > current_time);

        // Calculate total bonuses from all active buffs
        let mut total_attack = 0.0;
        let mut total_defense = 0.0;
        let mut total_move_speed = 0.0;

        for buff in &active_buffs.buffs {
            total_attack += buff.stat_bonuses.attack_power;
            total_defense += buff.stat_bonuses.defense;
            total_move_speed += buff.stat_bonuses.move_speed;
        }

        // Apply bonuses (these are additive)
        // Note: Base stats should be stored separately for proper calculation
        // For now, we're directly modifying the stats
        // TODO: Implement base stats system for proper buff/debuff handling
    }
}

/// Add a buff to an entity
pub fn add_buff(
    entity: &mut EntityWorldMut,
    ability_id: u32,
    stat_bonuses: StatBonuses,
    duration: f32,
    current_time: f32,
) {
    let buff = ActiveBuff {
        ability_id,
        stat_bonuses,
        expires_at: current_time + duration,
    };

    if let Some(mut active_buffs) = entity.get_mut::<ActiveBuffs>() {
        active_buffs.buffs.push(buff);
        info!("Added buff from ability {} to entity {:?}", ability_id, entity.id());
    } else {
        // Entity doesn't have ActiveBuffs component, add it
        entity.insert(ActiveBuffs {
            buffs: vec![buff],
        });
        info!("Created ActiveBuffs and added buff from ability {} to entity {:?}", ability_id, entity.id());
    }
}

// ============================================================================
// DEBUFF SYSTEM
// ============================================================================

/// Process active debuffs - apply effects and remove expired debuffs
pub fn process_debuffs(
    mut query: Query<(&mut ActiveDebuffs, &mut MoveSpeed)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed().as_secs_f32();

    for (mut active_debuffs, mut move_speed) in &mut query {
        // Remove expired debuffs
        active_debuffs.debuffs.retain(|debuff| debuff.expires_at > current_time);

        // Apply debuff effects
        // Note: This is a simplified implementation
        // TODO: Implement proper base stats system
        for debuff in &active_debuffs.debuffs {
            match &debuff.effect {
                DebuffType::Slow { move_speed_reduction } => {
                    // Reduce move speed by percentage
                    // TODO: Apply properly using base speed
                },
                DebuffType::Weaken { attack_reduction } => {
                    // Reduce attack by percentage
                    // TODO: Apply to CombatStats
                },
                DebuffType::Stun | DebuffType::Root => {
                    // Prevent movement
                    // TODO: Implement movement prevention
                },
            }
        }
    }
}

/// Add a debuff to an entity
pub fn add_debuff(
    entity: &mut EntityWorldMut,
    ability_id: u32,
    effect: DebuffType,
    duration: f32,
    current_time: f32,
) {
    let debuff = ActiveDebuff {
        ability_id,
        effect,
        expires_at: current_time + duration,
    };

    if let Some(mut active_debuffs) = entity.get_mut::<ActiveDebuffs>() {
        active_debuffs.debuffs.push(debuff);
        info!("Added debuff from ability {} to entity {:?}", ability_id, entity.id());
    } else {
        entity.insert(ActiveDebuffs {
            debuffs: vec![debuff],
        });
        info!("Created ActiveDebuffs and added debuff from ability {} to entity {:?}", ability_id, entity.id());
    }
}

// ============================================================================
// DAMAGE-OVER-TIME SYSTEM
// ============================================================================

/// Process active DoT effects - tick damage and remove expired DoTs
pub fn process_dots(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ActiveDoTs, &mut Health)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed().as_secs_f32();

    for (entity, mut active_dots, mut health) in &mut query {
        let mut dots_to_remove = Vec::new();

        for (index, dot) in active_dots.dots.iter_mut().enumerate() {
            // Check if it's time to tick
            if current_time >= dot.next_tick_at {
                // Apply damage
                health.current -= dot.damage_per_tick;
                info!("DoT tick: {} damage to entity {:?} ({} HP remaining)",
                    dot.damage_per_tick, entity, health.current);

                // Update tick counter
                dot.ticks_remaining = dot.ticks_remaining.saturating_sub(1);
                dot.next_tick_at = current_time + 1.0;  // Tick every 1 second

                // Mark for removal if no ticks left
                if dot.ticks_remaining == 0 {
                    dots_to_remove.push(index);
                }
            }
        }

        // Remove expired DoTs (in reverse order to maintain indices)
        for index in dots_to_remove.iter().rev() {
            active_dots.dots.remove(*index);
        }
    }
}

/// Add a DoT effect to an entity
pub fn add_dot(
    entity: &mut EntityWorldMut,
    ability_id: u32,
    caster: Entity,
    damage_per_tick: f32,
    ticks: u32,
    current_time: f32,
) {
    let dot = ActiveDoT {
        ability_id,
        caster,
        damage_per_tick,
        ticks_remaining: ticks,
        next_tick_at: current_time + 1.0,  // First tick in 1 second
    };

    if let Some(mut active_dots) = entity.get_mut::<ActiveDoTs>() {
        active_dots.dots.push(dot);
        info!("Added DoT from ability {} to entity {:?}", ability_id, entity.id());
    } else {
        entity.insert(ActiveDoTs {
            dots: vec![dot],
        });
        info!("Created ActiveDoTs and added DoT from ability {} to entity {:?}", ability_id, entity.id());
    }
}
