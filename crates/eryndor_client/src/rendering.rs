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
    query: Query<(Entity, &VisualShape), Without<VisualEntity>>,
    visual_query: Query<&VisualEntity>,
) {
    for (game_entity, visual_shape) in &query {
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

        let mut entity_commands = commands.spawn(VisualEntity { game_entity });

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
    label_query: Query<&NameLabel>,
) {
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
}

pub fn update_name_label_positions(
    game_entities: Query<(Entity, &Position), With<Player>>,
    mut label_entities: Query<(&NameLabel, &mut Transform)>,
) {
    for (label, mut transform) in &mut label_entities {
        if let Ok((_entity, position)) = game_entities.get(label.game_entity) {
            // Position name label above the character
            transform.translation = Vec3::new(position.0.x, position.0.y + 25.0, 1.0);
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
