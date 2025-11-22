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

// Resource to hold pending WebTransport connection from async task
#[cfg(target_family = "wasm")]
#[derive(Resource)]
pub struct PendingWebTransportConnection {
    receiver: std::sync::Arc<std::sync::Mutex<Option<(RenetClient, bevy_replicon_renet2::netcode::NetcodeClientTransport)>>>,
}

#[cfg(target_family = "wasm")]
pub fn connect_to_server(mut commands: Commands, channels: Res<RepliconChannels>, _time: Res<Time>) {
    // WASM Client: Uses WebTransport (HTTP/3 + QUIC)
    // - Production: Connects to SERVER_IP (from .do/app.yaml) via port 5002
    // - Local dev: Defaults to 127.0.0.1:5002
    // - Test local against prod: SERVER_IP=165.227.217.144 bevy run web
    //
    // Certificate hash is fetched from http://SERVER_IP:8080/cert
    // Server uses self-signed cert, hash validates the connection

    info!("Connecting to server via WebTransport...");

    // Environment-based configuration - allows testing local client against prod server
    // Local dev: defaults to 127.0.0.1
    // Test against prod: SERVER_IP=165.227.217.144 bevy run web
    // Production build: .do/app.yaml sets SERVER_IP
    let server_ip = option_env!("SERVER_IP").unwrap_or("127.0.0.1");
    let wt_port: u16 = option_env!("SERVER_PORT_WT")
        .and_then(|s| s.parse().ok())
        .unwrap_or(5002);
    let cert_port: u16 = option_env!("SERVER_CERT_PORT")
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);

    info!("Server IP: {}, WebTransport port: {}, Cert port: {}", server_ip, wt_port, cert_port);

    let connection_config = ConnectionConfig::from_channels(
        channels.server_configs(),
        channels.client_configs(),
    );

    // Get current time for netcode
    use std::time::Duration;
    let timestamp_ms = js_sys::Date::now() as u64;
    let current_time = Duration::from_millis(timestamp_ms);

    // Generate unique client ID
    let mut random_bytes = [0u8; 2];
    getrandom::getrandom(&mut random_bytes).expect("Failed to generate random bytes");
    let random_component = u16::from_le_bytes(random_bytes) as u64;
    let client_id = (timestamp_ms << 16) | random_component;

    info!("Generated client_id: {}", client_id);

    // Fetch certificate hash and connect - this must be async
    let cert_url = format!("http://{}:{}/cert", server_ip, cert_port);
    let server_url_str = format!("https://{}:{}", server_ip, wt_port);

    info!("Fetching WebTransport certificate hash from {}", cert_url);

    // Create channel to receive connection from async task
    let connection_receiver = std::sync::Arc::new(std::sync::Mutex::new(None));
    let connection_sender = connection_receiver.clone();

    // Insert resource for polling system
    commands.insert_resource(PendingWebTransportConnection {
        receiver: connection_receiver,
    });

    // Spawn async task to fetch cert and connect
    use wasm_bindgen_futures::spawn_local;

    spawn_local(async move {
        use bevy_renet2::prelude::RenetClient;
        use bevy_renet2::netcode::{NetcodeClientTransport, ClientAuthentication, WebTransportClient, WebTransportClientConfig, ServerCertHash, WebServerDestination, CongestionControl};

        match async {
            // Fetch certificate hash
            let response = reqwest::get(&cert_url).await
                .map_err(|e| format!("Failed to fetch cert: {}", e))?;
            let cert_hash_bytes: Vec<u8> = response.json().await
                .map_err(|e| format!("Failed to parse cert hash: {}", e))?;

            if cert_hash_bytes.len() != 32 {
                return Err(format!("Invalid cert hash length: {} (expected 32)", cert_hash_bytes.len()));
            }

            let mut hash_array = [0u8; 32];
            hash_array.copy_from_slice(&cert_hash_bytes);
            let cert_hash = ServerCertHash { hash: hash_array };

            info!("Received certificate hash, connecting to {}", server_url_str);

            // Build WebTransport URL
            let server_url: url::Url = server_url_str.parse()
                .map_err(|e| format!("Invalid server URL: {}", e))?;

            // Create WebServerDestination
            let server_dest = WebServerDestination::Url(server_url);

            // Create WebTransport configuration
            let wt_config = WebTransportClientConfig {
                server_dest: server_dest.clone(),
                congestion_control: CongestionControl::default(),
                server_cert_hashes: vec![cert_hash],
            };

            // Create WebTransport client (synchronous, no .await)
            let wt_client = WebTransportClient::new(wt_config);

            // Server address for netcode authentication
            // IMPORTANT: Must use the same WebServerDestination to ensure SocketAddr matches
            // what WebTransportClient stores internally (hash of URL, not literal IP:PORT)
            let server_addr: std::net::SocketAddr = server_dest.clone().into();

            let authentication = ClientAuthentication::Unsecure {
                client_id,
                protocol_id: 0,
                socket_id: 1,  // Socket 1 = WebTransport
                server_addr,
                user_data: None,
            };

            let transport = NetcodeClientTransport::new(current_time, authentication, wt_client)
                .map_err(|e| format!("Failed to create transport: {}", e))?;

            // Create RenetClient (use false to match channel config across all transports)
            let client = RenetClient::new(connection_config, false);

            Ok((client, transport))
        }.await {
            Ok((client, transport)) => {
                info!("Successfully connected to WebTransport server (client_id: {})", client_id);
                // Send the connection resources to the polling system
                if let Ok(mut guard) = connection_sender.lock() {
                    *guard = Some((client, transport));
                    info!("WebTransport connection ready for insertion into Bevy ECS");
                } else {
                    error!("Failed to lock connection sender mutex");
                }
            }
            Err(e) => {
                error!("Failed to connect via WebTransport: {}", e);
            }
        }
    });

    info!("WebTransport connection initiated...");
}

// System to poll for completed WebTransport connection and insert resources
#[cfg(target_family = "wasm")]
pub fn poll_webtransport_connection(
    mut commands: Commands,
    pending: Option<Res<PendingWebTransportConnection>>,
    existing_client: Option<Res<RenetClient>>,
) {
    // Only poll if we have a pending connection and no existing client
    if existing_client.is_some() {
        return;
    }

    if let Some(pending) = pending {
        if let Ok(mut guard) = pending.receiver.lock() {
            if let Some((client, transport)) = guard.take() {
                info!("Inserting WebTransport client and transport into Bevy ECS");
                commands.insert_resource(client);
                commands.insert_resource(transport);
                commands.remove_resource::<PendingWebTransportConnection>();
            }
        }
    }
}

// WebTransport doesn't need explicit update calls like WebSocket did
// The transport handles its own async operations

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
