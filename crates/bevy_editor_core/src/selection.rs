//! Generic selection system for editor entities
//!
//! This module provides a reusable selection system that can be integrated into
//! any Bevy editor. It handles multi-selection, selection visualization, and
//! basic selection operations.

use bevy::prelude::*;
use std::collections::HashSet;

/// Marker component for entities that can be selected in the editor
#[derive(Component, Debug, Clone)]
pub struct Selectable;

/// Resource tracking currently selected entities
#[derive(Resource, Default)]
pub struct Selection {
    pub selected: HashSet<Entity>,
}

impl Selection {
    pub fn clear(&mut self) {
        self.selected.clear();
    }

    pub fn select(&mut self, entity: Entity) {
        self.selected.clear();
        self.selected.insert(entity);
    }

    pub fn add(&mut self, entity: Entity) {
        self.selected.insert(entity);
    }

    pub fn remove(&mut self, entity: Entity) {
        self.selected.remove(&entity);
    }

    pub fn toggle(&mut self, entity: Entity) {
        if self.selected.contains(&entity) {
            self.selected.remove(&entity);
        } else {
            self.selected.insert(entity);
        }
    }

    pub fn is_selected(&self, entity: Entity) -> bool {
        self.selected.contains(&entity)
    }

    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    pub fn len(&self) -> usize {
        self.selected.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.selected.iter()
    }
}

/// Event for selection changes
#[derive(Event, Message, Debug, Clone)]
pub enum SelectionEvent {
    /// An entity was selected (replacing previous selection)
    Selected(Entity),
    /// An entity was added to the selection
    Added(Entity),
    /// An entity was removed from the selection
    Removed(Entity),
    /// Selection was cleared
    Cleared,
}

/// System to emit SelectionEvent when selection changes
pub fn emit_selection_events(
    selection: Res<Selection>,
    mut last_selection: Local<HashSet<Entity>>,
    mut events: MessageWriter<SelectionEvent>,
) {
    if selection.is_changed() {
        // Check what changed
        let current = &selection.selected;

        // Check for removals
        for entity in last_selection.difference(current) {
            events.write(SelectionEvent::Removed(*entity));
        }

        // Check for additions
        for entity in current.difference(&*last_selection) {
            events.write(SelectionEvent::Added(*entity));
        }

        // If completely cleared
        if current.is_empty() && !last_selection.is_empty() {
            events.write(SelectionEvent::Cleared);
        }

        *last_selection = current.clone();
    }
}

/// Helper function to get cursor world position in 2D
pub fn get_cursor_world_pos_2d(
    windows: &Query<&Window>,
    camera_q: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let Ok(window) = windows.single() else {
        return None;
    };
    let Ok((camera, camera_transform)) = camera_q.single() else {
        return None;
    };

    window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
}

/// System to handle basic 2D selection with mouse clicks
///
/// This is a basic implementation that can be customized or replaced.
/// It selects the nearest entity within a 20-unit radius.
pub fn handle_2d_selection_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    selectable_entities: Query<(Entity, &Transform), With<Selectable>>,
    mut selection: ResMut<Selection>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cursor_world_pos) = get_cursor_world_pos_2d(&windows, &camera_q) else {
        // Clicked on nothing, clear selection if not holding Ctrl
        if !keyboard.pressed(KeyCode::ControlLeft) && !keyboard.pressed(KeyCode::ControlRight) {
            selection.clear();
        }
        return;
    };

    // Find entity under cursor
    let mut closest_entity = None;
    let mut closest_distance = f32::MAX;

    for (entity, transform) in selectable_entities.iter() {
        let distance = transform.translation.xy().distance(cursor_world_pos);

        // Use a reasonable selection radius
        if distance < 20.0 && distance < closest_distance {
            closest_distance = distance;
            closest_entity = Some(entity);
        }
    }

    if let Some(entity) = closest_entity {
        if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
            // Multi-select with Ctrl
            selection.toggle(entity);
        } else {
            // Single select
            selection.select(entity);
        }
    } else if !keyboard.pressed(KeyCode::ControlLeft) && !keyboard.pressed(KeyCode::ControlRight) {
        // Clicked on empty space, clear selection
        selection.clear();
    }
}

/// Plugin to add selection system to your editor
pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Selection>()
            .add_message::<SelectionEvent>()
            .add_systems(Update, emit_selection_events);
    }
}
