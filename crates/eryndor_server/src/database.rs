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
        // Support DATABASE_PATH environment variable, default to "eryndor.db"
        let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "eryndor.db".to_string());
        let connection_string = format!("sqlite:{}?mode=rwc", db_path);

        eprintln!("Connecting to database: {}", connection_string);

        // Create database file with explicit create flag
        let pool = SqlitePool::connect(&connection_string)
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
                progress TEXT,
                FOREIGN KEY (character_id) REFERENCES characters(id),
                UNIQUE(character_id, quest_id)
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create quests table");

        // Migration: Add progress column to existing character_quests table if it doesn't exist
        // This handles databases created before the progress column was added
        let _ = sqlx::query(
            "ALTER TABLE character_quests ADD COLUMN progress TEXT"
        )
        .execute(&pool)
        .await;
        // Ignore error if column already exists

        // Migration: Update accounts table for new security features
        let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN email TEXT UNIQUE").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN email_verified BOOLEAN DEFAULT FALSE").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN account_type TEXT DEFAULT 'registered'").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN is_admin BOOLEAN DEFAULT FALSE").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN guest_token TEXT UNIQUE").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN guest_created_at INTEGER").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN guest_expires_at INTEGER").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN last_login_at INTEGER").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN last_login_ip TEXT").execute(&pool).await;

        // OAuth columns
        let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN oauth_provider TEXT").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN oauth_id TEXT").execute(&pool).await;
        // Ignore errors if columns already exist

        // Create indexes for accounts table
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_accounts_email ON accounts(email)").execute(&pool).await;
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_accounts_guest_token ON accounts(guest_token)").execute(&pool).await;
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_accounts_type ON accounts(account_type)").execute(&pool).await;
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_accounts_oauth_id ON accounts(oauth_id)").execute(&pool).await;

        // Create audit_logs table for security event tracking
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS audit_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                action_type TEXT NOT NULL,
                actor_account_id INTEGER,
                target_account_id INTEGER,
                target_username TEXT,
                ip_address TEXT,
                details TEXT
            )"
        ).execute(&pool).await;

        // Create indexes for audit_logs table (for efficient querying)
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp)").execute(&pool).await;
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_action_type ON audit_logs(action_type)").execute(&pool).await;
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_actor ON audit_logs(actor_account_id)").execute(&pool).await;
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_target ON audit_logs(target_account_id)").execute(&pool).await;

        // Delete all guest accounts (one-time migration for new auth system)
        let result = sqlx::query("DELETE FROM accounts WHERE account_type = 'guest'").execute(&pool).await;
        if let Ok(rows_affected) = result {
            if rows_affected.rows_affected() > 0 {
                info!("Deleted {} guest accounts during migration", rows_affected.rows_affected());
            }
        }

        // Migration: Add progression columns to characters table
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN current_xp INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_prof_sword INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_prof_dagger INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_prof_staff INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_prof_mace INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_prof_bow INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_prof_axe INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_exp_sword INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_exp_dagger INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_exp_staff INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_exp_mace INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_exp_bow INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_exp_axe INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN armor_prof_light INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN armor_prof_medium INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN armor_prof_heavy INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN armor_exp_light INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN armor_exp_medium INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN armor_exp_heavy INTEGER NOT NULL DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE characters ADD COLUMN unlocked_armor_passives TEXT").execute(&pool).await;
        // Ignore errors if columns already exist

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

        // ============================================================================
        // SECURITY TABLES
        // ============================================================================

        // Bans table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS bans (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ban_type TEXT NOT NULL,
                target TEXT NOT NULL,
                account_id INTEGER,
                reason TEXT NOT NULL,
                banned_by INTEGER,
                banned_at INTEGER NOT NULL,
                expires_at INTEGER,
                is_active BOOLEAN DEFAULT TRUE,
                notes TEXT,
                FOREIGN KEY(account_id) REFERENCES accounts(id),
                FOREIGN KEY(banned_by) REFERENCES accounts(id)
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create bans table");

        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_bans_target ON bans(target, is_active)").execute(&pool).await;
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_bans_account ON bans(account_id, is_active)").execute(&pool).await;
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_bans_expiry ON bans(expires_at)").execute(&pool).await;

        // Ban appeals table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS ban_appeals (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ban_id INTEGER NOT NULL,
                appeal_text TEXT NOT NULL,
                submitted_at INTEGER NOT NULL,
                status TEXT DEFAULT 'pending',
                reviewed_by INTEGER,
                reviewed_at INTEGER,
                review_notes TEXT,
                FOREIGN KEY(ban_id) REFERENCES bans(id),
                FOREIGN KEY(reviewed_by) REFERENCES accounts(id)
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create ban appeals table");

        // Content flags table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS content_flags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL,
                content_type TEXT NOT NULL,
                content TEXT NOT NULL,
                toxicity_score REAL,
                flagged_at INTEGER NOT NULL,
                status TEXT DEFAULT 'pending',
                reviewed_by INTEGER,
                reviewed_at INTEGER,
                FOREIGN KEY(account_id) REFERENCES accounts(id),
                FOREIGN KEY(reviewed_by) REFERENCES accounts(id)
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create content flags table");

        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_content_flags_status ON content_flags(status)").execute(&pool).await;
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_content_flags_account ON content_flags(account_id)").execute(&pool).await;

        // Admin actions audit log
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS admin_actions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                admin_id INTEGER NOT NULL,
                action_type TEXT NOT NULL,
                target_id INTEGER,
                details TEXT,
                ip_address TEXT,
                created_at INTEGER NOT NULL,
                FOREIGN KEY(admin_id) REFERENCES accounts(id),
                FOREIGN KEY(target_id) REFERENCES accounts(id)
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create admin actions table");

        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_admin_actions_admin ON admin_actions(admin_id)").execute(&pool).await;
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_admin_actions_target ON admin_actions(target_id)").execute(&pool).await;
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_admin_actions_created ON admin_actions(created_at)").execute(&pool).await;

        // Rate limit violations table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS rate_limit_violations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                identifier TEXT NOT NULL,
                violation_type TEXT NOT NULL,
                violated_at INTEGER NOT NULL,
                details TEXT
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create rate limit violations table");

        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_rate_limit_identifier ON rate_limit_violations(identifier, violation_type)").execute(&pool).await;

        // Active sessions table (for security)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS active_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL,
                session_token TEXT UNIQUE NOT NULL,
                ip_address TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                last_activity INTEGER NOT NULL,
                FOREIGN KEY(account_id) REFERENCES accounts(id)
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create active sessions table");

        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_account ON active_sessions(account_id)").execute(&pool).await;
        let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_token ON active_sessions(session_token)").execute(&pool).await;

        info!("Database initialized successfully with security tables");
        pool
    });

    db_res.pool = Some(pool);
}

/// Check if an email already exists in the database
pub async fn email_exists(pool: &SqlitePool, email: &str) -> Result<bool, String> {
    let result = sqlx::query("SELECT 1 FROM accounts WHERE email = ?1")
        .bind(email)
        .fetch_optional(pool)
        .await;

    match result {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(e) => Err(format!("Database error checking email: {}", e)),
    }
}

/// Check if a username already exists in the database
pub async fn username_exists(pool: &SqlitePool, username: &str) -> Result<bool, String> {
    let result = sqlx::query("SELECT 1 FROM accounts WHERE username = ?1")
        .bind(username)
        .fetch_optional(pool)
        .await;

    match result {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(e) => Err(format!("Database error checking username: {}", e)),
    }
}

pub async fn create_account(pool: &SqlitePool, email: &str, username: &str, password_hash: &str) -> Result<i64, String> {
    // Check for existing email
    match email_exists(pool, email).await {
        Ok(true) => return Err("Email already in use".to_string()),
        Ok(false) => {},
        Err(e) => return Err(e),
    }

    // Check for existing username
    match username_exists(pool, username).await {
        Ok(true) => return Err("Username already taken".to_string()),
        Ok(false) => {},
        Err(e) => return Err(e),
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let result = sqlx::query(
        "INSERT INTO accounts (email, username, password_hash, created_at, account_type) VALUES (?1, ?2, ?3, ?4, 'registered')"
    )
    .bind(email)
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

// ============================================================================
// PROGRESSION PERSISTENCE
// ============================================================================

pub async fn load_progression(
    pool: &SqlitePool,
    character_id: i64,
) -> Result<(Experience, WeaponProficiency, WeaponProficiencyExp, ArmorProficiency, ArmorProficiencyExp, UnlockedArmorPassives), String> {
    let result = sqlx::query(
        "SELECT current_xp,
                weapon_prof_sword, weapon_prof_dagger, weapon_prof_staff, weapon_prof_mace, weapon_prof_bow, weapon_prof_axe,
                weapon_exp_sword, weapon_exp_dagger, weapon_exp_staff, weapon_exp_mace, weapon_exp_bow, weapon_exp_axe,
                armor_prof_light, armor_prof_medium, armor_prof_heavy,
                armor_exp_light, armor_exp_medium, armor_exp_heavy,
                unlocked_armor_passives
         FROM characters WHERE id = ?1"
    )
    .bind(character_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(row)) => {
            // Experience
            let current_xp: i32 = row.try_get(0).unwrap_or(0);

            // Get character level for Experience initialization
            let level_result = sqlx::query("SELECT level FROM characters WHERE id = ?1")
                .bind(character_id)
                .fetch_one(pool)
                .await;

            let level = match level_result {
                Ok(row) => row.get::<i32, _>(0) as u32,
                Err(_) => 1,
            };

            let mut experience = Experience::new(level);
            experience.current_xp = current_xp as u32;

            // Weapon Proficiency
            let weapon_prof = WeaponProficiency {
                sword: row.try_get(1).unwrap_or(0),
                dagger: row.try_get(2).unwrap_or(0),
                staff: row.try_get(3).unwrap_or(0),
                wand: 1,  // Default wand proficiency (not in DB yet)
                mace: row.try_get(4).unwrap_or(0),
                bow: row.try_get(5).unwrap_or(0),
                axe: row.try_get(6).unwrap_or(0),
            };

            // Weapon Proficiency Experience
            let weapon_exp = WeaponProficiencyExp {
                sword_xp: row.try_get(7).unwrap_or(0),
                dagger_xp: row.try_get(8).unwrap_or(0),
                staff_xp: row.try_get(9).unwrap_or(0),
                wand_xp: 0,  // Default wand XP (not in DB yet)
                mace_xp: row.try_get(10).unwrap_or(0),
                bow_xp: row.try_get(11).unwrap_or(0),
                axe_xp: row.try_get(12).unwrap_or(0),
            };

            // Armor Proficiency
            let armor_prof = ArmorProficiency {
                light: row.try_get(13).unwrap_or(0),
                medium: row.try_get(14).unwrap_or(0),
                heavy: row.try_get(15).unwrap_or(0),
            };

            // Armor Proficiency Experience
            let armor_exp = ArmorProficiencyExp {
                light_xp: row.try_get(16).unwrap_or(0),
                medium_xp: row.try_get(17).unwrap_or(0),
                heavy_xp: row.try_get(18).unwrap_or(0),
            };

            // Unlocked Armor Passives
            let passives_json: Option<String> = row.try_get(19).ok().flatten();
            let unlocked_passives = if let Some(json_str) = passives_json {
                match serde_json::from_str(&json_str) {
                    Ok(passives) => passives,
                    Err(_) => UnlockedArmorPassives::default(),
                }
            } else {
                UnlockedArmorPassives::default()
            };

            Ok((experience, weapon_prof, weapon_exp, armor_prof, armor_exp, unlocked_passives))
        }
        Ok(None) => Err("Character not found".to_string()),
        Err(e) => Err(format!("Failed to load progression: {}", e)),
    }
}

pub async fn save_progression(
    pool: &SqlitePool,
    character_id: i64,
    character_level: u32,
    experience: &Experience,
    weapon_prof: &WeaponProficiency,
    weapon_exp: &WeaponProficiencyExp,
    armor_prof: &ArmorProficiency,
    armor_exp: &ArmorProficiencyExp,
    unlocked_passives: &UnlockedArmorPassives,
) -> Result<(), String> {
    let passives_json = serde_json::to_string(&unlocked_passives.passives)
        .unwrap_or_else(|_| "[]".to_string());

    let result = sqlx::query(
        "UPDATE characters SET
            level = ?1,
            current_xp = ?2,
            weapon_prof_sword = ?3, weapon_prof_dagger = ?4, weapon_prof_staff = ?5,
            weapon_prof_mace = ?6, weapon_prof_bow = ?7, weapon_prof_axe = ?8,
            weapon_exp_sword = ?9, weapon_exp_dagger = ?10, weapon_exp_staff = ?11,
            weapon_exp_mace = ?12, weapon_exp_bow = ?13, weapon_exp_axe = ?14,
            armor_prof_light = ?15, armor_prof_medium = ?16, armor_prof_heavy = ?17,
            armor_exp_light = ?18, armor_exp_medium = ?19, armor_exp_heavy = ?20,
            unlocked_armor_passives = ?21
         WHERE id = ?22"
    )
    .bind(character_level as i32)
    .bind(experience.current_xp as i32)
    .bind(weapon_prof.sword)
    .bind(weapon_prof.dagger)
    .bind(weapon_prof.staff)
    .bind(weapon_prof.mace)
    .bind(weapon_prof.bow)
    .bind(weapon_prof.axe)
    .bind(weapon_exp.sword_xp)
    .bind(weapon_exp.dagger_xp)
    .bind(weapon_exp.staff_xp)
    .bind(weapon_exp.mace_xp)
    .bind(weapon_exp.bow_xp)
    .bind(weapon_exp.axe_xp)
    .bind(armor_prof.light)
    .bind(armor_prof.medium)
    .bind(armor_prof.heavy)
    .bind(armor_exp.light_xp)
    .bind(armor_exp.medium_xp)
    .bind(armor_exp.heavy_xp)
    .bind(passives_json)
    .bind(character_id)
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to save progression: {}", e)),
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
            let equipment = Equipment {
                weapon: row.get(0),
                helmet: row.get(1),
                chest: row.get(2),
                legs: row.get(3),
                boots: row.get(4),
            };
            info!("Loaded equipment for character {}: weapon={:?}, helmet={:?}, chest={:?}, legs={:?}, boots={:?}",
                character_id, equipment.weapon, equipment.helmet, equipment.chest, equipment.legs, equipment.boots);
            Ok(equipment)
        }
        Ok(None) => {
            // No equipment row exists, return empty equipment
            info!("No equipment found for character {} - using default", character_id);
            Ok(Equipment::default())
        }
        Err(e) => Err(format!("Failed to load equipment: {}", e)),
    }
}

pub async fn save_equipment(pool: &SqlitePool, character_id: i64, equipment: &Equipment) -> Result<(), String> {
    info!("Saving equipment for character {}: weapon={:?}, helmet={:?}, chest={:?}, legs={:?}, boots={:?}",
        character_id, equipment.weapon, equipment.helmet, equipment.chest, equipment.legs, equipment.boots);

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

pub async fn save_inventory(pool: &SqlitePool, character_id: i64, inventory: &Inventory) -> Result<(), String> {
    // Count non-empty slots
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
            info!("Loaded {} learned abilities for character {}", abilities.abilities.len(), character_id);
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
            info!("Loading hotbar for character {}, found {} rows", character_id, rows.len());
            for row in rows {
                let slot_index: i32 = row.get(0);
                let slot_type: i32 = row.get(1);
                let slot_item_id: i32 = row.get(2);

                info!("Hotbar slot {}: type={}, item_id={}", slot_index, slot_type, slot_item_id);

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

// ============================================================================
// QUEST LOG PERSISTENCE
// ============================================================================

pub async fn load_quest_log(pool: &SqlitePool, character_id: i64) -> Result<QuestLog, String> {
    let result = sqlx::query("SELECT quest_id, completed, progress FROM character_quests WHERE character_id = ?1")
        .bind(character_id)
        .fetch_all(pool)
        .await;

    match result {
        Ok(rows) => {
            let mut quest_log = QuestLog::default();

            for row in rows {
                let quest_id: i32 = row.get(0);
                let completed: i32 = row.get(1);
                let progress_json: Option<String> = row.get(2);

                if completed == 1 {
                    // Quest is completed
                    quest_log.completed_quests.insert(quest_id as u32);
                } else {
                    // Quest is active - parse progress
                    let progress: Vec<u32> = if let Some(json_str) = progress_json {
                        serde_json::from_str(&json_str).unwrap_or_default()
                    } else {
                        Vec::new()
                    };

                    quest_log.active_quests.push(ActiveQuest {
                        quest_id: quest_id as u32,
                        progress,
                    });
                }
            }

            info!("Loaded {} active and {} completed quests for character {}",
                quest_log.active_quests.len(),
                quest_log.completed_quests.len(),
                character_id);
            Ok(quest_log)
        }
        Err(e) => Err(format!("Failed to load quest log: {}", e)),
    }
}

pub async fn save_quest_log(pool: &SqlitePool, character_id: i64, quest_log: &QuestLog) -> Result<(), String> {
    let total_quests = quest_log.active_quests.len() + quest_log.completed_quests.len();
    info!("Saving quest log for character {} ({} active, {} completed)",
        character_id, quest_log.active_quests.len(), quest_log.completed_quests.len());

    // Delete all existing quests for this character
    let delete_result = sqlx::query("DELETE FROM character_quests WHERE character_id = ?1")
        .bind(character_id)
        .execute(pool)
        .await;

    if let Err(e) = delete_result {
        return Err(format!("Failed to clear quest log: {}", e));
    }

    // Insert active quests
    for active_quest in &quest_log.active_quests {
        let progress_json = serde_json::to_string(&active_quest.progress)
            .unwrap_or_else(|_| "[]".to_string());

        let insert_result = sqlx::query(
            "INSERT INTO character_quests (character_id, quest_id, completed, progress)
             VALUES (?1, ?2, 0, ?3)"
        )
        .bind(character_id)
        .bind(active_quest.quest_id as i32)
        .bind(progress_json)
        .execute(pool)
        .await;

        if let Err(e) = insert_result {
            return Err(format!("Failed to save active quest {}: {}", active_quest.quest_id, e));
        }
    }

    // Insert completed quests
    for &quest_id in &quest_log.completed_quests {
        let insert_result = sqlx::query(
            "INSERT INTO character_quests (character_id, quest_id, completed, progress)
             VALUES (?1, ?2, 1, NULL)"
        )
        .bind(character_id)
        .bind(quest_id as i32)
        .execute(pool)
        .await;

        if let Err(e) = insert_result {
            return Err(format!("Failed to save completed quest {}: {}", quest_id, e));
        }
    }

    info!("Successfully saved {} quests for character {}", total_quests, character_id);
    Ok(())
}

// ============================================================================
// OAUTH ACCOUNT MANAGEMENT
// ============================================================================

/// Find account by OAuth provider and ID
pub async fn find_account_by_oauth(pool: &SqlitePool, provider: &str, oauth_id: &str) -> Result<Option<i64>, String> {
    let result = sqlx::query(
        "SELECT id FROM accounts WHERE oauth_provider = ?1 AND oauth_id = ?2"
    )
    .bind(provider)
    .bind(oauth_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let id: i64 = row.try_get("id").map_err(|e| e.to_string())?;
            Ok(Some(id))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Create a new OAuth account
pub async fn create_oauth_account(
    pool: &SqlitePool,
    email: &str,
    username: &str,
    provider: &str,
    oauth_id: &str
) -> Result<i64, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // OAuth accounts don't have passwords (use empty hash)
    let result = sqlx::query(
        "INSERT INTO accounts (email, username, password_hash, oauth_provider, oauth_id, created_at, account_type)
         VALUES (?1, ?2, '', ?3, ?4, ?5, 'registered')"
    )
    .bind(email)
    .bind(username)
    .bind(provider)
    .bind(oauth_id)
    .bind(now)
    .execute(pool)
    .await;

    match result {
        Ok(result) => Ok(result.last_insert_rowid()),
        Err(e) => Err(format!("Failed to create OAuth account: {}", e)),
    }
}

/// Log a rate limit violation to the database
pub async fn log_rate_limit_violation(
    pool: &SqlitePool,
    identifier: &str,
    violation_type: &str,
    details: &str,
) -> Result<(), String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let result = sqlx::query(
        "INSERT INTO rate_limit_violations (identifier, violation_type, violated_at, details)
         VALUES (?1, ?2, ?3, ?4)"
    )
    .bind(identifier)
    .bind(violation_type)
    .bind(now)
    .bind(details)
    .execute(pool)
    .await;

    match result {
        Ok(_) => {
            info!("Logged rate limit violation: {} - {}", identifier, violation_type);
            Ok(())
        }
        Err(e) => Err(format!("Failed to log rate limit violation: {}", e)),
    }
}

// ============================================================================
// BAN SYSTEM
// ============================================================================

/// Information about an active ban
#[derive(Debug, Clone)]
pub struct BanInfo {
    pub ban_type: String,
    pub reason: String,
    pub expires_at: Option<i64>,
    pub is_permanent: bool,
}

/// Check if an account is banned
/// Returns Ok(None) if not banned, Ok(Some(BanInfo)) if banned
pub async fn check_account_ban(
    pool: &SqlitePool,
    account_id: i64,
) -> Result<Option<BanInfo>, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let result = sqlx::query(
        "SELECT ban_type, reason, expires_at
         FROM bans
         WHERE banned_account_id = ?1
           AND is_active = TRUE
           AND (expires_at IS NULL OR expires_at > ?2)
         LIMIT 1"
    )
    .bind(account_id)
    .bind(now)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let expires_at: Option<i64> = row.get("expires_at");
            let is_permanent = expires_at.is_none();

            Ok(Some(BanInfo {
                ban_type: row.get("ban_type"),
                reason: row.get("reason"),
                expires_at,
                is_permanent,
            }))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Failed to check account ban: {}", e)),
    }
}

/// Check if an IP address is banned
/// Returns Ok(None) if not banned, Ok(Some(BanInfo)) if banned
pub async fn check_ip_ban(
    pool: &SqlitePool,
    ip_address: &str,
) -> Result<Option<BanInfo>, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let result = sqlx::query(
        "SELECT ban_type, reason, expires_at
         FROM bans
         WHERE banned_ip = ?1
           AND is_active = TRUE
           AND (expires_at IS NULL OR expires_at > ?2)
         LIMIT 1"
    )
    .bind(ip_address)
    .bind(now)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let expires_at: Option<i64> = row.get("expires_at");
            let is_permanent = expires_at.is_none();

            Ok(Some(BanInfo {
                ban_type: row.get("ban_type"),
                reason: row.get("reason"),
                expires_at,
                is_permanent,
            }))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Failed to check IP ban: {}", e)),
    }
}
