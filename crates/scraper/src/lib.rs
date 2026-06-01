mod fetch;
pub mod ical;
mod ingest;
mod parse;

use std::collections::HashSet;

use sqlx::PgPool;
use tracing::{Level, event};

const BASE_URL: &str = "https://footballmundial.com";

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScrapeResult {
    pub venues_upserted: usize,
    pub leagues_upserted: usize,
    pub divisions_upserted: usize,
    pub teams_upserted: usize,
    pub fixtures_upserted: usize,
    pub stale_fixtures_deleted: u64,
    pub duration_seconds: f64,
}

pub async fn run_scrape(pool: &PgPool) -> anyhow::Result<ScrapeResult> {
    let started_at = std::time::Instant::now();
    let fetcher = fetch::Fetcher::new()?;

    event!(
        name: "scrape.run.started",
        Level::INFO,
        scrape.source = "footballmundial.com",
        "scrape started for {{scrape.source}}",
    );

    let find_league_html = fetcher.fetch(&format!("{BASE_URL}/find_league")).await?;
    let league_group_paths = parse::parse_league_group_ids(&find_league_html);

    event!(
        name: "scrape.league_groups.discovered",
        Level::INFO,
        scrape.league_groups.count = league_group_paths.len(),
        "discovered {{scrape.league_groups.count}} league groups",
    );

    let mut all_venue_source_keys: HashSet<String> = HashSet::new();
    let mut all_league_groups = Vec::new();

    let league_group_count = league_group_paths.len();
    for (index, path) in league_group_paths.iter().enumerate() {
        let url = format!("{BASE_URL}{path}");
        let html = fetcher.fetch(&url).await?;
        let league_group = parse::parse_league_group(&html, path)?;

        event!(
            name: "scrape.league_group.fetched",
            Level::INFO,
            scrape.league_group.index = index + 1,
            scrape.league_group.total = league_group_count,
            scrape.league_group.name = league_group.league_group_name,
            "fetched league group {{scrape.league_group.index}}/{{scrape.league_group.total}}: {{scrape.league_group.name}}",
        );

        if let Some(venue_key) = &league_group.venue_source_key {
            all_venue_source_keys.insert(venue_key.clone());
        }

        all_league_groups.push(league_group);
    }

    let mut venues_upserted = 0;
    let venue_count = all_venue_source_keys.len();
    for venue_key in &all_venue_source_keys {
        let url = format!("{BASE_URL}/info/venues/{venue_key}");
        let html = fetcher.fetch(&url).await?;
        let venue = parse::parse_venue(&html);
        ingest::upsert_venue(pool, venue_key, &venue.name, venue.address.as_deref()).await?;
        venues_upserted += 1;

        event!(
            name: "scrape.venue.upserted",
            Level::INFO,
            scrape.venue.index = venues_upserted,
            scrape.venue.total = venue_count,
            scrape.venue.name = venue.name,
            "upserted venue {{scrape.venue.index}}/{{scrape.venue.total}}: {{scrape.venue.name}}",
        );
    }

    let mut leagues_upserted = 0;
    let mut divisions_upserted = 0;
    let mut all_league_ids_with_division_source_key: Vec<(String, String)> = Vec::new();

    for league_group in &all_league_groups {
        let league_source_key =
            format!("{BASE_URL}/info/leaguegroups/{}", league_group.league_group_id);
        let venue_endpoint = league_group
            .venue_source_key
            .as_ref()
            .map(|key| format!("{BASE_URL}/info/venues/{key}"))
            .unwrap_or_default();

        ingest::upsert_league(
            pool,
            &league_group.league_group_name,
            league_group.day_of_week,
            &league_source_key,
            league_group.number_of_players,
            league_group.starts_at.as_deref(),
            league_group.ends_at.as_deref(),
            league_group.price_pence,
            &venue_endpoint,
        )
        .await?;
        leagues_upserted += 1;

        for league_endpoint in &league_group.league_endpoints {
            let division_source_key =
                format!("{BASE_URL}/info/leagues/{}", league_endpoint.league_id);
            let division_name = league_endpoint
                .league_name
                .as_deref()
                .unwrap_or(&league_group.league_group_name);

            ingest::upsert_division(pool, division_name, &division_source_key, &league_source_key)
                .await?;
            divisions_upserted += 1;

            all_league_ids_with_division_source_key
                .push((league_endpoint.league_id.clone(), division_source_key));
        }
    }

    event!(
        name: "scrape.leagues.upserted",
        Level::INFO,
        scrape.leagues.count = leagues_upserted,
        scrape.divisions.count = divisions_upserted,
        "upserted {{scrape.leagues.count}} leagues and {{scrape.divisions.count}} divisions",
    );

    let mut teams_upserted = 0;
    let mut fixtures_upserted = 0;
    let mut stale_fixtures_deleted: u64 = 0;
    let division_count = all_league_ids_with_division_source_key.len();

    for (division_index, (league_id, division_source_key)) in
        all_league_ids_with_division_source_key.iter().enumerate()
    {
        let url = format!("{BASE_URL}/info/leagues/{league_id}");
        let html = fetcher.fetch(&url).await?;
        let fixture_data = parse::parse_league_fixtures(&html, league_id);

        event!(
            name: "scrape.division_fixtures.fetched",
            Level::INFO,
            scrape.division.index = division_index + 1,
            scrape.division.total = division_count,
            scrape.division.fixtures_count = fixture_data.len(),
            "fetched division fixtures {{scrape.division.index}}/{{scrape.division.total}}: {{scrape.division.fixtures_count}} fixtures",
        );

        let mut active_fixture_source_keys = Vec::with_capacity(fixture_data.len());

        for fixture in &fixture_data {
            let home_team_source_key =
                format!("{BASE_URL}/info/teams/{}", fixture.mundial_home_team_id);
            let away_team_source_key =
                format!("{BASE_URL}/info/teams/{}", fixture.mundial_away_team_id);
            let fixture_source_key = format!(
                "{}-{}-{}-{}",
                league_id,
                fixture.mundial_home_team_id,
                fixture.mundial_away_team_id,
                fixture.fixture_date
            );

            ingest::upsert_teams_and_fixture(
                pool,
                &fixture.mundial_home_team_name,
                &home_team_source_key,
                &fixture.mundial_away_team_name,
                &away_team_source_key,
                division_source_key,
                &fixture.fixture_date,
                &fixture_source_key,
            )
            .await?;

            active_fixture_source_keys.push(fixture_source_key);
            teams_upserted += 2;
            fixtures_upserted += 1;
        }

        let stale_deleted = ingest::delete_stale_fixtures(
            pool,
            division_source_key,
            &active_fixture_source_keys,
        )
        .await?;

        stale_fixtures_deleted += stale_deleted;

        if stale_deleted > 0 {
            event!(
                name: "scrape.fixtures.stale_deleted",
                Level::INFO,
                scrape.fixtures.stale_deleted = stale_deleted,
                scrape.division.source_key = division_source_key,
                "deleted {{scrape.fixtures.stale_deleted}} stale fixtures from division {{scrape.division.source_key}}",
            );
        }
    }

    event!(
        name: "scrape.fixtures.upserted",
        Level::INFO,
        scrape.teams.count = teams_upserted,
        scrape.fixtures.count = fixtures_upserted,
        "upserted {{scrape.teams.count}} teams and {{scrape.fixtures.count}} fixtures",
    );

    let duration_seconds = started_at.elapsed().as_secs_f64();

    event!(
        name: "scrape.run.completed",
        Level::INFO,
        scrape.duration_seconds = duration_seconds,
        scrape.venues.count = venues_upserted,
        scrape.leagues.count = leagues_upserted,
        scrape.divisions.count = divisions_upserted,
        scrape.fixtures.count = fixtures_upserted,
        scrape.stale_fixtures_deleted = stale_fixtures_deleted,
        "scrape completed in {{scrape.duration_seconds}}s: {{scrape.venues.count}} venues, {{scrape.leagues.count}} leagues, {{scrape.divisions.count}} divisions, {{scrape.fixtures.count}} fixtures, {{scrape.stale_fixtures_deleted}} stale deleted",
    );

    Ok(ScrapeResult {
        venues_upserted,
        leagues_upserted,
        divisions_upserted,
        teams_upserted,
        fixtures_upserted,
        stale_fixtures_deleted,
        duration_seconds,
    })
}
