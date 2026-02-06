use bevy::prelude::*;
use bevy_editor_app::EditorAppPlugin;

// Import the appropriate frontend based on feature flags
#[cfg(feature = "egui-ui")]
use bevy_editor_ui_egui::EguiFrontend;
#[cfg(feature = "egui-ui")]
use bevy_egui::EguiPlugin;

#[cfg(feature = "feathers-ui")]
use bevy_editor_ui_feathers::FeathersFrontend;

fn main() {
    // Log which UI frontend is active
    #[cfg(all(feature = "egui-ui", not(feature = "feathers-ui")))]
    info!("Starting Bevy Editor with egui frontend");
    #[cfg(feature = "feathers-ui")]
    info!("Starting Bevy Editor with bevy_feathers frontend");

    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy Editor".to_string(),
                    resolution: (1920, 1080).into(),
                    ..default()
                }),
                ..default()
            }),
    );

    // Add frontend-specific plugins
    // Features are mutually exclusive - prefer feathers-ui if both are enabled
    #[cfg(all(feature = "feathers-ui", not(feature = "egui-ui")))]
    {
        app.add_plugins(EditorAppPlugin::new(FeathersFrontend::default()));
    }

    #[cfg(all(feature = "feathers-ui", feature = "egui-ui"))]
    {
        warn!("Both egui-ui and feathers-ui features enabled - defaulting to feathers-ui");
        app.add_plugins(EditorAppPlugin::new(FeathersFrontend::default()));
    }

    #[cfg(all(feature = "egui-ui", not(feature = "feathers-ui")))]
    {
        app.add_plugins(EguiPlugin::default());
        app.add_plugins(EditorAppPlugin::new(EguiFrontend::default()));
    }

    app.run();
}
