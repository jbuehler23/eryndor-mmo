use bevy_egui::egui;

use crate::commands::{CommandHistory, TileClipboard};
use crate::project::Project;
use crate::EditorState;
use super::UiState;

pub fn render_menu_bar(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    editor_state: &mut EditorState,
    project: &mut Project,
    history: Option<&CommandHistory>,
    clipboard: Option<&TileClipboard>,
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
                // Undo
                let can_undo = history.map(|h| h.can_undo()).unwrap_or(false);
                let undo_text = history
                    .and_then(|h| h.undo_description())
                    .map(|desc| format!("Undo {}", desc))
                    .unwrap_or_else(|| "Undo".to_string());

                if ui.add_enabled(can_undo, egui::Button::new(&undo_text)).clicked() {
                    editor_state.pending_action = Some(PendingAction::Undo);
                    ui.close();
                }

                // Redo
                let can_redo = history.map(|h| h.can_redo()).unwrap_or(false);
                let redo_text = history
                    .and_then(|h| h.redo_description())
                    .map(|desc| format!("Redo {}", desc))
                    .unwrap_or_else(|| "Redo".to_string());

                if ui.add_enabled(can_redo, egui::Button::new(&redo_text)).clicked() {
                    editor_state.pending_action = Some(PendingAction::Redo);
                    ui.close();
                }

                ui.separator();

                // Cut
                let has_selection = !editor_state.tile_selection.is_empty();
                if ui.add_enabled(has_selection, egui::Button::new("Cut")).clicked() {
                    editor_state.pending_action = Some(PendingAction::Cut);
                    ui.close();
                }

                // Copy
                if ui.add_enabled(has_selection, egui::Button::new("Copy")).clicked() {
                    editor_state.pending_action = Some(PendingAction::Copy);
                    ui.close();
                }

                // Paste
                let can_paste = clipboard.map(|c| c.has_content()).unwrap_or(false);
                if ui.add_enabled(can_paste, egui::Button::new("Paste")).clicked() {
                    editor_state.pending_action = Some(PendingAction::Paste);
                    ui.close();
                }

                ui.separator();

                // Delete
                if ui.add_enabled(has_selection, egui::Button::new("Delete")).clicked() {
                    editor_state.pending_delete_selection = true;
                    ui.close();
                }

                // Select All
                if ui.button("Select All").clicked() {
                    editor_state.pending_action = Some(PendingAction::SelectAll);
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
                if ui.checkbox(&mut editor_state.show_grid, "Show Grid").clicked() {
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
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
}
