//! World/Zone Editor Module
//! Visual zone/level design with canvas, collision shapes, and spawn regions.

use bevy_egui::egui;
use crate::editor_state::{EditorState, WorldTool};

/// Render the world editor module
pub fn render(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    // Render side panels first so they claim their space
    render_left_sidebar(ui, editor_state);
    render_properties_panel(ui, editor_state);
    // Central panel fills remaining space
    render_canvas(ui, editor_state);
    // Render dialogs on top
    render_create_zone_dialog(ui, editor_state);
}

fn render_left_sidebar(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    egui::SidePanel::left("world_left_panel")
        .default_width(200.0)
        .show_inside(ui, |ui| {
            ui.heading("Zones");

            ui.horizontal(|ui| {
                // New zone button
                if ui.button("+ New Zone").clicked() {
                    editor_state.world.show_create_dialog = true;
                }
                // Refresh button
                if ui.button("Refresh").clicked() {
                    editor_state.action_load_zones = true;
                }
            });

            ui.separator();

            // Zone list
            egui::ScrollArea::vertical().show(ui, |ui| {
                if editor_state.world.zone_list.is_empty() {
                    ui.label("No zones loaded");
                    ui.label("Click 'Refresh' to load zones");
                } else {
                    for zone in &editor_state.world.zone_list {
                        let is_selected = editor_state.world.current_zone.as_ref() == Some(&zone.id);
                        if ui.selectable_label(is_selected, &zone.name).clicked() {
                            editor_state.world.current_zone = Some(zone.id.clone());
                            editor_state.status_message = format!("Selected zone: {}", zone.name);
                        }
                    }
                }
            });

            ui.separator();

            // Tools
            ui.heading("Tools");

            let tools = [
                (WorldTool::Select, "Select", "Select and move entities"),
                (WorldTool::Pan, "Pan", "Pan the camera"),
                (WorldTool::PlaceEntity, "Place", "Place entities from palette"),
                (WorldTool::DrawCollision, "Collision", "Draw collision shapes"),
                (WorldTool::DrawSpawnRegion, "Spawn", "Draw spawn regions"),
            ];

            for (tool, label, tooltip) in tools {
                let is_selected = editor_state.world.active_tool == tool;
                if ui.selectable_label(is_selected, label).on_hover_text(tooltip).clicked() {
                    editor_state.world.active_tool = tool;
                }
            }

            ui.separator();

            // Entity Palette
            ui.heading("Entity Palette");
            ui.label("NPCs");
            // TODO: List available NPCs to place

            ui.label("Enemies");
            // TODO: List available enemies to place
        });
}

fn render_canvas(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    egui::CentralPanel::default().show_inside(ui, |ui| {
        // Canvas toolbar
        ui.horizontal(|ui| {
            ui.checkbox(&mut editor_state.world.show_grid, "Grid");
            ui.checkbox(&mut editor_state.world.snap_to_grid, "Snap");
            ui.checkbox(&mut editor_state.world.show_collisions, "Collisions");
            ui.checkbox(&mut editor_state.world.show_spawn_regions, "Spawns");

            ui.separator();

            ui.label("Zoom:");
            if ui.button("-").clicked() {
                editor_state.world.zoom = (editor_state.world.zoom - 0.1).max(0.1);
            }
            ui.label(format!("{:.0}%", editor_state.world.zoom * 100.0));
            if ui.button("+").clicked() {
                editor_state.world.zoom = (editor_state.world.zoom + 0.1).min(5.0);
            }
            if ui.button("Reset").clicked() {
                editor_state.world.zoom = 1.0;
                editor_state.world.camera_pos = bevy::prelude::Vec2::ZERO;
            }
        });

        ui.separator();

        // Main canvas
        let available_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click_and_drag());

        // Draw grid
        if editor_state.world.show_grid {
            draw_grid(&painter, &response.rect, editor_state);
        }

        // Draw canvas content placeholder
        painter.text(
            response.rect.center(),
            egui::Align2::CENTER_CENTER,
            if editor_state.world.current_zone.is_some() {
                "Zone loaded - Canvas rendering coming soon"
            } else {
                "Select a zone from the list or create a new one"
            },
            egui::FontId::default(),
            egui::Color32::GRAY,
        );

        // Handle canvas interactions
        if response.dragged() {
            if editor_state.world.active_tool == WorldTool::Pan {
                let delta = response.drag_delta();
                editor_state.world.camera_pos.x -= delta.x / editor_state.world.zoom;
                editor_state.world.camera_pos.y -= delta.y / editor_state.world.zoom;
            }
        }
    });
}

fn render_properties_panel(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    egui::SidePanel::right("world_right_panel")
        .default_width(250.0)
        .show_inside(ui, |ui| {
            ui.heading("Properties");

            if let Some(zone_id) = &editor_state.world.current_zone {
                ui.label(format!("Zone: {}", zone_id));

                ui.separator();

                // Zone properties
                ui.label("Zone Settings:");
                // TODO: Zone name, bounds, etc.

                ui.separator();

                // Selected entity properties
                ui.label("Selected Entity:");
                ui.label("(No entity selected)");
                // TODO: Show selected entity properties
            } else {
                ui.label("No zone selected");
            }
        });
}

fn draw_grid(painter: &egui::Painter, rect: &egui::Rect, editor_state: &EditorState) {
    let grid_size = editor_state.world.grid_size * editor_state.world.zoom;

    // Safety guard: prevent infinite loop if grid size is too small
    if grid_size < 1.0 {
        return;
    }

    let grid_color = egui::Color32::from_rgba_unmultiplied(80, 80, 80, 60);

    let start_x = rect.left();
    let start_y = rect.top();

    // Vertical lines
    let mut x = start_x;
    while x < rect.right() {
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            egui::Stroke::new(1.0, grid_color),
        );
        x += grid_size;
    }

    // Horizontal lines
    let mut y = start_y;
    while y < rect.bottom() {
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(1.0, grid_color),
        );
        y += grid_size;
    }
}

fn render_create_zone_dialog(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    if !editor_state.world.show_create_dialog {
        return;
    }

    egui::Window::new("Create New Zone")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.set_min_width(300.0);

            ui.horizontal(|ui| {
                ui.label("Zone Name:");
                ui.text_edit_singleline(&mut editor_state.world.new_zone_name);
            });

            ui.horizontal(|ui| {
                ui.label("Width:");
                ui.add(egui::DragValue::new(&mut editor_state.world.new_zone_width)
                    .speed(10.0)
                    .range(100.0..=10000.0)
                    .suffix(" px"));
            });

            ui.horizontal(|ui| {
                ui.label("Height:");
                ui.add(egui::DragValue::new(&mut editor_state.world.new_zone_height)
                    .speed(10.0)
                    .range(100.0..=10000.0)
                    .suffix(" px"));
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Create").clicked() {
                    editor_state.action_create_zone = true;
                }
                if ui.button("Cancel").clicked() {
                    editor_state.world.show_create_dialog = false;
                    editor_state.world.new_zone_name.clear();
                }
            });
        });
}
