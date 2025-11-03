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
