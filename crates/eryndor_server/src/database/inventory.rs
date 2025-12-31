//! Inventory-related database operations.
//!
//! Handles inventory, equipment, learned abilities, and hotbar persistence.

use sqlx::{SqlitePool, Row};
use tracing::info;
use eryndor_shared::*;

// =============================================================================
// Equipment
// =============================================================================

/// Load equipment for a character
pub async fn load_equipment(pool: &SqlitePool, character_id: i64) -> Result<Equipment, String> {
    let result = sqlx::query(
        "SELECT weapon, helmet, chest, legs, boots FROM character_equipment WHERE character_id = ?1"
    )
    .bind(character_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let equipment = Equipment {
                weapon: row.get(0),
                helmet: row.get(1),
                chest: row.get(2),
                legs: row.get(3),
                boots: row.get(4),
            };
            info!(
                "Loaded equipment for character {}: weapon={:?}, helmet={:?}, chest={:?}, legs={:?}, boots={:?}",
                character_id, equipment.weapon, equipment.helmet, equipment.chest, equipment.legs, equipment.boots
            );
            Ok(equipment)
        }
        Ok(None) => {
            info!("No equipment found for character {} - using default", character_id);
            Ok(Equipment::default())
        }
        Err(e) => Err(format!("Failed to load equipment: {}", e)),
    }
}

/// Save equipment for a character
pub async fn save_equipment(
    pool: &SqlitePool,
    character_id: i64,
    equipment: &Equipment,
) -> Result<(), String> {
    info!(
        "Saving equipment for character {}: weapon={:?}, helmet={:?}, chest={:?}, legs={:?}, boots={:?}",
        character_id, equipment.weapon, equipment.helmet, equipment.chest, equipment.legs, equipment.boots
    );

    let result = sqlx::query(
        "INSERT INTO character_equipment (character_id, weapon, helmet, chest, legs, boots)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(character_id) DO UPDATE SET
            weapon = ?2, helmet = ?3, chest = ?4, legs = ?5, boots = ?6"
    )
    .bind(character_id)
    .bind(equipment.weapon)
    .bind(equipment.helmet)
    .bind(equipment.chest)
    .bind(equipment.legs)
    .bind(equipment.boots)
    .execute(pool)
    .await;

    match result {
        Ok(_) => {
            info!("Successfully saved equipment for character {}", character_id);
            Ok(())
        }
        Err(e) => Err(format!("Failed to save equipment: {}", e)),
    }
}

// =============================================================================
// Inventory
// =============================================================================

/// Load inventory for a character
pub async fn load_inventory(pool: &SqlitePool, character_id: i64) -> Result<Inventory, String> {
    let result = sqlx::query(
        "SELECT slot_index, item_id, quantity FROM character_inventory WHERE character_id = ?1 ORDER BY slot_index"
    )
    .bind(character_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(rows) => {
            let mut inventory = Inventory::new(20);
            let item_count = rows.len();

            for row in rows {
                let slot_index: i32 = row.get(0);
                let item_id: i32 = row.get(1);
                let quantity: i32 = row.get(2);

                if slot_index >= 0 && (slot_index as usize) < inventory.slots.len() {
                    inventory.slots[slot_index as usize] = Some(ItemStack {
                        item_id: item_id as u32,
                        quantity: quantity as u32,
                    });
                }
            }

            info!("Loaded {} items from inventory for character {}", item_count, character_id);
            Ok(inventory)
        }
        Err(e) => Err(format!("Failed to load inventory: {}", e)),
    }
}

/// Save inventory for a character
pub async fn save_inventory(
    pool: &SqlitePool,
    character_id: i64,
    inventory: &Inventory,
) -> Result<(), String> {
    let item_count = inventory.slots.iter().filter(|s| s.is_some()).count();
    info!("Saving {} items to inventory for character {}", item_count, character_id);

    // Delete all existing inventory items for this character
    let delete_result = sqlx::query("DELETE FROM character_inventory WHERE character_id = ?1")
        .bind(character_id)
        .execute(pool)
        .await;

    if let Err(e) = delete_result {
        return Err(format!("Failed to clear inventory: {}", e));
    }

    // Insert all current inventory items
    for (slot_index, item_slot) in inventory.slots.iter().enumerate() {
        if let Some(item_stack) = item_slot {
            let insert_result = sqlx::query(
                "INSERT INTO character_inventory (character_id, slot_index, item_id, quantity)
                 VALUES (?1, ?2, ?3, ?4)"
            )
            .bind(character_id)
            .bind(slot_index as i32)
            .bind(item_stack.item_id as i32)
            .bind(item_stack.quantity as i32)
            .execute(pool)
            .await;

            if let Err(e) = insert_result {
                return Err(format!("Failed to save inventory slot {}: {}", slot_index, e));
            }
        }
    }

    info!("Successfully saved inventory for character {}", character_id);
    Ok(())
}

// =============================================================================
// Learned Abilities
// =============================================================================

/// Load learned abilities for a character
pub async fn load_learned_abilities(
    pool: &SqlitePool,
    character_id: i64,
) -> Result<LearnedAbilities, String> {
    let result = sqlx::query(
        "SELECT ability_id FROM character_learned_abilities WHERE character_id = ?1"
    )
    .bind(character_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(rows) => {
            let mut abilities = LearnedAbilities::default();
            for row in rows {
                let ability_id: i32 = row.get(0);
                abilities.learn(ability_id as u32);
            }
            info!(
                "Loaded {} learned abilities for character {}",
                abilities.abilities.len(),
                character_id
            );
            Ok(abilities)
        }
        Err(e) => Err(format!("Failed to load learned abilities: {}", e)),
    }
}

/// Save learned abilities for a character
pub async fn save_learned_abilities(
    pool: &SqlitePool,
    character_id: i64,
    abilities: &LearnedAbilities,
) -> Result<(), String> {
    // Delete all existing abilities for this character
    let delete_result =
        sqlx::query("DELETE FROM character_learned_abilities WHERE character_id = ?1")
            .bind(character_id)
            .execute(pool)
            .await;

    if let Err(e) = delete_result {
        return Err(format!("Failed to clear learned abilities: {}", e));
    }

    // Insert all current abilities
    for &ability_id in &abilities.abilities {
        let insert_result = sqlx::query(
            "INSERT INTO character_learned_abilities (character_id, ability_id)
             VALUES (?1, ?2)"
        )
        .bind(character_id)
        .bind(ability_id as i32)
        .execute(pool)
        .await;

        if let Err(e) = insert_result {
            return Err(format!("Failed to save ability {}: {}", ability_id, e));
        }
    }

    Ok(())
}

// =============================================================================
// Hotbar
// =============================================================================

/// Load hotbar for a character
pub async fn load_hotbar(pool: &SqlitePool, character_id: i64) -> Result<Hotbar, String> {
    let result = sqlx::query(
        "SELECT slot_index, slot_type, slot_item_id FROM character_hotbar WHERE character_id = ?1"
    )
    .bind(character_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(rows) => {
            let mut hotbar = Hotbar::default();
            info!("Loading hotbar for character {}, found {} rows", character_id, rows.len());

            for row in rows {
                let slot_index: i32 = row.get(0);
                let slot_type: i32 = row.get(1);
                let slot_item_id: i32 = row.get(2);

                info!("Hotbar slot {}: type={}, item_id={}", slot_index, slot_type, slot_item_id);

                if slot_index >= 0 && (slot_index as usize) < hotbar.slots.len() {
                    let hotbar_slot = match slot_type {
                        0 => Some(HotbarSlot::Ability(slot_item_id as u32)),
                        _ => None,
                    };
                    hotbar.slots[slot_index as usize] = hotbar_slot;
                }
            }
            Ok(hotbar)
        }
        Err(e) => Err(format!("Failed to load hotbar: {}", e)),
    }
}

/// Save hotbar for a character
pub async fn save_hotbar(
    pool: &SqlitePool,
    character_id: i64,
    hotbar: &Hotbar,
) -> Result<(), String> {
    // Delete all existing hotbar slots for this character
    let delete_result = sqlx::query("DELETE FROM character_hotbar WHERE character_id = ?1")
        .bind(character_id)
        .execute(pool)
        .await;

    if let Err(e) = delete_result {
        return Err(format!("Failed to clear hotbar: {}", e));
    }

    // Insert all current hotbar slots
    for (slot_index, hotbar_slot) in hotbar.slots.iter().enumerate() {
        if let Some(slot) = hotbar_slot {
            let (slot_type, slot_item_id) = match slot {
                HotbarSlot::Ability(ability_id) => (0, *ability_id),
            };

            let insert_result = sqlx::query(
                "INSERT INTO character_hotbar (character_id, slot_index, slot_type, slot_item_id)
                 VALUES (?1, ?2, ?3, ?4)"
            )
            .bind(character_id)
            .bind(slot_index as i32)
            .bind(slot_type)
            .bind(slot_item_id as i32)
            .execute(pool)
            .await;

            if let Err(e) = insert_result {
                return Err(format!("Failed to save hotbar slot {}: {}", slot_index, e));
            }
        }
    }

    Ok(())
}
