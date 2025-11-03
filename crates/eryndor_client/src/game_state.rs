use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_renet::renet::{ConnectionConfig, RenetClient};
use bevy_replicon_renet::RenetChannelsExt;
use eryndor_shared::*;

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
    mut next_state: ResMut<NextState<GameState>>,
) {
    let response = trigger.event();
    if response.success {
        info!("Login successful!");
        client_state.account_id = response.account_id;
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

pub fn connect_to_server(mut commands: Commands, channels: Res<RepliconChannels>) {
    info!("Connecting to server...");

    let server_channels_config = channels.server_configs();
    let client_channels_config = channels.client_configs();

    let client = RenetClient::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let server_addr: std::net::SocketAddr = format!("{}:{}", SERVER_ADDR, SERVER_PORT)
        .parse()
        .expect("Invalid server address");

    // Create transport layer
    use std::net::UdpSocket;
    use std::time::SystemTime;
    use bevy_renet::netcode::{ClientAuthentication, NetcodeClientTransport};

    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;

    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: 0, // TODO: Use proper protocol ID
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket)
        .expect("Failed to create transport");

    commands.insert_resource(client);
    commands.insert_resource(transport);

    info!("Connected to server at {}", server_addr);
}

pub fn monitor_connection(
    client: Option<Res<RenetClient>>,
    mut client_state: ResMut<MyClientState>,
) {
    // Check if client exists and is disconnected
    if let Some(client) = client {
        if client.is_disconnected() && !client_state.connection_error_shown {
            error!("Lost connection to server!");
            client_state.notifications.push("ERROR: Cannot connect to server. Please make sure the server is running.".to_string());
            client_state.connection_error_shown = true;
        }
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
