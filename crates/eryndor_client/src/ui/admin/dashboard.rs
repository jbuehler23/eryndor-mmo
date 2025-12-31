//! Admin dashboard system menu and tab renderers.

use bevy::prelude::*;
use bevy_egui::egui;
use bevy_replicon::prelude::*;
use eryndor_shared::*;

use crate::ui::state::{SystemMenuState, SystemMenuTab};
use crate::ui::helpers::{format_timestamp, format_local_time};

/// System Menu window with tabs (some tabs are admin-only)
pub fn system_menu_window(
    ctx: &egui::Context,
    dashboard: &mut SystemMenuState,
    commands: &mut Commands,
    is_admin: bool,
) {
    egui::Window::new("System Menu")
        .default_width(800.0)
        .default_height(600.0)
        .show(ctx, |ui| {
            ui.heading("Server Information");
            ui.separator();

            // Tab selector (Ban and Audit tabs only visible to admins)
            ui.horizontal(|ui| {
                ui.selectable_value(&mut dashboard.active_tab, SystemMenuTab::Players, "Players");
                ui.selectable_value(&mut dashboard.active_tab, SystemMenuTab::Stats, "Stats");
                if is_admin {
                    ui.selectable_value(&mut dashboard.active_tab, SystemMenuTab::Bans, "Bans (Admin)");
                    ui.selectable_value(&mut dashboard.active_tab, SystemMenuTab::Logs, "Audit Logs (Admin)");
                }
            });

            ui.separator();

            // If non-admin is on an admin-only tab, switch to Players
            if !is_admin && (dashboard.active_tab == SystemMenuTab::Bans || dashboard.active_tab == SystemMenuTab::Logs) {
                dashboard.active_tab = SystemMenuTab::Players;
            }

            // Tab content
            match dashboard.active_tab {
                SystemMenuTab::Players => render_players_tab(ui, dashboard, commands),
                SystemMenuTab::Bans => {
                    if is_admin {
                        render_bans_tab(ui, dashboard, commands);
                    } else {
                        ui.label("This tab is only accessible to administrators.");
                    }
                }
                SystemMenuTab::Stats => render_stats_tab(ui, dashboard, commands),
                SystemMenuTab::Logs => {
                    if is_admin {
                        render_logs_tab(ui, dashboard, commands);
                    } else {
                        ui.label("This tab is only accessible to administrators.");
                    }
                }
            }
        });
}

/// Render the Players tab - shows online players and kick functionality
fn render_players_tab(
    ui: &mut egui::Ui,
    dashboard: &mut SystemMenuState,
    commands: &mut Commands,
) {
    ui.heading("Online Players");

    if ui.button("Refresh Player List").clicked() {
        commands.client_trigger(GetPlayerListRequest {});
    }

    ui.separator();

    if dashboard.player_list.is_empty() {
        ui.label("No players online or data not loaded yet.");
        ui.label("Click 'Refresh Player List' to fetch current data.");
    } else {
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                egui::Grid::new("players_grid")
                    .striped(true)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        // Header
                        ui.label("Username");
                        ui.label("Character");
                        ui.label("Level");
                        ui.label("Class");
                        ui.label("Position");
                        ui.label("Actions");
                        ui.end_row();

                        // Rows
                        for player in &dashboard.player_list {
                            ui.label(&player.username);
                            ui.label(&player.character_name);
                            ui.label(format!("{}", player.level));
                            ui.label(format!("{:?}", player.class));
                            ui.label(format!("({:.0}, {:.0})", player.position_x, player.position_y));

                            if ui.button("Kick").clicked() {
                                commands.client_trigger(AdminCommandRequest {
                                    command: format!("/kick {}", player.character_name),
                                });
                            }

                            ui.end_row();
                        }
                    });
            });
    }
}

/// Render the Bans tab - shows ban list and ban/unban functionality
fn render_bans_tab(
    ui: &mut egui::Ui,
    dashboard: &mut SystemMenuState,
    commands: &mut Commands,
) {
    ui.heading("Ban Management");

    if ui.button("Refresh Ban List").clicked() {
        commands.client_trigger(GetBanListRequest {});
    }

    ui.separator();

    // Create ban form
    ui.collapsing("Create New Ban", |ui| {
        ui.horizontal(|ui| {
            ui.label("Username:");
            ui.text_edit_singleline(&mut dashboard.ban_form_username);
        });

        ui.horizontal(|ui| {
            ui.label("Duration (hours, 0 = permanent):");
            ui.add(egui::DragValue::new(&mut dashboard.ban_form_duration).speed(1.0));
        });

        ui.horizontal(|ui| {
            ui.label("Reason:");
            ui.text_edit_singleline(&mut dashboard.ban_form_reason);
        });

        if ui.button("Create Ban").clicked() {
            let duration_str = if dashboard.ban_form_duration == 0 {
                "permanent".to_string()
            } else {
                format!("{}h", dashboard.ban_form_duration)
            };

            commands.client_trigger(AdminCommandRequest {
                command: format!("/ban {} {} {}",
                    dashboard.ban_form_username,
                    duration_str,
                    dashboard.ban_form_reason
                ),
            });

            dashboard.ban_form_username.clear();
            dashboard.ban_form_duration = 0;
            dashboard.ban_form_reason.clear();
        }
    });

    ui.separator();
    ui.heading("Active Bans");

    if dashboard.ban_list.is_empty() {
        ui.label("No active bans or data not loaded yet.");
        ui.label("Click 'Refresh Ban List' to fetch current data.");
    } else {
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                egui::Grid::new("bans_grid")
                    .striped(true)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        // Header
                        ui.label("ID");
                        ui.label("Type");
                        ui.label("Target");
                        ui.label("Reason");
                        ui.label("Expires");
                        ui.label("Actions");
                        ui.end_row();

                        // Rows
                        for ban in &dashboard.ban_list {
                            ui.label(format!("{}", ban.id));
                            ui.label(&ban.ban_type);
                            ui.label(&ban.target);
                            ui.label(&ban.reason);

                            if let Some(expires) = ban.expires_at {
                                ui.label(format!("Expires: {}", expires));
                            } else {
                                ui.label("Permanent");
                            }

                            if ui.button("Unban").clicked() {
                                commands.client_trigger(AdminCommandRequest {
                                    command: format!("/unban {}", ban.target),
                                });
                            }

                            ui.end_row();
                        }
                    });
            });
    }
}

/// Render the Stats tab - shows server statistics
fn render_stats_tab(
    ui: &mut egui::Ui,
    dashboard: &mut SystemMenuState,
    commands: &mut Commands,
) {
    ui.heading("Server Statistics");

    if ui.button("Refresh Stats").clicked() {
        commands.client_trigger(GetServerStatsRequest {});
    }

    ui.separator();

    if let Some(stats) = &dashboard.server_stats {
        egui::Grid::new("stats_grid")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Online Players:");
                ui.label(format!("{}", stats.online_players));
                ui.end_row();

                ui.label("Total Accounts:");
                ui.label(format!("{}", stats.total_accounts));
                ui.end_row();

                ui.label("Total Characters:");
                ui.label(format!("{}", stats.total_characters));
                ui.end_row();

                ui.label("Active Bans:");
                ui.label(format!("{}", stats.active_bans));
                ui.end_row();

                ui.label("Server Time (UTC):");
                let server_time = format_timestamp(stats.server_time_utc);
                ui.label(server_time);
                ui.end_row();

                ui.label("Local Time:");
                let local_time = format_local_time();
                ui.label(local_time);
                ui.end_row();
            });
    } else {
        ui.label("No stats data loaded yet.");
        ui.label("Click 'Refresh Stats' to fetch current data.");
    }
}

/// Render the Logs tab - shows audit logs with pagination
fn render_logs_tab(
    ui: &mut egui::Ui,
    dashboard: &mut SystemMenuState,
    commands: &mut Commands,
) {
    ui.heading("Audit Logs");

    // Pagination controls
    ui.horizontal(|ui| {
        if ui.button("Previous Page").clicked() && dashboard.audit_logs_offset >= dashboard.audit_logs_limit {
            dashboard.audit_logs_offset -= dashboard.audit_logs_limit;
            commands.client_trigger(GetAuditLogsRequest {
                limit: dashboard.audit_logs_limit,
                offset: dashboard.audit_logs_offset,
            });
        }

        ui.label(format!("Page {} | Total: {}",
            (dashboard.audit_logs_offset / dashboard.audit_logs_limit) + 1,
            dashboard.audit_logs_total
        ));

        if ui.button("Next Page").clicked() {
            let max_offset = dashboard.audit_logs_total.saturating_sub(dashboard.audit_logs_limit);
            if dashboard.audit_logs_offset < max_offset {
                dashboard.audit_logs_offset += dashboard.audit_logs_limit;
                commands.client_trigger(GetAuditLogsRequest {
                    limit: dashboard.audit_logs_limit,
                    offset: dashboard.audit_logs_offset,
                });
            }
        }

        if ui.button("Refresh").clicked() {
            commands.client_trigger(GetAuditLogsRequest {
                limit: dashboard.audit_logs_limit,
                offset: dashboard.audit_logs_offset,
            });
        }
    });

    ui.separator();

    if dashboard.audit_logs.is_empty() {
        ui.label("No audit logs or data not loaded yet.");
        ui.label("Click 'Refresh' to fetch current data.");
    } else {
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                egui::Grid::new("logs_grid")
                    .striped(true)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        // Header
                        ui.label("ID");
                        ui.label("Action");
                        ui.label("Account");
                        ui.label("Target");
                        ui.label("Details");
                        ui.label("Timestamp");
                        ui.end_row();

                        // Rows
                        for log in &dashboard.audit_logs {
                            ui.label(format!("{}", log.id));
                            ui.label(&log.action_type);

                            if let Some(account_id) = log.account_id {
                                ui.label(format!("{}", account_id));
                            } else {
                                ui.label("-");
                            }

                            if let Some(target) = &log.target_account {
                                ui.label(target);
                            } else {
                                ui.label("-");
                            }

                            if let Some(details) = &log.details {
                                ui.label(details);
                            } else {
                                ui.label("-");
                            }

                            ui.label(format!("{}", log.timestamp));
                            ui.end_row();
                        }
                    });
            });
    }
}
