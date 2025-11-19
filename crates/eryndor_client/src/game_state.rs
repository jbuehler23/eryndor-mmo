use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_renet2::prelude::{ConnectionConfig, RenetClient};
use bevy_replicon_renet2::RenetChannelsExt;
use eryndor_shared::*;
use crate::ui::UiState;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Login,
    CharacterSelect,
    InGame,
}

#[derive(Resource, Default)]
pub struct MyClientState {
    pub account_id: Option<i64>,
    pub characters: Vec<CharacterData>,
    pub selected_character_id: Option<i64>,
    pub player_entity: Option<Entity>,
    pub notifications: Vec<String>,
    pub connection_error_shown: bool,
}

pub fn handle_login_response(
    trigger: On<LoginResponse>,
    mut client_state: ResMut<MyClientState>,
    mut ui_state: ResMut<UiState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let response = trigger.event();
    if response.success {
        info!("Login successful!");
        client_state.account_id = response.account_id;
        ui_state.is_admin = response.is_admin;
        next_state.set(GameState::CharacterSelect);
    } else {
        warn!("Login failed: {}", response.message);
        client_state.notifications.push(response.message.clone());
    }
}

pub fn handle_character_list(
    trigger: On<CharacterListResponse>,
    mut client_state: ResMut<MyClientState>,
) {
    let response = trigger.event();
    info!("Received {} characters", response.characters.len());
    client_state.characters = response.characters.clone();
}

pub fn handle_create_account_response(
    trigger: On<CreateAccountResponse>,
    mut client_state: ResMut<MyClientState>,
) {
    let response = trigger.event();
    if response.success {
        info!("Account created successfully!");
    } else {
        warn!("Account creation failed: {}", response.message);
    }
    client_state.notifications.push(response.message.clone());
}

pub fn handle_create_character_response(
    trigger: On<CreateCharacterResponse>,
    mut client_state: ResMut<MyClientState>,
) {
    let response = trigger.event();
    if response.success {
        info!("Character created!");
        if let Some(character) = &response.character {
            client_state.characters.push(character.clone());
        }
    }
    client_state.notifications.push(response.message.clone());
}

pub fn handle_notifications(
    trigger: On<NotificationEvent>,
    mut client_state: ResMut<MyClientState>,
) {
    let notification = trigger.event();
    info!("[NOTIFICATION] {}", notification.message);
    client_state.notifications.push(notification.message.clone());
}

// ============================================================================
// OAUTH HANDLERS
// ============================================================================

pub fn handle_oauth_login_response(
    trigger: On<OAuthLoginResponse>,
    mut client_state: ResMut<MyClientState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let response = trigger.event();
    if response.success {
        info!("OAuth login successful!");
        client_state.account_id = response.account_id;
        next_state.set(GameState::CharacterSelect);
    } else {
        warn!("OAuth login failed: {}", response.message);
    }
    client_state.notifications.push(response.message.clone());
}

pub fn handle_select_character_response(
    trigger: On<SelectCharacterResponse>,
    mut client_state: ResMut<MyClientState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let response = trigger.event();

    info!("Character selected: ID {}, transitioning to InGame", response.character_id);
    client_state.selected_character_id = Some(response.character_id);
    // Transition to InGame state - the entity will be discovered via detect_player_entity
    next_state.set(GameState::InGame);
}

// Detect when our player entity has been replicated
pub fn detect_player_entity(
    mut client_state: ResMut<MyClientState>,
    player_query: Query<(Entity, &Character), (With<Player>, Added<Player>)>,
) {
    // Only detect if we don't have a player entity yet and we know which character we selected
    if client_state.player_entity.is_none() {
        if let Some(selected_id) = client_state.selected_character_id {
            // Find the character name from our character list
            if let Some(character_data) = client_state.characters.iter().find(|c| c.id == selected_id) {
                // Look for a newly added Player entity with matching name
                for (entity, character) in &player_query {
                    if character.name == character_data.name {
                        client_state.player_entity = Some(entity);
                        info!("Detected player entity {:?} for character: {}", entity, character.name);
                        return;
                    }
                }
            }
        }
    }
}

#[cfg(not(target_family = "wasm"))]
pub fn connect_to_server(mut commands: Commands, channels: Res<RepliconChannels>, _time: Res<Time>) {
    info!("Connecting to server...");

    let connection_config = ConnectionConfig::from_channels(
        channels.server_configs(),
        channels.client_configs(),
    );

    let server_addr: std::net::SocketAddr = format!("{}:{}", SERVER_ADDR, SERVER_PORT)
        .parse()
        .expect("Invalid server address");

    use bevy_renet2::netcode::{ClientAuthentication, NetcodeClientTransport};

    // Create RenetClient first (UDP is not reliable)
    let client = RenetClient::new(connection_config, false);

    // Use SystemTime for actual Unix timestamp (required for netcode authentication)
    use std::time::SystemTime;
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;

    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: 0, // TODO: Use proper protocol ID
        socket_id: 0,
        server_addr,
        user_data: None,
    };

    // Native: Use UDP transport
    use std::net::UdpSocket;
    use bevy_renet2::netcode::NativeSocket;

    info!("Using UDP transport (native)");
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");
    let transport = NetcodeClientTransport::new(current_time, authentication, NativeSocket::new(socket).unwrap())
        .expect("Failed to create UDP transport");

    commands.insert_resource(client);
    commands.insert_resource(transport);

    info!("Connected to server at {} (UDP)", server_addr);
}

#[cfg(target_family = "wasm")]
pub fn connect_to_server(mut commands: Commands, channels: Res<RepliconChannels>, time: Res<Time>) {
    info!("Connecting to server...");

    let connection_config = ConnectionConfig::from_channels(
        channels.server_configs(),
        channels.client_configs(),
    );

    use bevy_renet2::netcode::{ClientAuthentication, NetcodeClientTransport};
    use std::time::Duration;

    // Use js_sys::Date::now() to get Unix timestamp in WASM
    // (SystemTime is not available in WASM)
    let timestamp_ms = js_sys::Date::now() as u64; // Milliseconds since Unix epoch

    let current_time = Duration::from_millis(timestamp_ms);

    // Generate unique client ID from timestamp + random bytes to avoid collisions
    // Mix timestamp (upper 48 bits) with random data (lower 16 bits)
    let mut random_bytes = [0u8; 2];
    getrandom::getrandom(&mut random_bytes).expect("Failed to generate random bytes");
    let random_component = u16::from_le_bytes(random_bytes) as u64;
    let client_id = (timestamp_ms << 16) | random_component;

    info!("Generated client_id: {} (timestamp: {}, random: {})", client_id, timestamp_ms, random_component);

    // Use WebSocket for now (WebTransport requires complex server setup with certificates)
    // TODO: Add WebTransport support once server is properly configured
    use bevy_renet2::netcode::{WebSocketClient, WebSocketClientConfig};

    // Use compile-time environment variable for production, development default otherwise
    // For production builds: SERVER_WS_URL="wss://yourdomain.com/ws" cargo build --target wasm32-unknown-unknown --release
    let ws_url = option_env!("SERVER_WS_URL")
        .unwrap_or("ws://127.0.0.1:5003");

    info!("Connecting via WebSocket to {}", ws_url);

    // Parse the URL
    let url: url::Url = ws_url.parse().expect("Invalid WebSocket URL");

    // Extract host and port from WebSocket URL
    let host = url.host_str().expect("WebSocket URL missing host");
    let port = url.port().unwrap_or(5003); // Default WebSocket port

    // For WebSocket connections, we need a valid SocketAddr
    // The actual connection uses the WebSocket URL, so this is mainly for netcode metadata
    let ws_server_addr: std::net::SocketAddr = if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        // Host is already an IP address
        std::net::SocketAddr::new(ip, port)
    } else if host == "localhost" {
        // localhost -> 127.0.0.1
        std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), port)
    } else {
        // For production hostnames (like ws.eryndor-online.com), we can't do DNS resolution in WASM
        // Use a dummy valid IP since WebSocket uses the URL directly, not this SocketAddr
        info!("Using dummy IP 1.1.1.1 for hostname '{}' (WebSocket uses URL directly)", host);
        std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(1, 1, 1, 1)), port)
    };

    let ws_config = WebSocketClientConfig {
        server_url: ws_url.parse().expect("Invalid WebSocket URL"),
    };

    let ws_client = WebSocketClient::new(ws_config)
        .expect("Failed to create WebSocket client");

    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: 0,
        socket_id: 2,  // Socket 2 = WebSocket
        server_addr: ws_server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, ws_client)
        .expect("Failed to create WebSocket transport");

    // Create RenetClient (WebSocket is reliable)
    let client = RenetClient::new(connection_config, true);

    commands.insert_resource(client);
    commands.insert_resource(transport);

    info!("Connected to server via WebSocket (client_id: {})", client_id);
}

// Explicit system to update WebSocket transport for WASM
#[cfg(target_family = "wasm")]
pub fn update_websocket_transport(
    mut transport: ResMut<bevy_renet2::netcode::NetcodeClientTransport>,
    mut client: ResMut<RenetClient>,
    time: Res<Time>,
) {
    use std::time::Duration;
    let delta = Duration::from_secs_f64(time.delta_secs_f64());
    if let Err(e) = transport.update(delta, &mut client) {
        error!("Transport update error: {}", e);
    }
}

pub fn monitor_connection(
    client: Option<Res<RenetClient>>,
    mut client_state: ResMut<MyClientState>,
) {
    // Check if client resource exists
    if let Some(client) = client {
        if client.is_disconnected() && !client_state.connection_error_shown {
            error!("Lost connection to server!");
            client_state.notifications.push("ERROR: Cannot connect to server. Please make sure the server is running.".to_string());
            client_state.connection_error_shown = true;
        }

        // Log connection state periodically for debugging
        if !client_state.connection_error_shown {
            if client.is_connecting() {
                info!("Client is connecting...");
            } else if client.is_disconnected() {
                warn!("Client is disconnected");
            }
            // Note: Removed "Client is connected!" log spam
        }
    } else {
        // This would indicate the RenetClient resource was never created
        warn!("RenetClient resource not found!");
    }
}

// Handle when player entity is despawned (disconnected from character)
pub fn handle_character_despawn(
    mut client_state: ResMut<MyClientState>,
    mut next_state: ResMut<NextState<GameState>>,
    player_query: Query<Entity, With<Player>>,
    current_state: Res<State<GameState>>,
) {
    // Only check if we're in game and have a player entity reference
    if *current_state.get() != GameState::InGame {
        return;
    }

    if let Some(player_entity) = client_state.player_entity {
        // Check if the player entity still exists
        if player_query.get(player_entity).is_err() {
            info!("Player entity despawned, returning to character select");
            client_state.player_entity = None;
            client_state.selected_character_id = None;
            next_state.set(GameState::CharacterSelect);
        }
    }
}

/// Handle level up event from server
pub fn handle_level_up(
    trigger: On<LevelUpEvent>,
    mut client_state: ResMut<MyClientState>,
) {
    let event = trigger.event();
    info!("Received LevelUpEvent: level {}", event.new_level);
    let message = format!(
        "LEVEL UP! You are now level {}!\n+{} HP | +{} Mana | +{} Attack | +{} Defense",
        event.new_level,
        event.health_increase,
        event.mana_increase,
        event.attack_increase,
        event.defense_increase
    );
    info!("{}", message);
    client_state.notifications.push(message);
}

/// Handle proficiency level up event from server
pub fn handle_proficiency_level_up(
    trigger: On<ProficiencyLevelUpEvent>,
    mut client_state: ResMut<MyClientState>,
) {
    let event = trigger.event();
    let prof_type = match event.proficiency_type {
        ProficiencyType::Weapon => "Weapon",
        ProficiencyType::Armor => "Armor",
    };
    let message = format!(
        "{} Proficiency Level Up! {} is now level {}!\n{}",
        prof_type,
        event.weapon_or_armor,
        event.new_level,
        event.bonus_info
    );
    info!("{}", message);
    client_state.notifications.push(message);
}

// ============================================================================
// ADMIN DASHBOARD RESPONSE HANDLERS
// ============================================================================

/// Handle player list response from server
pub fn handle_player_list_response(
    trigger: On<PlayerListResponse>,
    mut ui_state: ResMut<crate::ui::UiState>,
) {
    let response = trigger.event();
    info!("Received player list with {} players", response.players.len());
    ui_state.system_menu.player_list = response.players.clone();
}

/// Handle ban list response from server
pub fn handle_ban_list_response(
    trigger: On<BanListResponse>,
    mut ui_state: ResMut<crate::ui::UiState>,
) {
    let response = trigger.event();
    info!("Received ban list with {} bans", response.bans.len());
    ui_state.system_menu.ban_list = response.bans.clone();
}

/// Handle server stats response from server
pub fn handle_server_stats_response(
    trigger: On<ServerStatsResponse>,
    mut ui_state: ResMut<crate::ui::UiState>,
) {
    let response = trigger.event();
    info!("Received server stats");
    ui_state.system_menu.server_stats = Some(response.clone());
}

/// Handle audit logs response from server
pub fn handle_audit_logs_response(
    trigger: On<AuditLogsResponse>,
    mut ui_state: ResMut<crate::ui::UiState>,
) {
    let response = trigger.event();
    info!("Received {} audit logs (total: {})", response.logs.len(), response.total_count);
    ui_state.system_menu.audit_logs = response.logs.clone();
    ui_state.system_menu.audit_logs_total = response.total_count;
}

/// Cleanup player entities when leaving InGame state
/// World entities (NPCs, enemies, items) persist across character sessions
pub fn cleanup_game_entities(
    mut commands: Commands,
    player_entities: Query<Entity, (With<Replicated>, With<Player>)>,
) {
    let count = player_entities.iter().count();
    if count > 0 {
        info!("Cleaning up {} player entities", count);
        for entity in &player_entities {
            commands.entity(entity).despawn();
        }
    }
}
