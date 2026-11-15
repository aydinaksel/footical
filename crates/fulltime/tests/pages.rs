use chrono::NaiveDate;
use footical_fulltime::{
    assemble, parse_fixture_page, parse_fixtures_page, parse_results_page, parse_table_page,
    parse_team_page, FixtureListPage, PageKind, Pages, Parsed,
};
use footical_snapshot::{CompetitionKind, Fixture, Outcome, PlayedScore, Score, Side, Snapshot};

const FIXTURES_PAGE: &str = include_str!("pages/fixtures.html");
const RESULTS_PAGE: &str = include_str!("pages/results.html");
const TABLE_PAGE: &str = include_str!("pages/table.html");
const TEAM_PAGE: &str = include_str!("pages/team.html");
const FIXTURE_PAGE: &str = include_str!("pages/fixture.html");

const WIGGINTON: &str = "team:296920867";

fn score(home_goals: u32, away_goals: u32) -> Score {
    Score {
        home_goals,
        away_goals,
    }
}

fn without_issues<T>(parsed: Parsed<T>) -> Result<T, Box<dyn std::error::Error>> {
    if parsed.issues.is_empty() {
        return Ok(parsed.content);
    }
    Err(format!("unexpected parse issues: {:?}", parsed.issues).into())
}

fn fixture_lists() -> Result<Vec<FixtureListPage>, Box<dyn std::error::Error>> {
    Ok(vec![
        without_issues(parse_fixtures_page(FIXTURES_PAGE)?)?,
        without_issues(parse_results_page(RESULTS_PAGE)?)?,
    ])
}

fn snapshot() -> Result<Snapshot, Box<dyn std::error::Error>> {
    let lists = fixture_lists()?;
    let tables = [without_issues(parse_table_page(TABLE_PAGE)?)?];
    let teams = [without_issues(parse_team_page(TEAM_PAGE)?)?];
    let venues = [without_issues(parse_fixture_page(FIXTURE_PAGE)?)?];
    without_issues(assemble(Pages {
        fixture_lists: &lists,
        tables: &tables,
        teams: &teams,
        venues: &venues,
    }))
}

fn fixture<'snapshot>(
    snapshot: &'snapshot Snapshot,
    key: &str,
) -> Result<&'snapshot Fixture, Box<dyn std::error::Error>> {
    snapshot
        .fixtures
        .iter()
        .find(|fixture| fixture.key == key)
        .ok_or_else(|| format!("no fixture {key}").into())
}

#[test]
fn fixtures_page_lists_every_unplayed_fixture_with_its_venue() {
    let page = parse_fixtures_page(FIXTURES_PAGE).unwrap().content;
    assert_eq!(page.rows.len(), 76);
    let postponed = page
        .rows
        .iter()
        .filter(|row| row.outcome == Outcome::Postponed);
    assert_eq!(postponed.count(), 25);
    assert!(page.rows.iter().all(|row| row.venue_name.is_some()));
    assert_eq!(page.rows.iter().filter(|row| row.is_county_cup).count(), 2);

    let tadcaster = page
        .rows
        .iter()
        .find(|row| row.fixture_key == "fixture:29961404")
        .unwrap();
    assert_eq!(tadcaster.home_team_name, "WIGGINTON GRASSHOPPERS Reserves");
    assert_eq!(tadcaster.away_team_name, "Tadcaster Magnets Reserves");
    assert_eq!(
        tadcaster.kicks_off_at,
        NaiveDate::from_ymd_opt(2026, 11, 21)
            .unwrap()
            .and_hms_opt(14, 0, 0)
            .unwrap()
    );
    assert_eq!(
        tadcaster.venue_name.as_deref(),
        Some("THE WIGGINTON SPORTS AND PLAYING FIELDS 11 v 11 Pitch 2")
    );
    assert_eq!(tadcaster.outcome, Outcome::Postponed);
}

#[test]
fn results_page_reads_every_score_format() {
    let page = parse_results_page(RESULTS_PAGE).unwrap().content;
    assert_eq!(page.rows.len(), 82);
    assert_eq!(page.rows.iter().filter(|row| row.is_county_cup).count(), 10);
    let outcome = |key: &str| {
        page.rows
            .iter()
            .find(|row| row.fixture_key == key)
            .map(|row| row.outcome.clone())
            .unwrap()
    };
    assert_eq!(
        outcome("fixture:31077372"),
        Outcome::Played(PlayedScore {
            full_time: score(2, 3),
            half_time: Some(score(1, 1)),
            penalties: None,
        })
    );
    assert_eq!(
        outcome("county-fixture:31017902"),
        Outcome::Played(PlayedScore {
            full_time: score(2, 2),
            half_time: Some(score(2, 2)),
            penalties: Some(score(2, 4)),
        })
    );
    assert_eq!(outcome("fixture:29961348"), Outcome::Postponed);
    assert_eq!(
        outcome("county-fixture:30966700"),
        Outcome::Walkover { winner: Side::Home }
    );
    assert_eq!(
        outcome("county-fixture:30311997"),
        Outcome::Walkover { winner: Side::Away }
    );
}

#[test]
fn table_page_gives_team_ids_and_standings() {
    let table = parse_table_page(TABLE_PAGE).unwrap().content;
    assert_eq!(table.competition_name, "Men’s Reserve A");
    assert_eq!(table.division_season, "780926466");
    assert_eq!(table.rows.len(), 11);
    let leader = table.rows.first().unwrap();
    assert_eq!(leader.team_id, "30192270");
    assert_eq!(leader.team_name, "POPPLETON FC Poppleton FC reserves");
    assert_eq!(
        (leader.position, leader.played, leader.won, leader.points),
        (1, 7, 7, 21)
    );
    let wigginton = table.rows.last().unwrap();
    assert_eq!(wigginton.team_id, "296920867");
    assert_eq!(
        (wigginton.drawn, wigginton.lost, wigginton.goal_difference),
        (2, 4, -6)
    );
}

#[test]
fn team_page_lists_the_squad() {
    let team = parse_team_page(TEAM_PAGE).unwrap().content;
    assert_eq!(team.team_id, "296920867");
    assert_eq!(team.team_name, "WIGGINTON GRASSHOPPERS Reserves");
    assert_eq!(team.player_names.len(), 53);
    assert!(team.player_names.iter().any(|name| name == "ethan hurren"));
}

#[test]
fn fixture_page_gives_the_venue_address_and_location() {
    let venue = parse_fixture_page(FIXTURE_PAGE).unwrap().content;
    assert_eq!(venue.name, "HOWDEN SCHOOL 11 v 11 Pitch 1 PM");
    assert_eq!(
        venue.address.as_deref(),
        Some("DERWENT ROAD, HOWDEN, GOOLE DN14 7AL")
    );
    let coordinates = venue.coordinates.unwrap();
    assert!((coordinates.latitude_degrees - 53.748_604).abs() < 0.000_001);
    assert!((coordinates.longitude_degrees + 0.860_55).abs() < 0.000_001);
}

#[test]
fn a_page_of_the_wrong_kind_is_rejected_not_read_as_empty() {
    let error = parse_results_page(TABLE_PAGE).unwrap_err();
    assert!(error.is_missing_element());
    assert_eq!(error.page(), PageKind::Results);
    assert!(parse_fixtures_page(RESULTS_PAGE)
        .unwrap_err()
        .is_missing_element());
    assert!(parse_table_page(FIXTURE_PAGE)
        .unwrap_err()
        .is_missing_element());
    assert!(parse_team_page(TABLE_PAGE)
        .unwrap_err()
        .is_missing_element());
}

#[test]
fn postponed_fixtures_on_both_pages_merge_and_keep_their_venue() {
    let snapshot = snapshot().unwrap();
    let osbaldwick = fixture(&snapshot, "fixture:29961016").unwrap();
    assert_eq!(osbaldwick.outcome, Outcome::Postponed);
    assert_eq!(osbaldwick.home_team_key, WIGGINTON);
    assert_eq!(
        osbaldwick.venue_key.as_deref(),
        Some("venue:THE WIGGINTON SPORTS AND PLAYING FIELDS 11 V 11 PITCH 2")
    );
    let keys = snapshot.fixtures.iter().map(|fixture| fixture.key.as_str());
    assert_eq!(
        keys.clone().count(),
        keys.collect::<std::collections::HashSet<_>>().len()
    );
}

#[test]
fn division_teams_resolve_to_ids_and_cup_opponents_to_names() {
    let snapshot = snapshot().unwrap();
    let scalby = fixture(&snapshot, "county-fixture:30311993").unwrap();
    assert_eq!(scalby.home_team_key, "team-name:SCALBY FIRST");
    assert_eq!(scalby.away_team_key, WIGGINTON);
    let division_team_keys = snapshot
        .teams
        .iter()
        .filter(|team| team.key.starts_with("team:"))
        .count();
    assert_eq!(division_team_keys, 11);
}

#[test]
fn competitions_come_from_groups_and_county_cups() {
    let snapshot = snapshot().unwrap();
    let competition = |key: &str| {
        snapshot
            .competitions
            .iter()
            .find(|competition| competition.key == key)
            .unwrap_or_else(|| panic!("no competition {key}"))
    };
    assert_eq!(
        competition("group:1_780926466").kind,
        CompetitionKind::League
    );
    assert_eq!(
        competition("group:2_451165387").name,
        "York FA Cup Reserves (Sat)"
    );
    assert_eq!(competition("group:2_451165387").kind, CompetitionKind::Cup);
    assert_eq!(
        competition("county-competition:NORTH RIDING SATURDAY CHALLENGE CUP").kind,
        CompetitionKind::Cup
    );
    assert_eq!(
        fixture(&snapshot, "fixture:31010390")
            .unwrap()
            .competition_key,
        "group:2_451165387"
    );
}

#[test]
fn venue_details_attach_to_the_venue_used_by_fixtures() {
    let snapshot = snapshot().unwrap();
    let howden = snapshot
        .venues
        .iter()
        .find(|venue| venue.key == "venue:HOWDEN SCHOOL 11 V 11 PITCH 1 PM")
        .unwrap();
    assert_eq!(
        howden.address.as_deref(),
        Some("DERWENT ROAD, HOWDEN, GOOLE DN14 7AL")
    );
    assert!(howden.coordinates.is_some());
}

#[test]
fn the_squad_belongs_to_wigginton() {
    let snapshot = snapshot().unwrap();
    assert_eq!(snapshot.players.len(), 53);
    assert!(snapshot
        .players
        .iter()
        .all(|player| player.team_key == WIGGINTON));
}
