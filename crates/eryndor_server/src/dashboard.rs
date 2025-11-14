// ============================================================================
// ADMIN DASHBOARD HANDLERS
// ============================================================================
// Backend handlers for admin dashboard queries
// All handlers require admin permissions

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use crate::database::DatabaseConnection;
use crate::admin::is_admin;
use crate::auth::Authenticated;
use sqlx::{SqlitePool, Row};

/// Handle request for online player list
pub fn handle_get_player_list(
    trigger: On<FromClient<GetPlayerListRequest>>,
    mut commands: Commands,
    client_query: Query<&Authenticated>,
    characters: Query<(&Character, &Position, &OwnedBy), With<Player>>,
    owners: Query<&Authenticated>,
    db: Res<DatabaseConnection>,
) {
    let Some(client_entity) = trigger.client_id.entity() else {
        warn!("No client entity in get player list request");
        return;
    };

    // Check if client is authenticated
    let Ok(auth) = client_query.get(client_entity) else {
        warn!("Get player list from unauthenticated client");
        return;
    };

    // Check admin permissions
    let Some(pool) = db.pool() else {
        error!("Database not available");
        return;
    };

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let is_admin_result = runtime.block_on(is_admin(pool, auth.account_id));

    match is_admin_result {
        Ok(true) => {
            // User is admin, collect player list
            let mut players = Vec::new();

            for (character, position, owned_by) in characters.iter() {
                // Get the account_id from the owner (client connection entity)
                if let Ok(owner_auth) = owners.get(owned_by.0) {
                    players.push(OnlinePlayerInfo {
                        character_name: character.name.clone(),
                        account_id: owner_auth.account_id,
                        level: character.level,
                        class: character.class,
                        position_x: position.0.x,
                        position_y: position.0.y,
                    });
                }
            }

            info!("Admin {} requested player list, returning {} players", auth.account_id, players.len());

            commands.server_trigger(ToClients {
                mode: SendMode::Direct(trigger.client_id),
                message: PlayerListResponse { players },
            });
        }
        Ok(false) => {
            warn!("Non-admin account {} attempted to get player list", auth.account_id);
        }
        Err(e) => {
            error!("Database error checking admin status: {}", e);
        }
    }
}

/// Handle request for ban list
pub fn handle_get_ban_list(
    trigger: On<FromClient<GetBanListRequest>>,
    mut commands: Commands,
    client_query: Query<&Authenticated>,
    db: Res<DatabaseConnection>,
) {
    let Some(client_entity) = trigger.client_id.entity() else {
        warn!("No client entity in get ban list request");
        return;
    };

    let Ok(auth) = client_query.get(client_entity) else {
        warn!("Get ban list from unauthenticated client");
        return;
    };

    let Some(pool) = db.pool() else {
        error!("Database not available");
        return;
    };

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let is_admin_result = runtime.block_on(is_admin(pool, auth.account_id));

    match is_admin_result {
        Ok(true) => {
            // Fetch bans from database
            let bans_result = runtime.block_on(fetch_ban_list(pool));

            match bans_result {
                Ok(bans) => {
                    info!("Admin {} requested ban list, returning {} bans", auth.account_id, bans.len());

                    commands.server_trigger(ToClients {
                        mode: SendMode::Direct(trigger.client_id),
                        message: BanListResponse { bans },
                    });
                }
                Err(e) => {
                    error!("Failed to fetch ban list: {}", e);
                }
            }
        }
        Ok(false) => {
            warn!("Non-admin account {} attempted to get ban list", auth.account_id);
        }
        Err(e) => {
            error!("Database error checking admin status: {}", e);
        }
    }
}

/// Handle request for server statistics
pub fn handle_get_server_stats(
    trigger: On<FromClient<GetServerStatsRequest>>,
    mut commands: Commands,
    client_query: Query<&Authenticated>,
    characters: Query<&Character>,
    db: Res<DatabaseConnection>,
) {
    let Some(client_entity) = trigger.client_id.entity() else {
        warn!("No client entity in get server stats request");
        return;
    };

    let Ok(auth) = client_query.get(client_entity) else {
        warn!("Get server stats from unauthenticated client");
        return;
    };

    let Some(pool) = db.pool() else {
        error!("Database not available");
        return;
    };

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let is_admin_result = runtime.block_on(is_admin(pool, auth.account_id));

    match is_admin_result {
        Ok(true) => {
            // Count online players
            let online_players = characters.iter().count() as u32;

            // Fetch database stats
            let stats_result = runtime.block_on(fetch_server_stats(pool, online_players));

            match stats_result {
                Ok(stats) => {
                    info!("Admin {} requested server stats", auth.account_id);

                    commands.server_trigger(ToClients {
                        mode: SendMode::Direct(trigger.client_id),
                        message: stats,
                    });
                }
                Err(e) => {
                    error!("Failed to fetch server stats: {}", e);
                }
            }
        }
        Ok(false) => {
            warn!("Non-admin account {} attempted to get server stats", auth.account_id);
        }
        Err(e) => {
            error!("Database error checking admin status: {}", e);
        }
    }
}

/// Handle request for audit logs
pub fn handle_get_audit_logs(
    trigger: On<FromClient<GetAuditLogsRequest>>,
    mut commands: Commands,
    client_query: Query<&Authenticated>,
    db: Res<DatabaseConnection>,
) {
    let Some(client_entity) = trigger.client_id.entity() else {
        warn!("No client entity in get audit logs request");
        return;
    };

    let Ok(auth) = client_query.get(client_entity) else {
        warn!("Get audit logs from unauthenticated client");
        return;
    };

    let Some(pool) = db.pool() else {
        error!("Database not available");
        return;
    };

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let is_admin_result = runtime.block_on(is_admin(pool, auth.account_id));

    match is_admin_result {
        Ok(true) => {
            let request = trigger.event();
            let logs_result = runtime.block_on(fetch_audit_logs(pool, request.limit, request.offset));

            match logs_result {
                Ok(response) => {
                    info!("Admin {} requested audit logs (limit: {}, offset: {})",
                          auth.account_id, request.limit, request.offset);

                    commands.server_trigger(ToClients {
                        mode: SendMode::Direct(trigger.client_id),
                        message: response,
                    });
                }
                Err(e) => {
                    error!("Failed to fetch audit logs: {}", e);
                }
            }
        }
        Ok(false) => {
            warn!("Non-admin account {} attempted to get audit logs", auth.account_id);
        }
        Err(e) => {
            error!("Database error checking admin status: {}", e);
        }
    }
}

// ============================================================================
// DATABASE QUERY FUNCTIONS
// ============================================================================

/// Fetch list of active bans from database
async fn fetch_ban_list(pool: &SqlitePool) -> Result<Vec<BanInfo>, String> {
    let rows = sqlx::query(
        "SELECT id, ban_type, target, reason, banned_by, banned_at, expires_at, is_active
         FROM bans
         WHERE is_active = 1
         ORDER BY banned_at DESC
         LIMIT 100"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    let mut bans = Vec::new();
    for row in rows {
        bans.push(BanInfo {
            id: row.try_get("id").unwrap_or(0),
            ban_type: row.try_get("ban_type").unwrap_or_else(|_| "account".to_string()),
            target: row.try_get("target").unwrap_or_else(|_| "unknown".to_string()),
            reason: row.try_get("reason").unwrap_or_else(|_| "No reason".to_string()),
            banned_by_id: row.try_get("banned_by").ok(),
            banned_at: row.try_get("banned_at").unwrap_or(0),
            expires_at: row.try_get("expires_at").ok(),
            is_active: row.try_get("is_active").unwrap_or(false),
        });
    }

    Ok(bans)
}

/// Fetch server statistics from database
async fn fetch_server_stats(pool: &SqlitePool, online_players: u32) -> Result<ServerStatsResponse, String> {
    // Count total accounts
    let row = sqlx::query("SELECT COUNT(*) FROM accounts")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to count accounts: {}", e))?;
    let total_accounts = row.get::<i64, _>(0) as u32;

    // Count total characters
    let row = sqlx::query("SELECT COUNT(*) FROM characters")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to count characters: {}", e))?;
    let total_characters = row.get::<i64, _>(0) as u32;

    // Count active bans
    let row = sqlx::query("SELECT COUNT(*) FROM bans WHERE is_active = 1")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to count bans: {}", e))?;
    let active_bans = row.get::<i64, _>(0) as u32;

    Ok(ServerStatsResponse {
        online_players,
        total_accounts,
        total_characters,
        active_bans,
    })
}

/// Fetch audit logs with pagination
async fn fetch_audit_logs(pool: &SqlitePool, limit: u32, offset: u32) -> Result<AuditLogsResponse, String> {
    // Get total count
    let row = sqlx::query("SELECT COUNT(*) FROM audit_logs")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to count audit logs: {}", e))?;
    let total_count = row.get::<i64, _>(0) as u32;

    // Fetch paginated logs
    let rows = sqlx::query(
        "SELECT id, action_type, actor_account_id, target_account_id, ip_address, details, timestamp
         FROM audit_logs
         ORDER BY timestamp DESC
         LIMIT ?1 OFFSET ?2"
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    let mut logs = Vec::new();
    for row in rows {
        logs.push(AuditLogEntry {
            id: row.try_get("id").unwrap_or(0),
            action_type: row.try_get("action_type").unwrap_or_else(|_| "unknown".to_string()),
            account_id: row.try_get("actor_account_id").ok(),
            target_account: row.try_get("target_account_id").ok(),
            ip_address: row.try_get("ip_address").ok(),
            details: row.try_get("details").ok(),
            timestamp: row.try_get("timestamp").unwrap_or(0),
        });
    }

    Ok(AuditLogsResponse {
        logs,
        total_count,
    })
}
