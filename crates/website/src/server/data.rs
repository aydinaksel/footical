use crate::types::{Division, Fixture, League, Team, TodayFixture};
use leptos::prelude::*;

#[server]
pub async fn get_leagues() -> Result<Vec<League>, ServerFnError> {
    let pool = use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool"))?;
    let rows = sqlx::query_as::<_, League>(
        "SELECT league_id, name FROM league ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))?;
    Ok(rows)
}

#[server]
pub async fn get_divisions() -> Result<Vec<Division>, ServerFnError> {
    let pool = use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool"))?;
    let rows = sqlx::query_as::<_, Division>(
        "SELECT division_id, league_id, name FROM division ORDER BY league_id, name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))?;
    Ok(rows)
}

#[server]
pub async fn get_teams() -> Result<Vec<Team>, ServerFnError> {
    let pool = use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool"))?;
    let rows = sqlx::query_as::<_, Team>(
        "SELECT team_id, division_id, name FROM team ORDER BY division_id, name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))?;
    Ok(rows)
}

#[server]
pub async fn get_fixtures() -> Result<Vec<Fixture>, ServerFnError> {
    let pool = use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool"))?;
    let rows = sqlx::query_as::<_, Fixture>(
        "SELECT
             fixture.fixture_id,
             fixture.home_team_id,
             fixture.away_team_id,
             home_team.name AS home_team_name,
             away_team.name AS away_team_name,
             fixture.scheduled_at,
             fixture.status
         FROM fixture
         JOIN team home_team ON home_team.team_id = fixture.home_team_id
         JOIN team away_team ON away_team.team_id = fixture.away_team_id
         ORDER BY fixture.scheduled_at",
    )
    .fetch_all(&pool)
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))?;
    Ok(rows)
}

#[server]
pub async fn get_todays_fixtures() -> Result<Vec<TodayFixture>, ServerFnError> {
    let pool = use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool"))?;
    let rows = sqlx::query_as::<_, TodayFixture>(
        "SELECT
             fixture.fixture_id,
             fixture.scheduled_at,
             fixture.status,
             home_team.name AS home_team_name,
             away_team.name AS away_team_name,
             division.name AS division_name,
             league.name AS league_name,
             venue.name AS venue_name,
             venue.address AS venue_address
         FROM fixture
         JOIN team home_team ON home_team.team_id = fixture.home_team_id
         JOIN team away_team ON away_team.team_id = fixture.away_team_id
         JOIN division ON division.division_id = fixture.division_id
         JOIN league ON league.league_id = division.league_id
         LEFT JOIN venue ON venue.venue_id = league.venue_id
         WHERE fixture.scheduled_at::date = CURRENT_DATE
         ORDER BY league.name, division.name, fixture.scheduled_at",
    )
    .fetch_all(&pool)
    .await
    .map_err(|error| ServerFnError::new(error.to_string()))?;
    Ok(rows)
}
