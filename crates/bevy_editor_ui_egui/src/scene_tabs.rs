use crate::icons::Icons;
use bevy::prelude::*;
use bevy_editor_formats::LevelData;
use bevy_editor_scene::{
    sync_active_scene, EditorScene, EditorSceneEntity, OpenScene, OpenScenes, SceneTabChanged,
};
use bevy_egui::egui;

/// Render the scene tabs UI content (called from the main UI system).
pub fn render_scene_tabs_content(
    ui: &mut egui::Ui,
    open_scenes: &mut OpenScenes,
    tab_changed_events: &mut MessageWriter<SceneTabChanged>,
) {
    ui.horizontal(|ui| {
        let mut scene_to_close: Option<usize> = None;
        let mut new_active_index: Option<usize> = None;

        for (idx, scene) in open_scenes.scenes.iter().enumerate() {
            let is_active = idx == open_scenes.active_index;

            let button_text = if scene.is_modified {
                format!("* {}", scene.name)
            } else {
                scene.name.clone()
            };

            let response = ui.selectable_label(is_active, button_text);
            if response.clicked() {
                new_active_index = Some(idx);
            }

            if ui
                .small_button(Icons::CLOSE)
                .on_hover_text("Close scene")
                .clicked()
            {
                scene_to_close = Some(idx);
            }

            ui.separator();
        }

        if ui.button(Icons::NEW).on_hover_text("New scene").clicked() {
            let new_scene = OpenScene::new(
                format!("Untitled {}", open_scenes.scenes.len() + 1),
                LevelData::new("New Level".to_string(), 2000.0, 1000.0),
            );
            open_scenes.add_scene(new_scene);
            tab_changed_events.write(SceneTabChanged {
                new_index: open_scenes.active_index,
            });
        }

        if let Some(index) = new_active_index {
            let old_index = open_scenes.active_index;
            open_scenes.set_active(index);
            if old_index != index {
                tab_changed_events.write(SceneTabChanged { new_index: index });
            }
        }

        if let Some(index) = scene_to_close {
            if open_scenes.scenes[index].is_modified {
                info!("Scene has unsaved changes, close anyway? (dialog not yet implemented)");
            }
            open_scenes.close_scene(index);
            tab_changed_events.write(SceneTabChanged {
                new_index: open_scenes.active_index,
            });
        }
    });
}

/// System to sync [`EditorScene`] with [`OpenScenes`] when tabs change.
pub fn sync_editor_scene_on_tab_change(
    mut tab_events: MessageReader<SceneTabChanged>,
    mut commands: Commands,
    mut editor_scene: ResMut<EditorScene>,
    scene_entities: Query<(Entity, Option<&ChildOf>), With<EditorSceneEntity>>,
    mut name_buffer: ResMut<crate::panel_manager::NameEditBuffer>,
    open_scenes: Res<OpenScenes>,
    asset_server: Res<AssetServer>,
) {
    for event in tab_events.read() {
        info!(
            "Scene tab changed to index {}, clearing editor scene entities",
            event.new_index
        );

        let existing: Vec<(Entity, Option<Entity>)> = scene_entities
            .iter()
            .map(|(entity, parent)| (entity, parent.map(|p| p.parent())))
            .collect();
        sync_active_scene(
            &mut commands,
            &mut editor_scene,
            &open_scenes,
            &asset_server,
            event.new_index,
            existing,
        );

        name_buffer.buffer.clear();
    }
}
