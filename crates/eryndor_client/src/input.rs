use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use crate::game_state::MyClientState;
use crate::ui::UiState;

#[derive(Resource, Default)]
pub struct InputState {
    pub selected_target: Option<Entity>,
}

pub fn handle_movement_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    client_state: Res<MyClientState>,
    ui_state: Res<UiState>,
    mut commands: Commands,
) {
    // Don't handle movement if ESC menu is open
    if ui_state.show_esc_menu {
        return;
    }

    let Some(_player_entity) = client_state.player_entity else { return };

    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    // Always send input, even if direction is zero (to stop movement)
    commands.client_trigger(MoveInput { direction });
}

pub fn handle_targeting_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    targetable_query: Query<(Entity, &Position, &VisualShape), With<Interactable>>,
    mut input_state: ResMut<InputState>,
    mut commands: Commands,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };

    let Ok((camera, camera_transform)) = camera_query.single() else { return };

    // Convert screen to world coordinates
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    // Find closest targetable entity to click based on visual size
    // Target selection works from any distance - you can click on what you see
    let mut closest_entity = None;
    let mut closest_distance = f32::MAX;

    for (entity, position, visual) in &targetable_query {
        let distance = position.0.distance(world_pos);

        // Use visual size as the "clickable" radius for targeting
        // This means if you can see it, you can target it
        let click_radius = visual.size;

        if distance < click_radius && distance < closest_distance {
            closest_distance = distance;
            closest_entity = Some(entity);
        }
    }

    if let Some(entity) = closest_entity {
        input_state.selected_target = Some(entity);
        commands.client_trigger(SetTargetRequest {
            target: Some(entity),
        });
        info!("Selected target: {:?}", entity);
    }
}

pub fn handle_ability_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    client_state: Res<MyClientState>,
    player_query: Query<&Hotbar>,
    mut commands: Commands,
) {
    let Some(player_entity) = client_state.player_entity else { return };
    let Ok(hotbar) = player_query.get(player_entity) else { return };

    // Check number keys 1-9 and 0
    let keys = [
        KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5,
        KeyCode::Digit6, KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9, KeyCode::Digit0,
    ];

    for (i, key) in keys.iter().enumerate() {
        if keyboard.just_pressed(*key) {
            if let Some(slot) = &hotbar.slots[i] {
                match slot {
                    HotbarSlot::Ability(ability_id) => {
                        commands.client_trigger(UseAbilityRequest {
                            ability_id: *ability_id,
                        });
                        info!("Used ability from slot {}", i + 1);
                    }
                }
            }
        }
    }
}

pub fn handle_auto_attack_toggle(
    mouse_button: Res<ButtonInput<MouseButton>>,
    client_state: Res<MyClientState>,
    ui_state: Res<UiState>,
    player_query: Query<&AutoAttack>,
    mut commands: Commands,
) {
    // Don't handle if ESC menu is open
    if ui_state.show_esc_menu {
        return;
    }

    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    let Some(player_entity) = client_state.player_entity else { return };
    let Ok(auto_attack) = player_query.get(player_entity) else { return };

    // Toggle the state
    let new_state = !auto_attack.enabled;
    commands.client_trigger(ToggleAutoAttackRequest {
        enabled: new_state,
    });

    info!("Toggling auto-attack: {}", if new_state { "ON" } else { "OFF" });
}

pub fn handle_interaction_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    input_state: Res<InputState>,
    ui_state: Res<UiState>,
    client_state: Res<MyClientState>,
    player_query: Query<&Position, With<Player>>,
    npc_query: Query<(Entity, &Position), With<Npc>>,
    world_item_query: Query<(Entity, &Position), With<WorldItem>>,
    mut commands: Commands,
) {
    // Don't handle interaction if ESC menu is open
    if ui_state.show_esc_menu {
        return;
    }

    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Some(target) = input_state.selected_target else {
        info!("E pressed but no target selected");
        return;
    };

    // Get player position for distance check
    let Some(player_entity) = client_state.player_entity else {
        return;
    };
    let Ok(player_pos) = player_query.get(player_entity) else {
        return;
    };

    // Check if target is NPC
    if let Ok((_, npc_pos)) = npc_query.get(target) {
        let distance = player_pos.0.distance(npc_pos.0);
        if distance <= INTERACTION_RANGE {
            commands.client_trigger(InteractNpcRequest { npc_entity: target });
            info!("Interacting with NPC: {:?} at distance {:.2}", target, distance);
        } else {
            info!("NPC too far away: {:.2} pixels (max: {})", distance, INTERACTION_RANGE);
        }
    }

    // Check if target is world item
    if let Ok((_, item_pos)) = world_item_query.get(target) {
        let distance = player_pos.0.distance(item_pos.0);
        if distance <= PICKUP_RANGE {
            commands.client_trigger(PickupItemRequest { item_entity: target });
            info!("Picking up item: {:?} at distance {:.2}", target, distance);
        } else {
            info!("Item too far away: {:.2} pixels (max: {})", distance, PICKUP_RANGE);
        }
    }
}
