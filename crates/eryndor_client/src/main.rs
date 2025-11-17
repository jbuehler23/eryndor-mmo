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

mod rendering;
mod ui;
mod input;
mod game_state;
mod item_cache;
mod ability_cache;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_replicon::prelude::*;
use bevy_replicon_renet2::RepliconRenetPlugins;
use bevy_prototype_lyon::prelude::*;

use eryndor_shared::*;
use game_state::*;

#[cfg(not(target_family = "wasm"))]
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Eryndor MMO".to_string(),
                    resolution: [1280, 720].into(),
                    ..default()
                }),
                ..default()
            }),
            RepliconPlugins,
            RepliconRenetPlugins,
            ShapePlugin,
            EguiPlugin::default(),
        ))
        // Game state
        .init_state::<GameState>()
        .init_resource::<MyClientState>()
        .init_resource::<input::InputState>()
        .init_resource::<ui::UiState>()
        .init_resource::<item_cache::ClientItemDatabase>()
        .init_resource::<ability_cache::ClientAbilityDatabase>()
        // Register replicated components (same as server)
        .replicate::<Player>()
        .replicate::<Character>()
        .replicate::<OwnedBy>()
        .replicate::<Position>()
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
        // Register client -> server events
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
        // Register server -> client events
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
        // Register observers for server -> client events
        .add_observer(game_state::handle_login_response)
        .add_observer(game_state::handle_character_list)
        .add_observer(game_state::handle_create_account_response)
        .add_observer(game_state::handle_oauth_login_response)
        .add_observer(game_state::handle_create_character_response)
        .add_observer(game_state::handle_select_character_response)
        .add_observer(game_state::handle_notifications)
        .add_observer(game_state::handle_level_up)
        .add_observer(game_state::handle_proficiency_level_up)
        // Admin dashboard response handlers
        .add_observer(game_state::handle_player_list_response)
        .add_observer(game_state::handle_ban_list_response)
        .add_observer(game_state::handle_server_stats_response)
        .add_observer(game_state::handle_audit_logs_response)
        .add_observer(ui::handle_quest_dialogue)
        .add_observer(ui::handle_trainer_dialogue)
        .add_observer(ui::handle_loot_container_contents)
        .add_observer(rendering::spawn_damage_numbers)
        // Systems
        .add_systems(Startup, (setup_camera, game_state::connect_to_server))
        // UI systems must be in EguiPrimaryContextPass for bevy_egui 0.38
        .add_systems(bevy_egui::EguiPrimaryContextPass, (
            ui::login_ui.run_if(in_state(GameState::Login)),
            ui::character_select_ui.run_if(in_state(GameState::CharacterSelect)),
            ui::game_ui.run_if(in_state(GameState::InGame)),
            ui::chat_window.run_if(in_state(GameState::InGame)),
        ))
        .add_systems(OnExit(GameState::InGame), game_state::cleanup_game_entities)
        .add_systems(Update, (
            // OAuth callback check (runs during Login state)
            ui::check_oauth_callback.run_if(in_state(GameState::Login)),
            // Chat message receiver
            ui::receive_chat_messages.run_if(in_state(GameState::InGame)),
            // Connection monitoring
            game_state::monitor_connection,
            // Player entity detection
            game_state::detect_player_entity.run_if(in_state(GameState::InGame)),
            game_state::handle_character_despawn.run_if(in_state(GameState::InGame)),
            // Rendering
            rendering::spawn_visual_entities,
            rendering::update_visual_positions,
            rendering::spawn_name_labels,
            rendering::update_name_label_positions,
            rendering::cleanup_despawned_entities,
            rendering::update_camera_follow.run_if(in_state(GameState::InGame)),
            // Debug rendering (commented out - uncomment when needed)
            // rendering::spawn_debug_grid.run_if(in_state(GameState::InGame)),
            // rendering::draw_debug_labels.run_if(in_state(GameState::InGame)),
            // Target indicator
            rendering::draw_target_indicator.run_if(in_state(GameState::InGame)),
            // Damage numbers
            rendering::update_damage_numbers.run_if(in_state(GameState::InGame)),
            // UI Input
            ui::handle_esc_key.run_if(in_state(GameState::InGame)),
            // Input
            input::handle_movement_input.run_if(in_state(GameState::InGame)),
            input::handle_targeting_input.run_if(in_state(GameState::InGame)),
            input::handle_ability_input.run_if(in_state(GameState::InGame)),
            input::handle_interaction_input.run_if(in_state(GameState::InGame)),
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[cfg(target_family = "wasm")]
fn main() {
    // Set up panic hook for better error messages in browser console
    console_error_panic_hook::set_once();

    // Start the app directly (no certificate needed for WebSocket)
    web_sys::console::log_1(&"Starting Eryndor MMO client...".into());
    start_app();
}

#[cfg(target_family = "wasm")]
fn start_app() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Eryndor MMO".to_string(),
                    resolution: [1280, 720].into(),
                    ..default()
                }),
                ..default()
            }),
            RepliconPlugins,
            RepliconRenetPlugins,
            ShapePlugin,
            EguiPlugin::default(),
        ))
        // Game state
        .init_state::<GameState>()
        .init_resource::<MyClientState>()
        .init_resource::<input::InputState>()
        .init_resource::<ui::UiState>()
        .init_resource::<item_cache::ClientItemDatabase>()
        .insert_resource(ability_cache::ClientAbilityDatabase::default())
        // Register replicated components (same as server)
        .replicate::<Player>()
        .replicate::<Character>()
        .replicate::<OwnedBy>()
        .replicate::<Position>()
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
        // Register client -> server events
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
        // Register server -> client events
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
        // Register observers for server -> client events
        .add_observer(game_state::handle_login_response)
        .add_observer(game_state::handle_character_list)
        .add_observer(game_state::handle_create_account_response)
        .add_observer(game_state::handle_oauth_login_response)
        .add_observer(game_state::handle_create_character_response)
        .add_observer(game_state::handle_select_character_response)
        .add_observer(game_state::handle_notifications)
        .add_observer(game_state::handle_level_up)
        .add_observer(game_state::handle_proficiency_level_up)
        // Admin dashboard response handlers
        .add_observer(game_state::handle_player_list_response)
        .add_observer(game_state::handle_ban_list_response)
        .add_observer(game_state::handle_server_stats_response)
        .add_observer(game_state::handle_audit_logs_response)
        .add_observer(ui::handle_quest_dialogue)
        .add_observer(ui::handle_trainer_dialogue)
        .add_observer(ui::handle_loot_container_contents)
        .add_observer(rendering::spawn_damage_numbers)
        // Systems
        .add_systems(Startup, (setup_camera, game_state::connect_to_server))
        // UI systems must be in EguiPrimaryContextPass for bevy_egui 0.38
        .add_systems(bevy_egui::EguiPrimaryContextPass, (
            ui::login_ui.run_if(in_state(GameState::Login)),
            ui::character_select_ui.run_if(in_state(GameState::CharacterSelect)),
            ui::game_ui.run_if(in_state(GameState::InGame)),
            ui::chat_window.run_if(in_state(GameState::InGame)),
        ))
        .add_systems(OnExit(GameState::InGame), game_state::cleanup_game_entities)
        .add_systems(Update, (
            // OAuth callback check (runs during Login state)
            ui::check_oauth_callback.run_if(in_state(GameState::Login)),
            // Chat message receiver
            ui::receive_chat_messages.run_if(in_state(GameState::InGame)),
            // Connection monitoring
            game_state::monitor_connection,
            // Player entity detection
            game_state::detect_player_entity.run_if(in_state(GameState::InGame)),
            game_state::handle_character_despawn.run_if(in_state(GameState::InGame)),
            // Rendering
            rendering::spawn_visual_entities,
            rendering::update_visual_positions,
            rendering::spawn_name_labels,
            rendering::update_name_label_positions,
            rendering::cleanup_despawned_entities,
            rendering::update_camera_follow.run_if(in_state(GameState::InGame)),
            // Target indicator
            rendering::draw_target_indicator.run_if(in_state(GameState::InGame)),
            // Damage numbers
            rendering::update_damage_numbers.run_if(in_state(GameState::InGame)),
            // UI Input
            ui::handle_esc_key.run_if(in_state(GameState::InGame)),
            // Input
            input::handle_movement_input.run_if(in_state(GameState::InGame)),
            input::handle_targeting_input.run_if(in_state(GameState::InGame)),
            input::handle_ability_input.run_if(in_state(GameState::InGame)),
            input::handle_interaction_input.run_if(in_state(GameState::InGame)),
        ))
        .run();
}
