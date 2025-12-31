//! OAuth account management.
//!
//! Handles OAuth provider account linking and creation.

use sqlx::{SqlitePool, Row};

/// Find account by OAuth provider and ID
pub async fn find_account_by_oauth(
    pool: &SqlitePool,
    provider: &str,
    oauth_id: &str,
) -> Result<Option<i64>, String> {
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
    oauth_id: &str,
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
