use std::sync::atomic::Ordering;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};

use crate::{
    auth::require_auth,
    errors::{ApiError, ApiResult},
    models::{
        CreateServerConnectionRequest, ServerAuthType, ServerConnection, ServerConnectionStatus,
        ServerCredential, SuccessResponse, UpdateServerConnectionRequest,
    },
    state::AppState,
    time::now_iso,
};

mod terminal;

pub(crate) use terminal::terminal_ws;

pub(crate) async fn list_servers(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<ServerConnection>>> {
    if let Some(pool) = &state.db {
        return Ok(Json(crate::db::list_servers(pool, None).await?));
    }

    let servers = state
        .servers
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access server state"))?;
    let mut servers = servers.values().cloned().collect::<Vec<_>>();
    servers.sort_by(|left, right| left.id.cmp(&right.id));

    Ok(Json(servers))
}

pub(crate) async fn create_server(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateServerConnectionRequest>,
) -> ApiResult<(StatusCode, Json<ServerConnection>)> {
    let user = require_auth(&state, &headers).await?;

    let credential = credential_from_create_request(&payload)?;
    let timestamp = now_iso();
    let id_number = state.next_server_id.fetch_add(1, Ordering::Relaxed);
    let id = format!("srv_{id_number:03}");
    let server = ServerConnection {
        id: id.clone(),
        name: required_trimmed(&payload.name, "Server name is required")?,
        host: required_trimmed(&payload.host, "Server host is required")?,
        port: payload.port.unwrap_or(22),
        username: required_trimmed(&payload.username, "Server username is required")?,
        auth_type: payload.auth_type,
        status: ServerConnectionStatus::Disconnected,
        last_connected_at: None,
        error: None,
        created_at: timestamp.clone(),
        updated_at: timestamp,
    };

    if let Some(pool) = &state.db {
        crate::db::put_server(pool, &user.id, &server).await?;
        crate::db::put_server_credential(pool, &user.id, &id, &credential, &server.updated_at)
            .await?;
    } else {
        let mut servers = state
            .servers
            .lock()
            .map_err(|_| ApiError::Internal("Failed to access server state"))?;
        servers.insert(id.clone(), server.clone());
    }

    {
        let mut credentials = state
            .server_credentials
            .lock()
            .map_err(|_| ApiError::Internal("Failed to access server credential state"))?;
        credentials.insert(id, credential);
    }

    Ok((StatusCode::CREATED, Json(server)))
}

pub(crate) async fn get_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<ServerConnection>> {
    if let Some(pool) = &state.db {
        let server = crate::db::get_server(pool, &id)
            .await?
            .ok_or(ApiError::NotFound("Server not found"))?;
        return Ok(Json(server));
    }

    let servers = state
        .servers
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access server state"))?;
    let server = servers
        .get(&id)
        .cloned()
        .ok_or(ApiError::NotFound("Server not found"))?;

    Ok(Json(server))
}

pub(crate) async fn update_server(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<UpdateServerConnectionRequest>,
) -> ApiResult<Json<ServerConnection>> {
    let user = require_auth(&state, &headers).await?;

    let mut server = load_server_for_update(&state, &id).await?;

    if let Some(name) = payload.name.as_ref() {
        server.name = required_trimmed(name, "Server name is required")?;
    }
    if let Some(host) = payload.host.as_ref() {
        server.host = required_trimmed(host, "Server host is required")?;
    }
    if let Some(port) = payload.port {
        server.port = port;
    }
    if let Some(username) = payload.username.as_ref() {
        server.username = required_trimmed(username, "Server username is required")?;
    }

    let next_auth_type = payload
        .auth_type
        .clone()
        .unwrap_or_else(|| server.auth_type.clone());
    let credential = credential_from_update_request(&payload, &next_auth_type)?;
    if payload.auth_type.is_some() {
        server.auth_type = next_auth_type;
    }

    server.status = ServerConnectionStatus::Disconnected;
    server.error = None;
    server.updated_at = now_iso();
    if let Some(credential) = credential {
        if let Some(pool) = &state.db {
            crate::db::put_server_credential(pool, &user.id, &id, &credential, &server.updated_at)
                .await?;
        }

        let mut credentials = state
            .server_credentials
            .lock()
            .map_err(|_| ApiError::Internal("Failed to access server credential state"))?;
        credentials.insert(id.clone(), credential);
    }

    if let Some(pool) = &state.db {
        crate::db::put_server(pool, &user.id, &server).await?;
    } else {
        let mut servers = state
            .servers
            .lock()
            .map_err(|_| ApiError::Internal("Failed to access server state"))?;
        servers.insert(id, server.clone());
    }

    Ok(Json(server))
}

pub(crate) async fn delete_server(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<SuccessResponse>> {
    require_auth(&state, &headers).await?;

    let removed = if let Some(pool) = &state.db {
        let removed = crate::db::delete_server(pool, &id).await?;
        let _ = crate::db::delete_server_credential(pool, &id).await;
        removed
    } else {
        let mut servers = state
            .servers
            .lock()
            .map_err(|_| ApiError::Internal("Failed to access server state"))?;
        servers.remove(&id).is_some()
    };

    if !removed {
        return Err(ApiError::NotFound("Server not found"));
    }

    let mut credentials = state
        .server_credentials
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access server credential state"))?;
    credentials.remove(&id);

    Ok(Json(SuccessResponse { success: true }))
}

async fn load_server_for_update(state: &AppState, id: &str) -> ApiResult<ServerConnection> {
    if let Some(pool) = &state.db {
        return crate::db::get_server(pool, id)
            .await?
            .ok_or(ApiError::NotFound("Server not found"));
    }

    let servers = state
        .servers
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access server state"))?;
    servers
        .get(id)
        .cloned()
        .ok_or(ApiError::NotFound("Server not found"))
}

fn credential_from_create_request(
    payload: &CreateServerConnectionRequest,
) -> ApiResult<ServerCredential> {
    match payload.auth_type {
        ServerAuthType::Password => {
            let password = required_trimmed(
                payload.password.as_deref().unwrap_or_default(),
                "SSH password is required",
            )?;
            Ok(ServerCredential::Password(password))
        }
        ServerAuthType::PrivateKey => {
            let private_key = required_trimmed(
                payload.private_key.as_deref().unwrap_or_default(),
                "SSH private key is required",
            )?;
            Ok(ServerCredential::PrivateKey(private_key))
        }
    }
}

fn credential_from_update_request(
    payload: &UpdateServerConnectionRequest,
    auth_type: &ServerAuthType,
) -> ApiResult<Option<ServerCredential>> {
    match auth_type {
        ServerAuthType::Password => {
            if let Some(password) = payload.password.as_deref() {
                return Ok(Some(ServerCredential::Password(required_trimmed(
                    password,
                    "SSH password is required",
                )?)));
            }
        }
        ServerAuthType::PrivateKey => {
            if let Some(private_key) = payload.private_key.as_deref() {
                return Ok(Some(ServerCredential::PrivateKey(required_trimmed(
                    private_key,
                    "SSH private key is required",
                )?)));
            }
        }
    }

    if payload.auth_type.is_some() {
        return match auth_type {
            ServerAuthType::Password => Err(ApiError::BadRequest("SSH password is required")),
            ServerAuthType::PrivateKey => Err(ApiError::BadRequest("SSH private key is required")),
        };
    }

    Ok(None)
}

fn required_trimmed(value: &str, error: &'static str) -> ApiResult<String> {
    let value = value.trim();
    if value.is_empty() {
        Err(ApiError::BadRequest(error))
    } else {
        Ok(value.to_owned())
    }
}
