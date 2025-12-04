//! Keyboard shortcut handling for editor operations
//!
//! Provides standard editor hotkeys (Ctrl+Z, Ctrl+S, etc.)

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::project::Project;
use crate::render::RenderState;
use crate::EditorState;
use crate::ui::PendingAction;

use super::{CommandHistory, TileClipboard};

/// System to handle keyboard shortcuts for editor operations
pub fn handle_keyboard_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut history: ResMut<CommandHistory>,
    mut project: ResMut<Project>,
    mut render_state: ResMut<RenderState>,
    mut editor_state: ResMut<EditorState>,
    mut clipboard: ResMut<TileClipboard>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Don't handle shortcuts if egui has keyboard focus (e.g., text input)
    if ctx.wants_keyboard_input() {
        return;
    }

    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // Undo: Ctrl+Z (without shift)
    if ctrl && keyboard.just_pressed(KeyCode::KeyZ) && !shift {
        if history.undo(&mut project, &mut render_state) {
            // Undo successful
        }
    }

    // Redo: Ctrl+Y or Ctrl+Shift+Z
    if ctrl && keyboard.just_pressed(KeyCode::KeyY) {
        if history.redo(&mut project, &mut render_state) {
            // Redo successful
        }
    }
    if ctrl && shift && keyboard.just_pressed(KeyCode::KeyZ) {
        if history.redo(&mut project, &mut render_state) {
            // Redo successful
        }
    }

    // Save: Ctrl+S
    if ctrl && keyboard.just_pressed(KeyCode::KeyS) {
        editor_state.pending_action = Some(PendingAction::Save);
    }

    // Copy: Ctrl+C
    if ctrl && keyboard.just_pressed(KeyCode::KeyC) {
        clipboard.copy_selection(&editor_state.tile_selection, &project, &editor_state);
    }

    // Cut: Ctrl+X
    if ctrl && keyboard.just_pressed(KeyCode::KeyX) {
        // Copy first
        clipboard.copy_selection(&editor_state.tile_selection, &project, &editor_state);
        // Then delete (handled in separate system for undo support)
        editor_state.pending_delete_selection = true;
    }

    // Paste: Ctrl+V
    if ctrl && keyboard.just_pressed(KeyCode::KeyV) {
        if clipboard.has_content() {
            editor_state.is_pasting = true;
        }
    }

    // Delete: Delete or Backspace key
    if keyboard.just_pressed(KeyCode::Delete) || keyboard.just_pressed(KeyCode::Backspace) {
        if !editor_state.tile_selection.is_empty() {
            editor_state.pending_delete_selection = true;
        }
    }

    // Escape: Clear selection or cancel paste mode
    if keyboard.just_pressed(KeyCode::Escape) {
        if editor_state.is_pasting {
            editor_state.is_pasting = false;
        } else {
            editor_state.tile_selection.clear();
        }
    }

    // Select All: Ctrl+A
    if ctrl && keyboard.just_pressed(KeyCode::KeyA) {
        select_all_visible_tiles(&mut editor_state, &project);
    }
}

/// Select all tiles in the current layer
fn select_all_visible_tiles(editor_state: &mut EditorState, project: &Project) {
    let Some(level_id) = editor_state.selected_level else {
        return;
    };
    let Some(layer_idx) = editor_state.selected_layer else {
        return;
    };

    let Some(level) = project.levels.iter().find(|l| l.id == level_id) else {
        return;
    };

    editor_state.tile_selection.clear();
    editor_state.tile_selection.select_rectangle(
        level_id,
        layer_idx,
        0,
        0,
        level.width.saturating_sub(1),
        level.height.saturating_sub(1),
        false,
    );
}
