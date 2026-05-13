#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};

    tracing_subscriber::fmt::init();

    let leptos_options = LeptosOptions::builder()
        .output_name("footical-app")
        .site_root("site")
        .site_pkg_dir("pkg")
        .site_addr(std::net::SocketAddr::from(([0, 0, 0, 0], 3000)))
        .build();

    let routes = generate_route_list(footical_app::app::App);

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to database");

    let ical_directory = std::env::var("ICAL_OUTPUT_DIR").unwrap_or_else(|_| "./ical".to_owned());
    let ical_path = std::path::PathBuf::from(&ical_directory);

    let scrape_state = footical_app::server::new_scrape_state();

    let ical_service = tower_http::services::ServeDir::new(&ical_directory);

    let app_state = footical_app::server::AppState {
        pool: pool.clone(),
        leptos_options: leptos_options.clone(),
        ical_directory: ical_path.clone(),
        scrape_state: scrape_state.clone(),
    };

    let app = Router::new()
        .nest_service("/ical", ical_service)
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let pool = pool.clone();
                let scrape_state = scrape_state.clone();
                let ical_path = ical_path.clone();
                move || {
                    leptos::context::provide_context(pool.clone());
                    leptos::context::provide_context(scrape_state.clone());
                    leptos::context::provide_context(ical_path.clone());
                }
            },
            footical_app::app::App,
        )
        .fallback(leptos_axum::file_and_error_handler::<
            footical_app::server::AppState,
            _,
        >(footical_app::app::shell))
        .with_state(app_state);

    tokio::spawn(run_scheduled_scrapes(
        pool.clone(),
        ical_path.clone(),
        scrape_state.clone(),
    ));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind");
    tracing::info!("listening on http://0.0.0.0:3000");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("server error");
}

#[cfg(feature = "ssr")]
async fn run_scheduled_scrapes(
    pool: sqlx::PgPool,
    ical_directory: std::path::PathBuf,
    scrape_state: footical_app::server::ScrapeStateHandle,
) {
    use chrono::{Local, NaiveTime};

    let scrape_hour = std::env::var("SCRAPE_HOUR")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(1);

    loop {
        let now = Local::now();
        let target_time = NaiveTime::from_hms_opt(scrape_hour, 0, 0).unwrap();
        let today_target = now.date_naive().and_time(target_time);

        let next_run = if now.naive_local() >= today_target {
            today_target + chrono::Duration::days(1)
        } else {
            today_target
        };

        let duration_until = next_run - now.naive_local();
        let sleep_duration = duration_until
            .to_std()
            .unwrap_or(std::time::Duration::from_secs(3600));

        tracing::info!(
            next_run = %next_run,
            sleep_seconds = sleep_duration.as_secs(),
            "scheduled scrape waiting",
        );

        tokio::time::sleep(sleep_duration).await;

        {
            let state = scrape_state.read().await;
            if state.is_running {
                tracing::info!("skipping scheduled scrape, one is already running");
                continue;
            }
        }

        {
            let mut state = scrape_state.write().await;
            state.is_running = true;
        }

        tracing::info!("starting scheduled scrape");
        let result = footical_scraper::run_scrape(&pool, &ical_directory).await;

        let mut state = scrape_state.write().await;
        state.is_running = false;
        state.last_run_at = Some(chrono::Utc::now());
        match result {
            Ok(scrape_result) => {
                tracing::info!(?scrape_result, "scheduled scrape completed");
                state.last_error = None;
                state.last_result = Some(scrape_result);
            }
            Err(error) => {
                tracing::error!(%error, "scheduled scrape failed");
                state.last_error = Some(error.to_string());
            }
        }
    }
}

#[cfg(not(feature = "ssr"))]
fn main() {}
