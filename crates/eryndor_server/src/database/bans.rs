//! Ban system database operations.
//!
//! Handles ban checking and rate limit violation logging.

use sqlx::{SqlitePool, Row};
use tracing::info;

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
pub async fn check_ip_ban(pool: &SqlitePool, ip_address: &str) -> Result<Option<BanInfo>, String> {
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
