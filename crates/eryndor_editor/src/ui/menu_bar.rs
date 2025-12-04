use bevy_egui::egui;

use crate::project::Project;
use crate::EditorState;
use super::UiState;

pub fn render_menu_bar(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    editor_state: &mut EditorState,
    project: &mut Project,
) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            // File menu
            ui.menu_button("File", |ui| {
                if ui.button("New Project...").clicked() {
                    editor_state.show_new_project_dialog = true;
                    ui.close();
                }
                if ui.button("Open Project...").clicked() {
                    editor_state.pending_action = Some(PendingAction::OpenProject);
                    ui.close();
                }
                ui.separator();
                if ui.button("Open Schema...").clicked() {
                    editor_state.pending_action = Some(PendingAction::OpenSchema);
                    ui.close();
                }
                if ui.button("Save Schema...").clicked() {
                    editor_state.pending_action = Some(PendingAction::SaveSchema);
                    ui.close();
                }
                ui.separator();
                if ui.button("Save Project").clicked() {
                    editor_state.pending_action = Some(PendingAction::Save);
                    ui.close();
                }
                if ui.button("Save Project As...").clicked() {
                    editor_state.pending_action = Some(PendingAction::SaveAs);
                    ui.close();
                }
                ui.separator();
                if ui.button("Export...").clicked() {
                    editor_state.pending_action = Some(PendingAction::Export);
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
                if ui.button("Cut").clicked() {
                    // TODO: Implement cut
                    ui.close();
                }
                if ui.button("Copy").clicked() {
                    // TODO: Implement copy
                    ui.close();
                }
                if ui.button("Paste").clicked() {
                    // TODO: Implement paste
                    ui.close();
                }
            });

            // View menu
            ui.menu_button("View", |ui| {
                if ui.checkbox(&mut ui_state.show_tree_view, "Tree View").clicked() {
                    ui.close();
                }
                if ui.checkbox(&mut ui_state.show_inspector, "Inspector").clicked() {
                    ui.close();
                }
                ui.separator();
                if ui.button("Zoom In").clicked() {
                    editor_state.zoom = (editor_state.zoom * 1.25).min(4.0);
                    ui.close();
                }
                if ui.button("Zoom Out").clicked() {
                    editor_state.zoom = (editor_state.zoom / 1.25).max(0.25);
                    ui.close();
                }
                if ui.button("Reset Zoom").clicked() {
                    editor_state.zoom = 1.0;
                    ui.close();
                }
            });

            // Create menu
            ui.menu_button("Create", |ui| {
                // Data types
                ui.menu_button("New Data...", |ui| {
                    for type_name in project.schema.data_type_names() {
                        if ui.button(type_name).clicked() {
                            editor_state.create_new_instance = Some(type_name.to_string());
                            ui.close();
                        }
                    }
                    if project.schema.data_types.is_empty() {
                        ui.label("(no data types in schema)");
                    }
                });

                ui.separator();

                if ui.button("New Level...").clicked() {
                    editor_state.show_new_level_dialog = true;
                    ui.close();
                }
            });

            // Tools menu
            ui.menu_button("Tools", |ui| {
                if ui.button("Schema Editor...").clicked() {
                    editor_state.show_schema_editor = true;
                    ui.close();
                }
                if ui.button("Tileset & Terrain Editor...").clicked() {
                    editor_state.show_tileset_editor = true;
                    ui.close();
                }
                ui.separator();
                if ui.button("Run Automapping").clicked() {
                    // TODO: Implement automapping
                    ui.close();
                }
                ui.separator();
                if ui.button("Validate Project").clicked() {
                    // TODO: Implement validation
                    ui.close();
                }
            });

            // Help menu
            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    editor_state.show_about_dialog = true;
                    ui.close();
                }
            });

            // Show project dirty state
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if project.is_dirty() {
                    ui.label(egui::RichText::new("*").color(egui::Color32::YELLOW));
                }
                ui.label(&project.name());
            });
        });
    });
}

#[derive(Debug, Clone)]
pub enum PendingAction {
    OpenProject,
    OpenSchema,
    SaveSchema,
    Save,
    SaveAs,
    Export,
}
