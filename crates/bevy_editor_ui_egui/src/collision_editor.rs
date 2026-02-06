use bevy::prelude::*;
use bevy_editor_formats::{CollisionShape, Vector2};
use bevy_editor_tilemap::{CollisionEditor, CollisionTool, TilesetManager};
use bevy_egui::{egui, EguiContexts};

/// UI system for collision editor
pub fn collision_editor_ui(
    mut contexts: EguiContexts,
    mut collision_editor: ResMut<CollisionEditor>,
    tileset_manager: Res<TilesetManager>,
) {
    if !collision_editor.active {
        return;
    }

    let Some(ctx) = contexts.ctx_mut().ok() else {
        return;
    };

    // Collision editor window
    let mut open = true;
    egui::Window::new("Collision Editor")
        .open(&mut open)
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading("Tile Collision Editor");
            ui.separator();

            // Tool selection
            ui.label("Tool:");
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        collision_editor.current_tool == CollisionTool::Select,
                        "Select",
                    )
                    .clicked()
                {
                    collision_editor.current_tool = CollisionTool::Select;
                }
                if ui
                    .selectable_label(
                        collision_editor.current_tool == CollisionTool::Rectangle,
                        "Rect",
                    )
                    .clicked()
                {
                    collision_editor.current_tool = CollisionTool::Rectangle;
                }
                if ui
                    .selectable_label(
                        collision_editor.current_tool == CollisionTool::Ellipse,
                        "Ellipse",
                    )
                    .clicked()
                {
                    collision_editor.current_tool = CollisionTool::Ellipse;
                }
            });
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        collision_editor.current_tool == CollisionTool::Polygon,
                        "Polygon",
                    )
                    .clicked()
                {
                    collision_editor.current_tool = CollisionTool::Polygon;
                }
                if ui
                    .selectable_label(
                        collision_editor.current_tool == CollisionTool::Polyline,
                        "Polyline",
                    )
                    .clicked()
                {
                    collision_editor.current_tool = CollisionTool::Polyline;
                }
                if ui
                    .selectable_label(
                        collision_editor.current_tool == CollisionTool::Point,
                        "Point",
                    )
                    .clicked()
                {
                    collision_editor.current_tool = CollisionTool::Point;
                }
            });

            ui.separator();

            // Tile selection
            if let (Some(tileset_id), Some(tile_id)) = (
                tileset_manager.selected_tileset_id,
                tileset_manager.selected_tile_id,
            ) {
                ui.label(format!(
                    "Editing Tile: {} (Tileset {})",
                    tile_id, tileset_id
                ));

                if collision_editor.current_tile_id != Some(tile_id) {
                    // Load collision shapes for this tile
                    collision_editor.current_tile_id = Some(tile_id);
                    // TODO: Load shapes from tileset collision data
                    collision_editor.shapes.clear();
                }
            } else {
                ui.label("Select a tile to edit collision shapes");
            }

            ui.separator();

            // Shape list
            ui.label("Collision Shapes:");
            let mut to_remove = None;
            let mut new_selection = None;
            let current_selection = collision_editor.selected_shape;

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for (idx, shape) in collision_editor.shapes.iter().enumerate() {
                        ui.horizontal(|ui| {
                            let shape_name = match shape {
                                CollisionShape::Rectangle { .. } => "Rectangle",
                                CollisionShape::Ellipse { .. } => "Ellipse",
                                CollisionShape::Polygon { .. } => "Polygon",
                                CollisionShape::Polyline { .. } => "Polyline",
                                CollisionShape::Point { .. } => "Point",
                            };

                            if ui
                                .selectable_label(current_selection == Some(idx), shape_name)
                                .clicked()
                            {
                                new_selection = Some(idx);
                            }

                            if ui.small_button("✖").clicked() {
                                to_remove = Some(idx);
                            }
                        });
                    }
                });

            if let Some(idx) = new_selection {
                collision_editor.selected_shape = Some(idx);
            }
            if let Some(idx) = to_remove {
                collision_editor.shapes.remove(idx);
                if collision_editor.selected_shape == Some(idx) {
                    collision_editor.selected_shape = None;
                }
            }

            ui.separator();

            // Shape properties
            if let Some(idx) = collision_editor.selected_shape {
                if let Some(shape) = collision_editor.shapes.get_mut(idx) {
                    ui.label("Shape Properties:");

                    match shape {
                        CollisionShape::Rectangle {
                            x,
                            y,
                            width,
                            height,
                        } => {
                            ui.add(egui::Slider::new(x, 0.0..=100.0).text("X"));
                            ui.add(egui::Slider::new(y, 0.0..=100.0).text("Y"));
                            ui.add(egui::Slider::new(width, 1.0..=100.0).text("Width"));
                            ui.add(egui::Slider::new(height, 1.0..=100.0).text("Height"));
                        }
                        CollisionShape::Ellipse { x, y, rx, ry } => {
                            ui.add(egui::Slider::new(x, 0.0..=100.0).text("X"));
                            ui.add(egui::Slider::new(y, 0.0..=100.0).text("Y"));
                            ui.add(egui::Slider::new(rx, 1.0..=50.0).text("Radius X"));
                            ui.add(egui::Slider::new(ry, 1.0..=50.0).text("Radius Y"));
                        }
                        CollisionShape::Point { x, y } => {
                            ui.add(egui::Slider::new(x, 0.0..=100.0).text("X"));
                            ui.add(egui::Slider::new(y, 0.0..=100.0).text("Y"));
                        }
                        CollisionShape::Polygon { points }
                        | CollisionShape::Polyline { points } => {
                            ui.label(format!("Points: {}", points.len()));
                            // Could add point editing UI here
                        }
                    }
                }
            }

            ui.separator();

            // Instructions
            ui.label("Instructions:");
            match collision_editor.current_tool {
                CollisionTool::Select => ui.label("Click to select shapes, drag to move"),
                CollisionTool::Rectangle | CollisionTool::Ellipse => {
                    ui.label("Click and drag to create shape")
                }
                CollisionTool::Polygon | CollisionTool::Polyline => {
                    ui.label("Click to add points, double-click to finish")
                }
                CollisionTool::Point => ui.label("Click to place point"),
            };
        });

    collision_editor.active = open;
}

/// System for handling collision shape input
pub fn handle_collision_input(
    mut collision_editor: ResMut<CollisionEditor>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if !collision_editor.active || collision_editor.current_tile_id.is_none() {
        return;
    }

    let Some(window) = windows.iter().next() else {
        return;
    };
    let Some((camera, camera_transform)) = camera_query.iter().next() else {
        return;
    };

    // Get mouse position in world space
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    // Handle mouse input based on current tool
    match collision_editor.current_tool {
        CollisionTool::Rectangle | CollisionTool::Ellipse => {
            if mouse_button.just_pressed(MouseButton::Left) {
                collision_editor.drawing = true;
                collision_editor.drag_start = Some(world_pos);
            }

            if mouse_button.just_released(MouseButton::Left) && collision_editor.drawing {
                if let Some(start) = collision_editor.drag_start {
                    let shape = match collision_editor.current_tool {
                        CollisionTool::Rectangle => {
                            let min_x = start.x.min(world_pos.x);
                            let min_y = start.y.min(world_pos.y);
                            let width = (world_pos.x - start.x).abs();
                            let height = (world_pos.y - start.y).abs();

                            CollisionShape::Rectangle {
                                x: min_x,
                                y: min_y,
                                width,
                                height,
                            }
                        }
                        CollisionTool::Ellipse => {
                            let center_x = (start.x + world_pos.x) / 2.0;
                            let center_y = (start.y + world_pos.y) / 2.0;
                            let rx = (world_pos.x - start.x).abs() / 2.0;
                            let ry = (world_pos.y - start.y).abs() / 2.0;

                            CollisionShape::Ellipse {
                                x: center_x,
                                y: center_y,
                                rx,
                                ry,
                            }
                        }
                        _ => unreachable!(),
                    };

                    collision_editor.shapes.push(shape);
                }

                collision_editor.drawing = false;
                collision_editor.drag_start = None;
            }
        }
        CollisionTool::Point => {
            if mouse_button.just_pressed(MouseButton::Left) {
                collision_editor.shapes.push(CollisionShape::Point {
                    x: world_pos.x,
                    y: world_pos.y,
                });
            }
        }
        CollisionTool::Polygon | CollisionTool::Polyline => {
            // Single click adds point
            if mouse_button.just_pressed(MouseButton::Left) {
                collision_editor
                    .polygon_points
                    .push(Vector2::new(world_pos.x, world_pos.y));
            }

            // Double click (or right click) finishes polygon/polyline
            if mouse_button.just_pressed(MouseButton::Right)
                && !collision_editor.polygon_points.is_empty()
            {
                let shape = match collision_editor.current_tool {
                    CollisionTool::Polygon => CollisionShape::Polygon {
                        points: collision_editor.polygon_points.clone(),
                    },
                    CollisionTool::Polyline => CollisionShape::Polyline {
                        points: collision_editor.polygon_points.clone(),
                    },
                    _ => unreachable!(),
                };

                collision_editor.shapes.push(shape);
                collision_editor.polygon_points.clear();
            }
        }
        CollisionTool::Select => {
            // TODO: Implement shape selection and movement
        }
    }
}

/// System to render collision shapes for the current tile
pub fn render_collision_shapes(mut gizmos: Gizmos, collision_editor: Res<CollisionEditor>) {
    if !collision_editor.active {
        return;
    }

    // Render all shapes
    for (idx, shape) in collision_editor.shapes.iter().enumerate() {
        let color = if collision_editor.selected_shape == Some(idx) {
            Color::srgb(1.0, 1.0, 0.0) // Yellow for selected
        } else {
            Color::srgb(0.0, 1.0, 0.0) // Green for normal
        };

        match shape {
            CollisionShape::Rectangle {
                x,
                y,
                width,
                height,
            } => {
                gizmos.rect_2d(
                    Isometry2d::new(
                        Vec2::new(*x + width / 2.0, *y + height / 2.0),
                        Rot2::default(),
                    ),
                    Vec2::new(*width, *height),
                    color,
                );
            }
            CollisionShape::Ellipse { x, y, rx, ry } => {
                gizmos.ellipse_2d(
                    Isometry2d::new(Vec2::new(*x, *y), Rot2::default()),
                    Vec2::new(*rx, *ry),
                    color,
                );
            }
            CollisionShape::Polygon { points } => {
                for i in 0..points.len() {
                    let start = points[i];
                    let end = points[(i + 1) % points.len()];
                    gizmos.line_2d(Vec2::new(start.x, start.y), Vec2::new(end.x, end.y), color);
                }
            }
            CollisionShape::Polyline { points } => {
                for i in 0..points.len().saturating_sub(1) {
                    let start = points[i];
                    let end = points[i + 1];
                    gizmos.line_2d(Vec2::new(start.x, start.y), Vec2::new(end.x, end.y), color);
                }
            }
            CollisionShape::Point { x, y } => {
                gizmos.circle_2d(Vec2::new(*x, *y), 3.0, color);
            }
        }
    }

    // Render current polygon/polyline being drawn
    if matches!(
        collision_editor.current_tool,
        CollisionTool::Polygon | CollisionTool::Polyline
    ) {
        for i in 0..collision_editor.polygon_points.len().saturating_sub(1) {
            let start = collision_editor.polygon_points[i];
            let end = collision_editor.polygon_points[i + 1];
            gizmos.line_2d(
                Vec2::new(start.x, start.y),
                Vec2::new(end.x, end.y),
                Color::srgb(1.0, 0.5, 0.0), // Orange for in-progress
            );
        }
    }

    // Render drag preview for rectangle/ellipse
    if collision_editor.drawing {
        if let Some(_start) = collision_editor.drag_start {
            // This would need actual mouse position - simplified for now
            // gizmos.rect_2d(...)
        }
    }
}
