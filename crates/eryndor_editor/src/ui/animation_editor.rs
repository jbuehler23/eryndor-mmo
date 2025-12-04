//! Standalone Animation Editor for sprite sheets
//!
//! This module provides a full-screen animation editor that allows:
//! - Selecting a spritesheet image
//! - Configuring frame width/height
//! - Viewing a grid overlay on the spritesheet
//! - Selecting frames by clicking on the grid
//! - Creating named animations with custom frame sequences
//! - Previewing animations with playback controls

use bevy_egui::egui;
use uuid::Uuid;

use crate::schema::{AnimationDef, LoopMode, SpriteData};

/// State for the sprite/animation editor
#[derive(Default, Clone)]
pub struct SpriteEditorState {
    /// The data instance ID being edited
    pub instance_id: Option<Uuid>,
    /// Property name containing the SpriteData
    pub property_name: String,
    /// Copy of sprite data being edited
    pub sprite_data: SpriteData,
    /// Currently selected animation name (for editing)
    pub selected_animation: Option<String>,
    /// New animation name input
    pub new_animation_name: String,
    /// Currently selected frames (for building animation)
    pub selected_frames: Vec<usize>,
    /// Animation preview state
    pub preview_playing: bool,
    pub preview_frame: usize,
    pub preview_timer: f32,
    /// Scroll position for spritesheet view
    pub scroll_offset: egui::Vec2,
    /// Zoom level for spritesheet
    pub zoom: f32,
    /// Path input for loading new spritesheet
    pub sheet_path_input: String,
    /// Last loaded sheet path (to detect changes)
    pub loaded_sheet_path: Option<String>,
    /// Texture ID for the loaded spritesheet (set externally)
    pub spritesheet_texture_id: Option<egui::TextureId>,
    /// Size of the loaded spritesheet in pixels
    pub spritesheet_size: Option<(f32, f32)>,
}

impl SpriteEditorState {
    pub fn new() -> Self {
        Self {
            zoom: 2.0, // Start at 2x zoom for better visibility
            ..Default::default()
        }
    }

    /// Initialize the editor with sprite data from an instance
    pub fn open(&mut self, instance_id: Uuid, property_name: String, sprite_data: SpriteData) {
        self.instance_id = Some(instance_id);
        self.property_name = property_name;
        self.sprite_data = sprite_data.clone();
        self.sheet_path_input = sprite_data.sheet_path;
        self.selected_animation = None;
        self.selected_frames.clear();
        self.preview_playing = false;
        self.preview_frame = 0;
        self.zoom = 1.0;
        // Clear texture - it will be reloaded by the main system
        self.loaded_sheet_path = None;
        self.spritesheet_texture_id = None;
        self.spritesheet_size = None;
    }

    /// Get the edited sprite data
    pub fn get_sprite_data(&self) -> SpriteData {
        self.sprite_data.clone()
    }

    /// Check if the spritesheet needs to be (re)loaded
    pub fn needs_texture_load(&self) -> bool {
        let current_path = &self.sprite_data.sheet_path;
        !current_path.is_empty()
            && (self.loaded_sheet_path.as_ref() != Some(current_path)
                || self.spritesheet_texture_id.is_none())
    }

    /// Set the loaded texture info
    pub fn set_texture(&mut self, texture_id: egui::TextureId, width: f32, height: f32) {
        self.spritesheet_texture_id = Some(texture_id);
        self.spritesheet_size = Some((width, height));
        self.loaded_sheet_path = Some(self.sprite_data.sheet_path.clone());
    }

    /// Clear the loaded texture
    pub fn clear_texture(&mut self) {
        self.spritesheet_texture_id = None;
        self.spritesheet_size = None;
        self.loaded_sheet_path = None;
    }
}

/// Result from animation editor rendering
#[derive(Default)]
pub struct AnimationEditorResult {
    /// Whether sprite data was changed and should be saved
    pub changed: bool,
    /// Whether the editor should close
    pub close: bool,
    /// Whether to open file browser for spritesheet selection
    pub browse_spritesheet: bool,
    /// Whether the spritesheet path changed and needs reloading
    pub reload_spritesheet: bool,
}

/// Render the full animation editor window
pub fn render_animation_editor(
    ctx: &egui::Context,
    state: &mut SpriteEditorState,
) -> AnimationEditorResult {
    let mut result = AnimationEditorResult::default();

    egui::Window::new("Sprite & Animation Editor")
        .resizable(true)
        .default_size([1100.0, 800.0])
        .show(ctx, |ui| {
            // Top toolbar
            ui.horizontal(|ui| {
                if ui.button("Save & Close").clicked() {
                    result.changed = true;
                    result.close = true;
                }
                if ui.button("Cancel").clicked() {
                    result.close = true;
                }
                ui.separator();
                ui.label("Zoom:");
                if ui.button("-").clicked() {
                    state.zoom = (state.zoom - 0.25).max(0.25);
                }
                ui.label(format!("{:.0}%", state.zoom * 100.0));
                if ui.button("+").clicked() {
                    state.zoom = (state.zoom + 0.25).min(4.0);
                }
            });

            ui.separator();

            // Main content - left: spritesheet settings & animations, right: spritesheet grid
            ui.columns(2, |columns| {
                // Left column: Settings and animation list
                columns[0].vertical(|ui| {
                    render_spritesheet_settings(ui, state, &mut result);
                    ui.separator();
                    render_animation_list(ui, state, &mut result);
                    ui.separator();
                    render_animation_editor_panel(ui, state, &mut result);
                });

                // Right column: Spritesheet grid view
                columns[1].vertical(|ui| {
                    render_spritesheet_grid(ui, state, &mut result);
                });
            });
        });

    result
}

/// Render spritesheet settings (path, frame size, etc.)
fn render_spritesheet_settings(
    ui: &mut egui::Ui,
    state: &mut SpriteEditorState,
    result: &mut AnimationEditorResult,
) {
    ui.heading("Spritesheet Settings");

    // Sheet path with Browse button
    ui.horizontal(|ui| {
        ui.label("Sheet Path:");
    });
    ui.horizontal(|ui| {
        let text_response = ui.add(
            egui::TextEdit::singleline(&mut state.sheet_path_input)
                .desired_width(200.0)
                .hint_text("path/to/spritesheet.png"),
        );
        if text_response.changed() {
            state.sprite_data.sheet_path = state.sheet_path_input.clone();
            result.changed = true;
            result.reload_spritesheet = true;
        }

        // Browse button (native only via rfd)
        #[cfg(not(target_arch = "wasm32"))]
        if ui.button("Browse...").clicked() {
            result.browse_spritesheet = true;
        }

        // Reload button
        if ui.button("⟳").on_hover_text("Reload spritesheet").clicked() {
            state.clear_texture();
            result.reload_spritesheet = true;
        }
    });

    // Show spritesheet info if loaded
    if let Some((width, height)) = state.spritesheet_size {
        ui.label(format!("Image: {}x{} px", width as u32, height as u32));
    } else if !state.sprite_data.sheet_path.is_empty() {
        ui.colored_label(egui::Color32::YELLOW, "Image not loaded");
    }

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label("Frame Width:");
        let mut width = state.sprite_data.frame_width as i32;
        if ui.add(egui::DragValue::new(&mut width).range(1..=1024)).changed() {
            state.sprite_data.frame_width = width.max(1) as u32;
            result.changed = true;
        }

        ui.label("Height:");
        let mut height = state.sprite_data.frame_height as i32;
        if ui.add(egui::DragValue::new(&mut height).range(1..=1024)).changed() {
            state.sprite_data.frame_height = height.max(1) as u32;
            result.changed = true;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Columns:");
        let mut cols = state.sprite_data.columns as i32;
        if ui.add(egui::DragValue::new(&mut cols).range(1..=100)).changed() {
            state.sprite_data.columns = cols.max(1) as u32;
            result.changed = true;
        }

        ui.label("Rows:");
        let mut rows = state.sprite_data.rows as i32;
        if ui.add(egui::DragValue::new(&mut rows).range(1..=100)).changed() {
            state.sprite_data.rows = rows.max(1) as u32;
            result.changed = true;
        }
    });

    // Auto-detect button
    if state.spritesheet_size.is_some() {
        ui.horizontal(|ui| {
            if ui.button("Auto-detect grid").on_hover_text("Calculate columns/rows from image size and frame dimensions").clicked() {
                if let Some((img_width, img_height)) = state.spritesheet_size {
                    if state.sprite_data.frame_width > 0 && state.sprite_data.frame_height > 0 {
                        state.sprite_data.columns = (img_width as u32) / state.sprite_data.frame_width;
                        state.sprite_data.rows = (img_height as u32) / state.sprite_data.frame_height;
                        result.changed = true;
                    }
                }
            }
        });
    }

    ui.horizontal(|ui| {
        ui.label("Pivot X:");
        let mut px = state.sprite_data.pivot_x;
        if ui.add(egui::DragValue::new(&mut px).range(0.0..=1.0).speed(0.01)).changed() {
            state.sprite_data.pivot_x = px;
            result.changed = true;
        }

        ui.label("Y:");
        let mut py = state.sprite_data.pivot_y;
        if ui.add(egui::DragValue::new(&mut py).range(0.0..=1.0).speed(0.01)).changed() {
            state.sprite_data.pivot_y = py;
            result.changed = true;
        }
    });

    let total_frames = state.sprite_data.total_frames();
    ui.label(format!("Total frames: {}", total_frames));
}

/// Render the list of animations
fn render_animation_list(
    ui: &mut egui::Ui,
    state: &mut SpriteEditorState,
    result: &mut AnimationEditorResult,
) {
    ui.heading("Animations");

    // List existing animations
    let anim_names: Vec<String> = state.sprite_data.animations.keys().cloned().collect();
    let mut to_delete = None;

    for name in &anim_names {
        ui.horizontal(|ui| {
            let selected = state.selected_animation.as_ref() == Some(name);
            if ui.selectable_label(selected, name).clicked() {
                state.selected_animation = Some(name.clone());
                // Load frames into selection
                if let Some(anim) = state.sprite_data.animations.get(name) {
                    state.selected_frames = anim.frames.clone();
                }
            }

            if ui.small_button("x").clicked() {
                to_delete = Some(name.clone());
            }
        });
    }

    if let Some(name) = to_delete {
        state.sprite_data.animations.remove(&name);
        if state.selected_animation.as_ref() == Some(&name) {
            state.selected_animation = None;
            state.selected_frames.clear();
        }
        result.changed = true;
    }

    // Add new animation
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut state.new_animation_name);
        if ui.button("+ Add").clicked() && !state.new_animation_name.is_empty() {
            let name = state.new_animation_name.clone();
            state.sprite_data.animations.insert(
                name.clone(),
                AnimationDef::default(),
            );
            state.selected_animation = Some(name);
            state.selected_frames.clear();
            state.new_animation_name.clear();
            result.changed = true;
        }
    });
}

/// Render the animation editor panel for the selected animation
fn render_animation_editor_panel(
    ui: &mut egui::Ui,
    state: &mut SpriteEditorState,
    result: &mut AnimationEditorResult,
) {
    let Some(anim_name) = state.selected_animation.clone() else {
        ui.label("Select an animation to edit");
        return;
    };

    ui.heading(format!("Edit: {}", anim_name));

    // Frame duration
    if let Some(anim) = state.sprite_data.animations.get_mut(&anim_name) {
        ui.horizontal(|ui| {
            ui.label("Frame Duration (ms):");
            let mut duration = anim.frame_duration_ms as i32;
            if ui.add(egui::DragValue::new(&mut duration).range(16..=2000)).changed() {
                anim.frame_duration_ms = duration.max(16) as u32;
                result.changed = true;
            }
        });

        // Loop mode
        ui.horizontal(|ui| {
            ui.label("Loop Mode:");
            egui::ComboBox::from_id_salt("loop_mode")
                .selected_text(anim.loop_mode.display_name())
                .show_ui(ui, |ui| {
                    for mode in LoopMode::all() {
                        if ui.selectable_label(anim.loop_mode == *mode, mode.display_name()).clicked() {
                            anim.loop_mode = *mode;
                            result.changed = true;
                        }
                    }
                });
        });

        // Selected frames display
        ui.label(format!("Frames: {:?}", state.selected_frames));

        ui.horizontal(|ui| {
            if ui.button("Apply Frames").clicked() {
                anim.frames = state.selected_frames.clone();
                result.changed = true;
            }
            if ui.button("Clear Selection").clicked() {
                state.selected_frames.clear();
            }
        });

        // Preview controls
        ui.separator();
        ui.label("Preview:");
        ui.horizontal(|ui| {
            if state.preview_playing {
                if ui.button("Stop").clicked() {
                    state.preview_playing = false;
                }
            } else if ui.button("Play").clicked() {
                state.preview_playing = true;
                state.preview_frame = 0;
                state.preview_timer = 0.0;
            }

            if !anim.frames.is_empty() {
                let frame_idx = state.preview_frame % anim.frames.len();
                ui.label(format!("Frame: {} / {}", frame_idx + 1, anim.frames.len()));
            }
        });

        // Animation preview box
        if state.spritesheet_texture_id.is_some() && !anim.frames.is_empty() {
            render_animation_preview(ui, state);
        }
    }
}

/// Render animated preview of the current animation
fn render_animation_preview(ui: &mut egui::Ui, state: &SpriteEditorState) {
    let Some(texture_id) = state.spritesheet_texture_id else { return };
    let Some((img_width, img_height)) = state.spritesheet_size else { return };
    let Some(anim_name) = &state.selected_animation else { return };
    let Some(anim) = state.sprite_data.animations.get(anim_name) else { return };

    if anim.frames.is_empty() {
        return;
    }

    let frame_w = state.sprite_data.frame_width as f32;
    let frame_h = state.sprite_data.frame_height as f32;
    let cols = state.sprite_data.columns.max(1);

    // Get current frame
    let frame_idx = state.preview_frame % anim.frames.len();
    let frame_num = anim.frames[frame_idx];

    let col = frame_num as u32 % cols;
    let row = frame_num as u32 / cols;

    // Calculate UV coordinates
    let u0 = (col as f32 * frame_w) / img_width;
    let v0 = (row as f32 * frame_h) / img_height;
    let u1 = ((col + 1) as f32 * frame_w) / img_width;
    let v1 = ((row + 1) as f32 * frame_h) / img_height;

    ui.add_space(8.0);

    // Larger animation preview (up to 256px, or actual size if smaller)
    let preview_size = egui::vec2(frame_w.min(256.0), frame_h.min(256.0));
    let scale = (preview_size.x / frame_w).min(preview_size.y / frame_h).max(1.0);
    let display_size = egui::vec2(frame_w * scale, frame_h * scale);

    ui.add(
        egui::Image::new(egui::load::SizedTexture::new(texture_id, display_size))
            .uv(egui::Rect::from_min_max(
                egui::pos2(u0, v0),
                egui::pos2(u1, v1),
            ))
    );
}

/// Render the spritesheet grid view
fn render_spritesheet_grid(
    ui: &mut egui::Ui,
    state: &mut SpriteEditorState,
    _result: &mut AnimationEditorResult,
) {
    ui.heading("Spritesheet Grid");
    ui.label("Click frames to add to selection. Ctrl+click to toggle.");

    let frame_w = state.sprite_data.frame_width as f32 * state.zoom;
    let frame_h = state.sprite_data.frame_height as f32 * state.zoom;
    let cols = state.sprite_data.columns.max(1);
    let rows = state.sprite_data.rows.max(1);

    let total_w = frame_w * cols as f32;
    let total_h = frame_h * rows as f32;

    // Scroll area for the grid
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(
                egui::vec2(total_w, total_h),
                egui::Sense::click(),
            );

            let rect = response.rect;

            // Draw background
            painter.rect_filled(rect, 0.0, egui::Color32::from_gray(40));

            // Draw spritesheet image if loaded
            if let (Some(texture_id), Some((img_width, img_height))) =
                (state.spritesheet_texture_id, state.spritesheet_size)
            {
                // Calculate the portion of the image to show based on grid settings
                let grid_width = state.sprite_data.columns as f32 * state.sprite_data.frame_width as f32;
                let grid_height = state.sprite_data.rows as f32 * state.sprite_data.frame_height as f32;

                // UV coordinates for the grid portion
                let u_max = (grid_width / img_width).min(1.0);
                let v_max = (grid_height / img_height).min(1.0);

                let mesh = egui::Mesh {
                    texture_id,
                    indices: vec![0, 1, 2, 0, 2, 3],
                    vertices: vec![
                        egui::epaint::Vertex {
                            pos: rect.min,
                            uv: egui::pos2(0.0, 0.0),
                            color: egui::Color32::WHITE,
                        },
                        egui::epaint::Vertex {
                            pos: egui::pos2(rect.max.x, rect.min.y),
                            uv: egui::pos2(u_max, 0.0),
                            color: egui::Color32::WHITE,
                        },
                        egui::epaint::Vertex {
                            pos: rect.max,
                            uv: egui::pos2(u_max, v_max),
                            color: egui::Color32::WHITE,
                        },
                        egui::epaint::Vertex {
                            pos: egui::pos2(rect.min.x, rect.max.y),
                            uv: egui::pos2(0.0, v_max),
                            color: egui::Color32::WHITE,
                        },
                    ],
                };
                painter.add(egui::Shape::mesh(mesh));
            }

            // Draw frame grid overlay
            for row in 0..rows {
                for col in 0..cols {
                    let frame_idx = state.sprite_data.grid_to_frame(col, row);
                    let x = rect.min.x + col as f32 * frame_w;
                    let y = rect.min.y + row as f32 * frame_h;
                    let frame_rect = egui::Rect::from_min_size(
                        egui::pos2(x, y),
                        egui::vec2(frame_w, frame_h),
                    );

                    // Check if frame is selected - highlight with semi-transparent overlay
                    let is_selected = state.selected_frames.contains(&frame_idx);
                    if is_selected {
                        painter.rect_filled(
                            frame_rect,
                            0.0,
                            egui::Color32::from_rgba_unmultiplied(100, 150, 255, 100)
                        );
                    }

                    // Draw grid lines
                    painter.rect_stroke(
                        frame_rect,
                        0.0,
                        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(200, 200, 200, 150)),
                        egui::StrokeKind::Middle,
                    );

                    // Draw frame number
                    painter.text(
                        egui::pos2(x + 4.0, y + 4.0),
                        egui::Align2::LEFT_TOP,
                        format!("{}", frame_idx),
                        egui::FontId::proportional(10.0 * state.zoom.max(0.5)),
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 200),
                    );
                }
            }

            // Handle clicks on frames
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let local_x = pos.x - rect.min.x;
                    let local_y = pos.y - rect.min.y;

                    let col = (local_x / frame_w) as u32;
                    let row = (local_y / frame_h) as u32;

                    if col < cols && row < rows {
                        let frame_idx = state.sprite_data.grid_to_frame(col, row);

                        // Ctrl+click toggles, regular click adds
                        if ui.input(|i| i.modifiers.ctrl) {
                            if let Some(pos) = state.selected_frames.iter().position(|&f| f == frame_idx) {
                                state.selected_frames.remove(pos);
                            } else {
                                state.selected_frames.push(frame_idx);
                            }
                        } else {
                            state.selected_frames.push(frame_idx);
                        }
                    }
                }
            }
        });
}

/// Open a file dialog to select a spritesheet (native only)
#[cfg(not(target_arch = "wasm32"))]
pub fn open_spritesheet_dialog() -> Option<String> {
    use rfd::FileDialog;

    let file = FileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif"])
        .add_filter("All Files", &["*"])
        .set_title("Select Spritesheet Image")
        .pick_file();

    file.map(|p| p.to_string_lossy().to_string())
}

/// Stub for WASM - returns None since native file dialogs aren't available
#[cfg(target_arch = "wasm32")]
pub fn open_spritesheet_dialog() -> Option<String> {
    None
}
