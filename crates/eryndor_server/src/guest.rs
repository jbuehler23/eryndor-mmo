use bevy::prelude::*;
use sqlx::{SqlitePool, Row};
use uuid::Uuid;

/// Guest account management and conversion

// ============================================================================
// GUEST ACCOUNT CREATION
// ============================================================================

pub async fn create_guest_account(pool: &SqlitePool) -> Result<(i64, String, String), String> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    // Generate guest token (UUID v4 - 128 bits of randomness)
    let guest_token = Uuid::new_v4().to_string();

    // Generate auto username: Guest_XXXXXX (6 random digits)
    let random_suffix: u32 = rand::random::<u32>() % 1_000_000;
    let username = format!("Guest_{:06}", random_suffix);

    // Hash the guest token before storing (same as password)
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let token_hash = argon2
        .hash_password(guest_token.as_bytes(), &salt)
        .map_err(|e| format!("Failed to hash guest token: {}", e))?
        .to_string();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Guest accounts expire in 7 days by default
    let expires_at = now + (7 * 24 * 60 * 60);

    // Create guest account
    let result = sqlx::query(
        "INSERT INTO accounts (
            username,
            password_hash,
            account_type,
            guest_token,
            guest_created_at,
            guest_expires_at,
            created_at
        ) VALUES (?1, ?2, 'guest', ?3, ?4, ?5, ?6)"
    )
    .bind(&username)
    .bind(&token_hash)
    .bind(&guest_token)  // Store original token for lookup (will be cleared on conversion)
    .bind(now)
    .bind(expires_at)
    .bind(now)
    .execute(pool)
    .await;

    match result {
        Ok(result) => {
            let account_id = result.last_insert_rowid();
            info!("Created guest account: {} (ID: {}) - expires at {}", username, account_id, expires_at);
            Ok((account_id, guest_token, username))
        }
        Err(e) => Err(format!("Failed to create guest account: {}", e)),
    }
}

// ============================================================================
// GUEST ACCOUNT LOGIN
// ============================================================================

pub async fn verify_guest_token(pool: &SqlitePool, guest_token: &str) -> Result<i64, String> {
    // Look up guest account by token
    let result = sqlx::query(
        "SELECT id, username, guest_expires_at
         FROM accounts
         WHERE guest_token = ?1 AND account_type = 'guest'"
    )
    .bind(guest_token)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let account_id: i64 = row.get(0);
            let username: String = row.get(1);
            let expires_at: i64 = row.get(2);

            // Check expiration
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            if now > expires_at {
                return Err(format!("Guest account '{}' has expired. Please create a new guest account or register.", username));
            }

            info!("Guest login successful: {} (ID: {})", username, account_id);
            Ok(account_id)
        }
        Ok(None) => Err("Invalid guest token".to_string()),
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

// ============================================================================
// GUEST TO REGISTERED CONVERSION
// ============================================================================

pub async fn convert_guest_to_registered(
    pool: &SqlitePool,
    guest_account_id: i64,
    email: String,
    password: String,
) -> Result<(), String> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    // Validate email format
    if !is_valid_email(&email) {
        return Err("Invalid email format".to_string());
    }

    // Validate password
    if password.len() < 8 {
        return Err("Password must be at least 8 characters".to_string());
    }

    // Check if email already exists
    let email_check = sqlx::query("SELECT id FROM accounts WHERE email = ?1")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if email_check.is_some() {
        return Err("Email already in use".to_string());
    }

    // Verify this is actually a guest account
    let account_check = sqlx::query("SELECT account_type FROM accounts WHERE id = ?1")
        .bind(guest_account_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    match account_check {
        Some(row) => {
            let account_type: String = row.get(0);
            if account_type != "guest" {
                return Err("Account is not a guest account".to_string());
            }
        }
        None => return Err("Account not found".to_string()),
    }

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Failed to hash password: {}", e))?
        .to_string();

    // Update account to registered
    let result = sqlx::query(
        "UPDATE accounts SET
            email = ?1,
            password_hash = ?2,
            account_type = 'registered',
            email_verified = FALSE,
            guest_token = NULL,
            guest_created_at = NULL,
            guest_expires_at = NULL
         WHERE id = ?3 AND account_type = 'guest'"
    )
    .bind(&email)
    .bind(&password_hash)
    .bind(guest_account_id)
    .execute(pool)
    .await;

    match result {
        Ok(rows) => {
            if rows.rows_affected() == 0 {
                return Err("Failed to convert account - account may have already been converted".to_string());
            }
            info!("Converted guest account {} to registered with email {}", guest_account_id, email);
            Ok(())
        }
        Err(e) => Err(format!("Failed to convert account: {}", e)),
    }
}

// ============================================================================
// GUEST ACCOUNT CLEANUP
// ============================================================================

pub async fn cleanup_expired_guests(pool: &SqlitePool) -> Result<usize, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Find expired guest accounts
    let expired_guests = sqlx::query(
        "SELECT id, username FROM accounts
         WHERE account_type = 'guest' AND guest_expires_at < ?1"
    )
    .bind(now)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to fetch expired guests: {}", e))?;

    let count = expired_guests.len();

    if count == 0 {
        return Ok(0);
    }

    info!("Cleaning up {} expired guest accounts", count);

    // Delete each guest and their associated data
    for row in expired_guests {
        let account_id: i64 = row.get(0);
        let username: String = row.get(1);

        // Delete characters first (cascade will handle their data)
        let _ = sqlx::query("DELETE FROM characters WHERE account_id = ?1")
            .bind(account_id)
            .execute(pool)
            .await;

        // Delete account
        let _ = sqlx::query("DELETE FROM accounts WHERE id = ?1")
            .bind(account_id)
            .execute(pool)
            .await;

        info!("Deleted expired guest account: {} (ID: {})", username, account_id);
    }

    Ok(count)
}

// ============================================================================
// GUEST ACCOUNT WARNINGS
// ============================================================================

pub async fn get_expiring_guests(pool: &SqlitePool, hours_until_expiry: i64) -> Result<Vec<(i64, String, i64)>, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let threshold = now + (hours_until_expiry * 60 * 60);

    let result = sqlx::query(
        "SELECT id, username, guest_expires_at
         FROM accounts
         WHERE account_type = 'guest'
         AND guest_expires_at < ?1
         AND guest_expires_at > ?2"
    )
    .bind(threshold)
    .bind(now)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to fetch expiring guests: {}", e))?;

    let guests: Vec<(i64, String, i64)> = result
        .iter()
        .map(|row| {
            let id: i64 = row.get(0);
            let username: String = row.get(1);
            let expires_at: i64 = row.get(2);
            (id, username, expires_at)
        })
        .collect();

    Ok(guests)
}

// ============================================================================
// UTILITIES
// ============================================================================

fn is_valid_email(email: &str) -> bool {
    // Basic email validation
    if email.len() > 254 {
        return false;
    }

    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let local = parts[0];
    let domain = parts[1];

    if local.is_empty() || domain.is_empty() {
        return false;
    }

    if !domain.contains('.') {
        return false;
    }

    // Check for valid characters (basic check)
    let valid_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789.!#$%&'*+/=?^_`{|}~-@";
    email.chars().all(|c| valid_chars.contains(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name@domain.co.uk"));
        assert!(is_valid_email("user+tag@example.com"));

        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email("user@domain"));
        assert!(!is_valid_email(""));
    }
}
