//! Tile properties panel - right sidebar showing selected tile info

use bevy_egui::egui;
use crate::editor_state::{
    EditorState, TilesetEditMode, TileMetadata, CollisionShape,
    CollisionShapeType, TilesetTerrainSet, TerrainDefinition, TerrainMatchMode,
};

/// Render the tile properties panel
pub fn render(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    let Some(tileset_index) = editor_state.tilesets.selected_tileset else {
        ui.label("No tileset selected");
        return;
    };

    // Render different panels based on edit mode
    match editor_state.tilesets.edit_mode {
        TilesetEditMode::Select => {
            if let Some(tile_index) = editor_state.tilesets.selected_tile {
                render_tile_info(ui, editor_state, tile_index, tileset_index);
            } else {
                ui.label("Select a tile to view info");
            }
        }
        TilesetEditMode::Terrain => {
            render_terrain_panel(ui, editor_state, tileset_index);
        }
        TilesetEditMode::Collision => {
            if let Some(tile_index) = editor_state.tilesets.selected_tile {
                render_collision_editor(ui, editor_state, tile_index, tileset_index);
            } else {
                ui.label("Select a tile to edit collision");
            }
        }
    }
}

/// Render basic tile info in Select mode
fn render_tile_info(
    ui: &mut egui::Ui,
    editor_state: &EditorState,
    tile_index: u32,
    tileset_index: usize,
) {
    ui.heading(format!("Tile #{}", tile_index));

    // Find which source this tile belongs to
    if let Some(tileset) = editor_state.world.tile_palette.tilesets.get(tileset_index) {
        if let Some((source, local_index)) = tileset.find_tile(tile_index) {
            match source {
                crate::editor_state::TileSource::Spritesheet { columns, .. } => {
                    let row = local_index / columns;
                    let col = local_index % columns;
                    ui.horizontal(|ui| {
                        ui.label("Position:");
                        ui.label(format!("({}, {})", col, row));
                    });
                }
                crate::editor_state::TileSource::SingleImage { name, .. } => {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.label(name);
                    });
                }
            }
        }

        ui.separator();

        // Show terrain assignments if any
        if let Some(metadata) = tileset.tile_metadata.get(&tile_index) {
            if metadata.terrain_corners.is_some() || metadata.terrain_edges.is_some() {
                ui.group(|ui| {
                    ui.label("Terrain Assignments:");

                    if let Some(corners) = &metadata.terrain_corners {
                        // Get terrain names if we have a selected terrain set
                        let terrain_names: Vec<String> = tileset.terrain_sets
                            .first()
                            .map(|ts| ts.terrains.iter().map(|t| t.name.clone()).collect())
                            .unwrap_or_default();

                        ui.label("  Corners:");
                        for (i, &corner_name) in ["NW", "NE", "SW", "SE"].iter().enumerate() {
                            let value = corners[i]
                                .and_then(|idx| terrain_names.get(idx))
                                .map(|s| s.as_str())
                                .unwrap_or("-");
                            ui.label(format!("    {}: {}", corner_name, value));
                        }
                    }

                    if let Some(edges) = &metadata.terrain_edges {
                        ui.label("  Edges:");
                        for (i, &edge_name) in ["N", "E", "S", "W"].iter().enumerate() {
                            let value = edges[i].map(|v| format!("{}", v)).unwrap_or("-".to_string());
                            ui.label(format!("    {}: {}", edge_name, value));
                        }
                    }
                });
            }

            // Show collision shapes if any
            if !metadata.collision_shapes.is_empty() {
                ui.group(|ui| {
                    ui.label(format!("Collision Shapes: {}", metadata.collision_shapes.len()));

                    for (i, shape) in metadata.collision_shapes.iter().enumerate() {
                        let desc = match shape {
                            CollisionShape::Rectangle { width, height, .. } => {
                                format!("[{}] Rectangle {}x{}", i, width, height)
                            }
                            CollisionShape::Polygon { points } => {
                                format!("[{}] Polygon ({} points)", i, points.len())
                            }
                            CollisionShape::Ellipse { width, height, .. } => {
                                format!("[{}] Ellipse {}x{}", i, width, height)
                            }
                            CollisionShape::Point { name, .. } => {
                                format!("[{}] Point: {}", i, name)
                            }
                        };
                        ui.small(&desc);
                    }
                });
            }

            // Show custom properties if any
            if !metadata.properties.is_empty() {
                ui.group(|ui| {
                    ui.label("Custom Properties:");
                    for (key, value) in &metadata.properties {
                        ui.small(format!("  {}: {}", key, value));
                    }
                });
            }
        } else {
            ui.label("No metadata for this tile");
            ui.small("Switch to Terrain or Collision");
            ui.small("mode to add metadata");
        }
    }
}

/// Render the terrain panel (terrain set management + assignment)
fn render_terrain_panel(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    tileset_index: usize,
) {
    ui.heading("Terrain Sets");

    // Terrain set management section
    render_terrain_set_management(ui, editor_state, tileset_index);

    ui.separator();

    // If a terrain set is selected, show terrains and assignment
    if let Some(set_idx) = editor_state.tilesets.selected_terrain_set {
        render_terrain_list(ui, editor_state, tileset_index, set_idx);

        ui.separator();

        // If a tile is selected, show assignment UI
        if let Some(tile_index) = editor_state.tilesets.selected_tile {
            render_terrain_assignment(ui, editor_state, tile_index, tileset_index, set_idx);
        } else {
            ui.label("Select a tile to assign terrain");
            ui.small("Click on tiles in the viewer");
        }
    }
}

/// Render terrain set management (list, create, delete)
fn render_terrain_set_management(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    tileset_index: usize,
) {
    // Get terrain sets
    let terrain_sets: Vec<(String, String, TerrainMatchMode)> = editor_state
        .world
        .tile_palette
        .tilesets
        .get(tileset_index)
        .map(|ts| ts.terrain_sets.iter().map(|s| (s.id.clone(), s.name.clone(), s.set_type)).collect())
        .unwrap_or_default();

    // Header with add button
    ui.horizontal(|ui| {
        ui.label("Terrain Sets:");
        if ui.small_button("+").on_hover_text("Add new terrain set").clicked() {
            // Create a new terrain set
            let new_id = format!("terrain_set_{}", terrain_sets.len());
            let new_set = TilesetTerrainSet {
                id: new_id.clone(),
                name: format!("Terrain Set {}", terrain_sets.len() + 1),
                set_type: TerrainMatchMode::Corner,
                terrains: vec![
                    TerrainDefinition {
                        name: "Terrain A".to_string(),
                        color: [100, 200, 100, 200],
                        probability: 1.0,
                    },
                    TerrainDefinition {
                        name: "Terrain B".to_string(),
                        color: [100, 100, 200, 200],
                        probability: 1.0,
                    },
                ],
            };

            if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_index) {
                tileset.terrain_sets.push(new_set);
                editor_state.tilesets.selected_terrain_set = Some(tileset.terrain_sets.len() - 1);
                editor_state.status_message = format!("Created terrain set '{}'", new_id);
            }
        }
    });

    if terrain_sets.is_empty() {
        ui.small("No terrain sets defined");
        ui.small("Click + to create one");
        return;
    }

    // List terrain sets
    let selected_set_idx = editor_state.tilesets.selected_terrain_set;
    for (i, (id, name, set_type)) in terrain_sets.iter().enumerate() {
        let is_selected = selected_set_idx == Some(i);

        ui.horizontal(|ui| {
            // Selection
            if ui.selectable_label(is_selected, format!("{} ({:?})", name, set_type)).clicked() {
                editor_state.tilesets.selected_terrain_set = Some(i);
                editor_state.tilesets.selected_terrain = None;
            }

            // Delete button
            if ui.small_button("X").on_hover_text("Delete terrain set").clicked() {
                if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_index) {
                    tileset.terrain_sets.remove(i);
                    if editor_state.tilesets.selected_terrain_set == Some(i) {
                        editor_state.tilesets.selected_terrain_set = None;
                    }
                    editor_state.status_message = format!("Deleted terrain set '{}'", id);
                }
            }
        });
    }

    // Edit selected terrain set properties
    if let Some(set_idx) = editor_state.tilesets.selected_terrain_set {
        if set_idx < terrain_sets.len() {
            ui.separator();
            ui.label("Edit Terrain Set:");

            // Name editing
            let mut name = terrain_sets[set_idx].1.clone();
            if ui.text_edit_singleline(&mut name).changed() {
                if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_index) {
                    if let Some(terrain_set) = tileset.terrain_sets.get_mut(set_idx) {
                        terrain_set.name = name;
                    }
                }
            }

            // Type selector
            let current_type = terrain_sets[set_idx].2;
            egui::ComboBox::from_label("Type")
                .selected_text(format!("{:?}", current_type))
                .show_ui(ui, |ui| {
                    for mode in [TerrainMatchMode::Corner, TerrainMatchMode::Edge, TerrainMatchMode::Mixed] {
                        if ui.selectable_label(current_type == mode, format!("{:?}", mode)).clicked() {
                            if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_index) {
                                if let Some(terrain_set) = tileset.terrain_sets.get_mut(set_idx) {
                                    terrain_set.set_type = mode;
                                }
                            }
                        }
                    }
                });
        }
    }
}

/// Render terrain list within a terrain set with color pickers
fn render_terrain_list(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    tileset_index: usize,
    set_idx: usize,
) {
    // Get terrains for this set
    let terrains: Vec<TerrainDefinition> = editor_state
        .world
        .tile_palette
        .tilesets
        .get(tileset_index)
        .and_then(|ts| ts.terrain_sets.get(set_idx))
        .map(|s| s.terrains.clone())
        .unwrap_or_default();

    // Header with add button
    ui.horizontal(|ui| {
        ui.label("Terrains:");
        if ui.small_button("+").on_hover_text("Add new terrain").clicked() {
            if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_index) {
                if let Some(terrain_set) = tileset.terrain_sets.get_mut(set_idx) {
                    let colors = [
                        [200, 100, 100, 200], // Red
                        [100, 200, 100, 200], // Green
                        [100, 100, 200, 200], // Blue
                        [200, 200, 100, 200], // Yellow
                        [200, 100, 200, 200], // Magenta
                        [100, 200, 200, 200], // Cyan
                    ];
                    let color = colors[terrain_set.terrains.len() % colors.len()];

                    terrain_set.terrains.push(TerrainDefinition {
                        name: format!("Terrain {}", terrain_set.terrains.len() + 1),
                        color,
                        probability: 1.0,
                    });
                }
            }
        }
    });

    if terrains.is_empty() {
        ui.small("No terrains defined");
        return;
    }

    // List terrains with color picker
    let selected_terrain = editor_state.tilesets.selected_terrain;
    let mut terrain_to_delete: Option<usize> = None;

    for (i, terrain) in terrains.iter().enumerate() {
        let is_selected = selected_terrain == Some(i);

        ui.horizontal(|ui| {
            // Color picker
            let mut color = [
                terrain.color[0] as f32 / 255.0,
                terrain.color[1] as f32 / 255.0,
                terrain.color[2] as f32 / 255.0,
            ];

            if ui.color_edit_button_rgb(&mut color).changed() {
                if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_index) {
                    if let Some(terrain_set) = tileset.terrain_sets.get_mut(set_idx) {
                        if let Some(t) = terrain_set.terrains.get_mut(i) {
                            t.color = [
                                (color[0] * 255.0) as u8,
                                (color[1] * 255.0) as u8,
                                (color[2] * 255.0) as u8,
                                200,
                            ];
                        }
                    }
                }
            }

            // Selectable name
            if ui.selectable_label(is_selected, &terrain.name).clicked() {
                editor_state.tilesets.selected_terrain = Some(i);
            }

            // Delete button
            if ui.small_button("X").on_hover_text("Delete terrain").clicked() {
                terrain_to_delete = Some(i);
            }
        });

        // Inline edit name if selected
        if is_selected {
            let mut name = terrain.name.clone();
            ui.horizontal(|ui| {
                ui.label("  Name:");
                if ui.text_edit_singleline(&mut name).changed() {
                    if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_index) {
                        if let Some(terrain_set) = tileset.terrain_sets.get_mut(set_idx) {
                            if let Some(t) = terrain_set.terrains.get_mut(i) {
                                t.name = name;
                            }
                        }
                    }
                }
            });
        }
    }

    // Delete terrain if requested
    if let Some(del_idx) = terrain_to_delete {
        if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_index) {
            if let Some(terrain_set) = tileset.terrain_sets.get_mut(set_idx) {
                terrain_set.terrains.remove(del_idx);
                if editor_state.tilesets.selected_terrain == Some(del_idx) {
                    editor_state.tilesets.selected_terrain = None;
                }
            }
        }
    }

    // Instructions
    ui.separator();
    ui.small("Select a terrain, then click");
    ui.small("tile corners in the viewer");
    ui.small("to assign it.");
}

/// Render terrain assignment UI for a specific tile
fn render_terrain_assignment(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    tile_index: u32,
    tileset_index: usize,
    set_idx: usize,
) {
    ui.heading(format!("Tile #{}", tile_index));

    // Get terrain set info
    let terrain_set: Option<TilesetTerrainSet> = editor_state
        .world
        .tile_palette
        .tilesets
        .get(tileset_index)
        .and_then(|ts| ts.terrain_sets.get(set_idx).cloned());

    let Some(terrain_set) = terrain_set else {
        return;
    };

    // Get current assignments
    let current_corners = editor_state
        .world
        .tile_palette
        .tilesets
        .get(tileset_index)
        .and_then(|ts| ts.tile_metadata.get(&tile_index))
        .and_then(|m| m.terrain_corners)
        .unwrap_or([None; 4]);

    let selected_terrain = editor_state.tilesets.selected_terrain;

    // Visual corner assignment grid
    ui.label("Corner Assignments:");

    // Draw a visual representation of the tile corners
    let button_size = egui::vec2(50.0, 50.0);

    ui.horizontal(|ui| {
        // NW corner
        let nw_label = get_corner_button_label(&terrain_set, current_corners[0], "NW");
        let nw_color = get_corner_button_color(&terrain_set, current_corners[0]);
        if ui.add_sized(button_size, egui::Button::new(&nw_label).fill(nw_color)).clicked() {
            set_corner(editor_state, tile_index, tileset_index, 0, selected_terrain);
        }

        // NE corner
        let ne_label = get_corner_button_label(&terrain_set, current_corners[1], "NE");
        let ne_color = get_corner_button_color(&terrain_set, current_corners[1]);
        if ui.add_sized(button_size, egui::Button::new(&ne_label).fill(ne_color)).clicked() {
            set_corner(editor_state, tile_index, tileset_index, 1, selected_terrain);
        }
    });

    ui.horizontal(|ui| {
        // SW corner
        let sw_label = get_corner_button_label(&terrain_set, current_corners[2], "SW");
        let sw_color = get_corner_button_color(&terrain_set, current_corners[2]);
        if ui.add_sized(button_size, egui::Button::new(&sw_label).fill(sw_color)).clicked() {
            set_corner(editor_state, tile_index, tileset_index, 2, selected_terrain);
        }

        // SE corner
        let se_label = get_corner_button_label(&terrain_set, current_corners[3], "SE");
        let se_color = get_corner_button_color(&terrain_set, current_corners[3]);
        if ui.add_sized(button_size, egui::Button::new(&se_label).fill(se_color)).clicked() {
            set_corner(editor_state, tile_index, tileset_index, 3, selected_terrain);
        }
    });

    ui.separator();

    // Quick actions
    ui.horizontal(|ui| {
        if ui.button("Fill All").on_hover_text("Fill all corners with selected terrain").clicked() {
            if let Some(terrain_idx) = selected_terrain {
                for corner in 0..4 {
                    set_corner(editor_state, tile_index, tileset_index, corner, Some(terrain_idx));
                }
            }
        }

        if ui.button("Clear All").clicked() {
            if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_index) {
                if let Some(metadata) = tileset.tile_metadata.get_mut(&tile_index) {
                    metadata.terrain_corners = None;
                }
            }
        }
    });
}

fn get_corner_button_label(terrain_set: &TilesetTerrainSet, terrain_idx: Option<usize>, fallback: &str) -> String {
    terrain_idx
        .and_then(|i| terrain_set.terrains.get(i))
        .map(|t| t.name.chars().take(2).collect::<String>())
        .unwrap_or_else(|| fallback.to_string())
}

fn get_corner_button_color(terrain_set: &TilesetTerrainSet, terrain_idx: Option<usize>) -> egui::Color32 {
    terrain_idx
        .and_then(|i| terrain_set.terrains.get(i))
        .map(|t| egui::Color32::from_rgba_unmultiplied(t.color[0], t.color[1], t.color[2], 150))
        .unwrap_or(egui::Color32::from_gray(60))
}

fn set_corner(
    editor_state: &mut EditorState,
    tile_index: u32,
    tileset_index: usize,
    corner_idx: usize,
    terrain: Option<usize>,
) {
    if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_index) {
        let metadata = tileset.tile_metadata.entry(tile_index).or_insert_with(TileMetadata::default);
        let corners = metadata.terrain_corners.get_or_insert([None; 4]);
        corners[corner_idx] = terrain;
    }
}

/// Render collision editor UI in Collision mode
fn render_collision_editor(
    ui: &mut egui::Ui,
    editor_state: &mut EditorState,
    tile_index: u32,
    tileset_index: usize,
) {
    ui.heading(format!("Tile #{} Collision", tile_index));

    // Tool selection
    ui.label("Shape Tool:");
    ui.horizontal_wrapped(|ui| {
        for shape_type in CollisionShapeType::all() {
            let is_selected = editor_state.tilesets.collision_editor.drawing_shape == Some(*shape_type);
            if ui.selectable_label(is_selected, shape_type.label()).clicked() {
                if is_selected {
                    editor_state.tilesets.collision_editor.drawing_shape = None;
                } else {
                    editor_state.tilesets.collision_editor.drawing_shape = Some(*shape_type);
                }
            }
        }
    });

    ui.separator();

    // Quick actions
    ui.horizontal(|ui| {
        if ui.button("Full Tile").on_hover_text("Add full tile collision").clicked() {
            add_full_tile_collision(editor_state, tile_index, tileset_index);
        }
        if ui.button("Clear All").clicked() {
            clear_collision(editor_state, tile_index, tileset_index);
        }
    });

    ui.separator();

    // List existing shapes
    ui.label("Collision Shapes:");

    let shapes: Vec<CollisionShape> = editor_state
        .world
        .tile_palette
        .tilesets
        .get(tileset_index)
        .and_then(|ts| ts.tile_metadata.get(&tile_index))
        .map(|m| m.collision_shapes.clone())
        .unwrap_or_default();

    if shapes.is_empty() {
        ui.small("No collision shapes");
    } else {
        let mut to_delete: Option<usize> = None;

        for (i, shape) in shapes.iter().enumerate() {
            let is_selected = editor_state.tilesets.collision_editor.selected_shape == Some(i);

            ui.horizontal(|ui| {
                let desc = match shape {
                    CollisionShape::Rectangle { x, y, width, height } => {
                        format!("Rect ({:.0},{:.0}) {:.0}x{:.0}", x, y, width, height)
                    }
                    CollisionShape::Polygon { points } => {
                        format!("Poly ({} pts)", points.len())
                    }
                    CollisionShape::Ellipse { x, y, width, height } => {
                        format!("Ellipse ({:.0},{:.0}) {:.0}x{:.0}", x, y, width, height)
                    }
                    CollisionShape::Point { x, y, name } => {
                        format!("Pt ({:.0},{:.0}) {}", x, y, name)
                    }
                };

                if ui.selectable_label(is_selected, &desc).clicked() {
                    editor_state.tilesets.collision_editor.selected_shape = Some(i);
                }

                if ui.small_button("X").clicked() {
                    to_delete = Some(i);
                }
            });
        }

        // Delete shape if requested
        if let Some(delete_idx) = to_delete {
            if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_index) {
                if let Some(metadata) = tileset.tile_metadata.get_mut(&tile_index) {
                    metadata.collision_shapes.remove(delete_idx);
                    editor_state.tilesets.collision_editor.selected_shape = None;
                }
            }
        }
    }
}

fn add_full_tile_collision(editor_state: &mut EditorState, tile_index: u32, tileset_index: usize) {
    if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_index) {
        let tile_size = tileset.display_tile_size as f32;
        let metadata = tileset.tile_metadata.entry(tile_index).or_insert_with(TileMetadata::default);

        metadata.collision_shapes.push(CollisionShape::Rectangle {
            x: 0.0,
            y: 0.0,
            width: tile_size,
            height: tile_size,
        });
    }
}

fn clear_collision(editor_state: &mut EditorState, tile_index: u32, tileset_index: usize) {
    if let Some(tileset) = editor_state.world.tile_palette.tilesets.get_mut(tileset_index) {
        if let Some(metadata) = tileset.tile_metadata.get_mut(&tile_index) {
            metadata.collision_shapes.clear();
        }
    }
    editor_state.tilesets.collision_editor.selected_shape = None;
}
