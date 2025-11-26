//! Loot Tables Editor Module
//! Create and manage drop tables including world drops.

use bevy_egui::egui;
use crate::editor_state::{EditorState, EditingLootTable};

/// Render the loot tables editor module
pub fn render(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    // Render side panel first so it claims its space
    egui::SidePanel::left("loot_list_panel")
        .default_width(250.0)
        .show_inside(ui, |ui| {
            ui.heading("Loot Tables");

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("+ New Loot Table").clicked() {
                    editor_state.loot.show_create_dialog = true;
                }
                if ui.button("Refresh").clicked() {
                    editor_state.action_load_loot_tables = true;
                }
            });

            ui.separator();

            // Search
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut editor_state.loot.search_query);
            });

            // Type filter
            egui::ComboBox::from_label("Type")
                .selected_text(editor_state.loot.type_filter.as_deref().unwrap_or("All"))
                .show_ui(ui, |ui| {
                    if ui.selectable_label(editor_state.loot.type_filter.is_none(), "All").clicked() {
                        editor_state.loot.type_filter = None;
                    }
                    for table_type in ["Enemy Drops", "World Drops", "Boss Drops", "Chest Loot", "Gathering"] {
                        if ui.selectable_label(editor_state.loot.type_filter.as_deref() == Some(table_type), table_type).clicked() {
                            editor_state.loot.type_filter = Some(table_type.to_string());
                        }
                    }
                });

            ui.separator();

            // Loot table list
            egui::ScrollArea::vertical().show(ui, |ui| {
                if editor_state.loot.loot_table_list.is_empty() {
                    ui.label("No loot tables loaded");
                    ui.label("Click 'Refresh' to load from server");
                } else {
                    for table in &editor_state.loot.loot_table_list {
                        // Apply search filter
                        if !editor_state.loot.search_query.is_empty() {
                            if !table.name.to_lowercase().contains(&editor_state.loot.search_query.to_lowercase()) {
                                continue;
                            }
                        }

                        // Apply type filter
                        if let Some(ref type_filter) = editor_state.loot.type_filter {
                            if &table.table_type != type_filter {
                                continue;
                            }
                        }

                        let is_selected = editor_state.loot.selected_loot_table.as_ref() == Some(&table.id);
                        let label = if table.table_type.is_empty() {
                            table.name.clone()
                        } else {
                            format!("[{}] {}", table.table_type, table.name)
                        };
                        if ui.selectable_label(is_selected, &label).clicked() {
                            editor_state.loot.selected_loot_table = Some(table.id.clone());
                            // Load loot table for editing
                            editor_state.loot.editing_loot_table = Some(EditingLootTable {
                                id: table.id.clone(),
                                name: table.name.clone(),
                                table_type: table.table_type.clone(),
                            });
                        }
                    }
                }
            });
        });

    // Right panel - loot table properties
    egui::CentralPanel::default().show_inside(ui, |ui| {
        if let Some(ref mut editing_loot_table) = editor_state.loot.editing_loot_table {
            ui.heading(format!("Loot Table: {} - {}", editing_loot_table.id, editing_loot_table.name));

            ui.separator();

            // Basic properties
            ui.group(|ui| {
                ui.heading("Basic Info");

                ui.horizontal(|ui| {
                    ui.label("ID:");
                    ui.label(&editing_loot_table.id);
                });

                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editing_loot_table.name);
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("loot_table_type")
                        .selected_text(if editing_loot_table.table_type.is_empty() { "None" } else { &editing_loot_table.table_type })
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(editing_loot_table.table_type.is_empty(), "None").clicked() {
                                editing_loot_table.table_type.clear();
                            }
                            for table_type in ["Enemy Drops", "World Drops", "Boss Drops", "Chest Loot", "Gathering"] {
                                if ui.selectable_label(&editing_loot_table.table_type == table_type, table_type).clicked() {
                                    editing_loot_table.table_type = table_type.to_string();
                                }
                            }
                        });
                });
            });

            ui.separator();

            // Actions
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    editor_state.action_save_loot_table = true;
                }
                if ui.button("Delete").on_hover_text("Delete this loot table").clicked() {
                    editor_state.action_delete_loot_table = true;
                }
            });
        } else if editor_state.loot.selected_loot_table.is_some() {
            ui.centered_and_justified(|ui| {
                ui.label("Loading loot table data...");
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select a loot table from the list or create a new one");
            });
        }
    });

    // Create new loot table dialog
    if editor_state.loot.show_create_dialog {
        egui::Window::new("Create New Loot Table")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editor_state.loot.new_loot_table_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("new_loot_table_type")
                        .selected_text(if editor_state.loot.new_loot_table_type.is_empty() { "None" } else { &editor_state.loot.new_loot_table_type })
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(editor_state.loot.new_loot_table_type.is_empty(), "None").clicked() {
                                editor_state.loot.new_loot_table_type.clear();
                            }
                            for table_type in ["Enemy Drops", "World Drops", "Boss Drops", "Chest Loot", "Gathering"] {
                                if ui.selectable_label(&editor_state.loot.new_loot_table_type == table_type, table_type).clicked() {
                                    editor_state.loot.new_loot_table_type = table_type.to_string();
                                }
                            }
                        });
                });

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        editor_state.action_create_loot_table = true;
                    }
                    if ui.button("Cancel").clicked() {
                        editor_state.loot.show_create_dialog = false;
                        editor_state.loot.new_loot_table_name.clear();
                        editor_state.loot.new_loot_table_type.clear();
                    }
                });
            });
    }
}
