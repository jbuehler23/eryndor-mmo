use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use crate::game_state::{GameState, MyClientState};

#[derive(Resource, Default)]
pub struct UiState {
    pub username: String,
    pub password: String,
    pub new_character_name: String,
    pub selected_class: CharacterClass,
    pub show_create_character: bool,
    pub show_inventory: bool,
    pub show_equipment: bool,
    pub show_character_stats: bool,
    pub show_esc_menu: bool,
    pub quest_dialogue: Option<QuestDialogueData>,
}

#[derive(Clone)]
pub struct QuestDialogueData {
    pub npc_name: String,
    pub quest_id: u32,
    pub quest_name: String,
    pub description: String,
    pub objectives_text: String,
    pub rewards_text: String,
}

pub fn login_ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut commands: Commands,
    client_state: Res<MyClientState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading("Eryndor MMO");
            ui.add_space(20.0);

            ui.label("Username:");
            ui.text_edit_singleline(&mut ui_state.username);
            ui.add_space(10.0);

            ui.label("Password:");
            ui.add(egui::TextEdit::singleline(&mut ui_state.password).password(true));
            ui.add_space(20.0);

            if ui.button("Login").clicked() {
                if !ui_state.username.is_empty() && !ui_state.password.is_empty() {
                    info!("Sending login request for user: {}", ui_state.username);
                    commands.client_trigger(LoginRequest {
                        username: ui_state.username.clone(),
                        password: ui_state.password.clone(),
                    });
                }
            }

            if ui.button("Create Account").clicked() {
                if !ui_state.username.is_empty() && !ui_state.password.is_empty() {
                    info!("Sending create account request for user: {}", ui_state.username);
                    commands.client_trigger(CreateAccountRequest {
                        username: ui_state.username.clone(),
                        password: ui_state.password.clone(),
                    });
                }
            }

            ui.add_space(20.0);

            // Show notifications
            for notification in &client_state.notifications {
                ui.colored_label(egui::Color32::YELLOW, notification);
            }
        });
    });
}

pub fn character_select_ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut commands: Commands,
    client_state: Res<MyClientState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("Select Character");
            ui.add_space(20.0);

            // List characters
            for character in &client_state.characters {
                ui.horizontal(|ui| {
                    ui.label(format!("{} - {} (Level {})", character.name, character.class.as_str(), character.level));

                    if ui.button("Play").clicked() {
                        commands.client_trigger(SelectCharacterRequest {
                            character_id: character.id,
                        });
                        next_state.set(GameState::InGame);
                    }
                });
                ui.add_space(10.0);
            }

            ui.add_space(20.0);

            if ui.button("Create New Character").clicked() {
                ui_state.show_create_character = true;
            }

            // Show notifications
            for notification in &client_state.notifications {
                ui.colored_label(egui::Color32::YELLOW, notification);
            }
        });
    });

    // Create character window
    if ui_state.show_create_character {
        egui::Window::new("Create Character")
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Character Name:");
                ui.text_edit_singleline(&mut ui_state.new_character_name);
                ui.add_space(10.0);

                ui.label("Class:");
                ui.horizontal(|ui| {
                    if ui.selectable_label(matches!(ui_state.selected_class, CharacterClass::Rogue), "Rogue").clicked() {
                        ui_state.selected_class = CharacterClass::Rogue;
                    }
                    if ui.selectable_label(matches!(ui_state.selected_class, CharacterClass::Mage), "Mage").clicked() {
                        ui_state.selected_class = CharacterClass::Mage;
                    }
                    if ui.selectable_label(matches!(ui_state.selected_class, CharacterClass::Knight), "Knight").clicked() {
                        ui_state.selected_class = CharacterClass::Knight;
                    }
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        if !ui_state.new_character_name.is_empty() {
                            commands.client_trigger(CreateCharacterRequest {
                                name: ui_state.new_character_name.clone(),
                                class: ui_state.selected_class,
                            });
                            ui_state.show_create_character = false;
                            ui_state.new_character_name.clear();
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        ui_state.show_create_character = false;
                    }
                });
            });
    }
}

pub fn game_ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut client_state: ResMut<MyClientState>,
    mut commands: Commands,
    player_query: Query<(Entity, &Health, &Mana, &CurrentTarget, &Hotbar, &Inventory, &Equipment, &CombatStats, &LearnedAbilities, &QuestLog, &Character), With<Player>>,
    progression_query: Query<(&Experience, &WeaponProficiency, &ArmorProficiency)>,
    target_query: Query<(&Health, Option<&Character>, Option<&NpcName>)>,
    item_db: Res<crate::item_cache::ClientItemDatabase>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let Some(player_entity) = client_state.player_entity else {
        return
    };

    // Silently wait for entity to be replicated with all components
    let Ok((_, health, mana, current_target, hotbar, inventory, equipment, combat_stats, _learned_abilities, quest_log, character)) = player_query.get(player_entity) else {
        return
    };

    // Get progression components (separate query to avoid hitting Bevy's query limit)
    let Ok((experience, weapon_prof, armor_prof)) = progression_query.get(player_entity) else {
        return
    };

    // Player health/mana bar (top left)
    egui::Window::new("Player Status")
        .fixed_pos([10.0, 10.0])
        .fixed_size([200.0, 80.0])
        .title_bar(false)
        .show(ctx, |ui| {
            ui.label("Health:");
            ui.add(egui::ProgressBar::new(health.percent()).text(format!("{:.0}/{:.0}", health.current, health.max)));

            ui.label("Mana:");
            ui.add(egui::ProgressBar::new(mana.percent()).text(format!("{:.0}/{:.0}", mana.current, mana.max)));
        });

    // Target frame (top center)
    if let Some(target_entity) = current_target.0 {
        if let Ok((target_health, target_char, target_npc)) = target_query.get(target_entity) {
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
    }

    // Hotbar (bottom center)
    egui::Window::new("Hotbar")
        .fixed_pos([440.0, 630.0])
        .fixed_size([400.0, 70.0])
        .title_bar(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, slot) in hotbar.slots.iter().enumerate() {
                    let button_text = if let Some(HotbarSlot::Ability(ability_id)) = slot {
                        format!("{}\n[{}]", ability_id, i + 1)
                    } else {
                        format!("[{}]", i + 1)
                    };

                    if ui.button(button_text).clicked() {
                        if let Some(HotbarSlot::Ability(ability_id)) = slot {
                            commands.client_trigger(UseAbilityRequest {
                                ability_id: *ability_id,
                            });
                        }
                    }
                }
            });
        });

    // Action buttons (top right)
    egui::Window::new("Actions")
        .fixed_pos([1090.0, 10.0])
        .fixed_size([180.0, 120.0])
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
        });

    // Equipment window
    if ui_state.show_equipment {
        egui::Window::new("Equipment")
            .collapsible(false)
            .resizable(false)
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Equipment Slots");
                ui.separator();

                // Helper function to show an equipment slot with context menu
                let mut show_slot = |ui: &mut egui::Ui, label: &str, item_id: Option<u32>, slot: EquipmentSlot| {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", label));
                        let response = if let Some(id) = item_id {
                            let item_name = item_db.get_item_name(id);
                            ui.label(&item_name)
                        } else {
                            ui.label("<Empty>")
                        };

                        // Add context menu for unequipping
                        if item_id.is_some() {
                            response.context_menu(|ui| {
                                if ui.button("Unequip").clicked() {
                                    commands.client_trigger(UnequipItemRequest { slot });
                                    ui.close_menu();
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

    // Character Stats window
    if ui_state.show_character_stats {
        egui::Window::new("Character Stats")
            .collapsible(false)
            .resizable(false)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading(&character.name);
                ui.label(format!("Class: {} | Level: {}", character.class.as_str(), character.level));

                // XP Progress Bar
                ui.add_space(5.0);
                let xp_percent = experience.current_xp as f32 / experience.xp_to_next_level as f32;
                ui.label("Experience:");
                ui.add(egui::ProgressBar::new(xp_percent)
                    .text(format!("{} / {} XP", experience.current_xp, experience.xp_to_next_level)));

                ui.separator();

                // Calculate equipment bonuses
                let equipment_bonuses = item_db.calculate_equipment_bonuses(equipment);

                // Base stats
                ui.label(format!("Attack Power: {:.1} (+{:.1})",
                    combat_stats.attack_power,
                    equipment_bonuses.attack_power));
                ui.label(format!("Defense: {:.1} (+{:.1})",
                    combat_stats.defense,
                    equipment_bonuses.defense));
                ui.label(format!("Crit Chance: {:.1}% (+{:.1}%)",
                    combat_stats.crit_chance * 100.0,
                    equipment_bonuses.crit_chance * 100.0));

                ui.add_space(5.0);

                ui.label(format!("Max Health: {:.0} (+{:.0})",
                    health.max,
                    equipment_bonuses.max_health));
                ui.label(format!("Max Mana: {:.0} (+{:.0})",
                    mana.max,
                    equipment_bonuses.max_mana));

                ui.add_space(10.0);
                ui.separator();
                ui.label("Total Stats (with equipment):");
                ui.label(format!("Attack Power: {:.1}",
                    combat_stats.attack_power + equipment_bonuses.attack_power));
                ui.label(format!("Defense: {:.1}",
                    combat_stats.defense + equipment_bonuses.defense));
                ui.label(format!("Crit Chance: {:.1}%",
                    (combat_stats.crit_chance + equipment_bonuses.crit_chance) * 100.0));

                // Weapon Proficiencies
                ui.add_space(10.0);
                ui.separator();
                ui.label("Weapon Proficiencies:");
                ui.label(format!("  Sword: {}", weapon_prof.sword));
                ui.label(format!("  Dagger: {}", weapon_prof.dagger));
                ui.label(format!("  Staff: {}", weapon_prof.staff));
                ui.label(format!("  Mace: {}", weapon_prof.mace));
                ui.label(format!("  Bow: {}", weapon_prof.bow));
                ui.label(format!("  Axe: {}", weapon_prof.axe));

                // Armor Proficiencies
                ui.add_space(10.0);
                ui.separator();
                ui.label("Armor Proficiencies:");
                ui.label(format!("  Light: {}", armor_prof.light));
                ui.label(format!("  Medium: {}", armor_prof.medium));
                ui.label(format!("  Heavy: {}", armor_prof.heavy));
            });
    }

    // Inventory window
    if ui_state.show_inventory {
        egui::Window::new("Inventory")
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Right-click items to equip/unequip");
                ui.separator();

                // Build set of equipped item IDs to hide them from inventory
                let mut equipped_ids = std::collections::HashSet::new();
                if let Some(id) = equipment.weapon { equipped_ids.insert(id); }
                if let Some(id) = equipment.helmet { equipped_ids.insert(id); }
                if let Some(id) = equipment.chest { equipped_ids.insert(id); }
                if let Some(id) = equipment.legs { equipped_ids.insert(id); }
                if let Some(id) = equipment.boots { equipped_ids.insert(id); }

                egui::Grid::new("inventory_grid").show(ui, |ui| {
                    for (i, slot) in inventory.slots.iter().enumerate() {
                        if let Some(item_stack) = slot {
                            // Hide equipped items from inventory display
                            if !equipped_ids.contains(&item_stack.item_id) {
                                let item_name = item_db.get_item_name(item_stack.item_id);
                                let response = ui.button(format!("{}\nx{}", item_name, item_stack.quantity));

                                // Add context menu for equipping/dropping
                                response.context_menu(|ui| {
                                    if ui.button("Equip").clicked() {
                                        commands.client_trigger(EquipItemRequest {
                                            slot_index: i,
                                        });
                                        ui.close_menu();
                                    }
                                    if ui.button("Drop").clicked() {
                                        commands.client_trigger(DropItemRequest {
                                            slot_index: i,
                                        });
                                        ui.close_menu();
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

    // Quest log (right side)
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

    // Notifications (bottom left)
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

    // Clear old notifications
    if client_state.notifications.len() > 50 {
        client_state.notifications.drain(0..25);
    }

    // Controls help (bottom right)
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

    // ESC Menu
    if ui_state.show_esc_menu {
        egui::Window::new("Menu")
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([300.0, 150.0])
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

    // Quest Dialogue Window
    if let Some(dialogue) = ui_state.quest_dialogue.clone() {
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

                // Description
                ui.label(egui::RichText::new("Description:").strong());
                ui.label(&dialogue.description);
                ui.add_space(10.0);

                // Objectives
                ui.label(egui::RichText::new("Objectives:").strong());
                ui.label(&dialogue.objectives_text);
                ui.add_space(10.0);

                // Rewards
                ui.label(egui::RichText::new("Rewards:").strong());
                ui.label(&dialogue.rewards_text);
                ui.add_space(20.0);

                ui.separator();
                ui.add_space(10.0);

                // Buttons
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
}

pub fn handle_esc_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<UiState>,
    current_state: Res<State<GameState>>,
) {
    // Only handle ESC in the InGame state
    if *current_state.get() != GameState::InGame {
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
    info!("[QUEST DIALOGUE] Dialogue window opened for quest: {}", event.quest_name);
}
