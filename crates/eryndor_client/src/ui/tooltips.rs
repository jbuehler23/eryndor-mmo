//! Tooltip helper functions for abilities and items.

use bevy_egui::egui;

/// Show ability tooltip on hover
pub fn show_ability_tooltip(
    response: egui::Response,
    ability_id: u32,
    ability_db: &crate::ability_cache::ClientAbilityDatabase,
) -> egui::Response {
    if let Some(ability) = ability_db.get_ability_info(ability_id) {
        response.on_hover_ui(|ui| {
            ui.set_max_width(300.0);

            // Title
            ui.heading(&ability.name);
            ui.separator();

            // Description
            ui.label(&ability.description);
            ui.add_space(8.0);

            // Stats in color-coded format
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(100, 180, 255), "Mana:");
                ui.label(format!("{}", ability.mana_cost));
            });

            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(100, 180, 255), "Cooldown:");
                ui.label(format!("{:.1}s", ability.cooldown));
            });

            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(100, 180, 255), "Range:");
                ui.label(format!("{:.1}", ability.range));
            });

            if ability.damage_multiplier > 0.0 {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 100, 100), "Damage:");
                    ui.label(format!("{}x", ability.damage_multiplier));
                });
            }

            // Effect summary
            if !ability.effect_summary.is_empty() {
                ui.add_space(4.0);
                ui.colored_label(egui::Color32::from_rgb(255, 220, 100), &ability.effect_summary);
            }

            // Unlock requirement
            if let Some(level) = ability.unlock_level {
                ui.add_space(4.0);
                ui.colored_label(egui::Color32::GRAY, format!("Requires Level {}", level));
            }
        })
    } else {
        response
    }
}

/// Show item tooltip on hover
pub fn show_item_tooltip(
    response: egui::Response,
    item_id: u32,
    item_db: &crate::item_cache::ClientItemDatabase,
    is_equipped: bool,
) -> egui::Response {
    if let Some(item) = item_db.get_item_info(item_id) {
        response.on_hover_ui(|ui| {
            ui.set_max_width(250.0);

            // Title with equipped indicator
            let title = if is_equipped {
                format!("{} (Equipped)", item.name)
            } else {
                item.name.clone()
            };
            ui.heading(&title);
            ui.separator();

            // Item type
            ui.colored_label(egui::Color32::LIGHT_GRAY, format!("{:?}", item.item_type));
            ui.add_space(6.0);

            // Stats
            let bonuses = &item.stat_bonuses;
            if bonuses.attack_power > 0.0 {
                ui.colored_label(egui::Color32::from_rgb(255, 100, 100), format!("+{:.1} Attack Power", bonuses.attack_power));
            }
            if bonuses.defense > 0.0 {
                ui.colored_label(egui::Color32::from_rgb(100, 150, 255), format!("+{:.1} Defense", bonuses.defense));
            }
            if bonuses.max_health > 0.0 {
                ui.colored_label(egui::Color32::from_rgb(100, 255, 100), format!("+{:.0} Health", bonuses.max_health));
            }
            if bonuses.max_mana > 0.0 {
                ui.colored_label(egui::Color32::from_rgb(100, 180, 255), format!("+{:.0} Mana", bonuses.max_mana));
            }
            if bonuses.crit_chance > 0.0 {
                ui.colored_label(egui::Color32::from_rgb(255, 220, 100), format!("+{:.1}% Crit Chance", bonuses.crit_chance * 100.0));
            }

            // Action hint
            ui.add_space(6.0);
            ui.colored_label(egui::Color32::DARK_GRAY, "Right-click for options");
        })
    } else {
        response
    }
}
