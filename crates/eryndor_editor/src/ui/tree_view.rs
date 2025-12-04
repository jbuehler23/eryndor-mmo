use bevy_egui::egui;
use uuid::Uuid;

use crate::project::Project;
use crate::schema::{PropType, Value};
use crate::EditorState;
use super::Selection;

/// Result from tree view rendering containing any actions to execute
#[derive(Default)]
pub struct TreeViewResult {
    /// Data instance to duplicate
    pub duplicate_data: Option<Uuid>,
    /// Data instance to delete
    pub delete_data: Option<Uuid>,
    /// Level to duplicate
    pub duplicate_level: Option<Uuid>,
    /// Level to delete
    pub delete_level: Option<Uuid>,
    /// Entity to delete (level_id, entity_id)
    pub delete_entity: Option<(Uuid, Uuid)>,
    /// Add tile layer to level
    pub add_tile_layer: Option<Uuid>,
    /// Add object layer to level
    pub add_object_layer: Option<Uuid>,
    /// Delete layer (level_id, layer_index)
    pub delete_layer: Option<(Uuid, usize)>,
    /// Move layer up (level_id, layer_index)
    pub move_layer_up: Option<(Uuid, usize)>,
    /// Move layer down (level_id, layer_index)
    pub move_layer_down: Option<(Uuid, usize)>,
    /// Toggle layer visibility (level_id, layer_index)
    pub toggle_layer_visibility: Option<(Uuid, usize)>,
}

pub fn render_tree_view(ui: &mut egui::Ui, editor_state: &mut EditorState, project: &mut Project) -> TreeViewResult {
    let mut result = TreeViewResult::default();

    ui.heading("Project");
    ui.separator();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Database section - show all data types
            let data_header = egui::CollapsingHeader::new("Data")
                .default_open(true);

            data_header.show(ui, |ui| {
                // Collect type names to avoid borrow issues
                let type_names: Vec<String> = project.schema.data_type_names().into_iter().map(|s| s.to_string()).collect();

                for type_name in &type_names {
                    let instances = project.data.get_by_type(type_name);
                    let count = instances.len();

                    let header_response = egui::CollapsingHeader::new(format!("{} ({})", type_name, count))
                        .id_salt(format!("data_{}", type_name))
                        .show(ui, |ui| {
                            // Collect instance data to avoid borrow issues
                            let instance_data: Vec<_> = instances
                                .iter()
                                .map(|i| (i.id, i.type_name.clone(), i.get_display_name(), i.properties.clone()))
                                .collect();

                            for (id, inst_type, name, properties) in instance_data {
                                let selected = editor_state.selection.is_selected_data(id);

                                // Make each instance expandable to show relationships
                                let inst_header = egui::CollapsingHeader::new(&name)
                                    .id_salt(format!("inst_{}", id))
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        // Main selection
                                        if ui.selectable_label(selected, "Properties").clicked() {
                                            editor_state.selection = Selection::DataInstance(id);
                                        }

                                        // Show relationships
                                        render_instance_relationships(
                                            ui,
                                            &inst_type,
                                            &properties,
                                            project,
                                            editor_state,
                                        );
                                    });

                                // Context menu for instance
                                inst_header.header_response.context_menu(|ui| {
                                    if ui.button("Select").clicked() {
                                        editor_state.selection = Selection::DataInstance(id);
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("Duplicate").clicked() {
                                        result.duplicate_data = Some(id);
                                        ui.close();
                                    }
                                    if ui.button("Delete").clicked() {
                                        result.delete_data = Some(id);
                                        ui.close();
                                    }
                                });
                            }

                            if instances.is_empty() {
                                ui.label(egui::RichText::new("(empty)").italics());
                            }

                            // Quick button for creating new
                            ui.horizontal(|ui| {
                                if ui.small_button(format!("+ New {}", type_name)).clicked() {
                                    editor_state.create_new_instance = Some(type_name.to_string());
                                }
                            });
                        });

                    // Context menu for type header
                    let type_name_clone = type_name.to_string();
                    header_response.header_response.context_menu(|ui| {
                        if ui.button(format!("New {}", type_name_clone)).clicked() {
                            editor_state.create_new_instance = Some(type_name_clone.clone());
                            ui.close();
                        }
                    });
                }

                if project.schema.data_types.is_empty() {
                    ui.label(egui::RichText::new("(no data types in schema)").italics());
                }
            });

            ui.separator();

            // Levels section
            let levels_header = egui::CollapsingHeader::new(format!("Levels ({})", project.levels.len()))
                .default_open(true)
                .show(ui, |ui| {
                    // Collect level data to avoid borrow issues
                    let level_data: Vec<_> = project.levels.iter().map(|l| {
                        (l.id, l.name.clone(), l.layers.len(),
                         l.entities.iter().map(|e| (e.id, e.get_display_name())).collect::<Vec<_>>(),
                         l.layers.iter().map(|l| (l.name.clone(), l.visible)).collect::<Vec<_>>())
                    }).collect();

                    for (level_id, level_name, layer_count, entities, layers) in level_data {
                        let selected = editor_state.selection.is_selected_level(level_id);

                        let level_header = egui::CollapsingHeader::new(&level_name)
                            .id_salt(format!("level_{}", level_id))
                            .default_open(false)
                            .show(ui, |ui| {
                                if ui.selectable_label(selected, "Properties").clicked() {
                                    editor_state.selection = Selection::Level(level_id);
                                }

                                // Layers
                                egui::CollapsingHeader::new(format!("Layers ({})", layer_count))
                                    .id_salt(format!("layers_{}", level_id))
                                    .show(ui, |ui| {
                                        let num_layers = layers.len();
                                        for (i, (layer_name, visible)) in layers.iter().enumerate() {
                                            let layer_selected = editor_state.selection.is_selected_layer(level_id, i);

                                            ui.horizontal(|ui| {
                                                // Visibility toggle button
                                                let vis_icon = if *visible { "v" } else { "." };
                                                if ui.small_button(vis_icon)
                                                    .on_hover_text(if *visible { "Hide layer" } else { "Show layer" })
                                                    .clicked()
                                                {
                                                    result.toggle_layer_visibility = Some((level_id, i));
                                                }

                                                // Layer name
                                                let response = ui.selectable_label(layer_selected, layer_name);

                                                if response.clicked() {
                                                    editor_state.selection = Selection::Layer(level_id, i);
                                                    editor_state.selected_layer = Some(i);
                                                    editor_state.selected_level = Some(level_id);
                                                }

                                                // Context menu for layer
                                                response.context_menu(|ui| {
                                                    if ui.button("Select").clicked() {
                                                        editor_state.selection = Selection::Layer(level_id, i);
                                                        editor_state.selected_layer = Some(i);
                                                        editor_state.selected_level = Some(level_id);
                                                        ui.close();
                                                    }
                                                    ui.separator();
                                                    if *visible {
                                                        if ui.button("Hide").clicked() {
                                                            result.toggle_layer_visibility = Some((level_id, i));
                                                            ui.close();
                                                        }
                                                    } else {
                                                        if ui.button("Show").clicked() {
                                                            result.toggle_layer_visibility = Some((level_id, i));
                                                            ui.close();
                                                        }
                                                    }
                                                    ui.separator();
                                                    ui.add_enabled_ui(i > 0, |ui| {
                                                        if ui.button("Move Up").clicked() {
                                                            result.move_layer_up = Some((level_id, i));
                                                            ui.close();
                                                        }
                                                    });
                                                    ui.add_enabled_ui(i < num_layers - 1, |ui| {
                                                        if ui.button("Move Down").clicked() {
                                                            result.move_layer_down = Some((level_id, i));
                                                            ui.close();
                                                        }
                                                    });
                                                    ui.separator();
                                                    if ui.button("Delete").clicked() {
                                                        result.delete_layer = Some((level_id, i));
                                                        ui.close();
                                                    }
                                                });
                                            });
                                        }

                                        // Add layer buttons
                                        ui.horizontal(|ui| {
                                            if ui.small_button("+ Tile").on_hover_text("Add tile layer").clicked() {
                                                result.add_tile_layer = Some(level_id);
                                            }
                                            if ui.small_button("+ Object").on_hover_text("Add object layer").clicked() {
                                                result.add_object_layer = Some(level_id);
                                            }
                                        });
                                    });

                                // Entities
                                egui::CollapsingHeader::new(format!("Entities ({})", entities.len()))
                                    .id_salt(format!("entities_{}", level_id))
                                    .show(ui, |ui| {
                                        for (entity_id, entity_name) in &entities {
                                            let entity_selected = editor_state.selection.is_selected_entity(level_id, *entity_id);

                                            let entity_response = ui.selectable_label(entity_selected, entity_name);
                                            if entity_response.clicked() {
                                                editor_state.selection = Selection::EntityInstance(level_id, *entity_id);
                                            }

                                            // Context menu for entity
                                            let eid = *entity_id;
                                            entity_response.context_menu(|ui| {
                                                if ui.button("Select").clicked() {
                                                    editor_state.selection = Selection::EntityInstance(level_id, eid);
                                                    ui.close();
                                                }
                                                ui.separator();
                                                if ui.button("Delete").clicked() {
                                                    result.delete_entity = Some((level_id, eid));
                                                    ui.close();
                                                }
                                            });
                                        }

                                        if entities.is_empty() {
                                            ui.label(egui::RichText::new("(no entities)").italics());
                                        }
                                    });
                            });

                        // Context menu for level header
                        level_header.header_response.context_menu(|ui| {
                            if ui.button("Select").clicked() {
                                editor_state.selection = Selection::Level(level_id);
                                ui.close();
                            }
                            ui.separator();
                            if ui.button("Duplicate").clicked() {
                                result.duplicate_level = Some(level_id);
                                ui.close();
                            }
                            if ui.button("Delete").clicked() {
                                result.delete_level = Some(level_id);
                                ui.close();
                            }
                        });
                    }

                    if project.levels.is_empty() {
                        ui.label(egui::RichText::new("(no levels)").italics());
                    }

                    if ui.small_button("+ New Level").clicked() {
                        editor_state.show_new_level_dialog = true;
                    }
                });

            // Context menu for Levels section header
            levels_header.header_response.context_menu(|ui| {
                if ui.button("New Level").clicked() {
                    editor_state.show_new_level_dialog = true;
                    ui.close();
                }
            });

            ui.separator();

            // Tilesets section
            let tilesets_header = egui::CollapsingHeader::new(format!("Tilesets ({})", project.tilesets.len()))
                .default_open(false)
                .show(ui, |ui| {
                    // Collect tileset data to avoid borrow issues
                    let tileset_data: Vec<_> = project.tilesets.iter()
                        .map(|t| (t.id, t.name.clone(), t.columns, t.rows, t.tile_size))
                        .collect();

                    for (tileset_id, tileset_name, columns, rows, tile_size) in tileset_data {
                        let selected = editor_state.selected_tileset == Some(tileset_id);

                        let response = ui.selectable_label(
                            selected,
                            format!("{} ({}x{}, {}px)", tileset_name, columns, rows, tile_size)
                        );

                        if response.clicked() {
                            editor_state.selected_tileset = Some(tileset_id);
                            // Clear selected tile when switching tilesets
                            editor_state.selected_tile = None;
                        }

                        response.context_menu(|ui| {
                            if ui.button("Select").clicked() {
                                editor_state.selected_tileset = Some(tileset_id);
                                editor_state.selected_tile = None;
                                ui.close();
                            }
                            // TODO: Add delete tileset option
                        });
                    }

                    if project.tilesets.is_empty() {
                        ui.label(egui::RichText::new("(no tilesets)").italics());
                    }

                    if ui.small_button("+ Import Tileset").clicked() {
                        editor_state.show_new_tileset_dialog = true;
                    }
                });

            // Context menu for Tilesets section header
            tilesets_header.header_response.context_menu(|ui| {
                if ui.button("Import Tileset").clicked() {
                    editor_state.show_new_tileset_dialog = true;
                    ui.close();
                }
            });
        });

    result
}

/// Render relationship sub-tree for an instance showing its references
fn render_instance_relationships(
    ui: &mut egui::Ui,
    type_name: &str,
    properties: &std::collections::HashMap<String, Value>,
    project: &Project,
    editor_state: &mut EditorState,
) {
    // Get the type definition to find ref properties
    let type_def = match project.schema.get_type(type_name) {
        Some(t) => t,
        None => return,
    };

    // Collect refs to display
    let mut has_refs = false;

    for prop in &type_def.properties {
        match prop.prop_type {
            PropType::Ref => {
                // Single reference
                if let Some(_ref_type) = &prop.ref_type {
                    if let Some(Value::String(ref_id)) = properties.get(&prop.name) {
                        if !ref_id.is_empty() {
                            if let Ok(uuid) = uuid::Uuid::parse_str(ref_id) {
                                if let Some(ref_instance) = project.data.get(uuid) {
                                    has_refs = true;
                                    let ref_name = ref_instance.get_display_name();
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("{}:", prop.name)).weak());
                                        if ui.small_button(&ref_name).clicked() {
                                            editor_state.selection = Selection::DataInstance(uuid);
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
            }
            PropType::Array => {
                // Array - check if it's an array of refs
                if prop.item_type.as_deref() == Some("ref") && prop.ref_type.is_some() {
                    if let Some(Value::Array(items)) = properties.get(&prop.name) {
                        let ref_items: Vec<_> = items
                            .iter()
                            .filter_map(|item| {
                                if let Value::String(ref_id) = item {
                                    if !ref_id.is_empty() {
                                        if let Ok(uuid) = uuid::Uuid::parse_str(ref_id) {
                                            if let Some(ref_instance) = project.data.get(uuid) {
                                                return Some((uuid, ref_instance.get_display_name()));
                                            }
                                        }
                                    }
                                }
                                None
                            })
                            .collect();

                        if !ref_items.is_empty() {
                            has_refs = true;
                            egui::CollapsingHeader::new(format!("{} ({})", prop.name, ref_items.len()))
                                .id_salt(format!("rel_{}_{}", type_name, prop.name))
                                .default_open(false)
                                .show(ui, |ui| {
                                    for (uuid, name) in ref_items {
                                        if ui.small_button(&name).clicked() {
                                            editor_state.selection = Selection::DataInstance(uuid);
                                        }
                                    }
                                });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if !has_refs {
        ui.label(egui::RichText::new("(no references)").weak().italics());
    }
}
