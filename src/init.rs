use axum::{Json, extract::State};

use crate::{
    errors::{ApiError, ApiResult},
    models::{DemoInitState, InitStatus},
    state::AppState,
    time::now_iso,
};

pub(crate) async fn get_init(State(state): State<AppState>) -> ApiResult<Json<DemoInitState>> {
    if let Some(pool) = &state.db
        && let Some(init) = crate::db::get_init(pool).await?
    {
        return Ok(Json(init));
    }

    let init = state
        .init
        .lock()
        .map_err(|_| ApiError::Internal("Failed to access init state"))?;

    Ok(Json(init.clone()))
}

pub(crate) async fn complete_init(State(state): State<AppState>) -> ApiResult<Json<DemoInitState>> {
    let mut init = if let Some(pool) = &state.db {
        crate::db::get_init(pool).await?.unwrap_or_else(|| {
            let timestamp = now_iso();
            DemoInitState {
                status: InitStatus::NotInitialized,
                api_healthy: true,
                completed_at: None,
                created_at: timestamp.clone(),
                updated_at: timestamp,
            }
        })
    } else {
        state
            .init
            .lock()
            .map_err(|_| ApiError::Internal("Failed to access init state"))?
            .clone()
    };

    let completed_at = init.completed_at.clone().unwrap_or_else(now_iso);
    init.status = InitStatus::Ready;
    init.api_healthy = true;
    init.completed_at = Some(completed_at.clone());
    init.updated_at = completed_at;

    if let Some(pool) = &state.db {
        crate::db::put_init(pool, &init).await?;
    } else {
        let mut state_init = state
            .init
            .lock()
            .map_err(|_| ApiError::Internal("Failed to access init state"))?;
        *state_init = init.clone();
    }

    Ok(Json(init))
}
