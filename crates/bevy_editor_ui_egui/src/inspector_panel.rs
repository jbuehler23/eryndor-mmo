//! Inspector panel for viewing and editing entity components

use bevy::prelude::*;
use bevy_editor_frontend_api::EntityComponentData;
use bevy_editor_scene::{EditorScene, NameEditEvent, SpriteTextureEvent, TransformEditEvent};
use bevy_egui::egui;

use crate::component_registry::ComponentRegistry;
use crate::icons::{IconLabel, Icons};

/// Render the inspector panel content
pub fn render_inspector_panel(
    ui: &mut egui::Ui,
    editor_scene: &EditorScene,
    component_data: Option<&EntityComponentData>,
    component_registry: &ComponentRegistry,
    transform_events: &mut MessageWriter<TransformEditEvent>,
    name_events: &mut MessageWriter<NameEditEvent>,
    name_edit_buffer: &mut String,
    project_root: Option<&std::path::PathBuf>,
    asset_server: &AssetServer,
    texture_events: &mut MessageWriter<SpriteTextureEvent>,
    sprite_texture_id: Option<egui::TextureId>,
    images: &Assets<Image>,
    add_component_events: &mut MessageWriter<bevy_editor_scene::AddComponentEvent>,
) {
    ui.heading("Inspector");
    ui.separator();

    // Check if an entity is selected
    let Some(selected_entity) = editor_scene.selected_entity else {
        ui.label("No entity selected");
        return;
    };

    // Check if we have component data
    let Some(data) = component_data else {
        ui.label("Entity not found");
        return;
    };

    // Entity header with inline name editing
    ui.horizontal(|ui| {
        ui.label(Icons::NODE);

        // Initialize buffer with current name
        if name_edit_buffer.is_empty() {
            *name_edit_buffer = data.name.clone().unwrap_or_else(|| "Unnamed".to_string());
        }

        let response = ui.text_edit_singleline(name_edit_buffer);

        // Send event when user finished editing (lost focus or pressed enter)
        if response.lost_focus() && !name_edit_buffer.is_empty() {
            name_events.write(NameEditEvent {
                entity: selected_entity,
                new_name: name_edit_buffer.clone(),
            });
        }
    });

    ui.label(format!("Entity ID: {:?}", selected_entity));
    ui.separator();

    // Scrollable area for components
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            // Show existing components
            render_existing_components(
                ui,
                data,
                selected_entity,
                transform_events,
                project_root,
                asset_server,
                texture_events,
                sprite_texture_id,
                images,
            );

            ui.separator();

            // Add Component button
            render_add_component_menu(ui, component_registry, selected_entity, add_component_events);
        });
}

/// Render existing components on the entity
fn render_existing_components(
    ui: &mut egui::Ui,
    data: &EntityComponentData,
    entity: Entity,
    transform_events: &mut MessageWriter<TransformEditEvent>,
    project_root: Option<&std::path::PathBuf>,
    asset_server: &AssetServer,
    texture_events: &mut MessageWriter<SpriteTextureEvent>,
    sprite_texture_id: Option<egui::TextureId>,
    images: &Assets<Image>,
) {
    // Transform component
    if let Some(transform) = &data.transform {
        render_transform_component(ui, transform, entity, transform_events);
    }

    // Name component
    if let Some(name) = &data.name {
        render_name_component(ui, name);
    }

    // Visibility component
    if let Some(visibility) = &data.visibility {
        render_visibility_component(ui, visibility);
    }

    // Sprite component
    if let Some(sprite) = &data.sprite {
        render_sprite_component(
            ui,
            sprite,
            entity,
            project_root,
            asset_server,
            texture_events,
            sprite_texture_id,
            images,
        );
    }

    // Camera2d component
    if data.has_camera2d {
        render_camera2d_component(ui);
    }

    // UI Node component
    if let Some(node) = &data.node {
        render_node_component(ui, node);
    }

    // Button component
    if data.has_button {
        render_button_component(ui);
    }

    // Text component
    if let Some(text) = &data.text {
        render_text_component(ui, text);
    }
}

fn render_transform_component(
    ui: &mut egui::Ui,
    transform: &Transform,
    entity: Entity,
    transform_events: &mut MessageWriter<TransformEditEvent>,
) {
    egui::CollapsingHeader::new(Icons::TRANSFORM.with_icon("Transform"))
        .default_open(true)
        .show(ui, |ui| {
            let mut translation = transform.translation;
            let rotation = transform.rotation.to_euler(EulerRot::XYZ);
            let mut rotation_deg = Vec3::new(
                rotation.0.to_degrees(),
                rotation.1.to_degrees(),
                rotation.2.to_degrees(),
            );
            let mut scale = transform.scale;

            // Translation
            ui.label(Icons::ARROW_UP.with_icon("Translation"));
            let changed_translation = edit_vec3(ui, &mut translation);

            // Rotation (convert to degrees for UI)
            ui.label(Icons::ARROW_RIGHT.with_icon("Rotation (deg)"));
            let changed_rotation = edit_vec3(ui, &mut rotation_deg);

            // Scale
            ui.label(Icons::ARROW_UP.with_icon("Scale"));
            let changed_scale = edit_vec3(ui, &mut scale);

            if changed_translation {
                transform_events.write(TransformEditEvent::SetPosition {
                    entity,
                    position: Vec2::new(translation.x, translation.y),
                });
            }

            if changed_rotation {
                transform_events.write(TransformEditEvent::SetRotation {
                    entity,
                    rotation: rotation_deg.z.to_radians(),
                });
            }

            if changed_scale {
                transform_events.write(TransformEditEvent::SetScale {
                    entity,
                    scale: Vec2::new(scale.x, scale.y),
                });
            }
        });
}

fn edit_vec3(ui: &mut egui::Ui, value: &mut Vec3) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        changed |= ui
            .add(egui::DragValue::new(&mut value.x).speed(0.1).prefix("X: "))
            .changed();
        changed |= ui
            .add(egui::DragValue::new(&mut value.y).speed(0.1).prefix("Y: "))
            .changed();
        changed |= ui
            .add(egui::DragValue::new(&mut value.z).speed(0.1).prefix("Z: "))
            .changed();
    });

    changed
}

fn render_name_component(ui: &mut egui::Ui, name: &str) {
    egui::CollapsingHeader::new(Icons::INFO.with_icon("Name"))
        .default_open(true)
        .show(ui, |ui| {
            ui.label(format!("Current name: {}", name));
            ui.label("(Rename via the panel header)");
        });
}

fn render_visibility_component(ui: &mut egui::Ui, visibility: &Visibility) {
    egui::CollapsingHeader::new(Icons::EYE.with_icon("Visibility"))
        .default_open(true)
        .show(ui, |ui| {
            ui.label(format!("Visible: {}", visibility == &Visibility::Visible));
        });
}

fn render_sprite_component(
    ui: &mut egui::Ui,
    sprite: &Sprite,
    entity: Entity,
    project_root: Option<&std::path::PathBuf>,
    asset_server: &AssetServer,
    texture_events: &mut MessageWriter<SpriteTextureEvent>,
    sprite_texture_id: Option<egui::TextureId>,
    _images: &Assets<Image>,
) {
    egui::CollapsingHeader::new(Icons::SPRITE.with_icon("Sprite"))
        .default_open(true)
        .show(ui, |ui| {
            if let Some(texture_id) = sprite_texture_id {
                ui.image((texture_id, egui::vec2(64.0, 64.0)));
            } else {
                ui.label("No texture assigned");
            }

            if ui
                .button(Icons::FOLDER_OPEN.with_icon("Assign Texture"))
                .clicked()
            {
                if let Some(project_root) = project_root {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_directory(project_root)
                        .pick_file()
                    {
                        let texture_path = path.to_string_lossy().replace('\\', "/");
                        let texture_handle: Handle<Image> = asset_server.load(&texture_path);

                        texture_events.write(SpriteTextureEvent {
                            entity,
                            texture_handle,
                        });

                        info!("Assigned texture to sprite {:?}", entity);
                    }
                } else {
                    ui.label("No project loaded");
                }
            }

            ui.separator();

            ui.label(format!("Color: {:?}", sprite.color));
            ui.label(format!("Custom Size: {:?}", sprite.custom_size));
        });
}

/// Render Camera2d component editor
fn render_camera2d_component(ui: &mut egui::Ui) {
    egui::CollapsingHeader::new(format!("{} Camera2D", Icons::CAMERA))
        .default_open(true)
        .show(ui, |ui| {
            ui.label("2D Camera");
            ui.label("(Editing coming soon)");
        });
}

/// Render Node component editor
fn render_node_component(ui: &mut egui::Ui, _node: &Node) {
    egui::CollapsingHeader::new("Node")
        .default_open(true)
        .show(ui, |ui| {
            ui.label("UI Node");
            ui.label("(Editing coming soon)");
        });
}

/// Render Button component editor
fn render_button_component(ui: &mut egui::Ui) {
    egui::CollapsingHeader::new("Button")
        .default_open(true)
        .show(ui, |ui| {
            ui.label("UI Button");
            ui.label("(Editing coming soon)");
        });
}

/// Render Text component editor
fn render_text_component(ui: &mut egui::Ui, text: &Text) {
    egui::CollapsingHeader::new("Text")
        .default_open(true)
        .show(ui, |ui| {
            // In Bevy 0.16, Text is a simple wrapper around String
            ui.label(format!("Text: {}", text.0));
            ui.label("(Editing coming soon)");
        });
}

/// Render "Add Component" menu
fn render_add_component_menu(
    ui: &mut egui::Ui,
    component_registry: &ComponentRegistry,
    selected_entity: Entity,
    add_component_events: &mut MessageWriter<bevy_editor_scene::AddComponentEvent>,
) {
    ui.menu_button(format!("{} Add Component", Icons::NEW), |ui| {
        for category in component_registry.categories() {
            let components = component_registry.get_by_category(category);
            if components.is_empty() {
                continue;
            }

            ui.menu_button(ComponentRegistry::category_name(category), |ui| {
                for component_info in components {
                    if ui.button(component_info.name).clicked() {
                        add_component_events.write(bevy_editor_scene::AddComponentEvent {
                            entity: selected_entity,
                            component_name: component_info.name.to_string(),
                        });
                        info!("Adding component: {} to entity {:?}", component_info.name, selected_entity);
                        ui.close();
                    }
                }
            });
        }
    });
}
