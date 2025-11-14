use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use crate::game_state::{GameState, MyClientState};

#[cfg(target_family = "wasm")]
use wasm_bindgen::JsCast;

pub struct SystemMenuState {
    pub active_tab: SystemMenuTab,
    pub player_list: Vec<OnlinePlayerInfo>,
    pub ban_list: Vec<BanInfo>,
    pub server_stats: Option<ServerStatsResponse>,
    pub audit_logs: Vec<AuditLogEntry>,
    pub audit_logs_total: u32,
    pub audit_logs_offset: u32,
    pub audit_logs_limit: u32,
    // UI input fields
    pub ban_form_username: String,
    pub ban_form_duration: u32,
    pub ban_form_reason: String,
    pub ban_username: String,
    pub ban_duration: String,
    pub ban_reason: String,
    pub kick_username: String,
    pub kick_reason: String,
}

impl Default for SystemMenuState {
    fn default() -> Self {
        Self {
            active_tab: SystemMenuTab::default(),
            player_list: Vec::new(),
            ban_list: Vec::new(),
            server_stats: None,
            audit_logs: Vec::new(),
            audit_logs_total: 0,
            audit_logs_offset: 0,
            audit_logs_limit: 20, // Default page size
            ban_form_username: String::new(),
            ban_form_duration: 0,
            ban_form_reason: String::new(),
            ban_username: String::new(),
            ban_duration: String::new(),
            ban_reason: String::new(),
            kick_username: String::new(),
            kick_reason: String::new(),
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum SystemMenuTab {
    #[default]
    Players,
    Bans,
    Stats,
    Logs,
}

#[derive(Resource)]
pub struct UiState {
    pub email: String,
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
    pub loot_window: Option<LootWindowData>,
    pub show_register_tab: bool,  // Toggle between Login and Register tabs
    pub oauth_checked: bool,  // Track if we've checked for OAuth callback
    pub chat_input: String,  // Current chat message being typed
    pub chat_history: Vec<String>,  // Last 50 chat messages
    pub chat_has_focus: bool,  // Whether chat input currently has focus
    pub chat_previous_focus: bool,  // Previous frame's chat focus state
    pub is_admin: bool,  // Whether the current player is an admin
    // System menu (accessible to all players, some tabs admin-only)
    pub show_system_menu: bool,
    pub system_menu: SystemMenuState,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            email: String::new(),
            username: String::new(),
            password: String::new(),
            new_character_name: String::new(),
            selected_class: CharacterClass::Rogue,
            show_create_character: false,
            show_inventory: false,
            show_equipment: false,
            show_character_stats: false,
            show_esc_menu: false,
            quest_dialogue: None,
            loot_window: None,
            show_register_tab: false,
            oauth_checked: false,
            chat_input: String::new(),
            chat_history: Vec::new(),
            chat_has_focus: false,
            chat_previous_focus: false,
            is_admin: false,
            show_system_menu: false,
            system_menu: SystemMenuState::default(),
        }
    }
}

#[derive(Clone)]
pub struct LootWindowData {
    pub container_entity: Entity,
    pub contents: Vec<LootContents>,
    pub source_name: String,
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
            ui.add_space(40.0);

            // Tab buttons
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 120.0);
                if ui.selectable_label(!ui_state.show_register_tab, "Login").clicked() {
                    ui_state.show_register_tab = false;
                }
                if ui.selectable_label(ui_state.show_register_tab, "Register").clicked() {
                    ui_state.show_register_tab = true;
                }
            });

            ui.add_space(30.0);

            // Show either Login or Register form
            if !ui_state.show_register_tab {
                // Login form
                ui.heading("Login");
                ui.add_space(10.0);

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
            } else {
                // Register form
                ui.heading("Create New Account");
                ui.add_space(10.0);

                ui.label("Email:");
                ui.text_edit_singleline(&mut ui_state.email);
                ui.add_space(10.0);

                ui.label("Username:");
                ui.text_edit_singleline(&mut ui_state.username);
                ui.add_space(10.0);

                ui.label("Password:");
                ui.add(egui::TextEdit::singleline(&mut ui_state.password).password(true));
                ui.add_space(5.0);
                ui.colored_label(egui::Color32::GRAY, "Min 8 characters, 1 uppercase, 1 number");
                ui.add_space(20.0);

                if ui.button("Create Account").clicked() {
                    if !ui_state.email.is_empty() && !ui_state.username.is_empty() && !ui_state.password.is_empty() {
                        info!("Sending create account request for user: {}", ui_state.username);
                        commands.client_trigger(CreateAccountRequest {
                            email: ui_state.email.clone(),
                            username: ui_state.username.clone(),
                            password: ui_state.password.clone(),
                        });
                    }
                }
            }

            ui.add_space(20.0);

            // OAuth Section - "Or sign in with"
            ui.separator();
            ui.add_space(10.0);
            ui.label("Or sign in with:");
            ui.add_space(5.0);

            // Google Sign-In button
            if ui.button("🔐 Sign in with Google").clicked() {
                #[cfg(target_family = "wasm")]
                {
                    use wasm_bindgen::JsCast;
                    if let Some(window) = web_sys::window() {
                        // Google OAuth Client ID
                        let client_id = "917714705564-l5eikmnq0n0miqaurh7vbmc3dbk26e4r.apps.googleusercontent.com";
                        info!("Google Sign-In clicked - opening OAuth popup");
                        let redirect_uri = window.location().origin().unwrap_or_else(|_| "http://localhost:4000".to_string());
                        let oauth_url = format!(
                            "https://accounts.google.com/o/oauth2/v2/auth?\
                             client_id={}&\
                             redirect_uri={}&\
                             response_type=token&\
                             scope=openid%20email%20profile",
                            client_id, redirect_uri
                        );

                        // Open OAuth popup
                        let _ = window.open_with_url_and_target_and_features(
                            &oauth_url,
                            "_blank",
                            "width=500,height=600,popup=yes"
                        );
                    }
                }
                #[cfg(not(target_family = "wasm"))]
                {
                    // For native clients, open browser and show instructions
                    let client_id = "917714705564-l5eikmnq0n0miqaurh7vbmc3dbk26e4r.apps.googleusercontent.com";
                    let redirect_uri = "http://localhost:8080";
                    let oauth_url = format!(
                        "https://accounts.google.com/o/oauth2/v2/auth?\
                         client_id={}&\
                         redirect_uri={}&\
                         response_type=token&\
                         scope=openid%20email%20profile",
                        client_id, redirect_uri
                    );

                    info!("Opening browser for Google Sign-In...");
                    if let Err(e) = webbrowser::open(&oauth_url) {
                        error!("Failed to open browser: {}", e);
                        warn!("Please manually open: {}", oauth_url);
                    }

                    info!("Native OAuth not fully implemented yet. Please use the web client for OAuth login.");
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
    player_query: Query<(Entity, &Health, &Mana, &CurrentTarget, &Hotbar, &Inventory, &Equipment, &CombatStats, &LearnedAbilities, &QuestLog, &Character, &Gold, &Position), With<Player>>,
    progression_query: Query<(&Experience, &WeaponProficiency, &WeaponProficiencyExp, &ArmorProficiency)>,
    target_query: Query<(&Health, Option<&Character>, Option<&NpcName>)>,
    loot_query: Query<(Entity, &Position), With<LootContainer>>,
    item_db: Res<crate::item_cache::ClientItemDatabase>,
    ability_db: Res<crate::ability_cache::ClientAbilityDatabase>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let Some(player_entity) = client_state.player_entity else {
        return
    };

    // Silently wait for entity to be replicated with all components
    let Ok((_, health, mana, current_target, hotbar, inventory, equipment, combat_stats, _learned_abilities, quest_log, character, gold, player_pos)) = player_query.get(player_entity) else {
        return
    };

    // Get progression components (separate query to avoid hitting Bevy's query limit)
    let Ok((experience, weapon_prof, weapon_exp, armor_prof)) = progression_query.get(player_entity) else {
        return
    };

    // Player health/mana bar (top left)
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
                        // Get ability name from database instead of just ID
                        let ability_name = ability_db.get_ability_name(*ability_id);
                        format!("{}\n[{}]", ability_name, i + 1)
                    } else {
                        format!("[{}]", i + 1)
                    };

                    let mut response = ui.button(button_text);

                    // Add tooltip on hover
                    if let Some(HotbarSlot::Ability(ability_id)) = slot {
                        response = show_ability_tooltip(response, *ability_id, &ability_db);
                    }

                    // Handle clicks
                    if response.clicked() {
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
            // Admin Panel button (only visible to admins)
            if ui_state.is_admin {
                if ui.button("Admin Panel").clicked() {
                    ui_state.show_system_menu = !ui_state.show_system_menu;
                }
            }
        });

    // Check for nearby loot containers and show hint
    let mut nearest_loot_distance = f32::MAX;
    for (_, loot_pos) in &loot_query {
        let distance = player_pos.0.distance(loot_pos.0);
        if distance < nearest_loot_distance {
            nearest_loot_distance = distance;
        }
    }

    // Show "Press E to Loot" hint at bottom center when near loot
    if nearest_loot_distance <= PICKUP_RANGE {
        egui::Window::new("Loot Hint")
            .fixed_pos([540.0, 600.0])
            .fixed_size([200.0, 40.0])
            .title_bar(false)
            .frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180)))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.colored_label(egui::Color32::YELLOW, "Press E to Loot");
                });
            });
    }

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
                        let mut response = if let Some(id) = item_id {
                            let item_name = item_db.get_item_name(id);
                            ui.label(&item_name)
                        } else {
                            ui.label("<Empty>")
                        };

                        // Add tooltip on hover for equipped items
                        if let Some(id) = item_id {
                            response = show_item_tooltip(response, id, &item_db, true);
                        }

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

                // Helper macro to show proficiency with progress bar
                macro_rules! show_weapon_prof {
                    ($name:expr, $level:expr, $xp:expr) => {
                        ui.label(format!("  {} (Level {})", $name, $level));
                        let xp_needed = WeaponProficiencyExp::xp_for_level($level + 1);
                        let progress = if xp_needed > 0 {
                            $xp as f32 / xp_needed as f32
                        } else {
                            1.0
                        };
                        ui.add(egui::ProgressBar::new(progress)
                            .text(format!("{} / {} XP", $xp, xp_needed)));
                    };
                }

                show_weapon_prof!("Sword", weapon_prof.sword, weapon_exp.sword_xp);
                show_weapon_prof!("Dagger", weapon_prof.dagger, weapon_exp.dagger_xp);
                show_weapon_prof!("Staff", weapon_prof.staff, weapon_exp.staff_xp);
                show_weapon_prof!("Wand", weapon_prof.wand, weapon_exp.wand_xp);
                show_weapon_prof!("Mace", weapon_prof.mace, weapon_exp.mace_xp);
                show_weapon_prof!("Bow", weapon_prof.bow, weapon_exp.bow_xp);
                show_weapon_prof!("Axe", weapon_prof.axe, weapon_exp.axe_xp);

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
                                let mut response = ui.button(format!("{}\nx{}", item_name, item_stack.quantity));

                                // Add tooltip on hover
                                response = show_item_tooltip(response, item_stack.item_id, &item_db, false);

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

    // Loot Container Window
    if let Some(loot_data) = ui_state.loot_window.clone() {
        let mut should_close = false;

        // Check distance to loot container
        let container_distance = if let Some((_, loot_pos)) = loot_query.iter()
            .find(|(entity, _)| *entity == loot_data.container_entity)
        {
            player_pos.0.distance(loot_pos.0)
        } else {
            f32::MAX // Container doesn't exist anymore
        };

        let in_range = container_distance <= PICKUP_RANGE;

        egui::Window::new(format!("Loot: {}", loot_data.source_name))
            .collapsible(false)
            .resizable(false)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.label(format!("{} items", loot_data.contents.len()));

                // Show distance warning if too far
                if !in_range && container_distance < f32::MAX {
                    ui.colored_label(egui::Color32::RED, "Too far away!");
                } else if container_distance == f32::MAX {
                    ui.colored_label(egui::Color32::RED, "Container no longer exists!");
                }

                ui.separator();

                // Track items to loot (by index)
                let mut items_to_loot: Vec<usize> = Vec::new();

                // Display each loot item
                for (i, content) in loot_data.contents.iter().enumerate() {
                    ui.horizontal(|ui| {
                        match content {
                            LootContents::Gold(amount) => {
                                ui.colored_label(egui::Color32::GOLD, format!("{} Gold", amount));
                                if ui.button("Take").clicked() {
                                    items_to_loot.push(i);
                                }
                            }
                            LootContents::Item(item_stack) => {
                                if let Some(item_def) = item_db.items.get(&item_stack.item_id) {
                                    let item_text = if item_stack.quantity > 1 {
                                        format!("{} (x{})", item_def.name, item_stack.quantity)
                                    } else {
                                        item_def.name.clone()
                                    };
                                    ui.label(item_text);
                                    if ui.button("Take").clicked() {
                                        items_to_loot.push(i);
                                    }
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
                        // Loot all items in order
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

                // Send loot requests for individual items
                for index in items_to_loot.iter() {
                    commands.client_trigger(LootItemRequest {
                        container_entity: loot_data.container_entity,
                        loot_index: *index,
                    });
                }
            });

        // Only close on explicit close/loot all button or if container no longer exists
        if should_close || container_distance == f32::MAX {
            ui_state.loot_window = None;
        }
    }

    // Admin Dashboard window (only shown if user is admin)
    if ui_state.show_system_menu {
        system_menu_window(ctx, &mut ui_state.system_menu, &mut commands);
    }
}

// Admin Dashboard window with tabs
fn system_menu_window(
    ctx: &egui::Context,
    dashboard: &mut SystemMenuState,
    commands: &mut Commands,
) {
    egui::Window::new("System Menu")
        .default_width(800.0)
        .default_height(600.0)
        .show(ctx, |ui| {
            ui.heading("Server Information");
            ui.separator();

            // Tab selector
            ui.horizontal(|ui| {
                ui.selectable_value(&mut dashboard.active_tab, SystemMenuTab::Players, "Players");
                ui.selectable_value(&mut dashboard.active_tab, SystemMenuTab::Bans, "Bans");
                ui.selectable_value(&mut dashboard.active_tab, SystemMenuTab::Stats, "Stats");
                ui.selectable_value(&mut dashboard.active_tab, SystemMenuTab::Logs, "Logs");
            });

            ui.separator();

            // Tab content
            match dashboard.active_tab {
                SystemMenuTab::Players => {
                    render_players_tab(ui, dashboard, commands);
                }
                SystemMenuTab::Bans => {
                    render_bans_tab(ui, dashboard, commands);
                }
                SystemMenuTab::Stats => {
                    render_stats_tab(ui, dashboard, commands);
                }
                SystemMenuTab::Logs => {
                    render_logs_tab(ui, dashboard, commands);
                }
            }
        });
}

// ============================================================================
// ADMIN DASHBOARD TAB RENDERERS
// ============================================================================

/// Render the Players tab - shows online players and kick functionality
fn render_players_tab(
    ui: &mut egui::Ui,
    dashboard: &mut SystemMenuState,
    commands: &mut Commands,
) {
    ui.heading("Online Players");

    // Refresh button
    if ui.button("Refresh Player List").clicked() {
        commands.client_trigger(GetPlayerListRequest {});
    }

    ui.separator();

    // Player list
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
                        ui.label("Character");
                        ui.label("Account ID");
                        ui.label("Level");
                        ui.label("Class");
                        ui.label("Position");
                        ui.label("Actions");
                        ui.end_row();

                        // Rows
                        for player in &dashboard.player_list {
                            ui.label(&player.character_name);
                            ui.label(format!("{}", player.account_id));
                            ui.label(format!("{}", player.level));
                            ui.label(format!("{:?}", player.class));
                            ui.label(format!("({:.0}, {:.0})", player.position_x, player.position_y));

                            // Kick button
                            if ui.button("Kick").clicked() {
                                // Send kick command via admin command system
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

    // Refresh button
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

            // Clear form
            dashboard.ban_form_username.clear();
            dashboard.ban_form_duration = 0;
            dashboard.ban_form_reason.clear();
        }
    });

    ui.separator();
    ui.heading("Active Bans");

    // Ban list
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

                            // Format expiration
                            if let Some(expires) = ban.expires_at {
                                ui.label(format!("Expires: {}", expires));
                            } else {
                                ui.label("Permanent");
                            }

                            // Unban button
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

    // Refresh button
    if ui.button("Refresh Stats").clicked() {
        commands.client_trigger(GetServerStatsRequest {});
    }

    ui.separator();

    // Stats display
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

    // Audit log list
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

// ============================================================================
// TOOLTIP HELPER FUNCTIONS
// ============================================================================

/// Show ability tooltip on hover
fn show_ability_tooltip(response: egui::Response, ability_id: u32, ability_db: &crate::ability_cache::ClientAbilityDatabase) -> egui::Response {
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
fn show_item_tooltip(
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

// Handle loot container contents event from server
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

// Check for OAuth callback tokens in URL (WASM only)
#[cfg(target_family = "wasm")]
pub fn check_oauth_callback(
    mut ui_state: ResMut<UiState>,
    mut commands: Commands,
) {
    // Only check once
    if ui_state.oauth_checked {
        return;
    }
    ui_state.oauth_checked = true;

    // Get window and location
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(href) = window.location().href() else {
        return;
    };

    info!("Checking URL for OAuth callback: {}", href);

    // Parse URL hash for OAuth 2.0 implicit flow response
    // Format: #access_token=TOKEN&token_type=Bearer&expires_in=3600...
    if let Some(hash) = href.split('#').nth(1) {
        let params: std::collections::HashMap<String, String> = hash
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.split('=');
                Some((parts.next()?.to_string(), parts.next()?.to_string()))
            })
            .collect();

        if let Some(token) = params.get("access_token") {
            info!("Found OAuth token in URL, sending to server");

            // Send OAuth login request to server
            commands.client_trigger(OAuthLoginRequest {
                provider: "google".to_string(),
                token: token.clone(),
            });

            // Clean up URL by removing hash (optional - keeps URL clean)
            let clean_url = href.split('#').next().unwrap_or(&href);
            if let Ok(history) = window.history() {
                let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(clean_url));
            }
        }
    }
}

// Stub for non-WASM builds
#[cfg(not(target_family = "wasm"))]
pub fn check_oauth_callback() {
    // OAuth callback only works in WASM
}

// ============================================================================
// CHAT SYSTEM
// ============================================================================

/// Chat window for sending admin commands and regular chat messages
pub fn chat_window(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    client_state: Res<MyClientState>,
    character_query: Query<&Character>,
    mut commands: Commands,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Get character name for displaying own messages
    let character_name = if let Some(player_entity) = client_state.player_entity {
        character_query.get(player_entity)
            .map(|c| c.name.clone())
            .unwrap_or_else(|_| "Unknown".to_string())
    } else {
        "Unknown".to_string()
    };

    // Chat window at bottom-left of screen - always visible
    egui::Window::new("Chat")
        .default_pos([10.0, 400.0])
        .default_size([500.0, 250.0])
        .resizable(true)
        .show(ctx, |ui| {
            // Chat history display (scrollable area)
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if ui_state.chat_history.is_empty() {
                        ui.label("No messages yet. Type to chat with other players!");
                        if ui_state.is_admin {
                            ui.label("Admin commands: /ban, /unban, /kick, /broadcast, /help");
                        }
                    } else {
                        for message in &ui_state.chat_history {
                            ui.label(message);
                        }
                    }
                });

            ui.separator();

            // Chat input field
            let response = ui.text_edit_singleline(&mut ui_state.chat_input);

            // Track focus state changes
            let current_focus = response.has_focus();

            // If chat just gained focus (wasn't focused last frame, is focused now)
            // send a stop movement command to prevent stuck movement
            if current_focus && !ui_state.chat_previous_focus {
                commands.client_trigger(MoveInput { direction: Vec2::ZERO });
            }

            ui_state.chat_previous_focus = ui_state.chat_has_focus;
            ui_state.chat_has_focus = current_focus;

            // Send message on Enter key
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let message = ui_state.chat_input.trim().to_string();

                if !message.is_empty() {
                    // Check if it's an admin command (starts with /)
                    if message.starts_with('/') {
                        commands.client_trigger(AdminCommandRequest {
                            command: message.clone(),
                        });
                        // Show command in chat history
                        ui_state.chat_history.push(format!("[{}] {}", character_name, message));
                    } else {
                        // Send as regular chat message
                        commands.client_trigger(SendChatMessage {
                            message: message.clone(),
                        });
                        // Show own message in chat history immediately
                        ui_state.chat_history.push(format!("[{}] {}", character_name, message));
                    }

                    // Keep only last 50 messages
                    if ui_state.chat_history.len() > 50 {
                        ui_state.chat_history.remove(0);
                    }

                    // Clear input after sending
                    ui_state.chat_input.clear();

                    // Re-focus the input field so user can continue typing
                    response.request_focus();
                }
            }

            let help_text = if ui_state.is_admin {
                "Press Enter to send | Type / for admin commands"
            } else {
                "Press Enter to send"
            };
            ui.label(help_text);
        });
}

/// Receive chat messages from server and add them to chat history
pub fn receive_chat_messages(
    mut ui_state: ResMut<UiState>,
    mut chat_events: Option<EventReader<ChatMessage>>,
) {
    // Handle case where ChatMessage events haven't been initialized yet
    let Some(chat_events) = chat_events.as_mut() else {
        return;
    };

    for chat_event in chat_events.read() {
        // Format: "[Sender] message"
        let formatted_message = format!("[{}] {}", chat_event.sender, chat_event.message);

        // Add to chat history
        ui_state.chat_history.push(formatted_message);

        // Keep only last 50 messages to prevent memory bloat
        if ui_state.chat_history.len() > 50 {
            ui_state.chat_history.remove(0);
        }
    }
}
