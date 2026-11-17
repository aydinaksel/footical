use anyhow::{anyhow, bail};
use footical_fulltime::{
    parse_fixtures_page, parse_results_page, parse_team_page, FixtureListPage, PageKind,
    ParseIssue, Parsed, TeamPage,
};
use sqlx::SqlitePool;
use tracing::{event, Level};

use crate::source_page;

const FA_FULLTIME_SOURCE_ID: i32 = 2;
const BASE_URL: &str = "https://fulltime.thefa.com";
const ITEMS_PER_LIST_PAGE: usize = 100;
const CLOUDFLARE_BLOCK_MARKER: &str = "Cloudflare Ray ID";

const DIVISION: Division = Division {
    season_id: "158072627",
    division_season: "780926466",
    fixture_group_key: "1_780926466",
};
const TEAM_ID: &str = "296920867";
const FIRST_LIST_PAGE_NUMBER: u32 = 1;

struct Division {
    season_id: &'static str,
    division_season: &'static str,
    fixture_group_key: &'static str,
}

impl Division {
    fn list_url(&self, page: PageKind, page_number: u32) -> String {
        let Self {
            season_id,
            fixture_group_key,
            ..
        } = self;
        format!(
            "{BASE_URL}/{page}/{page_number}/{ITEMS_PER_LIST_PAGE}.html\
             ?selectedSeason={season_id}\
             &selectedFixtureGroupAgeGroup=0\
             &previousSelectedFixtureGroupAgeGroup=\
             &selectedFixtureGroupKey={fixture_group_key}\
             &previousSelectedFixtureGroupKey={fixture_group_key}\
             &selectedDateCode=\
             &selectedRelatedFixtureOption=3\
             &selectedClub=\
             &previousSelectedClub=\
             &selectedTeam="
        )
    }

    fn list_page_key(&self, page_number: u32) -> String {
        format!("{}:{page_number}", self.fixture_group_key)
    }

    fn team_url(&self, team_id: &str) -> String {
        format!(
            "{BASE_URL}/displayTeam.html?divisionseason={}&teamID={team_id}",
            self.division_season
        )
    }
}

#[derive(Debug, Clone)]
pub struct ManualPageLink {
    pub page_kind: String,
    pub url: String,
}

pub fn manual_page_links() -> Vec<ManualPageLink> {
    [
        (
            PageKind::Results,
            DIVISION.list_url(PageKind::Results, FIRST_LIST_PAGE_NUMBER),
        ),
        (
            PageKind::Fixtures,
            DIVISION.list_url(PageKind::Fixtures, FIRST_LIST_PAGE_NUMBER),
        ),
        (PageKind::Team, DIVISION.team_url(TEAM_ID)),
    ]
    .into_iter()
    .map(|(page_kind, url)| ManualPageLink {
        page_kind: page_kind.to_string(),
        url,
    })
    .collect()
}

#[derive(Debug, Clone)]
pub struct PastedPage {
    pub page_kind: String,
    pub fixtures_parsed: usize,
    pub players_parsed: usize,
    pub issues: Vec<String>,
}

enum PastedContent {
    FixtureList(PageKind, Parsed<FixtureListPage>),
    Team(Parsed<TeamPage>),
}

fn identify_pasted_page(html: &str) -> anyhow::Result<PastedContent> {
    if html.contains(CLOUDFLARE_BLOCK_MARKER) {
        bail!("This is Cloudflare's block page, not a Full-Time page.");
    }
    let recognised = [
        parse_fixtures_page(html)
            .ok()
            .map(|parsed| PastedContent::FixtureList(PageKind::Fixtures, parsed)),
        parse_results_page(html)
            .ok()
            .map(|parsed| PastedContent::FixtureList(PageKind::Results, parsed)),
        parse_team_page(html).ok().map(PastedContent::Team),
    ];
    let mut recognised = recognised.into_iter().flatten();
    let content = recognised
        .next()
        .ok_or_else(|| anyhow!("This is not a Full-Time fixtures, results or team page."))?;
    if recognised.next().is_some() {
        bail!("This page looks like more than one kind of Full-Time page.");
    }
    Ok(content)
}

fn describe_issues(issues: &[ParseIssue]) -> Vec<String> {
    issues.iter().map(ToString::to_string).collect()
}

pub async fn store_pasted_page(pool: &SqlitePool, html: &str) -> anyhow::Result<PastedPage> {
    let (page_kind, page_key, pasted_page) = match identify_pasted_page(html)? {
        PastedContent::FixtureList(page_kind, parsed) => (
            page_kind,
            DIVISION.list_page_key(FIRST_LIST_PAGE_NUMBER),
            PastedPage {
                page_kind: page_kind.to_string(),
                fixtures_parsed: parsed.content.rows.len(),
                players_parsed: 0,
                issues: describe_issues(&parsed.issues),
            },
        ),
        PastedContent::Team(parsed) => (
            PageKind::Team,
            parsed.content.team_id.clone(),
            PastedPage {
                page_kind: PageKind::Team.to_string(),
                fixtures_parsed: 0,
                players_parsed: parsed.content.player_names.len(),
                issues: describe_issues(&parsed.issues),
            },
        ),
    };
    source_page::store(
        pool,
        FA_FULLTIME_SOURCE_ID,
        &page_kind.to_string(),
        &page_key,
        html,
    )
    .await?;
    event!(
        name: "scrape.fulltime.page.pasted",
        Level::INFO,
        scrape.page.kind = %page_kind,
        scrape.page.key = page_key,
        scrape.page.issues = pasted_page.issues.len(),
        "stored a pasted {{scrape.page.kind}} page under {{scrape.page.key}} with {{scrape.page.issues}} parse issues",
    );
    Ok(pasted_page)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn results_url_matches_the_page_full_time_serves() {
        assert_eq!(
            DIVISION.list_url(PageKind::Results, 1),
            "https://fulltime.thefa.com/results/1/100.html?selectedSeason=158072627\
             &selectedFixtureGroupAgeGroup=0&previousSelectedFixtureGroupAgeGroup=\
             &selectedFixtureGroupKey=1_780926466&previousSelectedFixtureGroupKey=1_780926466\
             &selectedDateCode=&selectedRelatedFixtureOption=3&selectedClub=\
             &previousSelectedClub=&selectedTeam="
        );
    }

    #[test]
    fn team_url_matches_the_page_full_time_serves() {
        assert_eq!(
            DIVISION.team_url(TEAM_ID),
            "https://fulltime.thefa.com/displayTeam.html?divisionseason=780926466&teamID=296920867"
        );
    }

    const SAVED_FIXTURES_PAGE: &str = include_str!("../../fulltime/tests/pages/fixtures.html");
    const SAVED_RESULTS_PAGE: &str = include_str!("../../fulltime/tests/pages/results.html");
    const SAVED_TEAM_PAGE: &str = include_str!("../../fulltime/tests/pages/team.html");
    const SAVED_TABLE_PAGE: &str = include_str!("../../fulltime/tests/pages/table.html");

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

    #[tokio::test]
    async fn each_saved_page_is_stored_under_its_own_kind() {
        let pool = migrated_pool().await;
        for page in [SAVED_FIXTURES_PAGE, SAVED_RESULTS_PAGE, SAVED_TEAM_PAGE] {
            store_pasted_page(&pool, page).await.unwrap();
        }
        let stored: Vec<(String, String)> =
            sqlx::query_as("SELECT page_kind, page_key FROM source_page ORDER BY page_kind")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(
            stored,
            vec![
                ("fixtures".to_owned(), "1_780926466:1".to_owned()),
                ("results".to_owned(), "1_780926466:1".to_owned()),
                ("team".to_owned(), "296920867".to_owned()),
            ]
        );
    }

    #[tokio::test]
    async fn a_pasted_team_page_counts_its_players() {
        let pool = migrated_pool().await;
        let pasted = store_pasted_page(&pool, SAVED_TEAM_PAGE).await.unwrap();
        assert_eq!(pasted.page_kind, "team");
        assert!(pasted.players_parsed > 0);
    }

    #[tokio::test]
    async fn pages_that_are_not_wanted_are_refused() {
        let pool = migrated_pool().await;
        for (page, expected_reason) in [
            (
                SAVED_TABLE_PAGE,
                "not a Full-Time fixtures, results or team page",
            ),
            (
                "<html><body>Cloudflare Ray ID: 1</body></html>",
                "Cloudflare's block page",
            ),
        ] {
            let refusal = store_pasted_page(&pool, page).await.unwrap_err();
            assert!(refusal.to_string().contains(expected_reason), "{refusal}");
        }
        let stored: i64 = sqlx::query_scalar("SELECT count(*) FROM source_page")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(stored, 0);
    }
}
