mod auth;
mod combat;
mod database;
mod game_data;
mod inventory;
mod movement;
mod quest;
mod spawn;
mod world;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetPlugins;

use eryndor_shared::*;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            bevy::log::LogPlugin::default(),
            bevy::state::app::StatesPlugin,
            RepliconPlugins,
        ))
        .add_plugins(RepliconRenetPlugins)
        // Database
        .init_resource::<database::DatabaseConnection>()
        // Game data resources
        .init_resource::<game_data::AbilityDatabase>()
        .init_resource::<game_data::ItemDatabase>()
        .init_resource::<game_data::QuestDatabase>()
        // Register replicated components
        .replicate::<Player>()
        .replicate::<Character>()
        .replicate::<OwnedBy>()
        .replicate::<Position>()
        .replicate::<Velocity>()
        .replicate::<MoveSpeed>()
        .replicate::<Health>()
        .replicate::<Mana>()
        .replicate::<CombatStats>()
        .replicate::<CurrentTarget>()
        .replicate::<InCombat>()
        .replicate::<Inventory>()
        .replicate::<Equipment>()
        .replicate::<Hotbar>()
        .replicate::<LearnedAbilities>()
        .replicate::<QuestLog>()
        .replicate::<Npc>()
        .replicate::<QuestGiver>()
        .replicate::<NpcName>()
        .replicate::<Enemy>()
        .replicate::<EnemyType>()
        .replicate::<AiState>()
        .replicate::<WorldItem>()
        .replicate::<VisualShape>()
        // Register client -> server events (Events API)
        .add_client_event::<LoginRequest>(Channel::Ordered)
        .add_client_event::<CreateAccountRequest>(Channel::Ordered)
        .add_client_event::<CreateCharacterRequest>(Channel::Ordered)
        .add_client_event::<SelectCharacterRequest>(Channel::Ordered)
        .add_client_event::<MoveInput>(Channel::Unreliable)
        .add_client_event::<SetTargetRequest>(Channel::Ordered)
        .add_client_event::<UseAbilityRequest>(Channel::Ordered)
        .add_client_event::<PickupItemRequest>(Channel::Ordered)
        .add_client_event::<DropItemRequest>(Channel::Ordered)
        .add_client_event::<EquipItemRequest>(Channel::Ordered)
        .add_client_event::<InteractNpcRequest>(Channel::Ordered)
        .add_client_event::<AcceptQuestRequest>(Channel::Ordered)
        .add_client_event::<CompleteQuestRequest>(Channel::Ordered)
        .add_client_event::<SetHotbarSlotRequest>(Channel::Ordered)
        // Register server -> client events (Events API)
        .add_server_event::<LoginResponse>(Channel::Ordered)
        .add_server_event::<CreateAccountResponse>(Channel::Ordered)
        .add_server_event::<CharacterListResponse>(Channel::Ordered)
        .add_server_event::<CreateCharacterResponse>(Channel::Ordered)
        .add_server_event::<SelectCharacterResponse>(Channel::Ordered)
        .add_server_event::<CombatEvent>(Channel::Ordered)
        .add_server_event::<QuestUpdateEvent>(Channel::Ordered)
        .add_server_event::<DeathEvent>(Channel::Ordered)
        .add_server_event::<NotificationEvent>(Channel::Ordered)
        // Register observers for client triggers
        .add_observer(auth::handle_login)
        .add_observer(auth::handle_create_account)
        .add_observer(auth::handle_create_character)
        .add_observer(auth::handle_select_character)
        .add_observer(movement::handle_move_input)
        .add_observer(combat::handle_set_target)
        .add_observer(combat::handle_use_ability)
        .add_observer(inventory::handle_pickup_item)
        .add_observer(inventory::handle_drop_item)
        .add_observer(inventory::handle_equip_item)
        .add_observer(inventory::handle_set_hotbar_slot)
        .add_observer(quest::handle_interact_npc)
        .add_observer(quest::handle_accept_quest)
        .add_observer(quest::handle_complete_quest)
        // Systems
        .add_systems(Startup, (
            setup_server,
            database::setup_database,
            world::spawn_world,
        ))
        .add_systems(Update, (
            // Auth systems
            auth::handle_client_disconnect,
            // Movement
            movement::update_positions,
            // Combat
            combat::update_ability_cooldowns,
            combat::check_deaths,
            combat::enemy_ai,
            // Quests
            quest::update_quest_progress,
        ))
        .run();
}

fn setup_server(mut commands: Commands, channels: Res<RepliconChannels>) {
    info!("Starting Eryndor MMO Server...");

    use bevy_renet::renet::{ConnectionConfig, RenetServer};
    use bevy_renet::netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
    use bevy_replicon_renet::RenetChannelsExt;
    use std::net::UdpSocket;
    use std::time::SystemTime;

    let server_channels_config = channels.server_configs();
    let client_channels_config = channels.client_configs();

    let server = RenetServer::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let server_addr: std::net::SocketAddr = format!("{}:{}", SERVER_ADDR, SERVER_PORT)
        .parse()
        .expect("Invalid server address");

    let socket = UdpSocket::bind(server_addr).expect("Failed to bind server socket");
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: 0, // TODO: Use proper protocol ID
        public_addresses: vec![server_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket)
        .expect("Failed to create server transport");

    commands.insert_resource(server);
    commands.insert_resource(transport);

    info!("Server listening on {}", server_addr);
}
