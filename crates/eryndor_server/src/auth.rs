use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use crate::database::{self, DatabaseConnection};

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

pub fn handle_login(
    trigger: On<FromClient<LoginRequest>>,
    mut commands: Commands,
    db: Res<DatabaseConnection>,
    authenticated_clients: Query<&Authenticated>,
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
    let request = trigger.event();

    info!("Login attempt from client {:?}: username={}", client_entity, request.username);

    // Verify credentials (blocking for simplicity in POC)
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime.block_on(database::verify_credentials(pool, &request.username, &request.password));

    match result {
        Ok(account_id) => {
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
                    },
                });
                return;
            }

            info!("Login successful for {} (ID: {})", request.username, account_id);

            // Mark client as authenticated
            commands.entity(client_entity).insert(Authenticated { account_id });

            // Send success response
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: LoginResponse {
                    success: true,
                    message: "Login successful".to_string(),
                    account_id: Some(account_id),
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
                },
            });
        }
    }
}

pub fn handle_create_account(
    trigger: On<FromClient<CreateAccountRequest>>,
    mut commands: Commands,
    db: Res<DatabaseConnection>,
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
    let request = trigger.event();

    info!("Account creation attempt: {}", request.username);

    // Validate username
    if request.username.len() < 3 || request.username.len() > 20 {
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: CreateAccountResponse {
                success: false,
                message: "Username must be 3-20 characters".to_string(),
            },
        });
        return;
    }

    let password_hash = hash_password(&request.password);

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime.block_on(database::create_account(pool, &request.username, &password_hash));

    match result {
        Ok(_account_id) => {
            info!("Account created: {}", request.username);
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
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: CreateAccountResponse {
                    success: false,
                    message: "Username already exists or invalid".to_string(),
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

    // Validate name
    if request.name.len() < 2 || request.name.len() > 20 {
        commands.server_trigger(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: CreateCharacterResponse {
                success: false,
                message: "Character name must be 2-20 characters".to_string(),
                character: None,
            },
        });
        return;
    }

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime.block_on(database::create_character(
        pool,
        auth.account_id,
        &request.name,
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
