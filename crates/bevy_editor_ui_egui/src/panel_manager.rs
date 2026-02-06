//! Panel manager for organizing editor panels with tabs

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::component_registry::EditorComponentRegistry;
use crate::inspector_panel::render_inspector_panel;
use crate::scene_tree_panel::render_scene_tree_panel;
use bevy_editor_frontend_api::{
    scene_tree::{SceneTreeCommand, SceneTreeNode},
    EntityComponentData, ProjectBrowserPanelState,
};
use bevy_editor_scene::{
    EditorScene, EditorSceneEntity, NameEditEvent, SpriteTextureEvent, TransformEditEvent,
};
use bevy_editor_tilemap::LayerManager;
use bevy_editor_tilemap::TilesetManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftPanelTab {
    SceneTree,
    Layers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightPanelTab {
    Inspector,
    Tilesets,
    Assets,
}

#[derive(Resource)]
pub struct PanelManager {
    pub left_tab: LeftPanelTab,
    pub right_tab: RightPanelTab,
    pub left_width: f32,
    pub right_width: f32,
}

impl Default for PanelManager {
    fn default() -> Self {
        Self {
            left_tab: LeftPanelTab::SceneTree,
            right_tab: RightPanelTab::Inspector,
            left_width: 250.0,
            right_width: 300.0,
        }
    }
}

/// Resource to hold temporary name edit buffer (for inspector entity naming)
#[derive(Resource, Default)]
pub struct NameEditBuffer {
    pub buffer: String,
}

/// System to render left panel with tabs
pub fn render_left_panel(
    mut contexts: EguiContexts,
    mut panel_manager: ResMut<PanelManager>,
    mut editor_scene: ResMut<EditorScene>,
    mut layer_manager: ResMut<LayerManager>,
    mut scene_tree_events: MessageWriter<SceneTreeCommand>,
    scene_entity_query: Query<(Entity, Option<&Name>, Option<&Children>), With<EditorSceneEntity>>,
) {
    let Some(ctx) = contexts.ctx_mut().ok() else {
        return;
    };

    egui::SidePanel::left("left_panel")
        .default_width(panel_manager.left_width)
        .min_width(200.0)
        .max_width(400.0)
        .resizable(true)
        .show(ctx, |ui| {
            // Tab buttons
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut panel_manager.left_tab,
                    LeftPanelTab::SceneTree,
                    "🌳 Scene Tree",
                );
                ui.selectable_value(
                    &mut panel_manager.left_tab,
                    LeftPanelTab::Layers,
                    "📚 Layers",
                );
            });

            ui.separator();

            // Render selected tab content
            match panel_manager.left_tab {
                LeftPanelTab::SceneTree => {
                    // Extract entity data from queries
                    let entity_data: Vec<SceneTreeNode> = scene_entity_query
                        .iter()
                        .map(|(entity, name, children)| {
                            SceneTreeNode::new(
                                entity,
                                name.map(|n| n.to_string())
                                    .unwrap_or_else(|| "Unnamed".to_string()),
                                children.is_some_and(|c| !c.is_empty()),
                                children.map_or_else(Vec::new, |c| c.iter().collect()),
                            )
                        })
                        .collect();

                    render_scene_tree_panel(
                        ui,
                        &mut editor_scene,
                        &entity_data,
                        &mut scene_tree_events,
                    );
                }
                LeftPanelTab::Layers => {
                    render_layers_tab(ui, &mut layer_manager);
                }
            }
        });
}

/// System to render right panel with tabs
pub fn render_right_panel(
    mut contexts: EguiContexts,
    mut panel_manager: ResMut<PanelManager>,
    editor_scene: Res<EditorScene>,
    component_registry: Res<EditorComponentRegistry>,
    tileset_manager: Res<TilesetManager>,
    mut tileset_zoom: ResMut<crate::tileset_panel::TilesetZoom>,
    mut transform_events: MessageWriter<TransformEditEvent>,
    mut name_events: MessageWriter<NameEditEvent>,
    mut name_edit_buffer: ResMut<NameEditBuffer>,
    mut project_browser: ResMut<crate::project_browser::ProjectBrowser>,
    mut project_browser_panel: ResMut<ProjectBrowserPanelState>,
    asset_server: Res<AssetServer>,
    mut texture_events: MessageWriter<SpriteTextureEvent>,
    mut add_component_events: MessageWriter<bevy_editor_scene::AddComponentEvent>,
    images: Res<Assets<Image>>,
    entity_query: Query<(
        Entity,
        Option<&Name>,
        Option<&Transform>,
        Option<&Visibility>,
        Option<&Sprite>,
        Option<&Camera2d>,
        Option<&Node>,
        Has<Button>,
        Option<&Text>,
    )>,
) {
    // Pre-register sprite texture with egui if inspector tab is active
    let sprite_texture_id = if panel_manager.right_tab == RightPanelTab::Inspector {
        if let Some(selected_entity) = editor_scene.selected_entity {
            if let Ok((_, _, _, _, sprite, _, _, _, _)) = entity_query.get(selected_entity) {
                if let Some(sprite) = sprite {
                    if sprite.image.is_strong() {
                        Some(contexts.add_image(bevy_egui::EguiTextureHandle::Strong(sprite.image.clone())))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let Some(ctx) = contexts.ctx_mut().ok() else {
        return;
    };

    egui::SidePanel::right("right_panel")
        .default_width(panel_manager.right_width)
        .min_width(200.0)
        .max_width(500.0)
        .resizable(true)
        .show(ctx, |ui| {
            // Tab buttons
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut panel_manager.right_tab,
                    RightPanelTab::Inspector,
                    "🔍 Inspector",
                );
                ui.selectable_value(
                    &mut panel_manager.right_tab,
                    RightPanelTab::Tilesets,
                    "🎨 Tilesets",
                );
                ui.selectable_value(
                    &mut panel_manager.right_tab,
                    RightPanelTab::Assets,
                    "📦 Assets",
                );
            });

            ui.separator();

            // Render selected tab content
            match panel_manager.right_tab {
                RightPanelTab::Inspector => {
                    // Get selected entity component data
                    let selected_entity_data =
                        if let Some(selected_entity) = editor_scene.selected_entity {
                            entity_query.get(selected_entity).ok().map(
                                |(
                                    entity,
                                    name,
                                    transform,
                                    visibility,
                                    sprite,
                                    camera2d,
                                    node,
                                    has_button,
                                    text,
                                )| {
                                    EntityComponentData {
                                        entity,
                                        name: name.map(|n| n.to_string()),
                                        transform: transform.copied(),
                                        visibility: visibility.copied(),
                                        sprite: sprite.cloned(),
                                        has_camera2d: camera2d.is_some(),
                                        node: node.cloned(),
                                        has_button,
                                        text: text.cloned(),
                                    }
                                },
                            )
                        } else {
                            None
                        };

                    render_inspector_panel(
                        ui,
                        &editor_scene,
                        selected_entity_data.as_ref(),
                        &component_registry.registry,
                        &mut transform_events,
                        &mut name_events,
                        &mut name_edit_buffer.buffer,
                        project_browser.project_root.as_ref(),
                        &asset_server,
                        &mut texture_events,
                        sprite_texture_id,
                        &images,
                        &mut add_component_events,
                    );
                }
                RightPanelTab::Tilesets => {
                    render_tilesets_tab(ui, &tileset_manager, &mut tileset_zoom);
                }
                RightPanelTab::Assets => {
                    crate::project_browser_panel::project_browser_panel_ui(
                        ui,
                        &mut project_browser,
                        &mut project_browser_panel,
                        &editor_scene,
                        &asset_server,
                        &mut texture_events,
                    );
                }
            }
        });
}

/// Render layers tab content
fn render_layers_tab(ui: &mut egui::Ui, layer_manager: &mut LayerManager) {
    use crate::icons::Icons;

    ui.heading("Layers");

    // Add layer button
    ui.horizontal(|ui| {
        if ui.button(format!("{} Add Layer", Icons::NEW)).clicked() {
            let new_layer = bevy_editor_tilemap::create_default_layer(
                bevy_editor_formats::LayerType::Tiles,
                &format!("Layer {}", layer_manager.layers.len()),
                layer_manager.layers.len() as i32,
                None,
            );
            layer_manager.add_layer(new_layer);
        }
    });

    ui.separator();

    // Show active layer info
    if let Some(active_idx) = layer_manager.active_layer {
        if let Some(layer) = layer_manager.get_layer(active_idx) {
            ui.label(format!("Active: {}", layer.metadata.identifier));
        }
    }

    ui.separator();

    // Layer list
    egui::ScrollArea::vertical().show(ui, |ui| {
        let layer_count = layer_manager.layers.len();
        for idx in 0..layer_count {
            if let Some(layer) = layer_manager.get_layer(idx) {
                let is_active = layer_manager.active_layer == Some(idx);
                let visible = layer_manager.is_layer_visible(layer.metadata.id);
                let layer_id = layer.metadata.id;
                let layer_name = layer.metadata.identifier.clone();

                ui.horizontal(|ui| {
                    // Visibility toggle
                    let vis_icon = if visible {
                        Icons::EYE
                    } else {
                        Icons::EYE_CLOSED
                    };
                    if ui.button(vis_icon).clicked() {
                        layer_manager.set_layer_visibility(layer_id, !visible);
                    }

                    // Layer name (selectable)
                    if ui.selectable_label(is_active, &layer_name).clicked() {
                        layer_manager.set_active_layer(idx);
                    }
                });
            }
        }
    });
}

/// Render tilesets tab content
fn render_tilesets_tab(
    ui: &mut egui::Ui,
    tileset_manager: &TilesetManager,
    tileset_zoom: &mut crate::tileset_panel::TilesetZoom,
) {
    use crate::icons::Icons;

    ui.heading("Tilesets");

    // Load tileset button
    if ui
        .button(format!("{} Load Tileset", Icons::FOLDER_OPEN))
        .clicked()
    {
        info!("Load tileset clicked");
        // This will be handled by the existing tileset loading system
    }

    ui.separator();

    // Show active tileset info
    if let Some(tileset) = tileset_manager.get_selected_tileset() {
        ui.label(format!("Active: {}", tileset.data.identifier));
        ui.label(format!("Tiles: {}", tileset.tile_count));
        ui.label(format!(
            "Size: {}x{}",
            tileset.data.tile_width, tileset.data.tile_height
        ));

        ui.separator();

        // Zoom controls
        ui.horizontal(|ui| {
            ui.label("Zoom:");
            if ui.button("-").clicked() {
                tileset_zoom.zoom = (tileset_zoom.zoom - 0.1).max(0.5);
            }
            ui.label(format!("{:.0}%", tileset_zoom.zoom * 100.0));
            if ui.button("+").clicked() {
                tileset_zoom.zoom = (tileset_zoom.zoom + 0.1).min(3.0);
            }
        });

        ui.separator();

        // Tileset preview would go here
        ui.label("(Tileset preview - use existing tileset panel rendering)");
    } else {
        ui.label("No tileset loaded");
        ui.label("Click 'Load Tileset' to begin");
    }
}
