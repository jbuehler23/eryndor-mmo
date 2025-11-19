# Eryndor MMO - Security & User Management Implementation Plan

## Overview

This document outlines the complete security and user management system for Eryndor MMO's public playtest. The system provides:

- **Email-based registration** with 6-digit verification codes
- **Guest accounts** for immediate play with conversion to permanent accounts
- **Content moderation** with real-time profanity filtering and AI-powered toxicity detection
- **Admin dashboard** with full player management and moderation tools
- **Rate limiting** to prevent abuse
- **Comprehensive ban system** with appeals

---

## Phase 1: Database Schema

### New Tables

#### `verification_codes`
Stores encrypted email verification codes with expiration.

```sql
CREATE TABLE verification_codes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    code_encrypted BLOB NOT NULL,      -- AES-256-GCM encrypted code
    nonce BLOB NOT NULL,                -- Encryption nonce
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    attempts INTEGER DEFAULT 0,
    verified BOOLEAN DEFAULT FALSE,
    FOREIGN KEY(account_id) REFERENCES accounts(id)
);

CREATE INDEX idx_verification_account ON verification_codes(account_id);
CREATE INDEX idx_verification_expiry ON verification_codes(expires_at);
```

#### `bans`
Track IP and account bans with full audit trail.

```sql
CREATE TABLE bans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ban_type TEXT NOT NULL,             -- 'ip', 'account', 'both'
    target TEXT NOT NULL,               -- IP address or account_id
    account_id INTEGER,                 -- NULL for IP-only bans
    reason TEXT NOT NULL,
    banned_by INTEGER,                  -- admin account_id
    banned_at INTEGER NOT NULL,
    expires_at INTEGER,                 -- NULL for permanent bans
    is_active BOOLEAN DEFAULT TRUE,
    notes TEXT,

    FOREIGN KEY(account_id) REFERENCES accounts(id),
    FOREIGN KEY(banned_by) REFERENCES accounts(id)
);

CREATE INDEX idx_bans_target ON bans(target, is_active);
CREATE INDEX idx_bans_account ON bans(account_id, is_active);
CREATE INDEX idx_bans_expiry ON bans(expires_at);
```

#### `ban_appeals`
Allow players to appeal bans.

```sql
CREATE TABLE ban_appeals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ban_id INTEGER NOT NULL,
    appeal_text TEXT NOT NULL,
    submitted_at INTEGER NOT NULL,
    status TEXT DEFAULT 'pending',      -- 'pending', 'approved', 'denied'
    reviewed_by INTEGER,
    reviewed_at INTEGER,
    review_notes TEXT,

    FOREIGN KEY(ban_id) REFERENCES bans(id),
    FOREIGN KEY(reviewed_by) REFERENCES accounts(id)
);
```

#### `content_flags`
Store flagged content for human review.

```sql
CREATE TABLE content_flags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    content_type TEXT NOT NULL,         -- 'chat', 'username', 'character_name'
    content TEXT NOT NULL,
    toxicity_score REAL,                -- From OpenAI API (0.0 - 1.0)
    flagged_at INTEGER NOT NULL,
    status TEXT DEFAULT 'pending',      -- 'pending', 'approved', 'removed', 'banned'
    reviewed_by INTEGER,
    reviewed_at INTEGER,

    FOREIGN KEY(account_id) REFERENCES accounts(id),
    FOREIGN KEY(reviewed_by) REFERENCES accounts(id)
);

CREATE INDEX idx_content_flags_status ON content_flags(status);
CREATE INDEX idx_content_flags_account ON content_flags(account_id);
```

#### `admin_actions`
Audit log for all admin actions.

```sql
CREATE TABLE admin_actions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    admin_id INTEGER NOT NULL,
    action_type TEXT NOT NULL,          -- 'ban', 'unban', 'kick', 'broadcast', etc.
    target_id INTEGER,                  -- Account ID affected
    details TEXT,                       -- JSON with action details
    ip_address TEXT,
    created_at INTEGER NOT NULL,

    FOREIGN KEY(admin_id) REFERENCES accounts(id),
    FOREIGN KEY(target_id) REFERENCES accounts(id)
);

CREATE INDEX idx_admin_actions_admin ON admin_actions(admin_id);
CREATE INDEX idx_admin_actions_target ON admin_actions(target_id);
CREATE INDEX idx_admin_actions_created ON admin_actions(created_at);
```

#### `rate_limit_violations`
Track rate limit violations for monitoring and auto-banning.

```sql
CREATE TABLE rate_limit_violations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    identifier TEXT NOT NULL,           -- IP address or account_id
    violation_type TEXT NOT NULL,       -- 'account_creation', 'login', 'chat', etc.
    violated_at INTEGER NOT NULL,
    details TEXT
);

CREATE INDEX idx_rate_limit_identifier ON rate_limit_violations(identifier, violation_type);
```

### Updated `accounts` Table

Add new columns to existing accounts table:

```sql
-- Migration queries (handle existing data)
ALTER TABLE accounts ADD COLUMN email TEXT UNIQUE;
ALTER TABLE accounts ADD COLUMN email_verified BOOLEAN DEFAULT FALSE;
ALTER TABLE accounts ADD COLUMN account_type TEXT DEFAULT 'registered';  -- 'guest' or 'registered'
ALTER TABLE accounts ADD COLUMN is_admin BOOLEAN DEFAULT FALSE;
ALTER TABLE accounts ADD COLUMN guest_token TEXT UNIQUE;
ALTER TABLE accounts ADD COLUMN guest_created_at INTEGER;
ALTER TABLE accounts ADD COLUMN guest_expires_at INTEGER;
ALTER TABLE accounts ADD COLUMN last_login_at INTEGER;
ALTER TABLE accounts ADD COLUMN last_login_ip TEXT;

CREATE INDEX idx_accounts_email ON accounts(email);
CREATE INDEX idx_accounts_guest_token ON accounts(guest_token);
CREATE INDEX idx_accounts_type ON accounts(account_type);
```

---

## Phase 2: Guest Account System

### Guest Account Flow

#### 1. Guest Account Creation

**Request:** `CreateGuestAccountRequest`
```rust
struct CreateGuestAccountRequest {
    // No fields - automatic generation
}
```

**Process:**
1. Generate UUID token (128 bits)
2. Generate username: `Guest_[random_6_digits]`
3. Hash guest token before storage
4. Set 7-day expiration
5. Return guest token to client (save in local storage)

**Response:** `CreateGuestAccountResponse`
```rust
struct CreateGuestAccountResponse {
    success: bool,
    guest_token: Option<String>,
    username: String,
    expires_at: i64,
}
```

#### 2. Guest Login

**Request:** `GuestLoginRequest`
```rust
struct GuestLoginRequest {
    guest_token: String,
}
```

**Process:**
1. Hash provided token
2. Look up account by token hash
3. Check expiration
4. Return account_id if valid

#### 3. Convert Guest to Registered

**Request:** `ConvertGuestAccountRequest`
```rust
struct ConvertGuestAccountRequest {
    email: String,
    password: String,
}
```

**Process:**
1. Verify guest is authenticated
2. Validate email format
3. Check email not already in use
4. Hash password with Argon2id
5. Update account:
   - Set email
   - Set password_hash
   - Change account_type to 'registered'
   - Clear guest_token
   - Clear guest_expires_at
   - Keep same account_id (preserves all characters/progress)
6. Send verification email

**Response:** `ConvertGuestAccountResponse`
```rust
struct ConvertGuestAccountResponse {
    success: bool,
    message: String,
}
```

#### 4. Guest Cleanup System

**Daily Task** (runs every 24 hours):
1. Find all expired guest accounts
2. Delete characters and associated data
3. Delete accounts
4. Log cleanup statistics

**Warning System** (runs every 6 hours):
- Find guests expiring within 24 hours
- Send in-game notification
- Encourage account conversion

### Guest Account Limitations

To reduce moderation burden and encourage conversion:

- **No public chat** - Can only see NPC dialogue
- **No trading** - Cannot trade with other players
- **Limited inventory** - 10 slots instead of 20
- **No character naming** - Auto-generated names only
- **Time-limited** - 7 days total playtime

---

## Phase 3: Email Verification System

### Dependencies

```toml
[dependencies]
lettre = { version = "0.11", features = ["tokio1-native-tls"] }
aes-gcm = "0.10"
rand = "0.8"
```

### Email Configuration

**Environment Variables:**
```
SMTP_SERVER=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=app-password
SMTP_FROM=noreply@eryndor.com
```

**Note:** Use app-specific passwords, not account passwords. For Gmail:
1. Enable 2FA on your Google account
2. Go to App Passwords in account settings
3. Generate app password for "Mail"

### 6-Digit Code Generation

```rust
use rand::Rng;

fn generate_verification_code() -> String {
    rand::thread_rng()
        .gen_range(100000..1000000)
        .to_string()
}
```

### Code Encryption (AES-256-GCM)

**Why encrypt instead of hash:**
- Need constant-time comparison
- Codes are temporary (15 minutes)
- Must be secure but verifiable
- AES-GCM provides authentication

```rust
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce
};

fn encrypt_code(code: &str, key: &[u8; 32]) -> Result<(Vec<u8>, Vec<u8>), Error> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, code.as_bytes())?;
    Ok((ciphertext, nonce.to_vec()))
}

fn decrypt_code(ciphertext: &[u8], nonce: &[u8], key: &[u8; 32]) -> Result<String, Error> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce);
    let plaintext = cipher.decrypt(nonce, ciphertext)?;
    Ok(String::from_utf8(plaintext)?)
}
```

### Email Sending

```rust
use lettre::{Message, SmtpTransport, Transport, message::header::ContentType};

async fn send_verification_email(
    email: &str,
    code: &str,
    config: &EmailConfig,
) -> Result<(), Error> {
    let email_body = format!(
        "Welcome to Eryndor MMO!\n\n\
         Your verification code is: {}\n\n\
         This code will expire in 15 minutes.\n\n\
         If you didn't request this code, please ignore this email.",
        code
    );

    let email = Message::builder()
        .from(config.from_address.parse()?)
        .to(email.parse()?)
        .subject("Eryndor MMO - Email Verification")
        .header(ContentType::TEXT_PLAIN)
        .body(email_body)?;

    let mailer = SmtpTransport::relay(&config.smtp_server)?
        .credentials(Credentials::new(
            config.smtp_username.clone(),
            config.smtp_password.clone(),
        ))
        .build();

    mailer.send(&email)?;
    Ok(())
}
```

### Verification Flow

#### 1. Send Verification Code

**Request:** `SendVerificationCodeRequest`
```rust
struct SendVerificationCodeRequest {
    // Email already associated with account
}
```

**Process:**
1. Check rate limit (3 codes per hour per account)
2. Generate 6-digit code
3. Encrypt code with AES-256-GCM
4. Store encrypted code + nonce in database
5. Set 15-minute expiration
6. Send email via SMTP
7. Log rate limit entry

**Rate Limiting:**
- Max 3 codes per hour per account
- Min 2 minutes between requests
- Track in `rate_limit_violations` table

#### 2. Verify Code

**Request:** `VerifyEmailCodeRequest`
```rust
struct VerifyEmailCodeRequest {
    code: String,
}
```

**Process:**
1. Look up account's active verification code
2. Check expiration
3. Check attempt count (max 3 attempts)
4. Decrypt stored code
5. Constant-time comparison with provided code
6. If match:
   - Set email_verified = TRUE
   - Delete verification code
   - Send confirmation
7. If no match:
   - Increment attempts
   - If >= 3 attempts, invalidate code

**Response:** `VerifyEmailCodeResponse`
```rust
struct VerifyEmailCodeResponse {
    success: bool,
    message: String,
    attempts_remaining: Option<u8>,
}
```

---

## Phase 4: Content Moderation

### Dependencies

```toml
[dependencies]
rustrict = "0.7"
reqwest = { version = "0.12", features = ["json"] }
serde_json = "1.0"
```

### Layer 1: Real-Time Profanity Filter (rustrict)

**Usage:**
```rust
use rustrict::{CensorStr, Type};

fn moderate_text_realtime(text: &str) -> ModerationResult {
    // Check for profanity
    if text.is_inappropriate() {
        return ModerationResult::Blocked {
            reason: "Inappropriate language detected".to_string(),
            censored: text.censor(),
        };
    }

    // Check for severe violations
    if text.is(Type::PROFANE & Type::SEVERE) {
        return ModerationResult::Blocked {
            reason: "Severe profanity detected".to_string(),
            censored: text.censor(),
        };
    }

    ModerationResult::Allowed
}
```

**Applied to:**
- Character names (at creation)
- Chat messages (real-time)
- Any user-generated text

### Layer 2: AI-Powered Moderation (OpenAI API)

**Async Processing** - runs after message is allowed through Layer 1:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ModerationRequest {
    input: String,
}

#[derive(Deserialize)]
struct ModerationResponse {
    results: Vec<ModerationResult>,
}

#[derive(Deserialize)]
struct ModerationResult {
    flagged: bool,
    categories: Categories,
    category_scores: CategoryScores,
}

#[derive(Deserialize)]
struct Categories {
    hate: bool,
    #[serde(rename = "hate/threatening")]
    hate_threatening: bool,
    harassment: bool,
    #[serde(rename = "harassment/threatening")]
    harassment_threatening: bool,
    #[serde(rename = "self-harm")]
    self_harm: bool,
    sexual: bool,
    violence: bool,
}

#[derive(Deserialize)]
struct CategoryScores {
    hate: f64,
    #[serde(rename = "hate/threatening")]
    hate_threatening: f64,
    harassment: f64,
    #[serde(rename = "harassment/threatening")]
    harassment_threatening: f64,
    #[serde(rename = "self-harm")]
    self_harm: f64,
    sexual: f64,
    violence: f64,
}

async fn check_openai_moderation(text: &str, api_key: &str) -> Result<f64, Error> {
    let client = reqwest::Client::new();

    let request = ModerationRequest {
        input: text.to_string(),
    };

    let response: ModerationResponse = client
        .post("https://api.openai.com/v1/moderations")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?
        .json()
        .await?;

    // Calculate max toxicity score
    let result = &response.results[0];
    let scores = &result.category_scores;

    let max_score = [
        scores.hate,
        scores.hate_threatening,
        scores.harassment,
        scores.harassment_threatening,
        scores.violence,
    ].iter().copied().fold(0.0f64, f64::max);

    Ok(max_score)
}
```

**Process:**
1. Message passes Layer 1 (rustrict)
2. Message is sent to player
3. Spawn async task to check OpenAI API
4. If toxicity score > 0.8:
   - Insert into `content_flags` table
   - Notify moderators
   - Add strike to player account
5. If toxicity score > 0.95:
   - Auto-mute player for 24 hours
   - Flag for admin review

### Layer 3: Human Review Queue

Admin dashboard shows:
- Recent flagged content
- Player history
- AI toxicity scores
- Quick actions (approve, warn, mute, ban)

### Moderation Actions

**Progressive Punishment System:**

1. **First Offense:** Warning message
2. **Second Offense:** 1-hour mute
3. **Third Offense:** 24-hour mute
4. **Fourth Offense:** 7-day mute
5. **Fifth Offense:** Permanent ban

**Severe Violations:** Immediate permanent ban
- Racism
- Death threats
- Doxxing
- Extreme toxicity (score > 0.95)

---

## Phase 5: Rate Limiting

### Dependencies

```toml
[dependencies]
governor = "0.6"
tower-governor = "0.4"
```

### Rate Limit Configuration

```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

struct RateLimiters {
    account_creation: RateLimiter<String, DefaultKeyedStateStore<String>>,
    login_attempts: RateLimiter<String, DefaultKeyedStateStore<String>>,
    email_verification: RateLimiter<i64, DefaultKeyedStateStore<i64>>,
    chat_messages: RateLimiter<i64, DefaultKeyedStateStore<i64>>,
}

impl Default for RateLimiters {
    fn default() -> Self {
        Self {
            // 5 accounts per hour per IP
            account_creation: RateLimiter::keyed(
                Quota::per_hour(NonZeroU32::new(5).unwrap())
            ),

            // 10 login attempts per hour per IP
            login_attempts: RateLimiter::keyed(
                Quota::per_hour(NonZeroU32::new(10).unwrap())
            ),

            // 3 verification emails per hour per account
            email_verification: RateLimiter::keyed(
                Quota::per_hour(NonZeroU32::new(3).unwrap())
            ),

            // 10 chat messages per minute per player
            chat_messages: RateLimiter::keyed(
                Quota::per_minute(NonZeroU32::new(10).unwrap())
            ),
        }
    }
}
```

### Apply Rate Limiting

```rust
// Before account creation
if rate_limiters.account_creation.check_key(&client_ip).is_err() {
    return Err("Too many account creation attempts. Try again later.".to_string());
}

// Before login attempt
if rate_limiters.login_attempts.check_key(&client_ip).is_err() {
    return Err("Too many login attempts. Try again later.".to_string());
}

// Before sending verification email
if rate_limiters.email_verification.check_key(&account_id).is_err() {
    return Err("Too many verification requests. Try again later.".to_string());
}

// Before sending chat message
if rate_limiters.chat_messages.check_key(&player_id).is_err() {
    return Err("You are sending messages too quickly. Slow down!".to_string());
}
```

### Violation Tracking

When rate limit is exceeded:
1. Log to `rate_limit_violations` table
2. Check violation count in last 24 hours
3. If > 10 violations:
   - Temporary IP ban (1 hour)
4. If > 50 violations:
   - Extended IP ban (24 hours)
5. If > 100 violations:
   - Permanent IP ban + admin review

---

## Phase 6: Admin Dashboard

### Architecture

**Web Framework:** Axum
**Integration:** Embedded in Bevy server
**Port:** 8080 (configurable)

### Dependencies

```toml
[dependencies]
axum = { version = "0.7", features = ["tokio"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "fs"] }
jsonwebtoken = "9"
argon2 = "0.5"
serde_json = "1.0"
```

### Authentication System

#### JWT-Based Auth

```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,      // account_id
    role: String,     // "admin"
    exp: usize,       // expiration timestamp
}

async fn admin_login(
    pool: &SqlitePool,
    email: String,
    password: String,
) -> Result<String, Error> {
    // Verify admin credentials
    let account = sqlx::query(
        "SELECT id, password_hash FROM accounts
         WHERE email = ? AND is_admin = TRUE"
    )
    .bind(&email)
    .fetch_optional(pool)
    .await?;

    let account = account.ok_or("Invalid credentials")?;
    let account_id: i64 = account.get(0);
    let password_hash: String = account.get(1);

    // Verify password with Argon2id
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let parsed_hash = PasswordHash::new(&password_hash)?;
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash)?;

    // Generate JWT
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .unwrap()
        .timestamp();

    let claims = Claims {
        sub: account_id.to_string(),
        role: "admin".to_string(),
        exp: expiration as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes())
    )?;

    Ok(token)
}
```

#### Middleware

```rust
use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

async fn require_admin<B>(
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let token = authorization.token();

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default()
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if claims.claims.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }

    // Add account_id to request extensions
    req.extensions_mut().insert(claims.claims.sub.parse::<i64>().unwrap());

    Ok(next.run(req).await)
}
```

### API Endpoints

#### Authentication
- `POST /admin/login` - Admin login, returns JWT
- `POST /admin/logout` - Invalidate token (client-side)

#### Player Management
- `GET /admin/players` - List all players (paginated)
- `GET /admin/player/:id` - Get player details
- `POST /admin/player/:id/kick` - Kick player from server
- `POST /admin/player/:id/mute` - Mute player
- `POST /admin/player/:id/unmute` - Unmute player
- `GET /admin/player/:id/history` - View player's moderation history

#### Ban Management
- `POST /admin/ban` - Create new ban
- `GET /admin/bans` - List all bans (active/expired)
- `POST /admin/ban/:id/revoke` - Revoke ban
- `GET /admin/ban/:id/appeals` - View appeals for ban

#### Content Moderation
- `GET /admin/flags` - Get flagged content queue
- `POST /admin/flag/:id/approve` - Approve flagged content
- `POST /admin/flag/:id/remove` - Remove content and warn player
- `POST /admin/flag/:id/ban` - Ban player for content

#### Server Management
- `GET /admin/stats` - Server statistics (players online, uptime)
- `POST /admin/broadcast` - Send server-wide message
- `GET /admin/logs` - View admin action logs

### Frontend

Simple HTML/CSS/JS dashboard served at `/admin`:

**Features:**
- Login page
- Player list with search/filter
- Ban management interface
- Flagged content review queue
- Server statistics dashboard
- Admin action logs

**Tech Stack:**
- Vanilla JavaScript (no framework needed)
- Fetch API for HTTP requests
- LocalStorage for JWT token
- Tailwind CSS for styling (optional)

---

## Phase 7: In-Game Admin Commands

### Command System

Parse chat messages starting with `/`:

```rust
fn parse_admin_command(message: &str) -> Option<AdminCommand> {
    let parts: Vec<&str> = message.split_whitespace().collect();

    match parts[0] {
        "/kick" => Some(AdminCommand::Kick {
            player: parts.get(1)?.to_string(),
        }),
        "/ban" => Some(AdminCommand::Ban {
            player: parts.get(1)?.to_string(),
            duration: parts.get(2).and_then(|d| d.parse().ok()),
            reason: parts.get(3..).map(|r| r.join(" ")).unwrap_or_default(),
        }),
        "/unban" => Some(AdminCommand::Unban {
            player: parts.get(1)?.to_string(),
        }),
        "/broadcast" => Some(AdminCommand::Broadcast {
            message: parts.get(1..).map(|m| m.join(" "))?,
        }),
        "/teleport" => Some(AdminCommand::Teleport {
            target: parts.get(1)?.to_string(),
        }),
        "/god" => Some(AdminCommand::ToggleGodMode),
        "/give" => Some(AdminCommand::GiveItem {
            item_id: parts.get(1)?.parse().ok()?,
            quantity: parts.get(2).and_then(|q| q.parse().ok()).unwrap_or(1),
        }),
        _ => None,
    }
}
```

### Command Implementations

#### `/kick <player>`
1. Find player entity by name
2. Send disconnect message
3. Remove from server
4. Log action

#### `/ban <player> <duration> <reason>`
1. Find player by name
2. Create ban entry in database
3. Kick player
4. Log action

**Example:**
```
/ban ToxicPlayer 7 Repeated harassment
```
Creates 7-day ban with reason "Repeated harassment"

#### `/unban <player>`
1. Find ban by player name
2. Set ban inactive
3. Log action

#### `/broadcast <message>`
1. Send server-wide notification
2. All players see message
3. Log action

#### `/teleport <player>`
1. Find target player
2. Teleport admin to player position

#### `/god`
1. Toggle invincibility for admin
2. For testing/debugging only

#### `/give <item_id> <quantity>`
1. Add item to admin inventory
2. For testing/rewards

### Authorization Check

```rust
fn execute_admin_command(
    command: AdminCommand,
    executor_id: i64,
    pool: &SqlitePool,
) -> Result<(), String> {
    // Check if executor is admin
    let is_admin = sqlx::query_scalar::<_, bool>(
        "SELECT is_admin FROM accounts WHERE id = ?"
    )
    .bind(executor_id)
    .fetch_one(pool)
    .await?;

    if !is_admin {
        return Err("You do not have permission to use admin commands.".to_string());
    }

    // Execute command
    match command {
        AdminCommand::Kick { player } => {
            kick_player(&player, executor_id, pool).await?;
        },
        // ... other commands
    }

    // Log action
    log_admin_action(executor_id, command, pool).await?;

    Ok(())
}
```

---

## Phase 8: Security Hardening

### Password Security

**Argon2id Configuration:**
```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, ParamsBuilder,
};

fn hash_password(password: &str) -> Result<String, Error> {
    // Configure Argon2id
    let mut params = ParamsBuilder::new();
    params.m_cost(19456)?;      // 19 MiB memory
    params.t_cost(2)?;          // 2 iterations
    params.p_cost(1)?;          // 1 thread
    let params = params.build()?;

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    );

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;

    Ok(password_hash.to_string())
}
```

**Password Requirements:**
- Minimum 8 characters
- At least one uppercase letter
- At least one lowercase letter
- At least one number
- No common passwords (check against list)

### Input Validation

```rust
fn validate_email(email: &str) -> Result<(), String> {
    let email_regex = regex::Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$"
    ).unwrap();

    if !email_regex.is_match(email) {
        return Err("Invalid email format".to_string());
    }

    if email.len() > 254 {
        return Err("Email too long".to_string());
    }

    Ok(())
}

fn validate_username(username: &str) -> Result<(), String> {
    if username.len() < 3 || username.len() > 16 {
        return Err("Username must be 3-16 characters".to_string());
    }

    let username_regex = regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
    if !username_regex.is_match(username) {
        return Err("Username can only contain letters, numbers, and underscores".to_string());
    }

    // Check profanity
    if username.is_inappropriate() {
        return Err("Inappropriate username".to_string());
    }

    Ok(())
}
```

### Session Management

**Track Active Sessions:**
```sql
CREATE TABLE active_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    session_token TEXT UNIQUE NOT NULL,
    ip_address TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    last_activity INTEGER NOT NULL,

    FOREIGN KEY(account_id) REFERENCES accounts(id)
);
```

**Session Invalidation:**
- On password change
- On explicit logout
- After 24 hours of inactivity
- When IP address changes (suspicious)

### Audit Logging

Log everything important:
- Account creation/deletion
- Login attempts (success/failure)
- Password changes
- Email changes
- Admin actions
- Ban creation/removal
- Content moderation actions

```rust
async fn log_audit_event(
    pool: &SqlitePool,
    account_id: Option<i64>,
    event_type: &str,
    details: &str,
    ip_address: &str,
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO audit_log (account_id, event_type, details, ip_address, created_at)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(account_id)
    .bind(event_type)
    .bind(details)
    .bind(ip_address)
    .bind(chrono::Utc::now().timestamp())
    .execute(pool)
    .await?;

    Ok(())
}
```

---

## Phase 9: Configuration

### `config.toml`

```toml
[server]
port = 5000
max_players = 100

[email]
smtp_server = "smtp.gmail.com"
smtp_port = 587
smtp_username = "your-email@gmail.com"
smtp_password = "app-password"
from_address = "noreply@eryndor.com"
verification_expiry_minutes = 15

[moderation]
enable_realtime_filter = true
enable_openai_api = false                    # Set to true when ready
openai_api_key = ""
toxicity_threshold = 0.8
auto_ban_threshold = 0.95

[admin]
dashboard_enabled = true
dashboard_port = 8080
jwt_secret = "CHANGE_THIS_TO_RANDOM_STRING"  # Generate: openssl rand -base64 32
jwt_expiry_hours = 24

[security]
guest_expiry_days = 7
max_login_attempts = 5
login_lockout_minutes = 30
password_min_length = 8
require_email_verification = true

[rate_limits]
account_creation_per_hour = 5
login_attempts_per_hour = 10
email_verification_per_hour = 3
chat_messages_per_minute = 10

[encryption]
aes_key = "CHANGE_THIS_TO_RANDOM_KEY"        # Generate: openssl rand -hex 32
```

### Loading Configuration

```rust
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Config {
    server: ServerConfig,
    email: EmailConfig,
    moderation: ModerationConfig,
    admin: AdminConfig,
    security: SecurityConfig,
    rate_limits: RateLimitsConfig,
    encryption: EncryptionConfig,
}

fn load_config() -> Result<Config, Error> {
    let config_str = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}
```

---

## Phase 10: Testing

### Unit Tests

#### Email Verification
```rust
#[tokio::test]
async fn test_email_verification_flow() {
    // Create account
    // Send verification code
    // Verify code
    // Check email_verified flag
}

#[tokio::test]
async fn test_verification_code_expiration() {
    // Create code with past expiration
    // Attempt to verify
    // Should fail
}

#[tokio::test]
async fn test_verification_attempt_limit() {
    // Try to verify with wrong code 3 times
    // 4th attempt should fail
}
```

#### Rate Limiting
```rust
#[tokio::test]
async fn test_rate_limiting_account_creation() {
    // Try to create 6 accounts from same IP
    // 6th should be rate limited
}

#[tokio::test]
async fn test_rate_limiting_login() {
    // Try to login 11 times from same IP
    // 11th should be rate limited
}
```

#### Content Moderation
```rust
#[test]
fn test_profanity_filter() {
    let clean = "Hello world";
    assert_eq!(moderate_text_realtime(clean), ModerationResult::Allowed);

    let profane = "bad word here";
    assert!(matches!(moderate_text_realtime(profane), ModerationResult::Blocked { .. }));
}
```

#### Ban System
```rust
#[tokio::test]
async fn test_ban_creation() {
    // Create ban
    // Check player is banned
}

#[tokio::test]
async fn test_temporary_ban_expiration() {
    // Create 1-day ban
    // Fast-forward time
    // Check ban expired
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_registration_flow() {
    // 1. Create guest account
    // 2. Play for a bit
    // 3. Convert to registered
    // 4. Receive verification email
    // 5. Verify email
    // 6. Check account fully verified
}

#[tokio::test]
async fn test_admin_ban_flow() {
    // 1. Admin logs in
    // 2. Admin bans player
    // 3. Player tries to connect
    // 4. Connection rejected
}
```

---

## Phase 11: Deployment Checklist

### Pre-Launch
- [ ] Install CMAKE (required for networking)
- [ ] Set up SMTP email service
- [ ] Generate secure JWT secret
- [ ] Generate AES encryption key
- [ ] Configure `config.toml`
- [ ] Set environment variables
- [ ] Test email delivery
- [ ] Create first admin account
- [ ] Test admin dashboard access
- [ ] Review rate limits
- [ ] Test ban system

### Launch Day
- [ ] Monitor server logs
- [ ] Watch rate limit violations
- [ ] Check email delivery rate
- [ ] Monitor content flags
- [ ] Be ready to adjust rate limits
- [ ] Have admin team ready for moderation

### Post-Launch Monitoring
- [ ] Daily review of flagged content
- [ ] Weekly ban appeal reviews
- [ ] Monitor email deliverability
- [ ] Adjust rate limits based on usage
- [ ] Update profanity filter as needed

---

## Estimated Timeline

| Phase | Tasks | Estimated Time |
|-------|-------|----------------|
| Phase 1 | Database schema updates | 4-6 hours |
| Phase 2 | Guest account system | 8-10 hours |
| Phase 3 | Email verification | 8-12 hours |
| Phase 4 | Content moderation | 8-12 hours |
| Phase 5 | Rate limiting | 4-6 hours |
| Phase 6 | Admin dashboard | 16-20 hours |
| Phase 7 | Admin commands | 6-8 hours |
| Phase 8 | Security hardening | 6-8 hours |
| Phase 9 | Configuration | 2-3 hours |
| Phase 10-11 | Testing & Documentation | 8-10 hours |
| **Total** | | **70-95 hours** |

**Realistic Schedule:** 8-12 working days with breaks

---

## Future Enhancements

### Phase 12+ (Post-Launch)
- Two-factor authentication (2FA)
- OAuth login (Google, Discord)
- Advanced analytics dashboard
- Player reputation system
- Community voting for bans
- Automated chat translation
- Image content moderation
- Voice chat moderation
- Hardware ID bans
- Geolocation-based rate limiting

---

## Security Best Practices Summary

1. **Never log sensitive data** - passwords, tokens, codes
2. **Always hash passwords** - use Argon2id, not bcrypt for new projects
3. **Encrypt verification codes** - don't hash them
4. **Rate limit everything** - prevent abuse at every endpoint
5. **Validate all inputs** - never trust client data
6. **Use HTTPS in production** - Let's Encrypt certificates
7. **Rotate secrets regularly** - JWT secret, AES key
8. **Monitor everything** - logs, violations, flags
9. **Test security** - unit tests for all security features
10. **Keep dependencies updated** - security patches

---

## Support & Resources

- [Argon2 RFC](https://datatracker.ietf.org/doc/html/rfc9106)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [OpenAI Moderation API](https://platform.openai.com/docs/guides/moderation)
- [Lettre Email Library](https://docs.rs/lettre/)
- [Governor Rate Limiting](https://docs.rs/governor/)
- [Axum Web Framework](https://docs.rs/axum/)
- [JWT Best Practices](https://tools.ietf.org/html/rfc8725)

---

**Last Updated:** 2025-01-11
**Version:** 1.0
**Status:** Implementation Ready
