use bevy::prelude::*;
use bevy_editor_project::CurrentProject;
use bevy_editor_scene::{load_scene_into_open_scenes, OpenScenes, SceneAutoLoader};

/// Event to trigger scene loading
#[derive(Event, Message)]
pub struct LoadSceneEvent {
    pub scene_path: String,
}

/// System to auto-load the last opened scene when a project is opened
pub fn auto_load_scene_system(
    project: Option<Res<CurrentProject>>,
    mut open_scenes: ResMut<OpenScenes>,
    mut auto_loader: ResMut<SceneAutoLoader>,
) {
    // Only run once when a project is first loaded
    if auto_loader.has_loaded {
        return;
    }

    let Some(project) = project else {
        return;
    };

    // Check if there's a last opened scene or default scene
    let scene_to_load = project
        .metadata
        .config
        .last_opened_scene
        .as_ref()
        .or(project.metadata.config.default_scene.as_ref());

    if let Some(scene_name) = scene_to_load {
        let scene_path = project.metadata.levels_path.join(scene_name);

        if scene_path.exists() {
            match load_scene_into_open_scenes(&mut open_scenes, &scene_path) {
                Ok(()) => {
                    info!("Auto-loaded scene: {}", scene_name);
                    auto_loader.has_loaded = true;
                }
                Err(err) => {
                    error!("Failed to auto-load scene {}: {}", scene_name, err);
                }
            }
        } else {
            debug!("Scene file not found: {:?}", scene_path);
            // Mark as loaded so we don't spam the console every frame
            auto_loader.has_loaded = true;
        }
    } else {
        debug!("No scene to auto-load");
        auto_loader.has_loaded = true;
    }
}

/// System to reset auto-loader when project changes
pub fn reset_auto_loader_on_project_change(
    mut auto_loader: ResMut<SceneAutoLoader>,
    project: Option<Res<CurrentProject>>,
) {
    // Detect project change by checking if Changed fires
    if project.is_some() {
        auto_loader.has_loaded = false;
    }
}
