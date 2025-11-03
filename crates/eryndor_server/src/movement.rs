use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use crate::auth::ActiveCharacterEntity;

pub fn handle_move_input(
    trigger: On<FromClient<MoveInput>>,
    clients: Query<&ActiveCharacterEntity>,
    mut players: Query<(&mut Velocity, &MoveSpeed, &OwnedBy)>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };
    let input = trigger.event();

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else { return };
    let char_entity = active_char.0;

    // Update velocity
    if let Ok((mut velocity, speed, _owner)) = players.get_mut(char_entity) {
        let direction = input.direction.normalize_or_zero();
        velocity.0 = direction * speed.0;
    }
}

pub fn update_positions(
    mut players: Query<(&mut Position, &Velocity)>,
    time: Res<Time>,
) {
    for (mut position, velocity) in &mut players {
        // Update position based on velocity
        position.0 += velocity.0 * time.delta_secs();

        // Clamp to world bounds
        position.0.x = position.0.x.clamp(-WORLD_WIDTH / 2.0, WORLD_WIDTH / 2.0);
        position.0.y = position.0.y.clamp(-WORLD_HEIGHT / 2.0, WORLD_HEIGHT / 2.0);
    }
}
