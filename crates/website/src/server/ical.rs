use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;

pub async fn handler(
    State(pool): State<sqlx::SqlitePool>,
    Path(filename): Path<String>,
) -> impl IntoResponse {
    let team_id: i32 = match filename.strip_suffix(".ics").and_then(|s| s.parse().ok()) {
        Some(id) => id,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    match footical_scraper::ical::generate_for_team(&pool, team_id).await {
        Ok(Some(body)) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "text/calendar; charset=utf-8"),
                (header::CACHE_CONTROL, "public, max-age=900"),
            ],
            body,
        )
            .into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
