mod fetch;
pub mod ical;
mod ingest;
mod parse;

use std::collections::HashSet;
use std::path::Path;

use sqlx::PgPool;

const BASE_URL: &str = "https://footballmundial.com";

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScrapeResult {
    pub venues_upserted: usize,
    pub leagues_upserted: usize,
    pub divisions_upserted: usize,
    pub teams_upserted: usize,
    pub fixtures_upserted: usize,
    pub duration_seconds: f64,
}

pub async fn run_scrape(
    pool: &PgPool,
    ical_output_directory: &Path,
) -> anyhow::Result<ScrapeResult> {
    let started_at = std::time::Instant::now();
    let fetcher = fetch::Fetcher::new()?;

    tracing::info!("starting scrape of footballmundial.com");

    let find_league_html = fetcher.fetch(&format!("{BASE_URL}/find_league")).await?;
    let league_group_paths = parse::parse_league_group_ids(&find_league_html);
    tracing::info!(count = league_group_paths.len(), "found league groups");

    let mut all_venue_source_keys: HashSet<String> = HashSet::new();
    let mut all_league_groups = Vec::new();

    for path in &league_group_paths {
        let url = format!("{BASE_URL}{path}");
        let html = fetcher.fetch(&url).await?;
        let league_group = parse::parse_league_group(&html, path)?;

        if let Some(venue_key) = &league_group.venue_source_key {
            all_venue_source_keys.insert(venue_key.clone());
        }

        all_league_groups.push(league_group);
    }

    let mut venues_upserted = 0;
    for venue_key in &all_venue_source_keys {
        let url = format!("{BASE_URL}/info/venues/{venue_key}");
        let html = fetcher.fetch(&url).await?;
        let venue = parse::parse_venue(&html);
        ingest::upsert_venue(pool, venue_key, &venue.name, venue.address.as_deref()).await?;
        venues_upserted += 1;
    }
    tracing::info!(venues_upserted, "upserted venues");

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
    tracing::info!(leagues_upserted, divisions_upserted, "upserted leagues and divisions");

    let mut teams_upserted = 0;
    let mut fixtures_upserted = 0;

    for (league_id, division_source_key) in &all_league_ids_with_division_source_key {
        let url = format!("{BASE_URL}/info/leagues/{league_id}");
        let html = fetcher.fetch(&url).await?;
        let fixture_data = parse::parse_league_fixtures(&html, league_id);

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

            teams_upserted += 2;
            fixtures_upserted += 1;
        }
    }
    tracing::info!(fixtures_upserted, "upserted teams and fixtures");

    ical::regenerate_icals(pool, ical_output_directory).await?;

    let duration_seconds = started_at.elapsed().as_secs_f64();
    tracing::info!(duration_seconds, "scrape completed");

    Ok(ScrapeResult {
        venues_upserted,
        leagues_upserted,
        divisions_upserted,
        teams_upserted,
        fixtures_upserted,
        duration_seconds,
    })
}
