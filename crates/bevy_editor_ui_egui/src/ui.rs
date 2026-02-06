use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::cli_output_panel::{render_cli_output_content, should_show_cli_output};
use crate::scene_tabs::render_scene_tabs_content;
use crate::toolbar::render_toolbar_content;
use crate::CurrentLevel;
use bevy_editor_foundation::EditorState;
use bevy_editor_frontend_api::CliOutputPanelState;
use bevy_editor_frontend_api::EditorAction;
use bevy_editor_project::{
    BevyCLIRunner, EditorWorkspace, ProjectSelection, ProjectSelectionState,
};
use bevy_editor_scene::{EditorScene, EditorSceneEntity, OpenScenes, SceneTabChanged};
use bevy_editor_tilemap::{CollisionEditor, TilePainter};

/// Main UI system - draws all editor UI panels
pub fn ui_system(
    mut contexts: EguiContexts,
    mut editor_state: ResMut<EditorState>,
    _current_level: ResMut<CurrentLevel>,
    mut collision_editor: ResMut<CollisionEditor>,
    workspace: Option<Res<EditorWorkspace>>,
    mut project_selection: Option<ResMut<ProjectSelection>>,
    mut open_scenes: ResMut<OpenScenes>, // Multi-scene support
    mut tile_painter: ResMut<TilePainter>,
    mut cli_runner: ResMut<BevyCLIRunner>,
    mut cli_panel: ResMut<CliOutputPanelState>,
    editor_scene: Res<EditorScene>,
    scene_entity_query: Query<Entity, With<EditorSceneEntity>>,
    mut tab_changed_events: EventWriter<SceneTabChanged>,
    mut editor_actions: EventWriter<EditorAction>,
) {
    let Some(ctx) = contexts.ctx_mut().ok() else {
        return;
    };

    // We need to draw panels in the correct order for egui 0.32
    // Bottom and Top panels first, then side panels, then central panel

    // CLI output panel (if visible)
    if should_show_cli_output(&*cli_runner) {
        cli_panel.visible = true;
    }
    if cli_panel.visible {
        egui::TopBottomPanel::bottom("cli_output")
            .min_height(150.0)
            .max_height(250.0)
            .default_height(200.0)
            .resizable(true)
            .show(ctx, |ui| {
                render_cli_output_content(ui, &mut cli_panel, &mut *cli_runner);
            });
    }

    // Bottom status bar
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if let Some(scene) = open_scenes.active_scene() {
                // Count scene entities (excluding root)
                let entity_count = scene_entity_query.iter().count().saturating_sub(1);

                ui.label(format!("Scene: {}", scene.name));
                ui.separator();
                ui.label(format!("Entities: {}", entity_count));
                ui.separator();
                if editor_scene.is_modified {
                    ui.label("● Modified");
                } else {
                    ui.label("○ Saved");
                }
            } else {
                ui.label("No scene loaded");
            }
        });
    });

    // Top menu bar
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                ui.label("Save/Load via Ctrl+S / Ctrl+O");
                ui.label("(Persists the active scene state)");
                ui.separator();

                // Recent Projects submenu
                if let Some(ref workspace) = workspace {
                    ui.menu_button("Recent Projects", |ui| {
                        if workspace.recent_projects.is_empty() {
                            ui.label("No recent projects");
                        } else {
                            for (idx, project_path) in workspace.recent_projects.iter().enumerate()
                            {
                                // Extract project name from path
                                let project_name = std::path::Path::new(project_path)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or(project_path);

                                if ui
                                    .button(format!("{}. {}", idx + 1, project_name))
                                    .on_hover_text(project_path)
                                    .clicked()
                                {
                                    // Open this project
                                    if let Some(ref mut selection) = project_selection {
                                        selection.state = ProjectSelectionState::Opening {
                                            path: project_path.clone(),
                                        };
                                        info!("Opening recent project: {}", project_path);
                                    }
                                    ui.close();
                                }
                            }

                            ui.separator();
                            if ui.button("Clear Recent Projects").clicked() {
                                // Note: We can't mutate workspace here, will need a separate system
                                info!("Clear recent projects requested (not yet implemented)");
                                ui.close();
                            }
                        }
                    });
                    ui.separator();
                }

                if ui.button("Exit").clicked() {
                    std::process::exit(0);
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Undo (Ctrl+Z)").clicked() {
                    // TODO: Undo system
                    ui.close();
                }
                if ui.button("Redo (Ctrl+Y)").clicked() {
                    // TODO: Redo system
                    ui.close();
                }
            });

            ui.menu_button("View", |ui| {
                ui.checkbox(&mut editor_state.grid_snap_enabled, "Grid Snap");
                ui.add(
                    egui::Slider::new(&mut editor_state.grid_size, 8.0..=128.0).text("Grid Size"),
                );
            });

            ui.menu_button("Window", |ui| {
                if ui
                    .checkbox(&mut collision_editor.active, "Collision Editor")
                    .clicked()
                {
                    ui.close();
                }
            });
        });
    });

    // Toolbar panel
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        render_toolbar_content(
            ui,
            &mut editor_state,
            &mut tile_painter,
            &*cli_runner,
            &mut editor_actions,
        );
    });

    // Scene tabs panel
    egui::TopBottomPanel::top("scene_tabs").show(ctx, |ui| {
        render_scene_tabs_content(ui, &mut open_scenes, &mut tab_changed_events);
    });
}
