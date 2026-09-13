#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};

    tracing_subscriber::fmt::init();

    footical_secrets::inject_from_bws(&[
        ("DATABASE_URL", "aa7a6d8e-af00-4910-937e-b44900b65003"),
        ("ADMIN_PASSWORD", "6f4d8d95-9208-405d-acd7-b44900b7df81"),
        ("COOKIE_SECRET", "5265df47-9b76-425c-898a-b44900b7eeb9"),
    ])
    .await?;

    let site_root = std::env::var("LEPTOS_SITE_ROOT").unwrap_or_else(|_| "target/site".to_owned());
    let site_addr = std::env::var("LEPTOS_SITE_ADDR")
        .ok()
        .and_then(|value| value.parse::<std::net::SocketAddr>().ok())
        .unwrap_or_else(|| std::net::SocketAddr::from(([0, 0, 0, 0], 3000)));
    let leptos_options = LeptosOptions::builder()
        .output_name("footical-website")
        .site_root(site_root)
        .site_pkg_dir("pkg")
        .site_addr(site_addr)
        .build();

    let routes = generate_route_list(footical_website::app::App);

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|source| anyhow::anyhow!("DATABASE_URL must be set: {source}"))?;
    let connect_options = database_url
        .parse::<sqlx::sqlite::SqliteConnectOptions>()
        .map_err(|source| anyhow::anyhow!("invalid DATABASE_URL: {source}"))?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect_with(connect_options)
        .await
        .map_err(|source| anyhow::anyhow!("failed to connect to database: {source}"))?;

    sqlx::raw_sql(include_str!("../schema.sql"))
        .execute(&pool)
        .await
        .map_err(|source| anyhow::anyhow!("failed to apply database schema: {source}"))?;

    let scrape_state = footical_website::server::new_scrape_state();

    let app_state = footical_website::server::AppState {
        pool: pool.clone(),
        leptos_options: leptos_options.clone(),
        scrape_state: scrape_state.clone(),
    };

    let site_router = Router::new()
        .route("/ical/{filename}", axum::routing::get(footical_website::server::ical::handler))
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let pool = pool.clone();
                let scrape_state = scrape_state.clone();
                move || {
                    leptos::context::provide_context(pool.clone());
                    leptos::context::provide_context(scrape_state.clone());
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || footical_website::app::shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler::<
            footical_website::server::AppState,
            _,
        >(footical_website::app::shell))
        .with_state(app_state);

    let app = Router::new()
        .fallback_service(site_router)
        .layer(axum::middleware::from_fn(route_calendar_subdomain));

    tokio::spawn(run_scheduled_scrapes(pool.clone(), scrape_state.clone()));

    let listener = tokio::net::TcpListener::bind(site_addr)
        .await
        .map_err(|source| anyhow::anyhow!("failed to bind {site_addr}: {source}"))?;
    tracing::event!(
        name: "server.started",
        tracing::Level::INFO,
        server.address = %site_addr.ip(),
        server.port = site_addr.port(),
        "server listening on {{server.address}}:{{server.port}}",
    );
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|source| anyhow::anyhow!("server error: {source}"))?;

    Ok(())
}

#[cfg(feature = "ssr")]
async fn run_scheduled_scrapes(
    pool: sqlx::SqlitePool,
    scrape_state: footical_website::server::ScrapeStateHandle,
) {
    use chrono::{Local, NaiveTime};

    const DEFAULT_SCRAPE_HOUR: u32 = 1;

    let scrape_hour = std::env::var("SCRAPE_HOUR")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(DEFAULT_SCRAPE_HOUR);
    let Some(target_time) = NaiveTime::from_hms_opt(scrape_hour, 0, 0) else {
        tracing::event!(
            name: "scrape.schedule.invalid_hour",
            tracing::Level::ERROR,
            scrape.hour = scrape_hour,
            "SCRAPE_HOUR {{scrape.hour}} is not a valid hour; scheduler stopped",
        );
        return;
    };

    loop {
        let now = Local::now();
        let today_target = now.date_naive().and_time(target_time);

        let next_run = if now.naive_local() >= today_target {
            match today_target.checked_add_signed(chrono::Duration::days(1)) {
                Some(tomorrow_target) => tomorrow_target,
                None => return,
            }
        } else {
            today_target
        };

        let duration_until = next_run.signed_duration_since(now.naive_local());
        let sleep_duration = duration_until
            .to_std()
            .unwrap_or(std::time::Duration::from_secs(3600));

        tracing::event!(
            name: "scrape.scheduled.waiting",
            tracing::Level::INFO,
            scrape.next_run = %next_run,
            scrape.sleep_seconds = sleep_duration.as_secs(),
            "scheduled scrape waiting until {{scrape.next_run}} ({{scrape.sleep_seconds}}s)",
        );

        tokio::time::sleep(sleep_duration).await;

        {
            let state = scrape_state.read().await;
            if state.is_running {
                tracing::event!(
                    name: "scrape.scheduled.skipped",
                    tracing::Level::INFO,
                    scrape.reason = "already_running",
                    "skipping scheduled scrape: {{scrape.reason}}",
                );
                continue;
            }
        }

        {
            let mut state = scrape_state.write().await;
            state.is_running = true;
        }

        tracing::event!(
            name: "scrape.scheduled.started",
            tracing::Level::INFO,
            scrape.trigger = "scheduled",
            "scheduled scrape started",
        );
        let result = footical_scraper::run_scrape(&pool).await;

        let mut state = scrape_state.write().await;
        state.is_running = false;
        state.last_run_at = Some(chrono::Utc::now());
        match result {
            Ok(scrape_result) => {
                tracing::event!(
                    name: "scrape.scheduled.completed",
                    tracing::Level::INFO,
                    scrape.trigger = "scheduled",
                    scrape.duration_seconds = scrape_result.duration_seconds,
                    scrape.fixtures.count = scrape_result.fixtures_upserted,
                    "scheduled scrape completed in {{scrape.duration_seconds}}s: {{scrape.fixtures.count}} fixtures",
                );
                state.last_error = None;
                state.last_result = Some(scrape_result);
            }
            Err(error) => {
                tracing::event!(
                    name: "scrape.scheduled.failed",
                    tracing::Level::ERROR,
                    scrape.trigger = "scheduled",
                    error.message = %error,
                    "scheduled scrape failed: {{error.message}}",
                );
                state.last_error = Some(error.to_string());
            }
        }
    }
}

#[cfg(feature = "ssr")]
const CALENDAR_SUBDOMAIN: &str = "calendar.footical.club";

#[cfg(feature = "ssr")]
const MAIN_SITE_URL: &str = "https://footical.club";

#[cfg(feature = "ssr")]
async fn route_calendar_subdomain(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::response::{IntoResponse, Redirect};

    let requested_host = request
        .headers()
        .get(axum::http::header::HOST)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.split(':').next().unwrap_or(value))
        .unwrap_or_default();

    if requested_host != CALENDAR_SUBDOMAIN {
        return next.run(request).await;
    }

    let path = request.uri().path();
    if path.starts_with("/ical/") {
        return next.run(request).await;
    }
    if !path.ends_with(".ics") {
        return Redirect::temporary(MAIN_SITE_URL).into_response();
    }

    match rewrite_to_feed_path(request) {
        Some(request) => next.run(request).await,
        None => Redirect::temporary(MAIN_SITE_URL).into_response(),
    }
}

#[cfg(feature = "ssr")]
fn rewrite_to_feed_path(mut request: axum::extract::Request) -> Option<axum::extract::Request> {
    use axum::http::uri::{PathAndQuery, Uri};

    let path = request.uri().path();
    let feed_path_and_query = match request.uri().query() {
        Some(query) => format!("/ical{path}?{query}"),
        None => format!("/ical{path}"),
    };

    let feed_path_and_query = feed_path_and_query.parse::<PathAndQuery>().ok()?;
    let mut parts = request.uri().clone().into_parts();
    parts.path_and_query = Some(feed_path_and_query);
    let uri = Uri::from_parts(parts).ok()?;
    *request.uri_mut() = uri;

    Some(request)
}

#[cfg(not(feature = "ssr"))]
fn main() {}
