//! Main game UI systems and windows.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_replicon::prelude::*;
use eryndor_shared::*;

use crate::game_state::MyClientState;
use crate::ui::state::{UiState, TrainerTab, QuestDialogueData, TrainerWindowData, LootWindowData};
use crate::ui::tooltips::{show_ability_tooltip, show_item_tooltip};
use crate::ui::admin::system_menu_window;

/// Main game UI system - renders all in-game windows
pub fn game_ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut client_state: ResMut<MyClientState>,
    mut commands: Commands,
    player_query: Query<(Entity, &Health, &Mana, &CurrentTarget, &Hotbar, &Inventory, &Equipment, &CombatStats, &LearnedAbilities, &QuestLog, &Character, &Gold, &Position), With<Player>>,
    progression_query: Query<(&Experience, &WeaponProficiency, &WeaponProficiencyExp, &ArmorProficiency)>,
    buffs_query: Query<Option<&ActiveBuffs>>,
    target_query: Query<(&Health, Option<&Character>, Option<&NpcName>)>,
    loot_query: Query<(Entity, &Position), With<LootContainer>>,
    item_db: Res<crate::item_cache::ClientItemDatabase>,
    ability_db: Res<crate::ability_cache::ClientAbilityDatabase>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let Some(player_entity) = client_state.player_entity else {
        return
    };

    let Ok((_, health, mana, current_target, hotbar, inventory, equipment, combat_stats, _learned_abilities, quest_log, character, gold, player_pos)) = player_query.get(player_entity) else {
        return
    };

    let Ok((experience, weapon_prof, weapon_exp, armor_prof)) = progression_query.get(player_entity) else {
        return
    };

    // Player status window
    render_player_status(ctx, health, mana, gold);

    // Active buffs display
    if let Ok(Some(active_buffs)) = buffs_query.get(player_entity) {
        render_buffs(ctx, active_buffs, &ability_db);
    }

    // Target frame
    render_target_frame(ctx, current_target, &target_query);

    // Hotbar
    render_hotbar(ctx, hotbar, &ability_db, &mut commands);

    // Action buttons
    render_action_buttons(ctx, &mut ui_state);

    // Loot hint
    render_loot_hint(ctx, player_pos, &loot_query);

    // Equipment window
    if ui_state.show_equipment {
        render_equipment_window(ctx, equipment, &item_db, &mut commands);
    }

    // Character stats window
    if ui_state.show_character_stats {
        render_character_stats(ctx, character, experience, health, mana, combat_stats, equipment, weapon_prof, weapon_exp, armor_prof, &item_db);
    }

    // Inventory window
    if ui_state.show_inventory {
        render_inventory_window(ctx, inventory, equipment, &item_db, &mut commands);
    }

    // Quest log
    render_quest_log(ctx, quest_log, &mut commands);

    // Notifications
    render_notifications(ctx, &mut client_state);

    // Controls help
    render_controls_help(ctx);

    // ESC Menu
    if ui_state.show_esc_menu {
        render_esc_menu(ctx, &mut ui_state, &mut commands);
    }

    // Quest Dialogue Window
    if let Some(dialogue) = ui_state.quest_dialogue.clone() {
        render_quest_dialogue(ctx, &dialogue, &mut ui_state, &mut commands);
    }

    // Trainer Window
    if let Some(trainer_data) = ui_state.trainer_window.clone() {
        render_trainer_window(ctx, trainer_data, &mut ui_state, gold, &item_db, &mut commands);
    }

    // Loot Container Window
    if let Some(loot_data) = ui_state.loot_window.clone() {
        render_loot_window(ctx, loot_data, &mut ui_state, player_pos, &loot_query, &item_db, &mut commands);
    }

    // System Menu
    if ui_state.show_system_menu {
        let is_admin = ui_state.is_admin;
        system_menu_window(ctx, &mut ui_state.system_menu, &mut commands, is_admin);
    }
}

fn render_player_status(ctx: &egui::Context, health: &Health, mana: &Mana, gold: &Gold) {
    egui::Window::new("Player Status")
        .fixed_pos([10.0, 10.0])
        .fixed_size([200.0, 100.0])
        .title_bar(false)
        .show(ctx, |ui| {
            ui.label("Health:");
            ui.add(egui::ProgressBar::new(health.percent()).text(format!("{:.0}/{:.0}", health.current, health.max)));

            ui.label("Mana:");
            ui.add(egui::ProgressBar::new(mana.percent()).text(format!("{:.0}/{:.0}", mana.current, mana.max)));

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Gold:");
                ui.colored_label(egui::Color32::GOLD, format!("{}", gold.0));
            });
        });
}

fn render_buffs(ctx: &egui::Context, active_buffs: &ActiveBuffs, ability_db: &crate::ability_cache::ClientAbilityDatabase) {
    if active_buffs.buffs.is_empty() {
        return;
    }

    egui::Window::new("Buffs")
        .fixed_pos([10.0, 115.0])
        .fixed_size([200.0, 60.0])
        .title_bar(false)
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                for buff in &active_buffs.buffs {
                    let ability_name = ability_db.get_ability_name(buff.ability_id);
                    let response = ui.colored_label(egui::Color32::from_rgb(100, 200, 255), &ability_name);
                    response.on_hover_ui(|ui| {
                        ui.label(&ability_name);
                        ui.separator();
                        if buff.stat_bonuses.attack_power > 0.0 {
                            ui.label(format!("+{:.0} Attack", buff.stat_bonuses.attack_power));
                        }
                        if buff.stat_bonuses.defense > 0.0 {
                            ui.label(format!("+{:.0} Defense", buff.stat_bonuses.defense));
                        }
                        if buff.stat_bonuses.move_speed > 0.0 {
                            ui.label(format!("+{:.0}% Speed", buff.stat_bonuses.move_speed * 100.0));
                        }
                    });
                    ui.add_space(5.0);
                }
            });
        });
}

fn render_target_frame(ctx: &egui::Context, current_target: &CurrentTarget, target_query: &Query<(&Health, Option<&Character>, Option<&NpcName>)>) {
    let Some(target_entity) = current_target.0 else { return };

    let Ok((target_health, target_char, target_npc)) = target_query.get(target_entity) else { return };

    let target_name = if let Some(character) = target_char {
        character.name.clone()
    } else if let Some(npc) = target_npc {
        npc.0.clone()
    } else {
        "Enemy".to_string()
    };

    egui::Window::new("Target")
        .fixed_pos([540.0, 10.0])
        .fixed_size([200.0, 60.0])
        .title_bar(false)
        .show(ctx, |ui| {
            ui.label(&target_name);
            ui.add(egui::ProgressBar::new(target_health.percent()).text(format!("{:.0}/{:.0}", target_health.current, target_health.max)));
        });
}

fn render_hotbar(ctx: &egui::Context, hotbar: &Hotbar, ability_db: &crate::ability_cache::ClientAbilityDatabase, commands: &mut Commands) {
    egui::Window::new("Hotbar")
        .fixed_pos([440.0, 630.0])
        .fixed_size([400.0, 70.0])
        .title_bar(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, slot) in hotbar.slots.iter().enumerate() {
                    let button_text = if let Some(HotbarSlot::Ability(ability_id)) = slot {
                        let ability_name = ability_db.get_ability_name(*ability_id);
                        format!("{}\n[{}]", ability_name, i + 1)
                    } else {
                        format!("[{}]", i + 1)
                    };

                    let mut response = ui.button(button_text);

                    if let Some(HotbarSlot::Ability(ability_id)) = slot {
                        response = show_ability_tooltip(response, *ability_id, ability_db);
                    }

                    if response.clicked() {
                        if let Some(HotbarSlot::Ability(ability_id)) = slot {
                            commands.client_trigger(UseAbilityRequest {
                                ability_id: *ability_id,
                                target_position: None,
                            });
                        }
                    }
                }
            });
        });
}

fn render_action_buttons(ctx: &egui::Context, ui_state: &mut UiState) {
    let button_count = if ui_state.is_admin { 4 } else { 3 };
    egui::Window::new("Actions")
        .fixed_pos([1090.0, 10.0])
        .fixed_size([180.0, 30.0 * button_count as f32 + 20.0])
        .title_bar(false)
        .show(ctx, |ui| {
            if ui.button("Equipment").clicked() {
                ui_state.show_equipment = !ui_state.show_equipment;
            }
            if ui.button("Character").clicked() {
                ui_state.show_character_stats = !ui_state.show_character_stats;
            }
            if ui.button("Inventory").clicked() {
                ui_state.show_inventory = !ui_state.show_inventory;
            }
            if ui.button("System Menu").clicked() {
                ui_state.show_system_menu = !ui_state.show_system_menu;
            }
        });
}

fn render_loot_hint(ctx: &egui::Context, player_pos: &Position, loot_query: &Query<(Entity, &Position), With<LootContainer>>) {
    let mut nearest_loot_distance = f32::MAX;
    for (_, loot_pos) in loot_query {
        let distance = player_pos.0.distance(loot_pos.0);
        if distance < nearest_loot_distance {
            nearest_loot_distance = distance;
        }
    }

    if nearest_loot_distance <= PICKUP_RANGE {
        egui::Window::new("Loot Hint")
            .fixed_pos([540.0, 600.0])
            .fixed_size([200.0, 40.0])
            .title_bar(false)
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180)))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.colored_label(egui::Color32::YELLOW, "Press E to Loot");
                });
            });
    }
}

fn render_equipment_window(ctx: &egui::Context, equipment: &Equipment, item_db: &crate::item_cache::ClientItemDatabase, commands: &mut Commands) {
    egui::Window::new("Equipment")
        .collapsible(false)
        .resizable(false)
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.heading("Equipment Slots");
            ui.separator();

            let mut show_slot = |ui: &mut egui::Ui, label: &str, item_id: Option<u32>, slot: EquipmentSlot| {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", label));
                    let mut response = if let Some(id) = item_id {
                        let item_name = item_db.get_item_name(id);
                        ui.label(&item_name)
                    } else {
                        ui.label("<Empty>")
                    };

                    if let Some(id) = item_id {
                        response = show_item_tooltip(response, id, item_db, true);
                    }

                    if item_id.is_some() {
                        response.context_menu(|ui| {
                            if ui.button("Unequip").clicked() {
                                commands.client_trigger(UnequipItemRequest { slot });
                                ui.close();
                            }
                        });
                    }
                });
            };

            show_slot(ui, "Weapon", equipment.weapon, EquipmentSlot::Weapon);
            show_slot(ui, "Helmet", equipment.helmet, EquipmentSlot::Helmet);
            show_slot(ui, "Chest", equipment.chest, EquipmentSlot::Chest);
            show_slot(ui, "Legs", equipment.legs, EquipmentSlot::Legs);
            show_slot(ui, "Boots", equipment.boots, EquipmentSlot::Boots);

            ui.add_space(10.0);
            ui.label("Right-click equipped items to unequip them.");
        });
}

fn render_character_stats(
    ctx: &egui::Context,
    character: &Character,
    experience: &Experience,
    health: &Health,
    mana: &Mana,
    combat_stats: &CombatStats,
    equipment: &Equipment,
    weapon_prof: &WeaponProficiency,
    weapon_exp: &WeaponProficiencyExp,
    armor_prof: &ArmorProficiency,
    item_db: &crate::item_cache::ClientItemDatabase,
) {
    egui::Window::new("Character Stats")
        .collapsible(false)
        .resizable(false)
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading(&character.name);
            ui.label(format!("Class: {} | Level: {}", character.class.as_str(), character.level));

            ui.add_space(5.0);
            let xp_percent = experience.current_xp as f32 / experience.xp_to_next_level as f32;
            ui.label("Experience:");
            ui.add(egui::ProgressBar::new(xp_percent)
                .text(format!("{} / {} XP", experience.current_xp, experience.xp_to_next_level)));

            ui.separator();

            let equipment_bonuses = item_db.calculate_equipment_bonuses(equipment);

            ui.label(format!("Attack Power: {:.1} (+{:.1})", combat_stats.attack_power, equipment_bonuses.attack_power));
            ui.label(format!("Defense: {:.1} (+{:.1})", combat_stats.defense, equipment_bonuses.defense));
            ui.label(format!("Crit Chance: {:.1}% (+{:.1}%)", combat_stats.crit_chance * 100.0, equipment_bonuses.crit_chance * 100.0));

            ui.add_space(5.0);
            ui.label(format!("Max Health: {:.0} (+{:.0})", health.max, equipment_bonuses.max_health));
            ui.label(format!("Max Mana: {:.0} (+{:.0})", mana.max, equipment_bonuses.max_mana));

            ui.add_space(10.0);
            ui.separator();
            ui.label("Total Stats (with equipment):");
            ui.label(format!("Attack Power: {:.1}", combat_stats.attack_power + equipment_bonuses.attack_power));
            ui.label(format!("Defense: {:.1}", combat_stats.defense + equipment_bonuses.defense));
            ui.label(format!("Crit Chance: {:.1}%", (combat_stats.crit_chance + equipment_bonuses.crit_chance) * 100.0));

            ui.add_space(10.0);
            ui.separator();
            ui.label("Weapon Proficiencies:");

            macro_rules! show_weapon_prof {
                ($name:expr, $level:expr, $xp:expr) => {
                    ui.label(format!("  {} (Level {})", $name, $level));
                    let xp_needed = WeaponProficiencyExp::xp_for_level($level + 1);
                    let progress = if xp_needed > 0 { $xp as f32 / xp_needed as f32 } else { 1.0 };
                    ui.add(egui::ProgressBar::new(progress).text(format!("{} / {} XP", $xp, xp_needed)));
                };
            }

            show_weapon_prof!("Sword", weapon_prof.sword, weapon_exp.sword_xp);
            show_weapon_prof!("Dagger", weapon_prof.dagger, weapon_exp.dagger_xp);
            show_weapon_prof!("Staff", weapon_prof.staff, weapon_exp.staff_xp);
            show_weapon_prof!("Wand", weapon_prof.wand, weapon_exp.wand_xp);
            show_weapon_prof!("Mace", weapon_prof.mace, weapon_exp.mace_xp);
            show_weapon_prof!("Bow", weapon_prof.bow, weapon_exp.bow_xp);
            show_weapon_prof!("Axe", weapon_prof.axe, weapon_exp.axe_xp);

            ui.add_space(10.0);
            ui.separator();
            ui.label("Armor Proficiencies:");
            ui.label(format!("  Light: {}", armor_prof.light));
            ui.label(format!("  Medium: {}", armor_prof.medium));
            ui.label(format!("  Heavy: {}", armor_prof.heavy));
        });
}

fn render_inventory_window(ctx: &egui::Context, inventory: &Inventory, equipment: &Equipment, item_db: &crate::item_cache::ClientItemDatabase, commands: &mut Commands) {
    egui::Window::new("Inventory")
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label("Right-click items to equip/unequip");
            ui.separator();

            let mut equipped_ids = std::collections::HashSet::new();
            if let Some(id) = equipment.weapon { equipped_ids.insert(id); }
            if let Some(id) = equipment.helmet { equipped_ids.insert(id); }
            if let Some(id) = equipment.chest { equipped_ids.insert(id); }
            if let Some(id) = equipment.legs { equipped_ids.insert(id); }
            if let Some(id) = equipment.boots { equipped_ids.insert(id); }

            egui::Grid::new("inventory_grid").show(ui, |ui| {
                for (i, slot) in inventory.slots.iter().enumerate() {
                    if let Some(item_stack) = slot {
                        if !equipped_ids.contains(&item_stack.item_id) {
                            let item_name = item_db.get_item_name(item_stack.item_id);
                            let mut response = ui.button(format!("{}\nx{}", item_name, item_stack.quantity));

                            response = show_item_tooltip(response, item_stack.item_id, item_db, false);

                            response.context_menu(|ui| {
                                if ui.button("Equip").clicked() {
                                    commands.client_trigger(EquipItemRequest { slot_index: i });
                                    ui.close();
                                }
                                if ui.button("Drop").clicked() {
                                    commands.client_trigger(DropItemRequest { slot_index: i });
                                    ui.close();
                                }
                            });
                        } else {
                            ui.label("[Equipped]");
                        }
                    } else {
                        ui.label("Empty");
                    }

                    if (i + 1) % 5 == 0 {
                        ui.end_row();
                    }
                }
            });
        });
}

fn render_quest_log(ctx: &egui::Context, quest_log: &QuestLog, commands: &mut Commands) {
    egui::Window::new("Quests")
        .fixed_pos([1000.0, 100.0])
        .fixed_size([270.0, 200.0])
        .show(ctx, |ui| {
            ui.label("Active Quests:");
            ui.separator();

            for active_quest in &quest_log.active_quests {
                ui.label(format!("Quest ID: {}", active_quest.quest_id));
                if ui.button("Complete").clicked() {
                    commands.client_trigger(CompleteQuestRequest {
                        quest_id: active_quest.quest_id,
                    });
                }
                ui.add_space(5.0);
            }

            if quest_log.active_quests.is_empty() {
                ui.label("No active quests");
                ui.label("Talk to theElder to start!");
            }
        });
}

fn render_notifications(ctx: &egui::Context, client_state: &mut MyClientState) {
    egui::Window::new("Notifications")
        .fixed_pos([10.0, 500.0])
        .fixed_size([300.0, 200.0])
        .title_bar(false)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for notification in client_state.notifications.iter().rev().take(10) {
                    ui.colored_label(egui::Color32::LIGHT_YELLOW, notification);
                    ui.add_space(3.0);
                }
            });
        });

    if client_state.notifications.len() > 50 {
        client_state.notifications.drain(0..25);
    }
}

fn render_controls_help(ctx: &egui::Context) {
    egui::Window::new("Controls")
        .fixed_pos([1000.0, 550.0])
        .fixed_size([270.0, 150.0])
        .show(ctx, |ui| {
            ui.label("WASD - Move");
            ui.label("Left Click - Select Target");
            ui.label("Right Click - Toggle Auto-Attack");
            ui.label("E - Interact/Pickup");
            ui.label("1-9,0 - Use Abilities");
            ui.label("ESC - Menu");
            ui.label("");
            ui.label("Click NPCs with E to talk");
        });
}

fn render_esc_menu(ctx: &egui::Context, ui_state: &mut UiState, commands: &mut Commands) {
    egui::Window::new("Menu")
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([300.0, 200.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading("Game Menu");
                ui.add_space(20.0);

                if ui.button("Return to Character Select").clicked() {
                    info!("Player requested disconnect to character select");
                    ui_state.show_esc_menu = false;
                    commands.client_trigger(DisconnectCharacterRequest);
                }

                ui.add_space(10.0);

                if ui.button("Resume").clicked() {
                    ui_state.show_esc_menu = false;
                }
            });
        });
}

fn render_quest_dialogue(ctx: &egui::Context, dialogue: &QuestDialogueData, ui_state: &mut UiState, commands: &mut Commands) {
    egui::Window::new(&dialogue.npc_name)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([500.0, 400.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading(&dialogue.quest_name);
                ui.add_space(10.0);
            });

            ui.separator();
            ui.add_space(10.0);

            ui.label(egui::RichText::new("Description:").strong());
            ui.label(&dialogue.description);
            ui.add_space(10.0);

            ui.label(egui::RichText::new("Objectives:").strong());
            ui.label(&dialogue.objectives_text);
            ui.add_space(10.0);

            ui.label(egui::RichText::new("Rewards:").strong());
            ui.label(&dialogue.rewards_text);
            ui.add_space(20.0);

            ui.separator();
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.add_space(100.0);

                if ui.button(egui::RichText::new("Accept Quest").size(16.0)).clicked() {
                    commands.client_trigger(AcceptQuestRequest {
                        quest_id: dialogue.quest_id,
                    });
                    ui_state.quest_dialogue = None;
                    info!("Accepted quest: {}", dialogue.quest_name);
                }

                ui.add_space(20.0);

                if ui.button(egui::RichText::new("Decline").size(16.0)).clicked() {
                    ui_state.quest_dialogue = None;
                    info!("Declined quest: {}", dialogue.quest_name);
                }
            });
        });
}

fn render_trainer_window(
    ctx: &egui::Context,
    mut trainer_data: TrainerWindowData,
    ui_state: &mut UiState,
    gold: &Gold,
    item_db: &crate::item_cache::ClientItemDatabase,
    commands: &mut Commands,
) {
    let window_title = match &trainer_data.trainer_type {
        Some(TrainerType::Weapon(weapon)) => format!("{} - {:?} Trainer", trainer_data.npc_name, weapon),
        Some(TrainerType::Armor(armor)) => format!("{} - {:?} Trainer", trainer_data.npc_name, armor),
        Some(TrainerType::Class(class)) => format!("{} - {:?} Trainer", trainer_data.npc_name, class),
        None => format!("{} - Shop", trainer_data.npc_name),
    };

    egui::Window::new(window_title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([450.0, 550.0])
        .show(ctx, |ui| {
            let has_shop = !trainer_data.items_for_sale.is_empty();
            let has_training = !trainer_data.teaching_quests.is_empty();

            if has_shop || has_training {
                ui.horizontal(|ui| {
                    if has_shop && ui.selectable_label(trainer_data.active_tab == TrainerTab::Shop, "Shop").clicked() {
                        trainer_data.active_tab = TrainerTab::Shop;
                        ui_state.trainer_window = Some(trainer_data.clone());
                    }
                    if has_training && ui.selectable_label(trainer_data.active_tab == TrainerTab::Training, "Training").clicked() {
                        trainer_data.active_tab = TrainerTab::Training;
                        ui_state.trainer_window = Some(trainer_data.clone());
                    }
                });
                ui.separator();
            }

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Your Gold:");
                ui.colored_label(egui::Color32::GOLD, format!("{}", gold.0));
            });
            ui.add_space(5.0);
            ui.separator();

            match trainer_data.active_tab {
                TrainerTab::Shop => render_trainer_shop(ui, &trainer_data, gold, item_db, commands),
                TrainerTab::Training => render_trainer_training(ui, &trainer_data, ui_state, commands),
            }

            ui.add_space(10.0);
            ui.separator();
            ui.vertical_centered(|ui| {
                if ui.button("Close").clicked() {
                    ui_state.trainer_window = None;
                }
            });
        });
}

fn render_trainer_shop(ui: &mut egui::Ui, trainer_data: &TrainerWindowData, gold: &Gold, item_db: &crate::item_cache::ClientItemDatabase, commands: &mut Commands) {
    if trainer_data.items_for_sale.is_empty() {
        ui.centered_and_justified(|ui| { ui.label("No items for sale."); });
    } else {
        egui::ScrollArea::vertical().max_height(350.0).show(ui, |ui| {
            for trainer_item in &trainer_data.items_for_sale {
                if let Some(item_def) = item_db.items.get(&trainer_item.item_id) {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&item_def.name).strong().size(16.0));
                                let bonuses = &item_def.stat_bonuses;
                                if bonuses.attack_power > 0.0 { ui.label(format!("Attack: +{:.1}", bonuses.attack_power)); }
                                if bonuses.crit_chance > 0.0 { ui.label(format!("Crit: +{:.1}%", bonuses.crit_chance * 100.0)); }
                                if bonuses.max_mana > 0.0 { ui.label(format!("Mana: +{:.0}", bonuses.max_mana)); }
                                if bonuses.max_health > 0.0 { ui.label(format!("Health: +{:.0}", bonuses.max_health)); }
                                if bonuses.defense > 0.0 { ui.label(format!("Defense: +{:.1}", bonuses.defense)); }
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let can_afford = gold.0 >= trainer_item.cost;
                                let button = egui::Button::new(format!("Buy\n{} gold", trainer_item.cost));
                                if ui.add_enabled(can_afford, button).clicked() {
                                    commands.client_trigger(PurchaseFromTrainerRequest {
                                        trainer_entity: Entity::PLACEHOLDER,
                                        item_id: trainer_item.item_id,
                                    });
                                }
                                if !can_afford {
                                    ui.colored_label(egui::Color32::RED, "Not enough gold");
                                }
                            });
                        });
                    });
                    ui.add_space(5.0);
                }
            }
        });
    }
}

fn render_trainer_training(ui: &mut egui::Ui, trainer_data: &TrainerWindowData, ui_state: &mut UiState, commands: &mut Commands) {
    if trainer_data.teaching_quests.is_empty() {
        ui.centered_and_justified(|ui| { ui.label("No training available."); });
    } else {
        egui::ScrollArea::vertical().max_height(350.0).show(ui, |ui| {
            for quest_info in &trainer_data.teaching_quests {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        let status_color = if quest_info.is_completed {
                            egui::Color32::GREEN
                        } else if quest_info.is_available {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::GRAY
                        };

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&quest_info.ability_reward_name)
                                .strong().size(16.0).color(status_color));
                            if quest_info.is_completed {
                                ui.colored_label(egui::Color32::GREEN, "(Learned)");
                            }
                        });

                        ui.label(egui::RichText::new(&quest_info.description).weak());

                        if quest_info.required_proficiency_level > 0 {
                            ui.label(format!("Requires: Proficiency Level {}", quest_info.required_proficiency_level));
                        }

                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            if quest_info.is_completed {
                                ui.add_enabled(false, egui::Button::new("Already Learned"));
                            } else if quest_info.is_available {
                                if ui.button("Begin Training").clicked() {
                                    commands.client_trigger(AcceptQuestRequest { quest_id: quest_info.quest_id });
                                    ui_state.trainer_window = None;
                                }
                            } else {
                                ui.add_enabled(false, egui::Button::new("Requirements Not Met"));
                            }
                        });
                    });
                });
                ui.add_space(5.0);
            }
        });
    }
}

fn render_loot_window(
    ctx: &egui::Context,
    loot_data: LootWindowData,
    ui_state: &mut UiState,
    player_pos: &Position,
    loot_query: &Query<(Entity, &Position), With<LootContainer>>,
    item_db: &crate::item_cache::ClientItemDatabase,
    commands: &mut Commands,
) {
    let mut should_close = false;

    let container_distance = if let Some((_, loot_pos)) = loot_query.iter()
        .find(|(entity, _)| *entity == loot_data.container_entity)
    {
        player_pos.0.distance(loot_pos.0)
    } else {
        f32::MAX
    };

    let in_range = container_distance <= PICKUP_RANGE;

    egui::Window::new(format!("Loot: {}", loot_data.source_name))
        .collapsible(false)
        .resizable(false)
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.label(format!("{} items", loot_data.contents.len()));

            if !in_range && container_distance < f32::MAX {
                ui.colored_label(egui::Color32::RED, "Too far away!");
            } else if container_distance == f32::MAX {
                ui.colored_label(egui::Color32::RED, "Container no longer exists!");
            }

            ui.separator();

            let mut items_to_loot: Vec<usize> = Vec::new();

            for (i, content) in loot_data.contents.iter().enumerate() {
                ui.horizontal(|ui| {
                    match content {
                        LootContents::Gold(amount) => {
                            ui.colored_label(egui::Color32::GOLD, format!("{} Gold", amount));
                            if ui.button("Take").clicked() { items_to_loot.push(i); }
                        }
                        LootContents::Item(item_stack) => {
                            if let Some(item_def) = item_db.items.get(&item_stack.item_id) {
                                let item_text = if item_stack.quantity > 1 {
                                    format!("{} (x{})", item_def.name, item_stack.quantity)
                                } else {
                                    item_def.name.clone()
                                };
                                ui.label(item_text);
                                if ui.button("Take").clicked() { items_to_loot.push(i); }
                            } else {
                                ui.label(format!("Unknown Item (ID: {})", item_stack.item_id));
                            }
                        }
                    }
                });
                ui.add_space(5.0);
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Loot All").clicked() {
                    for i in 0..loot_data.contents.len() {
                        commands.client_trigger(LootItemRequest {
                            container_entity: loot_data.container_entity,
                            loot_index: i,
                        });
                    }
                    should_close = true;
                }

                if ui.button("Close").clicked() {
                    should_close = true;
                }
            });

            for index in items_to_loot.iter() {
                commands.client_trigger(LootItemRequest {
                    container_entity: loot_data.container_entity,
                    loot_index: *index,
                });
            }
        });

    if should_close || container_distance == f32::MAX {
        ui_state.loot_window = None;
    }
}

// Event handlers

/// Handle ESC key press to toggle menu
pub fn handle_esc_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<UiState>,
    current_state: Res<State<crate::game_state::GameState>>,
) {
    if *current_state.get() != crate::game_state::GameState::InGame {
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        ui_state.show_esc_menu = !ui_state.show_esc_menu;
    }
}

/// Observer for QuestDialogueEvent - opens the quest dialogue window
pub fn handle_quest_dialogue(
    trigger: On<QuestDialogueEvent>,
    mut ui_state: ResMut<UiState>,
) {
    let event = trigger.event();
    info!("[QUEST DIALOGUE] Received event for quest: {} from NPC: {}", event.quest_name, event.npc_name);
    ui_state.quest_dialogue = Some(QuestDialogueData {
        npc_name: event.npc_name.clone(),
        quest_id: event.quest_id,
        quest_name: event.quest_name.clone(),
        description: event.description.clone(),
        objectives_text: event.objectives_text.clone(),
        rewards_text: event.rewards_text.clone(),
    });
}

/// Handle loot container contents event from server
pub fn handle_loot_container_contents(
    trigger: On<LootContainerContentsEvent>,
    mut ui_state: ResMut<UiState>,
) {
    let event = trigger.event();
    ui_state.loot_window = Some(LootWindowData {
        container_entity: event.container_entity,
        contents: event.contents.clone(),
        source_name: event.source_name.clone(),
    });
    info!("Opened loot container: {}", event.source_name);
}

/// Observer for TrainerDialogueEvent - opens the trainer shop window
pub fn handle_trainer_dialogue(
    trigger: On<TrainerDialogueEvent>,
    mut ui_state: ResMut<UiState>,
) {
    let event = trigger.event();
    info!("[TRAINER DIALOGUE] Received event from NPC: {} with {} items and {} training quests",
        event.npc_name, event.items_for_sale.len(), event.teaching_quests.len());

    let default_tab = if event.teaching_quests.is_empty() && !event.items_for_sale.is_empty() {
        TrainerTab::Shop
    } else if !event.teaching_quests.is_empty() {
        TrainerTab::Training
    } else {
        TrainerTab::Shop
    };

    ui_state.trainer_window = Some(TrainerWindowData {
        npc_name: event.npc_name.clone(),
        items_for_sale: event.items_for_sale.clone(),
        trainer_type: event.trainer_type,
        teaching_quests: event.teaching_quests.clone(),
        active_tab: default_tab,
    });
}
