use std::sync::Arc;

use tokio::sync::RwLock;

const ADMIN_PASSWORD_CREDENTIAL: &str = "admin-password";
const SESSION_SECRET_CREDENTIAL: &str = "cookie-secret";

#[derive(Clone, axum::extract::FromRef)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub leptos_options: leptos::prelude::LeptosOptions,
    pub scrape_state: ScrapeStateHandle,
    pub admin_credentials: AdminCredentialsHandle,
}

pub struct AdminCredentials {
    pub password: footical_secrets::SecretString,
    pub session_secret: footical_secrets::SecretString,
}

pub type AdminCredentialsHandle = Arc<AdminCredentials>;

impl AdminCredentials {
    pub fn from_systemd() -> Result<AdminCredentialsHandle, footical_secrets::CredentialsError> {
        let credentials = footical_secrets::Credentials::from_systemd()?;
        Ok(Arc::new(Self {
            password: credentials.read(ADMIN_PASSWORD_CREDENTIAL)?,
            session_secret: credentials.read(SESSION_SECRET_CREDENTIAL)?,
        }))
    }
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
