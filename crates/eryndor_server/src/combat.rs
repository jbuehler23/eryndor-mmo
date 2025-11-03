use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use crate::auth::ActiveCharacterEntity;
use crate::game_data::AbilityDatabase;

pub fn handle_set_target(
    trigger: On<FromClient<SetTargetRequest>>,
    clients: Query<&ActiveCharacterEntity>,
    mut players: Query<&mut CurrentTarget>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };
    let request = trigger.event();

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else { return };
    let char_entity = active_char.0;

    // Update target
    if let Ok(mut current_target) = players.get_mut(char_entity) {
        current_target.0 = request.target;
        info!("Player {:?} targeted {:?}", char_entity, request.target);
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
    )>,
    mut targets: Query<(&Position, &mut Health, &CombatStats), Without<Player>>,
    ability_db: Res<AbilityDatabase>,
    time: Res<Time>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };
    let request = trigger.event();

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else { return };
    let char_entity = active_char.0;

    // Get ability definition
    let Some(ability) = ability_db.abilities.get(&request.ability_id) else {
        warn!("Unknown ability: {}", request.ability_id);
        return;
    };

    // Get attacker data
    let Ok((attacker_pos, current_target, stats, mut mana, mut cooldowns, learned)) =
        attackers.get_mut(char_entity) else { return };

    // Check if ability is learned
    if !learned.knows(ability.id) {
        warn!("Player doesn't know ability {}", ability.id);
        return;
    }

    // Check cooldown
    if let Some(timer) = cooldowns.cooldowns.get(&ability.id) {
        if !timer.is_finished() {
            return;
        }
    }

    // Check mana
    if mana.current < ability.mana_cost {
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message: "Not enough mana!".to_string(),
                notification_type: NotificationType::Warning,
            },
        });
        return;
    }

    // Check target
    let Some(target_entity) = current_target.0 else {
        return;
    };

    // Get target data
    let Ok((target_pos, mut target_health, target_stats)) = targets.get_mut(target_entity) else {
        return;
    };

    // Check range
    let distance = attacker_pos.0.distance(target_pos.0);
    if distance > ability.range {
        return;
    }

    // Calculate damage
    let base_damage = stats.attack_power * ability.damage_multiplier;
    let mitigation = target_stats.defense / (target_stats.defense + 100.0);
    let mut damage = base_damage * (1.0 - mitigation);

    // Critical hit check
    let is_crit = rand::random::<f32>() < stats.crit_chance;
    if is_crit {
        damage *= 1.5;
    }

    // Apply damage
    target_health.current = (target_health.current - damage).max(0.0);

    // Consume mana
    mana.current -= ability.mana_cost;

    // Set cooldown
    cooldowns.cooldowns.insert(
        ability.id,
        Timer::from_seconds(ability.cooldown, TimerMode::Once),
    );

    info!(
        "Player {:?} used ability {} on {:?} for {:.1} damage (crit: {})",
        char_entity, ability.name, target_entity, damage, is_crit
    );

    // Send combat event to all clients for VFX
    commands.server_trigger(ToClients {
        mode: SendMode::Broadcast,
        message: CombatEvent {
            attacker: char_entity,
            target: target_entity,
            damage,
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
) {
    for (entity, health, is_enemy, is_player) in &query {
        if health.is_dead() {
            info!("Entity {:?} died", entity);

            // Send death event
            commands.server_trigger(ToClients {
                mode: SendMode::Broadcast,
                message: DeathEvent { entity },
            });

            if is_enemy.is_some() {
                // Despawn enemies immediately
                commands.entity(entity).despawn();
                // TODO: Respawn after delay
            }

            if is_player.is_some() {
                // For now, just reset health (no death penalty in POC)
                commands.entity(entity).insert(Health::new(100.0));
                info!("Player respawned");
            }
        }
    }
}

pub fn enemy_ai(
    mut enemies: Query<(
        &mut AiState,
        &mut Position,
        &mut Velocity,
        &mut CurrentTarget,
        &MoveSpeed,
        &CombatStats,
        &EnemyType,
    ), (With<Enemy>, Without<Player>)>,
    mut players: Query<(Entity, &Position, &mut Health), (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    for (mut ai_state, mut enemy_pos, mut velocity, mut current_target, move_speed, stats, _enemy_type) in &mut enemies {
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
                        continue;
                    }

                    // Check if in attack range
                    if distance < MELEE_RANGE {
                        *ai_state = AiState::Attacking(target_entity);
                        velocity.0 = Vec2::ZERO;
                    } else {
                        // Move towards target
                        let direction = (target_pos.0 - enemy_pos.0).normalize();
                        velocity.0 = direction * move_speed.0;
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
