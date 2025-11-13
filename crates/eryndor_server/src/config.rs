use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

#[derive(Resource, Clone, Deserialize)]
pub struct ServerConfig {
    pub server: Server,
    pub admin: Admin,
    pub security: Security,
    pub rate_limits: RateLimits,
    pub moderation: Moderation,
    pub oauth: OAuth,
}

#[derive(Clone, Deserialize)]
pub struct Server {
    pub port: u16,
    pub max_players: usize,
}

#[derive(Clone, Deserialize)]
pub struct Admin {
    pub dashboard_enabled: bool,
    pub dashboard_port: u16,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
}

#[derive(Clone, Deserialize)]
pub struct Security {
    pub guest_expiry_days: i64,
    pub max_login_attempts: u32,
    pub login_lockout_minutes: i64,
    pub password_min_length: usize,
    pub require_uppercase: bool,
    pub require_numbers: bool,
}

#[derive(Clone, Deserialize)]
pub struct RateLimits {
    pub account_creation_per_hour: u32,
    pub login_attempts_per_hour: u32,
    pub chat_messages_per_minute: u32,
    pub trade_requests_per_hour: u32,
}

#[derive(Clone, Deserialize)]
pub struct Moderation {
    pub enable_profanity_filter: bool,
    pub block_profane_messages: bool,
    pub censor_instead_of_block: bool,
}

#[derive(Clone, Deserialize)]
pub struct OAuth {
    pub google_client_id: String,
    pub google_client_secret: String,
}

impl OAuth {
    pub fn is_google_enabled(&self) -> bool {
        !self.google_client_id.is_empty()
    }
}

impl ServerConfig {
    pub fn load() -> Result<Self, String> {
        let config_str = fs::read_to_string("config.toml")
            .map_err(|e| format!("Failed to read config.toml: {}", e))?;

        let config: ServerConfig = toml::from_str(&config_str)
            .map_err(|e| format!("Failed to parse config.toml: {}", e))?;

        // Validate configuration
        if config.admin.dashboard_enabled && config.admin.jwt_secret == "dev-secret-change-in-production-please" {
            warn!("WARNING: Using default JWT secret! Generate a secure one for production with: openssl rand -base64 32");
        }

        if config.security.password_min_length < 6 {
            return Err("password_min_length must be at least 6".to_string());
        }

        info!("Configuration loaded successfully");
        Ok(config)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server: Server {
                port: 5000,
                max_players: 100,
            },
            admin: Admin {
                dashboard_enabled: true,
                dashboard_port: 8080,
                jwt_secret: "dev-secret-change-in-production-please".to_string(),
                jwt_expiry_hours: 24,
            },
            security: Security {
                guest_expiry_days: 7,
                max_login_attempts: 5,
                login_lockout_minutes: 30,
                password_min_length: 8,
                require_uppercase: false,
                require_numbers: false,
            },
            rate_limits: RateLimits {
                account_creation_per_hour: 5,
                login_attempts_per_hour: 10,
                chat_messages_per_minute: 10,
                trade_requests_per_hour: 10,
            },
            moderation: Moderation {
                enable_profanity_filter: true,
                block_profane_messages: true,
                censor_instead_of_block: false,
            },
            oauth: OAuth {
                google_client_id: String::new(),
                google_client_secret: String::new(),
            },
        }
    }
}
