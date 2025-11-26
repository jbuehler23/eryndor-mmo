//! NPCs Editor Module
//! Create and edit NPCs with various roles.

use bevy_egui::egui;
use crate::editor_state::{EditorState, EditingNpc};

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

            // Role filter
            egui::ComboBox::from_label("Role")
                .selected_text(editor_state.npcs.role_filter.as_deref().unwrap_or("All"))
                .show_ui(ui, |ui| {
                    if ui.selectable_label(editor_state.npcs.role_filter.is_none(), "All").clicked() {
                        editor_state.npcs.role_filter = None;
                    }
                    for role in ["Quest Giver", "Trainer", "Merchant", "Banker", "Innkeeper", "Guard", "Generic"] {
                        if ui.selectable_label(editor_state.npcs.role_filter.as_deref() == Some(role), role).clicked() {
                            editor_state.npcs.role_filter = Some(role.to_string());
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

                        // Apply role filter
                        if let Some(ref role_filter) = editor_state.npcs.role_filter {
                            if &npc.role != role_filter {
                                continue;
                            }
                        }

                        let is_selected = editor_state.npcs.selected_npc == Some(npc.id);
                        let label = if npc.role.is_empty() {
                            npc.name.clone()
                        } else {
                            format!("[{}] {}", npc.role, npc.name)
                        };
                        if ui.selectable_label(is_selected, &label).clicked() {
                            editor_state.npcs.selected_npc = Some(npc.id);
                            // Load NPC for editing
                            editor_state.npcs.editing_npc = Some(EditingNpc {
                                id: npc.id,
                                name: npc.name.clone(),
                                role: npc.role.clone(),
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
                    ui.label("Role:");
                    egui::ComboBox::from_id_salt("npc_role")
                        .selected_text(if editing_npc.role.is_empty() { "None" } else { &editing_npc.role })
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(editing_npc.role.is_empty(), "None").clicked() {
                                editing_npc.role.clear();
                            }
                            for role in ["Quest Giver", "Trainer", "Merchant", "Banker", "Innkeeper", "Guard", "Generic"] {
                                if ui.selectable_label(&editing_npc.role == role, role).clicked() {
                                    editing_npc.role = role.to_string();
                                }
                            }
                        });
                });
            });

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
                    ui.label("Role:");
                    egui::ComboBox::from_id_salt("new_npc_role")
                        .selected_text(if editor_state.npcs.new_npc_role.is_empty() { "None" } else { &editor_state.npcs.new_npc_role })
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(editor_state.npcs.new_npc_role.is_empty(), "None").clicked() {
                                editor_state.npcs.new_npc_role.clear();
                            }
                            for role in ["Quest Giver", "Trainer", "Merchant", "Banker", "Innkeeper", "Guard", "Generic"] {
                                if ui.selectable_label(&editor_state.npcs.new_npc_role == role, role).clicked() {
                                    editor_state.npcs.new_npc_role = role.to_string();
                                }
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
