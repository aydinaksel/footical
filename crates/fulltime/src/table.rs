use scraper::{ElementRef, Html, Selector};

use crate::error::{IssueField, PageError, PageKind, ParseIssue, Parsed};
use crate::text::{element_text, first_href, first_text, query_value, selector};

const PAGE: PageKind = PageKind::Table;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow {
    pub team_id: String,
    pub team_name: String,
    pub position: u32,
    pub played: u32,
    pub won: u32,
    pub drawn: u32,
    pub lost: u32,
    pub goal_difference: i32,
    pub points: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TablePage {
    pub competition_name: String,
    pub division_season: String,
    pub rows: Vec<TableRow>,
}

struct TableSelectors {
    cell: Selector,
    team_link: Selector,
}

pub fn parse_table_page(html: &str) -> Result<Parsed<TablePage>, PageError> {
    let document = Html::parse_document(html);
    let heading = selector(PAGE, "div.tab-1 h2")?;
    let competition_name = document
        .select(&heading)
        .map(element_text)
        .find(|text| !text.is_empty())
        .ok_or_else(|| PageError::missing_element(PAGE, "competition heading"))?;
    let row_selector = selector(PAGE, "div.tab-1 table.cell-dividers tbody > tr")?;
    let selectors = TableSelectors {
        cell: selector(PAGE, "td")?,
        team_link: selector(PAGE, "a[href*=\"teamID=\"]")?,
    };
    let mut issues = Vec::new();
    let rows: Vec<TableRow> = document
        .select(&row_selector)
        .filter_map(|row| {
            let parsed = table_row(row, &selectors);
            if parsed.is_none() {
                issues.push(ParseIssue::new(
                    PAGE,
                    &competition_name,
                    IssueField::Standing,
                    &element_text(row),
                ));
            }
            parsed
        })
        .collect();
    let division_season = document
        .select(&row_selector)
        .find_map(|row| first_href(row, &selectors.team_link))
        .and_then(|href| query_value(href, "divisionseason"))
        .ok_or_else(|| PageError::missing_element(PAGE, "team link with a division season"))?
        .to_owned();
    Ok(Parsed {
        content: TablePage {
            competition_name,
            division_season,
            rows,
        },
        issues,
    })
}

fn table_row(row: ElementRef<'_>, selectors: &TableSelectors) -> Option<TableRow> {
    let href = first_href(row, &selectors.team_link)?;
    let team_id = query_value(href, "teamID")?.to_owned();
    let team_name = first_text(row, &selectors.team_link)?;
    let cells: Vec<String> = row.select(&selectors.cell).map(element_text).collect();
    let number = |index: usize| cells.get(index)?.parse::<i32>().ok();
    let count = |index: usize| cells.get(index)?.parse::<u32>().ok();
    Some(TableRow {
        team_id,
        team_name,
        position: count(0)?,
        played: count(2)?,
        won: count(3)?,
        drawn: count(4)?,
        lost: count(5)?,
        goal_difference: number(6)?,
        points: number(7)?,
    })
}
