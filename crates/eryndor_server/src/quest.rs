use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use crate::auth::ActiveCharacterEntity;
use crate::game_data::QuestDatabase;

pub fn handle_interact_npc(
    trigger: On<FromClient<InteractNpcRequest>>,
    mut commands: Commands,
    clients: Query<&ActiveCharacterEntity>,
    players: Query<(&Position, &QuestLog)>,
    npcs: Query<(Entity, (&Position, &QuestGiver, &NpcName)), With<Npc>>,
    quest_db: Res<QuestDatabase>,
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
    let Ok((player_pos, quest_log)) = players.get(char_entity) else {
        info!("Failed to get player data for character {:?}", char_entity);
        return;
    };
    info!("Got player position: {:?}", player_pos.0);

    // Find the closest NPC to the player within interaction range
    let mut closest_npc: Option<(Entity, f32, &Position, &QuestGiver, &NpcName)> = None;
    for (entity, (npc_pos, quest_giver, npc_name)) in npcs.iter() {
        let distance = player_pos.0.distance(npc_pos.0);
        info!("Found NPC '{}' at {:?}, distance: {:.2}", npc_name.0, npc_pos.0, distance);

        if distance <= INTERACTION_RANGE {
            if let Some((_, closest_dist, _, _, _)) = closest_npc {
                if distance < closest_dist {
                    closest_npc = Some((entity, distance, npc_pos, quest_giver, npc_name));
                }
            } else {
                closest_npc = Some((entity, distance, npc_pos, quest_giver, npc_name));
            }
        }
    }

    let Some((npc_entity, distance, npc_pos, quest_giver, npc_name)) = closest_npc else {
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
    // Distance was already checked in the loop above, player_pos and quest_log already retrieved

    info!("Player interacting with NPC: {}", npc_name.0);
    info!("NPC has {} quests available", quest_giver.available_quests.len());
    info!("Player quest log - Active: {}, Completed: {}",
        quest_log.active_quests.len(), quest_log.completed_quests.len());

    // Check available quests
    let mut has_available_quest = false;
    for quest_id in &quest_giver.available_quests {
        info!("Checking quest {} - can_accept: {}", quest_id, quest_log.can_accept_quest(*quest_id));
        if quest_log.can_accept_quest(*quest_id) {
            has_available_quest = true;
            if let Some(quest_def) = quest_db.quests.get(quest_id) {
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
    mut players: Query<&mut QuestLog>,
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

    // Get player quest log
    let Ok(mut quest_log) = players.get_mut(char_entity) else { return };

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
    mut players: Query<(&mut QuestLog, &Character, &Position, &mut Inventory)>,
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

    // Get player data
    let Ok((mut quest_log, character, position, mut inventory)) = players.get_mut(char_entity) else { return };

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
