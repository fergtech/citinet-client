use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::{
    Router,
    extract::{Multipart, Path, State},
    http::{StatusCode, header, HeaderMap},
    response::IntoResponse,
    routing::{get, post},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_http::cors::{Any, CorsLayer};

use crate::storage_manager::StorageManager;
use crate::tunnel_manager::TunnelManager;
use crate::auth;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    pub token: String,
    pub expires_at: String,
}

#[derive(Clone)]
pub struct ApiState {
    pub storage_manager: Arc<Mutex<Option<StorageManager>>>,
    pub tunnel_manager: Arc<Mutex<Option<TunnelManager>>>,
    pub started_at: Instant,
}

pub async fn start_hub_api(state: ApiState, port: u16) -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/info", get(hub_info))
        .route("/api/status", get(hub_status))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/files", get(list_files).post(upload_file))
        .route("/api/files/{name}", get(download_file).delete(delete_file_handler))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    log::info!("Hub API listening on 0.0.0.0:{}", port);
    axum::serve(listener, app).await?;
    Ok(())
}

// GET /api/health
async fn health() -> Json<Value> {
    Json(json!({ "ok": true, "version": "0.1.0" }))
}

// GET /api/info
async fn hub_info(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let sm_lock = state.storage_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sm = sm_lock.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let config = sm.get_node_config()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    Ok(Json(json!({
        "node_id": config.node_id,
        "node_name": config.node_name,
        "node_type": config.node_type,
        "storage_quota_gb": config.disk_quota_gb,
    })))
}

// GET /api/status
async fn hub_status(State(state): State<ApiState>) -> Result<Json<Value>, StatusCode> {
    let sm_lock = state.storage_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sm = sm_lock.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let config = sm.get_node_config()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let storage = sm.get_storage_status()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let uptime = state.started_at.elapsed().as_secs();

    Ok(Json(json!({
        "node_name": config.node_name,
        "node_type": config.node_type,
        "uptime_seconds": uptime,
        "storage": {
            "used_gb": storage.used_gb,
            "quota_gb": storage.quota_gb,
            "file_count": storage.file_count,
        },
        "online": true,
    })))
}

// GET /api/files
async fn list_files(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    // Validate authentication and get user claims
    let claims = validate_auth_header(&headers)?;

    let sm_lock = state.storage_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sm = sm_lock.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let files = sm.list_files(Some(&claims.sub)).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let file_list: Vec<Value> = files.iter().map(|f| {
        json!({
            "file_id": f.file_id,
            "file_name": f.file_name,
            "size_bytes": f.size_bytes,
            "is_public": f.is_public,
            "owner_id": f.user_id,
            "created_at": f.created_at,
        })
    }).collect();

    Ok(Json(json!(file_list)))
}

// POST /api/files (multipart/form-data)
async fn upload_file(
    State(state): State<ApiState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<Value>, StatusCode> {
    // Validate authentication and get user claims
    let claims = validate_auth_header(&headers)?;
    
    let mut file_name = String::new();
    let mut file_data = Vec::new();
    let mut is_public = false;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            file_name = field.file_name().unwrap_or("upload").to_string();
            if let Ok(bytes) = field.bytes().await {
                file_data = bytes.to_vec();
            }
        } else if name == "is_public" {
            if let Ok(text) = field.text().await {
                is_public = text == "true" || text == "1";
            }
        }
    }

    if file_name.is_empty() || file_data.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let sm_lock = state.storage_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sm = sm_lock.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let file = sm.upload_file(&claims.sub, &file_name, &file_data, is_public)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "file_id": file.file_id,
        "file_name": file.file_name,
        "size_bytes": file_data.len(),
    })))
}

// GET /api/files/:name
async fn download_file(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    // Validate authentication and get user claims
    let claims = validate_auth_header(&headers)?;
    let sm_lock = state.storage_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sm = sm_lock.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let data = sm.read_file(&claims.sub, &name).map_err(|_| StatusCode::NOT_FOUND)?;

    let content_type = mime_from_ext(&name);
    let disposition = format!("attachment; filename=\"{}\"", name);

    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        data,
    ))
}

// DELETE /api/files/:name
async fn delete_file_handler(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<StatusCode, StatusCode> {
    // Validate authentication and get user claims
    let claims = validate_auth_header(&headers)?;
    let sm_lock = state.storage_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sm = sm_lock.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    sm.delete_file(&claims.sub, &name).map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(StatusCode::NO_CONTENT)
}

// Helper function to validate JWT from Authorization header
fn validate_auth_header(headers: &HeaderMap) -> Result<auth::Claims, StatusCode> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth::extract_bearer_token(auth_header)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    auth::validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)
}

fn mime_from_ext(name: &str) -> String {
    match name.rsplit('.').next().map(|e| e.to_lowercase()).as_deref() {
        Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
        Some("png") => "image/png".to_string(),
        Some("gif") => "image/gif".to_string(),
        Some("pdf") => "application/pdf".to_string(),
        Some("txt") => "text/plain".to_string(),
        Some("html") | Some("htm") => "text/html".to_string(),
        Some("json") => "application/json".to_string(),
        Some("zip") => "application/zip".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

// POST /api/auth/register
async fn register(
    State(state): State<ApiState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let sm_lock = state.storage_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sm = sm_lock.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Validate input
    if req.username.is_empty() || req.email.is_empty() || req.password.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check if username already exists
    if let Ok(Some(_)) = sm.get_user_by_username(&req.username) {
        return Err(StatusCode::CONFLICT);
    }

    // Hash password
    let password_hash = auth::hash_password(&req.password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // First user is admin by default
    let user_count = sm.list_users()
        .map(|users| users.len())
        .unwrap_or(0);
    let is_admin = user_count == 0;

    // Create user
    let user = sm.create_user(&req.username, &req.email, &password_hash, is_admin)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate JWT token
    let auth_token = auth::generate_token(&user.user_id, &user.username, user.is_admin)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        user_id: user.user_id,
        username: user.username,
        email: user.email,
        is_admin: user.is_admin,
        token: auth_token.token,
        expires_at: auth_token.expires_at,
    }))
}

// POST /api/auth/login
async fn login(
    State(state): State<ApiState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let sm_lock = state.storage_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sm = sm_lock.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Get user by username
    let user = sm.get_user_by_username(&req.username)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Get password hash
    let password_hash = sm.get_password_hash(&req.username)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify password
    let valid = auth::verify_password(&req.password, &password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Generate JWT token
    let auth_token = auth::generate_token(&user.user_id, &user.username, user.is_admin)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        user_id: user.user_id,
        username: user.username,
        email: user.email,
        is_admin: user.is_admin,
        token: auth_token.token,
        expires_at: auth_token.expires_at,
    }))
}
