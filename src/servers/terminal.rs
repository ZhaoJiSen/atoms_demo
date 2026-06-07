use std::io::{Read, Write};

use axum::{
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::HeaderMap,
    response::IntoResponse,
};
use ssh2::Session;
use tokio::net::TcpStream;

use crate::{
    auth::require_auth,
    errors::{ApiError, ApiResult},
    models::{
        ServerConnectionStatus, ServerCredential, TerminalClientMessage, TerminalServerMessage,
    },
    state::AppState,
    time::now_iso,
};

pub(crate) async fn terminal_ws(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
) -> ApiResult<impl IntoResponse> {
    require_auth(&state, &headers).await?;
    ensure_server_exists(&state, &id).await?;

    Ok(ws.on_upgrade(move |socket| terminal_session(socket, state, id)))
}

async fn terminal_session(mut socket: WebSocket, state: AppState, id: String) {
    set_server_status(&state, &id, ServerConnectionStatus::Connecting, None, false).await;
    let _ = send_terminal_message(
        &mut socket,
        TerminalServerMessage::Status {
            status: ServerConnectionStatus::Connecting,
        },
    )
    .await;

    let (host, port, username, credential) = match get_server_info(&state, &id).await {
        Ok(info) => info,
        Err(error) => {
            fail_terminal(socket, state, id, error).await;
            return;
        }
    };

    let addr = format!("{host}:{port}");
    let tcp = match TcpStream::connect(&addr).await {
        Ok(tcp) => tcp,
        Err(error) => {
            fail_terminal(
                socket,
                state,
                id,
                format!("Failed to connect to {addr}: {error}"),
            )
            .await;
            return;
        }
    };

    let mut session = match Session::new() {
        Ok(session) => session,
        Err(error) => {
            fail_terminal(
                socket,
                state,
                id,
                format!("Failed to create SSH session: {error}"),
            )
            .await;
            return;
        }
    };

    let tcp_std = match tcp.into_std() {
        Ok(stream) => stream,
        Err(error) => {
            fail_terminal(
                socket,
                state,
                id,
                format!("Failed to prepare TCP stream: {error}"),
            )
            .await;
            return;
        }
    };
    if let Err(error) = tcp_std.set_nonblocking(false) {
        fail_terminal(
            socket,
            state,
            id,
            format!("Failed to configure TCP stream: {error}"),
        )
        .await;
        return;
    }
    session.set_tcp_stream(tcp_std);

    if let Err(error) = session.handshake() {
        fail_terminal(socket, state, id, format!("SSH handshake failed: {error}")).await;
        return;
    }

    match credential {
        ServerCredential::Password(password) => {
            if let Err(error) = session.userauth_password(&username, &password) {
                fail_terminal(socket, state, id, format!("Authentication failed: {error}")).await;
                return;
            }
        }
        ServerCredential::PrivateKey(private_key) => {
            if let Err(error) = session.userauth_pubkey_memory(&username, None, &private_key, None)
            {
                fail_terminal(socket, state, id, format!("Authentication failed: {error}")).await;
                return;
            }
        }
    }

    if !session.authenticated() {
        fail_terminal(socket, state, id, "Authentication failed".into()).await;
        return;
    }

    let mut channel = match session.channel_session() {
        Ok(channel) => channel,
        Err(error) => {
            fail_terminal(
                socket,
                state,
                id,
                format!("Failed to open channel: {error}"),
            )
            .await;
            return;
        }
    };

    if let Err(error) = channel.request_pty("xterm-256color", None, None) {
        fail_terminal(socket, state, id, format!("Failed to request PTY: {error}")).await;
        return;
    }

    if let Err(error) = channel.shell() {
        fail_terminal(socket, state, id, format!("Failed to start shell: {error}")).await;
        return;
    }

    set_server_status(&state, &id, ServerConnectionStatus::Connected, None, true).await;
    let _ = send_terminal_message(
        &mut socket,
        TerminalServerMessage::Status {
            status: ServerConnectionStatus::Connected,
        },
    )
    .await;

    session.set_blocking(false);

    let mut buf = vec![0u8; 4096];
    loop {
        match channel.read(&mut buf) {
            Ok(n) if n > 0 => {
                let output = String::from_utf8_lossy(&buf[..n]).to_string();
                let _ = send_terminal_message(
                    &mut socket,
                    TerminalServerMessage::Output { data: output },
                )
                .await;
            }
            _ => {}
        }

        match tokio::time::timeout(tokio::time::Duration::from_millis(10), socket.recv()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                if let Ok(msg) = serde_json::from_str::<TerminalClientMessage>(&text) {
                    match msg {
                        TerminalClientMessage::Input { data } => {
                            if let Err(error) = channel.write_all(data.as_bytes()) {
                                let _ = send_terminal_message(
                                    &mut socket,
                                    TerminalServerMessage::Error {
                                        error: format!("Failed to write input: {error}"),
                                    },
                                )
                                .await;
                            } else if let Err(error) = channel.flush() {
                                let _ = send_terminal_message(
                                    &mut socket,
                                    TerminalServerMessage::Error {
                                        error: format!("Failed to flush input: {error}"),
                                    },
                                )
                                .await;
                            }
                        }
                        TerminalClientMessage::Resize { cols, rows } => {
                            let _ = channel.request_pty_size(cols as u32, rows as u32, None, None);
                        }
                        TerminalClientMessage::Disconnect => {
                            break;
                        }
                    }
                }
            }
            Ok(Some(Ok(Message::Close(_)))) | Ok(None) => break,
            Ok(_) | Err(_) => {}
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let _ = channel.close();
    let _ = session.disconnect(None, "User disconnected", None);
    set_server_status(
        &state,
        &id,
        ServerConnectionStatus::Disconnected,
        None,
        false,
    )
    .await;
    let _ = send_terminal_message(
        &mut socket,
        TerminalServerMessage::Status {
            status: ServerConnectionStatus::Disconnected,
        },
    )
    .await;
}

async fn get_server_info(
    state: &AppState,
    id: &str,
) -> Result<(String, u16, String, ServerCredential), String> {
    let server = if let Some(pool) = &state.db {
        crate::db::get_server(pool, id)
            .await
            .map_err(|_| "Failed to load server".to_owned())?
            .ok_or_else(|| "Server not found".to_owned())?
    } else {
        let servers = state
            .servers
            .lock()
            .map_err(|_| "Failed to access server state".to_owned())?;
        servers
            .get(id)
            .cloned()
            .ok_or_else(|| "Server not found".to_owned())?
    };

    let memory_credential = {
        let credentials = state
            .server_credentials
            .lock()
            .map_err(|_| "SSH credential is required".to_owned())?;
        credentials.get(id).cloned()
    };

    let credential = if let Some(credential) = memory_credential {
        credential
    } else if let Some(pool) = &state.db {
        crate::db::get_server_credential(pool, id)
            .await
            .map_err(|_| "Failed to load SSH credential".to_owned())?
            .ok_or_else(|| "SSH credential is required".to_owned())?
    } else {
        return Err("SSH credential is required".to_owned());
    };

    if credential.is_empty() {
        return Err("SSH credential is required".into());
    }

    Ok((server.host, server.port, server.username, credential))
}

async fn fail_terminal(mut socket: WebSocket, state: AppState, id: String, error: String) {
    set_server_status(
        &state,
        &id,
        ServerConnectionStatus::Failed,
        Some(error.clone()),
        false,
    )
    .await;
    let _ = send_terminal_message(&mut socket, TerminalServerMessage::Error { error }).await;
    let _ = send_terminal_message(
        &mut socket,
        TerminalServerMessage::Status {
            status: ServerConnectionStatus::Failed,
        },
    )
    .await;
    let _ = socket.send(Message::Close(None)).await;
}

async fn send_terminal_message(
    socket: &mut WebSocket,
    message: TerminalServerMessage,
) -> Result<(), axum::Error> {
    let payload = serde_json::to_string(&message).unwrap_or_else(|_| {
        r#"{"type":"error","error":"Failed to serialize terminal message"}"#.into()
    });
    socket.send(Message::Text(payload.into())).await
}

async fn ensure_server_exists(state: &AppState, id: &str) -> ApiResult<()> {
    if let Some(pool) = &state.db {
        if crate::db::get_server(pool, id).await?.is_some() {
            return Ok(());
        }
        return Err(ApiError::NotFound("Server not found"));
    }

    let servers = state
        .servers
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access server state"))?;
    if servers.contains_key(id) {
        Ok(())
    } else {
        Err(ApiError::NotFound("Server not found"))
    }
}

async fn set_server_status(
    state: &AppState,
    id: &str,
    status: ServerConnectionStatus,
    error: Option<String>,
    connected: bool,
) {
    if let Some(pool) = &state.db {
        if let Ok(Some(mut server)) = crate::db::get_server(pool, id).await {
            let timestamp = now_iso();
            server.status = status;
            server.error = error;
            server.updated_at = timestamp.clone();
            if connected {
                server.last_connected_at = Some(timestamp);
            }
            let _ = crate::db::put_server(pool, "unknown", &server).await;
        }
        return;
    }

    if let Ok(mut servers) = state.servers.lock()
        && let Some(server) = servers.get_mut(id)
    {
        let timestamp = now_iso();
        server.status = status;
        server.error = error;
        server.updated_at = timestamp.clone();
        if connected {
            server.last_connected_at = Some(timestamp);
        }
    }
}

#[allow(dead_code)]
fn parse_terminal_client_message(
    payload: &str,
) -> Result<TerminalClientMessage, serde_json::Error> {
    serde_json::from_str(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::to_bytes,
        http::StatusCode,
        response::{IntoResponse, Response},
    };
    use serde_json::{Value, json};

    #[tokio::test]
    async fn missing_terminal_server_uses_404_error_shape() {
        let response = ensure_server_exists(&AppState::default(), "missing")
            .await
            .unwrap_err()
            .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            to_json(response).await,
            json!({ "error": "Server not found" })
        );
    }

    async fn to_json(response: Response) -> Value {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }
}
