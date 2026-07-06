use sqlx::{SqliteConnection, SqlitePool};

pub async fn upsert_venue(
    pool: &SqlitePool,
    source_key: &str,
    name: &str,
    address: Option<&str>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO venue (source_key, name, address)
         VALUES (?, ?, ?)
         ON CONFLICT (source_key) DO UPDATE SET
           name = excluded.name,
           address = excluded.address,
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
    pool: &SqlitePool,
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
         SELECT 1, venue.venue_id, ?, ?, ?, ?, ?, ?, ?
         FROM venue
         WHERE venue.source_key = ?
         ON CONFLICT (organisation_id, source_key) DO UPDATE SET
           name = excluded.name,
           day_of_week = excluded.day_of_week,
           venue_id = excluded.venue_id,
           number_of_players = excluded.number_of_players,
           starts_at = excluded.starts_at,
           ends_at = excluded.ends_at,
           price_pence = excluded.price_pence,
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
    pool: &SqlitePool,
    name: &str,
    source_key: &str,
    league_source_key: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO division (league_id, name, source_key)
         SELECT league.league_id, ?, ?
         FROM league
         WHERE league.source_key = ?
         ON CONFLICT (league_id, source_key) DO UPDATE SET
           name = excluded.name,
           updated_at = CURRENT_TIMESTAMP",
    )
    .bind(name)
    .bind(source_key)
    .bind(league_source_key)
    .execute(pool)
    .await?;
    Ok(())
}

async fn upsert_team(
    connection: &mut SqliteConnection,
    division_source_key: &str,
    name: &str,
    source_key: &str,
) -> anyhow::Result<i32> {
    let team_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO team (division_id, name, source_key)
         SELECT division.division_id, ?, ?
         FROM division
         WHERE division.source_key = ?
         ON CONFLICT (division_id, source_key) DO UPDATE SET
           name = excluded.name,
           updated_at = CURRENT_TIMESTAMP
         RETURNING team_id",
    )
    .bind(name)
    .bind(source_key)
    .bind(division_source_key)
    .fetch_one(connection)
    .await?;
    Ok(team_id)
}

pub async fn upsert_teams_and_fixture(
    pool: &SqlitePool,
    home_team_name: &str,
    home_team_source_key: &str,
    away_team_name: &str,
    away_team_source_key: &str,
    division_source_key: &str,
    scheduled_at: &str,
    fixture_source_key: &str,
) -> anyhow::Result<()> {
    let mut transaction = pool.begin().await?;

    let home_team_id =
        upsert_team(&mut transaction, division_source_key, home_team_name, home_team_source_key)
            .await?;
    let away_team_id =
        upsert_team(&mut transaction, division_source_key, away_team_name, away_team_source_key)
            .await?;

    sqlx::query(
        "INSERT INTO fixture (division_id, home_team_id, away_team_id, scheduled_at, source_key)
         SELECT division.division_id, ?, ?, ?, ?
         FROM division
         WHERE division.source_key = ?
         ON CONFLICT (source_key) DO UPDATE SET
           scheduled_at = excluded.scheduled_at,
           home_team_id = excluded.home_team_id,
           away_team_id = excluded.away_team_id,
           updated_at = CURRENT_TIMESTAMP",
    )
    .bind(home_team_id)
    .bind(away_team_id)
    .bind(scheduled_at)
    .bind(fixture_source_key)
    .bind(division_source_key)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    Ok(())
}

pub async fn delete_stale_fixtures(
    pool: &SqlitePool,
    division_source_key: &str,
    active_fixture_source_keys: &[String],
) -> anyhow::Result<u64> {
    if active_fixture_source_keys.is_empty() {
        let result = sqlx::query(
            "DELETE FROM fixture
             WHERE division_id = (SELECT division_id FROM division WHERE source_key = ?)",
        )
        .bind(division_source_key)
        .execute(pool)
        .await?;
        return Ok(result.rows_affected());
    }

    let placeholders = std::iter::repeat("?")
        .take(active_fixture_source_keys.len())
        .collect::<Vec<_>>()
        .join(", ");
    let statement = format!(
        "DELETE FROM fixture
         WHERE division_id = (SELECT division_id FROM division WHERE source_key = ?)
           AND source_key NOT IN ({placeholders})"
    );

    let mut query = sqlx::query(sqlx::AssertSqlSafe(statement)).bind(division_source_key);
    for source_key in active_fixture_source_keys {
        query = query.bind(source_key);
    }

    let result = query.execute(pool).await?;
    Ok(result.rows_affected())
}
