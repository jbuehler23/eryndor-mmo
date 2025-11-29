//! Tileset list panel - left sidebar showing all tilesets

use bevy_egui::egui;
use crate::editor_state::EditorState;

/// Render the tileset list panel
pub fn render(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    ui.heading("Tilesets");

    // Action buttons
    ui.horizontal(|ui| {
        if ui.button("+ New").clicked() {
            editor_state.tilesets.show_create_dialog = true;
        }
        // TODO: Import from file button
    });

    ui.separator();

    // Tileset list
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let tilesets = &editor_state.world.tile_palette.tilesets;

            if tilesets.is_empty() {
                ui.label("No tilesets loaded");
                ui.label("Create a new tileset or");
                ui.label("wait for tilesets.json to load");
            } else {
                for (index, tileset) in tilesets.iter().enumerate() {
                    let is_selected = editor_state.tilesets.selected_tileset == Some(index);

                    // Display tileset with tile count
                    let label = format!(
                        "{} ({} tiles)",
                        tileset.name,
                        tileset.total_tiles
                    );

                    if ui.selectable_label(is_selected, &label).clicked() {
                        editor_state.tilesets.selected_tileset = Some(index);
                        editor_state.tilesets.selected_tile = None; // Deselect tile when switching tileset
                    }
                }
            }
        });

    ui.separator();

    // Selected tileset properties
    if let Some(tileset_index) = editor_state.tilesets.selected_tileset {
        if let Some(tileset) = editor_state.world.tile_palette.tilesets.get(tileset_index) {
            ui.heading("Tileset Info");

            ui.horizontal(|ui| {
                ui.label("ID:");
                ui.label(&tileset.id);
            });

            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.label(&tileset.name);
            });

            ui.horizontal(|ui| {
                ui.label("Category:");
                ui.label(format!("{:?}", tileset.category));
            });

            ui.horizontal(|ui| {
                ui.label("Tile Size:");
                ui.label(format!("{}px", tileset.display_tile_size));
            });

            ui.horizontal(|ui| {
                ui.label("Total Tiles:");
                ui.label(format!("{}", tileset.total_tiles));
            });

            // Show sources info
            ui.separator();
            ui.label("Sources:");

            for (i, source) in tileset.sources.iter().enumerate() {
                match source {
                    crate::editor_state::TileSource::Spritesheet {
                        path, columns, rows, tile_width, tile_height, ..
                    } => {
                        ui.small(format!(
                            "  [{}] Spritesheet {}x{} ({}x{}px)",
                            i, columns, rows, tile_width, tile_height
                        ));
                        ui.small(format!("      {}", path));
                    }
                    crate::editor_state::TileSource::SingleImage { name, path, .. } => {
                        ui.small(format!("  [{}] Image: {}", i, name));
                        ui.small(format!("      {}", path));
                    }
                }
            }

            // Terrain sets info
            if !tileset.terrain_sets.is_empty() {
                ui.separator();
                ui.label(format!("Terrain Sets: {}", tileset.terrain_sets.len()));
                for ts in &tileset.terrain_sets {
                    ui.small(format!("  - {} ({:?})", ts.name, ts.set_type));
                }
            }
        }
    }
}
