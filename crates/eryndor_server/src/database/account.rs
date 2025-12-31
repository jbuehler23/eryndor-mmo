//! Account-related database operations.
//!
//! Handles account creation, credential verification, and existence checks.

use sqlx::{SqlitePool, Row};

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

/// Create a new registered account
pub async fn create_account(
    pool: &SqlitePool,
    email: &str,
    username: &str,
    password_hash: &str,
) -> Result<i64, String> {
    // Check for existing email
    match email_exists(pool, email).await {
        Ok(true) => return Err("Email already in use".to_string()),
        Ok(false) => {}
        Err(e) => return Err(e),
    }

    // Check for existing username
    match username_exists(pool, username).await {
        Ok(true) => return Err("Username already taken".to_string()),
        Ok(false) => {}
        Err(e) => return Err(e),
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let result = sqlx::query(
        "INSERT INTO accounts (email, username, password_hash, created_at, account_type)
         VALUES (?1, ?2, ?3, ?4, 'registered')"
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

/// Verify login credentials and return account ID if valid
pub async fn verify_credentials(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<i64, String> {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };

    let result = sqlx::query("SELECT id, password_hash FROM accounts WHERE username = ?1")
        .bind(username)
        .fetch_optional(pool)
        .await;

    match result {
        Ok(Some(row)) => {
            let account_id: i64 = row.get(0);
            let stored_hash: String = row.get(1);

            let parsed_hash = PasswordHash::new(&stored_hash)
                .map_err(|e| format!("Failed to parse stored hash: {}", e))?;

            let argon2 = Argon2::default();
            argon2
                .verify_password(password.as_bytes(), &parsed_hash)
                .map_err(|_| "Invalid credentials".to_string())?;

            Ok(account_id)
        }
        Ok(None) => Err("Invalid credentials".to_string()),
        Err(e) => Err(format!("Database error: {}", e)),
    }
}
