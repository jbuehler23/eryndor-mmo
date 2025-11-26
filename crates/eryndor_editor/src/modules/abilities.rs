//! Abilities Editor Module
//! Create and edit abilities with composable effects.

use bevy_egui::egui;
use crate::editor_state::{EditorState, EditingAbility, EditingAbilityEffect, EditingDebuffType, EditingUnlockRequirement};

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

                        let is_selected = editor_state.abilities.selected_ability == Some(ability.id);
                        let label = format!("[{}] {}", ability.id, ability.name);
                        if ui.selectable_label(is_selected, &label).clicked() {
                            editor_state.abilities.selected_ability = Some(ability.id);
                            // Load ability for editing with full data
                            editor_state.abilities.editing_ability = Some(EditingAbility {
                                id: ability.id,
                                name: ability.name.clone(),
                                description: ability.description.clone(),
                                damage_multiplier: ability.damage_multiplier,
                                cooldown: ability.cooldown,
                                range: 0.0, // Will be loaded from full data
                                mana_cost: ability.mana_cost,
                                ability_effects: Vec::new(), // Will be loaded from full data
                                unlock_requirement: EditingUnlockRequirement::None,
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

            egui::ScrollArea::vertical().show(ui, |ui| {
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
                        ui.label("Description:");
                    });
                    ui.text_edit_multiline(&mut editing_ability.description);
                });

                ui.separator();

                // Combat stats
                ui.group(|ui| {
                    ui.heading("Combat Stats");

                    ui.horizontal(|ui| {
                        ui.label("Damage Multiplier:");
                        ui.add(egui::DragValue::new(&mut editing_ability.damage_multiplier)
                            .speed(0.1)
                            .range(0.0..=10.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Cooldown (seconds):");
                        ui.add(egui::DragValue::new(&mut editing_ability.cooldown)
                            .speed(0.1)
                            .range(0.0..=300.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Range:");
                        ui.add(egui::DragValue::new(&mut editing_ability.range)
                            .speed(0.1)
                            .range(0.0..=100.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Mana Cost:");
                        ui.add(egui::DragValue::new(&mut editing_ability.mana_cost)
                            .speed(1.0)
                            .range(0.0..=1000.0));
                    });
                });

                ui.separator();

                // Unlock requirement
                ui.group(|ui| {
                    ui.heading("Unlock Requirement");

                    let req_label = match &editing_ability.unlock_requirement {
                        EditingUnlockRequirement::None => "None (Starting Ability)".to_string(),
                        EditingUnlockRequirement::Level(lvl) => format!("Level {}", lvl),
                        EditingUnlockRequirement::Quest(q) => format!("Quest #{}", q),
                        EditingUnlockRequirement::WeaponProficiency { weapon, level } => {
                            format!("{} Proficiency {}", weapon, level)
                        }
                    };

                    egui::ComboBox::from_id_salt("unlock_requirement")
                        .selected_text(&req_label)
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(matches!(editing_ability.unlock_requirement, EditingUnlockRequirement::None), "None (Starting Ability)").clicked() {
                                editing_ability.unlock_requirement = EditingUnlockRequirement::None;
                            }
                            if ui.selectable_label(matches!(editing_ability.unlock_requirement, EditingUnlockRequirement::Level(_)), "Level Requirement").clicked() {
                                editing_ability.unlock_requirement = EditingUnlockRequirement::Level(1);
                            }
                            if ui.selectable_label(matches!(editing_ability.unlock_requirement, EditingUnlockRequirement::Quest(_)), "Quest Completion").clicked() {
                                editing_ability.unlock_requirement = EditingUnlockRequirement::Quest(1);
                            }
                            if ui.selectable_label(matches!(editing_ability.unlock_requirement, EditingUnlockRequirement::WeaponProficiency { .. }), "Weapon Proficiency").clicked() {
                                editing_ability.unlock_requirement = EditingUnlockRequirement::WeaponProficiency {
                                    weapon: "Sword".to_string(),
                                    level: 1,
                                };
                            }
                        });

                    // Show requirement-specific fields
                    match &mut editing_ability.unlock_requirement {
                        EditingUnlockRequirement::Level(level) => {
                            ui.horizontal(|ui| {
                                ui.label("Required Level:");
                                ui.add(egui::DragValue::new(level).range(1..=100));
                            });
                        }
                        EditingUnlockRequirement::Quest(quest_id) => {
                            ui.horizontal(|ui| {
                                ui.label("Required Quest ID:");
                                ui.add(egui::DragValue::new(quest_id).range(1..=10000));
                            });
                        }
                        EditingUnlockRequirement::WeaponProficiency { weapon, level } => {
                            ui.horizontal(|ui| {
                                ui.label("Weapon Type:");
                                egui::ComboBox::from_id_salt("weapon_type")
                                    .selected_text(weapon.as_str())
                                    .show_ui(ui, |ui| {
                                        for w in ["Sword", "Dagger", "Wand", "Staff", "Bow", "Axe", "Mace"] {
                                            if ui.selectable_label(weapon == w, w).clicked() {
                                                *weapon = w.to_string();
                                            }
                                        }
                                    });
                            });
                            ui.horizontal(|ui| {
                                ui.label("Required Proficiency Level:");
                                ui.add(egui::DragValue::new(level).range(1..=100));
                            });
                        }
                        EditingUnlockRequirement::None => {}
                    }
                });

                ui.separator();

                // Ability effects
                ui.group(|ui| {
                    ui.heading("Ability Effects");

                    if ui.button("+ Add Effect").clicked() {
                        editing_ability.ability_effects.push(EditingAbilityEffect::DirectDamage { multiplier: 1.0 });
                    }

                    let mut effect_to_remove = None;
                    for (i, effect) in editing_ability.ability_effects.iter_mut().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("Effect #{}", i + 1));
                                if ui.button("Remove").clicked() {
                                    effect_to_remove = Some(i);
                                }
                            });

                            let effect_type = match effect {
                                EditingAbilityEffect::DirectDamage { .. } => "Direct Damage",
                                EditingAbilityEffect::DamageOverTime { .. } => "Damage Over Time",
                                EditingAbilityEffect::AreaOfEffect { .. } => "Area of Effect",
                                EditingAbilityEffect::Buff { .. } => "Buff",
                                EditingAbilityEffect::Debuff { .. } => "Debuff",
                                EditingAbilityEffect::Mobility { .. } => "Mobility",
                                EditingAbilityEffect::Heal { .. } => "Heal",
                            };

                            egui::ComboBox::from_id_salt(format!("effect_type_{}", i))
                                .selected_text(effect_type)
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(matches!(effect, EditingAbilityEffect::DirectDamage { .. }), "Direct Damage").clicked() {
                                        *effect = EditingAbilityEffect::DirectDamage { multiplier: 1.0 };
                                    }
                                    if ui.selectable_label(matches!(effect, EditingAbilityEffect::DamageOverTime { .. }), "Damage Over Time").clicked() {
                                        *effect = EditingAbilityEffect::DamageOverTime { duration: 6.0, ticks: 6, damage_per_tick: 5.0 };
                                    }
                                    if ui.selectable_label(matches!(effect, EditingAbilityEffect::AreaOfEffect { .. }), "Area of Effect").clicked() {
                                        *effect = EditingAbilityEffect::AreaOfEffect { radius: 5.0, max_targets: 5 };
                                    }
                                    if ui.selectable_label(matches!(effect, EditingAbilityEffect::Buff { .. }), "Buff").clicked() {
                                        *effect = EditingAbilityEffect::Buff { duration: 10.0, attack_power: 0.0, defense: 0.0, move_speed: 0.0 };
                                    }
                                    if ui.selectable_label(matches!(effect, EditingAbilityEffect::Debuff { .. }), "Debuff").clicked() {
                                        *effect = EditingAbilityEffect::Debuff { duration: 4.0, debuff_type: EditingDebuffType::Stun };
                                    }
                                    if ui.selectable_label(matches!(effect, EditingAbilityEffect::Mobility { .. }), "Mobility").clicked() {
                                        *effect = EditingAbilityEffect::Mobility { distance: 8.0, dash_speed: 20.0 };
                                    }
                                    if ui.selectable_label(matches!(effect, EditingAbilityEffect::Heal { .. }), "Heal").clicked() {
                                        *effect = EditingAbilityEffect::Heal { amount: 0.2, is_percent: true };
                                    }
                                });

                            // Effect-specific fields
                            match effect {
                                EditingAbilityEffect::DirectDamage { multiplier } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Damage Multiplier:");
                                        ui.add(egui::DragValue::new(multiplier).speed(0.1).range(0.0..=10.0));
                                    });
                                }
                                EditingAbilityEffect::DamageOverTime { duration, ticks, damage_per_tick } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Duration:");
                                        ui.add(egui::DragValue::new(duration).speed(0.1).range(0.0..=60.0));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Ticks:");
                                        ui.add(egui::DragValue::new(ticks).range(1..=60));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Damage per Tick:");
                                        ui.add(egui::DragValue::new(damage_per_tick).speed(0.5).range(0.0..=100.0));
                                    });
                                }
                                EditingAbilityEffect::AreaOfEffect { radius, max_targets } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Radius:");
                                        ui.add(egui::DragValue::new(radius).speed(0.1).range(0.0..=50.0));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Max Targets:");
                                        ui.add(egui::DragValue::new(max_targets).range(1..=100));
                                    });
                                }
                                EditingAbilityEffect::Buff { duration, attack_power, defense, move_speed } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Duration:");
                                        ui.add(egui::DragValue::new(duration).speed(0.1).range(0.0..=300.0));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Attack Power Bonus:");
                                        ui.add(egui::DragValue::new(attack_power).speed(0.1).range(-100.0..=100.0));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Defense Bonus:");
                                        ui.add(egui::DragValue::new(defense).speed(0.1).range(-100.0..=100.0));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Move Speed Bonus:");
                                        ui.add(egui::DragValue::new(move_speed).speed(0.1).range(-100.0..=100.0));
                                    });
                                }
                                EditingAbilityEffect::Debuff { duration, debuff_type } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Duration:");
                                        ui.add(egui::DragValue::new(duration).speed(0.1).range(0.0..=60.0));
                                    });

                                    let debuff_label = match debuff_type {
                                        EditingDebuffType::Stun => "Stun",
                                        EditingDebuffType::Root => "Root",
                                        EditingDebuffType::Slow { .. } => "Slow",
                                        EditingDebuffType::Weaken { .. } => "Weaken",
                                    };

                                    egui::ComboBox::from_id_salt(format!("debuff_type_{}", i))
                                        .selected_text(debuff_label)
                                        .show_ui(ui, |ui| {
                                            if ui.selectable_label(matches!(debuff_type, EditingDebuffType::Stun), "Stun").clicked() {
                                                *debuff_type = EditingDebuffType::Stun;
                                            }
                                            if ui.selectable_label(matches!(debuff_type, EditingDebuffType::Root), "Root").clicked() {
                                                *debuff_type = EditingDebuffType::Root;
                                            }
                                            if ui.selectable_label(matches!(debuff_type, EditingDebuffType::Slow { .. }), "Slow").clicked() {
                                                *debuff_type = EditingDebuffType::Slow { move_speed_reduction: 0.5 };
                                            }
                                            if ui.selectable_label(matches!(debuff_type, EditingDebuffType::Weaken { .. }), "Weaken").clicked() {
                                                *debuff_type = EditingDebuffType::Weaken { attack_reduction: 0.3 };
                                            }
                                        });

                                    match debuff_type {
                                        EditingDebuffType::Slow { move_speed_reduction } => {
                                            ui.horizontal(|ui| {
                                                ui.label("Speed Reduction %:");
                                                ui.add(egui::DragValue::new(move_speed_reduction).speed(0.01).range(0.0..=1.0));
                                            });
                                        }
                                        EditingDebuffType::Weaken { attack_reduction } => {
                                            ui.horizontal(|ui| {
                                                ui.label("Attack Reduction %:");
                                                ui.add(egui::DragValue::new(attack_reduction).speed(0.01).range(0.0..=1.0));
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                                EditingAbilityEffect::Mobility { distance, dash_speed } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Distance:");
                                        ui.add(egui::DragValue::new(distance).speed(0.1).range(0.0..=50.0));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Dash Speed:");
                                        ui.add(egui::DragValue::new(dash_speed).speed(1.0).range(0.0..=200.0));
                                    });
                                }
                                EditingAbilityEffect::Heal { amount, is_percent } => {
                                    ui.horizontal(|ui| {
                                        ui.label("Heal Amount:");
                                        ui.add(egui::DragValue::new(amount).speed(0.01).range(0.0..=if *is_percent { 1.0 } else { 1000.0 }));
                                    });
                                    ui.checkbox(is_percent, "Is Percentage of Max HP");
                                }
                            }
                        });
                    }

                    if let Some(idx) = effect_to_remove {
                        editing_ability.ability_effects.remove(idx);
                    }
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
