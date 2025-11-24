use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use avian2d::prelude::{RigidBody, Collider, CollisionLayers};
use crate::{PhysicsPosition, PhysicsVelocity};
use crate::game_data::{EnemyDatabase, ZoneDatabase};
use crate::spawn::SpawnPoint;

pub fn spawn_world(
    mut commands: Commands,
    enemy_db: Res<EnemyDatabase>,
    zone_db: Res<ZoneDatabase>,
) {
    info!("Spawning world entities...");

    // Try to spawn from zone data first
    if let Some(zone) = zone_db.zones.get("starter_zone") {
        info!("Spawning world from zone: {}", zone.zone_name);
        spawn_zone_npcs(&mut commands, zone);
        spawn_zone_enemies(&mut commands, zone, &enemy_db);
        return;
    }

    // Fallback to hardcoded spawns if zone data not loaded yet
    warn!("Zone data not loaded, using hardcoded spawns");
    spawn_hardcoded_world(&mut commands, &enemy_db);
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
                    CurrentTarget::default(),
                )).id();

                commands.entity(enemy_entity).insert((
                    AiState::default(),
                    Interactable::enemy(),
                    VisualShape {
                        shape_type: ShapeType::Circle,
                        color: region.visual.color,
                        size: region.visual.size,
                    },
                    AbilityCooldowns::default(),
                    SpawnPoint {
                        position,
                        respawn_delay: region.respawn_delay,
                    },
                    region.loot_table.clone(),
                ));

                // Physics components
                commands.entity(enemy_entity).insert((
                    PhysicsPosition(position),
                    PhysicsVelocity(Vec2::ZERO),
                    RigidBody::Dynamic,
                    Collider::circle(region.visual.size / 2.0),
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

/// Hardcoded fallback spawn function (legacy)
fn spawn_hardcoded_world(commands: &mut Commands, enemy_db: &EnemyDatabase) {
    // Spawn NPC Quest Giver
    commands.spawn((
        Replicated,
        Npc,
        NpcName("Elder".to_string()),
        QuestGiver {
            available_quests: vec![QUEST_FIRST_WEAPON],
        },
        Position(NPC_POSITION),
        Interactable::npc(),
        VisualShape {
            shape_type: ShapeType::Circle,
            color: COLOR_NPC,
            size: NPC_SIZE,
        },
        // Physics components - static NPC
        PhysicsPosition(NPC_POSITION),
        RigidBody::Static,
        Collider::circle(NPC_SIZE / 2.0),
        CollisionLayers::new(GameLayer::Npc, [GameLayer::Player, GameLayer::Enemy]),
    ));

    info!("Spawned NPC: Elder");

    // Spawn Weapon Master Trainer
    commands.spawn((
        Replicated,
        Npc,
        NpcName("Weapon Master".to_string()),
        Trainer {
            items_for_sale: vec![
                TrainerItem { item_id: ITEM_DAGGER, cost: 50 },
                TrainerItem { item_id: ITEM_SWORD, cost: 75 },
                TrainerItem { item_id: ITEM_WAND, cost: 100 },
                TrainerItem { item_id: ITEM_STAFF, cost: 150 },
                TrainerItem { item_id: ITEM_MACE, cost: 125 },
                TrainerItem { item_id: ITEM_BOW, cost: 100 },
                TrainerItem { item_id: ITEM_AXE, cost: 125 },
            ],
        },
        Position(Vec2::new(60.0, -20.0)), // To the right of the Elder
        Interactable::npc(),
        VisualShape {
            shape_type: ShapeType::Circle,
            color: [0.8, 0.6, 0.2, 1.0], // Orange/bronze color for trainer
            size: NPC_SIZE,
        },
        // Physics components - static NPC
        PhysicsPosition(Vec2::new(60.0, -20.0)),
        RigidBody::Static,
        Collider::circle(NPC_SIZE / 2.0),
        CollisionLayers::new(GameLayer::Npc, [GameLayer::Player, GameLayer::Enemy]),
    ));

    info!("Spawned NPC: Weapon Master");

    // Note: Weapons are now given as quest rewards, not spawned in the world

    // Helper function to spawn enemies using the database
    let spawn_enemy = |commands: &mut Commands, enemy_type_id: u32, position: Vec2, shape: ShapeType, color: [f32; 4], loot_table: LootTable| {
        if let Some(def) = enemy_db.enemies.get(&enemy_type_id) {
            let enemy_entity = commands.spawn((
                Replicated,
                Enemy,
                EnemyType(enemy_type_id),
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
                CurrentTarget::default(),
            )).id();

            commands.entity(enemy_entity).insert((
                AiState::default(),
                Interactable::enemy(),
                VisualShape {
                    shape_type: shape,
                    color,
                    size: ENEMY_SIZE,
                },
                AbilityCooldowns::default(),
                crate::spawn::SpawnPoint {
                    position,
                    respawn_delay: 10.0, // 10 seconds
                },
                loot_table,
            ));

            // Physics components
            commands.entity(enemy_entity).insert((
                PhysicsPosition(position),
                PhysicsVelocity::default(),
                RigidBody::Dynamic,
                Collider::circle(ENEMY_SIZE / 2.0),
                CollisionLayers::new(GameLayer::Enemy, [GameLayer::Player, GameLayer::Npc, GameLayer::Enemy, GameLayer::Environment]),
            ));

            info!("Spawned {} at {:?}", def.name, position);
        }
    };

    info!("Spawning enemies throughout the world...");
    let mut total_enemies = 0;

    // ========== STARTER AREA - Slimes (Level 1) ==========
    // Northeast starter area - 4 slimes
    let slime_positions = vec![
        Vec2::new(150.0, 150.0),
        Vec2::new(200.0, 180.0),
        Vec2::new(180.0, 120.0),
        Vec2::new(220.0, 150.0),
    ];
    for pos in slime_positions {
        spawn_enemy(commands, ENEMY_TYPE_SLIME, pos, ShapeType::Circle, [0.2, 0.8, 0.2, 1.0], LootTable {
            gold_min: 3,
            gold_max: 8,
            items: vec![],
        });
        total_enemies += 1;
    }

    // ========== GOBLIN CAMP - Goblins (Level 2) ==========
    // Northwest area - 5 goblins
    let goblin_positions = vec![
        Vec2::new(-200.0, 200.0),
        Vec2::new(-250.0, 220.0),
        Vec2::new(-180.0, 180.0),
        Vec2::new(-220.0, 250.0),
        Vec2::new(-200.0, 150.0),
    ];
    for pos in goblin_positions {
        spawn_enemy(commands, ENEMY_TYPE_GOBLIN, pos, ShapeType::Square, [0.4, 0.6, 0.2, 1.0], LootTable {
            gold_min: 8,
            gold_max: 15,
            items: vec![],
        });
        total_enemies += 1;
    }

    // ========== WOLF PACK - Wolves (Level 3) ==========
    // East forest area - 4 wolves
    let wolf_positions = vec![
        Vec2::new(400.0, 50.0),
        Vec2::new(450.0, 80.0),
        Vec2::new(420.0, -20.0),
        Vec2::new(480.0, 40.0),
    ];
    for pos in wolf_positions {
        spawn_enemy(commands, ENEMY_TYPE_WOLF, pos, ShapeType::Circle, [0.6, 0.5, 0.3, 1.0], LootTable {
            gold_min: 12,
            gold_max: 20,
            items: vec![],
        });
        total_enemies += 1;
    }

    // ========== SPIDER DEN - Spiders (Level 3) ==========
    // Southwest corner - 5 spiders
    let spider_positions = vec![
        Vec2::new(-400.0, -300.0),
        Vec2::new(-450.0, -280.0),
        Vec2::new(-380.0, -350.0),
        Vec2::new(-420.0, -320.0),
        Vec2::new(-460.0, -340.0),
    ];
    for pos in spider_positions {
        spawn_enemy(commands, ENEMY_TYPE_SPIDER, pos, ShapeType::Circle, [0.3, 0.1, 0.3, 1.0], LootTable {
            gold_min: 10,
            gold_max: 18,
            items: vec![],
        });
        total_enemies += 1;
    }

    // ========== GRAVEYARD - Skeletons (Level 4) ==========
    // South central area - 4 skeletons
    let skeleton_positions = vec![
        Vec2::new(0.0, -400.0),
        Vec2::new(50.0, -420.0),
        Vec2::new(-50.0, -380.0),
        Vec2::new(20.0, -450.0),
    ];
    for pos in skeleton_positions {
        spawn_enemy(commands, ENEMY_TYPE_SKELETON, pos, ShapeType::Square, [0.9, 0.9, 0.8, 1.0], LootTable {
            gold_min: 15,
            gold_max: 25,
            items: vec![],
        });
        total_enemies += 1;
    }

    // ========== ORC STRONGHOLD - Orcs (Level 5) ==========
    // Far north area - 3 orcs (stronger enemies, fewer spawns)
    let orc_positions = vec![
        Vec2::new(0.0, 500.0),
        Vec2::new(-80.0, 520.0),
        Vec2::new(80.0, 480.0),
    ];
    for pos in orc_positions {
        spawn_enemy(commands, ENEMY_TYPE_ORC, pos, ShapeType::Square, [0.3, 0.5, 0.3, 1.0], LootTable {
            gold_min: 20,
            gold_max: 35,
            items: vec![],
        });
        total_enemies += 1;
    }

    info!("Finished spawning {} enemies across the world", total_enemies);

    // Spawn world boundaries - visible red walls
    let wall_thickness = 20.0;
    let half_width = WORLD_WIDTH / 2.0;
    let half_height = WORLD_HEIGHT / 2.0;
    let segment_size = 50.0; // Size of each visual segment

    // Offset colliders so they align with the inner edge of visual squares
    // Visual squares are centered at boundary, extending segment_size/2 inward and outward
    // We want collider at the inner edge, so offset by (segment_size/2 - wall_thickness/2)
    let collider_offset = (segment_size / 2.0) - (wall_thickness / 2.0);

    // North wall (top) - horizontal wall
    // Place collider at inner edge of visual boundary
    commands.spawn((
        PhysicsPosition(Vec2::new(0.0, half_height - collider_offset)),
        RigidBody::Static,
        Collider::rectangle(WORLD_WIDTH, wall_thickness),
        CollisionLayers::new(GameLayer::Environment, [GameLayer::Player, GameLayer::Enemy]),
    ));
    // Spawn visual segments along the wall
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

    // South wall (bottom) - horizontal wall
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

    // East wall (right) - vertical wall
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

    // West wall (left) - vertical wall
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

    info!("Spawned world boundaries");
    info!("World initialization complete!");
}
