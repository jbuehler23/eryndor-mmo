//! Editor API - HTTP endpoints for the game content editor.
//!
//! Provides CRUD operations for zones, items, enemies, NPCs, quests, abilities,
//! loot tables, and assets.
//!
//! ## Module Structure
//! - `crud` - Generic CRUD handlers for content types with id/name-based file storage

mod crud;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

// =============================================================================
// Shared Types
// =============================================================================

/// Shared state for editor API
#[derive(Clone)]
pub struct EditorApiState {
    pub assets_path: PathBuf,
    pub content_path: PathBuf,
}

impl EditorApiState {
    pub fn new() -> Self {
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

// =============================================================================
// Router
// =============================================================================

/// Create the editor API router
pub fn create_editor_router() -> Router {
    let state = EditorApiState::new();

    Router::new()
        // Health check
        .route("/health", get(health_check))
        // Zones (special handling for tilemap)
        .route("/zones", get(list_zones))
        .route("/zones", post(create_zone))
        .route("/zones/:id", get(get_zone))
        .route("/zones/:id", put(update_zone))
        .route("/zones/:id", delete(delete_zone))
        .route("/zones/:id/tilemap", get(get_zone_tilemap))
        .route("/zones/:id/tilemap", put(update_zone_tilemap))
        // Items (generic CRUD)
        .route("/items", get(crud::list_items))
        .route("/items", post(crud::create_item))
        .route("/items/:id", get(crud::get_item))
        .route("/items/:id", put(crud::update_item))
        .route("/items/:id", delete(crud::delete_item))
        // Enemies (generic CRUD)
        .route("/enemies", get(crud::list_enemies))
        .route("/enemies", post(crud::create_enemy))
        .route("/enemies/:id", get(crud::get_enemy))
        .route("/enemies/:id", put(crud::update_enemy))
        .route("/enemies/:id", delete(crud::delete_enemy))
        // NPCs (generic CRUD)
        .route("/npcs", get(crud::list_npcs))
        .route("/npcs", post(crud::create_npc))
        .route("/npcs/:id", get(crud::get_npc))
        .route("/npcs/:id", put(crud::update_npc))
        .route("/npcs/:id", delete(crud::delete_npc))
        // Quests (generic CRUD)
        .route("/quests", get(crud::list_quests))
        .route("/quests", post(crud::create_quest))
        .route("/quests/:id", get(crud::get_quest))
        .route("/quests/:id", put(crud::update_quest))
        .route("/quests/:id", delete(crud::delete_quest))
        // Abilities (generic CRUD)
        .route("/abilities", get(crud::list_abilities))
        .route("/abilities", post(crud::create_ability))
        .route("/abilities/:id", get(crud::get_ability))
        .route("/abilities/:id", put(crud::update_ability))
        .route("/abilities/:id", delete(crud::delete_ability))
        // Loot Tables (generic CRUD)
        .route("/loot-tables", get(crud::list_loot_tables))
        .route("/loot-tables", post(crud::create_loot_table))
        .route("/loot-tables/:id", get(crud::get_loot_table))
        .route("/loot-tables/:id", put(crud::update_loot_table))
        .route("/loot-tables/:id", delete(crud::delete_loot_table))
        // Assets
        .route("/assets", get(list_assets))
        .route("/assets/upload", post(upload_asset))
        .route("/assets/:path", get(get_asset))
        .route("/assets/:path", delete(delete_asset))
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
// Zone Handlers (special case - has tilemap sub-resource)
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
                        let id = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .replace(".zone", "")
                            .to_string();
                        let name = zone
                            .get("name")
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
    let zone_path = state
        .content_path
        .join("zones")
        .join(format!("{}.zone.json", id));

    match std::fs::read_to_string(&zone_path) {
        Ok(content) => match serde_json::from_str::<ZoneData>(&content) {
            Ok(zone) => ApiResponse::success(zone),
            Err(e) => ApiResponse::error(format!("Failed to parse zone: {}", e)),
        },
        Err(e) => ApiResponse::error(format!("Zone not found: {}", e)),
    }
}

async fn create_zone(
    State(state): State<EditorApiState>,
    Json(zone): Json<ZoneData>,
) -> impl IntoResponse {
    let zone_path = state
        .content_path
        .join("zones")
        .join(format!("{}.zone.json", zone.id));

    if zone_path.exists() {
        return (
            StatusCode::CONFLICT,
            ApiResponse::<ZoneData>::error("Zone already exists"),
        );
    }

    match serde_json::to_string_pretty(&zone) {
        Ok(content) => {
            if let Err(e) = std::fs::write(&zone_path, content) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::<ZoneData>::error(format!("Failed to write zone: {}", e)),
                );
            }
            info!("Created zone: {}", zone.id);
            (StatusCode::CREATED, ApiResponse::success(zone))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<ZoneData>::error(format!("Failed to serialize zone: {}", e)),
        ),
    }
}

async fn update_zone(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
    Json(zone): Json<ZoneData>,
) -> impl IntoResponse {
    let zone_path = state
        .content_path
        .join("zones")
        .join(format!("{}.zone.json", id));

    if !zone_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            ApiResponse::<ZoneData>::error("Zone not found"),
        );
    }

    match serde_json::to_string_pretty(&zone) {
        Ok(content) => {
            if let Err(e) = std::fs::write(&zone_path, content) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::<ZoneData>::error(format!("Failed to write zone: {}", e)),
                );
            }
            info!("Updated zone: {}", id);
            (StatusCode::OK, ApiResponse::success(zone))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<ZoneData>::error(format!("Failed to serialize zone: {}", e)),
        ),
    }
}

async fn delete_zone(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let zone_path = state
        .content_path
        .join("zones")
        .join(format!("{}.zone.json", id));

    if !zone_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            ApiResponse::<()>::error("Zone not found"),
        );
    }

    if let Err(e) = std::fs::remove_file(&zone_path) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("Failed to delete zone: {}", e)),
        );
    }

    info!("Deleted zone: {}", id);
    (StatusCode::OK, ApiResponse::success(()))
}

async fn get_zone_tilemap(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let zone_path = state
        .content_path
        .join("zones")
        .join(format!("{}.zone.json", id));

    match std::fs::read_to_string(&zone_path) {
        Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(zone_data) => {
                let tilemap = zone_data.get("tilemap").cloned().unwrap_or_else(|| {
                    serde_json::json!({
                        "tile_size": 16,
                        "chunk_size": 16,
                        "chunks": {}
                    })
                });
                (StatusCode::OK, ApiResponse::success(tilemap))
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::error(format!("Failed to parse zone: {}", e)),
            ),
        },
        Err(e) => (
            StatusCode::NOT_FOUND,
            ApiResponse::error(format!("Zone not found: {}", e)),
        ),
    }
}

async fn update_zone_tilemap(
    State(state): State<EditorApiState>,
    Path(id): Path<String>,
    Json(tilemap): Json<serde_json::Value>,
) -> impl IntoResponse {
    let zone_path = state
        .content_path
        .join("zones")
        .join(format!("{}.zone.json", id));

    // Read existing zone data
    let zone_content = match std::fs::read_to_string(&zone_path) {
        Ok(content) => content,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                ApiResponse::<serde_json::Value>::error(format!("Zone not found: {}", e)),
            )
        }
    };

    // Parse zone data
    let mut zone_data: serde_json::Value = match serde_json::from_str(&zone_content) {
        Ok(data) => data,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiResponse::error(format!("Failed to parse zone: {}", e)),
            )
        }
    };

    // Update tilemap field
    if let Some(obj) = zone_data.as_object_mut() {
        obj.insert("tilemap".to_string(), tilemap.clone());
    } else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error("Zone data is not an object"),
        );
    }

    // Write back to file
    match serde_json::to_string_pretty(&zone_data) {
        Ok(content) => {
            if let Err(e) = std::fs::write(&zone_path, content) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiResponse::error(format!("Failed to write zone: {}", e)),
                );
            }
            info!("Updated tilemap for zone: {}", id);
            (StatusCode::OK, ApiResponse::success(tilemap))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(format!("Failed to serialize zone: {}", e)),
        ),
    }
}

// =============================================================================
// Asset Handlers (stubs - different pattern from content CRUD)
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

                    let relative_path = entry_path
                        .strip_prefix(base_path)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    let name = entry_path
                        .file_name()
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

async fn get_asset(Path(_path): Path<String>) -> impl IntoResponse {
    ApiResponse::<serde_json::Value>::error("Not implemented yet - use direct file access")
}

async fn upload_asset() -> impl IntoResponse {
    ApiResponse::<AssetInfo>::error("Not implemented yet")
}

async fn delete_asset(Path(_path): Path<String>) -> impl IntoResponse {
    ApiResponse::<()>::error("Not implemented yet")
}
