use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use crate::auth::ActiveCharacterEntity;
use crate::abilities::AbilityDatabase;
use avian2d::prelude::LinearVelocity;

pub fn handle_set_target(
    trigger: On<FromClient<SetTargetRequest>>,
    clients: Query<&ActiveCharacterEntity>,
    mut players: Query<(&mut CurrentTarget, &mut AutoAttack, &mut InCombat, &Character)>,
    enemies: Query<&EnemyType, With<Enemy>>,
    npcs: Query<&NpcName, With<Npc>>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };
    let request = trigger.event();

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else { return };
    let char_entity = active_char.0;

    // Update target
    if let Ok((mut current_target, mut auto_attack, mut in_combat, character)) = players.get_mut(char_entity) {
        current_target.0 = request.target;

        // Auto-enable auto-attack and enter combat when targeting an enemy
        if let Some(target_entity) = request.target {
            if let Ok(enemy_type) = enemies.get(target_entity) {
                // Targeting an enemy - enter combat and enable auto-attack
                auto_attack.enabled = true;
                in_combat.0 = true;
                info!("{} targeted enemy: {} - auto-attack enabled",
                    character.name, enemy_type.0);
            } else if let Ok(npc_name) = npcs.get(target_entity) {
                // Targeting NPC - no combat
                info!("{} targeted NPC: {}",
                    character.name, npc_name.0);
            }
        } else {
            // Cleared target - leave combat and disable auto-attack
            auto_attack.enabled = false;
            in_combat.0 = false;
            info!("{} cleared target - exited combat", character.name);
        }
    }
}

// Auto-attack is now automatically enabled when targeting enemies
// and disabled when leaving combat - no manual toggle needed

/// Process auto-attacks for all entities with auto-attack enabled
pub fn process_auto_attacks(
    mut commands: Commands,
    mut attackers: Query<(
        Entity,
        &Position,
        &CurrentTarget,
        &CombatStats,
        &mut AutoAttack,
        &Equipment,
        &mut WeaponProficiencyExp,
    ), With<Player>>,
    mut targets: Query<(&Position, &mut Health, &CombatStats), With<Enemy>>,
    all_enemies: Query<Entity, With<Enemy>>,
    item_db: Res<crate::game_data::ItemDatabase>,
    time: Res<Time>,
) {
    for (attacker_entity, attacker_pos, current_target, attacker_stats, mut auto_attack, equipment, mut weapon_exp) in &mut attackers {
        // Skip if auto-attack is disabled
        if !auto_attack.enabled {
            continue;
        }

        // Debug: Log that we're processing auto-attack
        // if auto_attack.cooldown_timer <= 0.0 {
        //     info!("Processing auto-attack for {:?} - weapon: {:?}", attacker_entity, equipment.weapon);
        // }

        // Tick down cooldown timer
        auto_attack.cooldown_timer -= time.delta_secs();

        // Skip if still on cooldown
        if auto_attack.cooldown_timer > 0.0 {
            continue;
        }

        // Check if we have a target
        let Some(target_entity) = current_target.0 else {
            continue;
        };

        // Check if target is an enemy
        if all_enemies.get(target_entity).is_err() {
            continue;
        }

        // Get target data
        let Ok((target_pos, mut target_health, target_stats)) = targets.get_mut(target_entity) else {
            continue;
        };

        // Get weapon stats from equipped weapon (default to unarmed/fists if no weapon)
        let weapon_stats = if let Some(weapon_id) = equipment.weapon {
            // Has equipped weapon
            if let Some(weapon_type) = crate::weapon::WeaponType::from_item_id(weapon_id) {
                weapon_type.stats()
            } else {
                warn!("Unknown weapon item ID: {}, falling back to unarmed", weapon_id);
                // Unarmed fallback
                crate::weapon::WeaponStats {
                    weapon_type: crate::weapon::WeaponType::Dagger,
                    attack_speed: 1.5,
                    range: MELEE_RANGE,
                    damage_multiplier: 0.5,
                }
            }
        } else {
            // No weapon equipped - use unarmed combat
            crate::weapon::WeaponStats {
                weapon_type: crate::weapon::WeaponType::Dagger,
                attack_speed: 1.5,
                range: MELEE_RANGE,
                damage_multiplier: 0.5,
            }
        };

        // Check if target is in range
        let distance = attacker_pos.0.distance(target_pos.0);
        if distance > weapon_stats.range {
            // info!("Target {:?} out of range: {:.1} > {:.1}", target_entity, distance, weapon_stats.range);
            continue;
        }

        // info!("IN RANGE! Attacking {:?} at distance {:.1}", target_entity, distance);

        // Calculate equipment bonuses
        let equipment_bonuses = item_db.calculate_equipment_bonuses(equipment);

        // Apply equipment bonuses to combat stats
        let total_attack = attacker_stats.attack_power + equipment_bonuses.attack_power;
        let total_crit = attacker_stats.crit_chance + equipment_bonuses.crit_chance;

        // Calculate damage: total attack * weapon multiplier
        let base_damage = total_attack * weapon_stats.damage_multiplier;

        // Apply defense mitigation
        let mitigation = target_stats.defense / (target_stats.defense + 100.0);
        let mut damage = base_damage * (1.0 - mitigation);

        // Critical hit check
        let is_crit = rand::random::<f32>() < total_crit;
        if is_crit {
            damage *= 1.5;
        }

        // Apply damage
        target_health.current = (target_health.current - damage).max(0.0);

        // Reset cooldown based on weapon attack speed
        // attack_speed is attacks per second, so cooldown = 1.0 / attack_speed
        auto_attack.cooldown_timer = 1.0 / weapon_stats.attack_speed;

        info!(
            "Auto-attack: {:?} hit {:?} for {:.1} damage (crit: {})",
            attacker_entity, target_entity, damage, is_crit
        );

        // Award weapon proficiency XP for successful attack
        let weapon_xp_gain = 5; // TODO: Make this dynamic based on enemy level/difficulty
        match weapon_stats.weapon_type {
            crate::weapon::WeaponType::Sword => {
                weapon_exp.sword_xp += weapon_xp_gain;
                info!("Awarded {} Sword XP (total: {})", weapon_xp_gain, weapon_exp.sword_xp);
            },
            crate::weapon::WeaponType::Dagger => {
                weapon_exp.dagger_xp += weapon_xp_gain;
                info!("Awarded {} Dagger XP (total: {})", weapon_xp_gain, weapon_exp.dagger_xp);
            },
            crate::weapon::WeaponType::Staff => {
                weapon_exp.staff_xp += weapon_xp_gain;
                info!("Awarded {} Staff XP (total: {})", weapon_xp_gain, weapon_exp.staff_xp);
            },
            crate::weapon::WeaponType::Wand => {
                weapon_exp.wand_xp += weapon_xp_gain;
                info!("Awarded {} Wand XP (total: {})", weapon_xp_gain, weapon_exp.wand_xp);
            },
            crate::weapon::WeaponType::Mace => {
                weapon_exp.mace_xp += weapon_xp_gain;
                info!("Awarded {} Mace XP (total: {})", weapon_xp_gain, weapon_exp.mace_xp);
            },
            crate::weapon::WeaponType::Bow => {
                weapon_exp.bow_xp += weapon_xp_gain;
                info!("Awarded {} Bow XP (total: {})", weapon_xp_gain, weapon_exp.bow_xp);
            },
            crate::weapon::WeaponType::Axe => {
                weapon_exp.axe_xp += weapon_xp_gain;
                info!("Awarded {} Axe XP (total: {})", weapon_xp_gain, weapon_exp.axe_xp);
            },
        }

        // Send combat event to all clients for VFX
        commands.server_trigger(ToClients {
            mode: SendMode::Broadcast,
            message: CombatEvent {
                attacker: attacker_entity,
                target: target_entity,
                damage,
                ability_id: 0, // 0 indicates auto-attack (not an ability)
                is_crit,
            },
        });
    }
}

pub fn handle_use_ability(
    trigger: On<FromClient<UseAbilityRequest>>,
    mut commands: Commands,
    clients: Query<&ActiveCharacterEntity>,
    mut attackers: Query<(
        &Position,
        &CurrentTarget,
        &CombatStats,
        &mut Mana,
        &mut AbilityCooldowns,
        &LearnedAbilities,
        &Equipment,
    )>,
    mut targets: Query<(&Position, &mut Health, &CombatStats), With<Enemy>>,
    ability_db: Res<AbilityDatabase>,
    item_db: Res<crate::game_data::ItemDatabase>,
    time: Res<Time>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };
    let request = trigger.event();

    info!("SERVER RECEIVED UseAbilityRequest: ability_id = {}", request.ability_id);

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else {
        warn!("Could not find ActiveCharacterEntity for client {:?}", client_entity);
        return
    };
    let char_entity = active_char.0;

    // Get ability definition
    let Some(ability) = ability_db.get(request.ability_id) else {
        warn!("Unknown ability: {}", request.ability_id);
        return;
    };
    info!("Found ability: {} ({})", ability.name, ability.id);

    // Get attacker data
    let Ok((attacker_pos, current_target, stats, mut mana, mut cooldowns, learned, equipment)) =
        attackers.get_mut(char_entity) else {
            warn!("Could not get attacker components for {:?}", char_entity);
            return
        };
    info!("Got attacker components");

    // Check if ability is learned
    if !learned.knows(ability.id) {
        warn!("Player doesn't know ability {}", ability.id);
        return;
    }
    info!("Ability is learned");

    // Check cooldown
    if let Some(timer) = cooldowns.cooldowns.get(&ability.id) {
        if !timer.is_finished() {
            info!("Ability on cooldown");
            return;
        }
    }
    info!("Cooldown check passed");

    // Check mana
    if mana.current < ability.mana_cost {
        info!("Not enough mana: {} < {}", mana.current, ability.mana_cost);
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message: "Not enough mana!".to_string(),
                notification_type: NotificationType::Warning,
            },
        });
        return;
    }
    info!("Mana check passed");

    // Check target
    let Some(target_entity) = current_target.0 else {
        warn!("No target selected");
        return;
    };
    info!("Has target: {:?}", target_entity);

    // Get target data
    let Ok((target_pos, mut target_health, target_stats)) = targets.get_mut(target_entity) else {
        warn!("Could not get target components for {:?}", target_entity);
        return;
    };
    info!("Got target components");

    // Check range (convert ability range from meters to pixels: 1 meter = 20 pixels)
    const PIXELS_PER_METER: f32 = 20.0;
    let distance = attacker_pos.0.distance(target_pos.0);
    let ability_range_pixels = ability.range * PIXELS_PER_METER;
    if distance > ability_range_pixels {
        warn!("Target out of range: {:.1} > {:.1}", distance, ability_range_pixels);
        return;
    }
    info!("Range check passed: {:.1} <= {:.1}", distance, ability_range_pixels);

    // Calculate equipment bonuses
    let equipment_bonuses = item_db.calculate_equipment_bonuses(equipment);

    // Apply equipment bonuses to combat stats
    let total_attack = stats.attack_power + equipment_bonuses.attack_power;
    let total_crit = stats.crit_chance + equipment_bonuses.crit_chance;

    // Process each ability effect
    let mut total_damage = 0.0;
    let mut is_crit = false;
    let current_time = time.elapsed().as_secs_f32();

    for ability_type in &ability.ability_types {
        match ability_type {
            AbilityType::DirectDamage { multiplier } => {
                // Calculate damage
                let base_damage = total_attack * multiplier;
                let mitigation = target_stats.defense / (target_stats.defense + 100.0);
                let mut damage = base_damage * (1.0 - mitigation);

                // Critical hit check
                let crit_roll = rand::random::<f32>() < total_crit;
                if crit_roll {
                    damage *= 1.5;
                    is_crit = true;
                }

                total_damage += damage;
            }
            AbilityType::DamageOverTime { duration: _, ticks, damage_per_tick } => {
                // Add DoT effect to target
                let dot = ActiveDoT {
                    ability_id: ability.id,
                    caster: char_entity,
                    damage_per_tick: *damage_per_tick,
                    ticks_remaining: *ticks,
                    next_tick_at: current_time + 1.0,
                };

                if let Ok(mut target) = commands.get_entity(target_entity) {
                    target.insert(ActiveDoTs {
                        dots: vec![dot],
                    });
                }
            }
            AbilityType::Buff { duration, stat_bonuses } => {
                // Add buff to caster (self-buff)
                let buff = ActiveBuff {
                    ability_id: ability.id,
                    stat_bonuses: stat_bonuses.clone(),
                    expires_at: current_time + duration,
                };

                if let Ok(mut caster) = commands.get_entity(char_entity) {
                    caster.insert(ActiveBuffs {
                        buffs: vec![buff],
                    });
                }
            }
            AbilityType::Debuff { duration, effect } => {
                // Add debuff to target
                let debuff = ActiveDebuff {
                    ability_id: ability.id,
                    effect: effect.clone(),
                    expires_at: current_time + duration,
                };

                if let Ok(mut target) = commands.get_entity(target_entity) {
                    target.insert(ActiveDebuffs {
                        debuffs: vec![debuff],
                    });
                }
            }
            AbilityType::AreaOfEffect { radius: _, max_targets: _ } => {
                // TODO: Implement AoE - find multiple targets within radius
                warn!("AoE abilities not yet implemented");
            }
            AbilityType::Mobility { distance: _, dash_speed: _ } => {
                // TODO: Implement mobility - dash/teleport
                warn!("Mobility abilities not yet implemented");
            }
            AbilityType::Heal { amount: _, is_percent: _ } => {
                // TODO: Implement healing
                // Currently causes borrow checker issues if we try to heal the caster
                // while we already have a mutable borrow on the target
                warn!("Heal abilities not yet implemented");
            }
        }
    }

    // Apply total damage from all DirectDamage effects
    if total_damage > 0.0 {
        target_health.current = (target_health.current - total_damage).max(0.0);
    }

    // Consume mana
    mana.current -= ability.mana_cost;

    // Set cooldown
    cooldowns.cooldowns.insert(
        ability.id,
        Timer::from_seconds(ability.cooldown, TimerMode::Once),
    );

    info!(
        "Player {:?} used ability {} on {:?} for {:.1} damage (crit: {})",
        char_entity, ability.name, target_entity, total_damage, is_crit
    );

    // Send combat event to all clients for VFX
    commands.server_trigger(ToClients {
        mode: SendMode::Broadcast,
        message: CombatEvent {
            attacker: char_entity,
            target: target_entity,
            damage: total_damage,
            ability_id: ability.id,
            is_crit,
        },
    });
}

pub fn update_ability_cooldowns(
    mut query: Query<&mut AbilityCooldowns>,
    time: Res<Time>,
) {
    for mut cooldowns in &mut query {
        for timer in cooldowns.cooldowns.values_mut() {
            timer.tick(time.delta());
        }
    }
}

pub fn check_deaths(
    mut commands: Commands,
    query: Query<(Entity, &Health, Option<&Enemy>, Option<&Player>), Changed<Health>>,
    mut players: Query<(
        Entity,
        &CurrentTarget,
        &mut AutoAttack,
        &mut InCombat,
        &Character,
        &mut Experience,
    )>,
) {
    for (entity, health, is_enemy, is_player) in &query {
        if health.is_dead() {
            info!("Entity {:?} died", entity);

            // Trigger death event both for server (observers) and clients (visuals)
            commands.trigger(DeathEvent { entity });
            commands.server_trigger(ToClients {
                mode: SendMode::Broadcast,
                message: DeathEvent { entity },
            });

            if is_enemy.is_some() {
                // Grant XP to all players who had this enemy as their target
                for (_player_entity, current_target, mut auto_attack, mut in_combat, character, mut experience) in &mut players {
                    if current_target.0 == Some(entity) {
                        // Grant 50 base XP for killing an enemy
                        let xp_gained = 50;
                        let leveled_up = experience.add_xp(xp_gained, character.level);

                        info!("{} gained {} XP for killing enemy", character.name, xp_gained);

                        // If the player leveled up, trigger level-up event
                        if leveled_up {
                            // Note: Level-up will be handled in the check_level_ups system
                            info!("{} leveled up!", character.name);
                        }

                        // Exit combat
                        auto_attack.enabled = false;
                        in_combat.0 = false;
                        info!("Player exited combat - target died");
                    }
                }

                // Despawn enemies immediately (respawn will be handled by observer)
                commands.entity(entity).despawn();
            }

            if is_player.is_some() {
                // For now, just reset health (no death penalty in POC)
                commands.entity(entity).insert(Health::new(100.0));
                info!("Player respawned");
            }
        }
    }
}

/// Check for level-ups and apply stat increases
/// Note: Experience::add_xp already handles XP math. This system detects when
/// xp_to_next_level increased (indicating a level-up) and applies stat bonuses.
pub fn check_level_ups(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Character,
        &Experience,
        &mut Health,
        &mut Mana,
        &mut CombatStats,
    ), (With<Player>, Changed<Experience>)>,
) {
    for (_entity, mut character, experience, mut health, mut mana, mut stats) in &mut query {
        // Calculate what level the player SHOULD be based on their XP threshold
        // The xp_to_next_level is for (current_level + 1), so work backwards
        let mut expected_level = 1;
        for level in 1..100 {
            if Experience::xp_for_level(level + 1) == experience.xp_to_next_level {
                expected_level = level;
                break;
            }
        }

        // If character.level is behind expected_level, they leveled up
        if expected_level > character.level {
            let old_level = character.level;
            let levels_gained = expected_level - character.level;

            info!("check_level_ups: {} leveled up from {} to {} (gained {} levels)",
                character.name, old_level, expected_level, levels_gained);

            // Apply stat increases for each level gained
            for _ in 0..levels_gained {
                character.level += 1;

                // Calculate stat increases based on class
                let (health_increase, mana_increase, attack_increase, defense_increase) = match character.class {
                    CharacterClass::Knight => (20.0, 5.0, 3.0, 2.0),
                    CharacterClass::Mage => (10.0, 20.0, 2.0, 1.0),
                    CharacterClass::Rogue => (15.0, 10.0, 4.0, 1.5),
                };

                // Apply stat increases
                health.max += health_increase;
                health.current = health.max; // Fully heal on level-up
                mana.max += mana_increase;
                mana.current = mana.max; // Restore mana on level-up
                stats.attack_power += attack_increase;
                stats.defense += defense_increase;

                info!(
                    "{} leveled up! {} -> {} (HP: +{:.0}, MP: +{:.0}, ATK: +{:.0}, DEF: +{:.0})",
                    character.name, character.level - 1, character.level,
                    health_increase, mana_increase, attack_increase, defense_increase
                );

                // Send level-up event to all clients
                info!("Broadcasting LevelUpEvent for {} (level {})", character.name, character.level);
                commands.server_trigger(ToClients {
                    mode: SendMode::Broadcast,
                    message: LevelUpEvent {
                        new_level: character.level,
                        health_increase,
                        mana_increase,
                        attack_increase,
                        defense_increase,
                    },
                });
            }
        }
    }
}

/// Check for weapon proficiency level-ups
pub fn check_weapon_proficiency_level_ups(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Character,
        &mut WeaponProficiency,
        &mut WeaponProficiencyExp,
    ), Changed<WeaponProficiencyExp>>,
) {
    for (_entity, character, mut proficiency, mut prof_exp) in &mut query {
        info!("Checking weapon proficiency level-ups for {} - Dagger: {}/{}",
            character.name, prof_exp.dagger_xp, WeaponProficiencyExp::xp_for_level(proficiency.dagger + 1));
        // Helper macro to check level-up for a weapon type
        macro_rules! check_weapon_level_up {
            ($weapon_name:expr, $xp:expr, $level:expr) => {
                while $xp >= WeaponProficiencyExp::xp_for_level($level + 1) {
                    $level += 1;
                    let bonus_info = match $level {
                        10 => format!("+2% damage with {}", $weapon_name),
                        20 => format!("+5% attack speed with {}", $weapon_name),
                        30 => format!("+3% crit chance with {}", $weapon_name),
                        40 => format!("+4% damage with {}", $weapon_name),
                        50 => format!("+10% attack speed with {}", $weapon_name),
                        _ => format!("Improved mastery with {}", $weapon_name),
                    };

                    info!("{} - {} proficiency level up! Level {}", character.name, $weapon_name, $level);

                    commands.server_trigger(ToClients {
                        mode: SendMode::Broadcast,
                        message: ProficiencyLevelUpEvent {
                            proficiency_type: ProficiencyType::Weapon,
                            weapon_or_armor: $weapon_name.to_string(),
                            new_level: $level,
                            bonus_info,
                        },
                    });
                }
            };
        }

        check_weapon_level_up!("Sword", prof_exp.sword_xp, proficiency.sword);
        check_weapon_level_up!("Dagger", prof_exp.dagger_xp, proficiency.dagger);
        check_weapon_level_up!("Staff", prof_exp.staff_xp, proficiency.staff);
        check_weapon_level_up!("Mace", prof_exp.mace_xp, proficiency.mace);
        check_weapon_level_up!("Bow", prof_exp.bow_xp, proficiency.bow);
        check_weapon_level_up!("Axe", prof_exp.axe_xp, proficiency.axe);
    }
}

pub fn enemy_ai(
    mut enemies: Query<(
        &mut AiState,
        &mut Position,
        &mut Velocity,
        &mut LinearVelocity,
        &mut CurrentTarget,
        &MoveSpeed,
        &CombatStats,
        &EnemyType,
    ), (With<Enemy>, Without<Player>, Without<AiActivationDelay>)>,
    mut players: Query<(Entity, &Position, &mut Health), (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    for (mut ai_state, mut enemy_pos, mut velocity, mut physics_velocity, mut current_target, move_speed, stats, _enemy_type) in &mut enemies {
        match *ai_state {
            AiState::Idle => {
                // Look for nearby players
                for (player_entity, player_pos, _) in &players {
                    let distance = enemy_pos.0.distance(player_pos.0);
                    if distance < AGGRO_RANGE {
                        *ai_state = AiState::Chasing(player_entity);
                        current_target.0 = Some(player_entity);
                        break;
                    }
                }
            }
            AiState::Chasing(target_entity) => {
                // Check if target still exists
                if let Ok((_, target_pos, _)) = players.get(target_entity) {
                    let distance = enemy_pos.0.distance(target_pos.0);

                    // Check leash range
                    if distance > LEASH_RANGE {
                        *ai_state = AiState::Idle;
                        current_target.0 = None;
                        velocity.0 = Vec2::ZERO;
                        physics_velocity.0 = Vec2::ZERO;
                        continue;
                    }

                    // Check if in attack range
                    if distance < MELEE_RANGE {
                        *ai_state = AiState::Attacking(target_entity);
                        velocity.0 = Vec2::ZERO;
                        physics_velocity.0 = Vec2::ZERO;
                    } else {
                        // Move towards target
                        let direction = (target_pos.0 - enemy_pos.0).normalize();
                        let vel = direction * move_speed.0;
                        velocity.0 = vel;
                        physics_velocity.0 = vel;
                    }
                } else {
                    *ai_state = AiState::Idle;
                    current_target.0 = None;
                }
            }
            AiState::Attacking(target_entity) => {
                // Check if target still exists and in range
                if let Ok((_, target_pos, mut target_health)) = players.get_mut(target_entity) {
                    let distance = enemy_pos.0.distance(target_pos.0);

                    if distance > MELEE_RANGE {
                        *ai_state = AiState::Chasing(target_entity);
                    } else {
                        // Simple auto-attack every second
                        // TODO: Add attack cooldown timer
                        let damage = stats.attack_power;
                        target_health.current = (target_health.current - damage * time.delta_secs()).max(0.0);
                    }
                } else {
                    *ai_state = AiState::Idle;
                    current_target.0 = None;
                }
            }
        }
    }
}

/// Tick AI activation delay timers and remove the component when ready
/// This allows enemies to "warm up" after spawning to ensure replication completes
pub fn update_ai_activation_delays(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AiActivationDelay)>,
    time: Res<Time>,
) {
    for (entity, mut delay) in &mut query {
        delay.timer.tick(time.delta());

        if delay.timer.finished() {
            commands.entity(entity).remove::<AiActivationDelay>();
            info!("Enemy {:?} AI activated", entity);
        }
    }
}
