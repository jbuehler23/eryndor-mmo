//! Command system for undo/redo operations
//!
//! Implements the command pattern for editor operations, enabling undo/redo functionality.

pub mod command;
pub mod clipboard;
pub mod shortcuts;

pub use command::*;
pub use clipboard::TileClipboard;
pub use shortcuts::handle_keyboard_shortcuts;

use bevy::prelude::*;

use crate::project::Project;
use crate::render::RenderState;

/// Resource that manages command history for undo/redo
#[derive(Resource)]
pub struct CommandHistory {
    /// Stack of commands that can be undone (most recent at end)
    undo_stack: Vec<Box<dyn EditorCommand>>,
    /// Stack of commands that can be redone (most recent at end)
    redo_stack: Vec<Box<dyn EditorCommand>>,
    /// Maximum number of commands to keep in history
    max_size: usize,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size: 100,
        }
    }
}

impl CommandHistory {
    /// Execute a command and add it to the undo stack
    pub fn execute(
        &mut self,
        command: Box<dyn EditorCommand>,
        project: &mut Project,
        render_state: &mut RenderState,
    ) {
        // Execute the command and get the inverse for undo
        if let Some(inverse) = command.execute(project) {
            self.undo_stack.push(inverse);

            // Clear redo stack when new command is executed
            self.redo_stack.clear();

            // Trim undo stack if it exceeds max size
            while self.undo_stack.len() > self.max_size {
                self.undo_stack.remove(0);
            }

            // Mark render state for rebuild
            render_state.needs_rebuild = true;
        }
    }

    /// Push an already-executed command's inverse onto the undo stack
    /// Use this when the command was executed externally (e.g., terrain painting)
    pub fn push_undo(&mut self, inverse_command: Box<dyn EditorCommand>) {
        self.undo_stack.push(inverse_command);

        // Clear redo stack when new command is added
        self.redo_stack.clear();

        // Trim undo stack if it exceeds max size
        while self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }
    }

    /// Undo the last command
    /// Returns true if a command was undone
    pub fn undo(&mut self, project: &mut Project, render_state: &mut RenderState) -> bool {
        if let Some(command) = self.undo_stack.pop() {
            if let Some(inverse) = command.execute(project) {
                self.redo_stack.push(inverse);
                render_state.needs_rebuild = true;
                return true;
            }
        }
        false
    }

    /// Redo the last undone command
    /// Returns true if a command was redone
    pub fn redo(&mut self, project: &mut Project, render_state: &mut RenderState) -> bool {
        if let Some(command) = self.redo_stack.pop() {
            if let Some(inverse) = command.execute(project) {
                self.undo_stack.push(inverse);
                render_state.needs_rebuild = true;
                return true;
            }
        }
        false
    }

    /// Check if there are commands that can be undone
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if there are commands that can be redone
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the description of the command that would be undone
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.last().map(|cmd| cmd.description())
    }

    /// Get the description of the command that would be redone
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.last().map(|cmd| cmd.description())
    }

    /// Clear all command history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}
