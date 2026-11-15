use chrono::NaiveDateTime;
use footical_snapshot::Outcome;
use scraper::{ElementRef, Html, Selector};

use crate::error::{IssueField, PageError, PageKind, ParseIssue, Parsed};
use crate::outcome::{parse_fixture_status, parse_result_score};
use crate::text::{element_text, first_text, query_value, selector};

const KICK_OFF_FORMAT: &str = "%d/%m/%y %H:%M";
const COUNTY_FIXTURE_PAGE: &str = "displayCountyFixture.html";
const FIXTURE_PAGE: &str = "displayFixture.html";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompetitionGroup {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureRow {
    pub fixture_key: String,
    pub is_county_cup: bool,
    pub competition_name: String,
    pub kicks_off_at: NaiveDateTime,
    pub home_team_name: String,
    pub away_team_name: String,
    pub venue_name: Option<String>,
    pub outcome: Outcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureListPage {
    pub page: PageKind,
    pub groups: Vec<CompetitionGroup>,
    pub rows: Vec<FixtureRow>,
}

struct RowCells {
    fixture_key: String,
    is_county_cup: bool,
    kick_off_text: String,
    home_team_name: String,
    away_team_name: String,
    competition_name: String,
    venue_name: Option<String>,
    outcome_text: String,
}

struct FixturesSelectors {
    row: Selector,
    cell: Selector,
    link: Selector,
}

struct ResultsSelectors {
    row: Selector,
    link: Selector,
    date_time: Selector,
    team_name: Selector,
    score: Selector,
    competition: Selector,
}

pub fn parse_fixtures_page(html: &str) -> Result<Parsed<FixtureListPage>, PageError> {
    let page = PageKind::Fixtures;
    let document = Html::parse_document(html);
    require_element(&document, page, "div.fixtures-table", "fixtures table")?;
    let selectors = FixturesSelectors {
        row: selector(page, "div.fixtures-table tbody > tr")?,
        cell: selector(page, "td")?,
        link: selector(page, "a[href]")?,
    };
    let rows = document
        .select(&selectors.row)
        .map(|row| fixtures_row_cells(row, &selectors));
    build_page(&document, page, rows)
}

pub fn parse_results_page(html: &str) -> Result<Parsed<FixtureListPage>, PageError> {
    let page = PageKind::Results;
    let document = Html::parse_document(html);
    require_element(&document, page, "div.results-table-2", "results table")?;
    let selectors = ResultsSelectors {
        row: selector(
            page,
            "div.results-table-2 div.tbody > div[id^=\"fixture-\"]",
        )?,
        link: selector(page, "a[href]")?,
        date_time: selector(page, "div.datetime-col")?,
        team_name: selector(page, "div.team-name")?,
        score: selector(page, "div.score-col")?,
        competition: selector(page, "div.fg-col")?,
    };
    let rows = document
        .select(&selectors.row)
        .map(|row| results_row_cells(row, &selectors));
    build_page(&document, page, rows)
}

fn require_element(
    document: &Html,
    page: PageKind,
    css: &'static str,
    element: &'static str,
) -> Result<(), PageError> {
    let container = selector(page, css)?;
    document
        .select(&container)
        .next()
        .map(|_| ())
        .ok_or_else(|| PageError::missing_element(page, element))
}

fn build_page(
    document: &Html,
    page: PageKind,
    rows: impl Iterator<Item = Result<RowCells, String>>,
) -> Result<Parsed<FixtureListPage>, PageError> {
    let groups = competition_groups(document, page)?;
    let mut issues = Vec::new();
    let mut parsed_rows = Vec::new();
    for cells in rows {
        match cells {
            Ok(cells) => parsed_rows.extend(fixture_row(page, cells, &mut issues)),
            Err(raw) => issues.push(ParseIssue::new(page, "fixture row", IssueField::Row, &raw)),
        }
    }
    Ok(Parsed {
        content: FixtureListPage {
            page,
            groups,
            rows: parsed_rows,
        },
        issues,
    })
}

fn competition_groups(document: &Html, page: PageKind) -> Result<Vec<CompetitionGroup>, PageError> {
    let options = selector(page, "select[name=\"selectedFixtureGroupKey\"] option")?;
    let groups: Vec<CompetitionGroup> = document
        .select(&options)
        .filter_map(|option| {
            let key = option.value().attr("value")?.trim();
            let name = element_text(option);
            (!key.is_empty()).then(|| CompetitionGroup {
                key: key.to_owned(),
                name,
            })
        })
        .collect();
    if groups.is_empty() {
        return Err(PageError::missing_element(page, "competition group list"));
    }
    Ok(groups)
}

fn fixtures_row_cells(
    row: ElementRef<'_>,
    selectors: &FixturesSelectors,
) -> Result<RowCells, String> {
    let cells: Vec<String> = row.select(&selectors.cell).map(element_text).collect();
    let cell = |index: usize| cells.get(index).cloned().unwrap_or_default();
    let (fixture_key, is_county_cup) =
        row_fixture_key(row, &selectors.link).ok_or_else(|| element_text(row))?;
    let venue_name = cell(7);
    Ok(RowCells {
        fixture_key,
        is_county_cup,
        kick_off_text: cell(1),
        home_team_name: cell(2),
        away_team_name: cell(6),
        competition_name: cell(8),
        venue_name: (!venue_name.is_empty()).then_some(venue_name),
        outcome_text: cell(9),
    })
}

fn results_row_cells(
    row: ElementRef<'_>,
    selectors: &ResultsSelectors,
) -> Result<RowCells, String> {
    let (fixture_key, is_county_cup) =
        row_fixture_key(row, &selectors.link).ok_or_else(|| element_text(row))?;
    let mut team_names = row.select(&selectors.team_name).map(element_text);
    Ok(RowCells {
        fixture_key,
        is_county_cup,
        kick_off_text: first_text(row, &selectors.date_time).unwrap_or_default(),
        home_team_name: team_names.next().unwrap_or_default(),
        away_team_name: team_names.next().unwrap_or_default(),
        competition_name: first_text(row, &selectors.competition).unwrap_or_default(),
        venue_name: None,
        outcome_text: first_text(row, &selectors.score).unwrap_or_default(),
    })
}

fn row_fixture_key(row: ElementRef<'_>, link: &Selector) -> Option<(String, bool)> {
    row.select(link)
        .filter_map(|anchor| anchor.value().attr("href"))
        .find_map(fixture_key_from_href)
}

fn fixture_key_from_href(href: &str) -> Option<(String, bool)> {
    let identifier = query_value(href, "id")?;
    if href.contains(COUNTY_FIXTURE_PAGE) {
        return Some((format!("county-fixture:{identifier}"), true));
    }
    href.contains(FIXTURE_PAGE)
        .then(|| (format!("fixture:{identifier}"), false))
}

fn fixture_row(
    page: PageKind,
    cells: RowCells,
    issues: &mut Vec<ParseIssue>,
) -> Option<FixtureRow> {
    let subject = cells.fixture_key.as_str();
    let has_both_teams = !cells.home_team_name.is_empty() && !cells.away_team_name.is_empty();
    if !has_both_teams || cells.competition_name.is_empty() {
        issues.push(ParseIssue::new(
            page,
            subject,
            IssueField::Row,
            &cells.home_team_name,
        ));
        return None;
    }
    let Ok(kicks_off_at) = NaiveDateTime::parse_from_str(&cells.kick_off_text, KICK_OFF_FORMAT)
    else {
        issues.push(ParseIssue::new(
            page,
            subject,
            IssueField::KickOff,
            &cells.kick_off_text,
        ));
        return None;
    };
    let outcome = row_outcome(page, &cells.outcome_text).unwrap_or_else(|| {
        issues.push(ParseIssue::new(
            page,
            subject,
            IssueField::Outcome,
            &cells.outcome_text,
        ));
        Outcome::Unrecognised {
            raw: cells.outcome_text.clone(),
        }
    });
    Some(FixtureRow {
        fixture_key: cells.fixture_key,
        is_county_cup: cells.is_county_cup,
        competition_name: cells.competition_name,
        kicks_off_at,
        home_team_name: cells.home_team_name,
        away_team_name: cells.away_team_name,
        venue_name: cells.venue_name,
        outcome,
    })
}

fn row_outcome(page: PageKind, text: &str) -> Option<Outcome> {
    match page {
        PageKind::Results => parse_result_score(text),
        _ => parse_fixture_status(text),
    }
}
