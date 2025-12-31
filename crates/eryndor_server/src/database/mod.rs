//! Database module - handles all SQLite persistence.
//!
//! ## Module Structure
//! - `migrations` - Schema setup and migrations
//! - `account` - Account creation and verification
//! - `character` - Character load/save
//! - `progression` - XP and proficiency persistence
//! - `inventory` - Inventory, equipment, abilities, hotbar
//! - `quests` - Quest log persistence
//! - `oauth` - OAuth account management
//! - `bans` - Ban system

mod migrations;
pub mod account;
pub mod character;
pub mod progression;
pub mod inventory;
pub mod quests;
pub mod oauth;
pub mod bans;

use bevy::prelude::*;
use sqlx::SqlitePool;

// Re-export commonly used items
pub use account::{create_account, email_exists, username_exists, verify_credentials};
pub use character::{create_character, get_characters, load_character, save_character};
pub use progression::{load_progression, save_progression};
pub use inventory::{
    load_equipment, load_hotbar, load_inventory, load_learned_abilities,
    save_equipment, save_hotbar, save_inventory, save_learned_abilities,
};
pub use quests::{load_quest_log, save_quest_log};
pub use oauth::{create_oauth_account, find_account_by_oauth};
pub use bans::{check_account_ban, check_ip_ban, log_rate_limit_violation, BanInfo};

/// Database connection resource
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

/// Initialize the database connection and run migrations
pub fn setup_database(mut db_res: ResMut<DatabaseConnection>) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let pool = runtime.block_on(async {
        // Support DATABASE_PATH environment variable, default to "eryndor.db"
        let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "eryndor.db".to_string());
        let connection_string = format!("sqlite:{}?mode=rwc", db_path);

        eprintln!("Connecting to database: {}", connection_string);

        let pool = SqlitePool::connect(&connection_string)
            .await
            .expect("Failed to connect to database");

        // Run all migrations
        migrations::run_migrations(&pool).await;

        pool
    });

    db_res.pool = Some(pool);
}
