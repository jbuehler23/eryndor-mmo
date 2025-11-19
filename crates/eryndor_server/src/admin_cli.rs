use sqlx::SqlitePool;
use std::env;

pub async fn run_admin_command() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "make-admin" => {
            if args.len() < 3 {
                eprintln!("Usage: {} make-admin <email>", args[0]);
                std::process::exit(1);
            }
            make_admin(&args[2]).await;
        }
        "remove-admin" => {
            if args.len() < 3 {
                eprintln!("Usage: {} remove-admin <email>", args[0]);
                std::process::exit(1);
            }
            remove_admin(&args[2]).await;
        }
        "list-admins" => {
            list_admins().await;
        }
        "reset-password" => {
            if args.len() < 3 {
                eprintln!("Usage: {} reset-password <email>", args[0]);
                std::process::exit(1);
            }
            reset_password(&args[2]).await;
        }
        "list-users" => {
            list_users().await;
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!("Eryndor MMO Admin CLI");
    println!("\nUsage: server <command> [args]");
    println!("\nCommands:");
    println!("  make-admin <email>     - Grant admin privileges to a user");
    println!("  remove-admin <email>   - Revoke admin privileges from a user");
    println!("  list-admins            - List all admin users");
    println!("  reset-password <email> - Reset user password (generates temp password)");
    println!("  list-users             - List all users");
    println!("\nEnvironment Variables:");
    println!("  DATABASE_PATH          - Path to SQLite database (default: eryndor.db)");
}

async fn get_pool() -> SqlitePool {
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "eryndor.db".to_string());
    let connection_string = format!("sqlite:{}?mode=rwc", db_path);

    SqlitePool::connect(&connection_string)
        .await
        .expect("Failed to connect to database")
}

async fn make_admin(email: &str) {
    let pool = get_pool().await;

    let result = sqlx::query!(
        "UPDATE accounts SET is_admin = 1 WHERE email = ?",
        email
    )
    .execute(&pool)
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                println!("✓ Successfully granted admin privileges to: {}", email);
            } else {
                eprintln!("✗ User not found: {}", email);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("✗ Database error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn remove_admin(email: &str) {
    let pool = get_pool().await;

    let result = sqlx::query!(
        "UPDATE accounts SET is_admin = 0 WHERE email = ?",
        email
    )
    .execute(&pool)
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                println!("✓ Successfully revoked admin privileges from: {}", email);
            } else {
                eprintln!("✗ User not found: {}", email);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("✗ Database error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn list_admins() {
    let pool = get_pool().await;

    let admins = sqlx::query!(
        "SELECT email, username, created_at FROM accounts WHERE is_admin = 1 ORDER BY email"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query admins");

    if admins.is_empty() {
        println!("No admin users found.");
        return;
    }

    println!("\nAdmin Users:");
    println!("{:<30} {:<20} Created At", "Email", "Username");
    println!("{}", "-".repeat(70));

    for admin in admins {
        let email = admin.email.as_deref().unwrap_or("N/A");
        let created_ts = chrono::DateTime::from_timestamp(admin.created_at, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "Invalid date".to_string());
        println!("{:<30} {:<20} {}", email, admin.username, created_ts);
    }
}

async fn reset_password(email: &str) {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    let pool = get_pool().await;

    // Generate a temporary password
    let temp_password: String = (0..12)
        .map(|_| {
            let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            chars.chars().nth(rand::random::<usize>() % chars.len()).unwrap()
        })
        .collect();

    // Hash the password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(temp_password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();

    let result = sqlx::query!(
        "UPDATE accounts SET password_hash = ? WHERE email = ?",
        password_hash,
        email
    )
    .execute(&pool)
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                println!("✓ Password reset successfully for: {}", email);
                println!("\nTemporary password: {}", temp_password);
                println!("\nIMPORTANT: User should change this password immediately after logging in!");
            } else {
                eprintln!("✗ User not found: {}", email);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("✗ Database error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn list_users() {
    let pool = get_pool().await;

    let users = sqlx::query!(
        "SELECT email, username, is_admin, account_type, created_at FROM accounts ORDER BY created_at DESC LIMIT 50"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query users");

    if users.is_empty() {
        println!("No users found.");
        return;
    }

    println!("\nUsers (most recent 50):");
    println!("{:<30} {:<20} {:<8} {:<12} Created At", "Email", "Username", "Admin", "Type");
    println!("{}", "-".repeat(90));

    for user in users {
        let email = user.email.as_deref().unwrap_or("N/A");
        let is_admin = if user.is_admin.unwrap_or(false) { "Yes" } else { "No" };
        let account_type = user.account_type.as_deref().unwrap_or("unknown");
        let created_ts = chrono::DateTime::from_timestamp(user.created_at, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "Invalid date".to_string());
        println!("{:<30} {:<20} {:<8} {:<12} {}", email, user.username, is_admin, account_type, created_ts);
    }
}
