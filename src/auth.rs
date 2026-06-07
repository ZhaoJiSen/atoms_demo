use axum::{
    Json,
    extract::{Query, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{COOKIE, LOCATION, SET_COOKIE},
    },
    response::{IntoResponse, Response},
};
use rand::{Rng, distributions::Alphanumeric};
use serde::Deserialize;

use crate::{
    errors::{ApiError, ApiResult},
    models::{
        AuthLoginQuery, AuthSession, OAuthProviderMode, OAuthProviderSummary, OAuthUser,
        SuccessResponse,
    },
    state::AppState,
    time::now_iso,
};

const OAUTH_STATE_COOKIE: &str = "atoms_demo_oauth_state";
const OAUTH_REDIRECT_COOKIE: &str = "atoms_demo_oauth_redirect";
const SESSION_COOKIE: &str = "atoms_demo_session";
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthCallbackQuery {
    code: Option<String>,
    state: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    id: String,
    email: String,
    name: Option<String>,
}

pub(crate) async fn login(
    State(_state): State<AppState>,
    Query(query): Query<AuthLoginQuery>,
) -> ApiResult<Response> {
    let config = GoogleOAuthConfig::from_env()?;
    let oauth_state = random_token("oauth_state");
    let redirect = sanitize_redirect(query.redirect.as_deref());
    let mut auth_url = reqwest::Url::parse(GOOGLE_AUTH_URL)
        .map_err(|_| ApiError::Internal("Failed to create Google auth URL"))?;
    auth_url
        .query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair("redirect_uri", &config.redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", "openid email profile")
        .append_pair("state", &oauth_state)
        .append_pair("access_type", "online")
        .append_pair("prompt", "select_account");

    let mut headers = HeaderMap::new();
    headers.append(
        SET_COOKIE,
        HeaderValue::from_str(&state_cookie(&oauth_state))
            .map_err(|_| ApiError::Internal("Failed to create auth cookie"))?,
    );
    headers.append(
        SET_COOKIE,
        HeaderValue::from_str(&redirect_cookie(&redirect))
            .map_err(|_| ApiError::Internal("Failed to create auth cookie"))?,
    );
    headers.insert(
        LOCATION,
        HeaderValue::from_str(auth_url.as_str())
            .map_err(|_| ApiError::Internal("Failed to create auth redirect"))?,
    );

    Ok((StatusCode::FOUND, headers).into_response())
}

pub(crate) async fn callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuthCallbackQuery>,
) -> ApiResult<Response> {
    let config = GoogleOAuthConfig::from_env()?;
    let expected_state = cookie_value(&headers, OAUTH_STATE_COOKIE);
    let received_state = query.state.as_deref();
    let received_code = query.code.as_deref();
    let redirect = cookie_value(&headers, OAUTH_REDIRECT_COOKIE)
        .map(|value| sanitize_redirect(Some(&value)))
        .unwrap_or_else(|| "/".into());

    let is_valid = expected_state.as_deref() == received_state && received_code.is_some();
    if !is_valid {
        return redirect_response(
            &format!("{}/auth/callback?status=error", config.frontend_url),
            vec![
                clear_cookie(OAUTH_STATE_COOKIE),
                clear_cookie(OAUTH_REDIRECT_COOKIE),
            ],
        );
    }

    let user = google_user_from_code(&config, received_code.unwrap()).await?;
    let session_token = random_token("sess");

    if let Some(pool) = &state.db {
        crate::db::create_session(pool, &session_token, &user).await?;
    } else {
        let mut sessions = state
            .auth_sessions
            .lock()
            .map_err(|_| ApiError::Internal("Failed to access auth session state"))?;
        sessions.insert(session_token.clone(), user);
    }

    redirect_response(
        &format!(
            "{}/auth/callback?status=success&redirect={redirect}",
            config.frontend_url
        ),
        vec![
            session_cookie(&session_token),
            clear_cookie(OAUTH_STATE_COOKIE),
            clear_cookie(OAUTH_REDIRECT_COOKIE),
        ],
    )
}

pub(crate) async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    if let Some(session_token) = cookie_value(&headers, SESSION_COOKIE) {
        if let Some(pool) = &state.db {
            crate::db::delete_session(pool, &session_token).await?;
        } else {
            let mut sessions = state
                .auth_sessions
                .lock()
                .map_err(|_| ApiError::Internal("Failed to access auth session state"))?;
            sessions.remove(&session_token);
        }
    }

    let mut response = Json(SuccessResponse { success: true }).into_response();
    response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(&clear_cookie(SESSION_COOKIE))
            .map_err(|_| ApiError::Internal("Failed to clear auth cookie"))?,
    );

    Ok(response)
}

pub(crate) async fn demo_login(
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    let user = OAuthUser {
        id: "demo_user".into(),
        display_name: "Demo User".into(),
        email: "demo@atoms-demo.local".into(),
        provider: OAuthProviderSummary {
            id: "demo".into(),
            name: "Demo".into(),
            mode: OAuthProviderMode::Google, // Demo Login fallback, reusing Google mode
        },
        created_at: now_iso(),
    };

    let session_token = random_token("sess");

    if let Some(pool) = &state.db {
        crate::db::create_session(pool, &session_token, &user).await?;
    } else {
        let mut sessions = state
            .auth_sessions
            .lock()
            .map_err(|_| ApiError::Internal("Failed to access auth session state"))?;
        sessions.insert(session_token.clone(), user.clone());
    }

    // Set session cookie - reuse existing logic
    let cookie = session_cookie(&session_token);
    let session = AuthSession {
        authenticated: true,
        provider: user.provider.clone(),
        user: Some(user),
        expires_at: None,
    };

    let mut response = Json(session).into_response();
    response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(&cookie)
            .map_err(|_| ApiError::Internal("Failed to set session cookie"))?,
    );

    Ok(response)
}

pub(crate) async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<AuthSession>> {
    Ok(Json(current_session(&state, &headers).await?))
}

pub(crate) async fn require_auth(state: &AppState, headers: &HeaderMap) -> ApiResult<OAuthUser> {
    current_user(state, headers)
        .await?
        .ok_or(ApiError::Unauthorized("Authentication required"))
}

pub(crate) async fn current_session(
    state: &AppState,
    headers: &HeaderMap,
) -> ApiResult<AuthSession> {
    let user = current_user(state, headers).await?;
    Ok(AuthSession {
        authenticated: user.is_some(),
        provider: provider_summary(),
        user,
        expires_at: None,
    })
}

async fn current_user(state: &AppState, headers: &HeaderMap) -> ApiResult<Option<OAuthUser>> {
    let Some(session_token) = cookie_value(headers, SESSION_COOKIE) else {
        return Ok(None);
    };

    if let Some(pool) = &state.db {
        return crate::db::user_by_session(pool, &session_token).await;
    }

    let sessions = state
        .auth_sessions
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access auth session state"))?;
    Ok(sessions.get(&session_token).cloned())
}

fn provider_summary() -> OAuthProviderSummary {
    OAuthProviderSummary {
        id: "google".into(),
        name: "Google".into(),
        mode: OAuthProviderMode::Google,
    }
}

struct GoogleOAuthConfig {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    frontend_url: String,
}

impl GoogleOAuthConfig {
    fn from_env() -> ApiResult<Self> {
        Ok(Self {
            client_id: std::env::var("GOOGLE_OAUTH_CLIENT_ID")
                .map_err(|_| ApiError::Internal("GOOGLE_OAUTH_CLIENT_ID is required"))?,
            client_secret: std::env::var("GOOGLE_OAUTH_CLIENT_SECRET")
                .map_err(|_| ApiError::Internal("GOOGLE_OAUTH_CLIENT_SECRET is required"))?,
            redirect_uri: std::env::var("GOOGLE_OAUTH_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3001/api/auth/callback".into()),
            frontend_url: std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".into()),
        })
    }
}

async fn google_user_from_code(config: &GoogleOAuthConfig, code: &str) -> ApiResult<OAuthUser> {
    let client = reqwest::Client::new();
    let token = client
        .post(GOOGLE_TOKEN_URL)
        .form(&[
            ("code", code),
            ("client_id", &config.client_id),
            ("client_secret", &config.client_secret),
            ("redirect_uri", &config.redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|_| ApiError::Internal("Failed to exchange Google OAuth code"))?
        .error_for_status()
        .map_err(|_| ApiError::Internal("Google OAuth token exchange failed"))?
        .json::<GoogleTokenResponse>()
        .await
        .map_err(|_| ApiError::Internal("Failed to decode Google OAuth token"))?;

    let profile = client
        .get(GOOGLE_USERINFO_URL)
        .bearer_auth(token.access_token)
        .send()
        .await
        .map_err(|_| ApiError::Internal("Failed to load Google profile"))?
        .error_for_status()
        .map_err(|_| ApiError::Internal("Google profile request failed"))?
        .json::<GoogleUserInfo>()
        .await
        .map_err(|_| ApiError::Internal("Failed to decode Google profile"))?;

    Ok(OAuthUser {
        id: format!("google_{}", profile.id),
        display_name: profile.name.unwrap_or_else(|| profile.email.clone()),
        email: profile.email,
        provider: provider_summary(),
        created_at: now_iso(),
    })
}

fn redirect_response(location: &str, cookies: Vec<String>) -> ApiResult<Response> {
    let mut headers = HeaderMap::new();
    headers.insert(
        LOCATION,
        HeaderValue::from_str(location)
            .map_err(|_| ApiError::Internal("Failed to create auth redirect"))?,
    );
    for cookie in cookies {
        headers.append(
            SET_COOKIE,
            HeaderValue::from_str(&cookie)
                .map_err(|_| ApiError::Internal("Failed to create auth cookie"))?,
        );
    }

    Ok((StatusCode::FOUND, headers).into_response())
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let (key, value) = cookie.trim().split_once('=')?;
                (key == name).then(|| value.to_owned())
            })
        })
}

fn sanitize_redirect(value: Option<&str>) -> String {
    let value = value.unwrap_or("/auth/callback");
    if value.starts_with('/') && !value.starts_with("//") && !value.contains(['\r', '\n', ';']) {
        value.to_owned()
    } else {
        "/auth/callback".into()
    }
}

fn state_cookie(value: &str) -> String {
    format!("{OAUTH_STATE_COOKIE}={value}; Path=/; Max-Age=300; HttpOnly; SameSite=Lax")
}

fn redirect_cookie(value: &str) -> String {
    format!("{OAUTH_REDIRECT_COOKIE}={value}; Path=/; Max-Age=300; HttpOnly; SameSite=Lax")
}

fn session_cookie(value: &str) -> String {
    format!("{SESSION_COOKIE}={value}; Path=/; HttpOnly; SameSite=Lax")
}

fn clear_cookie(name: &str) -> String {
    format!("{name}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax")
}

fn random_token(prefix: &str) -> String {
    let value: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect();
    format!("{prefix}_{value}")
}
