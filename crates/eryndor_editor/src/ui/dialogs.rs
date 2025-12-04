use bevy_egui::egui;
use std::path::PathBuf;

use crate::map::Level;
use crate::project::{Project, Tileset};
use crate::schema::{load_schema, save_schema, DataInstance};
use crate::EditorState;
use crate::AssetsBasePath;
use super::menu_bar::PendingAction;
use super::render_schema_editor;
use super::open_tileset_dialog;

pub fn render_dialogs(ctx: &egui::Context, editor_state: &mut EditorState, project: &mut Project, assets_base_path: &AssetsBasePath) {
    // Process pending actions
    process_pending_actions(editor_state, project);

    // New Project Dialog
    if editor_state.show_new_project_dialog {
        egui::Window::new("New Project")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Project Name:");
                    ui.text_edit_singleline(&mut editor_state.new_project_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Schema:");
                    if editor_state.new_project_schema_path.is_some() {
                        ui.label("(schema loaded)");
                    } else {
                        ui.label("(default schema)");
                    }
                    // TODO: Add file dialog support
                    // For now, schema loading via path must be done via config
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        editor_state.show_new_project_dialog = false;
                        editor_state.new_project_name.clear();
                        editor_state.new_project_schema_path = None;
                    }
                    if ui.button("Create").clicked() {
                        create_new_project(editor_state, project);
                        editor_state.show_new_project_dialog = false;
                    }
                });
            });
    }

    // New Level Dialog
    if editor_state.show_new_level_dialog {
        egui::Window::new("New Level")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editor_state.new_level_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Width:");
                    ui.add(egui::DragValue::new(&mut editor_state.new_level_width).range(1..=1000));
                });

                ui.horizontal(|ui| {
                    ui.label("Height:");
                    ui.add(egui::DragValue::new(&mut editor_state.new_level_height).range(1..=1000));
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        editor_state.show_new_level_dialog = false;
                    }
                    if ui.button("Create").clicked() {
                        let level = Level::new(
                            editor_state.new_level_name.clone(),
                            editor_state.new_level_width,
                            editor_state.new_level_height,
                        );
                        project.add_level(level);
                        editor_state.show_new_level_dialog = false;
                        editor_state.new_level_name = "New Level".to_string();
                    }
                });
            });
    }

    // Add Image to Tileset Dialog
    if editor_state.show_add_tileset_image_dialog {
        render_add_tileset_image_dialog(ctx, editor_state, project, assets_base_path);
    }

    // New Tileset Dialog
    if editor_state.show_new_tileset_dialog {
        egui::Window::new("Import Tileset")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut editor_state.new_tileset_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Image Path:");
                    ui.add(egui::TextEdit::singleline(&mut editor_state.new_tileset_path)
                        .desired_width(200.0));
                    if ui.button("Browse...").clicked() {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if let Some(path) = open_tileset_dialog() {
                                editor_state.new_tileset_path = path;
                            }
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            editor_state.error_message = Some("File dialogs not available in web build.".to_string());
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Tile Size:");
                    ui.add(egui::DragValue::new(&mut editor_state.new_tileset_tile_size)
                        .range(8..=256)
                        .suffix("px"));
                });

                // Preview info if path is set
                if !editor_state.new_tileset_path.is_empty() {
                    ui.separator();
                    ui.label(format!("Path: {}", editor_state.new_tileset_path));
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        editor_state.show_new_tileset_dialog = false;
                        reset_tileset_dialog(editor_state);
                    }

                    let can_create = !editor_state.new_tileset_name.is_empty()
                        && !editor_state.new_tileset_path.is_empty()
                        && editor_state.new_tileset_tile_size > 0;

                    ui.add_enabled_ui(can_create, |ui| {
                        if ui.button("Import").clicked() {
                            // Convert absolute path to relative path for Bevy's AssetServer
                            let absolute_path = PathBuf::from(&editor_state.new_tileset_path);
                            let relative_path = assets_base_path.to_relative(&absolute_path);

                            // Create tileset (columns/rows will be calculated when image loads)
                            let tileset = Tileset::new(
                                editor_state.new_tileset_name.clone(),
                                relative_path,
                                editor_state.new_tileset_tile_size,
                                8, // Default columns, will be recalculated
                                8, // Default rows, will be recalculated
                            );
                            let tileset_id = tileset.id;
                            project.tilesets.push(tileset);
                            project.mark_dirty();

                            // Select the new tileset
                            editor_state.selected_tileset = Some(tileset_id);

                            editor_state.show_new_tileset_dialog = false;
                            reset_tileset_dialog(editor_state);
                        }
                    });
                });
            });
    }

    // Create New Instance
    if let Some(type_name) = editor_state.create_new_instance.take() {
        let instance = DataInstance::new(type_name.clone());
        let id = instance.id;
        project.add_data_instance(instance);
        editor_state.selection = super::Selection::DataInstance(id);
    }

    // About Dialog
    if editor_state.show_about_dialog {
        egui::Window::new("About")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Eryndor Editor");
                ui.label("Version 0.1.0");
                ui.separator();
                ui.label("A generic, schema-driven 2D level editor for Bevy games.");
                ui.separator();
                if ui.button("Close").clicked() {
                    editor_state.show_about_dialog = false;
                }
            });
    }

    // Error Dialog
    if let Some(error) = &editor_state.error_message {
        let error_clone = error.clone();
        egui::Window::new("Error")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(&error_clone);
                ui.separator();
                if ui.button("OK").clicked() {
                    editor_state.error_message = None;
                }
            });
    }

    // Schema Editor
    if editor_state.show_schema_editor {
        render_schema_editor(
            ctx,
            &mut editor_state.schema_editor_state,
            project,
            &mut editor_state.show_schema_editor,
        );
    }

    // Note: Tileset & Terrain Editor is rendered in mod.rs with tileset_cache access
}

fn process_pending_actions(editor_state: &mut EditorState, project: &mut Project) {
    if let Some(action) = editor_state.pending_action.take() {
        match action {
            PendingAction::OpenProject => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(path) = open_project_dialog() {
                        match Project::load(&path) {
                            Ok(loaded) => {
                                *project = loaded;
                            }
                            Err(e) => {
                                editor_state.error_message = Some(format!("Failed to load project: {}", e));
                            }
                        }
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    editor_state.error_message = Some("File dialogs not available in web build.".to_string());
                }
            }
            PendingAction::OpenSchema => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(path) = open_schema_dialog() {
                        match load_schema(&path) {
                            Ok(schema) => {
                                project.schema = schema;
                            }
                            Err(e) => {
                                editor_state.error_message = Some(format!("Failed to load schema: {}", e));
                            }
                        }
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    editor_state.error_message = Some("File dialogs not available in web build.".to_string());
                }
            }
            PendingAction::SaveSchema => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(path) = save_schema_dialog() {
                        if let Err(e) = save_schema(&project.schema, &path) {
                            editor_state.error_message = Some(format!("Failed to save schema: {}", e));
                        }
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    editor_state.error_message = Some("File dialogs not available in web build.".to_string());
                }
            }
            PendingAction::Save => {
                if project.path.is_some() {
                    if let Err(e) = project.save_current() {
                        editor_state.error_message = Some(format!("Failed to save: {}", e));
                    }
                } else {
                    // No path set, do Save As
                    editor_state.pending_action = Some(PendingAction::SaveAs);
                }
            }
            PendingAction::SaveAs => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(path) = save_project_dialog() {
                        if let Err(e) = project.save(&path) {
                            editor_state.error_message = Some(format!("Failed to save project: {}", e));
                        }
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    editor_state.error_message = Some("File dialogs not available in web build.".to_string());
                }
            }
            PendingAction::Export => {
                editor_state.error_message = Some("Export not yet implemented.".to_string());
            }
            // These are handled by the keyboard shortcuts system or have immediate effect
            PendingAction::Undo | PendingAction::Redo |
            PendingAction::Cut | PendingAction::Copy |
            PendingAction::Paste | PendingAction::SelectAll => {
                // Handled elsewhere
            }
        }
    }
}

/// Open a file dialog for opening a project (native only)
#[cfg(not(target_arch = "wasm32"))]
fn open_project_dialog() -> Option<std::path::PathBuf> {
    use rfd::FileDialog;

    FileDialog::new()
        .add_filter("Project Files", &["json", "eryndor"])
        .add_filter("All Files", &["*"])
        .set_title("Open Project")
        .pick_file()
}

/// Open a file dialog for opening a schema (native only)
#[cfg(not(target_arch = "wasm32"))]
fn open_schema_dialog() -> Option<std::path::PathBuf> {
    use rfd::FileDialog;

    FileDialog::new()
        .add_filter("Schema Files", &["json", "schema"])
        .add_filter("All Files", &["*"])
        .set_title("Open Schema")
        .pick_file()
}

/// Open a file dialog for saving a schema (native only)
#[cfg(not(target_arch = "wasm32"))]
fn save_schema_dialog() -> Option<std::path::PathBuf> {
    use rfd::FileDialog;

    FileDialog::new()
        .add_filter("Schema Files", &["json"])
        .add_filter("All Files", &["*"])
        .set_title("Save Schema")
        .set_file_name("schema.json")
        .save_file()
}

/// Open a file dialog for saving a project (native only)
#[cfg(not(target_arch = "wasm32"))]
fn save_project_dialog() -> Option<std::path::PathBuf> {
    use rfd::FileDialog;

    FileDialog::new()
        .add_filter("Project Files", &["json"])
        .add_filter("All Files", &["*"])
        .set_title("Save Project")
        .set_file_name("project.json")
        .save_file()
}

fn create_new_project(editor_state: &mut EditorState, project: &mut Project) {
    let schema = if let Some(path) = &editor_state.new_project_schema_path {
        match load_schema(path) {
            Ok(s) => s,
            Err(e) => {
                editor_state.error_message = Some(format!("Failed to load schema: {}", e));
                return;
            }
        }
    } else {
        crate::schema::default_schema()
    };

    *project = Project::new(schema);
    project.schema.project.name = editor_state.new_project_name.clone();

    editor_state.new_project_name.clear();
    editor_state.new_project_schema_path = None;
}

fn reset_tileset_dialog(editor_state: &mut EditorState) {
    editor_state.new_tileset_name = "New Tileset".to_string();
    editor_state.new_tileset_path.clear();
    editor_state.new_tileset_tile_size = 32;
}

/// Dialog for adding an image to an existing tileset
fn render_add_tileset_image_dialog(
    ctx: &egui::Context,
    editor_state: &mut EditorState,
    project: &mut Project,
    assets_base_path: &AssetsBasePath,
) {
    let tileset_name = editor_state.selected_tileset
        .and_then(|id| project.tilesets.iter().find(|t| t.id == id))
        .map(|t| t.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    egui::Window::new(format!("Add Image to {}", tileset_name))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Image Name:");
                ui.text_edit_singleline(&mut editor_state.add_image_name);
            });

            ui.horizontal(|ui| {
                ui.label("Image Path:");
                ui.add(egui::TextEdit::singleline(&mut editor_state.add_image_path)
                    .desired_width(200.0));
                if ui.button("Browse...").clicked() {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if let Some(path) = open_tileset_dialog() {
                            editor_state.add_image_path = path;
                        }
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        editor_state.error_message = Some("File dialogs not available in web build.".to_string());
                    }
                }
            });

            // Preview info if path is set
            if !editor_state.add_image_path.is_empty() {
                ui.separator();
                ui.label(format!("Path: {}", editor_state.add_image_path));
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    editor_state.show_add_tileset_image_dialog = false;
                    reset_add_image_dialog(editor_state);
                }

                let can_add = !editor_state.add_image_name.is_empty()
                    && !editor_state.add_image_path.is_empty()
                    && editor_state.selected_tileset.is_some();

                ui.add_enabled_ui(can_add, |ui| {
                    if ui.button("Add Image").clicked() {
                        if let Some(tileset_id) = editor_state.selected_tileset {
                            if let Some(tileset) = project.tilesets.iter_mut().find(|t| t.id == tileset_id) {
                                // Convert absolute path to relative path for Bevy's AssetServer
                                let absolute_path = PathBuf::from(&editor_state.add_image_path);
                                let relative_path = assets_base_path.to_relative(&absolute_path);

                                // Add the image with default columns/rows (will be recalculated when loaded)
                                tileset.add_image(
                                    editor_state.add_image_name.clone(),
                                    relative_path,
                                    8, // Default columns
                                    8, // Default rows
                                );
                                project.mark_dirty();
                            }
                        }

                        editor_state.show_add_tileset_image_dialog = false;
                        reset_add_image_dialog(editor_state);
                    }
                });
            });
        });
}

fn reset_add_image_dialog(editor_state: &mut EditorState) {
    editor_state.add_image_name.clear();
    editor_state.add_image_path.clear();
}
