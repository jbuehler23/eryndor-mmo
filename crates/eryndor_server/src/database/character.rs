//! Character-related database operations.
//!
//! Handles character creation, loading, and saving.

use bevy::prelude::*;
use sqlx::{SqlitePool, Row};
use eryndor_shared::*;

/// Get all characters for an account
pub async fn get_characters(pool: &SqlitePool, account_id: i64) -> Result<Vec<CharacterData>, String> {
    let result = sqlx::query("SELECT id, name, class, level FROM characters WHERE account_id = ?1")
        .bind(account_id)
        .fetch_all(pool)
        .await;

    match result {
        Ok(rows) => {
            let characters = rows
                .iter()
                .map(|row| {
                    let class_id: i32 = row.get(2);
                    let class = match class_id {
                        0 => CharacterClass::Rogue,
                        1 => CharacterClass::Mage,
                        2 => CharacterClass::Knight,
                        _ => CharacterClass::Rogue,
                    };

                    CharacterData {
                        id: row.get(0),
                        name: row.get(1),
                        class,
                        level: row.get::<i32, _>(3) as u32,
                    }
                })
                .collect();

            Ok(characters)
        }
        Err(e) => Err(format!("Failed to get characters: {}", e)),
    }
}

/// Create a new character for an account
pub async fn create_character(
    pool: &SqlitePool,
    account_id: i64,
    name: &str,
    class: CharacterClass,
) -> Result<CharacterData, String> {
    let class_id = match class {
        CharacterClass::Rogue => 0,
        CharacterClass::Mage => 1,
        CharacterClass::Knight => 2,
    };

    let result = sqlx::query("INSERT INTO characters (account_id, name, class) VALUES (?1, ?2, ?3)")
        .bind(account_id)
        .bind(name)
        .bind(class_id)
        .execute(pool)
        .await;

    match result {
        Ok(result) => {
            let character_id = result.last_insert_rowid();
            Ok(CharacterData {
                id: character_id,
                name: name.to_string(),
                class,
                level: 1,
            })
        }
        Err(e) => Err(format!("Failed to create character: {}", e)),
    }
}

/// Load character data from the database
pub async fn load_character(
    pool: &SqlitePool,
    character_id: i64,
) -> Result<(Character, Position, Health, Mana, Gold), String> {
    let result = sqlx::query(
        "SELECT name, class, level, position_x, position_y, health, mana, gold
         FROM characters WHERE id = ?1"
    )
    .bind(character_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let class_id: i32 = row.get(1);
            let class = match class_id {
                0 => CharacterClass::Rogue,
                1 => CharacterClass::Mage,
                2 => CharacterClass::Knight,
                _ => CharacterClass::Rogue,
            };

            let character = Character {
                name: row.get(0),
                class,
                level: row.get::<i32, _>(2) as u32,
            };

            let position = Position(Vec2::new(row.get(3), row.get(4)));
            let health = Health {
                current: row.get(5),
                max: 100.0,
            };
            let mana = Mana {
                current: row.get(6),
                max: 100.0,
            };
            let gold = Gold(row.try_get::<i32, _>(7).unwrap_or(0) as u32);

            Ok((character, position, health, mana, gold))
        }
        Ok(None) => Err("Character not found".to_string()),
        Err(e) => Err(format!("Failed to load character: {}", e)),
    }
}

/// Save character position, health, mana, and gold
pub async fn save_character(
    pool: &SqlitePool,
    character_id: i64,
    position: &Position,
    health: &Health,
    mana: &Mana,
    gold: &Gold,
) -> Result<(), String> {
    let result = sqlx::query(
        "UPDATE characters SET position_x = ?1, position_y = ?2, health = ?3, mana = ?4, gold = ?5
         WHERE id = ?6"
    )
    .bind(position.0.x)
    .bind(position.0.y)
    .bind(health.current)
    .bind(mana.current)
    .bind(gold.0 as i32)
    .bind(character_id)
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to save character: {}", e)),
    }
}
