pub mod asset_browser_panel;
pub mod build_progress_ui;
pub mod cli_output_panel;
pub mod collision_editor;
pub mod component_commands;
pub mod component_registry;
pub mod current_level;
pub mod editor_commands;
pub mod entity_templates;
pub mod frontend;
pub mod gizmos;
pub mod icons;
pub mod inspector_panel;
pub mod layer_panel;
pub mod panel_manager;
pub mod project_browser;
pub mod project_browser_panel;
pub mod scene_tabs;
pub mod scene_tree_panel;
pub mod shortcuts;
pub mod tilemap_ui;
pub mod tileset_panel;
pub mod toolbar;
pub mod ui;
pub mod viewport_selection;

use bevy::prelude::*;
use bevy_editor_assets::AssetBrowserSet;
use bevy_editor_project::ProjectManagerSet;
use bevy_editor_scene::SceneTabSystemSet;
use bevy_egui::EguiPrimaryContextPass;

pub use asset_browser_panel::asset_browser_panel_ui;
pub use bevy_editor_core::{GizmoMode, GizmoState};
pub use bevy_editor_frontend_api::AssetBrowserPanelState as AssetBrowserPanel;
pub use bevy_editor_frontend_api::ProjectBrowserPanelState as ProjectBrowserPanel;
pub use bevy_editor_frontend_api::{
    scene_tree::{SceneEntityTemplate, SceneTreeCommand, SceneTreeNode},
    AssetBrowserPanelState, CliOutputPanelState, EntityComponentData, InspectorPanelState,
    ProjectBrowserPanelState, SceneTreePanelState,
};
pub use build_progress_ui::build_progress_overlay_ui;
pub use cli_output_panel::{render_cli_output_content, should_show_cli_output};
pub use collision_editor::{collision_editor_ui, handle_collision_input, render_collision_shapes};
pub use component_registry::{
    ComponentCategory, ComponentInfo, ComponentRegistry, EditorComponentRegistry,
};
pub use current_level::CurrentLevel;
pub use editor_commands::{CreateEntityCommand, TransformCommand};
pub use entity_templates::spawn_from_template;
pub use frontend::EguiFrontend;
pub use gizmos::{draw_gizmo_mode_indicator, draw_grid, draw_selection_gizmos};
pub use inspector_panel::render_inspector_panel;
pub use layer_panel::{layer_panel_ui, CreateLayerEvent, DeleteLayerEvent, ReorderLayerEvent};
pub use panel_manager::{render_left_panel, render_right_panel, NameEditBuffer, PanelManager};
pub use project_browser::{
    clear_panel_state_on_project_switch, refresh_project_browser_system, sync_asset_browser_root,
    sync_project_browser_root, FileEntry, FileType, ProjectBrowser,
};
pub use project_browser_panel::project_browser_panel_ui;
pub use scene_tabs::render_scene_tabs_content;
pub use scene_tree_panel::handle_scene_tree_commands;
pub use shortcuts::handle_global_shortcuts;
pub use tilemap_ui::{handle_eyedropper, handle_tile_painting};
pub use tileset_panel::{
    handle_tile_selection_events, SelectTileEvent, SelectTilesetEvent, TilesetZoom,
};
pub use toolbar::render_toolbar_content;
pub use ui::ui_system;
pub use viewport_selection::{
    gizmo_drag_interaction_system, transform_with_undo_system, viewport_entity_selection_system,
    GizmoDragState,
};

/// System ordering buckets for the egui UI layer.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EditorUiSet {
    Input,
    Panels,
    Interaction,
}

/// Plugin wiring the egui-based UI for the editor.
#[derive(Clone, Copy, Default)]
pub struct EditorUiEguiPlugin;

impl Plugin for EditorUiEguiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CliOutputPanelState>()
            .init_resource::<PanelManager>()
            .init_resource::<NameEditBuffer>()
            .init_resource::<TilesetZoom>()
            .init_resource::<AssetBrowserPanel>()
            .init_resource::<ProjectBrowser>()
            .init_resource::<ProjectBrowserPanelState>()
            .init_resource::<EditorComponentRegistry>()
            .init_resource::<SceneTreePanelState>()
            .init_resource::<InspectorPanelState>()
            .init_resource::<GizmoDragState>()
            .init_resource::<CurrentLevel>()
            .add_message::<SceneTreeCommand>()
            .add_message::<SelectTileEvent>()
            .add_message::<SelectTilesetEvent>()
            .configure_sets(
                PostUpdate,
                (SceneTabSystemSet::Cache, SceneTabSystemSet::Apply).chain(),
            )
            .configure_sets(
                Update,
                (
                    EditorUiSet::Input,
                    EditorUiSet::Panels,
                    EditorUiSet::Interaction,
                )
                    .chain(),
            )
            // Input - runs in Update before egui rendering
            .add_systems(
                Update,
                handle_global_shortcuts
                    .in_set(EditorUiSet::Input)
                    .before(ProjectManagerSet),
            )
            // Non-egui state preparation - runs in Update
            .add_systems(
                Update,
                (
                    sync_project_browser_root.after(ProjectManagerSet),
                    project_browser::clear_panel_state_on_project_switch.after(ProjectManagerSet),
                    sync_asset_browser_root
                        .after(ProjectManagerSet)
                        .before(AssetBrowserSet),
                    refresh_project_browser_system.after(ProjectManagerSet),
                )
                    .in_set(EditorUiSet::Panels),
            )
            // CRITICAL: Egui UI rendering must run in EguiPrimaryContextPass schedule
            // to ensure the egui context is properly initialized before use
            .add_systems(
                EguiPrimaryContextPass,
                (
                    build_progress_overlay_ui,
                    (ui_system, render_left_panel, render_right_panel).chain(),
                    collision_editor_ui,
                    draw_gizmo_mode_indicator,
                ),
            )
            // Non-egui interaction systems - runs in Update
            .add_systems(
                Update,
                (
                    viewport_entity_selection_system,
                    gizmo_drag_interaction_system.after(viewport_entity_selection_system),
                    transform_with_undo_system.after(gizmo_drag_interaction_system),
                    handle_tile_selection_events.after(ProjectManagerSet),
                    handle_scene_tree_commands,
                    handle_tile_painting,
                    handle_eyedropper,
                    handle_collision_input,
                    render_collision_shapes,
                    draw_grid,
                    draw_selection_gizmos,
                    component_commands::handle_remove_component_events,
                )
                    .in_set(EditorUiSet::Interaction)
                    .after(EditorUiSet::Panels),
            )
            // Add component handler needs exclusive world access, runs separately
            .add_systems(Update, component_commands::handle_add_component_events)
            .add_systems(
                PostUpdate,
                scene_tabs::sync_editor_scene_on_tab_change
                    .after(ProjectManagerSet)
                    .in_set(SceneTabSystemSet::Apply),
            );
    }
}
