use crate::file_dialog_helper::FileDialogState;
use crate::project_generator::{generate_project, ProjectTemplate};
use crate::project_manager::{ProjectSelection, ProjectSelectionState};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::path::PathBuf;
use std::process::Command;

/// Resource to track the project wizard state
#[derive(Resource)]
pub struct ProjectWizard {
    pub project_name: String,
    pub project_path: Option<PathBuf>,
    pub selected_template: ProjectTemplate,
    pub show_wizard: bool,
    pub file_dialog_state: FileDialogState,
}

impl Default for ProjectWizard {
    fn default() -> Self {
        Self {
            project_name: String::new(),
            project_path: None,
            selected_template: ProjectTemplate::default(),
            show_wizard: false,
            file_dialog_state: FileDialogState::new(),
        }
    }
}

impl Default for ProjectTemplate {
    fn default() -> Self {
        ProjectTemplate::Tilemap2D
    }
}

/// UI for the project creation wizard
pub fn project_wizard_ui(
    mut contexts: EguiContexts,
    mut wizard: ResMut<ProjectWizard>,
    mut selection: ResMut<ProjectSelection>,
) {
    if !wizard.show_wizard {
        return;
    }

    egui::Window::new("Create New Project")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(contexts.ctx_mut().unwrap(), |ui| {
            ui.heading("New Bevy Project");
            ui.add_space(10.0);

            // Project Name
            ui.label("Project Name:");
            ui.text_edit_singleline(&mut wizard.project_name);
            ui.add_space(10.0);

            // Project Location
            ui.label("Project Location:");

            // Show current selection or manual entry UI
            if wizard.file_dialog_state.show_manual_entry {
                // Manual path entry mode
                if let Some(path) = wizard.file_dialog_state.render_manual_entry_ui(ui, "Path:") {
                    wizard.project_path = Some(path);
                    wizard.file_dialog_state.show_manual_entry = false;
                }
            } else {
                // Normal dialog mode
                ui.horizontal(|ui| {
                    let location_text = wizard
                        .project_path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "Not selected".to_string());

                    ui.label(location_text);

                    if ui.button("Browse...").clicked() {
                        if let Some(folder) = wizard.file_dialog_state.try_pick_folder("Select Project Location") {
                            wizard.project_path = Some(folder);
                        }
                    }

                    // Add manual entry button for convenience
                    if ui.small_button("✏ Manual").on_hover_text("Enter path manually").clicked() {
                        wizard.file_dialog_state.show_manual_entry = true;
                        if let Some(ref path) = wizard.project_path {
                            wizard.file_dialog_state.manual_path = path.display().to_string();
                        }
                    }
                });
            }
            ui.add_space(10.0);

            // Template Selection
            ui.label("Project Template:");

            ui.radio_value(
                &mut wizard.selected_template,
                ProjectTemplate::Empty,
                ProjectTemplate::Empty.name(),
            );
            ui.label(format!("  └─ {}", ProjectTemplate::Empty.description()));
            ui.add_space(5.0);

            ui.radio_value(
                &mut wizard.selected_template,
                ProjectTemplate::Tilemap2D,
                ProjectTemplate::Tilemap2D.name(),
            );
            ui.label(format!("  └─ {}", ProjectTemplate::Tilemap2D.description()));
            ui.add_space(5.0);

            // BevyNew2D template with CLI check
            let bevy_cli_available = check_bevy_cli_installed();

            ui.add_enabled_ui(bevy_cli_available, |ui| {
                ui.radio_value(
                    &mut wizard.selected_template,
                    ProjectTemplate::BevyNew2D,
                    ProjectTemplate::BevyNew2D.name(),
                );
            });

            if !bevy_cli_available {
                ui.label(format!(
                    "  └─ {} (requires bevy CLI)",
                    ProjectTemplate::BevyNew2D.description()
                ))
                .on_hover_text("Install with: cargo install bevy_cli");
                ui.horizontal(|ui| {
                    ui.label("    ");
                    if ui.small_button("📋 Copy Install Command").clicked() {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            use arboard::Clipboard;
                            if let Ok(mut clipboard) = Clipboard::new() {
                                let _ = clipboard.set_text("cargo install bevy_cli");
                                info!("Copied 'cargo install bevy_cli' to clipboard");
                            }
                        }
                    }
                });
            } else {
                ui.label(format!("  └─ {}", ProjectTemplate::BevyNew2D.description()));
            }

            ui.add_space(15.0);

            // Action Buttons
            ui.horizontal(|ui| {
                let can_create = !wizard.project_name.is_empty() && wizard.project_path.is_some();

                // Disable if BevyNew2D selected but CLI not available
                let can_create = can_create
                    && (wizard.selected_template != ProjectTemplate::BevyNew2D
                        || bevy_cli_available);

                if ui
                    .add_enabled(can_create, egui::Button::new("Create Project"))
                    .clicked()
                {
                    // Create the project
                    let project_path = wizard
                        .project_path
                        .clone()
                        .unwrap()
                        .join(&wizard.project_name);

                    match generate_project(
                        &project_path,
                        &wizard.project_name,
                        wizard.selected_template.clone(),
                    ) {
                        Ok(_) => {
                            info!("Project created successfully at: {:?}", project_path);

                            // Trigger project opening
                            selection.state = ProjectSelectionState::Opening {
                                path: project_path.to_string_lossy().to_string(),
                            };

                            // Reset wizard
                            wizard.show_wizard = false;
                            wizard.project_name.clear();
                            wizard.project_path = None;
                        }
                        Err(e) => {
                            error!("Failed to create project: {}", e);
                            selection.state = ProjectSelectionState::Error(format!(
                                "Failed to create project: {}",
                                e
                            ));
                            wizard.show_wizard = false;
                        }
                    }
                }

                if ui.button("Cancel").clicked() {
                    wizard.show_wizard = false;
                    wizard.project_name.clear();
                    wizard.project_path = None;
                }
            });
        });
}

/// Check if bevy CLI is installed on the system
fn check_bevy_cli_installed() -> bool {
    Command::new("bevy")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
