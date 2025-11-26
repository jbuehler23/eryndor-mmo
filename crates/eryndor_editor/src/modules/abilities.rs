//! Abilities Editor Module
//! Create and edit abilities with composable effects.

use bevy_egui::egui;
use crate::editor_state::{EditorState, EditingAbility};

/// Render the abilities editor module
pub fn render(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    // Render side panel first so it claims its space
    egui::SidePanel::left("abilities_list_panel")
        .default_width(250.0)
        .show_inside(ui, |ui| {
            ui.heading("Abilities");

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("+ New Ability").clicked() {
                    editor_state.abilities.show_create_dialog = true;
                }
                if ui.button("Refresh").clicked() {
                    editor_state.action_load_abilities = true;
                }
            });

            ui.separator();

            // Search
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut editor_state.abilities.search_query);
            });

            // Type filter
            egui::ComboBox::from_label("Type")
                .selected_text(editor_state.abilities.type_filter.as_deref().unwrap_or("All"))
                .show_ui(ui, |ui| {
                    if ui.selectable_label(editor_state.abilities.type_filter.is_none(), "All").clicked() {
                        editor_state.abilities.type_filter = None;
                    }
                    for ability_type in ["Active", "Passive", "Triggered", "Enemy-only"] {
                        if ui.selectable_label(editor_state.abilities.type_filter.as_deref() == Some(ability_type), ability_type).clicked() {
                            editor_state.abilities.type_filter = Some(ability_type.to_string());
                        }
                    }
                });

            ui.separator();

            // Ability list
            egui::ScrollArea::vertical().show(ui, |ui| {
                if editor_state.abilities.ability_list.is_empty() {
                    ui.label("No abilities loaded");
                    ui.label("Click 'Refresh' to load from server");
                } else {
                    for ability in &editor_state.abilities.ability_list {
                        // Apply search filter
                        if !editor_state.abilities.search_query.is_empty() {
                            if !ability.name.to_lowercase().contains(&editor_state.abilities.search_query.to_lowercase()) {
                                continue;
                            }
                        }

                        // Apply type filter
                        if let Some(ref type_filter) = editor_state.abilities.type_filter {
                            if &ability.ability_type != type_filter {
                                continue;
                            }
                        }

                        let is_selected = editor_state.abilities.selected_ability == Some(ability.id);
                        let label = if ability.ability_type.is_empty() {
                            ability.name.clone()
                        } else {
                            format!("[{}] {}", ability.ability_type, ability.name)
                        };
                        if ui.selectable_label(is_selected, &label).clicked() {
                            editor_state.abilities.selected_ability = Some(ability.id);
                            // Load ability for editing
                            editor_state.abilities.editing_ability = Some(EditingAbility {
                                id: ability.id,
                                name: ability.name.clone(),
                                ability_type: ability.ability_type.clone(),
                            });
                        }
                    }
                }
            });
        });

    // Right panel - ability properties
    egui::CentralPanel::default().show_inside(ui, |ui| {
        if let Some(ref mut editing_ability) = editor_state.abilities.editing_ability {
            ui.heading(format!("Ability #{} - {}", editing_ability.id, editing_ability.name));

            ui.separator();

            // Basic properties
            ui.group(|ui| {
                ui.heading("Basic Info");

                ui.horizontal(|ui| {
                    ui.label("ID:");
                    ui.label(format!("{}", editing_ability.id));
                });

                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editing_ability.name);
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("ability_type")
                        .selected_text(if editing_ability.ability_type.is_empty() { "None" } else { &editing_ability.ability_type })
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(editing_ability.ability_type.is_empty(), "None").clicked() {
                                editing_ability.ability_type.clear();
                            }
                            for ability_type in ["Active", "Passive", "Triggered", "Enemy-only"] {
                                if ui.selectable_label(&editing_ability.ability_type == ability_type, ability_type).clicked() {
                                    editing_ability.ability_type = ability_type.to_string();
                                }
                            }
                        });
                });
            });

            ui.separator();

            // Actions
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    editor_state.action_save_ability = true;
                }
                if ui.button("Delete").on_hover_text("Delete this ability").clicked() {
                    editor_state.action_delete_ability = true;
                }
            });
        } else if editor_state.abilities.selected_ability.is_some() {
            ui.centered_and_justified(|ui| {
                ui.label("Loading ability data...");
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select an ability from the list or create a new one");
            });
        }
    });

    // Create new ability dialog
    if editor_state.abilities.show_create_dialog {
        egui::Window::new("Create New Ability")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editor_state.abilities.new_ability_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("new_ability_type")
                        .selected_text(if editor_state.abilities.new_ability_type.is_empty() { "None" } else { &editor_state.abilities.new_ability_type })
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(editor_state.abilities.new_ability_type.is_empty(), "None").clicked() {
                                editor_state.abilities.new_ability_type.clear();
                            }
                            for ability_type in ["Active", "Passive", "Triggered", "Enemy-only"] {
                                if ui.selectable_label(&editor_state.abilities.new_ability_type == ability_type, ability_type).clicked() {
                                    editor_state.abilities.new_ability_type = ability_type.to_string();
                                }
                            }
                        });
                });

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        editor_state.action_create_ability = true;
                    }
                    if ui.button("Cancel").clicked() {
                        editor_state.abilities.show_create_dialog = false;
                        editor_state.abilities.new_ability_name.clear();
                        editor_state.abilities.new_ability_type.clear();
                    }
                });
            });
    }
}
