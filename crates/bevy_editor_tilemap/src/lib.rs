//! # Bevy Editor Tilemap
//!
//! Tilemap editing functionality for Bevy-based editors.
//!
//! This crate provides:
//! - **Tileset Management**: Load and manage multiple tilesets
//! - **Layer System**: Multi-layer tilemap editing with z-ordering
//! - **Painting Tools**: Brush, stamp, fill, line, and rectangle tools
//! - **Collision Editing**: Per-tile collision shape authoring
//! - **Tilemap Components**: Integration with bevy_ecs_tilemap
//!
//! ## Features
//!
//! - `tilemap` (default): Full tilemap editing support (requires bevy_ecs_tilemap)
//!
//! ## Example
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_editor_tilemap::TilemapEditorPlugin;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(TilemapEditorPlugin)
//!         .run();
//! }
//! ```

pub mod collision_editor;
pub mod layer_manager;
pub mod map_canvas;
pub mod tile_painter;
pub mod tilemap_component;
pub mod tileset_manager;

// Re-export commonly used types
pub use collision_editor::{CollisionEditor, CollisionTool};
pub use layer_manager::{create_default_layer, ensure_default_layer_system, LayerManager};
pub use map_canvas::{
    handle_canvas_click_painting, handle_paint_tile_events, setup_map_canvas,
    update_map_canvas_on_layer_changes, MapCanvas, MapDimensions, PaintTileEvent,
};
pub use tile_painter::{
    bucket_fill, paint_line, paint_rectangle, paint_single_tile, paint_stamp, PaintMode,
    TilePainter,
};
pub use tilemap_component::{
    cleanup_tilemap_entities, sync_tilemap_entities, TilemapComponent, TilemapLayers,
};
pub use tileset_manager::{
    handle_tileset_load_requests, load_tileset, update_tileset_dimensions, LoadTilesetEvent,
    TilesetInfo, TilesetManager,
};
// Re-export CollisionShape from formats crate
pub use bevy_editor_formats::CollisionShape;

use bevy::prelude::*;

/// Main plugin for tilemap editing functionality
pub struct TilemapEditorPlugin;

impl Plugin for TilemapEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<TilesetManager>()
            .init_resource::<LayerManager>()
            .init_resource::<TilePainter>()
            .init_resource::<MapDimensions>()
            .init_resource::<CollisionEditor>()
            // Events
            .add_message::<LoadTilesetEvent>()
            .add_message::<PaintTileEvent>()
            // Systems
            .add_systems(Startup, ensure_default_layer_system)
            .add_systems(
                Update,
                (
                    handle_tileset_load_requests,
                    update_tileset_dimensions,
                    setup_map_canvas,
                    update_map_canvas_on_layer_changes,
                    handle_paint_tile_events,
                    handle_canvas_click_painting,
                    // Tilemap component systems
                    sync_tilemap_entities,
                    cleanup_tilemap_entities,
                ),
            );

        // Register tilemap component for scene serialization (if bevy_ecs_tilemap feature is enabled)
        #[cfg(feature = "tilemap")]
        {
            app.register_type::<TilemapComponent>()
                .register_type::<TilemapLayers>();
        }
    }
}

/// Convenience plugin bundle for tilemap editing without collision editor
pub struct TilemapCorePlugin;

impl Plugin for TilemapCorePlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<TilesetManager>()
            .init_resource::<LayerManager>()
            .init_resource::<TilePainter>()
            .init_resource::<MapDimensions>()
            // Events
            .add_message::<LoadTilesetEvent>()
            .add_message::<PaintTileEvent>()
            // Systems
            .add_systems(Startup, ensure_default_layer_system)
            .add_systems(
                Update,
                (
                    handle_tileset_load_requests,
                    update_tileset_dimensions,
                    setup_map_canvas,
                    update_map_canvas_on_layer_changes,
                    handle_paint_tile_events,
                    handle_canvas_click_painting,
                    sync_tilemap_entities,
                    cleanup_tilemap_entities,
                ),
            );

        #[cfg(feature = "tilemap")]
        {
            app.register_type::<TilemapComponent>()
                .register_type::<TilemapLayers>();
        }
    }
}
