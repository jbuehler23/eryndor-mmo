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

    info!("Login attempt from client {:?}: {}", client_entity, request.username);

    // Verify credentials (blocking for simplicity in POC)
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime.block_on(database::verify_credentials(pool, &request.username, &request.password));

    match result {
        Ok(account_id) => {
            info!("Login successful for {}", request.username);

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

            // Determine visual shape based on class
            let visual = match character.class {
                CharacterClass::Rogue => VisualShape {
                    shape_type: ShapeType::Triangle,
                    color: COLOR_PLAYER,
                    size: PLAYER_SIZE,
                },
                CharacterClass::Mage => VisualShape {
                    shape_type: ShapeType::Circle,
                    color: COLOR_PLAYER,
                    size: PLAYER_SIZE,
                },
                CharacterClass::Knight => VisualShape {
                    shape_type: ShapeType::Square,
                    color: COLOR_PLAYER,
                    size: PLAYER_SIZE,
                },
            };

            // Grant class-based starting abilities
            let mut learned_abilities = LearnedAbilities::default();
            let mut hotbar = Hotbar::default();

            for (i, ability_id) in character.class.starting_abilities().iter().enumerate() {
                learned_abilities.learn(*ability_id);
                // Add to hotbar automatically
                if i < hotbar.slots.len() {
                    hotbar.slots[i] = Some(HotbarSlot::Ability(*ability_id));
                }
            }

            // Spawn character entity
            let character_entity = commands.spawn((
                Replicated,
                Player,
                character,
                position,
                Velocity::default(),
                MoveSpeed::default(),
                health,
                mana,
                CombatStats::default(),
                CurrentTarget::default(),
                InCombat(false),
                Inventory::new(MAX_INVENTORY_SLOTS),
                Equipment::default(),
                hotbar,
                learned_abilities,
            )).id();

            commands.entity(character_entity).insert((
                QuestLog::default(),
                AbilityCooldowns::default(),
                visual,
                OwnedBy(client_entity),
                CharacterDatabaseId(request.character_id),
            ));

            // Link client to character
            commands.entity(client_entity).insert(ActiveCharacterEntity(character_entity));

            // Tell the client which entity is theirs
            commands.server_trigger(ToClients {
                mode: SendMode::Direct(ClientId::Client(client_entity)),
                message: SelectCharacterResponse {
                    character_entity,
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
    active_chars: Query<&ActiveCharacterEntity>,
    characters: Query<(Entity, &Position, &Health, &Mana, &CharacterDatabaseId)>,
    db: Res<DatabaseConnection>,
) {
    let Some(pool) = db.pool() else { return };

    for client_entity in disconnected.read() {
        info!("Client disconnected: {:?}", client_entity);

        // Find and despawn their character
        if let Ok(active) = active_chars.get(client_entity) {
            let char_entity = active.0;

            if let Ok((entity, position, health, mana, db_id)) = characters.get(char_entity) {
                // Save character to database
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let _ = runtime.block_on(database::save_character(
                    pool,
                    db_id.0,
                    position,
                    health,
                    mana,
                ));

                // Despawn character
                commands.entity(entity).despawn();
                info!("Character saved and despawned");
            }
        }
    }
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
