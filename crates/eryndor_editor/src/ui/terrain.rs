//! Terrain panel UI for Tiled-style terrain configuration
//!
//! This module provides UI for:
//! - Creating/managing terrain sets (Corner, Edge, Mixed)
//! - Adding/removing terrain types within sets
//! - Marking tile corners/edges with terrain types
//! - Painting with terrain brush

use bevy::prelude::Color;
use bevy_egui::egui;

use crate::autotile::{TerrainSet, TerrainSetType, TerrainType};
use crate::project::Project;
use crate::EditorState;

/// Render the terrain panel (shown when Terrain tool is selected)
pub fn render_terrain_panel(ui: &mut egui::Ui, editor_state: &mut EditorState, project: &mut Project) {
    ui.heading("Terrain Sets");
    ui.separator();

    // Get terrain sets for current tileset
    let terrain_sets: Vec<(uuid::Uuid, String, TerrainSetType)> = if let Some(tileset_id) = editor_state.selected_tileset {
        project.autotile_config.terrain_sets.iter()
            .filter(|ts| ts.tileset_id == tileset_id)
            .map(|ts| (ts.id, ts.name.clone(), ts.set_type))
            .collect()
    } else {
        Vec::new()
    };

    if terrain_sets.is_empty() && project.autotile_config.terrains.is_empty() {
        ui.label("No terrain sets defined for this tileset.");
        ui.label("Select a tileset and create a terrain set to start.");
    } else {
        // Show new terrain sets
        for (set_id, name, set_type) in &terrain_sets {
            let type_label = match set_type {
                TerrainSetType::Corner => "[C]",
                TerrainSetType::Edge => "[E]",
                TerrainSetType::Mixed => "[M]",
            };

            let selected = editor_state.selected_terrain_set == Some(*set_id);
            if ui.selectable_label(selected, format!("{} {}", type_label, name)).clicked() {
                editor_state.selected_terrain_set = Some(*set_id);
                editor_state.selected_terrain = None; // Clear legacy terrain selection
            }
        }

        // Show legacy terrains (47-tile blob) if any
        if !project.autotile_config.terrains.is_empty() {
            ui.separator();
            ui.label("Legacy Terrains (47-tile blob):");

            let terrains: Vec<(uuid::Uuid, String)> = if let Some(tileset_id) = editor_state.selected_tileset {
                project.autotile_config.terrains.iter()
                    .filter(|t| t.tileset_id == tileset_id)
                    .map(|t| (t.id, t.name.clone()))
                    .collect()
            } else {
                Vec::new()
            };

            for (terrain_id, name) in &terrains {
                let selected = editor_state.selected_terrain == Some(*terrain_id);
                if ui.selectable_label(selected, name).clicked() {
                    editor_state.selected_terrain = Some(*terrain_id);
                    editor_state.selected_terrain_set = None; // Clear new terrain set selection
                }
            }
        }
    }

    ui.separator();

    // Create new terrain set button
    if editor_state.selected_tileset.is_some() {
        if ui.button("+ New Terrain Set").clicked() {
            editor_state.show_new_terrain_set_dialog = true;
            editor_state.new_terrain_name = "New Terrain Set".to_string();
        }

        // Legacy: Create new 47-tile terrain
        if ui.button("+ New Legacy Terrain (47-tile)").clicked() {
            editor_state.show_new_terrain_dialog = true;
            editor_state.new_terrain_name = "New Terrain".to_string();
            editor_state.new_terrain_first_tile = editor_state.selected_tile.unwrap_or(0);
        }
    } else {
        ui.label("Select a tileset first");
    }

    // Delete selected terrain set
    if let Some(set_id) = editor_state.selected_terrain_set {
        if ui.button("Delete Terrain Set").clicked() {
            project.autotile_config.remove_terrain_set(set_id);
            editor_state.selected_terrain_set = None;
            project.mark_dirty();
        }
    }

    // Delete legacy terrain
    if let Some(terrain_id) = editor_state.selected_terrain {
        if ui.button("Delete Legacy Terrain").clicked() {
            project.autotile_config.remove_terrain(terrain_id);
            editor_state.selected_terrain = None;
            project.mark_dirty();
        }
    }

    // Show terrain set details if selected
    if let Some(set_id) = editor_state.selected_terrain_set {
        // First pass: gather info we need to display (immutable borrow)
        let terrain_info: Option<(String, Vec<(String, [u8; 4])>, usize)> =
            project.autotile_config.get_terrain_set(set_id).map(|terrain_set| {
                let type_name = match terrain_set.set_type {
                    TerrainSetType::Corner => "Corner Set (4 positions)",
                    TerrainSetType::Edge => "Edge Set (4 positions)",
                    TerrainSetType::Mixed => "Mixed Set (8 positions)",
                }.to_string();

                let terrain_list: Vec<(String, [u8; 4])> = terrain_set.terrains.iter()
                    .map(|t| {
                        let srgba = t.color.to_srgba();
                        let color = [
                            (srgba.red * 255.0) as u8,
                            (srgba.green * 255.0) as u8,
                            (srgba.blue * 255.0) as u8,
                            (srgba.alpha * 255.0) as u8,
                        ];
                        (t.name.clone(), color)
                    })
                    .collect();

                let tile_count = terrain_set.tile_terrains.len();
                (type_name, terrain_list, tile_count)
            });

        if let Some((type_name, terrain_list, tile_count)) = terrain_info {
            ui.separator();
            ui.heading("Terrains in Set");
            ui.label(&type_name);

            // List terrains in the set
            for (idx, (name, color)) in terrain_list.iter().enumerate() {
                let selected = editor_state.selected_terrain_in_set == Some(idx);
                let color32 = egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);

                ui.horizontal(|ui| {
                    // Color swatch
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, color32);

                    if ui.selectable_label(selected, name).clicked() {
                        editor_state.selected_terrain_in_set = Some(idx);
                    }
                });
            }

            // Add new terrain to set
            if ui.button("+ Add Terrain").clicked() {
                editor_state.show_add_terrain_to_set_dialog = true;
                editor_state.new_terrain_name = "New Terrain".to_string();
            }

            // Remove selected terrain from set (now we can safely take mutable borrow)
            if let Some(idx) = editor_state.selected_terrain_in_set {
                if ui.button("Remove Selected").clicked() {
                    if let Some(set) = project.autotile_config.get_terrain_set_mut(set_id) {
                        set.remove_terrain(idx);
                        editor_state.selected_terrain_in_set = None;
                        project.mark_dirty();
                    }
                }
            }

            ui.separator();
            ui.label("Tile Configuration:");
            ui.label(format!("{} tiles configured", tile_count));
        }
    }

    // Show legacy terrain info if selected
    if let Some(terrain_id) = editor_state.selected_terrain {
        if let Some(terrain) = project.autotile_config.get_terrain(terrain_id) {
            ui.separator();
            ui.label(format!("Base tile: {}", terrain.base_tile));
            ui.label(format!("First tile: {}", terrain.base_tile.saturating_sub(46)));
            ui.label("47-tile blob format");
        }
    }

    // Instructions
    ui.separator();
    ui.label("Usage:");
    ui.label("1. Create a Terrain Set (Corner/Edge/Mixed)");
    ui.label("2. Add terrains to the set");
    ui.label("3. Mark tile corners/edges with terrains");
    ui.label("4. Paint with Terrain tool");
}

/// Render the new terrain set dialog
pub fn render_new_terrain_set_dialog(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    project: &mut Project,
) {
    if !editor_state.show_new_terrain_set_dialog {
        return;
    }

    egui::Window::new("New Terrain Set")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut editor_state.new_terrain_name);
            });

            ui.horizontal(|ui| {
                ui.label("Type:");
                egui::ComboBox::from_id_salt("terrain_set_type")
                    .selected_text(match editor_state.new_terrain_set_type {
                        TerrainSetType::Corner => "Corner",
                        TerrainSetType::Edge => "Edge",
                        TerrainSetType::Mixed => "Mixed",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut editor_state.new_terrain_set_type, TerrainSetType::Corner, "Corner (4 positions)");
                        ui.selectable_value(&mut editor_state.new_terrain_set_type, TerrainSetType::Edge, "Edge (4 positions)");
                        ui.selectable_value(&mut editor_state.new_terrain_set_type, TerrainSetType::Mixed, "Mixed (8 positions)");
                    });
            });

            ui.label(match editor_state.new_terrain_set_type {
                TerrainSetType::Corner => "Matches tile corners. Good for terrain transitions.",
                TerrainSetType::Edge => "Matches tile edges. Good for roads, platforms.",
                TerrainSetType::Mixed => "Matches both corners and edges. Most flexible.",
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    editor_state.show_new_terrain_set_dialog = false;
                }

                let can_create = !editor_state.new_terrain_name.is_empty()
                    && editor_state.selected_tileset.is_some();

                ui.add_enabled_ui(can_create, |ui| {
                    if ui.button("Create").clicked() {
                        if let Some(tileset_id) = editor_state.selected_tileset {
                            let terrain_set = TerrainSet::new(
                                editor_state.new_terrain_name.clone(),
                                tileset_id,
                                editor_state.new_terrain_set_type,
                            );
                            let set_id = terrain_set.id;
                            project.autotile_config.add_terrain_set(terrain_set);
                            project.mark_dirty();
                            editor_state.selected_terrain_set = Some(set_id);
                            editor_state.selected_terrain = None;
                        }
                        editor_state.show_new_terrain_set_dialog = false;
                    }
                });
            });
        });
}

/// Render the add terrain to set dialog
pub fn render_add_terrain_to_set_dialog(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    project: &mut Project,
) {
    if !editor_state.show_add_terrain_to_set_dialog {
        return;
    }

    egui::Window::new("Add Terrain")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut editor_state.new_terrain_name);
            });

            ui.horizontal(|ui| {
                ui.label("Color:");
                let mut color = [
                    editor_state.new_terrain_color[0],
                    editor_state.new_terrain_color[1],
                    editor_state.new_terrain_color[2],
                ];
                if ui.color_edit_button_rgb(&mut color).changed() {
                    editor_state.new_terrain_color = [color[0], color[1], color[2]];
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    editor_state.show_add_terrain_to_set_dialog = false;
                }

                let can_create = !editor_state.new_terrain_name.is_empty()
                    && editor_state.selected_terrain_set.is_some();

                ui.add_enabled_ui(can_create, |ui| {
                    if ui.button("Add").clicked() {
                        if let Some(set_id) = editor_state.selected_terrain_set {
                            if let Some(set) = project.autotile_config.get_terrain_set_mut(set_id) {
                                let color = Color::srgb(
                                    editor_state.new_terrain_color[0],
                                    editor_state.new_terrain_color[1],
                                    editor_state.new_terrain_color[2],
                                );
                                let idx = set.add_terrain(editor_state.new_terrain_name.clone(), color);
                                editor_state.selected_terrain_in_set = Some(idx);
                                project.mark_dirty();
                            }
                        }
                        editor_state.show_add_terrain_to_set_dialog = false;
                    }
                });
            });
        });
}

/// Render the new legacy terrain dialog (47-tile blob format)
pub fn render_new_terrain_dialog(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    project: &mut Project,
) {
    if !editor_state.show_new_terrain_dialog {
        return;
    }

    egui::Window::new("New Legacy Terrain (47-tile)")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut editor_state.new_terrain_name);
            });

            ui.horizontal(|ui| {
                ui.label("First Tile Index:");
                ui.add(egui::DragValue::new(&mut editor_state.new_terrain_first_tile)
                    .range(0..=9999));
            });

            ui.label("(Use the currently selected tile as first tile)");

            if let Some(tile) = editor_state.selected_tile {
                if ui.button(format!("Use selected tile ({})", tile)).clicked() {
                    editor_state.new_terrain_first_tile = tile;
                }
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    editor_state.show_new_terrain_dialog = false;
                }

                let can_create = !editor_state.new_terrain_name.is_empty()
                    && editor_state.selected_tileset.is_some();

                ui.add_enabled_ui(can_create, |ui| {
                    if ui.button("Create").clicked() {
                        if let Some(tileset_id) = editor_state.selected_tileset {
                            let terrain = TerrainType::new(
                                editor_state.new_terrain_name.clone(),
                                tileset_id,
                                editor_state.new_terrain_first_tile,
                            );
                            let terrain_id = terrain.id;
                            project.autotile_config.add_terrain(terrain);
                            project.mark_dirty();
                            editor_state.selected_terrain = Some(terrain_id);
                        }
                        editor_state.show_new_terrain_dialog = false;
                    }
                });
            });
        });
}
