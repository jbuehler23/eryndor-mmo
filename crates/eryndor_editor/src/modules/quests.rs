//! Quests Editor Module
//! Create and edit quests with objectives, rewards, and requirements.

use bevy_egui::egui;
use crate::editor_state::{EditorState, EditingQuest, EditingQuestObjective, EditingProficiencyRequirement};

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

                        let is_selected = editor_state.quests.selected_quest == Some(quest.id);
                        let label = format!("[{}] {} ({} XP)", quest.id, quest.name, quest.reward_exp);
                        if ui.selectable_label(is_selected, &label).clicked() {
                            editor_state.quests.selected_quest = Some(quest.id);
                            // Load quest for editing with expanded data
                            editor_state.quests.editing_quest = Some(EditingQuest {
                                id: quest.id,
                                name: quest.name.clone(),
                                description: quest.description.clone(),
                                objectives: Vec::new(),
                                reward_exp: quest.reward_exp,
                                proficiency_requirements: Vec::new(),
                                reward_abilities: Vec::new(),
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

            egui::ScrollArea::vertical().show(ui, |ui| {
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
                        ui.label("Description:");
                    });
                    ui.text_edit_multiline(&mut editing_quest.description);
                });

                ui.separator();

                // Objectives
                ui.group(|ui| {
                    ui.heading("Quest Objectives");

                    if ui.button("+ Add Objective").clicked() {
                        editing_quest.objectives.push(EditingQuestObjective::TalkToNpc { npc_id: 1 });
                    }

                    let mut objective_to_remove = None;
                    for (i, objective) in editing_quest.objectives.iter_mut().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("Objective #{}:", i + 1));
                                if ui.button("Remove").clicked() {
                                    objective_to_remove = Some(i);
                                }
                            });

                            let obj_type = match objective {
                                EditingQuestObjective::TalkToNpc { .. } => "Talk to NPC",
                                EditingQuestObjective::KillEnemy { .. } => "Kill Enemy",
                                EditingQuestObjective::ObtainItem { .. } => "Obtain Item",
                            };

                            egui::ComboBox::from_id_salt(format!("objective_type_{}", i))
                                .selected_text(obj_type)
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(matches!(objective, EditingQuestObjective::TalkToNpc { .. }), "Talk to NPC").clicked() {
                                        *objective = EditingQuestObjective::TalkToNpc { npc_id: 1 };
                                    }
                                    if ui.selectable_label(matches!(objective, EditingQuestObjective::KillEnemy { .. }), "Kill Enemy").clicked() {
                                        *objective = EditingQuestObjective::KillEnemy { enemy_type: 1, count: 5 };
                                    }
                                    if ui.selectable_label(matches!(objective, EditingQuestObjective::ObtainItem { .. }), "Obtain Item").clicked() {
                                        *objective = EditingQuestObjective::ObtainItem { item_id: 1, count: 1 };
                                    }
                                });

                            // Objective-specific fields
                            match objective {
                                EditingQuestObjective::TalkToNpc { npc_id } => {
                                    ui.horizontal(|ui| {
                                        ui.label("NPC ID:");
                                        ui.add(egui::DragValue::new(npc_id).range(1..=10000));
                                    });
                                }
                                EditingQuestObjective::KillEnemy { enemy_type, count } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Enemy Type ID:");
                                        ui.add(egui::DragValue::new(enemy_type).range(1..=10000));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Kill Count:");
                                        ui.add(egui::DragValue::new(count).range(1..=1000));
                                    });
                                }
                                EditingQuestObjective::ObtainItem { item_id, count } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Item ID:");
                                        ui.add(egui::DragValue::new(item_id).range(1..=10000));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Count:");
                                        ui.add(egui::DragValue::new(count).range(1..=1000));
                                    });
                                }
                            }
                        });
                    }

                    if let Some(idx) = objective_to_remove {
                        editing_quest.objectives.remove(idx);
                    }

                    if editing_quest.objectives.is_empty() {
                        ui.label("No objectives. Add objectives for players to complete.");
                    }
                });

                ui.separator();

                // Rewards
                ui.group(|ui| {
                    ui.heading("Rewards");

                    ui.horizontal(|ui| {
                        ui.label("Experience Points:");
                        ui.add(egui::DragValue::new(&mut editing_quest.reward_exp).range(0..=1000000));
                    });

                    ui.separator();

                    ui.label("Reward Abilities:");
                    if ui.button("+ Add Ability Reward").clicked() {
                        editing_quest.reward_abilities.push(100);
                    }

                    let mut ability_to_remove = None;
                    for (i, ability_id) in editing_quest.reward_abilities.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("Ability #{}:", i + 1));
                            ui.add(egui::DragValue::new(ability_id).range(1..=10000));
                            if ui.button("Remove").clicked() {
                                ability_to_remove = Some(i);
                            }
                        });
                    }

                    if let Some(idx) = ability_to_remove {
                        editing_quest.reward_abilities.remove(idx);
                    }
                });

                ui.separator();

                // Requirements
                ui.group(|ui| {
                    ui.heading("Proficiency Requirements");

                    if ui.button("+ Add Requirement").clicked() {
                        editing_quest.proficiency_requirements.push(EditingProficiencyRequirement {
                            weapon_type: "Sword".to_string(),
                            level: 1,
                        });
                    }

                    let mut req_to_remove = None;
                    for (i, req) in editing_quest.proficiency_requirements.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("Requirement #{}:", i + 1));

                            egui::ComboBox::from_id_salt(format!("weapon_type_{}", i))
                                .selected_text(&req.weapon_type)
                                .show_ui(ui, |ui| {
                                    for weapon in ["Sword", "Dagger", "Wand", "Staff", "Mace", "Bow", "Axe"] {
                                        if ui.selectable_label(&req.weapon_type == weapon, weapon).clicked() {
                                            req.weapon_type = weapon.to_string();
                                        }
                                    }
                                });

                            ui.label("Level:");
                            ui.add(egui::DragValue::new(&mut req.level).range(1..=100));

                            if ui.button("Remove").clicked() {
                                req_to_remove = Some(i);
                            }
                        });
                    }

                    if let Some(idx) = req_to_remove {
                        editing_quest.proficiency_requirements.remove(idx);
                    }

                    if editing_quest.proficiency_requirements.is_empty() {
                        ui.label("No requirements. Quest available to all players.");
                    }
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
