//! UI panel for the project file browser

use crate::icons::Icons;
use crate::project_browser::{FileEntry, FileType, ProjectBrowser};
use bevy::prelude::*;
use bevy_editor_frontend_api::ProjectBrowserPanelState;
use bevy_editor_scene::{EditorScene, SpriteTextureEvent};
use bevy_egui::egui;

/// Render the project browser panel
pub fn project_browser_panel_ui(
    ui: &mut egui::Ui,
    browser: &mut ProjectBrowser,
    panel: &mut ProjectBrowserPanelState,
    editor_scene: &EditorScene,
    asset_server: &AssetServer,
    texture_events: &mut bevy::ecs::message::MessageWriter<SpriteTextureEvent>,
) {
    ui.heading(format!("{} Project", Icons::FOLDER_OPEN));
    ui.separator();

    // Toolbar
    ui.horizontal(|ui| {
        if ui.button(format!("{} Refresh", Icons::REFRESH)).clicked() {
            browser.needs_refresh = true;
        }

        if ui.button(format!("{} New Folder", Icons::NEW)).clicked() {
            panel.show_new_folder_dialog = true;
            panel.new_folder_parent = browser
                .get_selected()
                .cloned()
                .or_else(|| browser.project_root.clone());
        }

        ui.label(format!(
            "{} items",
            count_total_items(&browser.root_entries)
        ));
    });

    ui.separator();

    // New folder dialog
    if panel.show_new_folder_dialog {
        egui::Window::new("Create New Folder")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label("Folder name:");
                ui.text_edit_singleline(&mut panel.new_folder_name);

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() && !panel.new_folder_name.is_empty() {
                        if let Some(ref parent) = panel.new_folder_parent {
                            match browser.create_folder(parent, &panel.new_folder_name) {
                                Ok(new_path) => {
                                    info!("Created folder: {:?}", new_path);
                                    panel.new_folder_name.clear();
                                    panel.show_new_folder_dialog = false;
                                }
                                Err(e) => {
                                    error!("Failed to create folder: {}", e);
                                }
                            }
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        panel.new_folder_name.clear();
                        panel.show_new_folder_dialog = false;
                    }
                });
            });
    }

    // File tree
    if browser.project_root.is_none() {
        ui.label("No project loaded");
    } else if browser.root_entries.is_empty() {
        ui.label("Empty project");
    } else {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let entries_clone = browser.root_entries.clone();
            render_file_tree(
                ui,
                browser,
                &entries_clone,
                0,
                editor_scene,
                asset_server,
                texture_events,
            );
        });
    }
}

/// Count total items recursively
fn count_total_items(entries: &[FileEntry]) -> usize {
    let mut count = entries.len();
    for entry in entries {
        if entry.is_directory {
            count += count_total_items(&entry.children);
        }
    }
    count
}

/// Render the file tree recursively
fn render_file_tree(
    ui: &mut egui::Ui,
    browser: &mut ProjectBrowser,
    entries: &[FileEntry],
    depth: usize,
    editor_scene: &EditorScene,
    asset_server: &AssetServer,
    texture_events: &mut bevy::ecs::message::MessageWriter<SpriteTextureEvent>,
) {
    for entry in entries {
        render_file_entry(
            ui,
            browser,
            entry,
            depth,
            editor_scene,
            asset_server,
            texture_events,
        );
    }
}

/// Render a single file or folder entry
fn render_file_entry(
    ui: &mut egui::Ui,
    browser: &mut ProjectBrowser,
    entry: &FileEntry,
    depth: usize,
    editor_scene: &EditorScene,
    asset_server: &AssetServer,
    texture_events: &mut bevy::ecs::message::MessageWriter<SpriteTextureEvent>,
) {
    let indent = depth as f32 * 16.0;
    let is_selected = browser
        .get_selected()
        .map(|p| p == &entry.path)
        .unwrap_or(false);

    ui.horizontal(|ui| {
        ui.add_space(indent);

        // Folder expand/collapse arrow
        if entry.is_directory {
            let is_expanded = browser.is_expanded(&entry.path);
            let arrow = if is_expanded {
                Icons::CHEVRON_DOWN
            } else {
                Icons::CHEVRON_RIGHT
            };

            if ui.small_button(arrow).clicked() {
                browser.toggle_folder(&entry.path);
            }
        } else {
            // Spacing for alignment with folders
            ui.add_space(20.0);
        }

        // Icon
        let icon = match entry.file_type {
            FileType::Folder => Icons::FOLDER,
            FileType::Scene => Icons::FILE,
            FileType::Image => Icons::IMAGE,
            FileType::Audio => Icons::AUDIO,
            FileType::Script => Icons::SCRIPT,
            FileType::Config => Icons::SETTINGS,
            FileType::Text => Icons::FILE,
            FileType::Tileset => Icons::TILEMAP,
            FileType::Unknown => Icons::FILE,
        };
        ui.label(icon);

        // Name (clickable)
        let name_text = if is_selected {
            egui::RichText::new(&entry.name)
                .color(egui::Color32::from_rgb(150, 200, 255))
                .strong()
        } else {
            egui::RichText::new(&entry.name)
        };

        let name_response = ui.selectable_label(is_selected, name_text);

        if name_response.clicked() {
            browser.select(&entry.path);
            info!("Selected: {:?}", entry.path);
        }

        // Double-click handler for images - assign to selected sprite
        if name_response.double_clicked() && entry.file_type == FileType::Image {
            if let Some(selected_entity) = editor_scene.selected_entity {
                // Load texture using absolute path since the AssetServer needs full path for user project assets
                let texture_path = entry.path.to_string_lossy().to_string().replace('\\', "/");

                info!("Loading texture from absolute path: '{}'", texture_path);
                let texture_handle: Handle<Image> = asset_server.load(&texture_path);

                // Send event to assign texture to sprite
                texture_events.write(SpriteTextureEvent {
                    entity: selected_entity,
                    texture_handle,
                });

                info!(
                    "Assigning texture '{}' to sprite {:?}",
                    entry.name, selected_entity
                );
            } else {
                warn!("Double-clicked image but no entity selected");
            }
        }

        // Context menu
        name_response.context_menu(|ui| {
            render_context_menu(ui, browser, entry);
        });

        // File size (for files only)
        if !entry.is_directory && entry.size > 0 {
            let size_kb = entry.size as f32 / 1024.0;
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{:.1} KB", size_kb))
                        .size(10.0)
                        .color(egui::Color32::from_rgb(120, 120, 120)),
                );
            });
        }
    });

    // Render children if folder is expanded
    if entry.is_directory && browser.is_expanded(&entry.path) && !entry.children.is_empty() {
        render_file_tree(
            ui,
            browser,
            &entry.children,
            depth + 1,
            editor_scene,
            asset_server,
            texture_events,
        );
    }
}

/// Render context menu for file/folder
fn render_context_menu(ui: &mut egui::Ui, browser: &mut ProjectBrowser, entry: &FileEntry) {
    if entry.is_directory {
        if ui.button(format!("{} New Folder", Icons::NEW)).clicked() {
            // TODO: Show new folder dialog with this as parent
            ui.close();
        }

        if ui
            .button(format!("{} Open in File Explorer", Icons::FOLDER_OPEN))
            .clicked()
        {
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("explorer")
                    .arg(&entry.path)
                    .spawn();
            }
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open").arg(&entry.path).spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("xdg-open")
                    .arg(&entry.path)
                    .spawn();
            }
            ui.close();
        }
    } else {
        // File-specific actions
        match entry.file_type {
            FileType::Scene => {
                if ui
                    .button(format!("{} Open Scene", Icons::FOLDER_OPEN))
                    .clicked()
                {
                    // TODO: Send event to load scene
                    info!("Open scene: {:?}", entry.path);
                    ui.close();
                }
            }
            FileType::Image => {
                if ui.button(format!("{} Preview", Icons::EYE)).clicked() {
                    // TODO: Show image preview
                    ui.close();
                }
            }
            _ => {}
        }

        if ui
            .button(format!("{} Show in Explorer", Icons::FOLDER_OPEN))
            .clicked()
        {
            #[cfg(target_os = "windows")]
            {
                if let Some(_parent) = entry.path.parent() {
                    let _ = std::process::Command::new("explorer")
                        .arg("/select,")
                        .arg(&entry.path)
                        .spawn();
                }
            }
            ui.close();
        }
    }

    ui.separator();

    // Danger zone
    ui.label(egui::RichText::new("Danger Zone").color(egui::Color32::from_rgb(255, 100, 100)));

    if ui.button(format!("{} Delete", Icons::CLOSE)).clicked() {
        match browser.delete(&entry.path) {
            Ok(()) => {
                info!("Deleted: {:?}", entry.path);
            }
            Err(e) => {
                error!("Failed to delete: {}", e);
            }
        }
        ui.close();
    }
}
