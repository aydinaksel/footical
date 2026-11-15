use sqlx::{SqliteConnection, SqlitePool};

const FOOTBALL_MUNDIAL_SOURCE_ID: i32 = 1;

pub async fn upsert_venue(
    pool: &SqlitePool,
    source_key: &str,
    name: &str,
    address: Option<&str>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO venue (source_id, source_key, name, address)
         VALUES (?, ?, ?, ?)
         ON CONFLICT (source_id, source_key) DO UPDATE SET
           name = excluded.name,
           address = excluded.address,
           updated_at = CURRENT_TIMESTAMP",
    )
    .bind(FOOTBALL_MUNDIAL_SOURCE_ID)
    .bind(source_key)
    .bind(name)
    .bind(address)
    .execute(pool)
    .await?;
    Ok(())
}

pub struct League<'a> {
    pub name: &'a str,
    pub day_of_week: Option<i32>,
    pub source_key: &'a str,
    pub number_of_players: Option<i32>,
    pub starts_at: Option<&'a str>,
    pub ends_at: Option<&'a str>,
    pub price_pence: Option<i32>,
    pub venue_source_key: &'a str,
}

pub async fn upsert_league(pool: &SqlitePool, league: &League<'_>) -> anyhow::Result<()> {
    let League {
        name,
        day_of_week,
        source_key,
        number_of_players,
        starts_at,
        ends_at,
        price_pence,
        venue_source_key,
    } = league;
    sqlx::query(
        "INSERT INTO league (source_id, venue_id, name, day_of_week, source_key,
                             number_of_players, starts_at, ends_at, price_pence)
         SELECT venue.source_id, venue.venue_id, ?, ?, ?, ?, ?, ?, ?
         FROM venue
         WHERE venue.source_id = ? AND venue.source_key = ?
         ON CONFLICT (source_id, source_key) DO UPDATE SET
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
    .bind(FOOTBALL_MUNDIAL_SOURCE_ID)
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
        "INSERT INTO competition (source_id, league_id, name, kind, source_key)
         SELECT league.source_id, league.league_id, ?, 'league', ?
         FROM league
         WHERE league.source_id = ? AND league.source_key = ?
         ON CONFLICT (source_id, source_key) DO UPDATE SET
           league_id = excluded.league_id,
           name = excluded.name,
           updated_at = CURRENT_TIMESTAMP",
    )
    .bind(name)
    .bind(source_key)
    .bind(FOOTBALL_MUNDIAL_SOURCE_ID)
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
        "INSERT INTO team (source_id, competition_id, name, source_key)
         SELECT competition.source_id, competition.competition_id, ?, ?
         FROM competition
         WHERE competition.source_id = ? AND competition.source_key = ?
         ON CONFLICT (source_id, source_key) DO UPDATE SET
           competition_id = excluded.competition_id,
           name = excluded.name,
           updated_at = CURRENT_TIMESTAMP
         RETURNING team_id",
    )
    .bind(name)
    .bind(format!("{division_source_key}|{source_key}"))
    .bind(FOOTBALL_MUNDIAL_SOURCE_ID)
    .bind(division_source_key)
    .fetch_one(connection)
    .await?;
    Ok(team_id)
}

pub struct Fixture<'a> {
    pub home_team_name: &'a str,
    pub home_team_source_key: &'a str,
    pub away_team_name: &'a str,
    pub away_team_source_key: &'a str,
    pub division_source_key: &'a str,
    pub scheduled_at: &'a str,
    pub source_key: &'a str,
}

pub async fn upsert_teams_and_fixture(
    pool: &SqlitePool,
    fixture: &Fixture<'_>,
) -> anyhow::Result<()> {
    let Fixture {
        home_team_name,
        home_team_source_key,
        away_team_name,
        away_team_source_key,
        division_source_key,
        scheduled_at,
        source_key: fixture_source_key,
    } = fixture;

    let mut transaction = pool.begin().await?;

    let home_team_id = upsert_team(
        &mut transaction,
        division_source_key,
        home_team_name,
        home_team_source_key,
    )
    .await?;
    let away_team_id = upsert_team(
        &mut transaction,
        division_source_key,
        away_team_name,
        away_team_source_key,
    )
    .await?;

    sqlx::query(
        "INSERT INTO fixture (source_id, competition_id, home_team_id, away_team_id, venue_id,
                              scheduled_at, source_key)
         SELECT competition.source_id, competition.competition_id, ?, ?, league.venue_id, ?, ?
         FROM competition
         JOIN league ON league.league_id = competition.league_id
         WHERE competition.source_id = ? AND competition.source_key = ?
         ON CONFLICT (source_id, source_key) DO UPDATE SET
           scheduled_at = excluded.scheduled_at,
           home_team_id = excluded.home_team_id,
           away_team_id = excluded.away_team_id,
           venue_id = excluded.venue_id,
           updated_at = CURRENT_TIMESTAMP",
    )
    .bind(home_team_id)
    .bind(away_team_id)
    .bind(scheduled_at)
    .bind(fixture_source_key)
    .bind(FOOTBALL_MUNDIAL_SOURCE_ID)
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
             WHERE competition_id = (
                 SELECT competition_id FROM competition WHERE source_id = ? AND source_key = ?
             )",
        )
        .bind(FOOTBALL_MUNDIAL_SOURCE_ID)
        .bind(division_source_key)
        .execute(pool)
        .await?;
        return Ok(result.rows_affected());
    }

    let placeholders = std::iter::repeat_n("?", active_fixture_source_keys.len())
        .collect::<Vec<_>>()
        .join(", ");
    let statement = format!(
        "DELETE FROM fixture
         WHERE competition_id = (
             SELECT competition_id FROM competition WHERE source_id = ? AND source_key = ?
         )
           AND source_key NOT IN ({placeholders})"
    );

    let mut query = sqlx::query(sqlx::AssertSqlSafe(statement))
        .bind(FOOTBALL_MUNDIAL_SOURCE_ID)
        .bind(division_source_key);
    for source_key in active_fixture_source_keys {
        query = query.bind(source_key);
    }

    let result = query.execute(pool).await?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VENUE: &str = "https://footballmundial.com/info/venues/3";
    const LEAGUE: &str = "https://footballmundial.com/info/leaguegroups/5";
    const DIVISION: &str = "https://footballmundial.com/info/leagues/9";

    async fn migrated_pool() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("../website/migrations")
            .run(&pool)
            .await
            .unwrap();
        pool
    }

    async fn ingest_one_division(pool: &SqlitePool, fixture_dates: &[&str]) {
        upsert_venue(pool, VENUE, "York Sports Village", Some("Hull Road"))
            .await
            .unwrap();
        upsert_league(
            pool,
            &League {
                name: "Tuesday 6-a-side",
                day_of_week: Some(2),
                source_key: LEAGUE,
                number_of_players: Some(6),
                starts_at: None,
                ends_at: None,
                price_pence: Some(3200),
                venue_source_key: VENUE,
            },
        )
        .await
        .unwrap();
        upsert_division(pool, "Division 1", DIVISION, LEAGUE)
            .await
            .unwrap();
        for date in fixture_dates {
            upsert_teams_and_fixture(
                pool,
                &Fixture {
                    home_team_name: "Ball Bags",
                    home_team_source_key: "https://footballmundial.com/info/teams/100",
                    away_team_name: "Net Gains",
                    away_team_source_key: "https://footballmundial.com/info/teams/101",
                    division_source_key: DIVISION,
                    scheduled_at: date,
                    source_key: &format!("9-100-101-{date}"),
                },
            )
            .await
            .unwrap();
        }
    }

    async fn team_ids(pool: &SqlitePool) -> Vec<i32> {
        sqlx::query_scalar("SELECT team_id FROM team ORDER BY team_id")
            .fetch_all(pool)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn fixtures_land_in_the_shared_tables_with_the_league_venue() {
        let pool = migrated_pool().await;
        ingest_one_division(&pool, &["2026-11-17T19:00:00"]).await;
        let (competition_kind, venue_name, status): (String, String, String) = sqlx::query_as(
            "SELECT competition.kind, venue.name, fixture.status
             FROM fixture
             JOIN competition ON competition.competition_id = fixture.competition_id
             JOIN venue ON venue.venue_id = fixture.venue_id",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(competition_kind, "league");
        assert_eq!(venue_name, "York Sports Village");
        assert_eq!(status, "scheduled");
    }

    #[tokio::test]
    async fn ingesting_again_keeps_team_ids_stable() {
        let pool = migrated_pool().await;
        ingest_one_division(&pool, &["2026-11-17T19:00:00"]).await;
        let first_ids = team_ids(&pool).await;
        ingest_one_division(&pool, &["2026-11-17T19:00:00", "2026-11-24T19:00:00"]).await;
        assert_eq!(team_ids(&pool).await, first_ids);
        let fixture_count: i32 = sqlx::query_scalar("SELECT count(*) FROM fixture")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(fixture_count, 2);
    }

    #[tokio::test]
    async fn fixtures_missing_from_a_scrape_are_deleted() {
        let pool = migrated_pool().await;
        ingest_one_division(&pool, &["2026-11-17T19:00:00", "2026-11-24T19:00:00"]).await;
        let deleted = delete_stale_fixtures(
            &pool,
            DIVISION,
            &["9-100-101-2026-11-24T19:00:00".to_owned()],
        )
        .await
        .unwrap();
        assert_eq!(deleted, 1);
    }
}
