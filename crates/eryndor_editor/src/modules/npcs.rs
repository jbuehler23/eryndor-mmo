//! NPCs Editor Module
//! Create and edit NPCs with various roles (Quest Givers, Trainers, etc.)

use bevy_egui::egui;
use crate::editor_state::{EditorState, EditingNpc, EditingTrainerItem};

/// Render the NPCs editor module
pub fn render(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    // Render side panel first so it claims its space
    egui::SidePanel::left("npcs_list_panel")
        .default_width(250.0)
        .show_inside(ui, |ui| {
            ui.heading("NPCs");

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("+ New NPC").clicked() {
                    editor_state.npcs.show_create_dialog = true;
                }
                if ui.button("Refresh").clicked() {
                    editor_state.action_load_npcs = true;
                }
            });

            ui.separator();

            // Search
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut editor_state.npcs.search_query);
            });

            // Type filter
            egui::ComboBox::from_label("Type")
                .selected_text(editor_state.npcs.role_filter.as_deref().unwrap_or("All"))
                .show_ui(ui, |ui| {
                    if ui.selectable_label(editor_state.npcs.role_filter.is_none(), "All").clicked() {
                        editor_state.npcs.role_filter = None;
                    }
                    for npc_type in ["QuestGiver", "Trainer"] {
                        if ui.selectable_label(editor_state.npcs.role_filter.as_deref() == Some(npc_type), npc_type).clicked() {
                            editor_state.npcs.role_filter = Some(npc_type.to_string());
                        }
                    }
                });

            ui.separator();

            // NPC list
            egui::ScrollArea::vertical().show(ui, |ui| {
                if editor_state.npcs.npc_list.is_empty() {
                    ui.label("No NPCs loaded");
                    ui.label("Click 'Refresh' to load from server");
                } else {
                    for npc in &editor_state.npcs.npc_list {
                        // Apply search filter
                        if !editor_state.npcs.search_query.is_empty() {
                            if !npc.name.to_lowercase().contains(&editor_state.npcs.search_query.to_lowercase()) {
                                continue;
                            }
                        }

                        // Apply type filter
                        if let Some(ref type_filter) = editor_state.npcs.role_filter {
                            if &npc.npc_type != type_filter {
                                continue;
                            }
                        }

                        let is_selected = editor_state.npcs.selected_npc == Some(npc.id);
                        let label = if npc.npc_type.is_empty() {
                            format!("[{}] {}", npc.id, npc.name)
                        } else {
                            format!("[{}] {} ({})", npc.id, npc.name, npc.npc_type)
                        };
                        if ui.selectable_label(is_selected, &label).clicked() {
                            editor_state.npcs.selected_npc = Some(npc.id);
                            // Load NPC for editing with expanded data
                            editor_state.npcs.editing_npc = Some(EditingNpc {
                                id: npc.id,
                                name: npc.name.clone(),
                                npc_type: npc.npc_type.clone(),
                                position_x: 0.0,
                                position_y: 0.0,
                                quests: npc.quests.clone(),
                                trainer_items: Vec::new(),
                                visual_shape: "Circle".to_string(),
                                visual_color: [0.2, 0.6, 0.9, 1.0],
                                visual_size: 20.0,
                            });
                        }
                    }
                }
            });
        });

    // Right panel - NPC properties
    egui::CentralPanel::default().show_inside(ui, |ui| {
        if let Some(ref mut editing_npc) = editor_state.npcs.editing_npc {
            ui.heading(format!("NPC #{} - {}", editing_npc.id, editing_npc.name));

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Basic properties
                ui.group(|ui| {
                    ui.heading("Basic Info");

                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        ui.label(format!("{}", editing_npc.id));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut editing_npc.name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Type:");
                        egui::ComboBox::from_id_salt("npc_type")
                            .selected_text(if editing_npc.npc_type.is_empty() { "None" } else { &editing_npc.npc_type })
                            .show_ui(ui, |ui| {
                                if ui.selectable_label(editing_npc.npc_type.is_empty(), "None").clicked() {
                                    editing_npc.npc_type.clear();
                                }
                                if ui.selectable_label(&editing_npc.npc_type == "QuestGiver", "QuestGiver").clicked() {
                                    editing_npc.npc_type = "QuestGiver".to_string();
                                }
                                if ui.selectable_label(&editing_npc.npc_type == "Trainer", "Trainer").clicked() {
                                    editing_npc.npc_type = "Trainer".to_string();
                                }
                            });
                    });
                });

                ui.separator();

                // Position
                ui.group(|ui| {
                    ui.heading("Position");

                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut editing_npc.position_x).speed(1.0));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut editing_npc.position_y).speed(1.0));
                    });
                });

                ui.separator();

                // Visual appearance
                ui.group(|ui| {
                    ui.heading("Visual Appearance");

                    ui.horizontal(|ui| {
                        ui.label("Shape:");
                        egui::ComboBox::from_id_salt("visual_shape")
                            .selected_text(&editing_npc.visual_shape)
                            .show_ui(ui, |ui| {
                                for shape in ["Circle", "Rectangle", "Triangle"] {
                                    if ui.selectable_label(&editing_npc.visual_shape == shape, shape).clicked() {
                                        editing_npc.visual_shape = shape.to_string();
                                    }
                                }
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Size:");
                        ui.add(egui::DragValue::new(&mut editing_npc.visual_size).speed(0.5).range(5.0..=100.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Color:");
                        let mut color = egui::Color32::from_rgba_unmultiplied(
                            (editing_npc.visual_color[0] * 255.0) as u8,
                            (editing_npc.visual_color[1] * 255.0) as u8,
                            (editing_npc.visual_color[2] * 255.0) as u8,
                            (editing_npc.visual_color[3] * 255.0) as u8,
                        );
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            editing_npc.visual_color = [
                                color.r() as f32 / 255.0,
                                color.g() as f32 / 255.0,
                                color.b() as f32 / 255.0,
                                color.a() as f32 / 255.0,
                            ];
                        }
                    });
                });

                ui.separator();

                // Quest Giver specific: Quests
                if editing_npc.npc_type == "QuestGiver" {
                    ui.group(|ui| {
                        ui.heading("Available Quests");

                        if ui.button("+ Add Quest").clicked() {
                            editing_npc.quests.push(1);
                        }

                        let mut quest_to_remove = None;
                        for (i, quest_id) in editing_npc.quests.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("Quest #{}:", i + 1));
                                ui.add(egui::DragValue::new(quest_id).range(1..=10000));
                                if ui.button("Remove").clicked() {
                                    quest_to_remove = Some(i);
                                }
                            });
                        }

                        if let Some(idx) = quest_to_remove {
                            editing_npc.quests.remove(idx);
                        }

                        if editing_npc.quests.is_empty() {
                            ui.label("No quests assigned. Add quests for this NPC to offer.");
                        }
                    });
                }

                // Trainer specific: Items for sale
                if editing_npc.npc_type == "Trainer" {
                    ui.group(|ui| {
                        ui.heading("Items for Sale");

                        if ui.button("+ Add Item").clicked() {
                            editing_npc.trainer_items.push(EditingTrainerItem {
                                item_id: 1,
                                cost: 100,
                            });
                        }

                        let mut item_to_remove = None;
                        for (i, item) in editing_npc.trainer_items.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("Item #{}:", i + 1));
                                ui.label("ID:");
                                ui.add(egui::DragValue::new(&mut item.item_id).range(1..=10000));
                                ui.label("Cost:");
                                ui.add(egui::DragValue::new(&mut item.cost).range(0..=100000));
                                if ui.button("Remove").clicked() {
                                    item_to_remove = Some(i);
                                }
                            });
                        }

                        if let Some(idx) = item_to_remove {
                            editing_npc.trainer_items.remove(idx);
                        }

                        if editing_npc.trainer_items.is_empty() {
                            ui.label("No items for sale. Add items this trainer will sell.");
                        }
                    });
                }

                ui.separator();

                // Actions
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        editor_state.action_save_npc = true;
                    }
                    if ui.button("Delete").on_hover_text("Delete this NPC").clicked() {
                        editor_state.action_delete_npc = true;
                    }
                });
            });
        } else if editor_state.npcs.selected_npc.is_some() {
            ui.centered_and_justified(|ui| {
                ui.label("Loading NPC data...");
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select an NPC from the list or create a new one");
            });
        }
    });

    // Create new NPC dialog
    if editor_state.npcs.show_create_dialog {
        egui::Window::new("Create New NPC")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editor_state.npcs.new_npc_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("new_npc_type")
                        .selected_text(if editor_state.npcs.new_npc_role.is_empty() { "QuestGiver" } else { &editor_state.npcs.new_npc_role })
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(&editor_state.npcs.new_npc_role == "QuestGiver" || editor_state.npcs.new_npc_role.is_empty(), "QuestGiver").clicked() {
                                editor_state.npcs.new_npc_role = "QuestGiver".to_string();
                            }
                            if ui.selectable_label(&editor_state.npcs.new_npc_role == "Trainer", "Trainer").clicked() {
                                editor_state.npcs.new_npc_role = "Trainer".to_string();
                            }
                        });
                });

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        editor_state.action_create_npc = true;
                    }
                    if ui.button("Cancel").clicked() {
                        editor_state.npcs.show_create_dialog = false;
                        editor_state.npcs.new_npc_name.clear();
                        editor_state.npcs.new_npc_role.clear();
                    }
                });
            });
    }
}
