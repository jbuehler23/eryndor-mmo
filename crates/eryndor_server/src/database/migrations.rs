//! Database schema migrations.
//!
//! All table creation and schema migrations are defined here for maintainability.

use sqlx::SqlitePool;
use tracing::info;

/// Run all database migrations and schema setup.
/// This is called once at server startup.
pub async fn run_migrations(pool: &SqlitePool) {
    // Core tables
    create_accounts_table(pool).await;
    create_characters_table(pool).await;
    create_inventory_table(pool).await;
    create_quests_table(pool).await;
    create_equipment_table(pool).await;
    create_learned_abilities_table(pool).await;
    create_hotbar_table(pool).await;

    // Run account migrations
    run_account_migrations(pool).await;

    // Run character migrations
    run_character_migrations(pool).await;

    // Security tables
    create_audit_logs_table(pool).await;
    create_bans_table(pool).await;
    create_ban_appeals_table(pool).await;
    create_content_flags_table(pool).await;
    create_admin_actions_table(pool).await;
    create_rate_limit_violations_table(pool).await;
    create_active_sessions_table(pool).await;

    // One-time cleanup migrations
    cleanup_guest_accounts(pool).await;

    info!("Database initialized successfully with security tables");
}

// =============================================================================
// Core Tables
// =============================================================================

async fn create_accounts_table(pool: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )"
    )
    .execute(pool)
    .await
    .expect("Failed to create accounts table");
}

async fn create_characters_table(pool: &SqlitePool) {
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
    .execute(pool)
    .await
    .expect("Failed to create characters table");
}

async fn create_inventory_table(pool: &SqlitePool) {
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
    .execute(pool)
    .await
    .expect("Failed to create inventory table");
}

async fn create_quests_table(pool: &SqlitePool) {
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
    .execute(pool)
    .await
    .expect("Failed to create quests table");

    // Migration: Add progress column if it doesn't exist
    let _ = sqlx::query("ALTER TABLE character_quests ADD COLUMN progress TEXT")
        .execute(pool)
        .await;
}

async fn create_equipment_table(pool: &SqlitePool) {
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
    .execute(pool)
    .await
    .expect("Failed to create equipment table");
}

async fn create_learned_abilities_table(pool: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS character_learned_abilities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            character_id INTEGER NOT NULL,
            ability_id INTEGER NOT NULL,
            FOREIGN KEY (character_id) REFERENCES characters(id),
            UNIQUE(character_id, ability_id)
        )"
    )
    .execute(pool)
    .await
    .expect("Failed to create learned abilities table");
}

async fn create_hotbar_table(pool: &SqlitePool) {
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
    .execute(pool)
    .await
    .expect("Failed to create hotbar table");
}

// =============================================================================
// Account Migrations
// =============================================================================

async fn run_account_migrations(pool: &SqlitePool) {
    // Add new columns to accounts table
    let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN email TEXT UNIQUE").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN email_verified BOOLEAN DEFAULT FALSE").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN account_type TEXT DEFAULT 'registered'").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN is_admin BOOLEAN DEFAULT FALSE").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN guest_token TEXT UNIQUE").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN guest_created_at INTEGER").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN guest_expires_at INTEGER").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN last_login_at INTEGER").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN last_login_ip TEXT").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN oauth_provider TEXT").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN oauth_id TEXT").execute(pool).await;

    // Create indexes
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_accounts_email ON accounts(email)").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_accounts_guest_token ON accounts(guest_token)").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_accounts_type ON accounts(account_type)").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_accounts_oauth_id ON accounts(oauth_id)").execute(pool).await;
}

// =============================================================================
// Character Migrations
// =============================================================================

async fn run_character_migrations(pool: &SqlitePool) {
    // Gold
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN gold INTEGER NOT NULL DEFAULT 0").execute(pool).await;

    // Progression columns
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN current_xp INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_prof_sword INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_prof_dagger INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_prof_staff INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_prof_mace INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_prof_bow INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_prof_axe INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_exp_sword INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_exp_dagger INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_exp_staff INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_exp_mace INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_exp_bow INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN weapon_exp_axe INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN armor_prof_light INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN armor_prof_medium INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN armor_prof_heavy INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN armor_exp_light INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN armor_exp_medium INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN armor_exp_heavy INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE characters ADD COLUMN unlocked_armor_passives TEXT").execute(pool).await;
}

// =============================================================================
// Security Tables
// =============================================================================

async fn create_audit_logs_table(pool: &SqlitePool) {
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
    ).execute(pool).await;

    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp)").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_action_type ON audit_logs(action_type)").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_actor ON audit_logs(actor_account_id)").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_logs_target ON audit_logs(target_account_id)").execute(pool).await;
}

async fn create_bans_table(pool: &SqlitePool) {
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
    .execute(pool)
    .await
    .expect("Failed to create bans table");

    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_bans_target ON bans(target, is_active)").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_bans_account ON bans(account_id, is_active)").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_bans_expiry ON bans(expires_at)").execute(pool).await;
}

async fn create_ban_appeals_table(pool: &SqlitePool) {
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
    .execute(pool)
    .await
    .expect("Failed to create ban appeals table");
}

async fn create_content_flags_table(pool: &SqlitePool) {
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
    .execute(pool)
    .await
    .expect("Failed to create content flags table");

    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_content_flags_status ON content_flags(status)").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_content_flags_account ON content_flags(account_id)").execute(pool).await;
}

async fn create_admin_actions_table(pool: &SqlitePool) {
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
    .execute(pool)
    .await
    .expect("Failed to create admin actions table");

    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_admin_actions_admin ON admin_actions(admin_id)").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_admin_actions_target ON admin_actions(target_id)").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_admin_actions_created ON admin_actions(created_at)").execute(pool).await;
}

async fn create_rate_limit_violations_table(pool: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS rate_limit_violations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            identifier TEXT NOT NULL,
            violation_type TEXT NOT NULL,
            violated_at INTEGER NOT NULL,
            details TEXT
        )"
    )
    .execute(pool)
    .await
    .expect("Failed to create rate limit violations table");

    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_rate_limit_identifier ON rate_limit_violations(identifier, violation_type)").execute(pool).await;
}

async fn create_active_sessions_table(pool: &SqlitePool) {
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
    .execute(pool)
    .await
    .expect("Failed to create active sessions table");

    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_account ON active_sessions(account_id)").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_token ON active_sessions(session_token)").execute(pool).await;
}

// =============================================================================
// One-time Cleanup Migrations
// =============================================================================

async fn cleanup_guest_accounts(pool: &SqlitePool) {
    let result = sqlx::query("DELETE FROM accounts WHERE account_type = 'guest'")
        .execute(pool)
        .await;
    if let Ok(rows_affected) = result {
        if rows_affected.rows_affected() > 0 {
            info!("Deleted {} guest accounts during migration", rows_affected.rows_affected());
        }
    }
}
