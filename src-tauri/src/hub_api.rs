use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::{
    Router,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State, ws::{WebSocket, WebSocketUpgrade, Message as WsMessage}},
    http::{StatusCode, header, HeaderMap},
    response::IntoResponse,
    routing::{get, patch, post},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

use crate::storage_manager::StorageManager;
use crate::tunnel_manager::TunnelManager;
use crate::auth;

// --- Rate limiter ---

#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
    max_tokens: u32,
    refill_per_sec: f64,
}

impl RateLimiter {
    pub fn new(max_tokens: u32, refill_per_sec: f64) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            max_tokens,
            refill_per_sec,
        }
    }

    fn check(&self, ip: &str) -> bool {
        let mut buckets = self.buckets.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        let entry = buckets.entry(ip.to_string()).or_insert((self.max_tokens, now));

        let elapsed = now.duration_since(entry.1).as_secs_f64();
        let refill = (elapsed * self.refill_per_sec) as u32;
        if refill > 0 {
            entry.0 = (entry.0 + refill).min(self.max_tokens);
            entry.1 = now;
        }

        if entry.0 > 0 {
            entry.0 -= 1;
            true
        } else {
            false
        }
    }
}

/// Extract real client IP from Cloudflare headers, falling back to "direct"
fn get_client_ip(headers: &HeaderMap) -> String {
    headers.get("cf-connecting-ip")
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("direct")
        .split(',').next().unwrap_or("direct")
        .trim()
        .to_string()
}

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub conversation_id: String,
    pub message: Value,
}

#[derive(Deserialize)]
pub struct CreateConversationRequest {
    pub kind: String,
    pub peer_user_id: Option<String>,
    pub name: Option<String>,
    pub member_ids: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct UpdateConversationRequest {
    pub name: Option<String>,
    pub add_members: Option<Vec<String>>,
    pub remove_members: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct UpdateFileRequest {
    pub is_public: bool,
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub body: String,
    #[serde(default)]
    pub attachment_ids: Vec<String>,
}

#[derive(Deserialize)]
pub struct MessagesQuery {
    pub limit: Option<u32>,
    pub before: Option<String>,
}

#[derive(Deserialize)]
pub struct WsQuery {
    pub token: String,
}

#[derive(Clone)]
pub struct ApiState {
    pub storage_manager: Arc<Mutex<Option<StorageManager>>>,
    pub tunnel_manager: Arc<Mutex<Option<TunnelManager>>>,
    pub started_at: Instant,
    pub msg_tx: broadcast::Sender<BroadcastMessage>,
    pub auth_limiter: RateLimiter,
}

pub const HUB_API_PORT: u16 = 9090;

pub async fn start_hub_api(state: ApiState, port: u16) -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/info", get(hub_info))
        .route("/api/status", get(hub_status))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/members", get(list_members))
        .route("/api/conversations", get(list_conversations_handler).post(create_conversation))
        .route("/api/conversations/{id}", patch(update_conversation))
        .route("/api/conversations/{id}/messages", get(get_messages).post(send_message))
        .route("/ws", get(ws_handler))
        .route("/api/files", get(list_files).post(upload_file))
        .route("/api/files/{name}", get(download_file).delete(delete_file_handler).patch(update_file_visibility_handler))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 100 MB upload limit
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

// GET /api/members
async fn list_members(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    let _claims = validate_auth_header(&headers)?;

    let sm_lock = state.storage_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sm = sm_lock.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let users = sm.list_users().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let members: Vec<Value> = users.iter().map(|u| {
        json!({
            "user_id": u.user_id,
            "username": u.username,
            "is_admin": u.is_admin,
            "created_at": u.created_at,
        })
    }).collect();

    Ok(Json(json!({ "members": members, "total": members.len() })))
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
    let disposition = if content_type.starts_with("image/") || content_type.starts_with("video/") {
        format!("inline; filename=\"{}\"", name)
    } else {
        format!("attachment; filename=\"{}\"", name)
    };

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

// PATCH /api/files/:name
async fn update_file_visibility_handler(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(body): Json<UpdateFileRequest>,
) -> Result<StatusCode, StatusCode> {
    let claims = validate_auth_header(&headers)?;
    let sm_lock = state.storage_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sm = sm_lock.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    sm.update_file_visibility(&claims.sub, &name, body.is_public)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(StatusCode::OK)
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
        Some("webp") => "image/webp".to_string(),
        Some("svg") => "image/svg+xml".to_string(),
        Some("bmp") => "image/bmp".to_string(),
        Some("mp4") | Some("m4v") => "video/mp4".to_string(),
        Some("webm") => "video/webm".to_string(),
        Some("mov") => "video/quicktime".to_string(),
        Some("avi") => "video/x-msvideo".to_string(),
        Some("mkv") => "video/x-matroska".to_string(),
        Some("ogv") => "video/ogg".to_string(),
        Some("3gp") => "video/3gpp".to_string(),
        Some("pdf") => "application/pdf".to_string(),
        Some("txt") => "text/plain".to_string(),
        Some("html") | Some("htm") => "text/html".to_string(),
        Some("json") => "application/json".to_string(),
        Some("zip") => "application/zip".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

// POST /api/conversations
async fn create_conversation(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(req): Json<CreateConversationRequest>,
) -> Result<Json<Value>, StatusCode> {
    let claims = validate_auth_header(&headers)?;

    let sm_lock = state.storage_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sm = sm_lock.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match req.kind.as_str() {
        "dm" => {
            let peer_id = req.peer_user_id.as_deref().ok_or(StatusCode::BAD_REQUEST)?;
            let conv = sm.create_dm_conversation(&claims.sub, peer_id)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let members = sm.get_conversation_members(&conv.conversation_id)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(json!({
                "conversation_id": conv.conversation_id,
                "kind": conv.kind,
                "name": conv.name,
                "members": members,
                "created_at": conv.created_at,
            })))
        }
        "group" => {
            let name = req.name.as_deref().ok_or(StatusCode::BAD_REQUEST)?;
            let member_ids = req.member_ids.as_ref().ok_or(StatusCode::BAD_REQUEST)?;
            let conv = sm.create_group_conversation(&claims.sub, name, member_ids)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let members = sm.get_conversation_members(&conv.conversation_id)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(json!({
                "conversation_id": conv.conversation_id,
                "kind": conv.kind,
                "name": conv.name,
                "members": members,
                "created_at": conv.created_at,
            })))
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

// GET /api/conversations
async fn list_conversations_handler(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    let claims = validate_auth_header(&headers)?;

    let sm_lock = state.storage_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sm = sm_lock.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let convs = sm.list_conversations(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({ "conversations": convs })))
}

// PATCH /api/conversations/:id
async fn update_conversation(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(conversation_id): Path<String>,
    Json(req): Json<UpdateConversationRequest>,
) -> Result<Json<Value>, StatusCode> {
    let claims = validate_auth_header(&headers)?;

    let sm_lock = state.storage_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sm = sm_lock.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Verify caller is a member
    let is_member = sm.is_conversation_member(&conversation_id, &claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !is_member {
        return Err(StatusCode::FORBIDDEN);
    }

    if let Some(name) = &req.name {
        sm.rename_conversation(&conversation_id, name)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    if let Some(add) = &req.add_members {
        for uid in add {
            sm.add_group_member(&conversation_id, uid)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }
    if let Some(remove) = &req.remove_members {
        for uid in remove {
            sm.remove_group_member(&conversation_id, uid)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    let members = sm.get_conversation_members(&conversation_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({ "ok": true, "members": members })))
}

// POST /api/conversations/:id/messages
async fn send_message(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(conversation_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<Value>, StatusCode> {
    let claims = validate_auth_header(&headers)?;

    if req.body.is_empty() && req.attachment_ids.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let message = {
        let sm_lock = state.storage_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let sm = sm_lock.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

        sm.create_message(&conversation_id, &claims.sub, &req.body, &req.attachment_ids)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let msg_json = json!({
        "message_id": message.message_id,
        "conversation_id": message.conversation_id,
        "sender_id": message.sender_id,
        "sender_username": message.sender_username,
        "body": message.body,
        "attachments": message.attachments,
        "created_at": message.created_at,
    });

    // Broadcast to WebSocket subscribers
    let _ = state.msg_tx.send(BroadcastMessage {
        conversation_id: conversation_id.clone(),
        message: msg_json.clone(),
    });

    Ok(Json(msg_json))
}

// GET /api/conversations/:id/messages
async fn get_messages(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(conversation_id): Path<String>,
    Query(query): Query<MessagesQuery>,
) -> Result<Json<Value>, StatusCode> {
    let claims = validate_auth_header(&headers)?;

    let sm_lock = state.storage_manager.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sm = sm_lock.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let is_member = sm.is_conversation_member(&conversation_id, &claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !is_member {
        return Err(StatusCode::FORBIDDEN);
    }

    let limit = query.limit.unwrap_or(50).min(100);
    let messages = sm.list_messages(&conversation_id, limit, query.before.as_deref())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({ "messages": messages })))
}

// GET /ws?token=JWT
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ApiState>,
    Query(query): Query<WsQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let claims = auth::validate_token(&query.token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(ws.on_upgrade(move |socket| handle_ws(socket, state, claims)))
}

async fn handle_ws(
    mut socket: WebSocket,
    state: ApiState,
    claims: auth::Claims,
) {
    let mut rx = state.msg_tx.subscribe();
    let user_id = claims.sub.clone();

    // Load user's conversation IDs for filtering
    let conversation_ids: Vec<String> = {
        let sm_lock = state.storage_manager.lock().ok();
        match sm_lock.as_ref().and_then(|l| l.as_ref()) {
            Some(sm) => sm.list_conversations(&user_id)
                .map(|convs| convs.into_iter().map(|c| c.conversation.conversation_id).collect())
                .unwrap_or_default(),
            None => vec![],
        }
    };

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(broadcast_msg) => {
                        if conversation_ids.contains(&broadcast_msg.conversation_id) {
                            let payload = serde_json::to_string(&broadcast_msg).unwrap_or_default();
                            if socket.send(WsMessage::Text(payload.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
            ws_msg = socket.recv() => {
                match ws_msg {
                    Some(Ok(_)) => {} // Ignore client messages; sends go through REST
                    _ => break,
                }
            }
        }
    }
}

// POST /api/auth/register
async fn register(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Rate limit
    let ip = get_client_ip(&headers);
    if !state.auth_limiter.check(&ip) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

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
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Rate limit
    let ip = get_client_ip(&headers);
    if !state.auth_limiter.check(&ip) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

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
