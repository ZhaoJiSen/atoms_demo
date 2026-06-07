use sqlx::{PgPool, Row, postgres::PgPoolOptions};

use crate::{
    errors::{ApiError, ApiResult},
    models::{App, DemoInitState, Note, OAuthUser, ServerConnection, ServerCredential},
};

pub(crate) async fn connect_from_env() -> ApiResult<PgPool> {
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| ApiError::Internal("DATABASE_URL is required"))?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|_| ApiError::Internal("Failed to connect to Postgres"))?;
    migrate(&pool).await?;
    Ok(pool)
}

pub(crate) async fn migrate(pool: &PgPool) -> ApiResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS demo_init (
          id BOOLEAN PRIMARY KEY DEFAULT TRUE,
          data JSONB NOT NULL,
          updated_at TEXT NOT NULL,
          CHECK (id)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal("Failed to create demo_init table"))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
          id TEXT PRIMARY KEY,
          email TEXT NOT NULL,
          display_name TEXT NOT NULL,
          provider_id TEXT NOT NULL,
          data JSONB NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal("Failed to create users table"))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
          token TEXT PRIMARY KEY,
          user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
          created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal("Failed to create sessions table"))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS apps (
          id TEXT PRIMARY KEY,
          user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
          data JSONB NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal("Failed to create apps table"))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS servers (
          id TEXT PRIMARY KEY,
          user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
          data JSONB NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal("Failed to create servers table"))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS server_credentials (
          server_id TEXT PRIMARY KEY REFERENCES servers(id) ON DELETE CASCADE,
          user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
          auth_type TEXT NOT NULL,
          secret TEXT NOT NULL,
          updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal("Failed to create server_credentials table"))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS notes (
          id TEXT PRIMARY KEY,
          user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
          data JSONB NOT NULL,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal("Failed to create notes table"))?;

    Ok(())
}

pub(crate) async fn get_init(pool: &PgPool) -> ApiResult<Option<DemoInitState>> {
    let row = sqlx::query("SELECT data FROM demo_init WHERE id = TRUE")
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiError::Internal("Failed to load init state"))?;
    row.map(|row| json_value(row, "data"))
        .transpose()?
        .map(from_json)
        .transpose()
}

pub(crate) async fn put_init(pool: &PgPool, init: &DemoInitState) -> ApiResult<()> {
    let data = to_json(init)?;
    sqlx::query(
        r#"
        INSERT INTO demo_init (id, data, updated_at)
        VALUES (TRUE, $1, $2)
        ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data, updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(data)
    .bind(&init.updated_at)
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal("Failed to save init state"))?;
    Ok(())
}

pub(crate) async fn upsert_user(pool: &PgPool, user: &OAuthUser) -> ApiResult<()> {
    let data = to_json(user)?;
    sqlx::query(
        r#"
        INSERT INTO users (id, email, display_name, provider_id, data, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $6)
        ON CONFLICT (id) DO UPDATE SET
          email = EXCLUDED.email,
          display_name = EXCLUDED.display_name,
          provider_id = EXCLUDED.provider_id,
          data = EXCLUDED.data,
          updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(&user.id)
    .bind(&user.email)
    .bind(&user.display_name)
    .bind(&user.provider.id)
    .bind(data)
    .bind(&user.created_at)
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal("Failed to save user"))?;
    Ok(())
}

pub(crate) async fn create_session(pool: &PgPool, token: &str, user: &OAuthUser) -> ApiResult<()> {
    upsert_user(pool, user).await?;
    sqlx::query("INSERT INTO sessions (token, user_id, created_at) VALUES ($1, $2, $3)")
        .bind(token)
        .bind(&user.id)
        .bind(&user.created_at)
        .execute(pool)
        .await
        .map_err(|_| ApiError::Internal("Failed to save session"))?;
    Ok(())
}

pub(crate) async fn delete_session(pool: &PgPool, token: &str) -> ApiResult<()> {
    sqlx::query("DELETE FROM sessions WHERE token = $1")
        .bind(token)
        .execute(pool)
        .await
        .map_err(|_| ApiError::Internal("Failed to delete session"))?;
    Ok(())
}

pub(crate) async fn user_by_session(pool: &PgPool, token: &str) -> ApiResult<Option<OAuthUser>> {
    let row = sqlx::query(
        r#"
        SELECT users.data
        FROM sessions
        JOIN users ON users.id = sessions.user_id
        WHERE sessions.token = $1
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .map_err(|_| ApiError::Internal("Failed to load session"))?;
    row.map(|row| json_value(row, "data"))
        .transpose()?
        .map(from_json)
        .transpose()
}

pub(crate) async fn list_apps(pool: &PgPool, user_id: &str) -> ApiResult<Vec<App>> {
    let rows = sqlx::query("SELECT data FROM apps WHERE user_id = $1 ORDER BY updated_at DESC")
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|_| ApiError::Internal("Failed to list apps"))?;
    rows.into_iter()
        .map(|row| json_value(row, "data").and_then(from_json))
        .collect()
}

pub(crate) async fn get_app(pool: &PgPool, id: &str) -> ApiResult<Option<App>> {
    let row = sqlx::query("SELECT data FROM apps WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiError::Internal("Failed to load app"))?;
    row.map(|row| json_value(row, "data"))
        .transpose()?
        .map(from_json)
        .transpose()
}

pub(crate) async fn put_app(pool: &PgPool, user_id: &str, app: &App) -> ApiResult<()> {
    let data = to_json(app)?;
    sqlx::query(
        r#"
        INSERT INTO apps (id, user_id, data, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data, updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(&app.id)
    .bind(user_id)
    .bind(data)
    .bind(&app.created_at)
    .bind(&app.updated_at)
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal("Failed to save app"))?;
    Ok(())
}

pub(crate) async fn delete_app(pool: &PgPool, id: &str) -> ApiResult<bool> {
    let result = sqlx::query("DELETE FROM apps WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|_| ApiError::Internal("Failed to delete app"))?;
    Ok(result.rows_affected() > 0)
}

pub(crate) async fn list_servers(
    pool: &PgPool,
    user_id: Option<&str>,
) -> ApiResult<Vec<ServerConnection>> {
    let rows = if let Some(user_id) = user_id {
        sqlx::query("SELECT data FROM servers WHERE user_id = $1 ORDER BY id ASC")
            .bind(user_id)
            .fetch_all(pool)
            .await
    } else {
        sqlx::query("SELECT data FROM servers ORDER BY id ASC")
            .fetch_all(pool)
            .await
    }
    .map_err(|_| ApiError::Internal("Failed to list servers"))?;

    rows.into_iter()
        .map(|row| json_value(row, "data").and_then(from_json))
        .collect()
}

pub(crate) async fn get_server(pool: &PgPool, id: &str) -> ApiResult<Option<ServerConnection>> {
    let row = sqlx::query("SELECT data FROM servers WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiError::Internal("Failed to load server"))?;
    row.map(|row| json_value(row, "data"))
        .transpose()?
        .map(from_json)
        .transpose()
}

pub(crate) async fn put_server(
    pool: &PgPool,
    user_id: &str,
    server: &ServerConnection,
) -> ApiResult<()> {
    let data = to_json(server)?;
    sqlx::query(
        r#"
        INSERT INTO servers (id, user_id, data, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data, updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(&server.id)
    .bind(user_id)
    .bind(data)
    .bind(&server.created_at)
    .bind(&server.updated_at)
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal("Failed to save server"))?;
    Ok(())
}

pub(crate) async fn delete_server(pool: &PgPool, id: &str) -> ApiResult<bool> {
    let result = sqlx::query("DELETE FROM servers WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|_| ApiError::Internal("Failed to delete server"))?;
    Ok(result.rows_affected() > 0)
}

pub(crate) async fn put_server_credential(
    pool: &PgPool,
    user_id: &str,
    server_id: &str,
    credential: &ServerCredential,
    updated_at: &str,
) -> ApiResult<()> {
    let (auth_type, secret) = match credential {
        ServerCredential::Password(password) => ("password", password),
        ServerCredential::PrivateKey(private_key) => ("private_key", private_key),
    };

    sqlx::query(
        r#"
        INSERT INTO server_credentials (server_id, user_id, auth_type, secret, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (server_id) DO UPDATE SET
          user_id = EXCLUDED.user_id,
          auth_type = EXCLUDED.auth_type,
          secret = EXCLUDED.secret,
          updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(server_id)
    .bind(user_id)
    .bind(auth_type)
    .bind(secret)
    .bind(updated_at)
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal("Failed to save server credential"))?;
    Ok(())
}

pub(crate) async fn get_server_credential(
    pool: &PgPool,
    server_id: &str,
) -> ApiResult<Option<ServerCredential>> {
    let row = sqlx::query(
        r#"
        SELECT auth_type, secret
        FROM server_credentials
        WHERE server_id = $1
        "#,
    )
    .bind(server_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| ApiError::Internal("Failed to load server credential"))?;

    let Some(row) = row else {
        return Ok(None);
    };

    let auth_type: String = row
        .try_get("auth_type")
        .map_err(|_| ApiError::Internal("Failed to decode server credential"))?;
    let secret: String = row
        .try_get("secret")
        .map_err(|_| ApiError::Internal("Failed to decode server credential"))?;

    match auth_type.as_str() {
        "password" => Ok(Some(ServerCredential::Password(secret))),
        "private_key" => Ok(Some(ServerCredential::PrivateKey(secret))),
        _ => Err(ApiError::Internal("Invalid server credential type")),
    }
}

pub(crate) async fn delete_server_credential(pool: &PgPool, server_id: &str) -> ApiResult<()> {
    sqlx::query("DELETE FROM server_credentials WHERE server_id = $1")
        .bind(server_id)
        .execute(pool)
        .await
        .map_err(|_| ApiError::Internal("Failed to delete server credential"))?;
    Ok(())
}

// Notes operations
pub(crate) async fn list_notes(pool: &PgPool, user_id: &str) -> ApiResult<Vec<Note>> {
    let rows = sqlx::query("SELECT data FROM notes WHERE user_id = $1 ORDER BY updated_at DESC")
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|_| ApiError::Internal("Failed to list notes"))?;
    rows.into_iter()
        .map(|row| json_value(row, "data").and_then(from_json))
        .collect()
}

pub(crate) async fn get_note(pool: &PgPool, id: &str) -> ApiResult<Option<Note>> {
    let row = sqlx::query("SELECT data FROM notes WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiError::Internal("Failed to load note"))?;
    row.map(|row| json_value(row, "data"))
        .transpose()?
        .map(from_json)
        .transpose()
}

pub(crate) async fn put_note(pool: &PgPool, user_id: &str, note: &Note) -> ApiResult<()> {
    let data = to_json(note)?;
    sqlx::query(
        r#"
        INSERT INTO notes (id, user_id, data, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data, updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(&note.id)
    .bind(user_id)
    .bind(data)
    .bind(&note.created_at)
    .bind(&note.updated_at)
    .execute(pool)
    .await
    .map_err(|_| ApiError::Internal("Failed to save note"))?;
    Ok(())
}

pub(crate) async fn delete_note(pool: &PgPool, id: &str) -> ApiResult<bool> {
    let result = sqlx::query("DELETE FROM notes WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|_| ApiError::Internal("Failed to delete note"))?;
    Ok(result.rows_affected() > 0)
}

fn json_value(row: sqlx::postgres::PgRow, column: &str) -> ApiResult<serde_json::Value> {
    row.try_get(column)
        .map_err(|_| ApiError::Internal("Failed to decode Postgres JSON"))
}

fn from_json<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> ApiResult<T> {
    serde_json::from_value(value).map_err(|_| ApiError::Internal("Failed to decode JSON data"))
}

fn to_json<T: serde::Serialize>(value: &T) -> ApiResult<serde_json::Value> {
    serde_json::to_value(value).map_err(|_| ApiError::Internal("Failed to encode JSON data"))
}
