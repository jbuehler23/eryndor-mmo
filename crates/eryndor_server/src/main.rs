// Allow common clippy warnings for game development
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_imports)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::manual_unwrap_or_default)]
#![allow(clippy::single_match)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::items_after_test_module)]
#![allow(clippy::unnecessary_unwrap)]

mod abilities;
mod admin;
mod audit;
mod auth;
mod character;
mod combat;
mod config;
mod dashboard;
mod database;
mod game_data;
mod inventory;
mod moderation;
mod movement;
mod quest;
mod spawn;
mod trainer;
mod weapon;
mod world;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet2::RepliconRenetPlugins;
use avian2d::prelude::*;
use governor::{Quota, RateLimiter, state::keyed::DefaultKeyedStateStore};
use std::num::NonZeroU32;
use std::net::IpAddr;

use eryndor_shared::*;

// Resource to keep tokio runtime alive for WebTransport/WebSocket servers
// The HTTP server task is managed by Bevy's IoTaskPool, but WebTransport/WebSocket
// need their own runtime as they're not integrated with Bevy's task system
#[derive(Resource)]
struct TokioRuntimeResource(tokio::runtime::Runtime);

// Rate limiters for security (per-IP rate limiting)
#[derive(Resource)]
pub struct RateLimiters {
    pub account_creation: RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, governor::clock::DefaultClock>,
    pub login_attempts: RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, governor::clock::DefaultClock>,
}

impl RateLimiters {
    fn from_config(config: &config::ServerConfig) -> Self {
        Self {
            account_creation: RateLimiter::keyed(
                Quota::per_hour(NonZeroU32::new(config.rate_limits.account_creation_per_hour).unwrap())
            ),
            login_attempts: RateLimiter::keyed(
                Quota::per_hour(NonZeroU32::new(config.rate_limits.login_attempts_per_hour).unwrap())
            ),
        }
    }
}

// Disambiguate Position - use our custom one for replication
use eryndor_shared::Position as SharedPosition;
// Import Avian physics components for movement
use avian2d::prelude::Position as PhysicsPosition;
use avian2d::prelude::LinearVelocity as PhysicsVelocity;

fn main() {
    // Initialize early logging BEFORE any other operations
    // This ensures we can see errors during config loading
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    // Load environment variables from .env file (if it exists)
    // This allows for production secrets to be configured via environment
    dotenvy::dotenv().ok();

    // Debug output to help diagnose startup issues
    eprintln!("=== Eryndor Server Startup Debug ===");
    eprintln!("Current directory: {:?}", std::env::current_dir());
    eprintln!("CONFIG_PATH env: {:?}", std::env::var("CONFIG_PATH"));
    eprintln!("DATABASE_PATH env: {:?}", std::env::var("DATABASE_PATH"));
    eprintln!("SERVER_ADDR env: {:?}", std::env::var("SERVER_ADDR"));
    eprintln!("====================================");

    // Load configuration from config.toml
    let config = match config::ServerConfig::load() {
        Ok(cfg) => {
            eprintln!("Configuration loaded successfully");
            cfg
        }
        Err(e) => {
            eprintln!("FATAL ERROR: Failed to load configuration: {}", e);
            eprintln!("Make sure config.toml exists and is valid.");
            eprintln!("Current directory: {:?}", std::env::current_dir());
            std::process::exit(1);
        }
    };

    App::new()
        .add_plugins((
            MinimalPlugins,
            bevy::log::LogPlugin::default(),
            bevy::state::app::StatesPlugin,
            RepliconPlugins,
        ))
        .add_plugins(RepliconRenetPlugins)
        // Physics - server-authoritative with fixed timestep
        // Use headless mode - bevy_scene feature disabled in Cargo.toml
        .add_plugins(PhysicsPlugins::default().with_length_unit(1.0))
        .insert_resource(Gravity(Vec2::ZERO))  // Top-down game, no gravity
        .insert_resource(Time::<Fixed>::from_hz(60.0))  // 60 Hz physics tick rate
        // Configuration
        .insert_resource(RateLimiters::from_config(&config))
        .insert_resource(config)
        // Database
        .init_resource::<database::DatabaseConnection>()
        // Game data resources
        .init_resource::<abilities::AbilityDatabase>()
        .init_resource::<game_data::ItemDatabase>()
        .init_resource::<game_data::QuestDatabase>()
        .init_resource::<game_data::TrainerDatabase>()
        .init_resource::<game_data::EnemyDatabase>()
        // Register replicated components
        .replicate::<Player>()
        .replicate::<Character>()
        .replicate::<OwnedBy>()
        .replicate::<SharedPosition>()
        .replicate::<Experience>()
        .replicate::<WeaponProficiencyExp>()
        .replicate::<ArmorProficiency>()
        .replicate::<ArmorProficiencyExp>()
        .replicate::<UnlockedArmorPassives>()
        .replicate::<Velocity>()
        .replicate::<MoveSpeed>()
        .replicate::<Health>()
        .replicate::<Mana>()
        .replicate::<HealthRegen>()
        .replicate::<ManaRegen>()
        .replicate::<CombatStats>()
        .replicate::<CurrentTarget>()
        .replicate::<InCombat>()
        .replicate::<AutoAttack>()
        .replicate::<WeaponProficiency>()
        .replicate::<Gold>()
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
        .replicate::<GoldDrop>()
        .replicate::<LootContainer>()
        .replicate::<Interactable>()
        .replicate::<VisualShape>()
        .replicate::<ActiveBuffs>()
        .replicate::<ActiveDebuffs>()
        .replicate::<ActiveDoTs>()
        // Register client -> server events (Events API)
        .add_client_event::<LoginRequest>(Channel::Ordered)
        .add_client_event::<CreateAccountRequest>(Channel::Ordered)
        .add_client_event::<OAuthLoginRequest>(Channel::Ordered)
        .add_client_event::<CreateCharacterRequest>(Channel::Ordered)
        .add_client_event::<SelectCharacterRequest>(Channel::Ordered)
        .add_client_event::<MoveInput>(Channel::Unreliable)
        .add_mapped_client_event::<SetTargetRequest>(Channel::Ordered)
        .add_client_event::<UseAbilityRequest>(Channel::Ordered)
        .add_mapped_client_event::<PickupItemRequest>(Channel::Ordered)
        .add_mapped_client_event::<OpenLootContainerRequest>(Channel::Ordered)
        .add_mapped_client_event::<LootItemRequest>(Channel::Ordered)
        .add_client_event::<AutoLootRequest>(Channel::Ordered)
        .add_client_event::<DropItemRequest>(Channel::Ordered)
        .add_client_event::<EquipItemRequest>(Channel::Ordered)
        .add_client_event::<UnequipItemRequest>(Channel::Ordered)
        .add_mapped_client_event::<InteractNpcRequest>(Channel::Ordered)
        .add_client_event::<AcceptQuestRequest>(Channel::Ordered)
        .add_client_event::<CompleteQuestRequest>(Channel::Ordered)
        .add_mapped_client_event::<PurchaseFromTrainerRequest>(Channel::Ordered)
        .add_client_event::<SetHotbarSlotRequest>(Channel::Ordered)
        .add_client_event::<DisconnectCharacterRequest>(Channel::Ordered)
        .add_client_event::<AdminCommandRequest>(Channel::Ordered)
        .add_client_event::<SendChatMessage>(Channel::Ordered)
        // Dashboard query events
        .add_client_event::<GetPlayerListRequest>(Channel::Ordered)
        .add_client_event::<GetBanListRequest>(Channel::Ordered)
        .add_client_event::<GetServerStatsRequest>(Channel::Ordered)
        .add_client_event::<GetAuditLogsRequest>(Channel::Ordered)
        // Register server -> client events (Events API)
        .add_server_event::<LoginResponse>(Channel::Ordered)
        .add_server_event::<CreateAccountResponse>(Channel::Ordered)
        .add_server_event::<OAuthLoginResponse>(Channel::Ordered)
        .add_server_event::<CharacterListResponse>(Channel::Ordered)
        .add_server_event::<CreateCharacterResponse>(Channel::Ordered)
        .add_server_event::<SelectCharacterResponse>(Channel::Ordered)
        .add_mapped_server_event::<CombatEvent>(Channel::Ordered)
        .add_server_event::<QuestUpdateEvent>(Channel::Ordered)
        .add_mapped_server_event::<DeathEvent>(Channel::Ordered)
        .add_server_event::<NotificationEvent>(Channel::Ordered)
        .add_server_event::<QuestDialogueEvent>(Channel::Ordered)
        .add_server_event::<TrainerDialogueEvent>(Channel::Ordered)
        .add_mapped_server_event::<LootContainerContentsEvent>(Channel::Ordered)
        .add_server_event::<LevelUpEvent>(Channel::Ordered)
        .add_server_event::<ProficiencyLevelUpEvent>(Channel::Ordered)
        .add_server_event::<ChatMessage>(Channel::Ordered)
        // Dashboard response events
        .add_server_event::<PlayerListResponse>(Channel::Ordered)
        .add_server_event::<BanListResponse>(Channel::Ordered)
        .add_server_event::<ServerStatsResponse>(Channel::Ordered)
        .add_server_event::<AuditLogsResponse>(Channel::Ordered)
        // Register observers for client triggers
        .add_observer(auth::handle_login)
        .add_observer(auth::handle_create_account)
        .add_observer(auth::handle_oauth_login)
        .add_observer(auth::handle_create_character)
        .add_observer(auth::handle_select_character)
        .add_observer(movement::handle_move_input)
        .add_observer(combat::handle_set_target)
        .add_observer(combat::handle_use_ability)
        .add_observer(inventory::handle_pickup_item)
        .add_observer(inventory::handle_drop_item)
        .add_observer(inventory::handle_equip_item)
        .add_observer(inventory::handle_unequip_item)
        .add_observer(inventory::handle_set_hotbar_slot)
        .add_observer(inventory::handle_open_loot_container)
        .add_observer(inventory::handle_loot_item)
        .add_observer(inventory::handle_auto_loot)
        .add_observer(quest::handle_interact_npc)
        .add_observer(quest::handle_accept_quest)
        .add_observer(quest::handle_complete_quest)
        .add_observer(trainer::handle_purchase_from_trainer)
        .add_observer(auth::handle_disconnect_character)
        .add_observer(admin::handle_admin_command)
        .add_observer(admin::handle_chat_message)
        // Dashboard observers
        .add_observer(dashboard::handle_get_player_list)
        .add_observer(dashboard::handle_get_ban_list)
        .add_observer(dashboard::handle_get_server_stats)
        .add_observer(dashboard::handle_get_audit_logs)
        // Respawn system
        .add_observer(spawn::schedule_respawn)
        // Systems
        .add_systems(Startup, (
            setup_server,
            database::setup_database,
            world::spawn_world,
        ))
        .add_systems(Update, (
            // Connection tracking (must run first to capture IPs)
            auth::track_client_connections,
            // Auth systems
            auth::handle_client_disconnect,
            // Movement
            movement::update_positions,
            // Combat
            combat::update_ai_activation_delays,
            combat::process_auto_attacks,
            combat::update_ability_cooldowns,
            combat::regenerate_resources,
            combat::check_deaths,
            combat::check_level_ups,
            combat::check_weapon_proficiency_level_ups,
            combat::enemy_ai,
            // Ability effects
            abilities::process_buffs,
            abilities::process_debuffs,
            abilities::process_dots,
            // Quests
            quest::update_quest_progress,
            // Respawn
            spawn::process_respawns,
        ))
        // Physics sync - runs after physics update to sync PhysicsPosition -> Position
        .add_systems(PostUpdate, sync_physics_to_position)
        .run();
}

/// Syncs Avian's PhysicsPosition to our replicated Position component
/// This runs after physics updates so clients get the physics-driven positions
/// Uses change detection to only sync when physics actually moved the entity
fn sync_physics_to_position(
    mut query: Query<(&PhysicsPosition, &mut SharedPosition), Changed<PhysicsPosition>>,
) {
    for (physics_pos, mut position) in &mut query {
        position.0 = physics_pos.0;
    }
}

fn setup_server(mut commands: Commands, channels: Res<RepliconChannels>) {
    info!("Starting Eryndor MMO Server with multi-transport support...");

    use bevy_renet2::prelude::{RenetServer, ConnectionConfig};
    use bevy_renet2::netcode::{
        NetcodeServerTransport, ServerAuthentication, ServerSetupConfig,
        NativeSocket, BoxedSocket, WebTransportServerConfig, WebSocketServerConfig,
        WebTransportServer, WebSocketServer
    };
    use bevy_replicon_renet2::RenetChannelsExt;
    use std::net::UdpSocket;
    use std::time::SystemTime;

    let connection_config = ConnectionConfig::from_channels(
        channels.server_configs(),
        channels.client_configs(),
    );

    let udp_addr: std::net::SocketAddr = format!("{}:{}", server_addr(), server_port())
        .parse()
        .expect("Invalid UDP address");
    let wt_addr: std::net::SocketAddr = format!("{}:{}", server_addr(), server_port_webtransport())
        .parse()
        .expect("Invalid WebTransport address");
    let ws_addr: std::net::SocketAddr = format!("{}:{}", server_addr(), server_port_websocket())
        .parse()
        .expect("Invalid WebSocket address");

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let max_clients = 64;

    // Create three sockets: Native UDP, WebTransport, and WebSocket
    // Socket 0: UDP for native clients
    let udp_socket = UdpSocket::bind(udp_addr).expect("Failed to bind UDP socket");
    let native_socket = BoxedSocket::new(NativeSocket::new(udp_socket).unwrap());
    info!("UDP socket listening on {}", udp_addr);

    // Socket 1: WebTransport for WASM clients (with self-signed cert)
    // WebTransport/WebSocket need a tokio runtime - create one for them to use
    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");
    let tokio_handle = tokio_runtime.handle().clone();

    let (wt_config, cert_hash) = WebTransportServerConfig::new_selfsigned(wt_addr, max_clients)
        .expect("Failed to create WebTransport config");
    let wt_server = WebTransportServer::new(wt_config, tokio_handle.clone())
        .expect("Failed to create WebTransport server");
    let wt_socket = BoxedSocket::new(wt_server);
    info!("WebTransport socket listening on {}", wt_addr);
    info!("WebTransport certificate hash: {:?}", cert_hash);

    // Socket 2: WebSocket fallback for WASM clients
    let ws_config = WebSocketServerConfig::new(ws_addr, max_clients);
    let ws_server = WebSocketServer::new(ws_config, tokio_handle.clone())
        .expect("Failed to create WebSocket server");
    let ws_socket = BoxedSocket::new(ws_server);
    info!("WebSocket socket listening on {}", ws_addr);

    // Spawn HTTP server to serve certificate hash for WASM clients
    // Must run on tokio runtime (not Bevy's IoTaskPool which uses async-executor)
    let cert_hash_clone = cert_hash.clone();
    tokio_handle.spawn(async move {
        serve_cert_hash(cert_hash_clone).await;
    });

    // Register all three socket addresses
    let server_config = ServerSetupConfig {
        current_time,
        max_clients,
        protocol_id: 0,
        socket_addresses: vec![
            vec![udp_addr], // Socket 0: UDP
            vec![wt_addr],  // Socket 1: WebTransport
            vec![ws_addr],  // Socket 2: WebSocket
        ],
        authentication: ServerAuthentication::Unsecure,
    };

    // Create transport with all three sockets
    let transport = NetcodeServerTransport::new_with_sockets(
        server_config,
        vec![native_socket, wt_socket, ws_socket],
    )
    .expect("Failed to create multi-transport");

    let server = RenetServer::new(connection_config);

    commands.insert_resource(server);
    commands.insert_resource(transport);
    // Keep tokio runtime alive for WebTransport/WebSocket servers
    commands.insert_resource(TokioRuntimeResource(tokio_runtime));

    info!("Server ready - UDP: {}, WebTransport: {}, WebSocket: {}", udp_addr, wt_addr, ws_addr);
}

// HTTP server to serve WebTransport certificate hash for WASM clients
async fn serve_cert_hash(cert_hash: bevy_renet2::netcode::ServerCertHash) {
    use axum::{routing::get, Router, Json};
    use tower_http::cors::{CorsLayer, Any};

    let app = Router::new()
        .route("/cert", get(move || async move {
            // Return the hash bytes as JSON array (matching renet2 example pattern)
            Json(cert_hash.hash.to_vec())
        }))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        );

    let cert_server_addr = format!("{}:{}",
        eryndor_shared::constants::server_addr(),
        eryndor_shared::constants::server_cert_port()
    );
    let listener = tokio::net::TcpListener::bind(&cert_server_addr)
        .await
        .expect("Failed to bind HTTP server");

    info!("HTTP server for certificate hash listening on http://{}/cert", cert_server_addr);

    axum::serve(listener, app)
        .await
        .expect("Failed to start HTTP server");
}
