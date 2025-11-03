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
    player_query: Query<(Entity, &Health, &Mana, &CurrentTarget, &Hotbar, &Inventory, &LearnedAbilities, &QuestLog), With<Player>>,
    target_query: Query<(&Health, Option<&Character>, Option<&NpcName>)>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let Some(player_entity) = client_state.player_entity else {
        return
    };

    // Silently wait for entity to be replicated with all components
    let Ok((_, health, mana, current_target, hotbar, inventory, _learned_abilities, quest_log)) = player_query.get(player_entity) else {
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

    // Inventory button
    egui::Window::new("Menu")
        .fixed_pos([1180.0, 10.0])
        .fixed_size([90.0, 60.0])
        .title_bar(false)
        .show(ctx, |ui| {
            if ui.button("Inventory").clicked() {
                ui_state.show_inventory = !ui_state.show_inventory;
            }
        });

    // Inventory window
    if ui_state.show_inventory {
        egui::Window::new("Inventory")
            .collapsible(false)
            .show(ctx, |ui| {
                egui::Grid::new("inventory_grid").show(ui, |ui| {
                    for (i, slot) in inventory.slots.iter().enumerate() {
                        if let Some(item_stack) = slot {
                            if ui.button(format!("Item {}\nx{}", item_stack.item_id, item_stack.quantity)).clicked() {
                                // Right-click to drop (simplified for POC)
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
            ui.label("Click - Select Target");
            ui.label("E - Interact/Pickup");
            ui.label("1-9,0 - Use Abilities");
            ui.label("");
            ui.label("Click NPCs with E to talk");
            ui.label("Click items with E to pickup");
        });
}
