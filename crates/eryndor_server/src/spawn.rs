use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use avian2d::prelude::{RigidBody, Collider, CollisionLayers};
use crate::{PhysicsPosition, PhysicsVelocity};

/// Defines a spawn point for an entity that can respawn
#[derive(Component, Clone, Debug)]
pub struct SpawnPoint {
    pub position: Vec2,
    pub respawn_delay: f32, // seconds
}

/// Server-only event triggered when an entity should respawn
/// Contains all data needed to respawn the entity (no querying needed)
#[derive(Event, Clone, Debug)]
pub struct RespawnEvent {
    pub spawn_position: Vec2,
    pub respawn_delay: f32,
    pub template: EntityTemplate,
}

/// Timer tracking when an entity should respawn
#[derive(Component)]
pub struct RespawnTimer {
    pub timer: Timer,
    pub template: EntityTemplate,
    pub spawn_position: Vec2,
}

/// Template for creating entities with all their components
#[derive(Clone, Debug)]
pub enum EntityTemplate {
    Enemy(EnemyTemplate),
    // Future: Boss(BossTemplate), Resource(ResourceTemplate), etc.
}

/// Template for spawning enemies
#[derive(Clone, Debug)]
pub struct EnemyTemplate {
    pub enemy_type_id: u32,
    pub name: String,
    pub health: f32,
    pub move_speed: f32,
    pub attack_power: f32,
    pub defense: f32,
    pub crit_chance: f32,
    pub visual_shape: ShapeType,
    pub color: [f32; 4],
    pub size: f32,
    pub loot_table: LootTable,
    pub aggro_range: f32,
    pub leash_range: f32,
}

impl EntityTemplate {
    /// Create an enemy template from existing component data
    pub fn from_enemy_components(
        enemy_type: &EnemyType,
        enemy_name: &EnemyName,
        health: &Health,
        move_speed: &MoveSpeed,
        stats: &CombatStats,
        visual: &VisualShape,
        loot_table: LootTable,
        aggro: &AggroRange,
    ) -> Self {
        EntityTemplate::Enemy(EnemyTemplate {
            enemy_type_id: enemy_type.0,
            name: enemy_name.0.clone(),
            health: health.max,
            move_speed: move_speed.0,
            attack_power: stats.attack_power,
            defense: stats.defense,
            crit_chance: stats.crit_chance,
            visual_shape: visual.shape_type,
            color: visual.color,
            size: visual.size,
            loot_table,
            aggro_range: aggro.aggro,
            leash_range: aggro.leash,
        })
    }

    /// Spawn an entity from this template at the given position
    pub fn spawn(&self, commands: &mut Commands, position: Vec2) -> Entity {
        match self {
            EntityTemplate::Enemy(template) => {
                let enemy_entity = commands.spawn((
                    Replicated,
                    Enemy,
                    EnemyType(template.enemy_type_id),
                    EnemyName(template.name.clone()),
                    Position(position),
                    Velocity::default(),
                    MoveSpeed(template.move_speed),
                    Health::new(template.health),
                    CombatStats {
                        attack_power: template.attack_power,
                        defense: template.defense,
                        crit_chance: template.crit_chance,
                    },
                    BaseStats::new(template.attack_power, template.defense, template.move_speed),
                    CurrentTarget::default(),
                )).id();

                // Add additional components in a second batch (Bevy bundle limit workaround)
                commands.entity(enemy_entity).insert((
                    AiState::default(),
                    Interactable::enemy(),
                    VisualShape {
                        shape_type: template.visual_shape,
                        color: template.color,
                        size: template.size,
                    },
                    AbilityCooldowns::default(),
                    template.loot_table.clone(),
                    AggroRange {
                        aggro: template.aggro_range,
                        leash: template.leash_range,
                    },
                ));

                // Add AI activation delay in third batch to stay within bundle size limits
                commands.entity(enemy_entity).insert(AiActivationDelay::default());

                // Add physics components
                commands.entity(enemy_entity).insert((
                    PhysicsPosition(position),
                    PhysicsVelocity::default(),
                    RigidBody::Dynamic,
                    Collider::circle(template.size / 2.0),
                    CollisionLayers::new(
                        GameLayer::Enemy,
                        [GameLayer::Player, GameLayer::Npc, GameLayer::Enemy, GameLayer::Environment]
                    ),
                ));

                info!("Respawned enemy (type {}) at {:?}", template.enemy_type_id, position);
                enemy_entity
            }
        }
    }
}

/// Resource to track all spawn points in the world
#[derive(Resource, Default)]
pub struct SpawnRegistry {
    pub spawn_points: Vec<(SpawnPoint, EntityTemplate)>,
}

impl SpawnRegistry {
    /// Register a spawn point with its template
    pub fn register(&mut self, spawn_point: SpawnPoint, template: EntityTemplate) {
        self.spawn_points.push((spawn_point, template));
    }

    /// Register an enemy spawn point
    pub fn register_enemy(
        &mut self,
        position: Vec2,
        respawn_delay: f32,
        template: EnemyTemplate,
    ) {
        self.register(
            SpawnPoint { position, respawn_delay },
            EntityTemplate::Enemy(template),
        );
    }
}

/// When RespawnEvent is triggered, create a respawn timer
/// All data is contained in the event, so no entity querying is needed
pub fn schedule_respawn(
    trigger: On<RespawnEvent>,
    mut commands: Commands,
) {
    let event = trigger.event();

    info!("Scheduling respawn in {:.1}s at {:?}", event.respawn_delay, event.spawn_position);

    // Create respawn timer entity
    commands.spawn(RespawnTimer {
        timer: Timer::from_seconds(event.respawn_delay, TimerMode::Once),
        template: event.template.clone(),
        spawn_position: event.spawn_position,
    });
}

/// System to handle respawning entities after their timer expires
pub fn process_respawns(
    mut commands: Commands,
    mut respawn_query: Query<(Entity, &mut RespawnTimer)>,
    time: Res<Time>,
) {
    for (timer_entity, mut respawn_timer) in &mut respawn_query {
        respawn_timer.timer.tick(time.delta());

        if respawn_timer.timer.is_finished() {
            // Spawn the entity from template at the stored position
            let new_entity = respawn_timer.template.spawn(&mut commands, respawn_timer.spawn_position);

            // Add SpawnPoint component to the newly spawned entity for future respawns
            commands.entity(new_entity).insert(SpawnPoint {
                position: respawn_timer.spawn_position,
                respawn_delay: respawn_timer.timer.duration().as_secs_f32(),
            });

            // Despawn the timer entity
            commands.entity(timer_entity).despawn();
            info!("Respawn completed at {:?}", respawn_timer.spawn_position);
        }
    }
}
