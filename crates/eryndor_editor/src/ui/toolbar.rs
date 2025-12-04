use bevy_egui::egui;

use crate::EditorState;

pub fn render_toolbar(ctx: &egui::Context, editor_state: &mut EditorState) {
    egui::TopBottomPanel::top("toolbar")
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Tool selection - grouped by category
                ui.label("Tools:");

                // Selection tools
                if ui.selectable_label(editor_state.current_tool == EditorTool::Select, "🔍 Select").clicked() {
                    editor_state.current_tool = EditorTool::Select;
                }

                ui.separator();

                // Painting tools
                let paint_tools = [
                    (EditorTool::Paint, "🖌 Paint"),
                    (EditorTool::Erase, "🧹 Erase"),
                    (EditorTool::Fill, "🪣 Fill"),
                ];

                for (tool, name) in paint_tools {
                    if ui.selectable_label(editor_state.current_tool == tool, name).clicked() {
                        editor_state.current_tool = tool;
                    }
                }

                ui.separator();

                // Terrain tool (for autotiling)
                if ui.selectable_label(editor_state.current_tool == EditorTool::Terrain, "🏔 Terrain").clicked() {
                    editor_state.current_tool = EditorTool::Terrain;
                }

                ui.separator();

                // Entity tool
                if ui.selectable_label(editor_state.current_tool == EditorTool::Entity, "📍 Entity").clicked() {
                    editor_state.current_tool = EditorTool::Entity;
                }

                ui.separator();

                // Tool mode dropdown (for applicable tools)
                if editor_state.current_tool.supports_modes() {
                    ui.label("Mode:");
                    egui::ComboBox::from_id_salt("tool_mode")
                        .selected_text(editor_state.tool_mode.label())
                        .width(80.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut editor_state.tool_mode, ToolMode::Point, ToolMode::Point.label());
                            ui.selectable_value(&mut editor_state.tool_mode, ToolMode::Rectangle, ToolMode::Rectangle.label());
                        });

                    ui.separator();
                }

                // Layer selection
                ui.label("Layer:");
                if let Some(layer_idx) = editor_state.selected_layer {
                    ui.label(format!("{}", layer_idx));
                } else {
                    ui.label("(none)");
                }

                ui.separator();

                // Grid toggle
                ui.checkbox(&mut editor_state.show_grid, "Grid");

                ui.separator();

                // Zoom controls
                if ui.button("-").clicked() {
                    editor_state.zoom = (editor_state.zoom / 1.25).max(0.25);
                }
                ui.label(format!("{}%", (editor_state.zoom * 100.0) as i32));
                if ui.button("+").clicked() {
                    editor_state.zoom = (editor_state.zoom * 1.25).min(4.0);
                }
            });
        });
}

/// Tool mode for painting operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolMode {
    /// Single tile/point painting (click or drag)
    #[default]
    Point,
    /// Rectangle fill (drag to define area)
    Rectangle,
}

impl ToolMode {
    pub fn label(&self) -> &'static str {
        match self {
            ToolMode::Point => "▪ Point",
            ToolMode::Rectangle => "▢ Rect",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorTool {
    #[default]
    Select,
    Paint,
    Erase,
    Fill,
    Terrain,
    Entity,
}

impl EditorTool {
    /// Returns true if this tool supports Point/Rectangle modes
    pub fn supports_modes(&self) -> bool {
        matches!(self, EditorTool::Paint | EditorTool::Erase | EditorTool::Terrain)
    }
}
