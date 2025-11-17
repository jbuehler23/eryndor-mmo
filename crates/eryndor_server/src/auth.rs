use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon::shared::backend::connected_client::{NetworkId, NetworkIdMap};
use bevy_renet2::prelude::ServerEvent;
use bevy_renet2::netcode::NetcodeServerTransport;
use eryndor_shared::*;
use crate::database::{self, DatabaseConnection};
use crate::config::ServerConfig;
use sqlx::Row;
use std::net::IpAddr;

/// Component marking authenticated clients
#[derive(Component)]
pub struct Authenticated {
    pub account_id: i64,
}

/// Component marking clients with an active character
#[derive(Component)]
pub struct ActiveCharacterEntity(pub Entity);

/// Component storing character database ID
#[derive(Component)]
pub struct CharacterDatabaseId(pub i64);

/// Component storing client connection metadata
#[derive(Component, Debug)]
pub struct ClientMetadata {
    pub ip_address: IpAddr,
    pub socket_id: usize,  // 0=UDP, 1=WebTransport, 2=WebSocket
    pub connect_time: std::time::SystemTime,
}

/// System that captures client IP addresses when they connect
/// and enforces IP bans
pub fn track_client_connections(
    mut server_events: MessageReader<ServerEvent>,
    network_id_map: Res<NetworkIdMap>,
    transport: Res<NetcodeServerTransport>,
    db: Res<DatabaseConnection>,
    mut commands: Commands,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                // Look up the entity using NetworkIdMap
                if let Some(&client_entity) = network_id_map.get(&NetworkId::new(*client_id)) {
                    // Extract IP address from transport
                    if let Some((socket_id, socket_addr)) = transport.client_addr(*client_id) {
                        let ip_address = socket_addr.ip();

                        let transport_name = match socket_id {
                            0 => "UDP",
                            1 => "WebTransport",
                            2 => "WebSocket",
                            _ => "Unknown"
                        };

                        info!(
                            "Client {:?} connected from IP {} via socket {} ({})",
                            client_id,
                            ip_address,
                            socket_id,
                            transport_name
                        );

                        // CHECK FOR IP BAN
                        if let Some(pool) = db.pool() {
                            let runtime = tokio::runtime::Runtime::new().unwrap();
                            let ip_str = ip_address.to_string();

                            let ban_check = runtime.block_on(database::check_ip_ban(pool, &ip_str));
                            if let Ok(Some(ban_info)) = ban_check {
                                let message = if ban_info.is_permanent {
                                    format!("This IP address has been permanently banned. Reason: {}", ban_info.reason)
                                } else if let Some(expires_at) = ban_info.expires_at {
                                    let expires_date = chrono::DateTime::from_timestamp(expires_at, 0)
                                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                                        .unwrap_or_else(|| "unknown time".to_string());
                                    format!("This IP address is banned until {}. Reason: {}", expires_date, ban_info.reason)
                                } else {
                                    format!("This IP address has been banned. Reason: {}", ban_info.reason)
                                };

                                warn!("Banned IP {} attempted connection via {} - disconnecting", ip_address, transport_name);
                                warn!("Ban details: {}", message);

                                // Disconnect the client immediately
                                commands.entity(client_entity).despawn();
                                continue; // Skip adding metadata and continue to next event
                            }

                            // Attach metadata to client entity
                            commands.entity(client_entity).insert(ClientMetadata {
                                ip_address,
                                socket_id,
                                connect_time: std::time::SystemTime::now(),
                            });
                        }
                    } else {
                        warn!("Could not get address for client {:?}", client_id);
                    }
                } else {
                    warn!("No entity found for client {:?}", client_id);
                }
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Client {:?} disconnected: {}", client_id, reason);
            }
        }
    }
}

pub fn handle_login(
    trigger: On<FromClient<LoginRequest>>,
    mut commands: Commands,
    db: Res<DatabaseConnection>,
    authenticated_clients: Query<&Authenticated>,
    rate_limiters: Res<crate::RateLimiters>,
    client_metadata: Query<&ClientMetadata>,
) {
    info!("handle_login observer triggered!");
    let Some(pool) = db.pool() else {
        warn!("Database pool not available");
        return
    };

    let Some(client_entity) = trigger.client_id.entity() else {
        warn!("No client entity in trigger");
        return
    };

    // RATE LIMIT CHECK
    let Ok(metadata) = client_metadata.get(client_entity) else {
        warn!("No IP address for client {:?}", client_entity);
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(trigger.client_id),
            message: LoginResponse {
                success: false,
                message: "Connection error. Please try again.".to_string(),
                account_id: None,
                is_admin: false,
            },
        });
        return;
    };

    if rate_limiters.login_attempts.check_key(&metadata.ip_address).is_err() {
        warn!("Rate limit exceeded for login from IP: {}", metadata.ip_address);

        // Log violation to database
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let _ = runtime.block_on(database::log_rate_limit_violation(
            pool,
            &metadata.ip_address.to_string(),
            "login_attempt",
            "Rate limit exceeded"
        ));

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(trigger.client_id),
            message: LoginResponse {
                success: false,
                message: "Too many login attempts. Please try again later.".to_string(),
                account_id: None,
                is_admin: false,
            },
        });
        return;
    }

    let request = trigger.event();

    info!("Login attempt from client {:?}: username={} (IP: {})", client_entity, request.username, metadata.ip_address);

    // Verify credentials (blocking for simplicity in POC)
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime.block_on(database::verify_credentials(pool, &request.username, &request.password));

    match result {
        Ok(account_id) => {
            // CHECK FOR ACCOUNT BAN
            let ban_check = runtime.block_on(database::check_account_ban(pool, account_id));
            if let Ok(Some(ban_info)) = ban_check {
                let message = if ban_info.is_permanent {
                    format!("Your account has been permanently banned. Reason: {}", ban_info.reason)
                } else if let Some(expires_at) = ban_info.expires_at {
                    let expires_date = chrono::DateTime::from_timestamp(expires_at, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                        .unwrap_or_else(|| "unknown time".to_string());
                    format!("Your account is banned until {}. Reason: {}", expires_date, ban_info.reason)
                } else {
                    format!("Your account has been banned. Reason: {}", ban_info.reason)
                };

                warn!("Banned account {} (ID: {}) attempted login", request.username, account_id);
                commands.server_trigger(ToClients {
                    mode: SendMode::Direct(ClientId::Client(client_entity)),
                    message: LoginResponse {
                        success: false,
                        message,
                        account_id: None,
                        is_admin: false,
                    },
                });
                return;
            }

            // Check if this account is already logged in
            let already_logged_in = authenticated_clients.iter().any(|auth| auth.account_id == account_id);

            if already_logged_in {
                warn!("Account {} (ID: {}) is already logged in, rejecting duplicate login", request.username, account_id);
                commands.server_trigger(ToClients {
                    mode: SendMode::Direct(ClientId::Client(client_entity)),
                    message: LoginResponse {
                        success: false,
                        message: "This account is already logged in".to_string(),
                        account_id: None,
                        is_admin: false,
                    },
                });
                return;
            }

            info!("Login successful for {} (ID: {})", request.username, account_id);

            // Check if user is admin
            let is_admin = runtime.block_on(crate::admin::is_admin(pool, account_id))
                .unwrap_or(false);

            // Mark client as authenticated
            commands.entity(client_entity).insert(Authenticated { account_id });

            // Send success response
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: LoginResponse {
                    success: true,
                    message: "Login successful".to_string(),
                    account_id: Some(account_id),
                    is_admin,
                },
            });

            // Load and send character list
            let chars_result = runtime.block_on(database::get_characters(pool, account_id));
            if let Ok(characters) = chars_result {
                commands.server_trigger(ToClients {
                    mode: SendMode::Direct(ClientId::Client(client_entity)),
                    message: CharacterListResponse { characters },
                });
            }
        }
        Err(e) => {
            warn!("Login failed for {}: {}", request.username, e);
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: LoginResponse {
                    success: false,
                    message: e,
                    account_id: None,
                    is_admin: false,
                },
            });
        }
    }
}

pub fn handle_create_account(
    trigger: On<FromClient<CreateAccountRequest>>,
    mut commands: Commands,
    db: Res<DatabaseConnection>,
    rate_limiters: Res<crate::RateLimiters>,
    client_metadata: Query<&ClientMetadata>,
) {
    info!("handle_create_account observer triggered!");
    let Some(pool) = db.pool() else {
        warn!("Database pool not available");
        return
    };

    let Some(client_entity) = trigger.client_id.entity() else {
        warn!("No client entity in trigger");
        return
    };

    // RATE LIMIT CHECK
    let Ok(metadata) = client_metadata.get(client_entity) else {
        warn!("No IP address for client {:?}", client_entity);
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: CreateAccountResponse {
                success: false,
                message: "Connection error. Please try again.".to_string(),
            },
        });
        return;
    };

    if rate_limiters.account_creation.check_key(&metadata.ip_address).is_err() {
        warn!("Rate limit exceeded for account creation from IP: {}", metadata.ip_address);

        // Log violation to database
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let _ = runtime.block_on(database::log_rate_limit_violation(
            pool,
            &metadata.ip_address.to_string(),
            "account_creation",
            "Rate limit exceeded"
        ));

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: CreateAccountResponse {
                success: false,
                message: "Too many account creation attempts. Please try again later.".to_string(),
            },
        });
        return;
    }

    let request = trigger.event();

    info!("Account creation attempt: {} (email: {}) (IP: {})", request.username, request.email, metadata.ip_address);

    // Validate email
    if !request.email.contains('@') || request.email.len() < 5 {
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: CreateAccountResponse {
                success: false,
                message: "Invalid email address".to_string(),
            },
        });
        return;
    }

    // CONTENT MODERATION CHECK - Username validation with profanity filtering
    let moderation_result = crate::moderation::check_username(&request.username);
    if !moderation_result.is_appropriate {
        warn!("Account creation rejected - inappropriate username: {}", request.username);

        // AUDIT LOG: Inappropriate content blocked
        let runtime_audit = tokio::runtime::Runtime::new().unwrap();
        let _ = runtime_audit.block_on(crate::audit::log_audit_event(
            pool,
            crate::audit::AuditActionType::InappropriateContentBlocked,
            None,
            None,
            Some(&request.username),
            Some(&metadata.ip_address.to_string()),
            Some(&format!("username moderation failed: {}", moderation_result.reason.clone().unwrap_or_default())),
        ));

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: CreateAccountResponse {
                success: false,
                message: moderation_result.reason.unwrap_or_else(|| "Username is not appropriate".to_string()),
            },
        });
        return;
    }

    // Use the validated/trimmed username from moderation
    let validated_username = moderation_result.filtered_text;

    // Validate password complexity (8+ chars, 1+ number, 1+ uppercase)
    if request.password.len() < 8 {
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: CreateAccountResponse {
                success: false,
                message: "Password must be at least 8 characters".to_string(),
            },
        });
        return;
    }

    if !request.password.chars().any(|c| c.is_numeric()) {
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: CreateAccountResponse {
                success: false,
                message: "Password must contain at least one number".to_string(),
            },
        });
        return;
    }

    if !request.password.chars().any(|c| c.is_uppercase()) {
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: CreateAccountResponse {
                success: false,
                message: "Password must contain at least one uppercase letter".to_string(),
            },
        });
        return;
    }

    let password_hash = hash_password(&request.password);

    let runtime = tokio::runtime::Runtime::new().unwrap();
    // Use validated_username from moderation check
    let result = runtime.block_on(database::create_account(pool, &request.email, &validated_username, &password_hash));

    match result {
        Ok(account_id) => {
            info!("Account created: {} ({})", validated_username, request.email);

            // AUDIT LOG: Account creation
            let runtime_audit = tokio::runtime::Runtime::new().unwrap();
            let _ = runtime_audit.block_on(crate::audit::log_audit_event(
                pool,
                crate::audit::AuditActionType::AccountCreated,
                Some(account_id),
                Some(account_id),
                Some(&validated_username),
                Some(&metadata.ip_address.to_string()),
                Some(&format!("email: {}", request.email)),
            ));

            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: CreateAccountResponse {
                    success: true,
                    message: "Account created successfully! Please log in.".to_string(),
                },
            });
        }
        Err(e) => {
            warn!("Account creation failed: {}", e);

            // Parse the error to provide more accurate messaging
            let error_message = if e.contains("no such column") {
                // Database schema issue - needs migration
                "Server database needs updating. Please contact administrator or restart the server.".to_string()
            } else if e.contains("UNIQUE constraint failed") {
                // Parse which field caused the unique constraint violation
                if e.contains("email") {
                    "An account with this email already exists".to_string()
                } else if e.contains("username") {
                    "An account with this username already exists".to_string()
                } else {
                    "Email or username already exists".to_string()
                }
            } else {
                // Generic database error
                "Failed to create account. Please try again or contact administrator.".to_string()
            };

            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: CreateAccountResponse {
                    success: false,
                    message: error_message,
                },
            });
        }
    }
}

pub fn handle_create_character(
    trigger: On<FromClient<CreateCharacterRequest>>,
    mut commands: Commands,
    clients: Query<&Authenticated>,
    db: Res<DatabaseConnection>,
) {
    let Some(pool) = db.pool() else { return };

    let Some(client_entity) = trigger.client_id.entity() else { return };
    let request = trigger.event();

    // Check if client is authenticated
    let Ok(auth) = clients.get(client_entity) else {
        warn!("Unauthenticated client tried to create character");
        return;
    };

    info!("Character creation: {} as {:?}", request.name, request.class);

    // CONTENT MODERATION CHECK
    let moderation_result = crate::moderation::check_character_name(&request.name);
    if !moderation_result.is_appropriate {
        warn!("Character creation rejected - inappropriate name: {}", request.name);
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: CreateCharacterResponse {
                success: false,
                message: moderation_result.reason.unwrap_or_else(|| "Character name is not appropriate".to_string()),
                character: None,
            },
        });
        return;
    }

    // Use the filtered/trimmed name from moderation
    let validated_name = moderation_result.filtered_text;

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime.block_on(database::create_character(
        pool,
        auth.account_id,
        &validated_name,  // Use the validated/filtered name from moderation
        request.class,
    ));

    match result {
        Ok(character_data) => {
            info!("Character created: {}", request.name);
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: CreateCharacterResponse {
                    success: true,
                    message: "Character created successfully!".to_string(),
                    character: Some(character_data),
                },
            });
        }
        Err(e) => {
            warn!("Character creation failed: {}", e);
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: CreateCharacterResponse {
                    success: false,
                    message: "Character name already exists".to_string(),
                    character: None,
                },
            });
        }
    }
}

pub fn handle_select_character(
    trigger: On<FromClient<SelectCharacterRequest>>,
    mut commands: Commands,
    clients: Query<&Authenticated>,
    db: Res<DatabaseConnection>,
) {
    let Some(pool) = db.pool() else { return };

    let Some(client_entity) = trigger.client_id.entity() else { return };
    let request = trigger.event();

    // Check if client is authenticated
    let Ok(auth) = clients.get(client_entity) else {
        warn!("Unauthenticated client tried to select character");
        return;
    };

    info!("Character selection: ID {}", request.character_id);

    // Load character from database
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime.block_on(database::load_character(pool, request.character_id));

    match result {
        Ok((character, position, health, mana)) => {
            info!("Spawning character: {}", character.name);

            // Load equipment, inventory, and quest log from database
            let equipment = runtime.block_on(database::load_equipment(pool, request.character_id))
                .unwrap_or_else(|e| {
                    warn!("Failed to load equipment: {}, using defaults", e);
                    Equipment::default()
                });

            let inventory = runtime.block_on(database::load_inventory(pool, request.character_id))
                .unwrap_or_else(|e| {
                    warn!("Failed to load inventory: {}, using defaults", e);
                    Inventory::new(MAX_INVENTORY_SLOTS)
                });

            let quest_log = runtime.block_on(database::load_quest_log(pool, request.character_id))
                .unwrap_or_else(|e| {
                    warn!("Failed to load quest log: {}, using defaults", e);
                    QuestLog::default()
                });

            let hotbar = runtime.block_on(database::load_hotbar(pool, request.character_id))
                .unwrap_or_else(|e| {
                    warn!("Failed to load hotbar: {}, using defaults", e);
                    Hotbar::default()
                });

            let learned_abilities = runtime.block_on(database::load_learned_abilities(pool, request.character_id))
                .unwrap_or_else(|e| {
                    warn!("Failed to load learned abilities: {}, using defaults", e);
                    LearnedAbilities::default()
                });

            // Load progression data
            let (experience, weapon_prof, weapon_exp, armor_prof, armor_exp, unlocked_passives) =
                runtime.block_on(database::load_progression(pool, request.character_id))
                .unwrap_or_else(|e| {
                    warn!("Failed to load progression: {}, using defaults", e);
                    // Return defaults based on character level
                    let level = 1; // Will be overridden by Experience::new
                    (
                        Experience::new(level),
                        WeaponProficiency::default(),
                        WeaponProficiencyExp::default(),
                        ArmorProficiency::default(),
                        ArmorProficiencyExp::default(),
                        UnlockedArmorPassives::default(),
                    )
                });

            // Use character module to spawn character with all components
            let character_entity = crate::character::spawn_character_components(
                &mut commands,
                character,
                position,
                health,
                mana,
                equipment,
                inventory,
                quest_log,
                client_entity,
                request.character_id,
            );

            // Override default progression components with loaded data
            commands.entity(character_entity).insert((
                experience,
                weapon_prof,
                weapon_exp,
                armor_prof,
                armor_exp,
                unlocked_passives,
                hotbar,
                learned_abilities,
            ));

            // Link client to character
            commands.entity(client_entity).insert(ActiveCharacterEntity(character_entity));

            // Tell the client which character was selected
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: SelectCharacterResponse {
                    character_id: request.character_id,
                },
            });

            info!("Character spawned: entity {:?}", character_entity);
        }
        Err(e) => {
            warn!("Failed to load character: {}", e);
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: NotificationEvent {
                    message: "Failed to load character".to_string(),
                    notification_type: NotificationType::Error,
                },
            });
        }
    }
}

pub fn handle_client_disconnect(
    mut commands: Commands,
    mut disconnected: RemovedComponents<ConnectedClient>,
    authenticated: Query<&Authenticated>,
    characters: Query<(
        Entity,
        &OwnedBy,
        &Character,
        &Position,
        &Health,
        &Mana,
        &CharacterDatabaseId,
        &Equipment,
        &Inventory,
        &QuestLog,
    )>,
    progression: Query<(
        &Experience,
        &WeaponProficiency,
        &WeaponProficiencyExp,
        &ArmorProficiency,
        &ArmorProficiencyExp,
        &UnlockedArmorPassives,
    )>,
    db: Res<DatabaseConnection>,
) {
    let Some(pool) = db.pool() else { return };

    for client_entity in disconnected.read() {
        // Log account info if available
        if let Ok(auth) = authenticated.get(client_entity) {
            info!("Client disconnected: {:?} (Account ID: {})", client_entity, auth.account_id);
        } else {
            info!("Client disconnected: {:?} (not authenticated)", client_entity);
        }

        // Find and despawn their character using OwnedBy component
        let mut found_character = false;
        for (entity, owned_by, character, position, health, mana, db_id, equipment, inventory, quest_log) in characters.iter() {
            if owned_by.0 == client_entity {
                // Get progression components
                let Ok((experience, weapon_prof, weapon_exp, armor_prof, armor_exp, unlocked_passives)) =
                    progression.get(entity) else {
                    error!("Failed to get progression components for character '{}'", character.name);
                    continue;
                };
                found_character = true;
                info!("Saving character '{}' (DB ID: {}) at position ({:.1}, {:.1})",
                    character.name, db_id.0, position.0.x, position.0.y);

                // Save all character data to database
                let runtime = tokio::runtime::Runtime::new().unwrap();

                // Save basic character data
                match runtime.block_on(database::save_character(
                    pool,
                    db_id.0,
                    position,
                    health,
                    mana,
                )) {
                    Ok(_) => info!("Character '{}' basic data saved", character.name),
                    Err(e) => error!("Failed to save character '{}': {}", character.name, e),
                }

                // Save equipment
                match runtime.block_on(database::save_equipment(pool, db_id.0, equipment)) {
                    Ok(_) => info!("Character '{}' equipment saved", character.name),
                    Err(e) => error!("Failed to save equipment for '{}': {}", character.name, e),
                }

                // Save inventory
                match runtime.block_on(database::save_inventory(pool, db_id.0, inventory)) {
                    Ok(_) => info!("Character '{}' inventory saved", character.name),
                    Err(e) => error!("Failed to save inventory for '{}': {}", character.name, e),
                }

                // Save quest log
                match runtime.block_on(database::save_quest_log(pool, db_id.0, quest_log)) {
                    Ok(_) => info!("Character '{}' quest log saved", character.name),
                    Err(e) => error!("Failed to save quest log for '{}': {}", character.name, e),
                }

                // Save progression data
                match runtime.block_on(database::save_progression(
                    pool,
                    db_id.0,
                    character.level,
                    experience,
                    weapon_prof,
                    weapon_exp,
                    armor_prof,
                    armor_exp,
                    unlocked_passives,
                )) {
                    Ok(_) => info!("Character '{}' progression saved (level: {})", character.name, character.level),
                    Err(e) => error!("Failed to save progression for '{}': {}", character.name, e),
                }

                // Despawn character - this will replicate to all clients
                commands.entity(entity).despawn();
                info!("Character '{}' despawned from world (will replicate to all clients)", character.name);
                break;
            }
        }

        if !found_character {
            info!("Client had no active character");
        }
    }
}

pub fn handle_disconnect_character(
    trigger: On<FromClient<DisconnectCharacterRequest>>,
    mut commands: Commands,
    clients: Query<&Authenticated>,
    characters: Query<(
        Entity,
        &OwnedBy,
        &Character,
        &Position,
        &Health,
        &Mana,
        &CharacterDatabaseId,
        &Equipment,
        &Inventory,
        &QuestLog,
    )>,
    progression: Query<(
        &Experience,
        &WeaponProficiency,
        &WeaponProficiencyExp,
        &ArmorProficiency,
        &ArmorProficiencyExp,
        &UnlockedArmorPassives,
    )>,
    db: Res<DatabaseConnection>,
) {
    let Some(pool) = db.pool() else { return };

    let Some(client_entity) = trigger.client_id.entity() else { return };

    // Check if client is authenticated
    let Ok(auth) = clients.get(client_entity) else {
        warn!("Unauthenticated client tried to disconnect character");
        return;
    };

    info!("Client {:?} (Account ID: {}) requested disconnect from character", client_entity, auth.account_id);

    // Find and save/despawn their character
    for (entity, owned_by, character, position, health, mana, db_id, equipment, inventory, quest_log) in characters.iter() {
        if owned_by.0 == client_entity {
            // Get progression components
            let Ok((experience, weapon_prof, weapon_exp, armor_prof, armor_exp, unlocked_passives)) =
                progression.get(entity) else {
                error!("Failed to get progression components for character '{}'", character.name);
                // Remove ActiveCharacterEntity link even if we fail to save progression
                commands.entity(client_entity).remove::<ActiveCharacterEntity>();
                return;
            };
            info!("Disconnecting character '{}' (DB ID: {}) from client {:?}", character.name, db_id.0, client_entity);

            // Save all character data to database
            let runtime = tokio::runtime::Runtime::new().unwrap();

            // Save basic character data
            match runtime.block_on(database::save_character(
                pool,
                db_id.0,
                position,
                health,
                mana,
            )) {
                Ok(_) => info!("Character '{}' basic data saved", character.name),
                Err(e) => error!("Failed to save character '{}': {}", character.name, e),
            }

            // Save equipment
            match runtime.block_on(database::save_equipment(pool, db_id.0, equipment)) {
                Ok(_) => info!("Character '{}' equipment saved", character.name),
                Err(e) => error!("Failed to save equipment for '{}': {}", character.name, e),
            }

            // Save inventory
            match runtime.block_on(database::save_inventory(pool, db_id.0, inventory)) {
                Ok(_) => info!("Character '{}' inventory saved", character.name),
                Err(e) => error!("Failed to save inventory for '{}': {}", character.name, e),
            }

            // Save quest log
            match runtime.block_on(database::save_quest_log(pool, db_id.0, quest_log)) {
                Ok(_) => info!("Character '{}' quest log saved", character.name),
                Err(e) => error!("Failed to save quest log for '{}': {}", character.name, e),
            }

            // Save progression data
            match runtime.block_on(database::save_progression(
                pool,
                db_id.0,
                character.level,
                experience,
                weapon_prof,
                weapon_exp,
                armor_prof,
                armor_exp,
                unlocked_passives,
            )) {
                Ok(_) => info!("Character '{}' progression saved (level: {})", character.name, character.level),
                Err(e) => error!("Failed to save progression for '{}': {}", character.name, e),
            }

            // Despawn character - this will replicate to all clients
            commands.entity(entity).despawn();
            info!("Character '{}' despawned (will replicate to all clients)", character.name);

            // Remove ActiveCharacterEntity link
            commands.entity(client_entity).remove::<ActiveCharacterEntity>();

            return;
        }
    }

    warn!("Client {:?} requested disconnect but had no active character", client_entity);
}

// Simple password hashing (use argon2 in production)
fn hash_password(password: &str) -> String {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

// ============================================================================
// OAUTH AUTHENTICATION HANDLERS
// ============================================================================

pub fn handle_oauth_login(
    trigger: On<FromClient<OAuthLoginRequest>>,
    mut commands: Commands,
    db: Res<DatabaseConnection>,
    config: Res<ServerConfig>,
    rate_limiters: Res<crate::RateLimiters>,
    client_metadata: Query<&ClientMetadata>,
) {
    info!("handle_oauth_login observer triggered!");

    let Some(pool) = db.pool() else {
        warn!("Database pool not available");
        return;
    };

    let Some(client_entity) = trigger.client_id.entity() else {
        warn!("No client entity in trigger");
        return;
    };

    // RATE LIMIT CHECK
    let Ok(metadata) = client_metadata.get(client_entity) else {
        warn!("No IP address for client {:?}", client_entity);
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: OAuthLoginResponse {
                success: false,
                message: "Connection error. Please try again.".to_string(),
                account_id: None,
            },
        });
        return;
    };

    if rate_limiters.login_attempts.check_key(&metadata.ip_address).is_err() {
        warn!("Rate limit exceeded for OAuth login from IP: {}", metadata.ip_address);

        // Log violation to database
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let _ = runtime.block_on(database::log_rate_limit_violation(
            pool,
            &metadata.ip_address.to_string(),
            "oauth_login_attempt",
            "Rate limit exceeded"
        ));

        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: OAuthLoginResponse {
                success: false,
                message: "Too many login attempts. Please try again later.".to_string(),
                account_id: None,
            },
        });
        return;
    }

    let request = trigger.event();
    info!("OAuth login attempt with provider: {} (IP: {})", request.provider, metadata.ip_address);

    if request.provider != "google" {
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: OAuthLoginResponse {
                success: false,
                message: "Unsupported OAuth provider".to_string(),
                account_id: None,
            },
        });
        return;
    }

    // Check if OAuth is enabled
    if !config.oauth.is_google_enabled() {
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: OAuthLoginResponse {
                success: false,
                message: "Google OAuth is not configured".to_string(),
                account_id: None,
            },
        });
        return;
    }

    // Verify the token with Google
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let token = request.token.clone();
    let client_id = config.oauth.google_client_id.clone();

    // Verify token against Google's API
    let verification_result = runtime.block_on(async {
        let client = reqwest::Client::new();
        let response = client
            .get("https://oauth2.googleapis.com/tokeninfo")
            .query(&[("access_token", &token)])
            .send()
            .await;

        let response = match response {
            Ok(r) => r,
            Err(e) => return Err(format!("Failed to verify token: {}", e)),
        };

        if !response.status().is_success() {
            return Err("Token verification failed".to_string());
        }

        let token_info: serde_json::Value = match response.json().await {
            Ok(info) => info,
            Err(e) => return Err(format!("Failed to parse token info: {}", e)),
        };

        // Verify the token is for our app
        let aud = token_info["aud"].as_str().unwrap_or("");
        if aud != client_id {
            return Err("Invalid audience".to_string());
        }

        // Extract user info
        let google_id = token_info["sub"].as_str().unwrap_or("").to_string();
        let email = token_info["email"].as_str().unwrap_or("").to_string();
        let name = token_info["name"].as_str().unwrap_or("Google User").to_string();

        if google_id.is_empty() || email.is_empty() {
            return Err("Missing required user information".to_string());
        }

        Ok((google_id, email, name))
    });

    match verification_result {
        Ok((google_id, email, name)) => {
            // Check if account with this OAuth ID already exists
            let account_result = runtime.block_on(
                database::find_account_by_oauth(pool, "google", &google_id)
            );

            let account_id = match account_result {
                Ok(Some(id)) => {
                    // Account exists, log them in
                    info!("Existing OAuth account logged in: {}", email);
                    id
                }
                Ok(None) => {
                    // New OAuth user - create account
                    match runtime.block_on(
                        database::create_oauth_account(pool, &email, &name, "google", &google_id)
                    ) {
                        Ok(id) => {
                            info!("New OAuth account created: {}", email);
                            id
                        }
                        Err(e) => {
                            error!("Failed to create OAuth account: {}", e);
                            commands.server_trigger(ToClients {
                                mode: SendMode::Direct(ClientId::Client(client_entity)),
                                message: OAuthLoginResponse {
                                    success: false,
                                    message: "Failed to create account".to_string(),
                                    account_id: None,
                                },
                            });
                            return;
                        }
                    }
                }
                Err(e) => {
                    error!("Database error: {}", e);
                    commands.server_trigger(ToClients {
                        mode: SendMode::Direct(ClientId::Client(client_entity)),
                        message: OAuthLoginResponse {
                            success: false,
                            message: "Database error".to_string(),
                            account_id: None,
                        },
                    });
                    return;
                }
            };

            // CHECK FOR ACCOUNT BAN
            let ban_check = runtime.block_on(database::check_account_ban(pool, account_id));
            if let Ok(Some(ban_info)) = ban_check {
                let message = if ban_info.is_permanent {
                    format!("Your account has been permanently banned. Reason: {}", ban_info.reason)
                } else if let Some(expires_at) = ban_info.expires_at {
                    let expires_date = chrono::DateTime::from_timestamp(expires_at, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                        .unwrap_or_else(|| "unknown time".to_string());
                    format!("Your account is banned until {}. Reason: {}", expires_date, ban_info.reason)
                } else {
                    format!("Your account has been banned. Reason: {}", ban_info.reason)
                };

                warn!("Banned account (ID: {}) attempted OAuth login", account_id);
                commands.server_trigger(ToClients {
                    mode: SendMode::Direct(ClientId::Client(client_entity)),
                    message: OAuthLoginResponse {
                        success: false,
                        message,
                        account_id: None,
                    },
                });
                return;
            }

            // Send character list
            match runtime.block_on(database::get_characters(pool, account_id)) {
                Ok(characters) => {
                    commands.server_trigger(ToClients {
                        mode: SendMode::Direct(ClientId::Client(client_entity)),
                        message: CharacterListResponse { characters },
                    });
                }
                Err(e) => {
                    error!("Failed to fetch characters: {}", e);
                }
            }

            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: OAuthLoginResponse {
                    success: true,
                    message: "Login successful!".to_string(),
                    account_id: Some(account_id),
                },
            });
        }
        Err(e) => {
            warn!("OAuth verification failed: {}", e);
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: OAuthLoginResponse {
                    success: false,
                    message: format!("Authentication failed: {}", e),
                    account_id: None,
                },
            });
        }
    }
}
