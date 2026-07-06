use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::server::ScrapeStateHandle;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScrapeStatus {
    pub is_running: bool,
    pub last_result_summary: Option<String>,
    pub last_error: Option<String>,
    pub last_run_at: Option<String>,
}

#[server]
pub async fn trigger_scrape() -> Result<(), ServerFnError> {
    let pool = use_context::<sqlx::SqlitePool>()
        .ok_or_else(|| ServerFnError::new("no database pool"))?;
    let scrape_state = use_context::<ScrapeStateHandle>()
        .ok_or_else(|| ServerFnError::new("no scrape state"))?;
    {
        let state = scrape_state.read().await;
        if state.is_running {
            return Err(ServerFnError::new("scrape already running"));
        }
    }

    {
        let mut state = scrape_state.write().await;
        state.is_running = true;
    }

    let scrape_state_clone = scrape_state.clone();
    tokio::spawn(async move {
        let result =
            footical_scraper::run_scrape(&pool).await;

        let mut state = scrape_state_clone.write().await;
        state.is_running = false;
        state.last_run_at = Some(chrono::Utc::now());
        match result {
            Ok(scrape_result) => {
                tracing::event!(
                    name: "scrape.manual.completed",
                    tracing::Level::INFO,
                    scrape.trigger = "manual",
                    scrape.duration_seconds = scrape_result.duration_seconds,
                    scrape.fixtures.count = scrape_result.fixtures_upserted,
                    "manual scrape completed in {{scrape.duration_seconds}}s: {{scrape.fixtures.count}} fixtures",
                );
                state.last_error = None;
                state.last_result = Some(scrape_result);
            }
            Err(error) => {
                tracing::event!(
                    name: "scrape.manual.failed",
                    tracing::Level::ERROR,
                    scrape.trigger = "manual",
                    error.message = %error,
                    "manual scrape failed: {{error.message}}",
                );
                state.last_error = Some(error.to_string());
            }
        }
    });

    Ok(())
}

#[server]
pub async fn get_scrape_status() -> Result<ScrapeStatus, ServerFnError> {
    let scrape_state = use_context::<ScrapeStateHandle>()
        .ok_or_else(|| ServerFnError::new("no scrape state"))?;

    let state = scrape_state.read().await;

    let last_result_summary = state.last_result.as_ref().map(|result| {
        format!(
            "{} venues, {} leagues, {} divisions, {} fixtures in {:.1}s",
            result.venues_upserted,
            result.leagues_upserted,
            result.divisions_upserted,
            result.fixtures_upserted,
            result.duration_seconds,
        )
    });

    Ok(ScrapeStatus {
        is_running: state.is_running,
        last_result_summary,
        last_error: state.last_error.clone(),
        last_run_at: state.last_run_at.map(|datetime| datetime.to_rfc3339()),
    })
}
