//! Assets Browser Module
//! Upload, browse, and manage sprites and other game assets.

use bevy_egui::egui;
use crate::editor_state::EditorState;

/// Render the assets browser module
pub fn render(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    // Render side panel first so it claims its space
    egui::SidePanel::left("assets_browser_panel")
            .default_width(300.0)
            .show_inside(ui, |ui| {
                ui.heading("Asset Browser");

                ui.horizontal(|ui| {
                    if ui.button("Upload").clicked() {
                        editor_state.assets.show_upload_dialog = true;
                    }
                    if ui.button("New Folder").clicked() {
                        editor_state.assets.show_new_folder_dialog = true;
                    }
                });

                ui.separator();

                // Search
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(&mut editor_state.assets.search_query);
                });

                // Type filter
                egui::ComboBox::from_label("Type")
                    .selected_text(editor_state.assets.type_filter.as_deref().unwrap_or("All"))
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(editor_state.assets.type_filter.is_none(), "All").clicked() {
                            editor_state.assets.type_filter = None;
                        }
                        for asset_type in ["Sprites", "Animations", "Tilesets", "UI", "Audio", "Particles"] {
                            if ui.selectable_label(editor_state.assets.type_filter.as_deref() == Some(asset_type), asset_type).clicked() {
                                editor_state.assets.type_filter = Some(asset_type.to_string());
                            }
                        }
                    });

                ui.separator();

                // Folder tree
                ui.collapsing("Folders", |ui| {
                    render_folder_tree(ui, editor_state);
                });

                ui.separator();

                // Asset grid/list
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if editor_state.assets.asset_list.is_empty() {
                        ui.label("No assets loaded");
                    } else {
                        // Grid view of assets
                        let available_width = ui.available_width();
                        let item_size = 80.0;
                        let items_per_row = (available_width / item_size).floor() as usize;

                        egui::Grid::new("assets_grid")
                            .num_columns(items_per_row.max(1))
                            .spacing([4.0, 4.0])
                            .show(ui, |ui| {
                                for (idx, asset) in editor_state.assets.asset_list.iter().enumerate() {
                                    let is_selected = editor_state.assets.selected_asset.as_ref() == Some(&asset.id);

                                    ui.vertical(|ui| {
                                        // Thumbnail placeholder
                                        let response = ui.allocate_response(
                                            egui::vec2(64.0, 64.0),
                                            egui::Sense::click()
                                        );

                                        let rect = response.rect;
                                        let visuals = if is_selected {
                                            ui.visuals().widgets.active
                                        } else if response.hovered() {
                                            ui.visuals().widgets.hovered
                                        } else {
                                            ui.visuals().widgets.inactive
                                        };

                                        ui.painter().rect_filled(rect, 4.0, visuals.bg_fill);
                                        ui.painter().rect_stroke(rect, 4.0, visuals.bg_stroke, egui::StrokeKind::Outside);

                                        // Asset type icon placeholder
                                        ui.painter().text(
                                            rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            &asset.asset_type.chars().next().unwrap_or('?').to_string(),
                                            egui::FontId::proportional(24.0),
                                            visuals.text_color(),
                                        );

                                        if response.clicked() {
                                            editor_state.assets.selected_asset = Some(asset.id.clone());
                                        }

                                        // Filename (truncated)
                                        let name = if asset.name.len() > 10 {
                                            format!("{}...", &asset.name[..10])
                                        } else {
                                            asset.name.clone()
                                        };
                                        ui.label(name);
                                    });

                                    if (idx + 1) % items_per_row.max(1) == 0 {
                                        ui.end_row();
                                    }
                                }
                            });
                    }
                });
            });

        // Right panel - asset details/preview
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(ref asset_id) = editor_state.assets.selected_asset {
                if let Some(asset) = editor_state.assets.asset_list.iter().find(|a| &a.id == asset_id) {
                    ui.heading(&asset.name);

                    ui.separator();

                    // Preview area
                    ui.group(|ui| {
                        ui.heading("Preview");

                        // Large preview placeholder
                        let preview_size = egui::vec2(256.0, 256.0);
                        let (rect, _response) = ui.allocate_exact_size(preview_size, egui::Sense::hover());

                        ui.painter().rect_filled(rect, 8.0, egui::Color32::from_gray(40));
                        ui.painter().rect_stroke(rect, 8.0, egui::Stroke::new(1.0, egui::Color32::from_gray(80)), egui::StrokeKind::Outside);
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "Preview",
                            egui::FontId::proportional(16.0),
                            egui::Color32::from_gray(128),
                        );
                    });

                    ui.separator();

                    // Asset info
                    ui.group(|ui| {
                        ui.heading("Info");
                        ui.label(format!("Type: {}", asset.asset_type));
                        ui.label(format!("Path: {}", asset.path));
                        ui.label(format!("Size: {} KB", asset.size_bytes / 1024));
                        if let Some(ref dims) = asset.dimensions {
                            ui.label(format!("Dimensions: {}x{}", dims.0, dims.1));
                        }
                    });

                    ui.separator();

                    // Sprite sheet settings (for sprites)
                    if asset.asset_type == "Sprite" || asset.asset_type == "Tileset" {
                        ui.group(|ui| {
                            ui.heading("Sprite Sheet Settings");
                            ui.label("Tile Width: (number)");
                            ui.label("Tile Height: (number)");
                            ui.label("Columns: (number)");
                            ui.label("Rows: (number)");
                            ui.label("Padding: (number)");
                        });

                        ui.separator();
                    }

                    // Animation settings (for animations)
                    if asset.asset_type == "Animation" {
                        ui.group(|ui| {
                            ui.heading("Animation Settings");
                            ui.label("Frame Count: (number)");
                            ui.label("Frame Duration: (ms)");
                            ui.label("Loop: (checkbox)");
                            if ui.button("Preview Animation").clicked() {
                                editor_state.status_message = "Playing animation preview...".to_string();
                            }
                        });

                        ui.separator();
                    }

                    // Usage info
                    ui.group(|ui| {
                        ui.heading("Used By");
                        ui.label("(List of entities using this asset)");
                        ui.label("- No references found");
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Replace").clicked() {
                            editor_state.status_message = "Opening file picker...".to_string();
                        }
                        if ui.button("Delete").clicked() {
                            editor_state.assets.show_delete_confirm = true;
                        }
                        if ui.button("Duplicate").clicked() {
                            editor_state.status_message = "Duplicating asset...".to_string();
                        }
                    });
                } else {
                    ui.label("Asset not found");
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select an asset from the browser or upload a new one");
                });
            }
        });

    // Upload dialog
    if editor_state.assets.show_upload_dialog {
        egui::Window::new("Upload Asset")
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.label("Asset Type:");
                egui::ComboBox::from_id_salt("upload_type")
                    .selected_text(editor_state.assets.upload_type.as_deref().unwrap_or("Select type"))
                    .show_ui(ui, |ui| {
                        for asset_type in ["Sprite", "Animation", "Tileset", "UI", "Audio", "Particle"] {
                            if ui.selectable_label(editor_state.assets.upload_type.as_deref() == Some(asset_type), asset_type).clicked() {
                                editor_state.assets.upload_type = Some(asset_type.to_string());
                            }
                        }
                    });

                ui.label("Target Folder:");
                ui.text_edit_singleline(&mut editor_state.assets.upload_folder);

                ui.separator();

                ui.label("Drag & drop files here or click to browse");

                let drop_area = ui.allocate_response(
                    egui::vec2(300.0, 100.0),
                    egui::Sense::click()
                );

                ui.painter().rect_stroke(
                    drop_area.rect,
                    8.0,
                    egui::Stroke::new(2.0, egui::Color32::from_gray(100)),
                    egui::StrokeKind::Outside,
                );

                ui.painter().text(
                    drop_area.rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Drop files here",
                    egui::FontId::proportional(14.0),
                    egui::Color32::from_gray(150),
                );

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Upload").clicked() {
                        editor_state.status_message = "Uploading asset...".to_string();
                        editor_state.assets.show_upload_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        editor_state.assets.show_upload_dialog = false;
                    }
                });
            });
    }

    // New folder dialog
    if editor_state.assets.show_new_folder_dialog {
        egui::Window::new("New Folder")
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.label("Folder Name:");
                ui.text_edit_singleline(&mut editor_state.assets.new_folder_name);

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        editor_state.status_message = format!("Creating folder: {}", editor_state.assets.new_folder_name);
                        editor_state.assets.new_folder_name.clear();
                        editor_state.assets.show_new_folder_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        editor_state.assets.new_folder_name.clear();
                        editor_state.assets.show_new_folder_dialog = false;
                    }
                });
            });
    }

    // Delete confirmation
    if editor_state.assets.show_delete_confirm {
        egui::Window::new("Confirm Delete")
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.label("Are you sure you want to delete this asset?");
                ui.label("This action cannot be undone.");

                ui.horizontal(|ui| {
                    if ui.button("Delete").clicked() {
                        editor_state.status_message = "Deleting asset...".to_string();
                        editor_state.assets.selected_asset = None;
                        editor_state.assets.show_delete_confirm = false;
                    }
                    if ui.button("Cancel").clicked() {
                        editor_state.assets.show_delete_confirm = false;
                    }
                });
            });
    }
}

/// Render the folder tree
fn render_folder_tree(ui: &mut egui::Ui, editor_state: &mut EditorState) {
    // Root folders
    let folders = vec![
        ("sprites", vec!["characters", "enemies", "npcs", "items"]),
        ("tilesets", vec!["terrain", "buildings", "decorations"]),
        ("animations", vec!["characters", "effects", "ui"]),
        ("ui", vec!["icons", "frames", "buttons"]),
        ("audio", vec!["music", "sfx", "ambient"]),
        ("particles", vec!["combat", "environment", "ui"]),
    ];

    for (folder, subfolders) in folders {
        let is_selected = editor_state.assets.current_folder.as_deref() == Some(folder);

        ui.collapsing(folder, |ui| {
            if ui.selectable_label(is_selected, "(all)").clicked() {
                editor_state.assets.current_folder = Some(folder.to_string());
            }

            for subfolder in subfolders {
                let path = format!("{}/{}", folder, subfolder);
                let is_sub_selected = editor_state.assets.current_folder.as_deref() == Some(&path);
                if ui.selectable_label(is_sub_selected, subfolder).clicked() {
                    editor_state.assets.current_folder = Some(path);
                }
            }
        });
    }
}
