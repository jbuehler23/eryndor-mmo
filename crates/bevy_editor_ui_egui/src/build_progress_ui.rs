use bevy::prelude::*;
use bevy_editor_project::{BuildProgress, ProjectSelection, ProjectSelectionState};
use bevy_egui::{egui, EguiContexts};

/// System to render build progress overlay
pub fn build_progress_overlay_ui(mut contexts: EguiContexts, selection: Res<ProjectSelection>) {
    let Some(ctx) = contexts.ctx_mut().ok() else {
        return;
    };

    // Only show overlay during build or template generation
    let show_overlay = matches!(
        selection.state,
        ProjectSelectionState::GeneratingTemplate | ProjectSelectionState::InitialBuild(_)
    );

    if !show_overlay {
        return;
    }

    // Fullscreen modal overlay
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::from_black_alpha(200)))
        .show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);

                    match &selection.state {
                        ProjectSelectionState::GeneratingTemplate => {
                            render_template_generation(ui);
                        }
                        ProjectSelectionState::InitialBuild(progress) => {
                            render_build_progress(ui, progress);
                        }
                        _ => {}
                    }
                });
            });
        });
}

fn render_template_generation(ui: &mut egui::Ui) {
    ui.heading(
        egui::RichText::new("dY>� Generating Project Template")
            .size(24.0)
            .color(egui::Color32::WHITE),
    );

    ui.add_space(20.0);

    ui.spinner();

    ui.add_space(10.0);

    ui.label(
        egui::RichText::new("Creating project files...")
            .size(16.0)
            .color(egui::Color32::LIGHT_GRAY),
    );
}

fn render_build_progress(ui: &mut egui::Ui, progress: &BuildProgress) {
    // Title
    ui.heading(
        egui::RichText::new("�sT Building Project Dependencies")
            .size(24.0)
            .color(egui::Color32::WHITE),
    );

    ui.add_space(10.0);

    // Subtitle explaining why this happens
    ui.label(
        egui::RichText::new("This only happens once per project")
            .size(14.0)
            .color(egui::Color32::from_rgb(180, 180, 180)),
    );

    ui.add_space(20.0);

    // Spinner
    ui.spinner();

    ui.add_space(15.0);

    // Current stage
    ui.label(
        egui::RichText::new(&progress.current_stage)
            .size(16.0)
            .color(egui::Color32::from_rgb(100, 200, 255)),
    );

    ui.add_space(10.0);

    // Time elapsed
    let elapsed = progress.elapsed_secs();
    let elapsed_mins = (elapsed / 60.0) as u32;
    let elapsed_secs = (elapsed % 60.0) as u32;

    ui.label(
        egui::RichText::new(format!("Time elapsed: {}m {}s", elapsed_mins, elapsed_secs))
            .size(14.0)
            .color(egui::Color32::LIGHT_GRAY),
    );

    ui.add_space(5.0);

    // Estimated time
    ui.label(
        egui::RichText::new("Estimated: 4-8 minutes (first build)")
            .size(12.0)
            .color(egui::Color32::from_rgb(150, 150, 150))
            .italics(),
    );

    ui.label(
        egui::RichText::new("Future builds: 10-30 seconds")
            .size(12.0)
            .color(egui::Color32::from_rgb(100, 200, 100))
            .italics(),
    );

    ui.add_space(20.0);

    // Build output (scrollable)
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(20, 20, 20))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)))
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.set_max_height(200.0);
            ui.set_max_width(600.0);

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if progress.output_lines.is_empty() {
                        ui.label(
                            egui::RichText::new("Waiting for build output...")
                                .color(egui::Color32::from_rgb(100, 100, 100))
                                .monospace(),
                        );
                    } else {
                        // Show last 20 lines
                        let start = progress.output_lines.len().saturating_sub(20);
                        for line in &progress.output_lines[start..] {
                            // Color code based on content
                            let color = if line.contains("Compiling") {
                                egui::Color32::from_rgb(100, 200, 255)
                            } else if line.contains("Finished") {
                                egui::Color32::from_rgb(100, 200, 100)
                            } else if line.contains("error") || line.contains("Error") {
                                egui::Color32::from_rgb(255, 100, 100)
                            } else {
                                egui::Color32::from_rgb(180, 180, 180)
                            };

                            ui.label(
                                egui::RichText::new(line)
                                    .color(color)
                                    .monospace()
                                    .size(11.0),
                            );
                        }
                    }
                });
        });

    ui.add_space(20.0);

    // Info box
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(30, 40, 50))
        .inner_margin(15.0)
        .corner_radius(5.0)
        .show(ui, |ui| {
            ui.set_max_width(500.0);
            ui.label(
                egui::RichText::new("dY'� Why does this take so long?")
                    .color(egui::Color32::from_rgb(100, 150, 255))
                    .size(13.0),
            );
            ui.add_space(5.0);
            ui.label(
                egui::RichText::new(
                    "The first build compiles all Bevy dependencies (graphics, physics, audio). \
                This creates a build cache, making future builds nearly instant.",
                )
                .color(egui::Color32::LIGHT_GRAY)
                .size(12.0),
            );
        });
}
