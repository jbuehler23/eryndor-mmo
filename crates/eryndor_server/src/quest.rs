use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use crate::auth::ActiveCharacterEntity;
use crate::game_data::QuestDatabase;
use crate::abilities::AbilityDatabase;

pub fn handle_interact_npc(
    trigger: On<FromClient<InteractNpcRequest>>,
    mut commands: Commands,
    clients: Query<&ActiveCharacterEntity>,
    players: Query<(&Position, &QuestLog, &WeaponProficiency, &ArmorProficiency)>,
    npcs: Query<(Entity, &Position, Option<&QuestGiver>, Option<&Trainer>, &NpcName), With<Npc>>,
    quest_db: Res<QuestDatabase>,
    ability_db: Res<AbilityDatabase>,
) {
    info!("=== INTERACT NPC HANDLER CALLED ===");
    let Some(client_entity) = trigger.client_id.entity() else {
        info!("No client entity found");
        return;
    };
    let request = trigger.event();
    info!("Interact request for NPC entity: {:?}", request.npc_entity);

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else {
        info!("Failed to get active character for client {:?}", client_entity);
        return;
    };
    let char_entity = active_char.0;
    info!("Character entity: {:?}", char_entity);

    // The client sends a replicated entity ID, but we need the server-side NPC entity
    // Instead of using the entity ID directly, find the NPC by proximity to the player
    info!("Client sent NPC entity: {:?} (this is a client-side replicated entity)", request.npc_entity);

    // Get player position first
    let Ok((player_pos, quest_log, weapon_prof, armor_prof)) = players.get(char_entity) else {
        info!("Failed to get player data for character {:?}", char_entity);
        return;
    };
    info!("Got player position: {:?}", player_pos.0);

    // Find the closest NPC to the player within interaction range
    let mut closest_npc: Option<(Entity, f32, &Position, Option<&QuestGiver>, Option<&Trainer>, &NpcName)> = None;
    for (entity, npc_pos, quest_giver, trainer, npc_name) in npcs.iter() {
        let distance = player_pos.0.distance(npc_pos.0);
        info!("Found NPC '{}' at {:?}, distance: {:.2}", npc_name.0, npc_pos.0, distance);

        if distance <= INTERACTION_RANGE {
            if let Some((_, closest_dist, _, _, _, _)) = closest_npc {
                if distance < closest_dist {
                    closest_npc = Some((entity, distance, npc_pos, quest_giver, trainer, npc_name));
                }
            } else {
                closest_npc = Some((entity, distance, npc_pos, quest_giver, trainer, npc_name));
            }
        }
    }

    let Some((npc_entity, distance, npc_pos, quest_giver, trainer, npc_name)) = closest_npc else {
        info!("No NPC found within interaction range");
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message: "No NPC nearby!".to_string(),
                notification_type: NotificationType::Warning,
            },
        });
        return;
    };

    info!("Found closest NPC: {} at distance {:.2}", npc_name.0, distance);
    info!("Player interacting with NPC: {}", npc_name.0);

    // Check if NPC is a trainer
    if let Some(trainer_comp) = trainer {
        info!("NPC is a trainer with {} items for sale, {} teaching quests",
            trainer_comp.items_for_sale.len(), trainer_comp.teaching_quests.len());

        // Build teaching quest info list
        let teaching_quests: Vec<TrainerQuestInfo> = trainer_comp.teaching_quests.iter()
            .filter_map(|quest_id| {
                let quest_def = quest_db.quests.get(quest_id)?;

                // Check if already completed
                let is_completed = quest_log.completed_quests.contains(quest_id);

                // Check proficiency requirements
                let mut is_available = !is_completed && !quest_log.active_quests.iter().any(|q| q.quest_id == *quest_id);
                let mut required_proficiency_level = 0u32;

                for (weapon_type, required_level) in &quest_def.proficiency_requirements {
                    required_proficiency_level = *required_level;
                    let current_level = crate::weapon::get_proficiency_level(weapon_prof, weapon_type);
                    if current_level < *required_level {
                        is_available = false;
                    }
                }

                // Get ability reward name (first reward ability)
                let ability_reward_name = quest_def.reward_abilities.first()
                    .and_then(|ability_id| ability_db.get(*ability_id))
                    .map(|def| def.name.clone())
                    .unwrap_or_else(|| "Unknown Ability".to_string());

                Some(TrainerQuestInfo {
                    quest_id: *quest_id,
                    quest_name: quest_def.name.clone(),
                    description: quest_def.description.clone(),
                    required_proficiency_level,
                    ability_reward_name,
                    is_available,
                    is_completed,
                })
            })
            .collect();

        // Send trainer dialogue event to open the shop window
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: TrainerDialogueEvent {
                npc_name: npc_name.0.clone(),
                items_for_sale: trainer_comp.items_for_sale.clone(),
                trainer_type: trainer_comp.trainer_type,
                teaching_quests,
            },
        });
        return;
    }

    // Check if NPC is a quest giver
    let Some(quest_giver) = quest_giver else {
        info!("NPC is neither a trainer nor a quest giver");
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message: "This NPC has nothing to offer.".to_string(),
                notification_type: NotificationType::Info,
            },
        });
        return;
    };

    info!("NPC has {} quests available", quest_giver.available_quests.len());
    info!("Player quest log - Active: {}, Completed: {}",
        quest_log.active_quests.len(), quest_log.completed_quests.len());

    // Check available quests
    let mut has_available_quest = false;
    for quest_id in &quest_giver.available_quests {
        info!("Checking quest {} - can_accept: {}", quest_id, quest_log.can_accept_quest(*quest_id));
        if quest_log.can_accept_quest(*quest_id) {
            if let Some(quest_def) = quest_db.quests.get(quest_id) {
                // Check proficiency requirements
                let mut meets_prof_requirements = true;
                for (weapon_type, required_level) in &quest_def.proficiency_requirements {
                    let current_level = crate::weapon::get_proficiency_level(weapon_prof, weapon_type);
                    if current_level < *required_level {
                        meets_prof_requirements = false;
                        info!("Player doesn't meet proficiency requirement for quest {}: {:?} level {} (has {})",
                            quest_id, weapon_type, required_level, current_level);
                        break;
                    }
                }

                if !meets_prof_requirements {
                    continue; // Skip this quest
                }

                has_available_quest = true;
                // Format objectives text
                let objectives_vec: Vec<String> = quest_def.objectives.iter().enumerate()
                    .map(|(i, obj)| match obj {
                        crate::game_data::QuestObjective::ObtainItem { item_id, count } => {
                            format!("{}. Obtain {} x{}", i + 1, item_id, count)
                        }
                        crate::game_data::QuestObjective::KillEnemy { enemy_type, count } => {
                            format!("{}. Kill {} enemies x{}", i + 1, enemy_type, count)
                        }
                        crate::game_data::QuestObjective::TalkToNpc { npc_id } => {
                            format!("{}. Talk to NPC {}", i + 1, npc_id)
                        }
                    })
                    .collect();
                let objectives_text = objectives_vec.join("\n");

                let rewards_text = format!("{} XP", quest_def.reward_exp);

                info!("Sending QuestDialogueEvent to client for quest: {} (ID: {})", quest_def.name, quest_id);

                // Send quest dialogue event to open the dialogue window
                commands.server_trigger(ToClients {
                    mode: SendMode::Direct(ClientId::Client(client_entity)),
                    message: QuestDialogueEvent {
                        npc_name: npc_name.0.clone(),
                        quest_id: *quest_id,
                        quest_name: quest_def.name.clone(),
                        description: quest_def.description.clone(),
                        objectives_text,
                        rewards_text,
                    },
                });
            }
        }
    }

    if !has_available_quest {
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message: format!("{}: Hello, adventurer!", npc_name.0),
                notification_type: NotificationType::Info,
            },
        });
    }
}

pub fn handle_accept_quest(
    trigger: On<FromClient<AcceptQuestRequest>>,
    mut commands: Commands,
    clients: Query<&ActiveCharacterEntity>,
    mut players: Query<(&mut QuestLog, &WeaponProficiency)>,
    quest_db: Res<QuestDatabase>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };
    let request = trigger.event();

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else { return };
    let char_entity = active_char.0;

    // Get quest definition
    let Some(quest_def) = quest_db.quests.get(&request.quest_id) else {
        return;
    };

    // Get player quest log and proficiency
    let Ok((mut quest_log, weapon_prof)) = players.get_mut(char_entity) else { return };

    // Check if can accept quest
    if !quest_log.can_accept_quest(request.quest_id) {
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message: "Quest already active or completed!".to_string(),
                notification_type: NotificationType::Warning,
            },
        });
        return;
    }

    // Check proficiency requirements
    for (weapon_type, required_level) in &quest_def.proficiency_requirements {
        let current_level = crate::weapon::get_proficiency_level(weapon_prof, weapon_type);
        if current_level < *required_level {
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: NotificationEvent {
                    message: format!("You need {:?} proficiency level {} to accept this quest!", weapon_type, required_level),
                    notification_type: NotificationType::Warning,
                },
            });
            return;
        }
    }

    // Add quest to active quests
    quest_log.active_quests.push(ActiveQuest {
        quest_id: request.quest_id,
        progress: vec![0; quest_def.objectives.len()],
    });

    info!("Player accepted quest: {}", quest_def.name);

    commands.server_trigger(ToClients {
        mode: SendMode::Direct(ClientId::Client(client_entity)),
        message: NotificationEvent {
            message: format!("Quest accepted: {}", quest_def.name),
            notification_type: NotificationType::Success,
        },
    });

    commands.server_trigger(ToClients {
        mode: SendMode::Direct(ClientId::Client(client_entity)),
        message: QuestUpdateEvent {
            quest_id: request.quest_id,
            message: format!("New quest: {}", quest_def.name),
        },
    });
}

pub fn handle_complete_quest(
    trigger: On<FromClient<CompleteQuestRequest>>,
    mut commands: Commands,
    clients: Query<&ActiveCharacterEntity>,
    mut players: Query<(&mut QuestLog, &Character, &Position, &mut Inventory, &mut LearnedAbilities)>,
    quest_db: Res<QuestDatabase>,
    ability_db: Res<crate::abilities::AbilityDatabase>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };
    let request = trigger.event();

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else { return };
    let char_entity = active_char.0;

    // Get quest definition
    let Some(quest_def) = quest_db.quests.get(&request.quest_id) else {
        return;
    };

    // Get player data
    let Ok((mut quest_log, character, position, mut inventory, mut learned_abilities)) = players.get_mut(char_entity) else { return };

    // Find active quest
    let quest_index = quest_log.active_quests.iter().position(|q| q.quest_id == request.quest_id);
    let Some(index) = quest_index else {
        return;
    };

    // Check if all objectives complete
    let active_quest = &quest_log.active_quests[index];
    let mut all_complete = true;
    for (i, objective) in quest_def.objectives.iter().enumerate() {
        match objective {
            crate::game_data::QuestObjective::ObtainItem { count, .. } => {
                if active_quest.progress[i] < *count {
                    all_complete = false;
                    break;
                }
            }
            crate::game_data::QuestObjective::TalkToNpc { .. } => {
                // Always completable when talking to NPC
                continue;
            }
            _ => {}
        }
    }

    if !all_complete {
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message: "Quest objectives not complete!".to_string(),
                notification_type: NotificationType::Warning,
            },
        });
        return;
    }

    // Complete quest
    quest_log.active_quests.remove(index);
    quest_log.completed_quests.insert(request.quest_id);

    info!("Player completed quest: {}", quest_def.name);

    // Grant quest rewards
    // For "Choose Your Path" quest, give class-appropriate weapon
    if request.quest_id == QUEST_FIRST_WEAPON {
        let weapon_id = character.class.starting_weapon();

        // Add weapon to inventory
        if inventory.add_item(ItemStack {
            item_id: weapon_id,
            quantity: 1,
        }) {
            let weapon_name = match weapon_id {
                ITEM_DAGGER => "Dagger",
                ITEM_WAND => "Wand",
                ITEM_SWORD => "Sword",
                _ => "Weapon",
            };

            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: NotificationEvent {
                    message: format!("Received: {}!", weapon_name),
                    notification_type: NotificationType::Success,
                },
            });
        }
    }

    // Grant ability rewards
    for ability_id in &quest_def.reward_abilities {
        if !learned_abilities.abilities.contains(ability_id) {
            learned_abilities.abilities.insert(*ability_id);
            info!("Player learned ability {} from quest {}", ability_id, quest_def.name);

            let ability_name = ability_db
                .get(*ability_id)
                .map(|def| def.name.clone())
                .unwrap_or_else(|| format!("Ability {}", ability_id));

            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: NotificationEvent {
                    message: format!("New ability unlocked: {}!", ability_name),
                    notification_type: NotificationType::Success,
                },
            });
        }
    }

    commands.server_trigger(ToClients {
        mode: SendMode::Direct(ClientId::Client(client_entity)),
        message: NotificationEvent {
            message: format!("Quest complete: {}! Gained {} XP", quest_def.name, quest_def.reward_exp),
            notification_type: NotificationType::Success,
        },
    });
}

pub fn update_quest_progress(
    mut players: Query<(&mut QuestLog, &Inventory), Changed<Inventory>>,
    quest_db: Res<QuestDatabase>,
) {
    for (mut quest_log, inventory) in &mut players {
        for active_quest in &mut quest_log.active_quests {
            let Some(quest_def) = quest_db.quests.get(&active_quest.quest_id) else {
                continue;
            };

            for (i, objective) in quest_def.objectives.iter().enumerate() {
                match objective {
                    crate::game_data::QuestObjective::ObtainItem { item_id, count } => {
                        // For "any weapon" quest, check for any weapon
                        if *item_id == 0 {
                            // Check for any weapon item
                            let has_weapon = inventory.has_item(ITEM_DAGGER)
                                || inventory.has_item(ITEM_WAND)
                                || inventory.has_item(ITEM_SWORD);

                            if has_weapon {
                                active_quest.progress[i] = 1;
                            }
                        } else {
                            // Check for specific item
                            if inventory.has_item(*item_id) {
                                active_quest.progress[i] = *count;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
