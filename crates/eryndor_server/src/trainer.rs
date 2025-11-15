use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use crate::auth::ActiveCharacterEntity;
use crate::game_data::ItemDatabase;

pub fn handle_purchase_from_trainer(
    trigger: On<FromClient<PurchaseFromTrainerRequest>>,
    mut commands: Commands,
    clients: Query<&ActiveCharacterEntity>,
    mut players: Query<(&Position, &mut Gold, &mut Inventory)>,
    trainers: Query<(Entity, &Position, &Trainer, &NpcName), With<Npc>>,
    item_db: Res<ItemDatabase>,
) {
    info!("=== PURCHASE FROM TRAINER HANDLER CALLED ===");
    let Some(client_entity) = trigger.client_id.entity() else {
        info!("No client entity found");
        return;
    };
    let request = trigger.event();
    info!("Purchase request - Item ID: {}", request.item_id);

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else {
        info!("Failed to get active character for client {:?}", client_entity);
        return;
    };
    let char_entity = active_char.0;
    info!("Character entity: {:?}", char_entity);

    // Get player data
    let Ok((player_pos, mut gold, mut inventory)) = players.get_mut(char_entity) else {
        info!("Failed to get player data for character {:?}", char_entity);
        return;
    };
    info!("Player position: {:?}, Gold: {}", player_pos.0, gold.0);

    // Find the closest trainer to the player within interaction range
    let mut closest_trainer: Option<(Entity, f32, &Trainer, &NpcName)> = None;
    for (entity, trainer_pos, trainer, npc_name) in trainers.iter() {
        let distance = player_pos.0.distance(trainer_pos.0);
        info!("Found Trainer '{}' at {:?}, distance: {:.2}", npc_name.0, trainer_pos.0, distance);

        if distance <= INTERACTION_RANGE {
            if let Some((_, closest_dist, _, _)) = closest_trainer {
                if distance < closest_dist {
                    closest_trainer = Some((entity, distance, trainer, npc_name));
                }
            } else {
                closest_trainer = Some((entity, distance, trainer, npc_name));
            }
        }
    }

    let Some((trainer_entity, distance, trainer, npc_name)) = closest_trainer else {
        info!("No trainer found within interaction range");
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message: "No trainer nearby!".to_string(),
                notification_type: NotificationType::Warning,
            },
        });
        return;
    };

    info!("Found closest trainer: {} at distance {:.2}", npc_name.0, distance);

    // Find the item in trainer's inventory
    let item_for_sale = trainer.items_for_sale.iter().find(|item| item.item_id == request.item_id);

    let Some(trainer_item) = item_for_sale else {
        info!("Item {} not found in trainer's inventory", request.item_id);
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message: "This item is not available from this trainer!".to_string(),
                notification_type: NotificationType::Warning,
            },
        });
        return;
    };

    info!("Found item in trainer inventory - Cost: {} gold", trainer_item.cost);

    // Get item definition for name
    let Some(item_def) = item_db.items.get(&request.item_id) else {
        error!("Item definition not found for item ID {}", request.item_id);
        return;
    };

    // Check if player has enough gold
    if gold.0 < trainer_item.cost {
        info!("Player doesn't have enough gold. Has: {}, Needs: {}", gold.0, trainer_item.cost);
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message: format!("Not enough gold! Need {} gold.", trainer_item.cost),
                notification_type: NotificationType::Warning,
            },
        });
        return;
    }

    // All checks passed - execute purchase
    // Deduct gold
    gold.0 -= trainer_item.cost;
    info!("Deducted {} gold. Player now has {} gold", trainer_item.cost, gold.0);

    // Add item to inventory
    if inventory.add_item(ItemStack {
        item_id: request.item_id,
        quantity: 1,
    }) {
        info!("Added {} to player inventory", item_def.name);

        // Send success notification
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message: format!("Purchased {} for {} gold!", item_def.name, trainer_item.cost),
                notification_type: NotificationType::Success,
            },
        });

        info!("Purchase complete: {} bought {} for {} gold", char_entity, item_def.name, trainer_item.cost);
    } else {
        info!("Player inventory is full, refunding purchase");
        // Refund the gold since inventory was full
        gold.0 += trainer_item.cost;

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: NotificationEvent {
                message: "Inventory is full!".to_string(),
                notification_type: NotificationType::Warning,
            },
        });
    }
}
