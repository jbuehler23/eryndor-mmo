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
    pub player_entity: Option<Entity>,
    pub notifications: Vec<String>,
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
) {
    let response = trigger.event();
    info!("Character entity assigned: {:?}", response.character_entity);
    client_state.player_entity = Some(response.character_entity);
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
