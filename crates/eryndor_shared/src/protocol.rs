use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::*;

// ============================================================================
// CLIENT -> SERVER MESSAGES
// ============================================================================

/// Login request from client
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Create new account
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct CreateAccountRequest {
    pub username: String,
    pub password: String,
}

/// Request to create a new character
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct CreateCharacterRequest {
    pub name: String,
    pub class: CharacterClass,
}

/// Request to select and spawn a character
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct SelectCharacterRequest {
    pub character_id: i64,
}

/// Movement input from client
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct MoveInput {
    pub direction: Vec2,
}

/// Set current target
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct SetTargetRequest {
    pub target: Option<Entity>,
}

/// Use an ability
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct UseAbilityRequest {
    pub ability_id: u32,
}

/// Pick up a world item
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct PickupItemRequest {
    pub item_entity: Entity,
}

/// Drop an item from inventory
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct DropItemRequest {
    pub slot_index: usize,
}

/// Equip an item from inventory
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct EquipItemRequest {
    pub slot_index: usize,
}

/// Interact with NPC
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct InteractNpcRequest {
    pub npc_entity: Entity,
}

/// Accept a quest from NPC
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct AcceptQuestRequest {
    pub quest_id: u32,
}

/// Complete a quest with NPC
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct CompleteQuestRequest {
    pub quest_id: u32,
}

/// Set hotbar slot
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct SetHotbarSlotRequest {
    pub slot_index: usize,
    pub content: Option<HotbarSlot>,
}

// ============================================================================
// SERVER -> CLIENT MESSAGES
// ============================================================================

/// Login response
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub account_id: Option<i64>,
}

/// Account creation response
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct CreateAccountResponse {
    pub success: bool,
    pub message: String,
}

/// List of characters for account
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct CharacterListResponse {
    pub characters: Vec<CharacterData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CharacterData {
    pub id: i64,
    pub name: String,
    pub class: CharacterClass,
    pub level: u32,
}

/// Character creation response
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct CreateCharacterResponse {
    pub success: bool,
    pub message: String,
    pub character: Option<CharacterData>,
}

/// Character selection response - tells client which character was selected
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct SelectCharacterResponse {
    pub character_id: i64,
}

/// Combat event for visual feedback
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct CombatEvent {
    pub attacker: Entity,
    pub target: Entity,
    pub damage: f32,
    pub ability_id: u32,
    pub is_crit: bool,
}

/// Quest update notification
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct QuestUpdateEvent {
    pub quest_id: u32,
    pub message: String,
}

/// Entity death event
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct DeathEvent {
    pub entity: Entity,
}

/// Chat message
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub sender: String,
    pub message: String,
}

/// Notification message to client
#[derive(Event, Message, Serialize, Deserialize, Clone, Debug)]
pub struct NotificationEvent {
    pub message: String,
    pub notification_type: NotificationType,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}
