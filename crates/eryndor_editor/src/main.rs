use bevy::prelude::*;
use bevy::asset::AssetPlugin;
use eryndor_editor::{EditorPlugin, AssetsBasePath};

fn main() {
    // Determine the assets path based on the current working directory
    // The assets folder is at the workspace root (eryndor-mmo/assets)
    let assets_path = find_assets_path();
    println!("Using assets path: {}", assets_path);

    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Eryndor Editor".to_string(),
                    resolution: (1280u32, 720u32).into(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                file_path: assets_path.clone(),
                ..default()
            })
        )
        .insert_resource(AssetsBasePath::new(assets_path))
        .add_plugins(EditorPlugin)
        .add_systems(Startup, setup)
        .run();
}

/// Find the assets folder by checking various possible locations
fn find_assets_path() -> String {
    use std::path::Path;

    // Get current working directory for debugging
    let cwd = std::env::current_dir().ok();
    if let Some(ref cwd) = cwd {
        println!("Current working directory: {}", cwd.display());
    }

    // Try these paths in order:
    let candidates = [
        "assets",           // Running from workspace root
        "../../assets",     // Running from crates/eryndor_editor
        "../../../assets",  // Running from crates/eryndor_editor/src or similar
    ];

    for candidate in &candidates {
        let path = Path::new(candidate);
        println!("Checking path: {} -> exists: {}", candidate, path.exists());
        if path.exists() && path.is_dir() {
            // Verify it contains expected content (like tiles folder)
            let tiles_path = path.join("tiles");
            if tiles_path.exists() {
                println!("Found assets folder with tiles at: {}", candidate);

                // Convert to absolute path to avoid any path resolution issues
                if let Ok(canonical) = path.canonicalize() {
                    let abs_path = canonical.to_string_lossy().to_string();
                    println!("Using absolute assets path: {}", abs_path);
                    return abs_path;
                }

                return candidate.to_string();
            }
        }
    }

    // Fallback: try to construct absolute path from CARGO_MANIFEST_DIR if available at runtime
    // This won't work at runtime, but we can try environment variable
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let workspace_assets = Path::new(&manifest_dir).join("../../assets");
        if workspace_assets.exists() {
            if let Ok(canonical) = workspace_assets.canonicalize() {
                let path_str = canonical.to_string_lossy().to_string();
                println!("Using canonical assets path: {}", path_str);
                return path_str;
            }
        }
    }

    // Last resort fallback
    println!("WARNING: Could not find assets folder, using default 'assets'");
    "assets".to_string()
}

fn setup(mut commands: Commands) {
    // Spawn 2D camera for viewport
    commands.spawn(Camera2d);
}
