// ============================================================================
// ADMIN COMMANDS SYSTEM
// ============================================================================
// In-game admin commands for server management
// All commands are logged to the audit system

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use crate::database::DatabaseConnection;
use crate::auth::{Authenticated, ClientMetadata};
use sqlx::{SqlitePool, Row};

/// Admin command parsing result
pub enum AdminCommand {
    Ban {
        username: String,
        duration: Option<i64>,  // Duration in seconds, None = permanent
        reason: String,
    },
    Unban {
        username: String,
    },
    Kick {
        username: String,
        reason: String,
    },
    Broadcast {
        message: String,
    },
    Help,
    Invalid(String),
}

/// Parse admin command string into structured command
pub fn parse_command(command: String) -> AdminCommand {
    let command = command.trim();

    // Must start with /
    if !command.starts_with('/') {
        return AdminCommand::Invalid("Admin commands must start with /".to_string());
    }

    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return AdminCommand::Invalid("Empty command".to_string());
    }

    match parts[0] {
        "/ban" => {
            if parts.len() < 3 {
                return AdminCommand::Invalid("/ban usage: /ban <username> <duration|perm> [reason]".to_string());
            }

            let username = parts[1].to_string();
            let duration_str = parts[2];
            let reason = parts.get(3..).map(|p| p.join(" ")).unwrap_or_else(|| "No reason provided".to_string());

            // Parse duration: "perm" = permanent, "1h" = 1 hour, "30m" = 30 minutes, "7d" = 7 days
            let duration = if duration_str == "perm" {
                None  // Permanent ban
            } else {
                parse_duration(duration_str)
            };

            AdminCommand::Ban { username, duration, reason }
        }

        "/unban" => {
            if parts.len() < 2 {
                return AdminCommand::Invalid("/unban usage: /unban <username>".to_string());
            }
            AdminCommand::Unban {
                username: parts[1].to_string(),
            }
        }

        "/kick" => {
            if parts.len() < 2 {
                return AdminCommand::Invalid("/kick usage: /kick <username> [reason]".to_string());
            }
            let username = parts[1].to_string();
            let reason = parts.get(2..).map(|p| p.join(" ")).unwrap_or_else(|| "No reason provided".to_string());

            AdminCommand::Kick { username, reason }
        }

        "/broadcast" => {
            if parts.len() < 2 {
                return AdminCommand::Invalid("/broadcast usage: /broadcast <message>".to_string());
            }
            let message = parts[1..].join(" ");
            AdminCommand::Broadcast { message }
        }

        "/help" => {
            AdminCommand::Help
        }

        _ => {
            AdminCommand::Invalid(format!("Unknown command: {}. Type /help for available commands.", parts[0]))
        }
    }
}

/// Parse duration string to seconds
/// Examples: "1h" = 3600, "30m" = 1800, "7d" = 604800
fn parse_duration(duration_str: &str) -> Option<i64> {
    if duration_str.is_empty() {
        return None;
    }

    let len = duration_str.len();
    if len < 2 {
        return None;
    }

    let number_part = &duration_str[..len-1];
    let unit = &duration_str[len-1..];

    let number: i64 = number_part.parse().ok()?;

    match unit {
        "m" => Some(number * 60),           // Minutes
        "h" => Some(number * 3600),         // Hours
        "d" => Some(number * 86400),        // Days
        "w" => Some(number * 604800),       // Weeks
        _ => None,
    }
}

/// Check if account has admin permissions
pub async fn is_admin(pool: &SqlitePool, account_id: i64) -> Result<bool, String> {
    let result = sqlx::query("SELECT is_admin FROM accounts WHERE id = ?1")
        .bind(account_id)
        .fetch_optional(pool)
        .await;

    match result {
        Ok(Some(row)) => {
            let is_admin: bool = row.try_get("is_admin").unwrap_or(false);
            Ok(is_admin)
        }
        Ok(None) => Ok(false),
        Err(e) => Err(format!("Database error checking admin status: {}", e)),
    }
}

/// Get admin help text
pub fn get_help_text() -> String {
    r#"
=== ADMIN COMMANDS ===
/ban <username|character> <duration|perm> [reason] - Ban a player
  Examples: /ban john123 1h spam
           /ban PlayerName perm harassment

/unban <username> - Remove ban from player
  Example: /unban john123

/kick <character_name> [reason] - Kick player from server
  Example: /kick PlayerName disruptive behavior

/broadcast <message> - Send server-wide message
  Example: /broadcast Server restart in 5 minutes

/help - Show this help message

Duration formats: m=minutes, h=hours, d=days, w=weeks, perm=permanent
Note: Ban and kick commands accept either username or character name
Note: Game assets automatically hot-reload when JSON files are saved
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ban_command() {
        let cmd = parse_command("/ban username123 1h spamming chat".to_string());
        match cmd {
            AdminCommand::Ban { username, duration, reason } => {
                assert_eq!(username, "username123");
                assert_eq!(duration, Some(3600));
                assert_eq!(reason, "spamming chat");
            }
            _ => panic!("Expected Ban command"),
        }
    }

    #[test]
    fn test_parse_permanent_ban() {
        let cmd = parse_command("/ban baduser perm harassment".to_string());
        match cmd {
            AdminCommand::Ban { username, duration, reason } => {
                assert_eq!(username, "baduser");
                assert_eq!(duration, None);
                assert_eq!(reason, "harassment");
            }
            _ => panic!("Expected Ban command"),
        }
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30m"), Some(1800));
        assert_eq!(parse_duration("2h"), Some(7200));
        assert_eq!(parse_duration("7d"), Some(604800));
        assert_eq!(parse_duration("invalid"), None);
    }

    #[test]
    fn test_invalid_command() {
        let cmd = parse_command("/unknown".to_string());
        match cmd {
            AdminCommand::Invalid(_) => {}
            _ => panic!("Expected Invalid command"),
        }
    }
}

// ============================================================================
// ADMIN COMMAND HANDLER
// ============================================================================

/// Handle admin command requests from clients
pub fn handle_admin_command(
    trigger: On<FromClient<AdminCommandRequest>>,
    mut commands: Commands,
    client_query: Query<(&Authenticated, Option<&ClientMetadata>)>,
    characters: Query<(Entity, &Character, &OwnedBy)>,
    db: Res<DatabaseConnection>,
    tokio_runtime: Res<crate::TokioRuntimeResource>,
) {
    let Some(client_entity) = trigger.client_id.entity() else {
        warn!("No client entity in admin command trigger");
        return;
    };

    // Check if client is authenticated
    let Ok((auth, metadata_opt)) = client_query.get(client_entity) else {
        warn!("Admin command from unauthenticated client");
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(trigger.client_id),
            message: NotificationEvent {
                message: "You must be logged in to use admin commands".to_string(),
                notification_type: NotificationType::Error,
            },
        });
        return;
    };

    let account_id = auth.account_id;
    let ip_address = metadata_opt.map(|m| m.ip_address.to_string());

    // Check if user has admin permissions
    let Some(pool) = db.pool() else {
        error!("Database not available for admin command");
        return;
    };

    // Use shared tokio runtime instead of creating a new one each time
    let is_admin_result = tokio_runtime.0.block_on(is_admin(pool, account_id));

    match is_admin_result {
        Ok(true) => {
            // User is admin, proceed with command
        }
        Ok(false) => {
            warn!("Non-admin account {} attempted admin command: {}", account_id, trigger.event().command);

            // AUDIT LOG: Unauthorized admin command attempt
            let _ = tokio_runtime.0.block_on(crate::audit::log_audit_event(
                pool,
                crate::audit::AuditActionType::SuspiciousActivity,
                Some(account_id),
                None,
                None,
                ip_address.as_deref(),
                Some(&format!("attempted admin command: {}", trigger.event().command)),
            ));

            commands.server_trigger(ToClients {
                mode: SendMode::Direct(trigger.client_id),
                message: NotificationEvent {
                    message: "You do not have permission to use admin commands".to_string(),
                    notification_type: NotificationType::Error,
                },
            });
            return;
        }
        Err(e) => {
            error!("Database error checking admin status: {}", e);
            return;
        }
    }

    // Parse the command
    let command = parse_command(trigger.event().command.clone());

    match command {
        AdminCommand::Help => {
            let help_text = get_help_text();
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(trigger.client_id),
                message: NotificationEvent {
                    message: help_text,
                    notification_type: NotificationType::Info,
                },
            });
        }

        AdminCommand::Ban { username, duration, reason } => {
            info!("Admin {} executing ban command: username={}, duration={:?}, reason={}",
                  account_id, username, duration, reason);

            let result = tokio_runtime.0.block_on(execute_ban(pool, &username, duration, &reason, account_id));

            match result {
                Ok(message) => {
                    // AUDIT LOG: Account banned
                    let _ = tokio_runtime.0.block_on(crate::audit::log_audit_event(
                        pool,
                        crate::audit::AuditActionType::AdminCommandExecuted,
                        Some(account_id),
                        None,
                        Some(&username),
                        ip_address.as_deref(),
                        Some(&format!("banned user: {} for {} (duration: {:?})", username, reason, duration)),
                    ));

                    commands.server_trigger(ToClients {
                        mode: SendMode::Direct(trigger.client_id),
                        message: NotificationEvent {
                            message,
                            notification_type: NotificationType::Success,
                        },
                    });
                }
                Err(e) => {
                    error!("Ban command failed: {}", e);
                    commands.server_trigger(ToClients {
                        mode: SendMode::Direct(trigger.client_id),
                        message: NotificationEvent {
                            message: format!("Ban failed: {}", e),
                            notification_type: NotificationType::Error,
                        },
                    });
                }
            }
        }

        AdminCommand::Unban { username } => {
            info!("Admin {} executing unban command: username={}", account_id, username);

            let result = tokio_runtime.0.block_on(execute_unban(pool, &username));

            match result {
                Ok(message) => {
                    // AUDIT LOG: Account unbanned
                    let _ = tokio_runtime.0.block_on(crate::audit::log_audit_event(
                        pool,
                        crate::audit::AuditActionType::AdminCommandExecuted,
                        Some(account_id),
                        None,
                        Some(&username),
                        ip_address.as_deref(),
                        Some(&format!("unbanned user: {}", username)),
                    ));

                    commands.server_trigger(ToClients {
                        mode: SendMode::Direct(trigger.client_id),
                        message: NotificationEvent {
                            message,
                            notification_type: NotificationType::Success,
                        },
                    });
                }
                Err(e) => {
                    error!("Unban command failed: {}", e);
                    commands.server_trigger(ToClients {
                        mode: SendMode::Direct(trigger.client_id),
                        message: NotificationEvent {
                            message: format!("Unban failed: {}", e),
                            notification_type: NotificationType::Error,
                        },
                    });
                }
            }
        }

        AdminCommand::Kick { username, reason } => {
            info!("Admin {} executing kick command: username={}, reason={}",
                  account_id, username, reason);

            // Find the character by name
            let mut found_character = None;
            for (char_entity, character, owned_by) in characters.iter() {
                if character.name.eq_ignore_ascii_case(&username) {
                    found_character = Some((char_entity, owned_by.0));
                    break;
                }
            }

            if let Some((char_entity, client_entity)) = found_character {
                // AUDIT LOG: Player kicked
                let _ = tokio_runtime.0.block_on(crate::audit::log_audit_event(
                    pool,
                    crate::audit::AuditActionType::PlayerKicked,
                    Some(account_id),
                    None,
                    Some(&username),
                    ip_address.as_deref(),
                    Some(&format!("kicked for: {}", reason)),
                ));

                // Notify the kicked player before despawning
                commands.server_trigger(ToClients {
                    mode: SendMode::Direct(ClientId::Client(client_entity)),
                    message: NotificationEvent {
                        message: format!("You have been kicked from the server. Reason: {}", reason),
                        notification_type: NotificationType::Error,
                    },
                });

                // Despawn the character - this will trigger disconnect handling
                commands.entity(char_entity).despawn();

                info!("Kicked character '{}' (entity {:?}) owned by client {:?}", username, char_entity, client_entity);

                // Confirm to admin
                commands.server_trigger(ToClients {
                    mode: SendMode::Direct(trigger.client_id),
                    message: NotificationEvent {
                        message: format!("Player '{}' has been kicked from the server", username),
                        notification_type: NotificationType::Success,
                    },
                });
            } else {
                // Character not found
                warn!("Attempted to kick '{}' but character not found (not online)", username);
                commands.server_trigger(ToClients {
                    mode: SendMode::Direct(trigger.client_id),
                    message: NotificationEvent {
                        message: format!("Player '{}' not found (may not be online)", username),
                        notification_type: NotificationType::Error,
                    },
                });
            }
        }

        AdminCommand::Broadcast { message } => {
            info!("Admin {} broadcasting message: {}", account_id, message);

            // AUDIT LOG: Broadcast sent
            let _ = tokio_runtime.0.block_on(crate::audit::log_audit_event(
                pool,
                crate::audit::AuditActionType::AdminBroadcast,
                Some(account_id),
                None,
                None,
                ip_address.as_deref(),
                Some(&message),
            ));

            // Send to all clients
            commands.server_trigger(ToClients {
                mode: SendMode::Broadcast,
                message: NotificationEvent {
                    message: format!("[ADMIN BROADCAST] {}", message),
                    notification_type: NotificationType::Warning,
                },
            });

            // Confirm to sender
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(trigger.client_id),
                message: NotificationEvent {
                    message: "Broadcast sent successfully".to_string(),
                    notification_type: NotificationType::Success,
                },
            });
        }

        AdminCommand::Invalid(error_msg) => {
            warn!("Invalid admin command from {}: {}", account_id, error_msg);
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(trigger.client_id),
                message: NotificationEvent {
                    message: error_msg,
                    notification_type: NotificationType::Error,
                },
            });
        }
    }
}

// ============================================================================
// COMMAND EXECUTION HELPERS
// ============================================================================

/// Execute a ban command
async fn execute_ban(
    pool: &SqlitePool,
    name_or_username: &str,
    duration: Option<i64>,
    reason: &str,
    banned_by_account_id: i64,
) -> Result<String, String> {
    // Try to find account by username first
    let account_result = sqlx::query(
        "SELECT id, username FROM accounts WHERE username = ?1"
    )
    .bind(name_or_username)
    .fetch_optional(pool)
    .await;

    // If not found by username, try to find by character name
    let (account_id, username) = match account_result {
        Ok(Some(row)) => {
            let id: i64 = row.try_get("id")
                .map_err(|e| format!("Failed to get id: {}", e))?;
            let username: String = row.try_get("username")
                .map_err(|e| format!("Failed to get username: {}", e))?;
            (id, username)
        }
        Ok(None) => {
            // Not found by username, try character name
            let char_result = sqlx::query(
                "SELECT account_id FROM characters WHERE name = ?1"
            )
            .bind(name_or_username)
            .fetch_optional(pool)
            .await;

            match char_result {
                Ok(Some(row)) => {
                    let acc_id: i64 = row.try_get("account_id")
                        .map_err(|e| format!("Failed to get account_id: {}", e))?;

                    // Get username for this account
                    let username_result = sqlx::query(
                        "SELECT username FROM accounts WHERE id = ?1"
                    )
                    .bind(acc_id)
                    .fetch_optional(pool)
                    .await;

                    match username_result {
                        Ok(Some(username_row)) => {
                            let username: String = username_row.try_get("username")
                                .map_err(|e| format!("Failed to get username: {}", e))?;
                            (acc_id, username)
                        }
                        _ => return Err(format!("Account not found for character '{}'", name_or_username)),
                    }
                }
                Ok(None) => {
                    return Err(format!("User or character '{}' not found", name_or_username));
                }
                Err(e) => {
                    return Err(format!("Database error: {}", e));
                }
            }
        }
        Err(e) => {
            return Err(format!("Database error: {}", e));
        }
    };

    // Calculate expiry time if not permanent
    let expires_at = duration.map(|seconds| {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        now + seconds
    });

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Insert ban record using new bans table schema
    let result = sqlx::query(
        "INSERT INTO bans (ban_type, target, account_id, reason, banned_by, banned_at, expires_at, is_active)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)"
    )
    .bind("account")  // ban_type: 'account', 'ip', or 'both'
    .bind(&username)  // target: username for account bans
    .bind(account_id)
    .bind(reason)
    .bind(banned_by_account_id)
    .bind(now)
    .bind(expires_at)
    .execute(pool)
    .await;

    match result {
        Ok(_) => {
            let message = if duration.is_none() {
                format!("User '{}' has been permanently banned. Reason: {}", username, reason)
            } else {
                let duration_str = format_duration(duration.unwrap());
                format!("User '{}' has been banned for {}. Reason: {}", username, duration_str, reason)
            };
            Ok(message)
        }
        Err(e) => Err(format!("Failed to ban user: {}", e)),
    }
}

/// Execute an unban command
async fn execute_unban(
    pool: &SqlitePool,
    username: &str,
) -> Result<String, String> {
    // Deactivate all bans for this username (using target field from new bans table)
    let result = sqlx::query(
        "UPDATE bans SET is_active = 0 WHERE target = ?1 AND is_active = 1 AND ban_type = 'account'"
    )
    .bind(username)
    .execute(pool)
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(format!("User '{}' has been unbanned", username))
            } else {
                Err(format!("User '{}' is not currently banned", username))
            }
        }
        Err(e) => Err(format!("Failed to unban user: {}", e)),
    }
}

/// Format duration in seconds to human-readable string
fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{} seconds", seconds)
    } else if seconds < 3600 {
        format!("{} minutes", seconds / 60)
    } else if seconds < 86400 {
        format!("{} hours", seconds / 3600)
    } else if seconds < 604800 {
        format!("{} days", seconds / 86400)
    } else {
        format!("{} weeks", seconds / 604800)
    }
}

// ============================================================================
// CHAT SYSTEM
// ============================================================================

/// Handle regular chat messages from players
/// Broadcasts the message to all connected players with the sender's name
pub fn handle_chat_message(
    trigger: On<FromClient<SendChatMessage>>,
    mut commands: Commands,
    characters: Query<(&Character, &OwnedBy), With<Player>>,
) {
    let Some(client_entity) = trigger.client_id.entity() else {
        warn!("No client entity in chat message trigger");
        return;
    };

    // Find the player character owned by this client
    let mut sender_name = None;
    for (character, owned_by) in characters.iter() {
        if owned_by.0 == client_entity {
            sender_name = Some(character.name.clone());
            break;
        }
    }

    let Some(sender_name) = sender_name else {
        warn!("Chat message from client {:?} without spawned character", client_entity);
        return;
    };

    // Create the chat message event to broadcast
    let chat_message = ChatMessage {
        sender: sender_name,
        message: trigger.message.message.clone(),
    };

    // Broadcast to all connected players
    commands.server_trigger(ToClients {
        mode: SendMode::Broadcast,
        message: chat_message,
    });
}
