use std::{
    collections::HashMap,
    sync::{Arc, Mutex, atomic::AtomicU64},
};

use sqlx::PgPool;

use crate::{
    models::{
        AiSettings, App, DemoInitState, InitStatus, Note, OAuthUser, ServerConnection,
        ServerCredential,
    },
    time::now_iso,
};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db: Option<PgPool>,
    pub(crate) apps: Arc<Mutex<HashMap<String, App>>>,
    pub(crate) notes: Arc<Mutex<HashMap<String, Note>>>,
    pub(crate) ai_settings: Arc<Mutex<Option<AiSettings>>>,
    pub(crate) init: Arc<Mutex<DemoInitState>>,
    pub(crate) next_id: Arc<AtomicU64>,
    pub(crate) next_server_id: Arc<AtomicU64>,
    pub(crate) next_note_id: Arc<AtomicU64>,
    pub(crate) auth_sessions: Arc<Mutex<HashMap<String, OAuthUser>>>,
    pub(crate) server_credentials: Arc<Mutex<HashMap<String, ServerCredential>>>,
    pub(crate) servers: Arc<Mutex<HashMap<String, ServerConnection>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::memory()
    }
}

impl AppState {
    pub(crate) fn memory() -> Self {
        let timestamp = now_iso();

        Self {
            db: None,
            apps: Arc::new(Mutex::new(HashMap::new())),
            notes: Arc::new(Mutex::new(HashMap::new())),
            ai_settings: Arc::new(Mutex::new(None)),
            init: Arc::new(Mutex::new(DemoInitState {
                status: InitStatus::NotInitialized,
                api_healthy: true,
                completed_at: None,
                created_at: timestamp.clone(),
                updated_at: timestamp,
            })),
            next_id: Arc::new(AtomicU64::new(1)),
            next_server_id: Arc::new(AtomicU64::new(1)),
            next_note_id: Arc::new(AtomicU64::new(1)),
            auth_sessions: Arc::new(Mutex::new(HashMap::new())),
            server_credentials: Arc::new(Mutex::new(HashMap::new())),
            servers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(crate) fn postgres(pool: PgPool) -> Self {
        let mut state = Self::memory();
        state.db = Some(pool);
        state
    }
}
