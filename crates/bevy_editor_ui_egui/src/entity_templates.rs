//! Entity templates for creating pre-configured entities.

use bevy::prelude::*;
use bevy_editor_frontend_api::scene_tree::SceneEntityTemplate;
use bevy_editor_scene::EditorSceneEntity;

/// Spawn an entity from a template with the specified parent.
pub fn spawn_from_template(
    commands: &mut Commands,
    template: SceneEntityTemplate,
    parent: Option<Entity>,
) -> Entity {
    match template {
        SceneEntityTemplate::Empty => spawn_empty(commands, parent),
        SceneEntityTemplate::Sprite => spawn_sprite(commands, parent),
        SceneEntityTemplate::Camera2D => spawn_camera_2d(commands, parent),
        SceneEntityTemplate::UiNode => spawn_ui_node(commands, parent),
        SceneEntityTemplate::Button => spawn_button(commands, parent),
        SceneEntityTemplate::Text => spawn_text(commands, parent),
    }
}

fn spawn_empty(commands: &mut Commands, parent: Option<Entity>) -> Entity {
    spawn_with_parent(
        commands,
        (
            Name::new(SceneEntityTemplate::Empty.default_name()),
            Transform::default(),
            Visibility::default(),
            EditorSceneEntity,
        ),
        parent,
    )
}

fn spawn_sprite(commands: &mut Commands, parent: Option<Entity>) -> Entity {
    let sprite = Sprite::from_color(Color::srgba(0.7, 0.7, 0.7, 0.8), Vec2::splat(64.0));

    spawn_with_parent(
        commands,
        (
            Name::new(SceneEntityTemplate::Sprite.default_name()),
            sprite,
            Transform::default(),
            EditorSceneEntity,
        ),
        parent,
    )
}

fn spawn_camera_2d(commands: &mut Commands, parent: Option<Entity>) -> Entity {
    spawn_with_parent(
        commands,
        (
            Name::new(SceneEntityTemplate::Camera2D.default_name()),
            Transform::default(),
            Visibility::default(),
            Camera2d,
            EditorSceneEntity,
        ),
        parent,
    )
}

fn spawn_ui_node(commands: &mut Commands, parent: Option<Entity>) -> Entity {
    spawn_with_parent(
        commands,
        (
            Name::new(SceneEntityTemplate::UiNode.default_name()),
            Node {
                width: Val::Px(200.0),
                height: Val::Px(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.8)),
            EditorSceneEntity,
        ),
        parent,
    )
}

fn spawn_button(commands: &mut Commands, parent: Option<Entity>) -> Entity {
    let button_id = spawn_with_parent(
        commands,
        (
            Name::new(SceneEntityTemplate::Button.default_name()),
            Button,
            Node {
                width: Val::Px(150.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            EditorSceneEntity,
        ),
        parent,
    );

    commands
        .spawn((
            Text::new("Button"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            EditorSceneEntity,
        ))
        .insert(ChildOf(button_id));

    button_id
}

fn spawn_text(commands: &mut Commands, parent: Option<Entity>) -> Entity {
    spawn_with_parent(
        commands,
        (
            Name::new(SceneEntityTemplate::Text.default_name()),
            Text::new("Text"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
            EditorSceneEntity,
        ),
        parent,
    )
}

fn spawn_with_parent(
    commands: &mut Commands,
    bundle: impl Bundle,
    parent: Option<Entity>,
) -> Entity {
    let mut entity_commands = commands.spawn(bundle);

    if let Some(parent_entity) = parent {
        entity_commands.insert(ChildOf(parent_entity));
    }

    entity_commands.id()
}
