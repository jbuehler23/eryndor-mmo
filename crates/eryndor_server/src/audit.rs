// ============================================================================
// AUDIT LOGGING SYSTEM
// ============================================================================
// Comprehensive logging system for administrative actions and security events
// Stores logs in database for accountability and security monitoring

use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};

/// Types of actions that can be logged
#[derive(Debug, Clone)]
pub enum AuditActionType {
    // Account management
    AccountCreated,
    AccountLogin,
    AccountLoginFailed,
    AccountBanned,
    AccountUnbanned,

    // Character management
    CharacterCreated,
    CharacterDeleted,

    // Admin actions
    AdminCommandExecuted,
    AdminBroadcast,
    PlayerKicked,

    // Content moderation
    InappropriateContentBlocked,

    // Security events
    RateLimitExceeded,
    SuspiciousActivity,
}

impl AuditActionType {
    /// Convert action type to string for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditActionType::AccountCreated => "account_created",
            AuditActionType::AccountLogin => "account_login",
            AuditActionType::AccountLoginFailed => "account_login_failed",
            AuditActionType::AccountBanned => "account_banned",
            AuditActionType::AccountUnbanned => "account_unbanned",
            AuditActionType::CharacterCreated => "character_created",
            AuditActionType::CharacterDeleted => "character_deleted",
            AuditActionType::AdminCommandExecuted => "admin_command_executed",
            AuditActionType::AdminBroadcast => "admin_broadcast",
            AuditActionType::PlayerKicked => "player_kicked",
            AuditActionType::InappropriateContentBlocked => "inappropriate_content_blocked",
            AuditActionType::RateLimitExceeded => "rate_limit_exceeded",
            AuditActionType::SuspiciousActivity => "suspicious_activity",
        }
    }
}

/// Log an audit event to the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `action_type` - Type of action being logged
/// * `actor_account_id` - Account ID of who performed the action (None for system actions)
/// * `target_account_id` - Account ID of who was affected (None if not applicable)
/// * `target_username` - Username of who was affected (for readability)
/// * `ip_address` - IP address of the actor
/// * `details` - Additional details about the action (JSON-formatted or plain text)
pub async fn log_audit_event(
    pool: &SqlitePool,
    action_type: AuditActionType,
    actor_account_id: Option<i64>,
    target_account_id: Option<i64>,
    target_username: Option<&str>,
    ip_address: Option<&str>,
    details: Option<&str>,
) -> Result<(), String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let action_str = action_type.as_str();

    let result = sqlx::query(
        "INSERT INTO audit_logs (timestamp, action_type, actor_account_id, target_account_id, target_username, ip_address, details)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
    )
    .bind(timestamp)
    .bind(action_str)
    .bind(actor_account_id)
    .bind(target_account_id)
    .bind(target_username)
    .bind(ip_address)
    .bind(details)
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to log audit event: {}", e)),
    }
}

/// Retrieve recent audit logs (for admin dashboard)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `limit` - Maximum number of logs to retrieve
/// * `offset` - Number of logs to skip (for pagination)
pub async fn get_audit_logs(
    pool: &SqlitePool,
    limit: i64,
    offset: i64,
) -> Result<Vec<AuditLogEntry>, String> {
    let result = sqlx::query_as::<_, AuditLogEntry>(
        "SELECT id, timestamp, action_type, actor_account_id, target_account_id, target_username, ip_address, details
         FROM audit_logs
         ORDER BY timestamp DESC
         LIMIT ?1 OFFSET ?2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await;

    match result {
        Ok(logs) => Ok(logs),
        Err(e) => Err(format!("Failed to retrieve audit logs: {}", e)),
    }
}

/// Retrieve audit logs for a specific account
pub async fn get_audit_logs_for_account(
    pool: &SqlitePool,
    account_id: i64,
    limit: i64,
) -> Result<Vec<AuditLogEntry>, String> {
    let result = sqlx::query_as::<_, AuditLogEntry>(
        "SELECT id, timestamp, action_type, actor_account_id, target_account_id, target_username, ip_address, details
         FROM audit_logs
         WHERE actor_account_id = ?1 OR target_account_id = ?1
         ORDER BY timestamp DESC
         LIMIT ?2"
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(pool)
    .await;

    match result {
        Ok(logs) => Ok(logs),
        Err(e) => Err(format!("Failed to retrieve audit logs for account: {}", e)),
    }
}

/// Audit log entry structure
#[derive(Debug, sqlx::FromRow)]
pub struct AuditLogEntry {
    pub id: i64,
    pub timestamp: i64,
    pub action_type: String,
    pub actor_account_id: Option<i64>,
    pub target_account_id: Option<i64>,
    pub target_username: Option<String>,
    pub ip_address: Option<String>,
    pub details: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_type_strings() {
        assert_eq!(AuditActionType::AccountCreated.as_str(), "account_created");
        assert_eq!(AuditActionType::AccountBanned.as_str(), "account_banned");
        assert_eq!(AuditActionType::AdminCommandExecuted.as_str(), "admin_command_executed");
    }
}
