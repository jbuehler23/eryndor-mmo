//! Quests Editor Module
//! Create and edit quests with objectives and dialogue.

use bevy_egui::egui;
use crate::editor_state::{EditorState, EditingQuest};

/// Render the quests editor module
pub fn render(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    // Render side panel first so it claims its space
    egui::SidePanel::left("quests_list_panel")
        .default_width(250.0)
        .show_inside(ui, |ui| {
            ui.heading("Quests");

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("+ New Quest").clicked() {
                    editor_state.quests.show_create_dialog = true;
                }
                if ui.button("Refresh").clicked() {
                    editor_state.action_load_quests = true;
                }
            });

            ui.separator();

            // Search
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut editor_state.quests.search_query);
            });

            // Type filter
            egui::ComboBox::from_label("Type")
                .selected_text(editor_state.quests.type_filter.as_deref().unwrap_or("All"))
                .show_ui(ui, |ui| {
                    if ui.selectable_label(editor_state.quests.type_filter.is_none(), "All").clicked() {
                        editor_state.quests.type_filter = None;
                    }
                    for quest_type in ["Main Story", "Side Quest", "Daily", "World Event", "Achievement"] {
                        if ui.selectable_label(editor_state.quests.type_filter.as_deref() == Some(quest_type), quest_type).clicked() {
                            editor_state.quests.type_filter = Some(quest_type.to_string());
                        }
                    }
                });

            ui.separator();

            // Quest list
            egui::ScrollArea::vertical().show(ui, |ui| {
                if editor_state.quests.quest_list.is_empty() {
                    ui.label("No quests loaded");
                    ui.label("Click 'Refresh' to load from server");
                } else {
                    for quest in &editor_state.quests.quest_list {
                        // Apply search filter
                        if !editor_state.quests.search_query.is_empty() {
                            if !quest.name.to_lowercase().contains(&editor_state.quests.search_query.to_lowercase()) {
                                continue;
                            }
                        }

                        // Apply type filter
                        if let Some(ref type_filter) = editor_state.quests.type_filter {
                            if &quest.quest_type != type_filter {
                                continue;
                            }
                        }

                        let is_selected = editor_state.quests.selected_quest == Some(quest.id);
                        let label = if quest.quest_type.is_empty() {
                            quest.name.clone()
                        } else {
                            format!("[{}] {}", quest.quest_type, quest.name)
                        };
                        if ui.selectable_label(is_selected, &label).clicked() {
                            editor_state.quests.selected_quest = Some(quest.id);
                            // Load quest for editing
                            editor_state.quests.editing_quest = Some(EditingQuest {
                                id: quest.id,
                                name: quest.name.clone(),
                                quest_type: quest.quest_type.clone(),
                            });
                        }
                    }
                }
            });
        });

    // Right panel - quest properties
    egui::CentralPanel::default().show_inside(ui, |ui| {
        if let Some(ref mut editing_quest) = editor_state.quests.editing_quest {
            ui.heading(format!("Quest #{} - {}", editing_quest.id, editing_quest.name));

            ui.separator();

            // Basic properties
            ui.group(|ui| {
                ui.heading("Basic Info");

                ui.horizontal(|ui| {
                    ui.label("ID:");
                    ui.label(format!("{}", editing_quest.id));
                });

                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editing_quest.name);
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("quest_type")
                        .selected_text(if editing_quest.quest_type.is_empty() { "None" } else { &editing_quest.quest_type })
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(editing_quest.quest_type.is_empty(), "None").clicked() {
                                editing_quest.quest_type.clear();
                            }
                            for quest_type in ["Main Story", "Side Quest", "Daily", "World Event", "Achievement"] {
                                if ui.selectable_label(&editing_quest.quest_type == quest_type, quest_type).clicked() {
                                    editing_quest.quest_type = quest_type.to_string();
                                }
                            }
                        });
                });
            });

            ui.separator();

            // Actions
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    editor_state.action_save_quest = true;
                }
                if ui.button("Delete").on_hover_text("Delete this quest").clicked() {
                    editor_state.action_delete_quest = true;
                }
            });
        } else if editor_state.quests.selected_quest.is_some() {
            ui.centered_and_justified(|ui| {
                ui.label("Loading quest data...");
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select a quest from the list or create a new one");
            });
        }
    });

    // Create new quest dialog
    if editor_state.quests.show_create_dialog {
        egui::Window::new("Create New Quest")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editor_state.quests.new_quest_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("new_quest_type")
                        .selected_text(if editor_state.quests.new_quest_type.is_empty() { "None" } else { &editor_state.quests.new_quest_type })
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(editor_state.quests.new_quest_type.is_empty(), "None").clicked() {
                                editor_state.quests.new_quest_type.clear();
                            }
                            for quest_type in ["Main Story", "Side Quest", "Daily", "World Event", "Achievement"] {
                                if ui.selectable_label(&editor_state.quests.new_quest_type == quest_type, quest_type).clicked() {
                                    editor_state.quests.new_quest_type = quest_type.to_string();
                                }
                            }
                        });
                });

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        editor_state.action_create_quest = true;
                    }
                    if ui.button("Cancel").clicked() {
                        editor_state.quests.show_create_dialog = false;
                        editor_state.quests.new_quest_name.clear();
                        editor_state.quests.new_quest_type.clear();
                    }
                });
            });
    }
}
