use std::sync::Arc;

use tokio::sync::RwLock;

#[derive(Clone, axum::extract::FromRef)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub leptos_options: leptos::prelude::LeptosOptions,
    pub scrape_state: ScrapeStateHandle,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScrapeState {
    pub is_running: bool,
    pub last_result: Option<footical_scraper::ScrapeResult>,
    pub last_error: Option<String>,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub type ScrapeStateHandle = Arc<RwLock<ScrapeState>>;

pub fn new_scrape_state() -> ScrapeStateHandle {
    Arc::new(RwLock::new(ScrapeState {
        is_running: false,
        last_result: None,
        last_error: None,
        last_run_at: None,
    }))
}
