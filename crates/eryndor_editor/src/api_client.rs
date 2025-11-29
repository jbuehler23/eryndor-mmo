//! API Client for Editor
//! Handles HTTP communication with the game server for CRUD operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// API Client configuration
#[derive(Clone)]
pub struct ApiConfig {
    pub base_url: String,
    pub auth_token: Option<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3000/api/editor".to_string(),
            auth_token: None,
        }
    }
}

/// Generic API response wrapper
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

/// API response that ignores the data field (for PUT/DELETE operations where we don't need the response data)
#[derive(Debug, Deserialize)]
pub struct ApiResponseAny {
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
    // Intentionally ignores `data` field - it can be any type
}

/// Paginated list response
#[derive(Debug, Deserialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}

/// Content type for API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentType {
    Zones,
    Items,
    Enemies,
    Npcs,
    Quests,
    Abilities,
    LootTables,
    Assets,
}

impl ContentType {
    pub fn endpoint(&self) -> &'static str {
        match self {
            ContentType::Zones => "zones",
            ContentType::Items => "items",
            ContentType::Enemies => "enemies",
            ContentType::Npcs => "npcs",
            ContentType::Quests => "quests",
            ContentType::Abilities => "abilities",
            ContentType::LootTables => "loot-tables",
            ContentType::Assets => "assets",
        }
    }
}

/// API Client for editor operations
pub struct ApiClient {
    config: ApiConfig,
    #[cfg(target_family = "wasm")]
    _phantom: std::marker::PhantomData<()>,
}

impl ApiClient {
    pub fn new(config: ApiConfig) -> Self {
        Self {
            config,
            #[cfg(target_family = "wasm")]
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the full URL for an endpoint
    fn url(&self, content_type: ContentType, id: Option<&str>) -> String {
        match id {
            Some(id) => format!("{}/{}/{}", self.config.base_url, content_type.endpoint(), id),
            None => format!("{}/{}", self.config.base_url, content_type.endpoint()),
        }
    }

    /// Set authentication token
    pub fn set_auth_token(&mut self, token: String) {
        self.config.auth_token = Some(token);
    }

    /// Clear authentication token
    pub fn clear_auth_token(&mut self) {
        self.config.auth_token = None;
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.config.auth_token.is_some()
    }
}

// WASM implementation using web_sys fetch
#[cfg(target_family = "wasm")]
mod wasm_impl {
    use super::*;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response, Headers};

    impl ApiClient {
        /// Perform a GET request
        pub async fn get<T: for<'de> Deserialize<'de>>(
            &self,
            content_type: ContentType,
            id: Option<&str>,
        ) -> Result<ApiResponse<T>, String> {
            let url = self.url(content_type, id);
            self.fetch::<(), T>("GET", &url, None).await
        }

        /// Perform a POST request (create)
        pub async fn create<T: Serialize, R: for<'de> Deserialize<'de>>(
            &self,
            content_type: ContentType,
            data: &T,
        ) -> Result<ApiResponse<R>, String> {
            let url = self.url(content_type, None);
            self.fetch("POST", &url, Some(data)).await
        }

        /// Perform a PUT request (update)
        pub async fn update<T: Serialize, R: for<'de> Deserialize<'de>>(
            &self,
            content_type: ContentType,
            id: &str,
            data: &T,
        ) -> Result<ApiResponse<R>, String> {
            let url = self.url(content_type, Some(id));
            self.fetch("PUT", &url, Some(data)).await
        }

        /// Perform a DELETE request
        pub async fn delete(
            &self,
            content_type: ContentType,
            id: &str,
        ) -> Result<ApiResponse<()>, String> {
            let url = self.url(content_type, Some(id));
            self.fetch::<(), ()>("DELETE", &url, None).await
        }

        /// List items with optional filters
        pub async fn list<T: for<'de> Deserialize<'de>>(
            &self,
            content_type: ContentType,
            filters: Option<&HashMap<String, String>>,
        ) -> Result<ApiResponse<ListResponse<T>>, String> {
            let mut url = self.url(content_type, None);

            if let Some(filters) = filters {
                let query: Vec<String> = filters
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                if !query.is_empty() {
                    url = format!("{}?{}", url, query.join("&"));
                }
            }

            self.fetch::<(), ListResponse<T>>("GET", &url, None).await
        }

        /// PUT JSON to custom URL (returns ApiResponseAny to handle any response data type)
        pub async fn put_json<T: Serialize>(
            &self,
            url: &str,
            data: &T,
        ) -> Result<ApiResponseAny, String> {
            let window = web_sys::window().ok_or("No window object")?;

            let mut opts = RequestInit::new();
            opts.method("PUT");
            opts.mode(RequestMode::Cors);

            let headers = Headers::new().map_err(|e| format!("Failed to create headers: {:?}", e))?;
            headers.set("Content-Type", "application/json").map_err(|e| format!("{:?}", e))?;

            if let Some(token) = &self.config.auth_token {
                headers.set("Authorization", &format!("Bearer {}", token))
                    .map_err(|e| format!("{:?}", e))?;
            }

            opts.headers(&headers);

            let json_body = serde_json::to_string(data)
                .map_err(|e| format!("Failed to serialize: {}", e))?;
            opts.body(Some(&JsValue::from_str(&json_body)));

            let request = Request::new_with_str_and_init(url, &opts)
                .map_err(|e| format!("Failed to create request: {:?}", e))?;

            let resp_value = JsFuture::from(window.fetch_with_request(&request))
                .await
                .map_err(|e| format!("Fetch failed: {:?}", e))?;

            let resp: Response = resp_value.dyn_into()
                .map_err(|_| "Response is not a Response object")?;

            let json = JsFuture::from(resp.json().map_err(|e| format!("{:?}", e))?)
                .await
                .map_err(|e| format!("Failed to get JSON: {:?}", e))?;

            // Use ApiResponseAny which ignores the data field
            let result: ApiResponseAny = serde_wasm_bindgen::from_value(json)
                .map_err(|e| format!("Failed to deserialize: {:?}", e))?;

            Ok(result)
        }

        /// Internal fetch implementation
        async fn fetch<T: Serialize, R: for<'de> Deserialize<'de>>(
            &self,
            method: &str,
            url: &str,
            body: Option<&T>,
        ) -> Result<ApiResponse<R>, String> {
            let window = web_sys::window().ok_or("No window object")?;

            let mut opts = RequestInit::new();
            opts.method(method);
            opts.mode(RequestMode::Cors);

            let headers = Headers::new().map_err(|e| format!("Failed to create headers: {:?}", e))?;
            headers.set("Content-Type", "application/json").map_err(|e| format!("{:?}", e))?;

            if let Some(token) = &self.config.auth_token {
                headers.set("Authorization", &format!("Bearer {}", token))
                    .map_err(|e| format!("{:?}", e))?;
            }

            opts.headers(&headers);

            if let Some(data) = body {
                let json = serde_json::to_string(data)
                    .map_err(|e| format!("Failed to serialize: {}", e))?;
                opts.body(Some(&JsValue::from_str(&json)));
            }

            let request = Request::new_with_str_and_init(url, &opts)
                .map_err(|e| format!("Failed to create request: {:?}", e))?;

            let resp_value = JsFuture::from(window.fetch_with_request(&request))
                .await
                .map_err(|e| format!("Fetch failed: {:?}", e))?;

            let resp: Response = resp_value.dyn_into()
                .map_err(|_| "Response is not a Response object")?;

            let json = JsFuture::from(resp.json().map_err(|e| format!("{:?}", e))?)
                .await
                .map_err(|e| format!("Failed to get JSON: {:?}", e))?;

            let result: ApiResponse<R> = serde_wasm_bindgen::from_value(json)
                .map_err(|e| format!("Failed to deserialize: {:?}", e))?;

            Ok(result)
        }
    }
}

// Native implementation using reqwest (for desktop testing)
#[cfg(not(target_family = "wasm"))]
mod native_impl {
    use super::*;

    impl ApiClient {
        /// Perform a GET request
        pub async fn get<T: for<'de> Deserialize<'de>>(
            &self,
            content_type: ContentType,
            id: Option<&str>,
        ) -> Result<ApiResponse<T>, String> {
            let url = self.url(content_type, id);
            let client = reqwest::Client::new();

            let mut request = client.get(&url);
            if let Some(token) = &self.config.auth_token {
                request = request.header("Authorization", format!("Bearer {}", token));
            }

            let response = request.send().await
                .map_err(|e| format!("Request failed: {}", e))?;

            response.json().await
                .map_err(|e| format!("Failed to parse response: {}", e))
        }

        /// Perform a POST request (create)
        pub async fn create<T: Serialize, R: for<'de> Deserialize<'de>>(
            &self,
            content_type: ContentType,
            data: &T,
        ) -> Result<ApiResponse<R>, String> {
            let url = self.url(content_type, None);
            let client = reqwest::Client::new();

            let mut request = client.post(&url).json(data);
            if let Some(token) = &self.config.auth_token {
                request = request.header("Authorization", format!("Bearer {}", token));
            }

            let response = request.send().await
                .map_err(|e| format!("Request failed: {}", e))?;

            response.json().await
                .map_err(|e| format!("Failed to parse response: {}", e))
        }

        /// Perform a PUT request (update)
        pub async fn update<T: Serialize, R: for<'de> Deserialize<'de>>(
            &self,
            content_type: ContentType,
            id: &str,
            data: &T,
        ) -> Result<ApiResponse<R>, String> {
            let url = self.url(content_type, Some(id));
            let client = reqwest::Client::new();

            let mut request = client.put(&url).json(data);
            if let Some(token) = &self.config.auth_token {
                request = request.header("Authorization", format!("Bearer {}", token));
            }

            let response = request.send().await
                .map_err(|e| format!("Request failed: {}", e))?;

            response.json().await
                .map_err(|e| format!("Failed to parse response: {}", e))
        }

        /// Perform a DELETE request
        pub async fn delete(
            &self,
            content_type: ContentType,
            id: &str,
        ) -> Result<ApiResponse<()>, String> {
            let url = self.url(content_type, Some(id));
            let client = reqwest::Client::new();

            let mut request = client.delete(&url);
            if let Some(token) = &self.config.auth_token {
                request = request.header("Authorization", format!("Bearer {}", token));
            }

            let response = request.send().await
                .map_err(|e| format!("Request failed: {}", e))?;

            response.json().await
                .map_err(|e| format!("Failed to parse response: {}", e))
        }

        /// List items with optional filters
        pub async fn list<T: for<'de> Deserialize<'de>>(
            &self,
            content_type: ContentType,
            filters: Option<&HashMap<String, String>>,
        ) -> Result<ApiResponse<ListResponse<T>>, String> {
            let mut url = self.url(content_type, None);

            if let Some(filters) = filters {
                let query: Vec<String> = filters
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                if !query.is_empty() {
                    url = format!("{}?{}", url, query.join("&"));
                }
            }

            let client = reqwest::Client::new();

            let mut request = client.get(&url);
            if let Some(token) = &self.config.auth_token {
                request = request.header("Authorization", format!("Bearer {}", token));
            }

            let response = request.send().await
                .map_err(|e| format!("Request failed: {}", e))?;

            response.json().await
                .map_err(|e| format!("Failed to parse response: {}", e))
        }

        /// PUT JSON to custom URL (returns ApiResponseAny to handle any response data type)
        pub async fn put_json<T: Serialize>(
            &self,
            url: &str,
            data: &T,
        ) -> Result<ApiResponseAny, String> {
            let client = reqwest::Client::new();

            let mut request = client.put(url).json(data);
            if let Some(token) = &self.config.auth_token {
                request = request.header("Authorization", format!("Bearer {}", token));
            }

            let response = request.send().await
                .map_err(|e| format!("Request failed: {}", e))?;

            response.json().await
                .map_err(|e| format!("Failed to parse response: {}", e))
        }
    }
}

/// Authentication request
#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Authentication response
#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: String,
    pub user: UserInfo,
}

/// User info from auth
#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub role: String,
}

/// Asset upload request metadata
#[derive(Debug, Serialize)]
pub struct AssetUploadMeta {
    pub name: String,
    pub asset_type: String,
    pub folder: String,
}

/// Schema definition for content types
#[derive(Debug, Deserialize)]
pub struct ContentSchema {
    pub name: String,
    pub fields: Vec<SchemaField>,
}

/// Field definition in schema
#[derive(Debug, Deserialize)]
pub struct SchemaField {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub validation: Option<FieldValidation>,
}

/// Field types for schema
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum FieldType {
    String,
    Integer { min: Option<i64>, max: Option<i64> },
    Float { min: Option<f64>, max: Option<f64> },
    Boolean,
    Enum { values: Vec<String> },
    Reference { content_type: String },
    Array { item_type: Box<FieldType> },
    Object { fields: Vec<SchemaField> },
    Vec2,
    Color,
    Asset { asset_type: String },
}

/// Field validation rules
#[derive(Debug, Deserialize)]
pub struct FieldValidation {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
}
