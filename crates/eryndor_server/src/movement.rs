use bevy::prelude::*;
use bevy_replicon::prelude::*;
use eryndor_shared::*;
use crate::auth::ActiveCharacterEntity;
use avian2d::prelude::LinearVelocity;

pub fn handle_move_input(
    trigger: On<FromClient<MoveInput>>,
    clients: Query<&ActiveCharacterEntity>,
    mut players: Query<(&mut Velocity, &mut LinearVelocity, &MoveSpeed, &OwnedBy)>,
) {
    let Some(client_entity) = trigger.client_id.entity() else { return };
    let input = trigger.event();

    // Get client's character
    let Ok(active_char) = clients.get(client_entity) else { return };
    let char_entity = active_char.0;

    // Update both our custom velocity (for replication) and physics velocity
    if let Ok((mut velocity, mut physics_velocity, speed, _owner)) = players.get_mut(char_entity) {
        let direction = input.direction.normalize_or_zero();
        let vel = direction * speed.0;
        velocity.0 = vel;
        physics_velocity.0 = vel;
    }
}

pub fn update_positions(
    mut players: Query<(&mut Position, &Velocity)>,
    time: Res<Time>,
) {
    // NOTE: This system is now deprecated and will be removed once physics is fully integrated
    // Physics engine handles position updates via PhysicsPosition
    // The sync_physics_to_position system copies PhysicsPosition -> Position
    //
    // Keeping this for now to avoid breaking non-physics entities
    // TODO: Remove this once all entities use physics
}
