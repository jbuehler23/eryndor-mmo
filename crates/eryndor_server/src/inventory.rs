use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use crate::auth::ActiveCharacterEntity;
use crate::game_data::ItemDatabase;

pub fn handle_pickup_item(
    trigger: On<FromClient<PickupItemRequest>>,
    mut commands: Commands,
    clients: Query<&ActiveCharacterEntity>,
    mut players: Query<(&Position, &mut Inventory, &mut LearnedAbilities, &mut Hotbar)>,
    world_items: Query<(&Position, &WorldItem)>,
    item_db: Res<ItemDatabase>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };
    let request = trigger.event();

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else { return };
    let char_entity = active_char.0;

    // Check if item exists
    let Ok((item_pos, world_item)) = world_items.get(request.item_entity) else {
        return;
    };

    // Get player data
    let Ok((player_pos, mut inventory, mut learned_abilities, mut hotbar)) =
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

                let (slot_name, equipped) = match item_def.item_type {
                    ItemType::Weapon => {
                        equipment.weapon = Some(item_id);
                        ("Weapon", true)
                    }
                    ItemType::Helmet => {
                        equipment.helmet = Some(item_id);
                        ("Helmet", true)
                    }
                    ItemType::Chest => {
                        equipment.chest = Some(item_id);
                        ("Chest", true)
                    }
                    ItemType::Legs => {
                        equipment.legs = Some(item_id);
                        ("Legs", true)
                    }
                    ItemType::Boots => {
                        equipment.boots = Some(item_id);
                        ("Boots", true)
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
