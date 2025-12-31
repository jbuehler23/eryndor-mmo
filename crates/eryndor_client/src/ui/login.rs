//! Login and character selection UI screens.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_replicon::prelude::*;
use eryndor_shared::*;

use crate::game_state::{GameState, MyClientState};
use super::state::UiState;

/// Login screen UI system
pub fn login_ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut commands: Commands,
    client_state: Res<MyClientState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading("Eryndor MMO");
            ui.add_space(40.0);

            // Tab buttons
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 120.0);
                if ui.selectable_label(!ui_state.show_register_tab, "Login").clicked() {
                    ui_state.show_register_tab = false;
                }
                if ui.selectable_label(ui_state.show_register_tab, "Register").clicked() {
                    ui_state.show_register_tab = true;
                }
            });

            ui.add_space(30.0);

            // Show either Login or Register form
            if !ui_state.show_register_tab {
                login_form(ui, &mut ui_state, &mut commands);
            } else {
                register_form(ui, &mut ui_state, &mut commands);
            }

            ui.add_space(20.0);
            oauth_section(ui);
            ui.add_space(20.0);

            // Show notifications
            for notification in &client_state.notifications {
                ui.colored_label(egui::Color32::YELLOW, notification);
            }
        });
    });
}

fn login_form(ui: &mut egui::Ui, ui_state: &mut UiState, commands: &mut Commands) {
    ui.heading("Login");
    ui.add_space(10.0);

    ui.label("Username:");
    ui.text_edit_singleline(&mut ui_state.username);
    ui.add_space(10.0);

    ui.label("Password:");
    ui.add(egui::TextEdit::singleline(&mut ui_state.password).password(true));
    ui.add_space(20.0);

    if ui.button("Login").clicked()
        && !ui_state.username.is_empty() && !ui_state.password.is_empty() {
        info!("Sending login request for user: {}", ui_state.username);
        commands.client_trigger(LoginRequest {
            username: ui_state.username.clone(),
            password: ui_state.password.clone(),
        });
    }
}

fn register_form(ui: &mut egui::Ui, ui_state: &mut UiState, commands: &mut Commands) {
    ui.heading("Create New Account");
    ui.add_space(10.0);

    ui.label("Email:");
    ui.text_edit_singleline(&mut ui_state.email);
    ui.add_space(10.0);

    ui.label("Username:");
    ui.text_edit_singleline(&mut ui_state.username);
    ui.add_space(10.0);

    ui.label("Password:");
    ui.add(egui::TextEdit::singleline(&mut ui_state.password).password(true));
    ui.add_space(5.0);
    ui.colored_label(egui::Color32::GRAY, "Min 8 characters, 1 uppercase, 1 number");
    ui.add_space(20.0);

    if ui.button("Create Account").clicked()
        && !ui_state.email.is_empty() && !ui_state.username.is_empty() && !ui_state.password.is_empty() {
        info!("Sending create account request for user: {}", ui_state.username);
        commands.client_trigger(CreateAccountRequest {
            email: ui_state.email.clone(),
            username: ui_state.username.clone(),
            password: ui_state.password.clone(),
        });
    }
}

fn oauth_section(ui: &mut egui::Ui) {
    ui.separator();
    ui.add_space(10.0);
    ui.label("Or sign in with:");
    ui.add_space(5.0);

    // Google Sign-In button
    if ui.button("Sign in with Google").clicked() {
        #[cfg(target_family = "wasm")]
        {
            if let Some(window) = web_sys::window() {
                let client_id = "917714705564-l5eikmnq0n0miqaurh7vbmc3dbk26e4r.apps.googleusercontent.com";
                info!("Google Sign-In clicked - opening OAuth popup");
                let redirect_uri = window.location().origin().unwrap_or_else(|_| "http://localhost:4000".to_string());
                let oauth_url = format!(
                    "https://accounts.google.com/o/oauth2/v2/auth?\
                     client_id={}&\
                     redirect_uri={}&\
                     response_type=token&\
                     scope=openid%20email%20profile",
                    client_id, redirect_uri
                );

                let _ = window.open_with_url_and_target_and_features(
                    &oauth_url,
                    "_blank",
                    "width=500,height=600,popup=yes"
                );
            }
        }
        #[cfg(not(target_family = "wasm"))]
        {
            let client_id = "917714705564-l5eikmnq0n0miqaurh7vbmc3dbk26e4r.apps.googleusercontent.com";
            let redirect_uri = "http://localhost:8080";
            let oauth_url = format!(
                "https://accounts.google.com/o/oauth2/v2/auth?\
                 client_id={}&\
                 redirect_uri={}&\
                 response_type=token&\
                 scope=openid%20email%20profile",
                client_id, redirect_uri
            );

            info!("Opening browser for Google Sign-In...");
            if let Err(e) = webbrowser::open(&oauth_url) {
                error!("Failed to open browser: {}", e);
                warn!("Please manually open: {}", oauth_url);
            }

            info!("Native OAuth not fully implemented yet. Please use the web client for OAuth login.");
        }
    }
}

/// Character selection screen UI system
pub fn character_select_ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut commands: Commands,
    client_state: Res<MyClientState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("Select Character");
            ui.add_space(20.0);

            // List characters
            for character in &client_state.characters {
                ui.horizontal(|ui| {
                    ui.label(format!("{} - {} (Level {})", character.name, character.class.as_str(), character.level));

                    if ui.button("Play").clicked() {
                        commands.client_trigger(SelectCharacterRequest {
                            character_id: character.id,
                        });
                        next_state.set(GameState::InGame);
                    }
                });
                ui.add_space(10.0);
            }

            ui.add_space(20.0);

            if ui.button("Create New Character").clicked() {
                ui_state.show_create_character = true;
            }

            // Show notifications
            for notification in &client_state.notifications {
                ui.colored_label(egui::Color32::YELLOW, notification);
            }
        });
    });

    // Create character window
    if ui_state.show_create_character {
        create_character_window(ctx, &mut ui_state, &mut commands);
    }
}

fn create_character_window(ctx: &egui::Context, ui_state: &mut UiState, commands: &mut Commands) {
    egui::Window::new("Create Character")
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label("Character Name:");
            ui.text_edit_singleline(&mut ui_state.new_character_name);
            ui.add_space(10.0);

            ui.label("Class:");
            ui.horizontal(|ui| {
                if ui.selectable_label(matches!(ui_state.selected_class, CharacterClass::Rogue), "Rogue").clicked() {
                    ui_state.selected_class = CharacterClass::Rogue;
                }
                if ui.selectable_label(matches!(ui_state.selected_class, CharacterClass::Mage), "Mage").clicked() {
                    ui_state.selected_class = CharacterClass::Mage;
                }
                if ui.selectable_label(matches!(ui_state.selected_class, CharacterClass::Knight), "Knight").clicked() {
                    ui_state.selected_class = CharacterClass::Knight;
                }
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("Create").clicked()
                    && !ui_state.new_character_name.is_empty() {
                    commands.client_trigger(CreateCharacterRequest {
                        name: ui_state.new_character_name.clone(),
                        class: ui_state.selected_class,
                    });
                    ui_state.show_create_character = false;
                    ui_state.new_character_name.clear();
                }

                if ui.button("Cancel").clicked() {
                    ui_state.show_create_character = false;
                }
            });
        });
}

// Check for OAuth callback tokens in URL (WASM only)
#[cfg(target_family = "wasm")]
pub fn check_oauth_callback(
    mut ui_state: ResMut<UiState>,
    mut commands: Commands,
) {
    use wasm_bindgen::JsCast;

    // Only check once
    if ui_state.oauth_checked {
        return;
    }
    ui_state.oauth_checked = true;

    // Get window and location
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(href) = window.location().href() else {
        return;
    };

    info!("Checking URL for OAuth callback: {}", href);

    // Parse URL hash for OAuth 2.0 implicit flow response
    if let Some(hash) = href.split('#').nth(1) {
        let params: std::collections::HashMap<String, String> = hash
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.split('=');
                Some((parts.next()?.to_string(), parts.next()?.to_string()))
            })
            .collect();

        if let Some(token) = params.get("access_token") {
            info!("Found OAuth token in URL, sending to server");

            commands.client_trigger(OAuthLoginRequest {
                provider: "google".to_string(),
                token: token.clone(),
            });

            // Clean up URL by removing hash
            let clean_url = href.split('#').next().unwrap_or(&href);
            if let Ok(history) = window.history() {
                let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(clean_url));
            }
        }
    }
}

// Stub for non-WASM builds
#[cfg(not(target_family = "wasm"))]
pub fn check_oauth_callback() {
    // OAuth callback only works in WASM
}
