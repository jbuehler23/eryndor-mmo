//! Generic CRUD handlers for content types.
//!
//! This module provides generic handlers that work for any JSON content type
//! (items, enemies, NPCs, quests, abilities, loot tables) by parameterizing
//! the directory name, file extension, and display name.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::path::PathBuf;
use tracing::{info, warn};

use super::{ApiResponse, EditorApiState, ListQuery, ListResponse};

/// Configuration for a content type's CRUD operations.
#[derive(Clone)]
pub struct ContentConfig {
    /// Directory name under content/ (e.g., "items", "enemies")
    pub directory: &'static str,
    /// File extension without dot (e.g., "item", "enemy")
    pub extension: &'static str,
    /// Display name for error messages (e.g., "Item", "Enemy")
    pub display_name: &'static str,
}

/// Content type configurations
pub mod configs {
    use super::ContentConfig;

    pub const ITEM: ContentConfig = ContentConfig {
        directory: "items",
        extension: "item",
        display_name: "Item",
    };

    pub const ENEMY: ContentConfig = ContentConfig {
        directory: "enemies",
        extension: "enemy",
        display_name: "Enemy",
    };

    pub const NPC: ContentConfig = ContentConfig {
        directory: "npcs",
        extension: "npc",
        display_name: "NPC",
    };

    pub const QUEST: ContentConfig = ContentConfig {
        directory: "quests",
        extension: "quest",
        display_name: "Quest",
    };

    pub const ABILITY: ContentConfig = ContentConfig {
        directory: "abilities",
        extension: "ability",
        display_name: "Ability",
    };

    pub const LOOT_TABLE: ContentConfig = ContentConfig {
        directory: "loot",
        extension: "loot",
        display_name: "Loot table",
    };
}

/// Convert a name to a filesystem-safe slug (lowercase, spaces to underscores)
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            ' ' | '-' => '_',
            _ => '_',
        })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// Find a file by looking up the content item by ID in the directory
fn find_file_by_id(dir_path: &std::path::Path, id: &str) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                        let file_id = match data.get("id") {
                            Some(serde_json::Value::Number(n)) => n.to_string(),
                            Some(serde_json::Value::String(s)) => s.clone(),
                            _ => continue,
                        };
                        if file_id == id {
                            return Some(path);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Generic list handler - reads all JSON files from the content directory
pub async fn list(
    state: &EditorApiState,
    query: &ListQuery,
    config: &ContentConfig,
) -> impl IntoResponse {
    let content_path = state.content_path.join(config.directory);
    let mut items = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&content_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                        items.push(data);
                    }
                }
            }
        }
    }

    let total = items.len();
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);

    ApiResponse::success(ListResponse {
        items,
        total,
        page,
        per_page,
    })
}

/// Generic get handler - finds a content item by ID
pub async fn get(
    state: &EditorApiState,
    id: &str,
    config: &ContentConfig,
) -> impl IntoResponse {
    let content_path = state.content_path.join(config.directory);

    let file_path = match find_file_by_id(&content_path, id) {
        Some(path) => path,
        None => return (
            StatusCode::NOT_FOUND,
            ApiResponse::<serde_json::Value>::error(format!("{} not found", config.display_name)),
        ),
    };

    match std::fs::read_to_string(&file_path) {
        Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(data) => (StatusCode::OK, ApiResponse::success(data)),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::error(format!("Failed to parse {}: {}", config.display_name.to_lowercase(), e)),
            ),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("Failed to read {}: {}", config.display_name.to_lowercase(), e)),
        ),
    }
}

/// Generic create handler - creates a new content item
pub async fn create(
    state: &EditorApiState,
    data: serde_json::Value,
    config: &ContentConfig,
) -> impl IntoResponse {
    let content_path = state.content_path.join(config.directory);

    // Ensure directory exists
    if let Err(e) = std::fs::create_dir_all(&content_path) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<serde_json::Value>::error(format!(
                "Failed to create {} directory: {}",
                config.directory, e
            )),
        );
    }

    // Extract name for filename
    let name = match data.get("name") {
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (
            StatusCode::BAD_REQUEST,
            ApiResponse::error(format!("{} must have a 'name' field", config.display_name)),
        ),
    };

    // Extract ID for validation
    let id = match data.get("id") {
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (
            StatusCode::BAD_REQUEST,
            ApiResponse::error(format!("{} must have an 'id' field", config.display_name)),
        ),
    };

    let slug = slugify(&name);
    let file_path = content_path.join(format!("{}.{}.json", slug, config.extension));

    // Check for name conflict
    if file_path.exists() {
        return (
            StatusCode::CONFLICT,
            ApiResponse::error(format!(
                "{} with name '{}' already exists",
                config.display_name, name
            )),
        );
    }

    // Check for ID conflict
    if find_file_by_id(&content_path, &id).is_some() {
        return (
            StatusCode::CONFLICT,
            ApiResponse::error(format!(
                "{} with id '{}' already exists",
                config.display_name, id
            )),
        );
    }

    // Write the file
    let content = match serde_json::to_string_pretty(&data) {
        Ok(c) => c,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("Failed to serialize {}: {}", config.display_name.to_lowercase(), e)),
        ),
    };

    if let Err(e) = std::fs::write(&file_path, &content) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("Failed to write {}: {}", config.display_name.to_lowercase(), e)),
        );
    }

    info!("Created {}: {} ({})", config.display_name.to_lowercase(), name, slug);
    (StatusCode::CREATED, ApiResponse::success(data))
}

/// Generic update handler - updates an existing content item
pub async fn update(
    state: &EditorApiState,
    id: &str,
    data: serde_json::Value,
    config: &ContentConfig,
) -> impl IntoResponse {
    let content_path = state.content_path.join(config.directory);

    // Find existing file
    let old_path = match find_file_by_id(&content_path, id) {
        Some(path) => path,
        None => return (
            StatusCode::NOT_FOUND,
            ApiResponse::<serde_json::Value>::error(format!("{} not found", config.display_name)),
        ),
    };

    // Extract name for potentially renamed file
    let name = match data.get("name") {
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (
            StatusCode::BAD_REQUEST,
            ApiResponse::error(format!("{} must have a 'name' field", config.display_name)),
        ),
    };

    let slug = slugify(&name);
    let new_path = content_path.join(format!("{}.{}.json", slug, config.extension));

    // Check for name conflict with different file
    if new_path != old_path && new_path.exists() {
        return (
            StatusCode::CONFLICT,
            ApiResponse::error(format!(
                "{} with name '{}' already exists",
                config.display_name, name
            )),
        );
    }

    // Serialize the data
    let content = match serde_json::to_string_pretty(&data) {
        Ok(c) => c,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("Failed to serialize {}: {}", config.display_name.to_lowercase(), e)),
        ),
    };

    // Remove old file if name changed
    if new_path != old_path {
        if let Err(e) = std::fs::remove_file(&old_path) {
            warn!("Failed to remove old {} file: {}", config.display_name.to_lowercase(), e);
        }
    }

    // Write the updated file
    if let Err(e) = std::fs::write(&new_path, &content) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("Failed to write {}: {}", config.display_name.to_lowercase(), e)),
        );
    }

    info!("Updated {}: {} ({})", config.display_name.to_lowercase(), name, slug);
    (StatusCode::OK, ApiResponse::success(data))
}

/// Generic delete handler - deletes a content item by ID
pub async fn delete(
    state: &EditorApiState,
    id: &str,
    config: &ContentConfig,
) -> impl IntoResponse {
    let content_path = state.content_path.join(config.directory);

    let file_path = match find_file_by_id(&content_path, id) {
        Some(path) => path,
        None => return (
            StatusCode::NOT_FOUND,
            ApiResponse::<()>::error(format!("{} not found", config.display_name)),
        ),
    };

    if let Err(e) = std::fs::remove_file(&file_path) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("Failed to delete {}: {}", config.display_name.to_lowercase(), e)),
        );
    }

    info!("Deleted {}: {} (id: {})", config.display_name.to_lowercase(), file_path.display(), id);
    (StatusCode::OK, ApiResponse::success(()))
}

// =============================================================================
// Axum Handler Wrappers
// =============================================================================

// Items
pub async fn list_items(
    State(state): State<EditorApiState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    list(&state, &query, &configs::ITEM).await
}

pub async fn get_item(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    get(&state, &id, &configs::ITEM).await
}

pub async fn create_item(
    State(state): State<EditorApiState>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    create(&state, data, &configs::ITEM).await
}

pub async fn update_item(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    update(&state, &id, data, &configs::ITEM).await
}

pub async fn delete_item(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    delete(&state, &id, &configs::ITEM).await
}

// Enemies
pub async fn list_enemies(
    State(state): State<EditorApiState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    list(&state, &query, &configs::ENEMY).await
}

pub async fn get_enemy(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    get(&state, &id, &configs::ENEMY).await
}

pub async fn create_enemy(
    State(state): State<EditorApiState>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    create(&state, data, &configs::ENEMY).await
}

pub async fn update_enemy(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    update(&state, &id, data, &configs::ENEMY).await
}

pub async fn delete_enemy(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    delete(&state, &id, &configs::ENEMY).await
}

// NPCs
pub async fn list_npcs(
    State(state): State<EditorApiState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    list(&state, &query, &configs::NPC).await
}

pub async fn get_npc(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    get(&state, &id, &configs::NPC).await
}

pub async fn create_npc(
    State(state): State<EditorApiState>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    create(&state, data, &configs::NPC).await
}

pub async fn update_npc(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    update(&state, &id, data, &configs::NPC).await
}

pub async fn delete_npc(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    delete(&state, &id, &configs::NPC).await
}

// Quests
pub async fn list_quests(
    State(state): State<EditorApiState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    list(&state, &query, &configs::QUEST).await
}

pub async fn get_quest(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    get(&state, &id, &configs::QUEST).await
}

pub async fn create_quest(
    State(state): State<EditorApiState>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    create(&state, data, &configs::QUEST).await
}

pub async fn update_quest(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    update(&state, &id, data, &configs::QUEST).await
}

pub async fn delete_quest(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    delete(&state, &id, &configs::QUEST).await
}

// Abilities
pub async fn list_abilities(
    State(state): State<EditorApiState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    list(&state, &query, &configs::ABILITY).await
}

pub async fn get_ability(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    get(&state, &id, &configs::ABILITY).await
}

pub async fn create_ability(
    State(state): State<EditorApiState>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    create(&state, data, &configs::ABILITY).await
}

pub async fn update_ability(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    update(&state, &id, data, &configs::ABILITY).await
}

pub async fn delete_ability(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    delete(&state, &id, &configs::ABILITY).await
}

// Loot Tables
pub async fn list_loot_tables(
    State(state): State<EditorApiState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    list(&state, &query, &configs::LOOT_TABLE).await
}

pub async fn get_loot_table(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    get(&state, &id, &configs::LOOT_TABLE).await
}

pub async fn create_loot_table(
    State(state): State<EditorApiState>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    create(&state, data, &configs::LOOT_TABLE).await
}

pub async fn update_loot_table(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    update(&state, &id, data, &configs::LOOT_TABLE).await
}

pub async fn delete_loot_table(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    delete(&state, &id, &configs::LOOT_TABLE).await
}
