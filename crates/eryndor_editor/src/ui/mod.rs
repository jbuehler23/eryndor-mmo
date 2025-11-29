//! UI components for the editor

use bevy_egui::egui;
use crate::editor_state::{EditorState, EditorTab};

/// Render the main menu bar at the top of the editor
pub fn render_main_menu(ctx: &egui::Context, editor_state: &mut EditorState) {
    egui::TopBottomPanel::top("main_menu").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            // File menu
            ui.menu_button("File", |ui| {
                if ui.button("New Zone...").clicked() {
                    editor_state.active_tab = EditorTab::World;
                    // TODO: Open new zone dialog
                    ui.close();
                }
                if ui.button("Save All").clicked() {
                    editor_state.status_message = "Saving...".to_string();
                    // TODO: Save all changes
                    ui.close();
                }
                ui.separator();
                if ui.button("Refresh from Server").clicked() {
                    editor_state.status_message = "Refreshing...".to_string();
                    // TODO: Reload all data
                    ui.close();
                }
            });

            // Edit menu
            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    // TODO: Implement undo
                    ui.close();
                }
                if ui.button("Redo").clicked() {
                    // TODO: Implement redo
                    ui.close();
                }
                ui.separator();
                if ui.button("Preferences...").clicked() {
                    // TODO: Open preferences dialog
                    ui.close();
                }
            });

            // View menu
            ui.menu_button("View", |ui| {
                if ui.checkbox(&mut editor_state.world.show_grid, "Show Grid").clicked() {
                    ui.close();
                }
                if ui.checkbox(&mut editor_state.world.show_collisions, "Show Collisions").clicked() {
                    ui.close();
                }
                if ui.checkbox(&mut editor_state.world.show_spawn_regions, "Show Spawn Regions").clicked() {
                    ui.close();
                }
                ui.separator();
                if ui.button("Reset Zoom").clicked() {
                    editor_state.world.zoom = 1.0;
                    ui.close();
                }
            });

            // Create menu - quick access to create new content
            ui.menu_button("Create", |ui| {
                if ui.button("New Item...").clicked() {
                    editor_state.active_tab = EditorTab::Items;
                    editor_state.items.show_create_dialog = true;
                    ui.close();
                }
                if ui.button("New Enemy...").clicked() {
                    editor_state.active_tab = EditorTab::Enemies;
                    editor_state.enemies.show_create_dialog = true;
                    ui.close();
                }
                if ui.button("New NPC...").clicked() {
                    editor_state.active_tab = EditorTab::Npcs;
                    editor_state.npcs.show_create_dialog = true;
                    ui.close();
                }
                if ui.button("New Quest...").clicked() {
                    editor_state.active_tab = EditorTab::Quests;
                    editor_state.quests.show_create_dialog = true;
                    ui.close();
                }
                if ui.button("New Ability...").clicked() {
                    editor_state.active_tab = EditorTab::Abilities;
                    editor_state.abilities.show_create_dialog = true;
                    ui.close();
                }
                if ui.button("New Loot Table...").clicked() {
                    editor_state.active_tab = EditorTab::Loot;
                    editor_state.loot.show_create_dialog = true;
                    ui.close();
                }
            });

            // Help menu
            ui.menu_button("Help", |ui| {
                if ui.button("Documentation").clicked() {
                    // TODO: Open docs
                    ui.close();
                }
                ui.separator();
                if ui.button("About Eryndor Editor").clicked() {
                    // TODO: Show about dialog
                    ui.close();
                }
            });

            // Right-aligned items
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Save indicator
                if editor_state.has_unsaved_changes {
                    ui.label(egui::RichText::new("Unsaved Changes").color(egui::Color32::YELLOW));
                }
            });
        });
    });
}

/// Render the tab bar for switching between editor modules
pub fn render_tab_bar(ctx: &egui::Context, editor_state: &mut EditorState) {
    egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;

            for tab in EditorTab::all() {
                let is_selected = editor_state.active_tab == *tab;

                let button = egui::Button::new(tab.label())
                    .fill(if is_selected {
                        egui::Color32::from_rgb(60, 60, 80)
                    } else {
                        egui::Color32::from_rgb(40, 40, 50)
                    })
                    .min_size(egui::vec2(80.0, 28.0));

                if ui.add(button).clicked() {
                    editor_state.active_tab = *tab;
                }
            }
        });
    });
}

/// Render the status bar at the bottom of the editor
pub fn render_status_bar(ctx: &egui::Context, editor_state: &EditorState) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Status message
            ui.label(&editor_state.status_message);

            ui.separator();

            // Current tab info
            ui.label(format!("Tab: {}", editor_state.active_tab.label()));

            // World editor specific status
            if editor_state.active_tab == EditorTab::World {
                ui.separator();
                ui.label(format!("Zoom: {:.0}%", editor_state.world.zoom * 100.0));

                if let Some(zone) = &editor_state.world.current_zone {
                    ui.separator();
                    ui.label(format!("Zone: {}", zone));
                }
            }

            // Right-aligned items
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("Eryndor Editor v0.1.0");
            });
        });
    });
}

/// Render the error popup window if there is an error to display
pub fn render_error_popup(ctx: &egui::Context, editor_state: &mut EditorState) {
    let mut should_close = false;

    if let Some(error) = &editor_state.error_popup {
        egui::Window::new("Error")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(300.0);

                // Error icon and message
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::RED, "\u{26A0}"); // Warning symbol
                    ui.label(error);
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);

                // OK button to dismiss
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("OK").clicked() {
                        should_close = true;
                    }
                });
            });
    }

    if should_close {
        editor_state.error_popup = None;
    }
}
