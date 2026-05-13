use sqlx::PgPool;

pub async fn upsert_venue(
    pool: &PgPool,
    source_key: &str,
    name: &str,
    address: Option<&str>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO venue (source_key, name, address)
         VALUES ($1, $2, $3)
         ON CONFLICT (source_key) DO UPDATE SET
           name = EXCLUDED.name,
           address = EXCLUDED.address,
           updated_at = CURRENT_TIMESTAMP",
    )
    .bind(source_key)
    .bind(name)
    .bind(address)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn upsert_league(
    pool: &PgPool,
    name: &str,
    day_of_week: Option<i32>,
    source_key: &str,
    number_of_players: Option<i32>,
    starts_at: Option<&str>,
    ends_at: Option<&str>,
    price_pence: Option<i32>,
    venue_source_key: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO league (organisation_id, venue_id, name, day_of_week, source_key,
                             number_of_players, starts_at, ends_at, price_pence)
         SELECT 1, venue.venue_id, $1, $2, $3, $4, $5::time, $6::time, $7
         FROM venue
         WHERE venue.source_key = $8
         ON CONFLICT (organisation_id, source_key) DO UPDATE SET
           name = EXCLUDED.name,
           day_of_week = EXCLUDED.day_of_week,
           venue_id = EXCLUDED.venue_id,
           number_of_players = EXCLUDED.number_of_players,
           starts_at = EXCLUDED.starts_at,
           ends_at = EXCLUDED.ends_at,
           price_pence = EXCLUDED.price_pence,
           updated_at = CURRENT_TIMESTAMP",
    )
    .bind(name)
    .bind(day_of_week)
    .bind(source_key)
    .bind(number_of_players)
    .bind(starts_at)
    .bind(ends_at)
    .bind(price_pence)
    .bind(venue_source_key)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn upsert_division(
    pool: &PgPool,
    name: &str,
    source_key: &str,
    league_source_key: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO division (league_id, name, source_key)
         SELECT league.league_id, $1, $2
         FROM league
         WHERE league.source_key = $3
         ON CONFLICT (league_id, source_key) DO UPDATE SET
           name = EXCLUDED.name,
           updated_at = CURRENT_TIMESTAMP",
    )
    .bind(name)
    .bind(source_key)
    .bind(league_source_key)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn upsert_teams_and_fixture(
    pool: &PgPool,
    home_team_name: &str,
    home_team_source_key: &str,
    away_team_name: &str,
    away_team_source_key: &str,
    division_source_key: &str,
    scheduled_at: &str,
    fixture_source_key: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "WITH upserted_home_team AS (
           INSERT INTO team (division_id, name, source_key)
           SELECT division.division_id, $1, $2
           FROM division
           WHERE division.source_key = $5
           ON CONFLICT (division_id, source_key) DO UPDATE SET
             name = EXCLUDED.name,
             updated_at = CURRENT_TIMESTAMP
           RETURNING team_id
         ),
         upserted_away_team AS (
           INSERT INTO team (division_id, name, source_key)
           SELECT division.division_id, $3, $4
           FROM division
           WHERE division.source_key = $5
           ON CONFLICT (division_id, source_key) DO UPDATE SET
             name = EXCLUDED.name,
             updated_at = CURRENT_TIMESTAMP
           RETURNING team_id
         )
         INSERT INTO fixture (division_id, home_team_id, away_team_id, scheduled_at, source_key)
         SELECT
           division.division_id,
           upserted_home_team.team_id,
           upserted_away_team.team_id,
           $6::timestamp,
           $7
         FROM division, upserted_home_team, upserted_away_team
         WHERE division.source_key = $5
         ON CONFLICT (source_key) DO UPDATE SET
           scheduled_at = EXCLUDED.scheduled_at,
           home_team_id = EXCLUDED.home_team_id,
           away_team_id = EXCLUDED.away_team_id,
           updated_at = CURRENT_TIMESTAMP",
    )
    .bind(home_team_name)
    .bind(home_team_source_key)
    .bind(away_team_name)
    .bind(away_team_source_key)
    .bind(division_source_key)
    .bind(scheduled_at)
    .bind(fixture_source_key)
    .execute(pool)
    .await?;
    Ok(())
}
