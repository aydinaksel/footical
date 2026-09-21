use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use tracing::{event, Level};

pub async fn handler(
    State(pool): State<sqlx::SqlitePool>,
    Path(filename): Path<String>,
) -> impl IntoResponse {
    let team_id: i32 = match filename
        .strip_suffix(".ics")
        .and_then(|stem| stem.parse().ok())
    {
        Some(team_id) => team_id,
        None => {
            event!(
                name: "calendar.feed.rejected",
                Level::DEBUG,
                url.path = filename,
                "calendar feed {{url.path}} is not a team feed",
            );
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    match footical_scraper::ical::generate_for_team(&pool, team_id).await {
        Ok(Some(body)) => {
            event!(
                name: "calendar.feed.served",
                Level::DEBUG,
                team.id = team_id,
                http.response.body.size = body.len(),
                "served calendar feed for team {{team.id}}",
            );
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "text/calendar; charset=utf-8"),
                    (header::CACHE_CONTROL, "public, max-age=900"),
                ],
                body,
            )
                .into_response()
        }
        Ok(None) => {
            event!(
                name: "calendar.feed.missing",
                Level::DEBUG,
                team.id = team_id,
                "no calendar feed for team {{team.id}}",
            );
            StatusCode::NOT_FOUND.into_response()
        }
        Err(error) => {
            event!(
                name: "calendar.feed.failure",
                Level::ERROR,
                team.id = team_id,
                error.message = %error,
                "calendar feed for team {{team.id}} failed: {{error.message}}",
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
