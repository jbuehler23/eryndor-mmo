//! Thin application shell that wires the modular editor backend together with a chosen frontend.
//!
//! The [`EditorAppPlugin`] installs the core editor crates, registers shared
//! events, and delegates UI responsibilities to a supplied frontend that
//! implements [`bevy_editor_frontend_api::EditorFrontend`].

pub mod scene_loader;
pub mod systems;

use bevy::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy_editor_assets::AssetBrowserPlugin;
use bevy_editor_commands::EditorHistory;
use bevy_editor_core::{handle_gizmo_mode_shortcuts, EditorCameraPlugin, EditorCorePlugin};
use bevy_editor_foundation::{EditorState, EditorStatePlugin};
use bevy_editor_frontend_api::{EditorAction, EditorEvent, EditorFrontend, ProjectCommand};
use bevy_editor_play_mode::PlayModePlugin;
use bevy_editor_project::{BevyCLIRunner, CLICommand, ProjectManagerPlugin, ProjectManagerSet};
use bevy_editor_scene::{mark_loaded_scene_entities, SceneAutoLoader, SceneEditorPlugin};
use bevy_editor_tilemap::TilemapEditorPlugin;

use scene_loader::{auto_load_scene_system, reset_auto_loader_on_project_change, LoadSceneEvent};
use systems::{
    cache_runtime_scene_on_scene_switch, handle_save_load, restore_tilemap_from_level,
    sync_tilemap_on_scene_switch, PendingTilemapRestore, PreviousSceneIndex,
};

/// Tracks the currently running project command so we can emit lifecycle events.
#[derive(Resource, Default)]
struct ActiveProjectCommand {
    current: Option<ProjectCommand>,
    is_running: bool,
}

/// Top-level plugin wiring backend crates and the selected frontend together.
pub struct EditorAppPlugin<F: EditorFrontend + Clone> {
    frontend: F,
}

impl<F: EditorFrontend + Clone> EditorAppPlugin<F> {
    pub fn new(frontend: F) -> Self {
        Self { frontend }
    }
}

impl<F: EditorFrontend + Clone> Plugin for EditorAppPlugin<F> {
    fn build(&self, app: &mut App) {
        app.add_message::<EditorAction>()
            .add_message::<EditorEvent>()
            .add_message::<LoadSceneEvent>()
            .add_message::<bevy_editor_scene::SceneTabChanged>()
            .init_resource::<ActiveProjectCommand>()
            .init_resource::<EditorHistory>()
            .init_resource::<PendingTilemapRestore>()
            .init_resource::<PreviousSceneIndex>()
            .init_resource::<SceneAutoLoader>()
            .init_resource::<bevy_editor_scene::OpenScenes>()
            .add_plugins((
                EditorStatePlugin,
                EditorCameraPlugin,
                EditorCorePlugin,
                AssetBrowserPlugin,
                ProjectManagerPlugin,
                SceneEditorPlugin,
                TilemapEditorPlugin,
                bevy_ecs_tilemap::TilemapPlugin,
                FrameTimeDiagnosticsPlugin::default(),
                PlayModePlugin,
            ))
            .add_systems(Startup, setup_editor_camera)
            // Core systems
            .add_systems(Update, handle_gizmo_mode_shortcuts)
            .add_systems(Update, auto_load_scene_system.after(ProjectManagerSet))
            .add_systems(
                Update,
                (
                    sync_tilemap_on_scene_switch,
                    mark_loaded_scene_entities,
                    handle_save_load,
                )
                    .chain()
                    .after(ProjectManagerSet),
            )
            .add_systems(Update, restore_tilemap_from_level.after(ProjectManagerSet))
            .add_systems(
                PostUpdate,
                cache_runtime_scene_on_scene_switch
                    .in_set(bevy_editor_scene::SceneTabSystemSet::Cache),
            )
            .add_systems(
                Update,
                (
                    reset_auto_loader_on_project_change.after(ProjectManagerSet),
                    handle_editor_actions.after(ProjectManagerSet),
                    monitor_cli_runner.after(ProjectManagerSet),
                ),
            );

        self.frontend.install(app);
    }
}

/// Convert a [`ProjectCommand`] into the underlying CLI command.
fn to_cli_command(command: ProjectCommand) -> CLICommand {
    match command {
        ProjectCommand::Run => CLICommand::Run,
        ProjectCommand::RunScene => CLICommand::RunScene,
        ProjectCommand::RunWeb => CLICommand::RunWeb,
        ProjectCommand::Build => CLICommand::Build,
        ProjectCommand::Lint => CLICommand::Lint,
    }
}

/// Handle frontend-driven actions and bridge them to backend resources.
fn handle_editor_actions(
    mut actions: MessageReader<EditorAction>,
    mut editor_state: ResMut<EditorState>,
    mut cli_runner: ResMut<BevyCLIRunner>,
    mut active_command: ResMut<ActiveProjectCommand>,
    mut editor_events: MessageWriter<EditorEvent>,
    open_scenes: Res<bevy_editor_scene::OpenScenes>,
) {
    for action in actions.read() {
        match action {
            EditorAction::SelectTool(tool) => {
                editor_state.current_tool = *tool;
            }
            EditorAction::SetGridSnap { enabled } => {
                editor_state.grid_snap_enabled = *enabled;
            }
            EditorAction::RunProjectCommand { command } => {
                // If running a scene, set the scene name in the CLI runner
                if matches!(command, ProjectCommand::RunScene) {
                    if let Some(scene_name) = open_scenes.get_active_scene_name() {
                        cli_runner.scene_to_run = Some(scene_name);
                    } else {
                        editor_events.write(EditorEvent::Error {
                            message: "No scene is currently open to run".to_string(),
                        });
                        continue;
                    }
                }

                let cli_command = to_cli_command(*command);
                match cli_runner.run_command(cli_command) {
                    Ok(()) => {
                        editor_events
                            .write(EditorEvent::ProjectCommandStarted { command: *command });
                        active_command.current = Some(*command);
                        active_command.is_running = true;
                    }
                    Err(err) => {
                        editor_events.write(EditorEvent::Error {
                            message: err.clone(),
                        });
                    }
                }
            }
            EditorAction::CancelProjectCommand => {
                if let Some(command) = active_command.current.take() {
                    cli_runner.stop_current_process();
                    editor_events.write(EditorEvent::ProjectCommandFinished {
                        command,
                        success: false,
                        message: Some("Command cancelled by user".to_string()),
                    });
                    active_command.is_running = false;
                } else if cli_runner.is_running() {
                    cli_runner.stop_current_process();
                }
            }
            EditorAction::RequestOpenProject { .. }
            | EditorAction::RequestCreateProject { .. }
            | EditorAction::RequestCloseProject
            | EditorAction::RequestOpenScene { .. }
            | EditorAction::RequestSaveScene { .. }
            | EditorAction::TogglePanel { .. } => {
                debug!("Unhandled editor action: {:?}", action);
            }
        }
    }
}

/// Observe the CLI runner and emit completion events when processes end.
fn monitor_cli_runner(
    cli_runner: Res<BevyCLIRunner>,
    mut active_command: ResMut<ActiveProjectCommand>,
    mut editor_events: MessageWriter<EditorEvent>,
) {
    let running = cli_runner.is_running();

    if active_command.is_running && !running {
        if let Some(command) = active_command.current.take() {
            let (success, message) = cli_runner
                .output_lines
                .last()
                .map(|line| (!line.is_error, Some(line.text.clone())))
                .unwrap_or((true, None));

            editor_events.write(EditorEvent::ProjectCommandFinished {
                command,
                success,
                message,
            });
        }
        active_command.is_running = false;
    } else if running {
        active_command.is_running = true;
    }
}

fn setup_editor_camera(mut commands: Commands) {
    // Camera2d automatically includes Camera as a required component with correct settings for UI
    // DON'T override the Camera component or UI won't render!
    // Temporarily removed EditorCamera to test if it's interfering with UI rendering
    commands.spawn(Camera2d);

    // TODO: Add back EditorCamera once UI is working:
    // commands.spawn((Camera2d, bevy_editor_core::EditorCamera::default()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{App, EventReader, ResMut, Resource, Update};
    use bevy::MinimalPlugins;

    #[derive(Resource, Default)]
    struct CapturedEvents(Vec<EditorEvent>);

    fn capture_editor_events(
        mut reader: EventReader<EditorEvent>,
        mut captured: ResMut<CapturedEvents>,
    ) {
        for event in reader.read() {
            captured.0.push(event.clone());
        }
    }

    #[test]
    fn run_command_without_project_emits_error_event() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<EditorAction>();
        app.add_event::<EditorEvent>();
        app.insert_resource(EditorState::default());
        app.insert_resource(BevyCLIRunner::default());
        app.insert_resource(ActiveProjectCommand::default());
        app.insert_resource(CapturedEvents::default());
        app.add_systems(
            Update,
            (
                handle_editor_actions,
                capture_editor_events.after(handle_editor_actions),
            ),
        );
        app.update();

        app.world_mut().send_event(EditorAction::RunProjectCommand {
            command: ProjectCommand::Run,
        });
        app.update();

        let captured = app.world().resource::<CapturedEvents>();
        assert!(
            captured
                .0
                .iter()
                .any(|event| matches!(event, EditorEvent::Error { .. })),
            "expected an error event when running without a project"
        );
    }
}
