use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use avian2d::prelude::{RigidBody, Collider, CollisionLayers};
use crate::{PhysicsPosition, PhysicsVelocity};
use crate::game_data::{EnemyDatabase, ZoneDatabase};
use crate::spawn::SpawnPoint;

/// Marker resource indicating the world has been spawned
#[derive(Resource, Default)]
pub struct WorldSpawned;

/// Run condition: returns true when zone data is loaded and world hasn't been spawned
pub fn zone_data_loaded(
    zone_db: Res<ZoneDatabase>,
    world_spawned: Option<Res<WorldSpawned>>,
) -> bool {
    // Only spawn if zone data exists and world hasn't been spawned yet
    world_spawned.is_none() && zone_db.zones.contains_key("starter_zone")
}

/// System to spawn world boundaries at startup (doesn't depend on JSON data)
pub fn spawn_world_boundaries(mut commands: Commands) {
    info!("Spawning world boundaries...");

    let wall_thickness = 20.0;
    let half_width = WORLD_WIDTH / 2.0;
    let half_height = WORLD_HEIGHT / 2.0;
    let segment_size = 50.0;
    let collider_offset = (segment_size / 2.0) - (wall_thickness / 2.0);

    // North wall (top)
    commands.spawn((
        PhysicsPosition(Vec2::new(0.0, half_height - collider_offset)),
        RigidBody::Static,
        Collider::rectangle(WORLD_WIDTH, wall_thickness),
        CollisionLayers::new(GameLayer::Environment, [GameLayer::Player, GameLayer::Enemy]),
    ));
    let num_segments = (WORLD_WIDTH / segment_size).ceil() as i32;
    for i in 0..num_segments {
        let x = -half_width + (i as f32 * segment_size) + (segment_size / 2.0);
        commands.spawn((
            Replicated,
            Position(Vec2::new(x, half_height)),
            VisualShape {
                shape_type: ShapeType::Square,
                color: [1.0, 0.0, 0.0, 1.0],
                size: segment_size,
            },
        ));
    }

    // South wall (bottom)
    commands.spawn((
        PhysicsPosition(Vec2::new(0.0, -half_height + collider_offset)),
        RigidBody::Static,
        Collider::rectangle(WORLD_WIDTH, wall_thickness),
        CollisionLayers::new(GameLayer::Environment, [GameLayer::Player, GameLayer::Enemy]),
    ));
    for i in 0..num_segments {
        let x = -half_width + (i as f32 * segment_size) + (segment_size / 2.0);
        commands.spawn((
            Replicated,
            Position(Vec2::new(x, -half_height)),
            VisualShape {
                shape_type: ShapeType::Square,
                color: [1.0, 0.0, 0.0, 1.0],
                size: segment_size,
            },
        ));
    }

    // East wall (right)
    commands.spawn((
        PhysicsPosition(Vec2::new(half_width - collider_offset, 0.0)),
        RigidBody::Static,
        Collider::rectangle(wall_thickness, WORLD_HEIGHT),
        CollisionLayers::new(GameLayer::Environment, [GameLayer::Player, GameLayer::Enemy]),
    ));
    let num_segments_vertical = (WORLD_HEIGHT / segment_size).ceil() as i32;
    for i in 0..num_segments_vertical {
        let y = -half_height + (i as f32 * segment_size) + (segment_size / 2.0);
        commands.spawn((
            Replicated,
            Position(Vec2::new(half_width, y)),
            VisualShape {
                shape_type: ShapeType::Square,
                color: [1.0, 0.0, 0.0, 1.0],
                size: segment_size,
            },
        ));
    }

    // West wall (left)
    commands.spawn((
        PhysicsPosition(Vec2::new(-half_width + collider_offset, 0.0)),
        RigidBody::Static,
        Collider::rectangle(wall_thickness, WORLD_HEIGHT),
        CollisionLayers::new(GameLayer::Environment, [GameLayer::Player, GameLayer::Enemy]),
    ));
    for i in 0..num_segments_vertical {
        let y = -half_height + (i as f32 * segment_size) + (segment_size / 2.0);
        commands.spawn((
            Replicated,
            Position(Vec2::new(-half_width, y)),
            VisualShape {
                shape_type: ShapeType::Square,
                color: [1.0, 0.0, 0.0, 1.0],
                size: segment_size,
            },
        ));
    }

    info!("World boundaries spawned");
}

/// Marker component for tilemap collision entities
#[derive(Component)]
pub struct TilemapCollider;

/// System to spawn world entities from zone data (runs when zone data is loaded)
pub fn spawn_world(
    mut commands: Commands,
    enemy_db: Res<EnemyDatabase>,
    zone_db: Res<ZoneDatabase>,
) {
    info!("Zone data loaded, spawning world entities...");

    // Mark world as spawned
    commands.insert_resource(WorldSpawned);

    // Spawn from zone data
    if let Some(zone) = zone_db.zones.get("starter_zone") {
        info!("Spawning world from zone: {}", zone.zone_name);
        spawn_zone_npcs(&mut commands, zone);
        spawn_zone_enemies(&mut commands, zone, &enemy_db);
        spawn_tilemap_collision(&mut commands, zone);
    }

    info!("World initialization complete!");
}

/// Spawn NPCs from zone definition
fn spawn_zone_npcs(commands: &mut Commands, zone: &crate::game_data::ZoneDefinition) {
    for npc in &zone.npc_spawns {
        let position = Vec2::from(npc.position);

        match npc.npc_type.as_str() {
            "QuestGiver" => {
                commands.spawn((
                    Replicated,
                    Npc,
                    NpcName(npc.name.clone()),
                    QuestGiver {
                        available_quests: npc.quests.clone(),
                    },
                    Position(position),
                    Interactable::npc(),
                    VisualShape {
                        shape_type: ShapeType::Circle,
                        color: npc.visual.color,
                        size: npc.visual.size,
                    },
                    PhysicsPosition(position),
                    RigidBody::Static,
                    Collider::circle(npc.visual.size / 2.0),
                    CollisionLayers::new(GameLayer::Npc, [GameLayer::Player, GameLayer::Enemy]),
                ));
                info!("Spawned NPC Quest Giver: {}", npc.name);
            }
            "Trainer" => {
                commands.spawn((
                    Replicated,
                    Npc,
                    NpcName(npc.name.clone()),
                    Trainer {
                        items_for_sale: npc.trainer_items.iter().map(|ti| TrainerItem {
                            item_id: ti.item_id,
                            cost: ti.cost,
                        }).collect(),
                        trainer_type: npc.trainer_type.clone(),
                        teaching_quests: npc.teaching_quests.clone(),
                    },
                    Position(position),
                    Interactable::npc(),
                    VisualShape {
                        shape_type: ShapeType::Circle,
                        color: npc.visual.color,
                        size: npc.visual.size,
                    },
                    PhysicsPosition(position),
                    RigidBody::Static,
                    Collider::circle(npc.visual.size / 2.0),
                    CollisionLayers::new(GameLayer::Npc, [GameLayer::Player, GameLayer::Enemy]),
                ));
                info!("Spawned NPC Trainer: {}", npc.name);
            }
            _ => {
                warn!("Unknown NPC type: {}", npc.npc_type);
            }
        }
    }
}

/// Spawn enemies from zone definition
fn spawn_zone_enemies(
    commands: &mut Commands,
    zone: &crate::game_data::ZoneDefinition,
    enemy_db: &EnemyDatabase,
) {
    for region in &zone.enemy_spawns {
        if let Some(def) = enemy_db.enemies.get(&region.enemy_type) {
            // Parse shape type from enemy definition
            let shape_type = match def.visual.shape.as_str() {
                "Square" | "Rectangle" => ShapeType::Square,
                _ => ShapeType::Circle,
            };

            for spawn_point in &region.spawn_points {
                let position = Vec2::from(*spawn_point);

                let enemy_entity = commands.spawn((
                    Replicated,
                    Enemy,
                    EnemyType(region.enemy_type),
                    EnemyName(def.name.clone()),
                    Position(position),
                    Velocity::default(),
                    MoveSpeed(def.move_speed),
                    Health::new(def.max_health),
                    CombatStats {
                        attack_power: def.attack_power,
                        defense: def.defense,
                        crit_chance: 0.0,
                    },
                    BaseStats::new(def.attack_power, def.defense, def.move_speed),
                    CurrentTarget::default(),
                )).id();

                commands.entity(enemy_entity).insert((
                    AiState::default(),
                    Interactable::enemy(),
                    VisualShape {
                        shape_type,
                        color: def.visual.color,
                        size: def.visual.size,
                    },
                    AbilityCooldowns::default(),
                    SpawnPoint {
                        position,
                        respawn_delay: def.respawn_delay,
                    },
                    def.loot_table.clone(),
                    AiActivationDelay::default(),
                    AggroRange {
                        aggro: def.aggro_range,
                        leash: def.leash_range,
                    },
                ));

                // Physics components
                commands.entity(enemy_entity).insert((
                    PhysicsPosition(position),
                    PhysicsVelocity(Vec2::ZERO),
                    RigidBody::Dynamic,
                    Collider::circle(def.visual.size / 2.0),
                    CollisionLayers::new(GameLayer::Enemy, [GameLayer::Player, GameLayer::Npc]),
                ));
            }
            info!("Spawned {} {} enemies in region: {}",
                region.spawn_points.len(), def.name, region.region_id);
        } else {
            warn!("Enemy type {} not found in database for region: {}",
                region.enemy_type, region.region_id);
        }
    }
}

/// Spawn tilemap collision entities from zone tilemap data
fn spawn_tilemap_collision(commands: &mut Commands, zone: &crate::game_data::ZoneDefinition) {
    let Some(tilemap) = &zone.tilemap else {
        info!("No tilemap data for zone, skipping collision spawning");
        return;
    };

    let tile_size = tilemap.tile_size as f32;
    let chunk_size = tilemap.chunk_size as i32;
    let mut collision_count = 0;

    for (chunk_key, chunk) in &tilemap.chunks {
        let Some((chunk_x, chunk_y)) = eryndor_shared::ZoneTilemap::parse_chunk_key(chunk_key) else {
            continue;
        };

        // Iterate through collision layer
        for (row_idx, row) in chunk.collision.iter().enumerate() {
            for (col_idx, &is_blocked) in row.iter().enumerate() {
                if is_blocked == 0 {
                    continue;
                }

                // Calculate world position for this tile
                let world_x = (chunk_x * chunk_size + col_idx as i32) as f32 * tile_size + (tile_size / 2.0);
                let world_y = (chunk_y * chunk_size + row_idx as i32) as f32 * tile_size + (tile_size / 2.0);

                // Spawn collision entity
                commands.spawn((
                    TilemapCollider,
                    PhysicsPosition(Vec2::new(world_x, world_y)),
                    RigidBody::Static,
                    Collider::rectangle(tile_size, tile_size),
                    CollisionLayers::new(GameLayer::Environment, [GameLayer::Player, GameLayer::Enemy]),
                ));
                collision_count += 1;
            }
        }
    }

    if collision_count > 0 {
        info!("Spawned {} tilemap collision entities for zone: {}", collision_count, zone.zone_id);
    }
}
