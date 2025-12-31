//! UI module - all user interface systems and components.
//!
//! This module is organized into submodules:
//! - `state` - UI state types and data structures
//! - `login` - Login and character selection screens
//! - `game` - Main game UI systems and windows
//! - `chat` - Chat system
//! - `admin` - Admin dashboard
//! - `tooltips` - Tooltip helper functions
//! - `helpers` - Helper functions for formatting

pub mod state;
pub mod login;
pub mod game;
pub mod chat;
pub mod admin;
pub mod tooltips;
pub mod helpers;

// Re-export commonly used items
pub use state::{UiState, SystemMenuState, SystemMenuTab, LootWindowData, QuestDialogueData, TrainerWindowData, TrainerTab};
pub use login::{login_ui, character_select_ui, check_oauth_callback};
pub use game::{game_ui, handle_esc_key, handle_quest_dialogue, handle_loot_container_contents, handle_trainer_dialogue};
pub use chat::{chat_window, receive_chat_messages};
