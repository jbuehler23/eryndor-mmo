use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use crate::auth::ActiveCharacterEntity;
use crate::abilities::AbilityDatabase;
use crate::spawn::{SpawnPoint, RespawnEvent, EntityTemplate};
use avian2d::prelude::{LinearVelocity, Position as PhysicsPosition};
use rand::Rng;

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
        &WeaponProficiency,
    ), With<Player>>,
    mut targets: Query<(&Position, &mut Health, &CombatStats), (With<Enemy>, Without<AiActivationDelay>)>,
    all_enemies: Query<Entity, With<Enemy>>,
    item_db: Res<crate::game_data::ItemDatabase>,
    time: Res<Time>,
) {
    for (attacker_entity, attacker_pos, current_target, attacker_stats, mut auto_attack, equipment, mut weapon_exp, weapon_prof) in &mut attackers {
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

        // Apply weapon proficiency bonus (2% per level)
        let prof_level = crate::weapon::get_proficiency_level(weapon_prof, &weapon_stats.weapon_type);
        let prof_bonus = 1.0 + ((prof_level - 1) as f32 * 0.02);
        let damage_with_prof = base_damage * prof_bonus;

        // Apply defense mitigation
        let mitigation = target_stats.defense / (target_stats.defense + 100.0);
        let mut damage = damage_with_prof * (1.0 - mitigation);

        // Critical hit check
        let is_crit = rand::random::<f32>() < total_crit;
        if is_crit {
            damage *= 1.5;
        }

        // Apply damage
        target_health.current = (target_health.current - damage).max(0.0);

        // Make enemy aggro on the attacker
        if let Ok(mut enemy) = commands.get_entity(target_entity) {
            enemy.insert(AiState::Chasing(attacker_entity));
            enemy.insert(CurrentTarget(Some(attacker_entity)));
            info!("Enemy {:?} aggroed on attacker {:?}", target_entity, attacker_entity);
        }

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

        // Send combat event to all clients for VFX (using positions to avoid entity mapping issues)
        commands.server_trigger(ToClients {
            mode: SendMode::Broadcast,
            message: CombatEvent {
                attacker_position: attacker_pos.0,
                target_position: target_pos.0,
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
        Entity,
        &Position,
        &CurrentTarget,
        &CombatStats,
        &mut Mana,
        &mut Health,
        &mut AbilityCooldowns,
        &LearnedAbilities,
        &Equipment,
    ), Without<Enemy>>,
    mut targets: Query<(Entity, &Position, &mut Health, &CombatStats), (With<Enemy>, Without<AiActivationDelay>)>,
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
    let Ok((attacker_entity, attacker_pos, current_target, stats, mut mana, mut attacker_health, mut cooldowns, learned, equipment)) =
        attackers.get_mut(char_entity) else {
            warn!("Could not get attacker components for {:?}", char_entity);
            return
        };
    info!("Got attacker components");

    // Store attacker position for later use (after mutable borrow ends)
    let attacker_position = attacker_pos.0;

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

    // Check target (some abilities like self-heals might not need a target)
    let target_entity_opt = current_target.0;

    // Check if this ability requires a target
    let requires_target = ability.ability_types.iter().any(|t| matches!(t,
        AbilityType::DirectDamage { .. } |
        AbilityType::DamageOverTime { .. } |
        AbilityType::AreaOfEffect { .. } |
        AbilityType::Debuff { .. }
    ));

    // Validate target if required
    let primary_target_data = if requires_target {
        let Some(target_entity) = target_entity_opt else {
            warn!("No target selected for ability that requires target");
            return;
        };
        info!("Has target: {:?}", target_entity);

        let Ok((_, target_pos, target_health, target_stats)) = targets.get(target_entity) else {
            warn!("Could not get target components for {:?}", target_entity);
            return;
        };

        // Check range (convert ability range from meters to pixels: 1 meter = 20 pixels)
        const PIXELS_PER_METER: f32 = 20.0;
        let distance = attacker_position.distance(target_pos.0);
        let ability_range_pixels = ability.range * PIXELS_PER_METER;
        if distance > ability_range_pixels {
            warn!("Target out of range: {:.1} > {:.1}", distance, ability_range_pixels);
            return;
        }
        info!("Range check passed: {:.1} <= {:.1}", distance, ability_range_pixels);

        Some((target_entity, target_pos.0, *target_stats))
    } else {
        None
    };

    // Calculate equipment bonuses
    let equipment_bonuses = item_db.calculate_equipment_bonuses(equipment);

    // Apply equipment bonuses to combat stats
    let total_attack = stats.attack_power + equipment_bonuses.attack_power;
    let total_crit = stats.crit_chance + equipment_bonuses.crit_chance;

    // Process ability effects
    let current_time = time.elapsed().as_secs_f32();
    const PIXELS_PER_METER: f32 = 20.0;

    // First, determine if this is an AoE ability and collect targets
    let mut aoe_params: Option<(f32, u32)> = None;
    for ability_type in &ability.ability_types {
        if let AbilityType::AreaOfEffect { radius, max_targets } = ability_type {
            aoe_params = Some((*radius * PIXELS_PER_METER, *max_targets));
            break;
        }
    }

    // Collect targets for damage/effects
    let affected_targets: Vec<(Entity, Vec2, CombatStats)> = if let Some((radius, max_targets)) = aoe_params {
        // AoE: Find all enemies within radius of the primary target
        let (_, target_pos, _) = primary_target_data.as_ref()
            .expect("AoE ability should have a target");

        let mut nearby_enemies: Vec<(Entity, Vec2, CombatStats, f32)> = Vec::new();
        for (enemy_entity, enemy_pos, _, enemy_stats) in targets.iter() {
            let dist = target_pos.distance(enemy_pos.0);
            if dist <= radius {
                nearby_enemies.push((enemy_entity, enemy_pos.0, *enemy_stats, dist));
            }
        }

        // Sort by distance and take up to max_targets
        nearby_enemies.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());
        nearby_enemies.truncate(max_targets as usize);

        info!("AoE ability hit {} targets within {} radius", nearby_enemies.len(), radius);

        nearby_enemies.into_iter().map(|(e, p, s, _)| (e, p, s)).collect()
    } else if let Some((target_entity, target_pos, target_stats)) = primary_target_data {
        // Single target
        vec![(target_entity, target_pos, target_stats)]
    } else {
        vec![]
    };

    // Track total damage for combat events
    let mut total_damage = 0.0;
    let mut is_crit = false;
    let mut primary_target_pos = attacker_position;

    // Process each ability effect
    for ability_type in &ability.ability_types {
        match ability_type {
            AbilityType::DirectDamage { multiplier } => {
                // Apply damage to all affected targets
                for (target_entity, target_pos, target_stats) in &affected_targets {
                    let base_damage = total_attack * multiplier;
                    let mitigation = target_stats.defense / (target_stats.defense + 100.0);
                    let mut damage = base_damage * (1.0 - mitigation);

                    // Critical hit check (roll once per target)
                    let crit_roll = rand::random::<f32>() < total_crit;
                    if crit_roll {
                        damage *= 1.5;
                        is_crit = true;
                    }

                    // Apply damage
                    if let Ok((_, _, mut target_health, _)) = targets.get_mut(*target_entity) {
                        target_health.current = (target_health.current - damage).max(0.0);
                    }

                    // Make enemy aggro on attacker
                    if let Ok(mut enemy) = commands.get_entity(*target_entity) {
                        enemy.insert(AiState::Chasing(attacker_entity));
                        enemy.insert(CurrentTarget(Some(attacker_entity)));
                    }

                    total_damage += damage;
                    primary_target_pos = *target_pos;

                    info!("DirectDamage: {:?} took {:.1} damage (crit: {})", target_entity, damage, crit_roll);
                }
            }
            AbilityType::DamageOverTime { duration: _, ticks, damage_per_tick } => {
                // Apply DoT to all affected targets
                for (target_entity, _, _) in &affected_targets {
                    let dot = ActiveDoT {
                        ability_id: ability.id,
                        caster: char_entity,
                        damage_per_tick: *damage_per_tick,
                        ticks_remaining: *ticks,
                        next_tick_at: current_time + 1.0,
                    };

                    if let Ok(mut target) = commands.get_entity(*target_entity) {
                        target.insert(ActiveDoTs {
                            dots: vec![dot],
                        });
                        info!("Applied DoT to {:?}", target_entity);
                    }
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
                    info!("Applied buff from ability {} to caster", ability.id);
                }
            }
            AbilityType::Debuff { duration, effect } => {
                // Apply debuff to all affected targets
                for (target_entity, _, _) in &affected_targets {
                    let debuff = ActiveDebuff {
                        ability_id: ability.id,
                        effect: effect.clone(),
                        expires_at: current_time + duration,
                    };

                    if let Ok(mut target) = commands.get_entity(*target_entity) {
                        target.insert(ActiveDebuffs {
                            debuffs: vec![debuff],
                        });
                        info!("Applied debuff to {:?}", target_entity);
                    }
                }
            }
            AbilityType::AreaOfEffect { .. } => {
                // AoE is handled by target collection above, no additional action needed
            }
            AbilityType::Mobility { distance, dash_speed: _ } => {
                // Determine dash direction - prefer target entity, fall back to cursor position
                let dash_target: Option<Vec2> = primary_target_data
                    .as_ref()
                    .map(|(_, target_pos, _)| *target_pos)
                    .or(request.target_position);

                if let Some(target_pos) = dash_target {
                    let direction = (target_pos - attacker_position).normalize_or_zero();
                    let move_distance = distance * PIXELS_PER_METER;
                    let new_position = attacker_position + direction * move_distance;

                    // Update caster's position using commands to avoid query conflicts
                    if let Ok(mut entity_cmd) = commands.get_entity(char_entity) {
                        entity_cmd.insert(Position(new_position));
                        entity_cmd.insert(PhysicsPosition(new_position));
                        info!("Mobility: Moved caster {:.1} units towards target (new pos: {:?})", move_distance, new_position);
                    } else {
                        warn!("Mobility: Could not get entity commands for {:?}", char_entity);
                    }
                } else {
                    warn!("Mobility ability used without target or cursor position - no movement");
                }
            }
            AbilityType::Heal { amount, is_percent } => {
                // Heal the caster
                let heal_amount = if *is_percent {
                    attacker_health.max * (*amount / 100.0)
                } else {
                    *amount
                };

                let old_health = attacker_health.current;
                attacker_health.current = (attacker_health.current + heal_amount).min(attacker_health.max);
                let actual_heal = attacker_health.current - old_health;

                info!("Heal: Restored {:.1} HP to caster (was {:.1}, now {:.1})",
                    actual_heal, old_health, attacker_health.current);

                // Send heal event to clients
                commands.server_trigger(ToClients {
                    mode: SendMode::Broadcast,
                    message: CombatEvent {
                        attacker_position,
                        target_position: attacker_position, // Healing self
                        damage: -actual_heal, // Negative damage = heal
                        ability_id: ability.id,
                        is_crit: false,
                    },
                });
            }
        }
    }

    // Consume mana
    mana.current -= ability.mana_cost;

    // Set cooldown
    cooldowns.cooldowns.insert(
        ability.id,
        Timer::from_seconds(ability.cooldown, TimerMode::Once),
    );

    info!(
        "Player {:?} used ability {} for {:.1} total damage (crit: {}, targets: {})",
        char_entity, ability.name, total_damage, is_crit, affected_targets.len()
    );

    // Send combat event to all clients for VFX (only if there was damage dealt)
    if total_damage > 0.0 {
        commands.server_trigger(ToClients {
            mode: SendMode::Broadcast,
            message: CombatEvent {
                attacker_position,
                target_position: primary_target_pos,
                damage: total_damage,
                ability_id: ability.id,
                is_crit,
            },
        });
    }
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

/// Regenerate health and mana over time
pub fn regenerate_resources(
    mut query: Query<(&mut Health, &mut Mana, &HealthRegen, &ManaRegen, &InCombat), With<Player>>,
    time: Res<Time>,
) {
    for (mut health, mut mana, health_regen, mana_regen, in_combat) in &mut query {
        let delta = time.delta_secs();

        // Calculate health regen based on combat state
        let health_multiplier = if in_combat.0 {
            health_regen.in_combat_multiplier
        } else {
            1.0
        };
        let health_regen_amount = health_regen.base_regen * health_multiplier * delta;

        // Calculate mana regen based on combat state
        let mana_multiplier = if in_combat.0 {
            mana_regen.in_combat_multiplier
        } else {
            1.0
        };
        let mana_regen_amount = mana_regen.base_regen * mana_multiplier * delta;

        // Apply regeneration (don't exceed max values)
        if health.current < health.max && health_regen_amount > 0.0 {
            health.current = (health.current + health_regen_amount).min(health.max);
        }

        if mana.current < mana.max && mana_regen_amount > 0.0 {
            mana.current = (mana.current + mana_regen_amount).min(mana.max);
        }
    }
}

pub fn check_deaths(
    mut commands: Commands,
    query: Query<(
        Entity,
        &Health,
        &Position,
        Option<&Enemy>,
        Option<&Player>,
        Option<&LootTable>,
        Option<&EnemyType>,
        Option<&EnemyName>,
        Option<&SpawnPoint>,
        Option<&MoveSpeed>,
        Option<&CombatStats>,
        Option<&VisualShape>,
        Option<&AggroRange>,
    ), Changed<Health>>,
    mut players: Query<(
        Entity,
        &mut CurrentTarget,
        &mut AutoAttack,
        &mut InCombat,
        &Character,
        &mut Experience,
        &mut QuestLog,
    )>,
    quest_db: Res<crate::game_data::QuestDatabase>,
) {
    for (entity, health, position, is_enemy, is_player, loot_table, enemy_type, enemy_name, spawn_point, move_speed, combat_stats, visual_shape, aggro_range) in &query {
        if health.is_dead() {
            info!("Entity {:?} died", entity);

            // Trigger death event both for server (observers) and clients (visuals)
            commands.trigger(DeathEvent { entity, position: position.0 });
            commands.server_trigger(ToClients {
                mode: SendMode::Broadcast,
                message: DeathEvent { entity, position: position.0 },
            });

            if is_enemy.is_some() {
                // Grant XP to all players who had this enemy as their target
                for (player_entity, mut current_target, mut auto_attack, mut in_combat, character, mut experience, mut quest_log) in &mut players {
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

                        // Update quest progress for kill objectives
                        if let Some(enemy_type_val) = enemy_type {
                            for active_quest in &mut quest_log.active_quests {
                                if let Some(quest_def) = quest_db.quests.get(&active_quest.quest_id) {
                                    for (i, objective) in quest_def.objectives.iter().enumerate() {
                                        if let crate::game_data::QuestObjective::KillEnemy { enemy_type: required_type, count } = objective {
                                            if enemy_type_val.0 == *required_type && active_quest.progress[i] < *count {
                                                active_quest.progress[i] += 1;
                                                info!("{} quest {} kill progress: {}/{}",
                                                    character.name, quest_def.name, active_quest.progress[i], count);

                                                // Notify player of progress
                                                commands.server_trigger(ToClients {
                                                    mode: SendMode::Direct(player_entity.into()),
                                                    message: QuestUpdateEvent {
                                                        quest_id: active_quest.quest_id,
                                                        message: format!("{}: {}/{}", quest_def.name, active_quest.progress[i], count),
                                                    },
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Exit combat and clear target
                        auto_attack.enabled = false;
                        in_combat.0 = false;
                        current_target.0 = None;
                        info!("Player exited combat - target died");
                    }
                }

                // Drop loot if enemy has a loot table
                if let Some(loot) = loot_table {
                    drop_loot(&mut commands, loot, *position, enemy_name);
                }

                // Schedule respawn if enemy has a spawn point
                // Build template from components BEFORE despawning
                if let (Some(sp), Some(et), Some(en), Some(ms), Some(cs), Some(vs), Some(ar)) =
                    (spawn_point, enemy_type, enemy_name, move_speed, combat_stats, visual_shape, aggro_range)
                {
                    let template = EntityTemplate::from_enemy_components(
                        et,
                        en,
                        health,
                        ms,
                        cs,
                        vs,
                        loot_table.cloned().unwrap_or_default(),
                        ar,
                    );

                    // Trigger respawn event with all data embedded
                    commands.trigger(RespawnEvent {
                        spawn_position: sp.position,
                        respawn_delay: sp.respawn_delay,
                        template,
                    });

                    info!("Scheduled respawn for {} at {:?} in {:.1}s", en.0, sp.position, sp.respawn_delay);
                }

                // Despawn enemy (use get_entity to check existence before despawning)
                if let Ok(mut entity_commands) = commands.get_entity(entity) {
                    entity_commands.despawn();
                }
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
        &OwnedBy,
    ), Changed<WeaponProficiencyExp>>,
) {
    for (_entity, character, mut proficiency, mut prof_exp, owned_by) in &mut query {
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
                        mode: SendMode::Direct(ClientId::Client(owned_by.0)),
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
        check_weapon_level_up!("Wand", prof_exp.wand_xp, proficiency.wand);
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
        &AggroRange,
    ), (With<Enemy>, Without<Player>, Without<AiActivationDelay>)>,
    mut players: Query<(Entity, &Position, &mut Health), (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    for (mut ai_state, enemy_pos, mut velocity, mut physics_velocity, mut current_target, move_speed, stats, _enemy_type, aggro_range) in &mut enemies {
        match *ai_state {
            AiState::Idle => {
                // Look for nearby players
                for (player_entity, player_pos, _) in &players {
                    let distance = enemy_pos.0.distance(player_pos.0);
                    if distance < aggro_range.aggro {
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
                    if distance > aggro_range.leash {
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

        if delay.timer.is_finished() {
            commands.entity(entity).remove::<AiActivationDelay>();
            info!("Enemy {:?} AI activated", entity);
        }
    }
}

/// Drop loot from an enemy based on its loot table
/// Spawns a LootContainer entity containing all dropped gold and items
fn drop_loot(commands: &mut Commands, loot_table: &LootTable, position: Position, enemy_name: Option<&EnemyName>) {
    let mut rng = rand::thread_rng();
    let mut loot_contents = Vec::new();

    // Get enemy name from component
    let source_name = enemy_name
        .map(|n| n.0.clone())
        .unwrap_or_else(|| "Unknown Enemy".to_string());

    // Always drop gold if the range is non-zero
    if loot_table.gold_max > 0 {
        let gold_amount = if loot_table.gold_min == loot_table.gold_max {
            loot_table.gold_max
        } else {
            rng.gen_range(loot_table.gold_min..=loot_table.gold_max)
        };

        if gold_amount > 0 {
            loot_contents.push(LootContents::Gold(gold_amount));
            info!("Rolling {} gold into loot container at {:?}", gold_amount, position.0);
        }
    }

    // Roll for item drops
    for loot_item in &loot_table.items {
        let roll: f32 = rng.gen();
        if roll <= loot_item.drop_chance {
            let quantity = if loot_item.quantity_min == loot_item.quantity_max {
                loot_item.quantity_max
            } else {
                rng.gen_range(loot_item.quantity_min..=loot_item.quantity_max)
            };

            if quantity > 0 {
                loot_contents.push(LootContents::Item(ItemStack {
                    item_id: loot_item.item_id,
                    quantity,
                }));
                info!("Rolling item {} (x{}) into loot container at {:?}", loot_item.item_id, quantity, position.0);
            }
        }
    }

    // Only spawn loot container if there's something to loot
    if !loot_contents.is_empty() {
        commands.spawn((
            Replicated,
            LootContainer {
                contents: loot_contents.clone(),
                source_name: source_name.clone(),
            },
            position,
            Interactable::loot_container(),
            VisualShape {
                shape_type: ShapeType::Square,
                color: COLOR_LOOT_CONTAINER,
                size: LOOT_CONTAINER_SIZE,
            },
            crate::PhysicsPosition(position.0),
        ));
        info!("Spawned loot container from {} with {} items at {:?}", source_name, loot_contents.len(), position.0);
    }
}
