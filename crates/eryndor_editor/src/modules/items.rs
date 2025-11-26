//! Items Editor Module
//! Create and edit weapons, armor, and consumables.

use bevy_egui::egui;
use crate::editor_state::{EditorState, EditingItem};

const ITEM_TYPES: &[&str] = &["Weapon", "Helmet", "Chest", "Legs", "Boots", "Consumable", "Quest Item"];

/// Render the items editor module
pub fn render(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    // Render side panel first so it claims its space
    egui::SidePanel::left("items_list_panel")
        .default_width(250.0)
        .show_inside(ui, |ui| {
            ui.heading("Items");

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("+ New Item").clicked() {
                    editor_state.items.show_create_dialog = true;
                    editor_state.items.new_item_type = "Weapon".to_string();
                }
                if ui.button("Refresh").clicked() {
                    editor_state.action_load_items = true;
                }
            });

            ui.separator();

            // Search
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut editor_state.items.search_query);
            });

            // Type filter
            egui::ComboBox::from_label("Type")
                .selected_text(editor_state.items.type_filter.as_deref().unwrap_or("All"))
                .show_ui(ui, |ui| {
                    if ui.selectable_label(editor_state.items.type_filter.is_none(), "All").clicked() {
                        editor_state.items.type_filter = None;
                    }
                    for item_type in ITEM_TYPES {
                        if ui.selectable_label(editor_state.items.type_filter.as_deref() == Some(item_type), *item_type).clicked() {
                            editor_state.items.type_filter = Some(item_type.to_string());
                        }
                    }
                });

            ui.separator();

            // Item list
            egui::ScrollArea::vertical().show(ui, |ui| {
                if editor_state.items.item_list.is_empty() {
                    ui.label("No items loaded");
                    ui.label("Click 'Refresh' to load from server");
                } else {
                    for item in &editor_state.items.item_list {
                        // Apply filters
                        if let Some(ref filter) = editor_state.items.type_filter {
                            if &item.item_type != filter {
                                continue;
                            }
                        }
                        if !editor_state.items.search_query.is_empty() {
                            if !item.name.to_lowercase().contains(&editor_state.items.search_query.to_lowercase()) {
                                continue;
                            }
                        }

                        let is_selected = editor_state.items.selected_item == Some(item.id);
                        let label = format!("[{}] {}", item.item_type, item.name);
                        if ui.selectable_label(is_selected, &label).clicked() {
                            editor_state.items.selected_item = Some(item.id);
                            // Load item for editing
                            editor_state.items.editing_item = Some(EditingItem {
                                id: item.id,
                                name: item.name.clone(),
                                item_type: item.item_type.clone(),
                                grants_ability: None,
                                attack_power: 0.0,
                                defense: 0.0,
                                max_health: 0.0,
                                max_mana: 0.0,
                                crit_chance: 0.0,
                            });
                        }
                    }
                }
            });
        });

    // Right panel - item properties
    egui::CentralPanel::default().show_inside(ui, |ui| {
        if let Some(ref mut editing_item) = editor_state.items.editing_item {
            ui.heading(format!("Item #{} - {}", editing_item.id, editing_item.name));

            ui.separator();

            // Basic properties
            ui.group(|ui| {
                ui.heading("Basic Info");

                ui.horizontal(|ui| {
                    ui.label("ID:");
                    ui.label(format!("{}", editing_item.id));
                });

                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editing_item.name);
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("item_type")
                        .selected_text(&editing_item.item_type)
                        .show_ui(ui, |ui| {
                            for item_type in ITEM_TYPES {
                                if ui.selectable_label(editing_item.item_type == *item_type, *item_type).clicked() {
                                    editing_item.item_type = item_type.to_string();
                                }
                            }
                        });
                });
            });

            ui.separator();

            // Stats
            ui.group(|ui| {
                ui.heading("Stats");

                ui.horizontal(|ui| {
                    ui.label("Attack Power:");
                    ui.add(egui::DragValue::new(&mut editing_item.attack_power).range(0.0..=1000.0).speed(0.5));
                });

                ui.horizontal(|ui| {
                    ui.label("Defense:");
                    ui.add(egui::DragValue::new(&mut editing_item.defense).range(0.0..=1000.0).speed(0.5));
                });

                ui.horizontal(|ui| {
                    ui.label("Max Health:");
                    ui.add(egui::DragValue::new(&mut editing_item.max_health).range(0.0..=10000.0).speed(1.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Max Mana:");
                    ui.add(egui::DragValue::new(&mut editing_item.max_mana).range(0.0..=10000.0).speed(1.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Crit Chance:");
                    ui.add(egui::DragValue::new(&mut editing_item.crit_chance).range(0.0..=1.0).speed(0.01).suffix("%"));
                });
            });

            ui.separator();

            // Behavior
            ui.group(|ui| {
                ui.heading("Behavior");

                ui.horizontal(|ui| {
                    ui.label("Grants Ability ID:");
                    let mut ability_id = editing_item.grants_ability.unwrap_or(0);
                    if ui.add(egui::DragValue::new(&mut ability_id).range(0..=9999)).changed() {
                        editing_item.grants_ability = if ability_id == 0 { None } else { Some(ability_id) };
                    }
                    ui.label("(0 = none)");
                });
            });

            ui.separator();

            // Actions
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    editor_state.action_save_item = true;
                }
                if ui.button("Delete").on_hover_text("Delete this item").clicked() {
                    editor_state.action_delete_item = true;
                }
            });
        } else if editor_state.items.selected_item.is_some() {
            ui.centered_and_justified(|ui| {
                ui.label("Loading item data...");
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select an item from the list or create a new one");
            });
        }
    });

    // Create new item dialog
    if editor_state.items.show_create_dialog {
        egui::Window::new("Create New Item")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editor_state.items.new_item_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("new_item_type")
                        .selected_text(&editor_state.items.new_item_type)
                        .show_ui(ui, |ui| {
                            for item_type in ITEM_TYPES {
                                if ui.selectable_label(editor_state.items.new_item_type == *item_type, *item_type).clicked() {
                                    editor_state.items.new_item_type = item_type.to_string();
                                }
                            }
                        });
                });

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        editor_state.action_create_item = true;
                    }
                    if ui.button("Cancel").clicked() {
                        editor_state.items.show_create_dialog = false;
                        editor_state.items.new_item_name.clear();
                    }
                });
            });
    }
}
