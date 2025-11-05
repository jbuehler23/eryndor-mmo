mod rendering;
mod ui;
mod input;
mod game_state;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetPlugins;
use bevy_prototype_lyon::prelude::*;

use eryndor_shared::*;
use game_state::*;

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
            ShapePlugin,
            EguiPlugin::default(),
        ))
        .add_plugins(RepliconRenetPlugins)
        // Game state
        .init_state::<GameState>()
        .init_resource::<MyClientState>()
        .init_resource::<input::InputState>()
        .init_resource::<ui::UiState>()
        // Register replicated components (same as server)
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
        .replicate::<AutoAttack>()
        .replicate::<WeaponProficiency>()
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
        .replicate::<Interactable>()
        .replicate::<VisualShape>()
        // Register client -> server events
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
        .add_client_event::<DisconnectCharacterRequest>(Channel::Ordered)
        .add_client_event::<ToggleAutoAttackRequest>(Channel::Ordered)
        // Register server -> client events
        .add_server_event::<LoginResponse>(Channel::Ordered)
        .add_server_event::<CreateAccountResponse>(Channel::Ordered)
        .add_server_event::<CharacterListResponse>(Channel::Ordered)
        .add_server_event::<CreateCharacterResponse>(Channel::Ordered)
        .add_server_event::<SelectCharacterResponse>(Channel::Ordered)
        .add_server_event::<CombatEvent>(Channel::Ordered)
        .add_server_event::<QuestUpdateEvent>(Channel::Ordered)
        .add_server_event::<DeathEvent>(Channel::Ordered)
        .add_server_event::<NotificationEvent>(Channel::Ordered)
        .add_server_event::<QuestDialogueEvent>(Channel::Ordered)
        // Register observers for server -> client events
        .add_observer(game_state::handle_login_response)
        .add_observer(game_state::handle_character_list)
        .add_observer(game_state::handle_create_account_response)
        .add_observer(game_state::handle_create_character_response)
        .add_observer(game_state::handle_select_character_response)
        .add_observer(game_state::handle_notifications)
        .add_observer(ui::handle_quest_dialogue)
        // Systems
        .add_systems(Startup, (setup_camera, game_state::connect_to_server))
        // UI systems must be in EguiPrimaryContextPass for bevy_egui 0.38
        .add_systems(bevy_egui::EguiPrimaryContextPass, (
            ui::login_ui.run_if(in_state(GameState::Login)),
            ui::character_select_ui.run_if(in_state(GameState::CharacterSelect)),
            ui::game_ui.run_if(in_state(GameState::InGame)),
        ))
        .add_systems(Update, (
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
            // UI Input
            ui::handle_esc_key.run_if(in_state(GameState::InGame)),
            // Input
            input::handle_movement_input.run_if(in_state(GameState::InGame)),
            input::handle_targeting_input.run_if(in_state(GameState::InGame)),
            input::handle_ability_input.run_if(in_state(GameState::InGame)),
            input::handle_interaction_input.run_if(in_state(GameState::InGame)),
            input::handle_auto_attack_toggle.run_if(in_state(GameState::InGame)),
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
