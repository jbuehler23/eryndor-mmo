use bevy_egui::egui::{self, Color32, CornerRadius, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};
use uuid::Uuid;

use crate::schema::{DialogueChoice, DialogueNode, DialogueNodeType, DialogueTree};

/// State for the dialogue editor
#[derive(Default)]
pub struct DialogueEditorState {
    /// Instance being edited
    pub instance_id: Option<Uuid>,
    /// Property name being edited
    pub property_name: String,
    /// The dialogue tree being edited
    pub dialogue_tree: DialogueTree,
    /// Canvas pan offset
    pub pan_offset: Vec2,
    /// Selected node ID
    pub selected_node: Option<String>,
    /// Node being dragged
    pub dragging_node: Option<String>,
    /// Connection being created: (source_node_id, is_choice, choice_index)
    pub creating_connection: Option<(String, bool, usize)>,
    /// Zoom level
    pub zoom: f32,
    /// Show node creation menu at position
    pub show_create_menu: Option<Pos2>,
    /// Name input for new dialogue
    pub name_input: String,
}

impl DialogueEditorState {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            ..Default::default()
        }
    }

    /// Open the editor with existing dialogue data
    pub fn open(&mut self, instance_id: Uuid, property_name: String, dialogue: DialogueTree) {
        self.instance_id = Some(instance_id);
        self.property_name = property_name;
        self.dialogue_tree = dialogue;
        self.name_input = self.dialogue_tree.name.clone();
        self.selected_node = None;
        self.dragging_node = None;
        self.creating_connection = None;
        self.pan_offset = Vec2::ZERO;
        self.zoom = 1.0;
        self.show_create_menu = None;
    }

    /// Get the current dialogue tree
    pub fn get_dialogue_tree(&self) -> DialogueTree {
        self.dialogue_tree.clone()
    }
}

/// Result from dialogue editor rendering
#[derive(Default)]
pub struct DialogueEditorResult {
    /// Whether dialogue was modified
    pub changed: bool,
    /// Whether to close the editor
    pub close: bool,
}

const NODE_WIDTH: f32 = 200.0;
const NODE_HEADER_HEIGHT: f32 = 28.0;
const NODE_PADDING: f32 = 8.0;
const CONNECTION_RADIUS: f32 = 6.0;

/// Render the dialogue editor window
pub fn render_dialogue_editor(
    ctx: &egui::Context,
    state: &mut DialogueEditorState,
) -> DialogueEditorResult {
    let mut result = DialogueEditorResult::default();

    egui::Window::new("Dialogue Editor")
        .min_size([600.0, 400.0])
        .default_size([900.0, 700.0])
        .resizable(true)
        .collapsible(false)
        .scroll(false)
        .show(ctx, |ui| {
            // Top toolbar
            ui.horizontal(|ui| {
                ui.label("Name:");
                if ui.text_edit_singleline(&mut state.name_input).changed() {
                    state.dialogue_tree.name = state.name_input.clone();
                    result.changed = true;
                }

                ui.separator();

                if ui.button("Add Text Node").clicked() {
                    let node = DialogueNode::new_text("New text");
                    let pos = state.pan_offset * -1.0 + Vec2::new(100.0, 100.0);
                    add_node_at_position(state, node, pos);
                    result.changed = true;
                }

                if ui.button("Add Choice Node").clicked() {
                    let node = DialogueNode::new_choice();
                    let pos = state.pan_offset * -1.0 + Vec2::new(100.0, 100.0);
                    add_node_at_position(state, node, pos);
                    result.changed = true;
                }

                ui.separator();

                // Handle selected node actions
                let selected_id = state.selected_node.clone();
                if let Some(ref selected) = selected_id {
                    if ui.button("Delete Selected").clicked() {
                        state.dialogue_tree.remove_node(selected);
                        state.selected_node = None;
                        result.changed = true;
                    }

                    // Set as start node button
                    if state.dialogue_tree.start_node != *selected {
                        if ui.button("Set as Start").clicked() {
                            state.dialogue_tree.start_node = selected.clone();
                            result.changed = true;
                        }
                    } else {
                        ui.label("(Start Node)");
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        result.close = true;
                    }
                });
            });

            ui.separator();

            // Main content area: canvas on left, properties on right
            let available = ui.available_size();
            let properties_width = 250.0;
            let canvas_width = available.x - properties_width - 8.0;

            ui.horizontal(|ui| {
                // Canvas area
                let (canvas_response, painter) = ui.allocate_painter(
                    Vec2::new(canvas_width, available.y - 4.0),
                    Sense::click_and_drag(),
                );

                let canvas_rect = canvas_response.rect;

                // Handle canvas interactions
                handle_canvas_interaction(state, &canvas_response, &mut result);

                // Draw background grid
                draw_grid(&painter, canvas_rect, state.pan_offset);

                // Draw connections
                draw_connections(&painter, canvas_rect, state);

                // Draw nodes
                let node_changes = draw_nodes(ui, &painter, canvas_rect, state);
                if node_changes.changed {
                    result.changed = true;
                }
                if let Some(selected) = node_changes.selected {
                    state.selected_node = Some(selected);
                }
                if let Some(dragging) = node_changes.dragging {
                    state.dragging_node = Some(dragging);
                }
                if node_changes.stop_dragging {
                    state.dragging_node = None;
                }
                if let Some(conn) = node_changes.start_connection {
                    state.creating_connection = Some(conn);
                }
                if let Some((source_id, is_choice, choice_idx, target_id)) = node_changes.complete_connection {
                    if is_choice {
                        if let Some(source) = state.dialogue_tree.get_node_mut(&source_id) {
                            if let Some(choice) = source.choices.get_mut(choice_idx) {
                                choice.next_node = Some(target_id);
                                result.changed = true;
                            }
                        }
                    } else {
                        if let Some(source) = state.dialogue_tree.get_node_mut(&source_id) {
                            source.next_node = Some(target_id);
                            result.changed = true;
                        }
                    }
                    state.creating_connection = None;
                }

                // Draw connection being created
                if let Some((source_id, is_choice, choice_idx)) = &state.creating_connection {
                    if let Some(source_node) = state.dialogue_tree.get_node(source_id) {
                        let source_pos = node_output_pos(source_node, canvas_rect, state, *is_choice, *choice_idx);
                        if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                            painter.line_segment(
                                [source_pos, pointer_pos],
                                Stroke::new(2.0, Color32::YELLOW),
                            );
                        }
                    }
                }

                // Clear connection if released not on a target
                if ui.input(|i| i.pointer.any_released()) && state.creating_connection.is_some() && !node_changes.connection_dropped_on_target {
                    state.creating_connection = None;
                }

                ui.separator();

                // Properties panel
                ui.vertical(|ui| {
                    ui.set_min_width(properties_width - 8.0);
                    ui.heading("Properties");
                    ui.separator();

                    if let Some(selected_id) = &state.selected_node.clone() {
                        if let Some(node) = state.dialogue_tree.get_node_mut(selected_id) {
                            if render_node_properties(ui, node) {
                                result.changed = true;
                            }
                        }
                    } else {
                        ui.label("Select a node to edit its properties");
                    }
                });
            });
        });

    result
}

/// Add a node at a specific position
fn add_node_at_position(state: &mut DialogueEditorState, mut node: DialogueNode, pos: Vec2) {
    node.position = (pos.x, pos.y);
    let id = state.dialogue_tree.add_node(node);
    state.selected_node = Some(id);
}

/// Handle canvas pan and zoom
fn handle_canvas_interaction(
    state: &mut DialogueEditorState,
    response: &egui::Response,
    _result: &mut DialogueEditorResult,
) {
    // Pan with middle mouse or right drag
    if response.dragged_by(egui::PointerButton::Secondary) || response.dragged_by(egui::PointerButton::Middle) {
        state.pan_offset += response.drag_delta();
    }

    // Cancel connection creation on right click
    if response.secondary_clicked() {
        state.creating_connection = None;
    }
}

/// Draw background grid
fn draw_grid(painter: &egui::Painter, rect: Rect, offset: Vec2) {
    let grid_spacing = 50.0;
    let grid_color = Color32::from_gray(40);

    // Vertical lines
    let start_x = ((-offset.x) / grid_spacing).floor() as i32;
    let end_x = ((rect.width() - offset.x) / grid_spacing).ceil() as i32;

    for i in start_x..=end_x {
        let x = rect.left() + (i as f32 * grid_spacing) + offset.x;
        if x >= rect.left() && x <= rect.right() {
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, grid_color),
            );
        }
    }

    // Horizontal lines
    let start_y = ((-offset.y) / grid_spacing).floor() as i32;
    let end_y = ((rect.height() - offset.y) / grid_spacing).ceil() as i32;

    for i in start_y..=end_y {
        let y = rect.top() + (i as f32 * grid_spacing) + offset.y;
        if y >= rect.top() && y <= rect.bottom() {
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, grid_color),
            );
        }
    }
}

/// Draw connections between nodes
fn draw_connections(painter: &egui::Painter, canvas_rect: Rect, state: &DialogueEditorState) {
    let nodes: Vec<_> = state.dialogue_tree.nodes.values().cloned().collect();

    for node in &nodes {
        // Draw next_node connection
        if let Some(next_id) = &node.next_node {
            if let Some(target) = state.dialogue_tree.get_node(next_id) {
                let start = node_output_pos(node, canvas_rect, state, false, 0);
                let end = node_input_pos(target, canvas_rect, state);
                draw_bezier_connection(painter, start, end, Color32::WHITE);
            }
        }

        // Draw choice connections
        for (i, choice) in node.choices.iter().enumerate() {
            if let Some(next_id) = &choice.next_node {
                if let Some(target) = state.dialogue_tree.get_node(next_id) {
                    let start = node_output_pos(node, canvas_rect, state, true, i);
                    let end = node_input_pos(target, canvas_rect, state);
                    draw_bezier_connection(painter, start, end, Color32::from_rgb(255, 180, 100));
                }
            }
        }
    }
}

/// Draw a bezier curve connection
fn draw_bezier_connection(painter: &egui::Painter, start: Pos2, end: Pos2, color: Color32) {
    let dist = (end.x - start.x).abs().max(50.0);
    let ctrl1 = Pos2::new(start.x + dist * 0.5, start.y);
    let ctrl2 = Pos2::new(end.x - dist * 0.5, end.y);

    let points: Vec<Pos2> = (0..=20)
        .map(|i| {
            let t = i as f32 / 20.0;
            let t2 = t * t;
            let t3 = t2 * t;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;

            Pos2::new(
                mt3 * start.x + 3.0 * mt2 * t * ctrl1.x + 3.0 * mt * t2 * ctrl2.x + t3 * end.x,
                mt3 * start.y + 3.0 * mt2 * t * ctrl1.y + 3.0 * mt * t2 * ctrl2.y + t3 * end.y,
            )
        })
        .collect();

    for window in points.windows(2) {
        painter.line_segment([window[0], window[1]], Stroke::new(2.0, color));
    }

    // Arrow head
    let arrow_size = 8.0;
    let dir = (end - points[points.len() - 2]).normalized();
    let perp = Vec2::new(-dir.y, dir.x);
    let arrow_base = end - dir * arrow_size;

    painter.add(egui::Shape::convex_polygon(
        vec![
            end,
            arrow_base + perp * arrow_size * 0.5,
            arrow_base - perp * arrow_size * 0.5,
        ],
        color,
        Stroke::NONE,
    ));
}

/// Get the input connection position for a node
fn node_input_pos(node: &DialogueNode, canvas_rect: Rect, state: &DialogueEditorState) -> Pos2 {
    let node_rect = node_rect(node, canvas_rect, state);
    Pos2::new(node_rect.left(), node_rect.top() + NODE_HEADER_HEIGHT / 2.0)
}

/// Get the output connection position for a node
fn node_output_pos(node: &DialogueNode, canvas_rect: Rect, state: &DialogueEditorState, is_choice: bool, choice_index: usize) -> Pos2 {
    let node_rect = node_rect(node, canvas_rect, state);
    if is_choice {
        let y_offset = NODE_HEADER_HEIGHT + NODE_PADDING + (choice_index as f32 + 0.5) * 24.0;
        Pos2::new(node_rect.right(), node_rect.top() + y_offset)
    } else {
        Pos2::new(node_rect.right(), node_rect.top() + NODE_HEADER_HEIGHT / 2.0)
    }
}

/// Calculate the rect for a node
fn node_rect(node: &DialogueNode, canvas_rect: Rect, state: &DialogueEditorState) -> Rect {
    let base_height = NODE_HEADER_HEIGHT + NODE_PADDING * 2.0 + 60.0;
    let choice_height = node.choices.len() as f32 * 24.0;
    let height = base_height + choice_height;

    let pos = Pos2::new(
        canvas_rect.left() + node.position.0 + state.pan_offset.x,
        canvas_rect.top() + node.position.1 + state.pan_offset.y,
    );

    Rect::from_min_size(pos, Vec2::new(NODE_WIDTH, height))
}

/// Result from node drawing operations
#[derive(Default)]
struct NodeDrawResult {
    changed: bool,
    selected: Option<String>,
    dragging: Option<String>,
    stop_dragging: bool,
    start_connection: Option<(String, bool, usize)>,
    complete_connection: Option<(String, bool, usize, String)>,
    connection_dropped_on_target: bool,
}

/// Draw all nodes
fn draw_nodes(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    canvas_rect: Rect,
    state: &mut DialogueEditorState,
) -> NodeDrawResult {
    let mut result = NodeDrawResult::default();
    let node_ids: Vec<String> = state.dialogue_tree.nodes.keys().cloned().collect();

    for node_id in node_ids {
        let node = match state.dialogue_tree.get_node(&node_id) {
            Some(n) => n.clone(),
            None => continue,
        };

        let rect = node_rect(&node, canvas_rect, state);

        // Skip if outside canvas
        if !canvas_rect.intersects(rect) {
            continue;
        }

        let is_selected = state.selected_node.as_ref() == Some(&node_id);
        let is_start = state.dialogue_tree.start_node == node_id;

        // Node colors
        let (r, g, b) = node.node_type.color();
        let header_color = Color32::from_rgb(r, g, b);
        let body_color = Color32::from_gray(50);
        let border_color = if is_selected {
            Color32::YELLOW
        } else if is_start {
            Color32::GREEN
        } else {
            Color32::from_gray(80)
        };

        // Draw node body
        painter.rect_filled(rect, CornerRadius::same(4), body_color);

        // Draw header
        let header_rect = Rect::from_min_size(rect.min, Vec2::new(NODE_WIDTH, NODE_HEADER_HEIGHT));
        painter.rect_filled(
            header_rect,
            CornerRadius {
                nw: 4,
                ne: 4,
                sw: 0,
                se: 0,
            },
            header_color,
        );

        // Draw border
        painter.rect_stroke(rect, CornerRadius::same(4), Stroke::new(2.0, border_color), StrokeKind::Outside);

        // Draw header text
        let header_text = format!("{}: {}", node.node_type.display_name(), truncate_str(&node.speaker, 15));
        painter.text(
            header_rect.center(),
            egui::Align2::CENTER_CENTER,
            header_text,
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );

        // Draw text preview
        let text_rect = Rect::from_min_size(
            rect.min + Vec2::new(NODE_PADDING, NODE_HEADER_HEIGHT + NODE_PADDING),
            Vec2::new(NODE_WIDTH - NODE_PADDING * 2.0, 40.0),
        );
        painter.text(
            text_rect.left_top(),
            egui::Align2::LEFT_TOP,
            truncate_str(&node.text, 50),
            egui::FontId::proportional(11.0),
            Color32::LIGHT_GRAY,
        );

        // Draw input connector
        let input_pos = node_input_pos(&node, canvas_rect, state);
        painter.circle_filled(input_pos, CONNECTION_RADIUS, Color32::from_rgb(100, 200, 100));

        // Draw output connector
        let output_pos = node_output_pos(&node, canvas_rect, state, false, 0);
        painter.circle_filled(output_pos, CONNECTION_RADIUS, Color32::from_rgb(200, 100, 100));

        // Draw choice connectors
        for (i, choice) in node.choices.iter().enumerate() {
            let choice_y = rect.top() + NODE_HEADER_HEIGHT + NODE_PADDING + 50.0 + (i as f32) * 24.0;
            let choice_pos = Pos2::new(rect.right(), choice_y + 12.0);
            painter.circle_filled(choice_pos, CONNECTION_RADIUS * 0.8, Color32::from_rgb(255, 180, 100));

            // Draw choice text
            let choice_text_pos = Pos2::new(rect.left() + NODE_PADDING, choice_y);
            painter.text(
                choice_text_pos,
                egui::Align2::LEFT_TOP,
                format!("{}. {}", i + 1, truncate_str(&choice.text, 20)),
                egui::FontId::proportional(10.0),
                Color32::from_rgb(255, 200, 150),
            );
        }

        // Handle node interactions using ui.interact
        let node_response = ui.interact(rect, egui::Id::new(&node_id), Sense::click_and_drag());

        if node_response.clicked() {
            result.selected = Some(node_id.clone());
        }

        // Handle dragging
        if node_response.drag_started() {
            result.dragging = Some(node_id.clone());
        }

        if state.dragging_node.as_ref() == Some(&node_id) {
            if node_response.dragged() {
                let delta = node_response.drag_delta();
                if let Some(n) = state.dialogue_tree.get_node_mut(&node_id) {
                    n.position.0 += delta.x;
                    n.position.1 += delta.y;
                    result.changed = true;
                }
            }

            if node_response.drag_stopped() {
                result.stop_dragging = true;
            }
        }

        // Handle connection creation from output
        let output_rect = Rect::from_center_size(output_pos, Vec2::splat(CONNECTION_RADIUS * 2.5));
        let output_response = ui.interact(output_rect, egui::Id::new(format!("{}_out", node_id)), Sense::click_and_drag());

        if output_response.drag_started() {
            result.start_connection = Some((node_id.clone(), false, 0));
        }

        // Handle connection drop on input
        let input_rect = Rect::from_center_size(input_pos, Vec2::splat(CONNECTION_RADIUS * 2.5));
        let input_response = ui.interact(input_rect, egui::Id::new(format!("{}_in", node_id)), Sense::click());

        if input_response.hovered() && ui.input(|i| i.pointer.any_released()) {
            if let Some((source_id, is_choice, choice_idx)) = &state.creating_connection {
                if source_id != &node_id {
                    result.complete_connection = Some((source_id.clone(), *is_choice, *choice_idx, node_id.clone()));
                    result.connection_dropped_on_target = true;
                }
            }
        }

        // Handle choice output connections
        for (i, _) in node.choices.iter().enumerate() {
            let choice_out_pos = node_output_pos(&node, canvas_rect, state, true, i);
            let choice_out_rect = Rect::from_center_size(choice_out_pos, Vec2::splat(CONNECTION_RADIUS * 2.0));
            let choice_out_response = ui.interact(
                choice_out_rect,
                egui::Id::new(format!("{}_choice_{}", node_id, i)),
                Sense::click_and_drag(),
            );

            if choice_out_response.drag_started() {
                result.start_connection = Some((node_id.clone(), true, i));
            }
        }
    }

    result
}

/// Render node properties panel
fn render_node_properties(ui: &mut egui::Ui, node: &mut DialogueNode) -> bool {
    let mut changed = false;

    // Node type
    ui.horizontal(|ui| {
        ui.label("Type:");
        egui::ComboBox::from_id_salt("node_type")
            .selected_text(node.node_type.display_name())
            .show_ui(ui, |ui| {
                for t in DialogueNodeType::all() {
                    if ui.selectable_label(node.node_type == *t, t.display_name()).clicked() {
                        node.node_type = *t;
                        changed = true;
                    }
                }
            });
    });

    ui.separator();

    // Speaker
    ui.horizontal(|ui| {
        ui.label("Speaker:");
        if ui.text_edit_singleline(&mut node.speaker).changed() {
            changed = true;
        }
    });

    // Text
    ui.label("Text:");
    if ui
        .add(egui::TextEdit::multiline(&mut node.text).desired_rows(3).desired_width(f32::INFINITY))
        .changed()
    {
        changed = true;
    }

    ui.separator();

    // Condition (optional) - TODO: implement scripting language
    ui.collapsing("Condition", |ui| {
        let mut condition = node.condition.clone().unwrap_or_default();
        ui.label("Show this node if:");
        if ui.add(egui::TextEdit::singleline(&mut condition).hint_text("e.g. has_quest(\"quest_id\")")).changed() {
            node.condition = if condition.is_empty() { None } else { Some(condition) };
            changed = true;
        }
        ui.small("(Scripting not yet implemented)");
    });

    // Action (optional) - TODO: implement scripting language
    ui.collapsing("Action", |ui| {
        let mut action = node.action.clone().unwrap_or_default();
        ui.label("Execute when entering:");
        if ui.add(egui::TextEdit::singleline(&mut action).hint_text("e.g. give_item(\"item_id\")")).changed() {
            node.action = if action.is_empty() { None } else { Some(action) };
            changed = true;
        }
        ui.small("(Scripting not yet implemented)");
    });

    ui.separator();

    // Choices (for choice nodes)
    if node.node_type == DialogueNodeType::Choice || !node.choices.is_empty() {
        ui.heading("Choices");

        let mut to_remove = None;
        for (i, choice) in node.choices.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("{}.", i + 1));
                if ui.text_edit_singleline(&mut choice.text).changed() {
                    changed = true;
                }
                if ui.small_button("x").clicked() {
                    to_remove = Some(i);
                }
            });

            // Choice condition
            ui.horizontal(|ui| {
                ui.label("  If:");
                let mut cond = choice.condition.clone().unwrap_or_default();
                if ui.add(egui::TextEdit::singleline(&mut cond).desired_width(150.0)).changed() {
                    choice.condition = if cond.is_empty() { None } else { Some(cond) };
                    changed = true;
                }
            });
        }

        if let Some(idx) = to_remove {
            node.choices.remove(idx);
            changed = true;
        }

        if ui.button("+ Add Choice").clicked() {
            node.choices.push(DialogueChoice {
                text: "New choice".to_string(),
                next_node: None,
                condition: None,
            });
            changed = true;
        }
    }

    // Connection info
    ui.separator();
    ui.label("Connections:");

    let has_next = node.next_node.is_some();
    let next_display = node.next_node.as_ref().map(|n| truncate_str(n, 12)).unwrap_or_else(|| "(none)".to_string());

    ui.horizontal(|ui| {
        ui.label(format!("Next: {}", next_display));
        if has_next && ui.small_button("x").clicked() {
            node.next_node = None;
            changed = true;
        }
    });

    changed
}

/// Truncate a string to a maximum length
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
