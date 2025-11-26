//! Editor API - HTTP endpoints for the game content editor
//! Provides CRUD operations for zones, items, enemies, NPCs, quests, abilities, loot tables, and assets.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn};

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

/// Find a file by looking up the item/enemy by ID in the directory
fn find_file_by_id(dir_path: &std::path::Path, extension: &str, id: &str) -> Option<std::path::PathBuf> {
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

/// Shared state for editor API
#[derive(Clone)]
pub struct EditorApiState {
    pub assets_path: PathBuf,
    pub content_path: PathBuf,
}

impl EditorApiState {
    pub fn new() -> Self {
        // Use workspace root assets folder
        // When running from workspace root, the path is just "assets"
        let assets_path = PathBuf::from("assets");
        let content_path = assets_path.join("content");

        Self {
            assets_path,
            content_path,
        }
    }
}

/// Generic API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            data: Some(data),
            error: None,
        })
    }

    pub fn error(message: impl Into<String>) -> Json<Self> {
        Json(Self {
            success: false,
            data: None,
            error: Some(message.into()),
        })
    }
}

/// Paginated list response
#[derive(Debug, Serialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}

/// Query parameters for list endpoints
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub search: Option<String>,
    pub filter_type: Option<String>,
}

/// Create the editor API router
pub fn create_editor_router() -> Router {
    let state = EditorApiState::new();

    Router::new()
        // Health check
        .route("/health", get(health_check))
        // Zones
        .route("/zones", get(list_zones))
        .route("/zones", post(create_zone))
        .route("/zones/:id", get(get_zone))
        .route("/zones/:id", put(update_zone))
        .route("/zones/:id", delete(delete_zone))
        // Items
        .route("/items", get(list_items))
        .route("/items", post(create_item))
        .route("/items/:id", get(get_item))
        .route("/items/:id", put(update_item))
        .route("/items/:id", delete(delete_item))
        // Enemies
        .route("/enemies", get(list_enemies))
        .route("/enemies", post(create_enemy))
        .route("/enemies/:id", get(get_enemy))
        .route("/enemies/:id", put(update_enemy))
        .route("/enemies/:id", delete(delete_enemy))
        // NPCs
        .route("/npcs", get(list_npcs))
        .route("/npcs", post(create_npc))
        .route("/npcs/:id", get(get_npc))
        .route("/npcs/:id", put(update_npc))
        .route("/npcs/:id", delete(delete_npc))
        // Quests
        .route("/quests", get(list_quests))
        .route("/quests", post(create_quest))
        .route("/quests/:id", get(get_quest))
        .route("/quests/:id", put(update_quest))
        .route("/quests/:id", delete(delete_quest))
        // Abilities
        .route("/abilities", get(list_abilities))
        .route("/abilities", post(create_ability))
        .route("/abilities/:id", get(get_ability))
        .route("/abilities/:id", put(update_ability))
        .route("/abilities/:id", delete(delete_ability))
        // Loot Tables
        .route("/loot-tables", get(list_loot_tables))
        .route("/loot-tables", post(create_loot_table))
        .route("/loot-tables/:id", get(get_loot_table))
        .route("/loot-tables/:id", put(update_loot_table))
        .route("/loot-tables/:id", delete(delete_loot_table))
        // Assets (sprites, audio, etc.)
        .route("/assets", get(list_assets))
        .route("/assets/upload", post(upload_asset))
        .route("/assets/:path", get(get_asset))
        .route("/assets/:path", delete(delete_asset))
        // Share state with all routes
        .with_state(state)
}

// =============================================================================
// Health Check
// =============================================================================

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": "0.1.0"
    }))
}

// =============================================================================
// Zones API
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ZoneData {
    pub id: String,
    pub name: String,
    pub width: f32,
    pub height: f32,
    pub spawn_point: [f32; 2],
    pub background_color: Option<[f32; 4]>,
    pub entities: Vec<serde_json::Value>,
    pub collision_shapes: Vec<serde_json::Value>,
    pub spawn_regions: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ZoneListItem {
    pub id: String,
    pub name: String,
}

async fn list_zones(
    State(state): State<EditorApiState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let zones_path = state.content_path.join("zones");

    let mut items = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&zones_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(zone) = serde_json::from_str::<serde_json::Value>(&content) {
                        let id = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .replace(".zone", "")
                            .to_string();
                        let name = zone.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&id)
                            .to_string();

                        items.push(ZoneListItem { id, name });
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

async fn get_zone(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let zone_path = state.content_path.join("zones").join(format!("{}.zone.json", id));

    match std::fs::read_to_string(&zone_path) {
        Ok(content) => {
            match serde_json::from_str::<ZoneData>(&content) {
                Ok(zone) => ApiResponse::success(zone),
                Err(e) => ApiResponse::error(format!("Failed to parse zone: {}", e)),
            }
        }
        Err(e) => ApiResponse::error(format!("Zone not found: {}", e)),
    }
}

async fn create_zone(
    State(state): State<EditorApiState>,
    Json(zone): Json<ZoneData>,
) -> impl IntoResponse {
    let zone_path = state.content_path.join("zones").join(format!("{}.zone.json", zone.id));

    if zone_path.exists() {
        return (StatusCode::CONFLICT, ApiResponse::<ZoneData>::error("Zone already exists"));
    }

    match serde_json::to_string_pretty(&zone) {
        Ok(content) => {
            if let Err(e) = std::fs::write(&zone_path, content) {
                return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<ZoneData>::error(format!("Failed to write zone: {}", e)));
            }
            info!("Created zone: {}", zone.id);
            (StatusCode::CREATED, ApiResponse::success(zone))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<ZoneData>::error(format!("Failed to serialize zone: {}", e))),
    }
}

async fn update_zone(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
    Json(zone): Json<ZoneData>,
) -> impl IntoResponse {
    let zone_path = state.content_path.join("zones").join(format!("{}.zone.json", id));

    if !zone_path.exists() {
        return (StatusCode::NOT_FOUND, ApiResponse::<ZoneData>::error("Zone not found"));
    }

    match serde_json::to_string_pretty(&zone) {
        Ok(content) => {
            if let Err(e) = std::fs::write(&zone_path, content) {
                return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<ZoneData>::error(format!("Failed to write zone: {}", e)));
            }
            info!("Updated zone: {}", id);
            (StatusCode::OK, ApiResponse::success(zone))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<ZoneData>::error(format!("Failed to serialize zone: {}", e))),
    }
}

async fn delete_zone(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let zone_path = state.content_path.join("zones").join(format!("{}.zone.json", id));

    if !zone_path.exists() {
        return (StatusCode::NOT_FOUND, ApiResponse::<()>::error("Zone not found"));
    }

    if let Err(e) = std::fs::remove_file(&zone_path) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to delete zone: {}", e)));
    }

    info!("Deleted zone: {}", id);
    (StatusCode::OK, ApiResponse::success(()))
}

// =============================================================================
// Items API (Stub - to be implemented)
// =============================================================================

async fn list_items(
    State(state): State<EditorApiState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let items_path = state.content_path.join("items");

    let mut items = Vec::new();

    // Read from items.json or iterate through item files
    if let Ok(entries) = std::fs::read_dir(&items_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(item_data) = serde_json::from_str::<serde_json::Value>(&content) {
                        items.push(item_data);
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

async fn get_item(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let items_path = state.content_path.join("items");

    // Find item file by ID (since filenames are now name-based)
    let item_path = match find_file_by_id(&items_path, "item", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<serde_json::Value>::error("Item not found")),
    };

    match std::fs::read_to_string(&item_path) {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(item) => (StatusCode::OK, ApiResponse::success(item)),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to parse item: {}", e))),
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to read item: {}", e))),
    }
}

async fn create_item(
    State(state): State<EditorApiState>,
    Json(item): Json<serde_json::Value>,
) -> impl IntoResponse {
    let items_path = state.content_path.join("items");

    // Ensure items directory exists
    if let Err(e) = std::fs::create_dir_all(&items_path) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<serde_json::Value>::error(format!("Failed to create items directory: {}", e)));
    }

    // Extract name from item for filename
    let name = match item.get("name") {
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("Item must have a 'name' field")),
    };

    // Extract ID from item for validation
    let id = match item.get("id") {
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("Item must have an 'id' field")),
    };

    let slug = slugify(&name);
    let item_path = items_path.join(format!("{}.item.json", slug));

    // Check if item with same name already exists
    if item_path.exists() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("Item with name '{}' already exists", name)));
    }

    // Check if item with same ID already exists
    if find_file_by_id(&items_path, "item", &id).is_some() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("Item with id '{}' already exists", id)));
    }

    // Write the item
    let content = match serde_json::to_string_pretty(&item) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to serialize item: {}", e))),
    };

    if let Err(e) = std::fs::write(&item_path, &content) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to write item: {}", e)));
    }

    info!("Created item: {} ({})", name, slug);
    (StatusCode::CREATED, ApiResponse::success(item))
}

async fn update_item(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
    Json(item): Json<serde_json::Value>,
) -> impl IntoResponse {
    let items_path = state.content_path.join("items");

    // Find existing item file by ID
    let old_path = match find_file_by_id(&items_path, "item", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<serde_json::Value>::error("Item not found")),
    };

    // Extract new name from item for potentially renamed file
    let name = match item.get("name") {
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("Item must have a 'name' field")),
    };

    let slug = slugify(&name);
    let new_path = items_path.join(format!("{}.item.json", slug));

    // If name changed and new name conflicts with existing file (that isn't this one)
    if new_path != old_path && new_path.exists() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("Item with name '{}' already exists", name)));
    }

    // Write the updated item
    let content = match serde_json::to_string_pretty(&item) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to serialize item: {}", e))),
    };

    // If name changed, delete old file first
    if new_path != old_path {
        if let Err(e) = std::fs::remove_file(&old_path) {
            warn!("Failed to remove old item file: {}", e);
        }
    }

    if let Err(e) = std::fs::write(&new_path, &content) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to write item: {}", e)));
    }

    info!("Updated item: {} ({})", name, slug);
    (StatusCode::OK, ApiResponse::success(item))
}

async fn delete_item(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let items_path = state.content_path.join("items");

    // Find item file by ID
    let item_path = match find_file_by_id(&items_path, "item", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<()>::error("Item not found")),
    };

    if let Err(e) = std::fs::remove_file(&item_path) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to delete item: {}", e)));
    }

    info!("Deleted item: {} (id: {})", item_path.display(), id);
    (StatusCode::OK, ApiResponse::success(()))
}

// =============================================================================
// Enemies API
// =============================================================================

async fn list_enemies(
    State(state): State<EditorApiState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let enemies_path = state.content_path.join("enemies");

    let mut items = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&enemies_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(enemy_data) = serde_json::from_str::<serde_json::Value>(&content) {
                        items.push(enemy_data);
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

async fn get_enemy(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let enemies_path = state.content_path.join("enemies");

    // Find enemy file by ID (since filenames are now name-based)
    let enemy_path = match find_file_by_id(&enemies_path, "enemy", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<serde_json::Value>::error("Enemy not found")),
    };

    match std::fs::read_to_string(&enemy_path) {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(enemy) => (StatusCode::OK, ApiResponse::success(enemy)),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to parse enemy: {}", e))),
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to read enemy: {}", e))),
    }
}

async fn create_enemy(
    State(state): State<EditorApiState>,
    Json(enemy): Json<serde_json::Value>,
) -> impl IntoResponse {
    let enemies_path = state.content_path.join("enemies");

    // Ensure enemies directory exists
    if let Err(e) = std::fs::create_dir_all(&enemies_path) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<serde_json::Value>::error(format!("Failed to create enemies directory: {}", e)));
    }

    // Extract name from enemy for filename
    let name = match enemy.get("name") {
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("Enemy must have a 'name' field")),
    };

    // Extract ID from enemy for validation
    let id = match enemy.get("id") {
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("Enemy must have an 'id' field")),
    };

    let slug = slugify(&name);
    let enemy_path = enemies_path.join(format!("{}.enemy.json", slug));

    // Check if enemy with same name already exists
    if enemy_path.exists() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("Enemy with name '{}' already exists", name)));
    }

    // Check if enemy with same ID already exists
    if find_file_by_id(&enemies_path, "enemy", &id).is_some() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("Enemy with id '{}' already exists", id)));
    }

    // Write the enemy
    let content = match serde_json::to_string_pretty(&enemy) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to serialize enemy: {}", e))),
    };

    if let Err(e) = std::fs::write(&enemy_path, &content) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to write enemy: {}", e)));
    }

    info!("Created enemy: {} ({})", name, slug);
    (StatusCode::CREATED, ApiResponse::success(enemy))
}

async fn update_enemy(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
    Json(enemy): Json<serde_json::Value>,
) -> impl IntoResponse {
    let enemies_path = state.content_path.join("enemies");

    // Find existing enemy file by ID
    let old_path = match find_file_by_id(&enemies_path, "enemy", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<serde_json::Value>::error("Enemy not found")),
    };

    // Extract new name from enemy for potentially renamed file
    let name = match enemy.get("name") {
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("Enemy must have a 'name' field")),
    };

    let slug = slugify(&name);
    let new_path = enemies_path.join(format!("{}.enemy.json", slug));

    // If name changed and new name conflicts with existing file (that isn't this one)
    if new_path != old_path && new_path.exists() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("Enemy with name '{}' already exists", name)));
    }

    // Write the updated enemy
    let content = match serde_json::to_string_pretty(&enemy) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to serialize enemy: {}", e))),
    };

    // If name changed, delete old file first
    if new_path != old_path {
        if let Err(e) = std::fs::remove_file(&old_path) {
            warn!("Failed to remove old enemy file: {}", e);
        }
    }

    if let Err(e) = std::fs::write(&new_path, &content) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to write enemy: {}", e)));
    }

    info!("Updated enemy: {} ({})", name, slug);
    (StatusCode::OK, ApiResponse::success(enemy))
}

async fn delete_enemy(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let enemies_path = state.content_path.join("enemies");

    // Find enemy file by ID
    let enemy_path = match find_file_by_id(&enemies_path, "enemy", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<()>::error("Enemy not found")),
    };

    if let Err(e) = std::fs::remove_file(&enemy_path) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to delete enemy: {}", e)));
    }

    info!("Deleted enemy: {} (id: {})", enemy_path.display(), id);
    (StatusCode::OK, ApiResponse::success(()))
}

// =============================================================================
// NPCs API
// =============================================================================

async fn list_npcs(
    State(state): State<EditorApiState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let npcs_path = state.content_path.join("npcs");

    let mut items = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&npcs_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(npc_data) = serde_json::from_str::<serde_json::Value>(&content) {
                        items.push(npc_data);
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

async fn get_npc(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let npcs_path = state.content_path.join("npcs");

    let npc_path = match find_file_by_id(&npcs_path, "npc", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<serde_json::Value>::error("NPC not found")),
    };

    match std::fs::read_to_string(&npc_path) {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(npc) => (StatusCode::OK, ApiResponse::success(npc)),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to parse NPC: {}", e))),
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to read NPC: {}", e))),
    }
}

async fn create_npc(
    State(state): State<EditorApiState>,
    Json(npc): Json<serde_json::Value>,
) -> impl IntoResponse {
    let npcs_path = state.content_path.join("npcs");

    if let Err(e) = std::fs::create_dir_all(&npcs_path) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<serde_json::Value>::error(format!("Failed to create npcs directory: {}", e)));
    }

    let name = match npc.get("name") {
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("NPC must have a 'name' field")),
    };

    let id = match npc.get("id") {
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("NPC must have an 'id' field")),
    };

    let slug = slugify(&name);
    let npc_path = npcs_path.join(format!("{}.npc.json", slug));

    if npc_path.exists() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("NPC with name '{}' already exists", name)));
    }

    if find_file_by_id(&npcs_path, "npc", &id).is_some() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("NPC with id '{}' already exists", id)));
    }

    let content = match serde_json::to_string_pretty(&npc) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to serialize NPC: {}", e))),
    };

    if let Err(e) = std::fs::write(&npc_path, &content) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to write NPC: {}", e)));
    }

    info!("Created NPC: {} ({})", name, slug);
    (StatusCode::CREATED, ApiResponse::success(npc))
}

async fn update_npc(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
    Json(npc): Json<serde_json::Value>,
) -> impl IntoResponse {
    let npcs_path = state.content_path.join("npcs");

    let old_path = match find_file_by_id(&npcs_path, "npc", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<serde_json::Value>::error("NPC not found")),
    };

    let name = match npc.get("name") {
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("NPC must have a 'name' field")),
    };

    let slug = slugify(&name);
    let new_path = npcs_path.join(format!("{}.npc.json", slug));

    if new_path != old_path && new_path.exists() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("NPC with name '{}' already exists", name)));
    }

    let content = match serde_json::to_string_pretty(&npc) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to serialize NPC: {}", e))),
    };

    if new_path != old_path {
        if let Err(e) = std::fs::remove_file(&old_path) {
            warn!("Failed to remove old NPC file: {}", e);
        }
    }

    if let Err(e) = std::fs::write(&new_path, &content) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to write NPC: {}", e)));
    }

    info!("Updated NPC: {} ({})", name, slug);
    (StatusCode::OK, ApiResponse::success(npc))
}

async fn delete_npc(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let npcs_path = state.content_path.join("npcs");

    let npc_path = match find_file_by_id(&npcs_path, "npc", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<()>::error("NPC not found")),
    };

    if let Err(e) = std::fs::remove_file(&npc_path) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to delete NPC: {}", e)));
    }

    info!("Deleted NPC: {} (id: {})", npc_path.display(), id);
    (StatusCode::OK, ApiResponse::success(()))
}

// =============================================================================
// Quests API
// =============================================================================

async fn list_quests(
    State(state): State<EditorApiState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let quests_path = state.content_path.join("quests");

    let mut items = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&quests_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(quest_data) = serde_json::from_str::<serde_json::Value>(&content) {
                        items.push(quest_data);
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

async fn get_quest(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let quests_path = state.content_path.join("quests");

    let quest_path = match find_file_by_id(&quests_path, "quest", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<serde_json::Value>::error("Quest not found")),
    };

    match std::fs::read_to_string(&quest_path) {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(quest) => (StatusCode::OK, ApiResponse::success(quest)),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to parse quest: {}", e))),
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to read quest: {}", e))),
    }
}

async fn create_quest(
    State(state): State<EditorApiState>,
    Json(quest): Json<serde_json::Value>,
) -> impl IntoResponse {
    let quests_path = state.content_path.join("quests");

    if let Err(e) = std::fs::create_dir_all(&quests_path) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<serde_json::Value>::error(format!("Failed to create quests directory: {}", e)));
    }

    let name = match quest.get("name") {
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("Quest must have a 'name' field")),
    };

    let id = match quest.get("id") {
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("Quest must have an 'id' field")),
    };

    let slug = slugify(&name);
    let quest_path = quests_path.join(format!("{}.quest.json", slug));

    if quest_path.exists() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("Quest with name '{}' already exists", name)));
    }

    if find_file_by_id(&quests_path, "quest", &id).is_some() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("Quest with id '{}' already exists", id)));
    }

    let content = match serde_json::to_string_pretty(&quest) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to serialize quest: {}", e))),
    };

    if let Err(e) = std::fs::write(&quest_path, &content) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to write quest: {}", e)));
    }

    info!("Created quest: {} ({})", name, slug);
    (StatusCode::CREATED, ApiResponse::success(quest))
}

async fn update_quest(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
    Json(quest): Json<serde_json::Value>,
) -> impl IntoResponse {
    let quests_path = state.content_path.join("quests");

    let old_path = match find_file_by_id(&quests_path, "quest", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<serde_json::Value>::error("Quest not found")),
    };

    let name = match quest.get("name") {
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("Quest must have a 'name' field")),
    };

    let slug = slugify(&name);
    let new_path = quests_path.join(format!("{}.quest.json", slug));

    if new_path != old_path && new_path.exists() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("Quest with name '{}' already exists", name)));
    }

    let content = match serde_json::to_string_pretty(&quest) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to serialize quest: {}", e))),
    };

    if new_path != old_path {
        if let Err(e) = std::fs::remove_file(&old_path) {
            warn!("Failed to remove old quest file: {}", e);
        }
    }

    if let Err(e) = std::fs::write(&new_path, &content) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to write quest: {}", e)));
    }

    info!("Updated quest: {} ({})", name, slug);
    (StatusCode::OK, ApiResponse::success(quest))
}

async fn delete_quest(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let quests_path = state.content_path.join("quests");

    let quest_path = match find_file_by_id(&quests_path, "quest", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<()>::error("Quest not found")),
    };

    if let Err(e) = std::fs::remove_file(&quest_path) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to delete quest: {}", e)));
    }

    info!("Deleted quest: {} (id: {})", quest_path.display(), id);
    (StatusCode::OK, ApiResponse::success(()))
}

// =============================================================================
// Abilities API
// =============================================================================

async fn list_abilities(
    State(state): State<EditorApiState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let abilities_path = state.content_path.join("abilities");

    let mut items = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&abilities_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(ability_data) = serde_json::from_str::<serde_json::Value>(&content) {
                        items.push(ability_data);
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

async fn get_ability(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let abilities_path = state.content_path.join("abilities");

    let ability_path = match find_file_by_id(&abilities_path, "ability", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<serde_json::Value>::error("Ability not found")),
    };

    match std::fs::read_to_string(&ability_path) {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(ability) => (StatusCode::OK, ApiResponse::success(ability)),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to parse ability: {}", e))),
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to read ability: {}", e))),
    }
}

async fn create_ability(
    State(state): State<EditorApiState>,
    Json(ability): Json<serde_json::Value>,
) -> impl IntoResponse {
    let abilities_path = state.content_path.join("abilities");

    if let Err(e) = std::fs::create_dir_all(&abilities_path) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<serde_json::Value>::error(format!("Failed to create abilities directory: {}", e)));
    }

    let name = match ability.get("name") {
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("Ability must have a 'name' field")),
    };

    let id = match ability.get("id") {
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("Ability must have an 'id' field")),
    };

    let slug = slugify(&name);
    let ability_path = abilities_path.join(format!("{}.ability.json", slug));

    if ability_path.exists() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("Ability with name '{}' already exists", name)));
    }

    if find_file_by_id(&abilities_path, "ability", &id).is_some() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("Ability with id '{}' already exists", id)));
    }

    let content = match serde_json::to_string_pretty(&ability) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to serialize ability: {}", e))),
    };

    if let Err(e) = std::fs::write(&ability_path, &content) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to write ability: {}", e)));
    }

    info!("Created ability: {} ({})", name, slug);
    (StatusCode::CREATED, ApiResponse::success(ability))
}

async fn update_ability(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
    Json(ability): Json<serde_json::Value>,
) -> impl IntoResponse {
    let abilities_path = state.content_path.join("abilities");

    let old_path = match find_file_by_id(&abilities_path, "ability", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<serde_json::Value>::error("Ability not found")),
    };

    let name = match ability.get("name") {
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("Ability must have a 'name' field")),
    };

    let slug = slugify(&name);
    let new_path = abilities_path.join(format!("{}.ability.json", slug));

    if new_path != old_path && new_path.exists() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("Ability with name '{}' already exists", name)));
    }

    let content = match serde_json::to_string_pretty(&ability) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to serialize ability: {}", e))),
    };

    if new_path != old_path {
        if let Err(e) = std::fs::remove_file(&old_path) {
            warn!("Failed to remove old ability file: {}", e);
        }
    }

    if let Err(e) = std::fs::write(&new_path, &content) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to write ability: {}", e)));
    }

    info!("Updated ability: {} ({})", name, slug);
    (StatusCode::OK, ApiResponse::success(ability))
}

async fn delete_ability(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let abilities_path = state.content_path.join("abilities");

    let ability_path = match find_file_by_id(&abilities_path, "ability", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<()>::error("Ability not found")),
    };

    if let Err(e) = std::fs::remove_file(&ability_path) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to delete ability: {}", e)));
    }

    info!("Deleted ability: {} (id: {})", ability_path.display(), id);
    (StatusCode::OK, ApiResponse::success(()))
}

// =============================================================================
// Loot Tables API
// =============================================================================

async fn list_loot_tables(
    State(state): State<EditorApiState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let loot_path = state.content_path.join("loot");

    let mut items = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&loot_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(loot_data) = serde_json::from_str::<serde_json::Value>(&content) {
                        items.push(loot_data);
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

async fn get_loot_table(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let loot_path = state.content_path.join("loot");

    let loot_table_path = match find_file_by_id(&loot_path, "loot", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<serde_json::Value>::error("Loot table not found")),
    };

    match std::fs::read_to_string(&loot_table_path) {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(loot) => (StatusCode::OK, ApiResponse::success(loot)),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to parse loot table: {}", e))),
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to read loot table: {}", e))),
    }
}

async fn create_loot_table(
    State(state): State<EditorApiState>,
    Json(loot): Json<serde_json::Value>,
) -> impl IntoResponse {
    let loot_path = state.content_path.join("loot");

    if let Err(e) = std::fs::create_dir_all(&loot_path) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::<serde_json::Value>::error(format!("Failed to create loot directory: {}", e)));
    }

    let name = match loot.get("name") {
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("Loot table must have a 'name' field")),
    };

    let id = match loot.get("id") {
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("Loot table must have an 'id' field")),
    };

    let slug = slugify(&name);
    let loot_table_path = loot_path.join(format!("{}.loot.json", slug));

    if loot_table_path.exists() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("Loot table with name '{}' already exists", name)));
    }

    if find_file_by_id(&loot_path, "loot", &id).is_some() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("Loot table with id '{}' already exists", id)));
    }

    let content = match serde_json::to_string_pretty(&loot) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to serialize loot table: {}", e))),
    };

    if let Err(e) = std::fs::write(&loot_table_path, &content) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to write loot table: {}", e)));
    }

    info!("Created loot table: {} ({})", name, slug);
    (StatusCode::CREATED, ApiResponse::success(loot))
}

async fn update_loot_table(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
    Json(loot): Json<serde_json::Value>,
) -> impl IntoResponse {
    let loot_path = state.content_path.join("loot");

    let old_path = match find_file_by_id(&loot_path, "loot", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<serde_json::Value>::error("Loot table not found")),
    };

    let name = match loot.get("name") {
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => return (StatusCode::BAD_REQUEST, ApiResponse::error("Loot table must have a 'name' field")),
    };

    let slug = slugify(&name);
    let new_path = loot_path.join(format!("{}.loot.json", slug));

    if new_path != old_path && new_path.exists() {
        return (StatusCode::CONFLICT, ApiResponse::error(format!("Loot table with name '{}' already exists", name)));
    }

    let content = match serde_json::to_string_pretty(&loot) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to serialize loot table: {}", e))),
    };

    if new_path != old_path {
        if let Err(e) = std::fs::remove_file(&old_path) {
            warn!("Failed to remove old loot table file: {}", e);
        }
    }

    if let Err(e) = std::fs::write(&new_path, &content) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to write loot table: {}", e)));
    }

    info!("Updated loot table: {} ({})", name, slug);
    (StatusCode::OK, ApiResponse::success(loot))
}

async fn delete_loot_table(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let loot_path = state.content_path.join("loot");

    let loot_table_path = match find_file_by_id(&loot_path, "loot", &id) {
        Some(path) => path,
        None => return (StatusCode::NOT_FOUND, ApiResponse::<()>::error("Loot table not found")),
    };

    if let Err(e) = std::fs::remove_file(&loot_table_path) {
        return (StatusCode::INTERNAL_SERVER_ERROR, ApiResponse::error(format!("Failed to delete loot table: {}", e)));
    }

    info!("Deleted loot table: {} (id: {})", loot_table_path.display(), id);
    (StatusCode::OK, ApiResponse::success(()))
}

// =============================================================================
// Assets API (Stub)
// =============================================================================

#[derive(Debug, Serialize)]
pub struct AssetInfo {
    pub path: String,
    pub name: String,
    pub asset_type: String,
    pub size_bytes: u64,
}

async fn list_assets(
    State(state): State<EditorApiState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let sprites_path = state.assets_path.join("sprites");

    let mut assets = Vec::new();

    fn scan_dir(path: &PathBuf, base_path: &PathBuf, assets: &mut Vec<AssetInfo>) {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    scan_dir(&entry_path, base_path, assets);
                } else {
                    let ext = entry_path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    let asset_type = match ext {
                        "png" | "jpg" | "jpeg" | "webp" => "Sprite",
                        "wav" | "ogg" | "mp3" => "Audio",
                        "json" => "Data",
                        _ => continue,
                    };

                    let relative_path = entry_path.strip_prefix(base_path)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    let name = entry_path.file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    assets.push(AssetInfo {
                        path: relative_path,
                        name,
                        asset_type: asset_type.to_string(),
                        size_bytes: size,
                    });
                }
            }
        }
    }

    scan_dir(&sprites_path, &state.assets_path, &mut assets);

    let total = assets.len();
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(50);

    ApiResponse::success(ListResponse {
        items: assets,
        total,
        page,
        per_page,
    })
}

async fn get_asset(Path(path): Path<String>) -> impl IntoResponse {
    ApiResponse::<serde_json::Value>::error("Not implemented yet - use direct file access")
}

async fn upload_asset(/* TODO: multipart upload */) -> impl IntoResponse {
    ApiResponse::<AssetInfo>::error("Not implemented yet")
}

async fn delete_asset(Path(path): Path<String>) -> impl IntoResponse {
    ApiResponse::<()>::error("Not implemented yet")
}
