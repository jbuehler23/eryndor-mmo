use bevy::prelude::*;
use bevy_editor_formats::LayerType;
use bevy_editor_tilemap::{create_default_layer, LayerManager};
use bevy_egui::{egui, EguiContexts};

/// Layer panel system - shows layer list with controls
pub fn layer_panel_ui(mut contexts: EguiContexts, mut layer_manager: ResMut<LayerManager>) {
    let Some(ctx) = contexts.ctx_mut().ok() else {
        return;
    };

    egui::SidePanel::left("layer_panel")
        .default_width(250.0)
        .min_width(200.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Layers");
            ui.separator();

            // Layer controls
            ui.horizontal(|ui| {
                if ui.button("�z Add Layer").clicked() {
                    let new_layer = create_default_layer(
                        LayerType::Tiles,
                        &format!("Layer {}", layer_manager.layers.len() + 1),
                        layer_manager.layers.len() as i32,
                        None,
                    );
                    layer_manager.add_layer(new_layer);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button("dY-`")
                        .on_hover_text("Delete Selected Layer")
                        .clicked()
                    {
                        if let Some(active_idx) = layer_manager.active_layer {
                            layer_manager.remove_layer(active_idx);
                        }
                    }
                });
            });

            ui.separator();

            // Layer list
            if layer_manager.layers.is_empty() {
                ui.label("No layers");
                ui.label("Click '�z Add Layer' to create one");
            } else {
                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - 50.0)
                    .show(ui, |ui| {
                        // Iterate in reverse order to display top layers first
                        let layer_count = layer_manager.layers.len();

                        // Collect mutations to apply after iteration
                        let mut visibility_changes = Vec::new();
                        let mut new_active_layer = None;
                        let mut move_up_idx = None;
                        let mut move_down_idx = None;

                        for idx in (0..layer_count).rev() {
                            let layer = &layer_manager.layers[idx];
                            let is_active = layer_manager.active_layer == Some(idx);
                            let layer_id = layer.metadata.id;
                            let layer_name = layer.metadata.identifier.clone();
                            let layer_type_str = layer.metadata.layer_type.as_str();

                            ui.push_id(layer_id, |ui| {
                                // Layer row with background
                                let row_rect = ui
                                    .horizontal(|ui| {
                                        // Visibility toggle
                                        let mut visible = layer_manager.is_layer_visible(layer_id);
                                        if ui.checkbox(&mut visible, "").changed() {
                                            visibility_changes.push((layer_id, visible));
                                        }

                                        // Layer name (selectable)
                                        let response = ui.selectable_label(
                                            is_active,
                                            format!("{} ({})", layer_name, layer_type_str),
                                        );

                                        if response.clicked() {
                                            new_active_layer = Some(idx);
                                        }

                                        // Reorder buttons
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .small_button("�-�")
                                                    .on_hover_text("Move Down")
                                                    .clicked()
                                                {
                                                    move_down_idx = Some(idx);
                                                }
                                                if ui
                                                    .small_button("�-�")
                                                    .on_hover_text("Move Up")
                                                    .clicked()
                                                {
                                                    move_up_idx = Some(idx);
                                                }
                                            },
                                        );
                                    })
                                    .response
                                    .rect;

                                // Highlight active layer
                                if is_active {
                                    ui.painter().rect_filled(
                                        row_rect.expand(2.0),
                                        egui::CornerRadius::ZERO,
                                        egui::Color32::from_rgba_premultiplied(100, 149, 237, 30),
                                    );
                                }
                            });

                            ui.separator();
                        }

                        // Apply mutations after iteration
                        for (layer_id, visible) in visibility_changes {
                            layer_manager.set_layer_visibility(layer_id, visible);
                        }
                        if let Some(idx) = new_active_layer {
                            layer_manager.set_active_layer(idx);
                        }
                        if let Some(idx) = move_up_idx {
                            layer_manager.move_layer_up(idx);
                        }
                        if let Some(idx) = move_down_idx {
                            layer_manager.move_layer_down(idx);
                        }
                    });

                // Layer properties for active layer
                if let Some(active_idx) = layer_manager.active_layer {
                    if let Some(layer) = layer_manager.get_layer(active_idx) {
                        ui.separator();
                        ui.heading("Layer Properties");

                        ui.label(format!("Type: {}", layer.metadata.layer_type.as_str()));
                        ui.label(format!(
                            "Size: {}x{}",
                            layer.metadata.width, layer.metadata.height
                        ));
                        ui.label(format!("Tiles: {}", layer.tiles.len()));
                        ui.label(format!("Z-Index: {}", layer.metadata.z_index));

                        // Could add opacity slider, parallax settings, etc.
                    }
                }
            }
        });
}

/// Event to request creating a new layer
#[derive(Event)]
pub struct CreateLayerEvent {
    pub layer_type: LayerType,
    pub name: String,
}

/// Event to request deleting a layer
#[derive(Event)]
pub struct DeleteLayerEvent {
    pub layer_index: usize,
}

/// Event to request reordering layers
#[derive(Event)]
pub struct ReorderLayerEvent {
    pub from_index: usize,
    pub to_index: usize,
}
