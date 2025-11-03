use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;

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
        VisualShape {
            shape_type: ShapeType::Circle,
            color: COLOR_NPC,
            size: NPC_SIZE,
        },
    ));

    info!("Spawned NPC: Elder");

    // Note: Weapons are now given as quest rewards, not spawned in the world

    // Spawn enemies
    for (i, pos) in [ENEMY_SPAWN_1, ENEMY_SPAWN_2, ENEMY_SPAWN_3].iter().enumerate() {
        commands.spawn((
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
            AiState::default(),
            VisualShape {
                shape_type: ShapeType::Circle,
                color: COLOR_ENEMY,
                size: ENEMY_SIZE,
            },
            AbilityCooldowns::default(),
        ));
    }

    info!("Spawned 3 enemies");
    info!("World initialization complete!");
}
