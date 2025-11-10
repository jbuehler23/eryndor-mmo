use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use crate::auth::ActiveCharacterEntity;
use crate::game_data::ItemDatabase;

pub fn handle_pickup_item(
    trigger: On<FromClient<PickupItemRequest>>,
    mut commands: Commands,
    clients: Query<&ActiveCharacterEntity>,
    mut players: Query<(&Position, &mut Inventory, &mut LearnedAbilities, &mut Hotbar, &mut Gold)>,
    world_items: Query<(&Position, Option<&WorldItem>, Option<&GoldDrop>)>,
    item_db: Res<ItemDatabase>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };
    let request = trigger.event();

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else { return };
    let char_entity = active_char.0;

    // Check if entity exists
    let Ok((item_pos, world_item, gold_drop)) = world_items.get(request.item_entity) else {
        return;
    };

    // Get player data
    let Ok((player_pos, mut inventory, mut learned_abilities, mut hotbar, mut player_gold)) =
        players.get_mut(char_entity) else { return };

    // Check range
    let distance = player_pos.0.distance(item_pos.0);
    if distance > PICKUP_RANGE {
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message: "Too far away!".to_string(),
                notification_type: NotificationType::Warning,
            },
        });
        return;
    }

    // Check if this is a gold drop
    if let Some(gold_drop) = gold_drop {
        let amount = gold_drop.0;
        player_gold.0 += amount;

        info!("Player picked up {} gold (new total: {})", amount, player_gold.0);
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message: format!("Picked up {} gold", amount),
                notification_type: NotificationType::Info,
            },
        });

        // Remove gold from world
        commands.entity(request.item_entity).despawn();
        return;
    }

    // Otherwise, handle as a regular item
    if let Some(world_item) = world_item {
        // Add item to inventory
        let item_stack = ItemStack {
            item_id: world_item.item_id,
            quantity: 1,
        };

        if inventory.add_item(item_stack.clone()) {
            // Get item definition
            if let Some(item_def) = item_db.items.get(&world_item.item_id) {
                info!("Player picked up: {}", item_def.name);

                // Note: Abilities are now class-based, not item-based
                // Items can still be picked up for other purposes (consumables, materials, etc.)

                commands.server_trigger(ToClients {
                    mode: SendMode::Direct(ClientId::Client(client_entity)),
                    message: NotificationEvent {
                        message: format!("Picked up {}", item_def.name),
                        notification_type: NotificationType::Info,
                    },
                });
            }

            // Remove item from world
            commands.entity(request.item_entity).despawn();
        } else {
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: NotificationEvent {
                    message: "Inventory full!".to_string(),
                    notification_type: NotificationType::Warning,
                },
            });
        }
    }
}

pub fn handle_drop_item(
    trigger: On<FromClient<DropItemRequest>>,
    mut commands: Commands,
    clients: Query<&ActiveCharacterEntity>,
    mut players: Query<(&Position, &mut Inventory)>,
    item_db: Res<ItemDatabase>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };
    let request = trigger.event();

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else { return };
    let char_entity = active_char.0;

    // Get player data
    let Ok((player_pos, mut inventory)) = players.get_mut(char_entity) else { return };

    // Remove item from inventory
    if let Some(item_stack) = inventory.remove_item(request.slot_index) {
        // Get item visual
        let visual = get_item_visual(item_stack.item_id);

        // Spawn item in world near player
        let drop_offset = Vec2::new(rand::random::<f32>() * 40.0 - 20.0, 30.0);
        commands.spawn((
            Replicated,
            WorldItem {
                item_id: item_stack.item_id,
            },
            Position(player_pos.0 + drop_offset),
            visual,
        ));

        if let Some(item_def) = item_db.items.get(&item_stack.item_id) {
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: NotificationEvent {
                    message: format!("Dropped {}", item_def.name),
                    notification_type: NotificationType::Info,
                },
            });
        }
    }
}

pub fn handle_equip_item(
    trigger: On<FromClient<EquipItemRequest>>,
    mut commands: Commands,
    clients: Query<&ActiveCharacterEntity>,
    mut players: Query<(&mut Inventory, &mut Equipment)>,
    item_db: Res<crate::game_data::ItemDatabase>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };
    let request = trigger.event();

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else { return };
    let char_entity = active_char.0;

    // Get player data
    let Ok((mut inventory, mut equipment)) = players.get_mut(char_entity) else { return };

    // Get item from inventory
    if request.slot_index < inventory.slots.len() {
        if let Some(item_stack) = &inventory.slots[request.slot_index] {
            let item_id = item_stack.item_id;

            // Look up item definition to determine slot
            if let Some(item_def) = item_db.items.get(&item_id) {
                use crate::game_data::ItemType;

                let (slot_name, equipped, old_item) = match item_def.item_type {
                    ItemType::Weapon => {
                        let old = equipment.weapon.take();
                        equipment.weapon = Some(item_id);
                        ("Weapon", true, old)
                    }
                    ItemType::Helmet => {
                        let old = equipment.helmet.take();
                        equipment.helmet = Some(item_id);
                        ("Helmet", true, old)
                    }
                    ItemType::Chest => {
                        let old = equipment.chest.take();
                        equipment.chest = Some(item_id);
                        ("Chest", true, old)
                    }
                    ItemType::Legs => {
                        let old = equipment.legs.take();
                        equipment.legs = Some(item_id);
                        ("Legs", true, old)
                    }
                    ItemType::Boots => {
                        let old = equipment.boots.take();
                        equipment.boots = Some(item_id);
                        ("Boots", true, old)
                    }
                    _ => {
                        commands.server_trigger(ToClients {
                            mode: SendMode::Direct(ClientId::Client(client_entity)),
                            message: NotificationEvent {
                                message: "This item cannot be equipped!".to_string(),
                                notification_type: NotificationType::Warning,
                            },
                        });
                        return;
                    }
                };

                if equipped {
                    // Put the old item back in the inventory slot (swap)
                    inventory.slots[request.slot_index] = old_item.map(|id| ItemStack {
                        item_id: id,
                        quantity: 1,
                    });

                    info!("Player equipped {} in {} slot", item_def.name, slot_name);
                    commands.server_trigger(ToClients {
                        mode: SendMode::Direct(ClientId::Client(client_entity)),
                        message: NotificationEvent {
                            message: format!("{} equipped!", item_def.name),
                            notification_type: NotificationType::Success,
                        },
                    });
                }
            }
        }
    }
}

pub fn handle_unequip_item(
    trigger: On<FromClient<UnequipItemRequest>>,
    mut commands: Commands,
    clients: Query<&ActiveCharacterEntity>,
    mut players: Query<(&mut Inventory, &mut Equipment)>,
    item_db: Res<crate::game_data::ItemDatabase>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };
    let request = trigger.event();

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else { return };
    let char_entity = active_char.0;

    // Get player data
    let Ok((mut inventory, mut equipment)) = players.get_mut(char_entity) else { return };

    // Get the item ID from the equipment slot
    let item_id_opt = match request.slot {
        EquipmentSlot::Weapon => equipment.weapon.take(),
        EquipmentSlot::Helmet => equipment.helmet.take(),
        EquipmentSlot::Chest => equipment.chest.take(),
        EquipmentSlot::Legs => equipment.legs.take(),
        EquipmentSlot::Boots => equipment.boots.take(),
    };

    if let Some(item_id) = item_id_opt {
        // Try to add the item back to inventory
        let item_stack = ItemStack {
            item_id,
            quantity: 1,
        };
        if inventory.add_item(item_stack) {
            let item_name = item_db.items.get(&item_id)
                .map(|i| i.name.clone())
                .unwrap_or_else(|| format!("Item {}", item_id));

            info!("Player unequipped {} from {:?} slot", item_name, request.slot);
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: NotificationEvent {
                    message: format!("{} unequipped!", item_name),
                    notification_type: NotificationType::Success,
                },
            });
        } else {
            // Inventory is full - re-equip the item
            match request.slot {
                EquipmentSlot::Weapon => equipment.weapon = Some(item_id),
                EquipmentSlot::Helmet => equipment.helmet = Some(item_id),
                EquipmentSlot::Chest => equipment.chest = Some(item_id),
                EquipmentSlot::Legs => equipment.legs = Some(item_id),
                EquipmentSlot::Boots => equipment.boots = Some(item_id),
            }

            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: NotificationEvent {
                    message: "Inventory is full!".to_string(),
                    notification_type: NotificationType::Warning,
                },
            });
        }
    }
}

pub fn handle_set_hotbar_slot(
    trigger: On<FromClient<SetHotbarSlotRequest>>,
    clients: Query<&ActiveCharacterEntity>,
    mut players: Query<&mut Hotbar>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };
    let request = trigger.event();

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else { return };
    let char_entity = active_char.0;

    // Update hotbar
    if let Ok(mut hotbar) = players.get_mut(char_entity) {
        if request.slot_index < hotbar.slots.len() {
            hotbar.slots[request.slot_index] = request.content;
        }
    }
}

fn get_item_visual(item_id: u32) -> VisualShape {
    match item_id {
        ITEM_DAGGER => VisualShape {
            shape_type: ShapeType::Triangle,
            color: COLOR_ITEM_DAGGER,
            size: ITEM_SIZE,
        },
        ITEM_WAND => VisualShape {
            shape_type: ShapeType::Diamond,
            color: COLOR_ITEM_WAND,
            size: ITEM_SIZE,
        },
        ITEM_SWORD => VisualShape {
            shape_type: ShapeType::Square,
            color: COLOR_ITEM_SWORD,
            size: ITEM_SIZE,
        },
        _ => VisualShape {
            shape_type: ShapeType::Circle,
            color: [0.5, 0.5, 0.5, 1.0],
            size: ITEM_SIZE,
        },
    }
}

/// Handle opening a loot container to preview its contents
/// Sends LootContainerContentsEvent to the client with all items
pub fn handle_open_loot_container(
    trigger: On<FromClient<OpenLootContainerRequest>>,
    mut commands: Commands,
    clients: Query<&ActiveCharacterEntity>,
    loot_containers: Query<&LootContainer>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };
    let request = trigger.event();

    // Get client's character
    let Ok(_active_char) = clients.get(client_entity) else { return };

    // Get loot container
    let Ok(container) = loot_containers.get(request.container_entity) else {
        info!("Loot container not found: {:?}", request.container_entity);
        return;
    };

    // Send container contents to client (no range restriction for preview)
    commands.server_trigger(ToClients {
        mode: SendMode::Direct(ClientId::Client(client_entity)),
        message: LootContainerContentsEvent {
            container_entity: request.container_entity,
            contents: container.contents.clone(),
            source_name: container.source_name.clone(),
        },
    });

    info!("Sent loot container contents from {} to client", container.source_name);
}

/// Handle looting a specific item from a container by index
pub fn handle_loot_item(
    trigger: On<FromClient<LootItemRequest>>,
    mut commands: Commands,
    clients: Query<&ActiveCharacterEntity>,
    mut players: Query<(&Position, &mut Inventory, &mut Gold)>,
    mut loot_containers: Query<(&Position, &mut LootContainer)>,
    item_db: Res<ItemDatabase>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };
    let request = trigger.event();

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else { return };
    let char_entity = active_char.0;

    // Get loot container
    let Ok((container_pos, mut container)) = loot_containers.get_mut(request.container_entity) else {
        return;
    };

    // Get player data
    let Ok((player_pos, mut inventory, mut player_gold)) = players.get_mut(char_entity) else {
        return;
    };

    // Check range
    let distance = player_pos.0.distance(container_pos.0);
    if distance > PICKUP_RANGE {
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message: "Too far away!".to_string(),
                notification_type: NotificationType::Warning,
            },
        });
        return;
    }

    // Check if loot index is valid
    if request.loot_index >= container.contents.len() {
        return;
    }

    // Get and remove the loot item
    let loot_item = container.contents.remove(request.loot_index);

    // Process the loot
    match loot_item {
        LootContents::Gold(amount) => {
            player_gold.0 += amount;
            info!("Player looted {} gold (new total: {})", amount, player_gold.0);
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: NotificationEvent {
                    message: format!("Looted {} gold", amount),
                    notification_type: NotificationType::Info,
                },
            });
        }
        LootContents::Item(item_stack) => {
            if inventory.add_item(item_stack.clone()) {
                if let Some(item_def) = item_db.items.get(&item_stack.item_id) {
                    info!("Player looted: {} (x{})", item_def.name, item_stack.quantity);
                    commands.server_trigger(ToClients {
                        mode: SendMode::Direct(ClientId::Client(client_entity)),
                        message: NotificationEvent {
                            message: format!("Looted {} (x{})", item_def.name, item_stack.quantity),
                            notification_type: NotificationType::Info,
                        },
                    });
                }
            } else {
                // Inventory full, put item back
                container.contents.insert(request.loot_index, LootContents::Item(item_stack));
                commands.server_trigger(ToClients {
                    mode: SendMode::Direct(ClientId::Client(client_entity)),
                    message: NotificationEvent {
                        message: "Inventory full!".to_string(),
                        notification_type: NotificationType::Warning,
                    },
                });
                return;
            }
        }
    }

    // If container is empty, despawn it
    if container.contents.is_empty() {
        commands.entity(request.container_entity).despawn();
        info!("Loot container emptied and despawned");
    }
}

/// Handle auto-looting the nearest container (loot all items)
pub fn handle_auto_loot(
    trigger: On<FromClient<AutoLootRequest>>,
    mut commands: Commands,
    clients: Query<&ActiveCharacterEntity>,
    mut players: Query<(&Position, &mut Inventory, &mut Gold)>,
    mut loot_containers: Query<(Entity, &Position, &mut LootContainer)>,
    item_db: Res<ItemDatabase>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else { return };
    let char_entity = active_char.0;

    // Get player data
    let Ok((player_pos, mut inventory, mut player_gold)) = players.get_mut(char_entity) else {
        return;
    };

    // Find nearest loot container within pickup range
    let mut nearest_container_entity: Option<Entity> = None;
    let mut nearest_distance = f32::MAX;

    for (container_entity, container_pos, _) in loot_containers.iter() {
        let distance = player_pos.0.distance(container_pos.0);
        if distance <= PICKUP_RANGE && distance < nearest_distance {
            nearest_distance = distance;
            nearest_container_entity = Some(container_entity);
        }
    }

    let Some(container_entity) = nearest_container_entity else {
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message: "No loot nearby!".to_string(),
                notification_type: NotificationType::Warning,
            },
        });
        return;
    };

    // Now get mutable access to the container
    let Ok((_, _, mut container)) = loot_containers.get_mut(container_entity) else {
        return;
    };

    // Loot all items from the container
    let mut total_gold = 0u32;
    let mut looted_items: Vec<(String, u32)> = Vec::new();
    let mut inventory_full = false;

    // Process all contents
    while !container.contents.is_empty() && !inventory_full {
        let loot_item = container.contents.remove(0);

        match loot_item {
            LootContents::Gold(amount) => {
                total_gold += amount;
            }
            LootContents::Item(item_stack) => {
                if inventory.add_item(item_stack.clone()) {
                    if let Some(item_def) = item_db.items.get(&item_stack.item_id) {
                        looted_items.push((item_def.name.clone(), item_stack.quantity));
                    }
                } else {
                    // Inventory full, put item back and stop looting
                    container.contents.insert(0, LootContents::Item(item_stack));
                    inventory_full = true;
                }
            }
        }
    }

    // Apply gold
    if total_gold > 0 {
        player_gold.0 += total_gold;
    }

    // Build notification message
    let mut message_parts = Vec::new();
    if total_gold > 0 {
        message_parts.push(format!("{} gold", total_gold));
    }
    for (item_name, quantity) in looted_items {
        if quantity > 1 {
            message_parts.push(format!("{} (x{})", item_name, quantity));
        } else {
            message_parts.push(item_name);
        }
    }

    if !message_parts.is_empty() {
        let message = if inventory_full {
            format!("Looted: {} (Inventory full!)", message_parts.join(", "))
        } else {
            format!("Looted: {}", message_parts.join(", "))
        };

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message,
                notification_type: if inventory_full {
                    NotificationType::Warning
                } else {
                    NotificationType::Info
                },
            },
        });
    }

    // If container is empty, despawn it
    if container.contents.is_empty() {
        commands.entity(container_entity).despawn();
        info!("Loot container auto-looted and despawned");
    } else {
        info!("Loot container partially looted (inventory full)");
    }
}
