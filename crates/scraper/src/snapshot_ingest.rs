use std::collections::{HashMap, HashSet};

use anyhow::anyhow;
use chrono::NaiveDateTime;
use footical_snapshot::{
    Competition, CompetitionKind, Fixture, Outcome, Score, Side, Snapshot, Team, Venue,
};
use sqlx::SqliteConnection;

const SCHEDULED_AT_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

pub struct SnapshotTarget<'target> {
    pub source_id: i32,
    pub league_source_key: &'target str,
    pub league_name: &'target str,
    pub belongs_to_league: fn(&Competition) -> bool,
    pub removes_unlisted_fixtures_after: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IngestCounts {
    pub teams_written: usize,
    pub fixtures_written: usize,
    pub fixtures_removed: u64,
}

struct KeyedIds<'snapshot> {
    competitions: HashMap<&'snapshot str, i64>,
    teams: HashMap<&'snapshot str, i64>,
    venues: HashMap<&'snapshot str, i64>,
}

#[derive(Default)]
struct OutcomeColumns {
    status: &'static str,
    home_goals: Option<i64>,
    away_goals: Option<i64>,
    half_time_home_goals: Option<i64>,
    half_time_away_goals: Option<i64>,
    penalty_home_goals: Option<i64>,
    penalty_away_goals: Option<i64>,
    walkover_winner: Option<&'static str>,
    unrecognised_outcome: Option<String>,
}

fn home_goals(score: Option<Score>) -> Option<i64> {
    score.map(|score| i64::from(score.home_goals))
}

fn away_goals(score: Option<Score>) -> Option<i64> {
    score.map(|score| i64::from(score.away_goals))
}

fn outcome_columns(outcome: &Outcome) -> OutcomeColumns {
    match outcome {
        Outcome::Scheduled => OutcomeColumns {
            status: "scheduled",
            ..OutcomeColumns::default()
        },
        Outcome::Postponed => OutcomeColumns {
            status: "postponed",
            ..OutcomeColumns::default()
        },
        Outcome::Played(score) => OutcomeColumns {
            status: "played",
            home_goals: home_goals(Some(score.full_time)),
            away_goals: away_goals(Some(score.full_time)),
            half_time_home_goals: home_goals(score.half_time),
            half_time_away_goals: away_goals(score.half_time),
            penalty_home_goals: home_goals(score.penalties),
            penalty_away_goals: away_goals(score.penalties),
            ..OutcomeColumns::default()
        },
        Outcome::Walkover { winner } => OutcomeColumns {
            status: "walkover",
            walkover_winner: Some(match winner {
                Side::Home => "home",
                Side::Away => "away",
            }),
            ..OutcomeColumns::default()
        },
        Outcome::Unrecognised { raw } => OutcomeColumns {
            status: "unrecognised",
            unrecognised_outcome: Some(raw.clone()),
            ..OutcomeColumns::default()
        },
    }
}

fn competition_kind(kind: CompetitionKind) -> &'static str {
    match kind {
        CompetitionKind::League => "league",
        CompetitionKind::Cup => "cup",
    }
}

fn team_competitions(snapshot: &Snapshot) -> HashMap<&str, &str> {
    let league_competitions: HashSet<&str> = snapshot
        .competitions
        .iter()
        .filter(|competition| competition.kind == CompetitionKind::League)
        .map(|competition| competition.key.as_str())
        .collect();
    let appearances = snapshot.fixtures.iter().flat_map(|fixture| {
        [
            (
                fixture.home_team_key.as_str(),
                fixture.competition_key.as_str(),
            ),
            (
                fixture.away_team_key.as_str(),
                fixture.competition_key.as_str(),
            ),
        ]
    });
    let mut chosen: HashMap<&str, &str> = HashMap::new();
    for (team_key, competition_key) in appearances {
        let current = chosen.entry(team_key).or_insert(competition_key);
        if !league_competitions.contains(*current) && league_competitions.contains(competition_key)
        {
            *current = competition_key;
        }
    }
    chosen
}

async fn upsert_league(
    connection: &mut SqliteConnection,
    target: &SnapshotTarget<'_>,
) -> anyhow::Result<i64> {
    Ok(sqlx::query_scalar(
        "INSERT INTO league (source_id, name, source_key)
         VALUES (?, ?, ?)
         ON CONFLICT (source_id, source_key) DO UPDATE SET
           name = excluded.name,
           updated_at = CURRENT_TIMESTAMP
         RETURNING league_id",
    )
    .bind(target.source_id)
    .bind(target.league_name)
    .bind(target.league_source_key)
    .fetch_one(connection)
    .await?)
}

async fn upsert_competition(
    connection: &mut SqliteConnection,
    source_id: i32,
    league_id: Option<i64>,
    competition: &Competition,
) -> anyhow::Result<i64> {
    Ok(sqlx::query_scalar(
        "INSERT INTO competition (source_id, league_id, name, kind, source_key)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT (source_id, source_key) DO UPDATE SET
           league_id = excluded.league_id,
           name = excluded.name,
           kind = excluded.kind,
           updated_at = CURRENT_TIMESTAMP
         RETURNING competition_id",
    )
    .bind(source_id)
    .bind(league_id)
    .bind(&competition.name)
    .bind(competition_kind(competition.kind))
    .bind(&competition.key)
    .fetch_one(connection)
    .await?)
}

async fn upsert_venue(
    connection: &mut SqliteConnection,
    source_id: i32,
    venue: &Venue,
) -> anyhow::Result<i64> {
    Ok(sqlx::query_scalar(
        "INSERT INTO venue (source_id, source_key, name, address, latitude_degrees, longitude_degrees)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT (source_id, source_key) DO UPDATE SET
           name = excluded.name,
           address = COALESCE(excluded.address, venue.address),
           latitude_degrees = COALESCE(excluded.latitude_degrees, venue.latitude_degrees),
           longitude_degrees = COALESCE(excluded.longitude_degrees, venue.longitude_degrees),
           updated_at = CURRENT_TIMESTAMP
         RETURNING venue_id",
    )
    .bind(source_id)
    .bind(&venue.key)
    .bind(&venue.name)
    .bind(venue.address.as_deref())
    .bind(venue.coordinates.map(|coordinates| coordinates.latitude_degrees))
    .bind(venue.coordinates.map(|coordinates| coordinates.longitude_degrees))
    .fetch_one(connection)
    .await?)
}

async fn upsert_team(
    connection: &mut SqliteConnection,
    source_id: i32,
    competition_id: Option<i64>,
    team: &Team,
) -> anyhow::Result<i64> {
    Ok(sqlx::query_scalar(
        "INSERT INTO team (source_id, competition_id, name, source_key)
         VALUES (?, ?, ?, ?)
         ON CONFLICT (source_id, source_key) DO UPDATE SET
           competition_id = COALESCE(excluded.competition_id, team.competition_id),
           name = excluded.name,
           updated_at = CURRENT_TIMESTAMP
         RETURNING team_id",
    )
    .bind(source_id)
    .bind(competition_id)
    .bind(&team.name)
    .bind(&team.key)
    .fetch_one(connection)
    .await?)
}

fn known_id(ids: &HashMap<&str, i64>, key: &str, fixture: &Fixture) -> anyhow::Result<i64> {
    ids.get(key).copied().ok_or_else(|| {
        anyhow!(
            "fixture {} refers to {key}, which is not in the snapshot",
            fixture.key
        )
    })
}

async fn write_fixture(
    connection: &mut SqliteConnection,
    source_id: i32,
    ids: &KeyedIds<'_>,
    fixture: &Fixture,
) -> anyhow::Result<()> {
    let competition_id = known_id(&ids.competitions, &fixture.competition_key, fixture)?;
    let home_team_id = known_id(&ids.teams, &fixture.home_team_key, fixture)?;
    let away_team_id = known_id(&ids.teams, &fixture.away_team_key, fixture)?;
    let venue_id = fixture
        .venue_key
        .as_deref()
        .map(|key| known_id(&ids.venues, key, fixture))
        .transpose()?;
    let outcome = outcome_columns(&fixture.outcome);
    sqlx::query(
        "INSERT INTO fixture (source_id, competition_id, home_team_id, away_team_id, venue_id,
                              scheduled_at, status, home_goals, away_goals,
                              half_time_home_goals, half_time_away_goals,
                              penalty_home_goals, penalty_away_goals,
                              walkover_winner, unrecognised_outcome, source_key)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT (source_id, source_key) DO UPDATE SET
           competition_id = excluded.competition_id,
           home_team_id = excluded.home_team_id,
           away_team_id = excluded.away_team_id,
           venue_id = COALESCE(excluded.venue_id, fixture.venue_id),
           scheduled_at = excluded.scheduled_at,
           status = excluded.status,
           home_goals = excluded.home_goals,
           away_goals = excluded.away_goals,
           half_time_home_goals = excluded.half_time_home_goals,
           half_time_away_goals = excluded.half_time_away_goals,
           penalty_home_goals = excluded.penalty_home_goals,
           penalty_away_goals = excluded.penalty_away_goals,
           walkover_winner = excluded.walkover_winner,
           unrecognised_outcome = excluded.unrecognised_outcome,
           updated_at = CURRENT_TIMESTAMP",
    )
    .bind(source_id)
    .bind(competition_id)
    .bind(home_team_id)
    .bind(away_team_id)
    .bind(venue_id)
    .bind(fixture.kicks_off_at.format(SCHEDULED_AT_FORMAT).to_string())
    .bind(outcome.status)
    .bind(outcome.home_goals)
    .bind(outcome.away_goals)
    .bind(outcome.half_time_home_goals)
    .bind(outcome.half_time_away_goals)
    .bind(outcome.penalty_home_goals)
    .bind(outcome.penalty_away_goals)
    .bind(outcome.walkover_winner)
    .bind(outcome.unrecognised_outcome)
    .bind(&fixture.key)
    .execute(connection)
    .await?;
    Ok(())
}

async fn remove_unlisted_fixtures(
    connection: &mut SqliteConnection,
    source_id: i32,
    cutoff: NaiveDateTime,
    snapshot: &Snapshot,
) -> anyhow::Result<u64> {
    if snapshot.fixtures.is_empty() {
        return Ok(0);
    }
    let placeholders = std::iter::repeat_n("?", snapshot.fixtures.len())
        .collect::<Vec<_>>()
        .join(", ");
    let statement = format!(
        "UPDATE fixture
         SET status = 'removed', updated_at = CURRENT_TIMESTAMP
         WHERE source_id = ?
           AND status <> 'removed'
           AND scheduled_at >= ?
           AND source_key NOT IN ({placeholders})"
    );
    let mut query = sqlx::query(sqlx::AssertSqlSafe(statement))
        .bind(source_id)
        .bind(cutoff.format(SCHEDULED_AT_FORMAT).to_string());
    for fixture in &snapshot.fixtures {
        query = query.bind(&fixture.key);
    }
    Ok(query.execute(connection).await?.rows_affected())
}

pub async fn ingest_snapshot(
    connection: &mut SqliteConnection,
    target: &SnapshotTarget<'_>,
    snapshot: &Snapshot,
) -> anyhow::Result<IngestCounts> {
    let league_id = upsert_league(connection, target).await?;
    let mut ids = KeyedIds {
        competitions: HashMap::new(),
        teams: HashMap::new(),
        venues: HashMap::new(),
    };
    for competition in &snapshot.competitions {
        let league_id = (target.belongs_to_league)(competition).then_some(league_id);
        let competition_id =
            upsert_competition(connection, target.source_id, league_id, competition).await?;
        ids.competitions.insert(&competition.key, competition_id);
    }
    for venue in &snapshot.venues {
        let venue_id = upsert_venue(connection, target.source_id, venue).await?;
        ids.venues.insert(&venue.key, venue_id);
    }
    let competition_of_team = team_competitions(snapshot);
    for team in &snapshot.teams {
        let competition_id = competition_of_team
            .get(team.key.as_str())
            .and_then(|key| ids.competitions.get(key))
            .copied();
        let team_id = upsert_team(connection, target.source_id, competition_id, team).await?;
        ids.teams.insert(&team.key, team_id);
    }
    for fixture in &snapshot.fixtures {
        write_fixture(connection, target.source_id, &ids, fixture).await?;
    }
    let fixtures_removed = match target.removes_unlisted_fixtures_after {
        Some(cutoff) => {
            remove_unlisted_fixtures(connection, target.source_id, cutoff, snapshot).await?
        }
        None => 0,
    };
    Ok(IngestCounts {
        teams_written: snapshot.teams.len(),
        fixtures_written: snapshot.fixtures.len(),
        fixtures_removed,
    })
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use footical_snapshot::PlayedScore;

    use super::*;

    const SOURCE_ID: i32 = 2;
    const DIVISION_KEY: &str = "group:1_100";
    const COUNTY_CUP_KEY: &str = "county-competition:SENIOR CUP";

    fn not_county(competition: &Competition) -> bool {
        competition.key != COUNTY_CUP_KEY
    }

    fn kick_off(day: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, day)
            .and_then(|date| date.and_hms_opt(14, 0, 0))
            .unwrap()
    }

    fn fixture(key: &str, competition_key: &str, day: u32, outcome: Outcome) -> Fixture {
        Fixture {
            key: key.to_owned(),
            competition_key: competition_key.to_owned(),
            home_team_key: "team:1".to_owned(),
            away_team_key: "team-name:DRINGHOUSES".to_owned(),
            kicks_off_at: kick_off(day),
            venue_key: Some("venue:CLIFTON PARK".to_owned()),
            outcome,
        }
    }

    fn snapshot(fixtures: Vec<Fixture>) -> Snapshot {
        Snapshot {
            competitions: vec![
                Competition {
                    key: DIVISION_KEY.to_owned(),
                    name: "Division One".to_owned(),
                    kind: CompetitionKind::League,
                },
                Competition {
                    key: COUNTY_CUP_KEY.to_owned(),
                    name: "Senior Cup".to_owned(),
                    kind: CompetitionKind::Cup,
                },
            ],
            teams: vec![
                Team {
                    key: "team:1".to_owned(),
                    name: "Wigginton Grasshoppers".to_owned(),
                },
                Team {
                    key: "team-name:DRINGHOUSES".to_owned(),
                    name: "Dringhouses".to_owned(),
                },
            ],
            venues: vec![Venue {
                key: "venue:CLIFTON PARK".to_owned(),
                name: "Clifton Park".to_owned(),
                address: None,
                coordinates: None,
            }],
            fixtures,
            ..Snapshot::default()
        }
    }

    fn target(cutoff: Option<NaiveDateTime>) -> SnapshotTarget<'static> {
        SnapshotTarget {
            source_id: SOURCE_ID,
            league_source_key: "league:4051434",
            league_name: "York Football League",
            belongs_to_league: not_county,
            removes_unlisted_fixtures_after: cutoff,
        }
    }

    async fn migrated_pool() -> sqlx::SqlitePool {
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

    async fn ingest(pool: &sqlx::SqlitePool, target: &SnapshotTarget<'_>, snapshot: &Snapshot) {
        let mut connection = pool.acquire().await.unwrap();
        ingest_snapshot(&mut connection, target, snapshot)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn a_played_fixture_keeps_its_score_and_its_league() {
        let pool = migrated_pool().await;
        let played = Outcome::Played(PlayedScore {
            full_time: Score {
                home_goals: 3,
                away_goals: 1,
            },
            half_time: None,
            penalties: None,
        });
        ingest(
            &pool,
            &target(None),
            &snapshot(vec![fixture("fixture:1", DIVISION_KEY, 7, played)]),
        )
        .await;
        let row: (String, i64, i64, String, String) = sqlx::query_as(
            "SELECT fixture.status, fixture.home_goals, fixture.away_goals,
                    fixture.scheduled_at, league.name
             FROM fixture
             JOIN competition ON competition.competition_id = fixture.competition_id
             JOIN league ON league.league_id = competition.league_id",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            row,
            (
                "played".to_owned(),
                3,
                1,
                "2026-11-07T14:00:00".to_owned(),
                "York Football League".to_owned()
            )
        );
    }

    #[tokio::test]
    async fn a_team_belongs_to_its_league_competition_before_a_cup() {
        let pool = migrated_pool().await;
        let fixtures = vec![
            fixture("county-fixture:9", COUNTY_CUP_KEY, 7, Outcome::Scheduled),
            fixture("fixture:1", DIVISION_KEY, 14, Outcome::Scheduled),
        ];
        ingest(&pool, &target(None), &snapshot(fixtures)).await;
        let competition_names: Vec<String> = sqlx::query_scalar(
            "SELECT competition.name FROM team
             JOIN competition ON competition.competition_id = team.competition_id
             ORDER BY team.name",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(competition_names, vec!["Division One", "Division One"]);
    }

    #[tokio::test]
    async fn only_upcoming_fixtures_missing_from_the_pages_are_removed() {
        let pool = migrated_pool().await;
        let everything = vec![
            fixture("fixture:1", DIVISION_KEY, 7, Outcome::Scheduled),
            fixture("fixture:2", DIVISION_KEY, 21, Outcome::Scheduled),
            fixture("fixture:3", DIVISION_KEY, 28, Outcome::Scheduled),
        ];
        ingest(&pool, &target(None), &snapshot(everything)).await;
        let only_last = vec![fixture("fixture:3", DIVISION_KEY, 28, Outcome::Scheduled)];
        ingest(&pool, &target(Some(kick_off(14))), &snapshot(only_last)).await;
        let statuses: Vec<(String, String)> =
            sqlx::query_as("SELECT source_key, status FROM fixture ORDER BY source_key")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(
            statuses,
            vec![
                ("fixture:1".to_owned(), "scheduled".to_owned()),
                ("fixture:2".to_owned(), "removed".to_owned()),
                ("fixture:3".to_owned(), "scheduled".to_owned()),
            ]
        );
    }
}
