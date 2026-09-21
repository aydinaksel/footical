use crate::types::{Fixture, TeamListing, TodayFixture};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::server::database::{database_pool, report_query_failure};

#[cfg(feature = "ssr")]
const TEAM_LISTING_SELECT: &str = "SELECT
         team.team_id,
         team.name AS team_name,
         division.name AS division_name,
         league.name AS league_name
     FROM team
     JOIN division ON division.division_id = team.division_id
     JOIN league ON league.league_id = division.league_id";

#[cfg(feature = "ssr")]
const TEAM_SEARCH_LIMIT: i32 = 20;

#[server]
pub async fn search_teams(search_text: String) -> Result<Vec<TeamListing>, ServerFnError> {
    let trimmed_search_text = search_text.trim();
    if trimmed_search_text.is_empty() {
        return Ok(vec![]);
    }

    let pool = database_pool()?;
    let statement = format!(
        "{TEAM_LISTING_SELECT} WHERE team.name LIKE '%' || ? || '%' ORDER BY team.name LIMIT ?"
    );
    let rows = sqlx::query_as::<_, TeamListing>(sqlx::AssertSqlSafe(statement))
        .bind(trimmed_search_text)
        .bind(TEAM_SEARCH_LIMIT)
        .fetch_all(&pool)
        .await
        .map_err(|error| report_query_failure("search_teams", error))?;
    Ok(rows)
}

#[server]
pub async fn get_team_listing(team_id: i32) -> Result<Option<TeamListing>, ServerFnError> {
    let pool = database_pool()?;
    let statement = format!("{TEAM_LISTING_SELECT} WHERE team.team_id = ?");
    let row = sqlx::query_as::<_, TeamListing>(sqlx::AssertSqlSafe(statement))
        .bind(team_id)
        .fetch_optional(&pool)
        .await
        .map_err(|error| report_query_failure("select_team_listing", error))?;
    Ok(row)
}

#[server]
pub async fn get_team_fixtures(team_id: i32) -> Result<Vec<Fixture>, ServerFnError> {
    let pool = database_pool()?;
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
         WHERE (fixture.home_team_id = ? OR fixture.away_team_id = ?)
           AND fixture.scheduled_at >= datetime('now')
         ORDER BY fixture.scheduled_at",
    )
    .bind(team_id)
    .bind(team_id)
    .fetch_all(&pool)
    .await
    .map_err(|error| report_query_failure("select_team_fixtures", error))?;
    Ok(rows)
}

#[server]
pub async fn get_todays_fixtures() -> Result<Vec<TodayFixture>, ServerFnError> {
    let pool = database_pool()?;
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
         WHERE date(fixture.scheduled_at) = date('now', 'localtime')
         ORDER BY league.name, division.name, fixture.scheduled_at",
    )
    .fetch_all(&pool)
    .await
    .map_err(|error| report_query_failure("select_todays_fixtures", error))?;
    Ok(rows)
}
