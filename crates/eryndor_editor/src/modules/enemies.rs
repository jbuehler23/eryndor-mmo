//! Enemies Editor Module
//! Create and edit enemy types.

use bevy_egui::egui;
use crate::editor_state::{EditorState, EditingEnemy};

/// Render the enemies editor module
pub fn render(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    // Render side panel first so it claims its space
    egui::SidePanel::left("enemies_list_panel")
        .default_width(250.0)
        .show_inside(ui, |ui| {
            ui.heading("Enemies");

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("+ New Enemy").clicked() {
                    editor_state.enemies.show_create_dialog = true;
                }
                if ui.button("Refresh").clicked() {
                    editor_state.action_load_enemies = true;
                }
            });

            ui.separator();

            // Search
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut editor_state.enemies.search_query);
            });

            ui.separator();

            // Enemy list
            egui::ScrollArea::vertical().show(ui, |ui| {
                if editor_state.enemies.enemy_list.is_empty() {
                    ui.label("No enemies loaded");
                    ui.label("Click 'Refresh' to load from server");
                } else {
                    for enemy in &editor_state.enemies.enemy_list {
                        // Apply search filter
                        if !editor_state.enemies.search_query.is_empty() {
                            if !enemy.name.to_lowercase().contains(&editor_state.enemies.search_query.to_lowercase()) {
                                continue;
                            }
                        }

                        let is_selected = editor_state.enemies.selected_enemy == Some(enemy.id);
                        let label = format!("{} (HP: {})", enemy.name, enemy.max_health as i32);
                        if ui.selectable_label(is_selected, &label).clicked() {
                            editor_state.enemies.selected_enemy = Some(enemy.id);
                            // Load enemy for editing
                            editor_state.enemies.editing_enemy = Some(EditingEnemy {
                                id: enemy.id,
                                name: enemy.name.clone(),
                                max_health: enemy.max_health,
                                attack_power: enemy.attack_power,
                                defense: enemy.defense,
                                move_speed: enemy.move_speed,
                            });
                        }
                    }
                }
            });
        });

    // Right panel - enemy properties
    egui::CentralPanel::default().show_inside(ui, |ui| {
        if let Some(ref mut editing_enemy) = editor_state.enemies.editing_enemy {
            ui.heading(format!("Enemy #{} - {}", editing_enemy.id, editing_enemy.name));

            ui.separator();

            // Basic properties
            ui.group(|ui| {
                ui.heading("Basic Info");

                ui.horizontal(|ui| {
                    ui.label("ID:");
                    ui.label(format!("{}", editing_enemy.id));
                });

                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editing_enemy.name);
                });
            });

            ui.separator();

            // Stats
            ui.group(|ui| {
                ui.heading("Stats");

                ui.horizontal(|ui| {
                    ui.label("Max Health:");
                    ui.add(egui::DragValue::new(&mut editing_enemy.max_health).range(1.0..=100000.0).speed(1.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Attack Power:");
                    ui.add(egui::DragValue::new(&mut editing_enemy.attack_power).range(0.0..=10000.0).speed(0.5));
                });

                ui.horizontal(|ui| {
                    ui.label("Defense:");
                    ui.add(egui::DragValue::new(&mut editing_enemy.defense).range(0.0..=10000.0).speed(0.5));
                });

                ui.horizontal(|ui| {
                    ui.label("Move Speed:");
                    ui.add(egui::DragValue::new(&mut editing_enemy.move_speed).range(0.0..=1000.0).speed(1.0));
                });
            });

            ui.separator();

            // Actions
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    editor_state.action_save_enemy = true;
                }
                if ui.button("Delete").on_hover_text("Delete this enemy").clicked() {
                    editor_state.action_delete_enemy = true;
                }
            });
        } else if editor_state.enemies.selected_enemy.is_some() {
            ui.centered_and_justified(|ui| {
                ui.label("Loading enemy data...");
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select an enemy from the list or create a new one");
            });
        }
    });

    // Create new enemy dialog
    if editor_state.enemies.show_create_dialog {
        egui::Window::new("Create New Enemy")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editor_state.enemies.new_enemy_name);
                });

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        editor_state.action_create_enemy = true;
                    }
                    if ui.button("Cancel").clicked() {
                        editor_state.enemies.show_create_dialog = false;
                        editor_state.enemies.new_enemy_name.clear();
                    }
                });
            });
    }
}
