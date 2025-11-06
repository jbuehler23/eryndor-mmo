use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use avian2d::prelude::{RigidBody, Collider, CollisionLayers};
use crate::{PhysicsPosition, PhysicsVelocity};

pub fn spawn_world(mut commands: Commands) {
    info!("Spawning world entities...");

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

    // Note: Weapons are now given as quest rewards, not spawned in the world

    // Spawn enemies with respawn points
    for (i, pos) in [ENEMY_SPAWN_1, ENEMY_SPAWN_2, ENEMY_SPAWN_3].iter().enumerate() {
        let enemy_entity = commands.spawn((
            Replicated,
            Enemy,
            EnemyType(ENEMY_TYPE_SLIME),
            Position(*pos),
            Velocity::default(),
            MoveSpeed(80.0),
            Health::new(50.0),
            CombatStats {
                attack_power: 5.0,
                defense: 2.0,
                crit_chance: 0.0,
            },
            CurrentTarget::default(),
        )).id();

        commands.entity(enemy_entity).insert((
            AiState::default(),
            Interactable::enemy(),
            VisualShape {
                shape_type: ShapeType::Circle,
                color: COLOR_ENEMY,
                size: ENEMY_SIZE,
            },
            AbilityCooldowns::default(),
            // Add spawn point for respawn system
            crate::spawn::SpawnPoint {
                position: *pos,
                respawn_delay: 30.0, // 30 seconds
            },
        ));

        // Physics components
        commands.entity(enemy_entity).insert((
            PhysicsPosition(*pos),
            PhysicsVelocity::default(),
            RigidBody::Dynamic,
            Collider::circle(ENEMY_SIZE / 2.0),
            CollisionLayers::new(GameLayer::Enemy, [GameLayer::Player, GameLayer::Npc, GameLayer::Enemy, GameLayer::Environment]),
        ));

        info!("Spawned enemy #{} at {:?} with entity ID: {:?}", i + 1, pos, enemy_entity);
    }

    info!("Finished spawning 3 enemies");

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
