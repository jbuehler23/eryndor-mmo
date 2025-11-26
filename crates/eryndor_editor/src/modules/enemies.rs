//! Enemies Editor Module
//! Create and edit enemy types.

use bevy_egui::egui;
use crate::editor_state::{EditorState, EditingEnemy, EditingLootItem};

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
                            // Load enemy for editing with all fields (extracting from nested JSON structure)
                            editor_state.enemies.editing_enemy = Some(EditingEnemy {
                                id: enemy.id,
                                name: enemy.name.clone(),
                                max_health: enemy.max_health,
                                attack_power: enemy.attack_power,
                                defense: enemy.defense,
                                move_speed: enemy.move_speed,
                                aggro_range: enemy.aggro_range,
                                leash_range: enemy.leash_range,
                                respawn_delay: enemy.respawn_delay,
                                visual_shape: enemy.visual.shape.clone(),
                                visual_color: enemy.visual.color,
                                visual_size: enemy.visual.size,
                                gold_min: enemy.loot_table.gold_min,
                                gold_max: enemy.loot_table.gold_max,
                                loot_items: enemy.loot_table.items.iter().map(|item| EditingLootItem {
                                    item_id: item.item_id,
                                    drop_chance: item.drop_chance,
                                    quantity_min: item.quantity_min,
                                    quantity_max: item.quantity_max,
                                }).collect(),
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
                ui.heading("Combat Stats");

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

            // AI/Behavior
            ui.group(|ui| {
                ui.heading("AI & Behavior");

                ui.horizontal(|ui| {
                    ui.label("Aggro Range:");
                    ui.add(egui::DragValue::new(&mut editing_enemy.aggro_range).range(0.0..=1000.0).speed(5.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Leash Range:");
                    ui.add(egui::DragValue::new(&mut editing_enemy.leash_range).range(0.0..=2000.0).speed(10.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Respawn Delay (s):");
                    ui.add(egui::DragValue::new(&mut editing_enemy.respawn_delay).range(0.0..=3600.0).speed(1.0));
                });
            });

            ui.separator();

            // Visual
            ui.group(|ui| {
                ui.heading("Visual");

                ui.horizontal(|ui| {
                    ui.label("Shape:");
                    egui::ComboBox::from_id_salt("enemy_shape")
                        .selected_text(if editing_enemy.visual_shape.is_empty() { "Circle" } else { &editing_enemy.visual_shape })
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(&editing_enemy.visual_shape == "Circle", "Circle").clicked() {
                                editing_enemy.visual_shape = "Circle".to_string();
                            }
                            if ui.selectable_label(&editing_enemy.visual_shape == "Square", "Square").clicked() {
                                editing_enemy.visual_shape = "Square".to_string();
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Color:");
                    let mut color = egui::Color32::from_rgba_unmultiplied(
                        (editing_enemy.visual_color[0] * 255.0) as u8,
                        (editing_enemy.visual_color[1] * 255.0) as u8,
                        (editing_enemy.visual_color[2] * 255.0) as u8,
                        (editing_enemy.visual_color[3] * 255.0) as u8,
                    );
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        editing_enemy.visual_color = [
                            color.r() as f32 / 255.0,
                            color.g() as f32 / 255.0,
                            color.b() as f32 / 255.0,
                            color.a() as f32 / 255.0,
                        ];
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Size:");
                    ui.add(egui::DragValue::new(&mut editing_enemy.visual_size).range(4.0..=100.0).speed(1.0));
                });
            });

            ui.separator();

            // Loot Table
            ui.group(|ui| {
                ui.heading("Loot Table");

                ui.horizontal(|ui| {
                    ui.label("Gold Min:");
                    ui.add(egui::DragValue::new(&mut editing_enemy.gold_min).range(0..=100000).speed(1.0));
                    ui.label("Max:");
                    ui.add(egui::DragValue::new(&mut editing_enemy.gold_max).range(0..=100000).speed(1.0));
                });

                ui.separator();
                ui.label("Item Drops:");

                // Item drops list
                let mut remove_idx = None;
                for (idx, item) in editing_enemy.loot_items.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("#{}", idx + 1));
                        ui.label("Item ID:");
                        ui.add(egui::DragValue::new(&mut item.item_id).range(1..=1000));
                        ui.label("Chance:");
                        ui.add(egui::DragValue::new(&mut item.drop_chance).range(0.0..=1.0).speed(0.01).fixed_decimals(2));
                        ui.label("Qty:");
                        ui.add(egui::DragValue::new(&mut item.quantity_min).range(1..=99));
                        ui.label("-");
                        ui.add(egui::DragValue::new(&mut item.quantity_max).range(1..=99));
                        if ui.button("X").clicked() {
                            remove_idx = Some(idx);
                        }
                    });
                }

                if let Some(idx) = remove_idx {
                    editing_enemy.loot_items.remove(idx);
                }

                if ui.button("+ Add Item Drop").clicked() {
                    editing_enemy.loot_items.push(EditingLootItem {
                        item_id: 1,
                        drop_chance: 0.5,
                        quantity_min: 1,
                        quantity_max: 1,
                    });
                }
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
