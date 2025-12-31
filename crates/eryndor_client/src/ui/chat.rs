//! Chat system UI.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_replicon::prelude::*;
use eryndor_shared::*;

use crate::game_state::MyClientState;
use super::state::UiState;

/// Chat window for sending admin commands and regular chat messages
pub fn chat_window(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    client_state: Res<MyClientState>,
    character_query: Query<&Character>,
    mut commands: Commands,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Get character name for displaying own messages
    let character_name = if let Some(player_entity) = client_state.player_entity {
        character_query.get(player_entity)
            .map(|c| c.name.clone())
            .unwrap_or_else(|_| "Unknown".to_string())
    } else {
        "Unknown".to_string()
    };

    // Chat window at bottom-left of screen - always visible
    egui::Window::new("Chat")
        .default_pos([10.0, 400.0])
        .default_size([500.0, 250.0])
        .resizable(true)
        .show(ctx, |ui| {
            // Chat history display (scrollable area)
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if ui_state.chat_history.is_empty() {
                        ui.label("No messages yet. Type to chat with other players!");
                        if ui_state.is_admin {
                            ui.label("Admin commands: /ban, /unban, /kick, /broadcast, /help");
                        }
                    } else {
                        for message in &ui_state.chat_history {
                            ui.label(message);
                        }
                    }
                });

            ui.separator();

            // Chat input field
            let response = ui.text_edit_singleline(&mut ui_state.chat_input);

            // Track focus state changes
            let current_focus = response.has_focus();

            // If chat just gained focus, send stop movement command
            if current_focus && !ui_state.chat_previous_focus {
                commands.client_trigger(MoveInput { direction: Vec2::ZERO });
            }

            ui_state.chat_previous_focus = ui_state.chat_has_focus;
            ui_state.chat_has_focus = current_focus;

            // Send message on Enter key
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let message = ui_state.chat_input.trim().to_string();

                if !message.is_empty() {
                    // Check if it's an admin command (starts with /)
                    if message.starts_with('/') {
                        commands.client_trigger(AdminCommandRequest {
                            command: message.clone(),
                        });
                        ui_state.chat_history.push(format!("[{}] {}", character_name, message));
                    } else {
                        commands.client_trigger(SendChatMessage {
                            message: message.clone(),
                        });
                        ui_state.chat_history.push(format!("[{}] {}", character_name, message));
                    }

                    // Keep only last 50 messages
                    if ui_state.chat_history.len() > 50 {
                        ui_state.chat_history.remove(0);
                    }

                    ui_state.chat_input.clear();
                    response.request_focus();
                }
            }

            let help_text = if ui_state.is_admin {
                "Press Enter to send | Type / for admin commands"
            } else {
                "Press Enter to send"
            };
            ui.label(help_text);
        });
}

/// Receive chat messages from server and add them to chat history
pub fn receive_chat_messages(
    mut ui_state: ResMut<UiState>,
    mut chat_events: Option<MessageReader<ChatMessage>>,
) {
    let Some(chat_events) = chat_events.as_mut() else {
        return;
    };

    for chat_event in chat_events.read() {
        let formatted_message = format!("[{}] {}", chat_event.sender, chat_event.message);
        ui_state.chat_history.push(formatted_message);

        if ui_state.chat_history.len() > 50 {
            ui_state.chat_history.remove(0);
        }
    }
}
