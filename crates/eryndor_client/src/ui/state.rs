//! UI state types and data structures.

use bevy::prelude::*;
use eryndor_shared::*;

/// State for the system menu / admin dashboard
pub struct SystemMenuState {
    pub active_tab: SystemMenuTab,
    pub player_list: Vec<OnlinePlayerInfo>,
    pub ban_list: Vec<BanInfo>,
    pub server_stats: Option<ServerStatsResponse>,
    pub audit_logs: Vec<AuditLogEntry>,
    pub audit_logs_total: u32,
    pub audit_logs_offset: u32,
    pub audit_logs_limit: u32,
    // UI input fields
    pub ban_form_username: String,
    pub ban_form_duration: u32,
    pub ban_form_reason: String,
    pub ban_username: String,
    pub ban_duration: String,
    pub ban_reason: String,
    pub kick_username: String,
    pub kick_reason: String,
}

impl Default for SystemMenuState {
    fn default() -> Self {
        Self {
            active_tab: SystemMenuTab::default(),
            player_list: Vec::new(),
            ban_list: Vec::new(),
            server_stats: None,
            audit_logs: Vec::new(),
            audit_logs_total: 0,
            audit_logs_offset: 0,
            audit_logs_limit: 20,
            ban_form_username: String::new(),
            ban_form_duration: 0,
            ban_form_reason: String::new(),
            ban_username: String::new(),
            ban_duration: String::new(),
            ban_reason: String::new(),
            kick_username: String::new(),
            kick_reason: String::new(),
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum SystemMenuTab {
    #[default]
    Players,
    Bans,
    Stats,
    Logs,
}

/// Main UI state resource
#[derive(Resource)]
pub struct UiState {
    pub email: String,
    pub username: String,
    pub password: String,
    pub new_character_name: String,
    pub selected_class: CharacterClass,
    pub show_create_character: bool,
    pub show_inventory: bool,
    pub show_equipment: bool,
    pub show_character_stats: bool,
    pub show_esc_menu: bool,
    pub quest_dialogue: Option<QuestDialogueData>,
    pub trainer_window: Option<TrainerWindowData>,
    pub loot_window: Option<LootWindowData>,
    pub show_register_tab: bool,
    pub oauth_checked: bool,
    pub chat_input: String,
    pub chat_history: Vec<String>,
    pub chat_has_focus: bool,
    pub chat_previous_focus: bool,
    pub is_admin: bool,
    pub show_system_menu: bool,
    pub system_menu: SystemMenuState,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            email: String::new(),
            username: String::new(),
            password: String::new(),
            new_character_name: String::new(),
            selected_class: CharacterClass::Rogue,
            show_create_character: false,
            show_inventory: false,
            show_equipment: false,
            show_character_stats: false,
            show_esc_menu: false,
            quest_dialogue: None,
            trainer_window: None,
            loot_window: None,
            show_register_tab: false,
            oauth_checked: false,
            chat_input: String::new(),
            chat_history: Vec::new(),
            chat_has_focus: false,
            chat_previous_focus: false,
            is_admin: false,
            show_system_menu: false,
            system_menu: SystemMenuState::default(),
        }
    }
}

/// Data for the loot container window
#[derive(Clone)]
pub struct LootWindowData {
    pub container_entity: Entity,
    pub contents: Vec<LootContents>,
    pub source_name: String,
}

/// Data for the quest dialogue window
#[derive(Clone)]
pub struct QuestDialogueData {
    pub npc_name: String,
    pub quest_id: u32,
    pub quest_name: String,
    pub description: String,
    pub objectives_text: String,
    pub rewards_text: String,
}

/// Data for the trainer window
#[derive(Clone)]
pub struct TrainerWindowData {
    pub npc_name: String,
    pub items_for_sale: Vec<TrainerItem>,
    pub trainer_type: Option<TrainerType>,
    pub teaching_quests: Vec<TrainerQuestInfo>,
    pub active_tab: TrainerTab,
}

/// Tab selection for trainer window
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TrainerTab {
    #[default]
    Shop,
    Training,
}
