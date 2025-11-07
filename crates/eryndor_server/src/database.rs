use bevy::prelude::*;
use sqlx::{SqlitePool, Row};
use eryndor_shared::*;

#[derive(Resource)]
pub struct DatabaseConnection {
    pool: Option<SqlitePool>,
}

impl Default for DatabaseConnection {
    fn default() -> Self {
        Self { pool: None }
    }
}

impl DatabaseConnection {
    pub fn pool(&self) -> Option<&SqlitePool> {
        self.pool.as_ref()
    }
}

pub fn setup_database(mut db_res: ResMut<DatabaseConnection>) {
    // Create database connection in blocking context
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let pool = runtime.block_on(async {
        // Create database file with explicit create flag
        let pool = SqlitePool::connect("sqlite:eryndor.db?mode=rwc")
            .await
            .expect("Failed to connect to database");

        // Create tables
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create accounts table");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS characters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL,
                name TEXT UNIQUE NOT NULL,
                class INTEGER NOT NULL,
                level INTEGER NOT NULL DEFAULT 1,
                position_x REAL NOT NULL DEFAULT 0.0,
                position_y REAL NOT NULL DEFAULT 0.0,
                health REAL NOT NULL DEFAULT 100.0,
                mana REAL NOT NULL DEFAULT 100.0,
                FOREIGN KEY (account_id) REFERENCES accounts(id)
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create characters table");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS character_inventory (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                character_id INTEGER NOT NULL,
                slot_index INTEGER NOT NULL,
                item_id INTEGER NOT NULL,
                quantity INTEGER NOT NULL DEFAULT 1,
                FOREIGN KEY (character_id) REFERENCES characters(id),
                UNIQUE(character_id, slot_index)
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create inventory table");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS character_quests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                character_id INTEGER NOT NULL,
                quest_id INTEGER NOT NULL,
                completed INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (character_id) REFERENCES characters(id),
                UNIQUE(character_id, quest_id)
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create quests table");

        // Equipment table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS character_equipment (
                character_id INTEGER PRIMARY KEY,
                weapon INTEGER,
                helmet INTEGER,
                chest INTEGER,
                legs INTEGER,
                boots INTEGER,
                FOREIGN KEY (character_id) REFERENCES characters(id)
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create equipment table");

        // Learned abilities table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS character_learned_abilities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                character_id INTEGER NOT NULL,
                ability_id INTEGER NOT NULL,
                FOREIGN KEY (character_id) REFERENCES characters(id),
                UNIQUE(character_id, ability_id)
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create learned abilities table");

        // Hotbar table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS character_hotbar (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                character_id INTEGER NOT NULL,
                slot_index INTEGER NOT NULL,
                slot_type INTEGER NOT NULL,
                slot_item_id INTEGER NOT NULL,
                FOREIGN KEY (character_id) REFERENCES characters(id),
                UNIQUE(character_id, slot_index)
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create hotbar table");

        info!("Database initialized successfully");
        pool
    });

    db_res.pool = Some(pool);
}

pub async fn create_account(pool: &SqlitePool, username: &str, password_hash: &str) -> Result<i64, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let result = sqlx::query(
        "INSERT INTO accounts (username, password_hash, created_at) VALUES (?1, ?2, ?3)"
    )
    .bind(username)
    .bind(password_hash)
    .bind(now)
    .execute(pool)
    .await;

    match result {
        Ok(result) => Ok(result.last_insert_rowid()),
        Err(e) => Err(format!("Failed to create account: {}", e)),
    }
}

pub async fn verify_credentials(pool: &SqlitePool, username: &str, password: &str) -> Result<i64, String> {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };

    // Get the stored password hash for this username
    let result = sqlx::query(
        "SELECT id, password_hash FROM accounts WHERE username = ?1"
    )
    .bind(username)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let account_id: i64 = row.get(0);
            let stored_hash: String = row.get(1);

            // Parse the stored hash (which includes the salt)
            let parsed_hash = PasswordHash::new(&stored_hash)
                .map_err(|e| format!("Failed to parse stored hash: {}", e))?;

            // Verify the password
            let argon2 = Argon2::default();
            argon2.verify_password(password.as_bytes(), &parsed_hash)
                .map_err(|_| "Invalid credentials".to_string())?;

            Ok(account_id)
        }
        Ok(None) => Err("Invalid credentials".to_string()),
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

pub async fn get_characters(pool: &SqlitePool, account_id: i64) -> Result<Vec<CharacterData>, String> {
    let result = sqlx::query(
        "SELECT id, name, class, level FROM characters WHERE account_id = ?1"
    )
    .bind(account_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(rows) => {
            let characters = rows.iter().map(|row| {
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
            }).collect();

            Ok(characters)
        }
        Err(e) => Err(format!("Failed to get characters: {}", e)),
    }
}

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

    let result = sqlx::query(
        "INSERT INTO characters (account_id, name, class) VALUES (?1, ?2, ?3)"
    )
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

pub async fn load_character(pool: &SqlitePool, character_id: i64) -> Result<(Character, Position, Health, Mana), String> {
    let result = sqlx::query(
        "SELECT name, class, level, position_x, position_y, health, mana
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

            Ok((character, position, health, mana))
        }
        Ok(None) => Err("Character not found".to_string()),
        Err(e) => Err(format!("Failed to load character: {}", e)),
    }
}

pub async fn save_character(
    pool: &SqlitePool,
    character_id: i64,
    position: &Position,
    health: &Health,
    mana: &Mana,
) -> Result<(), String> {
    let result = sqlx::query(
        "UPDATE characters SET position_x = ?1, position_y = ?2, health = ?3, mana = ?4
         WHERE id = ?5"
    )
    .bind(position.0.x)
    .bind(position.0.y)
    .bind(health.current)
    .bind(mana.current)
    .bind(character_id)
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to save character: {}", e)),
    }
}

// ============================================================================
// EQUIPMENT PERSISTENCE
// ============================================================================

pub async fn load_equipment(pool: &SqlitePool, character_id: i64) -> Result<Equipment, String> {
    let result = sqlx::query(
        "SELECT weapon, helmet, chest, legs, boots FROM character_equipment WHERE character_id = ?1"
    )
    .bind(character_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(row)) => {
            Ok(Equipment {
                weapon: row.get(0),
                helmet: row.get(1),
                chest: row.get(2),
                legs: row.get(3),
                boots: row.get(4),
            })
        }
        Ok(None) => {
            // No equipment row exists, return empty equipment
            Ok(Equipment::default())
        }
        Err(e) => Err(format!("Failed to load equipment: {}", e)),
    }
}

pub async fn save_equipment(pool: &SqlitePool, character_id: i64, equipment: &Equipment) -> Result<(), String> {
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
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to save equipment: {}", e)),
    }
}

// ============================================================================
// INVENTORY PERSISTENCE
// ============================================================================

pub async fn load_inventory(pool: &SqlitePool, character_id: i64) -> Result<Inventory, String> {
    let result = sqlx::query(
        "SELECT slot_index, item_id, quantity FROM character_inventory WHERE character_id = ?1 ORDER BY slot_index"
    )
    .bind(character_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(rows) => {
            let mut inventory = Inventory::new(20); // Default 20 slots
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
            Ok(inventory)
        }
        Err(e) => Err(format!("Failed to load inventory: {}", e)),
    }
}

pub async fn save_inventory(pool: &SqlitePool, character_id: i64, inventory: &Inventory) -> Result<(), String> {
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

    Ok(())
}

// ============================================================================
// LEARNED ABILITIES PERSISTENCE
// ============================================================================

pub async fn load_learned_abilities(pool: &SqlitePool, character_id: i64) -> Result<LearnedAbilities, String> {
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
            Ok(abilities)
        }
        Err(e) => Err(format!("Failed to load learned abilities: {}", e)),
    }
}

pub async fn save_learned_abilities(pool: &SqlitePool, character_id: i64, abilities: &LearnedAbilities) -> Result<(), String> {
    // Delete all existing abilities for this character
    let delete_result = sqlx::query("DELETE FROM character_learned_abilities WHERE character_id = ?1")
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

// ============================================================================
// HOTBAR PERSISTENCE
// ============================================================================

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
            for row in rows {
                let slot_index: i32 = row.get(0);
                let slot_type: i32 = row.get(1);
                let slot_item_id: i32 = row.get(2);

                if slot_index >= 0 && (slot_index as usize) < hotbar.slots.len() {
                    // slot_type: 0 = Ability
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

pub async fn save_hotbar(pool: &SqlitePool, character_id: i64, hotbar: &Hotbar) -> Result<(), String> {
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
