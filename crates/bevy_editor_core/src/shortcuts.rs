//! Generic keyboard shortcut system for editors
//!
//! Provides a registry for keyboard shortcuts that can be customized per editor.

use bevy::prelude::*;
use std::collections::HashMap;

/// A keyboard shortcut binding
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyboardShortcut {
    pub key: KeyCode,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyboardShortcut {
    pub fn new(key: KeyCode) -> Self {
        Self {
            key,
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    /// Check if this shortcut matches the current keyboard state
    pub fn matches(&self, keyboard: &ButtonInput<KeyCode>) -> bool {
        if !keyboard.just_pressed(self.key) {
            return false;
        }

        let ctrl_pressed =
            keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
        let shift_pressed =
            keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
        let alt_pressed = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);

        ctrl_pressed == self.ctrl && shift_pressed == self.shift && alt_pressed == self.alt
    }
}

/// Event emitted when a registered shortcut is triggered
#[derive(Event, Message, Debug, Clone)]
pub struct ShortcutEvent {
    pub id: String,
}

/// Resource for managing keyboard shortcuts
#[derive(Resource, Default)]
pub struct ShortcutRegistry {
    shortcuts: HashMap<String, KeyboardShortcut>,
}

impl ShortcutRegistry {
    pub fn register(&mut self, id: impl Into<String>, shortcut: KeyboardShortcut) {
        self.shortcuts.insert(id.into(), shortcut);
    }

    pub fn unregister(&mut self, id: &str) {
        self.shortcuts.remove(id);
    }

    pub fn get(&self, id: &str) -> Option<&KeyboardShortcut> {
        self.shortcuts.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &KeyboardShortcut)> {
        self.shortcuts.iter()
    }
}

/// System to process registered shortcuts and emit events
pub fn process_shortcuts_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    registry: Res<ShortcutRegistry>,
    mut events: MessageWriter<ShortcutEvent>,
) {
    for (id, shortcut) in registry.iter() {
        if shortcut.matches(&keyboard) {
            events.write(ShortcutEvent { id: id.clone() });
        }
    }
}

/// Helper function to check if any modifier keys are pressed
pub fn any_modifiers_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
        KeyCode::AltLeft,
        KeyCode::AltRight,
    ])
}

/// Plugin to add shortcut system to your editor
pub struct ShortcutPlugin;

impl Plugin for ShortcutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShortcutRegistry>()
            .add_message::<ShortcutEvent>()
            .add_systems(Update, process_shortcuts_system);
    }
}
