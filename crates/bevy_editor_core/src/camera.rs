//! Editor camera controls
//!
//! Custom camera system for the editor viewport that provides:
//! - Middle mouse button drag to pan
//! - Mouse wheel to zoom
//! - Does NOT respond to WASD (reserved for other editor functions)

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

/// Marker component for the editor camera
#[derive(Component)]
pub struct EditorCamera {
    pub zoom: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for EditorCamera {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            min_zoom: 0.1,
            max_zoom: 10.0,
        }
    }
}

/// System to handle editor camera panning with middle mouse button
pub fn camera_pan_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<bevy::input::mouse::MouseMotion>,
    mut camera_query: Query<(&mut Transform, &mut EditorCamera)>,
) {
    // TODO: Add UI interaction blocking when bevy_feathers UI is active

    // Only pan when middle mouse button is held
    if !mouse_button.pressed(MouseButton::Middle) {
        return;
    }

    let mut cameras = camera_query.iter_mut();
    let Some((mut transform, editor_camera)) = cameras.next() else {
        return;
    };

    // Accumulate mouse motion
    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    if delta != Vec2::ZERO {
        // Use the editor camera's zoom as scale
        let scale = editor_camera.zoom;

        // Pan camera (inverted for natural feel)
        transform.translation.x -= delta.x * scale;
        transform.translation.y += delta.y * scale; // Y is inverted in screen space
    }
}

/// System to handle editor camera zoom with mouse wheel
pub fn camera_zoom_system(
    mut scroll_events: MessageReader<MouseWheel>,
    mut camera_query: Query<&mut EditorCamera>,
) {
    // TODO: Add UI interaction blocking when bevy_feathers UI is active

    let mut cameras = camera_query.iter_mut();
    let Some(mut editor_camera) = cameras.next() else {
        return;
    };

    for event in scroll_events.read() {
        let zoom_delta = match event.unit {
            MouseScrollUnit::Line => event.y * 0.1,
            MouseScrollUnit::Pixel => event.y * 0.01,
        };

        // Update zoom (stored in EditorCamera for use in pan system)
        editor_camera.zoom *= 1.0 - zoom_delta;
        editor_camera.zoom = editor_camera
            .zoom
            .clamp(editor_camera.min_zoom, editor_camera.max_zoom);
    }
}
