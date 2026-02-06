use bevy::prelude::*;
use bevy_editor_formats::{ProjectConfig, ProjectMetadata};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(feature = "cli")]
use crate::bevy_cli_runner::BevyCLIRunner;
#[cfg(feature = "ui")]
use crate::file_dialog_helper::FileDialogState;
#[cfg(feature = "ui")]
use crate::project_wizard::ProjectWizard;
#[cfg(feature = "workspace")]
use crate::workspace::EditorWorkspace;
#[cfg(feature = "ui")]
use bevy_egui::{egui, EguiContexts};

/// Resource holding the current project information
#[derive(Resource)]
pub struct CurrentProject {
    pub metadata: ProjectMetadata,
}

impl CurrentProject {
    /// Create a new project at the given path
    pub fn create_new<P: Into<PathBuf>>(
        path: P,
        name: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let path_buf = path.into();

        // Create the project directory if it doesn't exist
        std::fs::create_dir_all(&path_buf)?;

        // Create project config
        let mut config = ProjectConfig::new(name.clone());
        config.client_config.window_title = name.clone();
        config.default_scene = Some("main.bscene".to_string());

        // Save config as .bvy file
        let config_path = path_buf.join("project.bvy");
        config.save_to_file(&config_path)?;

        // Load metadata (this will create the directory structure)
        let metadata = ProjectMetadata::from_project_path(&path_buf)?;

        // Auto-create default scene (main.bscene)
        let default_scene_path = metadata.levels_path.join("main.bscene");
        if !default_scene_path.exists() {
            let default_level_data = bevy_editor_formats::LevelData::new(
                format!("{} - Main Scene", name),
                2000.0, // default width
                1000.0, // default height
            );
            let scene = bevy_editor_formats::BevyScene::new(default_level_data);
            scene.save_to_file(&default_scene_path)?;
            info!("Created default scene: {:?}", default_scene_path);
        }

        info!("Created new project at: {:?}", path_buf);

        sanitize_project_cargo_config(&path_buf);

        Ok(Self { metadata })
    }

    /// Open an existing project
    pub fn open_existing<P: Into<PathBuf>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path_buf = path.into();

        // Check if project.bvy exists
        let config_path = path_buf.join("project.bvy");
        if !config_path.exists() {
            return Err(format!("No project.bvy found at {:?}", path_buf).into());
        }

        // Load metadata
        let metadata = ProjectMetadata::from_project_path(&path_buf)?;

        info!("Opened project: {} at {:?}", metadata.config.name, path_buf);

        sanitize_project_cargo_config(&path_buf);

        Ok(Self { metadata })
    }

    /// Get the assets directory path
    pub fn assets_path(&self) -> &PathBuf {
        &self.metadata.assets_path
    }

    /// Get the levels directory path
    pub fn levels_path(&self) -> &PathBuf {
        &self.metadata.levels_path
    }

    /// Get the project root path
    pub fn root_path(&self) -> &PathBuf {
        &self.metadata.root_path
    }

    /// Get the project name
    pub fn name(&self) -> &str {
        &self.metadata.config.name
    }

    /// Update and save the project configuration
    pub fn update_config<F>(&mut self, update_fn: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut ProjectConfig),
    {
        update_fn(&mut self.metadata.config);
        self.metadata.save_config()
    }
}

fn sanitize_project_cargo_config(project_root: &Path) {
    let config_path = project_root.join(".cargo").join("config.toml");
    let Ok(original) = fs::read_to_string(&config_path) else {
        return;
    };

    let mut changed = false;
    let mut sanitized = Vec::new();

    for line in original.lines() {
        if line.contains("-Zshare-generics") && !line.trim_start().starts_with('#') {
            changed = true;
            let trimmed = line.trim_start();
            let indent_len = line.len() - trimmed.len();
            let indent = &line[..indent_len];
            sanitized.push(format!("{indent}# {trimmed}"));
        } else {
            sanitized.push(line.to_string());
        }
    }

    if changed {
        let mut output = sanitized.join("\n");
        output.push('\n');
        match fs::write(&config_path, output) {
            Ok(_) => info!(
                "Updated {} to remove nightly-only rustflags",
                config_path.display()
            ),
            Err(err) => warn!("Failed to update {}: {}", config_path.display(), err),
        }
    }
}

/// Build progress information
#[derive(Clone)]
pub struct BuildProgress {
    pub start_time: std::time::Instant,
    pub output_lines: Vec<String>,
    pub current_stage: String,
}

impl BuildProgress {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            output_lines: Vec::new(),
            current_stage: "Starting build...".to_string(),
        }
    }

    pub fn elapsed_secs(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }
}

/// System set used by project management systems so downstream apps can order them.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectManagerSet;

/// Project selection state for UI
#[derive(Default)]
pub enum ProjectSelectionState {
    #[default]
    NeedSelection,
    Creating {
        path: String,
        name: String,
    },
    Opening {
        path: String,
    },
    GeneratingTemplate,          // Running bevy new or custom template generation
    InitialBuild(BuildProgress), // First build to warm cache
    Ready,
    Error(String),
}

/// Resource to track project selection UI state
#[derive(Resource)]
pub struct ProjectSelection {
    pub state: ProjectSelectionState,
    #[cfg(feature = "ui")]
    pub file_dialog_state: FileDialogState,
}

impl Default for ProjectSelection {
    fn default() -> Self {
        Self {
            state: ProjectSelectionState::default(),
            #[cfg(feature = "ui")]
            file_dialog_state: FileDialogState::new(),
        }
    }
}

/// System to handle project selection before editor is fully initialized
pub fn handle_project_selection(
    mut commands: Commands,
    mut selection: ResMut<ProjectSelection>,
    #[cfg(feature = "workspace")] mut workspace: Option<ResMut<EditorWorkspace>>,
    #[cfg(feature = "cli")] mut cli_runner: Option<ResMut<BevyCLIRunner>>,
) {
    match &selection.state {
        ProjectSelectionState::Creating { path, name } => {
            match CurrentProject::create_new(path.clone(), name.clone()) {
                Ok(project) => {
                    info!("Project created successfully");

                    // Add to workspace recent projects
                    #[cfg(feature = "workspace")]
                    if let Some(ref mut workspace) = workspace {
                        workspace.add_recent_project(path.clone());
                    }

                    // Update CLI runner with new project path
                    #[cfg(feature = "cli")]
                    if let Some(ref mut cli_runner) = cli_runner {
                        cli_runner.set_project_path(std::path::PathBuf::from(path));
                    }

                    commands.insert_resource(project);
                    selection.state = ProjectSelectionState::Ready;
                }
                Err(e) => {
                    error!("Failed to create project: {}", e);
                    selection.state =
                        ProjectSelectionState::Error(format!("Failed to create project: {}", e));
                }
            }
        }
        ProjectSelectionState::Opening { path } => {
            match CurrentProject::open_existing(path.clone()) {
                Ok(project) => {
                    info!("Project opened successfully");

                    // Add to workspace recent projects
                    #[cfg(feature = "workspace")]
                    if let Some(ref mut workspace) = workspace {
                        workspace.add_recent_project(path.clone());
                    }

                    // Update CLI runner with new project path
                    #[cfg(feature = "cli")]
                    if let Some(ref mut cli_runner) = cli_runner {
                        cli_runner.set_project_path(std::path::PathBuf::from(path));
                    }

                    commands.insert_resource(project);
                    selection.state = ProjectSelectionState::Ready;
                }
                Err(e) => {
                    error!("Failed to open project: {}", e);
                    selection.state =
                        ProjectSelectionState::Error(format!("Failed to open project: {}", e));
                }
            }
        }
        _ => {}
    }
}

/// UI for project selection dialog
#[cfg(feature = "ui")]
pub fn project_selection_ui(
    mut contexts: EguiContexts,
    mut selection: ResMut<ProjectSelection>,
    mut wizard: ResMut<ProjectWizard>,
) {
    // Clone the error message if needed to avoid borrow issues
    let error_msg = if let ProjectSelectionState::Error(msg) = &selection.state {
        Some(msg.clone())
    } else {
        None
    };

    match selection.state {
        ProjectSelectionState::NeedSelection => {
            egui::Window::new("Welcome to Bevy Editor")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(contexts.ctx_mut().unwrap(), |ui| {
                    ui.heading("Select a Project");
                    ui.add_space(10.0);

                    ui.label("Create a new Bevy project or open an existing one");
                    ui.add_space(20.0);

                    if ui.button("📁 Create New Project").clicked() {
                        // Show project wizard
                        wizard.show_wizard = true;
                    }

                    ui.add_space(10.0);

                    if ui.button("📂 Open Existing Project").clicked() {
                        // Open file dialog to select project directory
                        if let Some(folder) = selection.file_dialog_state.try_pick_folder("Open Project") {
                            selection.state = ProjectSelectionState::Opening {
                                path: folder.to_string_lossy().to_string(),
                            };
                        }
                    }

                    // Show manual entry option if dialog failed
                    if selection.file_dialog_state.show_manual_entry {
                        ui.add_space(10.0);
                        if let Some(folder) = selection.file_dialog_state.render_manual_entry_ui(ui, "Project Path:") {
                            selection.state = ProjectSelectionState::Opening {
                                path: folder.to_string_lossy().to_string(),
                            };
                            selection.file_dialog_state.show_manual_entry = false;
                        }
                    }
                });
        }
        ProjectSelectionState::Error(_) => {
            if let Some(msg) = error_msg {
                egui::Window::new("Error")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(contexts.ctx_mut().unwrap(), |ui| {
                        ui.colored_label(egui::Color32::RED, "Error:");
                        ui.label(&msg);
                        ui.add_space(10.0);

                        if ui.button("Try Again").clicked() {
                            selection.state = ProjectSelectionState::NeedSelection;
                        }
                    });
            }
        }
        _ => {}
    }
}
