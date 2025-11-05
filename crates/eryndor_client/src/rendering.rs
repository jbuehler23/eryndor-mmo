use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use eryndor_shared::*;
use crate::game_state::MyClientState;

/// Marker for visual representation entities
#[derive(Component)]
pub struct VisualEntity {
    pub game_entity: Entity,
}

/// Marker for name label entities
#[derive(Component)]
pub struct NameLabel {
    pub game_entity: Entity,
}

pub fn spawn_visual_entities(
    mut commands: Commands,
    query: Query<(Entity, &VisualShape, &Position), Without<VisualEntity>>,
    visual_query: Query<&VisualEntity>,
) {
    for (game_entity, visual_shape, position) in &query {
        // Check if visual already exists
        let already_has_visual = visual_query.iter().any(|v| v.game_entity == game_entity);
        if already_has_visual {
            continue;
        }

        // Create visual entity with the new API
        let color = Color::srgba(
            visual_shape.color[0],
            visual_shape.color[1],
            visual_shape.color[2],
            visual_shape.color[3],
        );

        // Set initial transform to match game entity position
        let initial_transform = Transform::from_translation(Vec3::new(position.0.x, position.0.y, 0.0));

        let mut entity_commands = commands.spawn((
            VisualEntity { game_entity },
            initial_transform,
        ));

        match visual_shape.shape_type {
            ShapeType::Circle => {
                let circle = shapes::Circle {
                    center: Vec2::ZERO,
                    radius: visual_shape.size / 2.0,
                };
                entity_commands.insert(ShapeBuilder::with(&circle).fill(color).build());
            }
            ShapeType::Triangle => {
                let points = vec![
                    Vec2::new(0.0, visual_shape.size / 2.0),
                    Vec2::new(-visual_shape.size / 2.0, -visual_shape.size / 2.0),
                    Vec2::new(visual_shape.size / 2.0, -visual_shape.size / 2.0),
                ];
                let polygon = shapes::Polygon {
                    points,
                    closed: true,
                };
                entity_commands.insert(ShapeBuilder::with(&polygon).fill(color).build());
            }
            ShapeType::Square => {
                let rect = shapes::Rectangle {
                    extents: Vec2::new(visual_shape.size, visual_shape.size),
                    origin: RectangleOrigin::Center,
                    ..default()
                };
                entity_commands.insert(ShapeBuilder::with(&rect).fill(color).build());
            }
            ShapeType::Diamond => {
                let points = vec![
                    Vec2::new(0.0, visual_shape.size / 2.0),
                    Vec2::new(visual_shape.size / 2.0, 0.0),
                    Vec2::new(0.0, -visual_shape.size / 2.0),
                    Vec2::new(-visual_shape.size / 2.0, 0.0),
                ];
                let polygon = shapes::Polygon {
                    points,
                    closed: true,
                };
                entity_commands.insert(ShapeBuilder::with(&polygon).fill(color).build());
            }
        }
    }
}

pub fn update_visual_positions(
    game_entities: Query<(Entity, &Position), Changed<Position>>,
    mut visual_entities: Query<(&VisualEntity, &mut Transform)>,
) {
    for (visual, mut transform) in &mut visual_entities {
        if let Ok((_entity, position)) = game_entities.get(visual.game_entity) {
            transform.translation = Vec3::new(position.0.x, position.0.y, 0.0);
        }
    }
}

pub fn spawn_name_labels(
    mut commands: Commands,
    player_query: Query<(Entity, &Character), (With<Player>, Without<NameLabel>)>,
    npc_query: Query<(Entity, &NpcName), (With<Npc>, Without<NameLabel>)>,
    label_query: Query<&NameLabel>,
) {
    // Spawn labels for players (white)
    for (game_entity, character) in &player_query {
        // Check if name label already exists
        let already_has_label = label_query.iter().any(|l| l.game_entity == game_entity);
        if already_has_label {
            continue;
        }

        // Spawn name label text entity
        commands.spawn((
            NameLabel { game_entity },
            Text2d::new(character.name.clone()),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    }

    // Spawn labels for NPCs (gold/yellow)
    for (game_entity, npc_name) in &npc_query {
        // Check if name label already exists
        let already_has_label = label_query.iter().any(|l| l.game_entity == game_entity);
        if already_has_label {
            continue;
        }

        // Spawn NPC name label with gold color
        commands.spawn((
            NameLabel { game_entity },
            Text2d::new(npc_name.0.clone()),
            TextFont {
                font_size: 18.0, // Slightly larger for NPCs
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.84, 0.0)), // Gold color
        ));
    }
}

pub fn update_name_label_positions(
    player_entities: Query<(Entity, &Position), With<Player>>,
    npc_entities: Query<(Entity, &Position), With<Npc>>,
    mut label_entities: Query<(&NameLabel, &mut Transform)>,
) {
    for (label, mut transform) in &mut label_entities {
        // Try to find position from either players or NPCs
        let position = player_entities
            .get(label.game_entity)
            .map(|(_, pos)| pos)
            .or_else(|_| npc_entities.get(label.game_entity).map(|(_, pos)| pos));

        if let Ok(position) = position {
            // Position name label above the entity
            transform.translation = Vec3::new(position.0.x, position.0.y + 25.0, 1.0);
        }
    }
}

pub fn cleanup_despawned_entities(
    mut commands: Commands,
    visual_entities: Query<(Entity, &VisualEntity)>,
    label_entities: Query<(Entity, &NameLabel)>,
    all_entities: Query<Entity>,
) {
    // Clean up visual entities whose game entity no longer exists
    for (visual_entity, visual) in &visual_entities {
        if all_entities.get(visual.game_entity).is_err() {
            commands.entity(visual_entity).despawn();
            info!("Despawned visual entity for game entity {:?}", visual.game_entity);
        }
    }

    // Clean up name labels whose game entity no longer exists
    for (label_entity, label) in &label_entities {
        if all_entities.get(label.game_entity).is_err() {
            commands.entity(label_entity).despawn();
            info!("Despawned name label for game entity {:?}", label.game_entity);
        }
    }
}

pub fn update_camera_follow(
    client_state: Res<MyClientState>,
    player_query: Query<&Position, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Some(player_entity) = client_state.player_entity else {
        return
    };

    // Silently wait for entity to be replicated
    if let Ok(position) = player_query.get(player_entity) {
        if let Ok(mut camera_transform) = camera_query.single_mut() {
            camera_transform.translation = Vec3::new(position.0.x, position.0.y, camera_transform.translation.z);
        }
    }
}

/// Marker component for debug grid entities
#[derive(Component)]
pub struct DebugGrid;

pub fn spawn_debug_grid(
    mut commands: Commands,
    existing: Query<Entity, With<DebugGrid>>,
) {
    // Only spawn once
    if !existing.is_empty() {
        return;
    }

    let grid_spacing = 50.0;
    let grid_extent = 500.0;
    let num_lines = (grid_extent / grid_spacing * 2.0) as i32 + 1;

    let grid_color = Color::srgba(0.3, 0.3, 0.3, 0.5);
    let axis_color = Color::srgba(0.5, 0.5, 0.5, 0.8);

    // Draw vertical lines
    for i in -(num_lines/2)..=(num_lines/2) {
        let x = i as f32 * grid_spacing;
        let color = if i == 0 { axis_color } else { grid_color };

        let line = shapes::Line(
            Vec2::new(x, -grid_extent),
            Vec2::new(x, grid_extent),
        );
        let stroke = Stroke::new(color, if i == 0 { 2.0 } else { 1.0 });
        commands.spawn((
            DebugGrid,
            ShapeBuilder::with(&line)
                .stroke(stroke)
                .build(),
        ));
    }

    // Draw horizontal lines
    for i in -(num_lines/2)..=(num_lines/2) {
        let y = i as f32 * grid_spacing;
        let color = if i == 0 { axis_color } else { grid_color };

        let line = shapes::Line(
            Vec2::new(-grid_extent, y),
            Vec2::new(grid_extent, y),
        );
        let stroke = Stroke::new(color, if i == 0 { 2.0 } else { 1.0 });
        commands.spawn((
            DebugGrid,
            ShapeBuilder::with(&line)
                .stroke(stroke)
                .build(),
        ));
    }
}

pub fn draw_debug_labels(
    mut gizmos: Gizmos,
) {
    // Draw coordinate markers at key positions
    // Draw a small circle at origin
    gizmos.circle_2d(Vec2::ZERO, 5.0, Color::srgba(1.0, 1.0, 0.0, 0.8));

    // Draw markers every 50 pixels on axes
    for i in -10..=10 {
        if i == 0 { continue; }
        let pos = i as f32 * 50.0;

        // X-axis markers
        gizmos.circle_2d(Vec2::new(pos, 0.0), 3.0, Color::srgba(0.8, 0.8, 0.8, 0.6));

        // Y-axis markers
        gizmos.circle_2d(Vec2::new(0.0, pos), 3.0, Color::srgba(0.8, 0.8, 0.8, 0.6));
    }
}

/// Draw target indicator circles around selected entities
pub fn draw_target_indicator(
    mut gizmos: Gizmos,
    input_state: Res<crate::input::InputState>,
    npc_query: Query<(&Position, &VisualShape), With<Npc>>,
    enemy_query: Query<(&Position, &VisualShape), With<Enemy>>,
) {
    let Some(target) = input_state.selected_target else {
        return;
    };

    // Check if target is an NPC
    if let Ok((position, visual)) = npc_query.get(target) {
        let radius = (visual.size / 2.0) + 5.0; // Slightly larger than entity
        gizmos.circle_2d(
            position.0,
            radius,
            Color::srgba(0.0, 1.0, 0.0, 0.4), // Green with 40% alpha
        );
    }

    // Check if target is an Enemy
    if let Ok((position, visual)) = enemy_query.get(target) {
        let radius = (visual.size / 2.0) + 5.0; // Slightly larger than entity
        gizmos.circle_2d(
            position.0,
            radius,
            Color::srgba(1.0, 0.0, 0.0, 0.4), // Red with 40% alpha
        );
    }
}
