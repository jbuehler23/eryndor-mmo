use bevy::prelude::*;
use eryndor_shared::*;

// ============================================================================
// BUFF/DEBUFF SYSTEM (Combined to avoid query conflicts)
// ============================================================================

/// Process active buffs and debuffs - apply stat bonuses/penalties and remove expired effects
pub fn process_buffs_and_debuffs(
    mut query: Query<(
        Option<&mut ActiveBuffs>,
        Option<&mut ActiveDebuffs>,
        &BaseStats,
        &mut CombatStats,
        &mut MoveSpeed,
    )>,
    time: Res<Time>,
) {
    let current_time = time.elapsed().as_secs_f32();

    for (active_buffs, active_debuffs, base_stats, mut stats, mut move_speed) in &mut query {
        // Start with base stats
        let mut final_attack = base_stats.attack_power;
        let mut final_defense = base_stats.defense;
        let mut final_move_speed = base_stats.move_speed;

        // Process buffs first (additive bonuses)
        if let Some(mut buffs) = active_buffs {
            // Remove expired buffs
            buffs.buffs.retain(|buff| buff.expires_at > current_time);

            // Calculate total bonuses from all active buffs
            for buff in &buffs.buffs {
                final_attack += buff.stat_bonuses.attack_power;
                final_defense += buff.stat_bonuses.defense;
                final_move_speed += buff.stat_bonuses.move_speed;
            }
        }

        // Process debuffs second (multiplicative penalties)
        let mut is_rooted = false;
        if let Some(mut debuffs) = active_debuffs {
            // Remove expired debuffs
            debuffs.debuffs.retain(|debuff| debuff.expires_at > current_time);

            let mut speed_multiplier = 1.0;
            let mut attack_multiplier = 1.0;

            for debuff in &debuffs.debuffs {
                match &debuff.effect {
                    DebuffType::Slow { move_speed_reduction } => {
                        speed_multiplier *= 1.0 - move_speed_reduction;
                    },
                    DebuffType::Weaken { attack_reduction } => {
                        attack_multiplier *= 1.0 - attack_reduction;
                    },
                    DebuffType::Stun | DebuffType::Root => {
                        is_rooted = true;
                    },
                }
            }

            // Apply debuff multipliers
            final_attack *= attack_multiplier;
            final_move_speed *= speed_multiplier;
        }

        // Apply final values
        stats.attack_power = final_attack;
        stats.defense = final_defense;
        move_speed.0 = if is_rooted { 0.0 } else { final_move_speed };
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
