use sqlx::PgPool;

#[derive(sqlx::FromRow)]
pub struct LeagueRow {
    pub league_id: i32,
    pub name: String,
}

#[derive(sqlx::FromRow)]
pub struct DivisionRow {
    pub division_id: i32,
    pub league_id: i32,
    pub name: String,
}

#[derive(sqlx::FromRow)]
pub struct TeamRow {
    pub team_id: i32,
    pub division_id: i32,
    pub name: String,
}

#[derive(sqlx::FromRow)]
pub struct FixtureRow {
    pub fixture_id: i32,
    pub home_team_id: i32,
    pub away_team_id: i32,
    pub home_team_name: String,
    pub away_team_name: String,
    pub scheduled_at: chrono::NaiveDateTime,
    pub status: String,
    pub venue_name: Option<String>,
    pub venue_address: Option<String>,
}

pub async fn connect(database_url: &str) -> anyhow::Result<PgPool> {
    Ok(PgPool::connect(database_url).await?)
}

pub async fn load_leagues(pool: &PgPool) -> anyhow::Result<Vec<LeagueRow>> {
    Ok(sqlx::query_as::<_, LeagueRow>(
        "SELECT league_id, name FROM league ORDER BY name",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn load_divisions(pool: &PgPool) -> anyhow::Result<Vec<DivisionRow>> {
    Ok(sqlx::query_as::<_, DivisionRow>(
        "SELECT division_id, league_id, name FROM division ORDER BY league_id, name",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn load_teams(pool: &PgPool) -> anyhow::Result<Vec<TeamRow>> {
    Ok(sqlx::query_as::<_, TeamRow>(
        "SELECT team_id, division_id, name FROM team ORDER BY division_id, name",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn load_fixtures(pool: &PgPool) -> anyhow::Result<Vec<FixtureRow>> {
    Ok(sqlx::query_as::<_, FixtureRow>(
        "SELECT
             fixture_id,
             home_team_id,
             away_team_id,
             home_team.name AS home_team_name,
             away_team.name AS away_team_name,
             scheduled_at,
             status,
             venue.name AS venue_name,
             venue.address AS venue_address
         FROM fixture
         JOIN team home_team ON home_team.team_id = fixture.home_team_id
         JOIN team away_team ON away_team.team_id = fixture.away_team_id
         JOIN division ON division.division_id = fixture.division_id
         JOIN league ON league.league_id = division.league_id
         LEFT JOIN venue ON venue.venue_id = league.venue_id
         ORDER BY scheduled_at",
    )
    .fetch_all(pool)
    .await?)
}
