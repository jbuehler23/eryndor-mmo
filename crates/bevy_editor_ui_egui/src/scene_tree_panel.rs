//! Scene tree panel for viewing and editing the entity hierarchy

use crate::icons::Icons;
use bevy::prelude::*;
use bevy_editor_frontend_api::scene_tree::{SceneEntityTemplate, SceneTreeCommand, SceneTreeNode};
use bevy_editor_scene::{EditorScene, EditorSceneEntity};
use bevy_egui::egui;

/// Render the scene tree panel content
pub fn render_scene_tree_panel(
    ui: &mut egui::Ui,
    editor_scene: &mut EditorScene,
    entity_nodes: &[SceneTreeNode],
    events: &mut bevy::prelude::EventWriter<SceneTreeCommand>,
) {
    ui.heading("Scene Tree");
    ui.separator();

    // Add entity button with menu
    ui.horizontal(|ui| {
        // Menu button for adding different entity types
        ui.menu_button(format!("{} Add Entity", Icons::NEW), |ui| {
            ui.label("Choose entity type to add:");
            ui.separator();

            // List all entity templates
            for template in SceneEntityTemplate::ALL {
                if matches!(template, SceneEntityTemplate::Camera2D) {
                    continue;
                }

                if ui.button(template.display_name()).clicked() {
                    events.write(SceneTreeCommand::AddTemplateEntity {
                        template,
                        parent: editor_scene.selected_entity,
                    });
                    info!("Add {} entity command sent", template.display_name());
                    ui.close();
                }
            }
        });

        if let Some(selected) = editor_scene.selected_entity {
            if ui.button(format!("{} Delete", Icons::CLOSE)).clicked() {
                events.write(SceneTreeCommand::DeleteEntity { entity: selected });
                info!("Delete entity command sent: {:?}", selected);
            }
        }
    });

    ui.separator();

    // Render the entity tree
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            if let Some(root_entity) = editor_scene.root_entity {
                // Debug: show entity count
                ui.label(format!("Entities: {}", entity_nodes.len()));
                render_entity_node(ui, root_entity, editor_scene, entity_nodes, 0);

                // Debug: Show all entities (temporary for debugging)
                ui.separator();
                ui.label("All EditorSceneEntity instances:");
                for data in entity_nodes {
                    if data.entity != root_entity {
                        ui.label(format!(
                            "  {:?}: {} (children: {})",
                            data.entity,
                            data.name,
                            data.children.len()
                        ));
                    }
                }
            } else {
                ui.label("No scene loaded - root_entity is None");
            }
        });
}

/// Recursively render an entity and its children
fn render_entity_node(
    ui: &mut egui::Ui,
    entity: Entity,
    editor_scene: &mut EditorScene,
    entity_nodes: &[SceneTreeNode],
    depth: usize,
) {
    // Find this entity's data
    let Some(data) = entity_nodes.iter().find(|d| d.entity == entity) else {
        return;
    };

    // Indentation for hierarchy
    let indent = depth as f32 * 16.0;
    ui.add_space(indent);

    // Entity row
    ui.horizontal(|ui| {
        // Expand/collapse icon (if has children)
        if data.has_children {
            ui.label(Icons::CHEVRON_DOWN);
        } else {
            ui.add_space(12.0); // Space for alignment
        }

        // Entity icon
        ui.label(Icons::NODE);

        // Entity name (selectable)
        let is_selected = editor_scene.is_selected(entity);
        let response = ui.selectable_label(is_selected, &data.name);

        if response.clicked() {
            editor_scene.select_entity(entity);
            info!("Selected entity: {} ({:?})", data.name, entity);
        }

        // Show entity ID on hover
        response.on_hover_text(format!("Entity ID: {:?}", entity));
    });

    // Render children recursively
    if data.has_children {
        for child in &data.children {
            render_entity_node(ui, *child, editor_scene, entity_nodes, depth + 1);
        }
    }
}

/// System to handle scene tree commands
pub fn handle_scene_tree_commands(
    mut commands: Commands,
    mut events: EventReader<SceneTreeCommand>,
    mut editor_scene: ResMut<EditorScene>,
) {
    for event in events.read() {
        match event {
            SceneTreeCommand::AddTemplateEntity { template, parent } => {
                let entity =
                    crate::entity_templates::spawn_from_template(&mut commands, *template, *parent);

                // Set parent if not already done by template
                if parent.is_none() {
                    if let Some(root) = editor_scene.root_entity {
                        info!("Parenting new entity {:?} to scene root {:?}", entity, root);
                        commands.entity(entity).insert(ChildOf(root));
                    } else {
                        warn!(
                            "No parent for new entity {:?} - root_entity is None!",
                            entity
                        );
                    }
                }

                editor_scene.select_entity(entity);
                editor_scene.mark_modified();
                info!("Added new {:?} entity: {:?}", template, entity);
            }

            SceneTreeCommand::AddEntity { parent } => {
                let entity = commands
                    .spawn((
                        Name::new("New Entity"),
                        Transform::default(),
                        Visibility::default(),
                        Sprite {
                            color: Color::srgba(0.7, 0.7, 0.7, 0.8), // Gray semi-transparent
                            custom_size: Some(Vec2::new(64.0, 64.0)),
                            ..default()
                        },
                        EditorSceneEntity,
                    ))
                    .id();

                // Set parent if specified
                if let Some(parent_entity) = parent {
                    info!(
                        "Parenting new entity {:?} to selected parent {:?}",
                        entity, parent_entity
                    );
                    commands.entity(entity).insert(ChildOf(*parent_entity));
                } else if let Some(root) = editor_scene.root_entity {
                    info!("Parenting new entity {:?} to scene root {:?}", entity, root);
                    commands.entity(entity).insert(ChildOf(root));
                } else {
                    warn!(
                        "No parent for new entity {:?} - root_entity is None!",
                        entity
                    );
                }

                editor_scene.select_entity(entity);
                editor_scene.mark_modified();
                info!("Added new entity: {:?}", entity);
            }

            SceneTreeCommand::DeleteEntity { entity } => {
                commands.entity(*entity).despawn();
                editor_scene.clear_selection();
                editor_scene.mark_modified();
                info!("Deleted entity: {:?}", entity);
            }

            SceneTreeCommand::RenameEntity { entity, new_name } => {
                commands.entity(*entity).insert(Name::new(new_name.clone()));
                editor_scene.mark_modified();
                info!("Renamed entity {:?} to: {}", entity, new_name);
            }

            SceneTreeCommand::ReparentEntity { entity, new_parent } => {
                if let Some(parent) = new_parent {
                    commands.entity(*entity).insert(ChildOf(*parent));
                } else {
                    commands.entity(*entity).remove::<ChildOf>();
                }
                editor_scene.mark_modified();
                info!("Reparented entity {:?} to {:?}", entity, new_parent);
            }
        }
    }
}
