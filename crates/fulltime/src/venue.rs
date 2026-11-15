use footical_snapshot::Coordinates;
use scraper::Html;

use crate::error::{IssueField, PageError, PageKind, ParseIssue, Parsed};
use crate::text::{element_text, query_value, selector};

const PAGE: PageKind = PageKind::Fixture;

#[derive(Debug, Clone, PartialEq)]
pub struct VenuePage {
    pub name: String,
    pub address: Option<String>,
    pub coordinates: Option<Coordinates>,
}

pub fn parse_fixture_page(html: &str) -> Result<Parsed<VenuePage>, PageError> {
    let document = Html::parse_document(html);
    let venue_name = selector(PAGE, "section#fixture-venue h2 span")?;
    let name = document
        .select(&venue_name)
        .map(element_text)
        .find(|text| !text.is_empty())
        .ok_or_else(|| PageError::missing_element(PAGE, "venue name"))?;
    let address_selector = selector(PAGE, "section#fixture-venue address")?;
    let address = document
        .select(&address_selector)
        .map(element_text)
        .find(|text| !text.is_empty());
    let map_link = selector(PAGE, "section#fixture-venue a[href*=\"maps.google.com\"]")?;
    let map_href = document
        .select(&map_link)
        .find_map(|link| link.value().attr("href"));
    let mut issues = Vec::new();
    let coordinates = map_href.and_then(|href| {
        let parsed = parse_coordinates(href);
        if parsed.is_none() {
            issues.push(ParseIssue::new(PAGE, &name, IssueField::Coordinates, href));
        }
        parsed
    });
    Ok(Parsed {
        content: VenuePage {
            name,
            address,
            coordinates,
        },
        issues,
    })
}

fn parse_coordinates(href: &str) -> Option<Coordinates> {
    let (latitude, longitude) = query_value(href, "ll")?.split_once(',')?;
    Some(Coordinates {
        latitude_degrees: latitude.trim().parse().ok()?,
        longitude_degrees: longitude.trim().parse().ok()?,
    })
}
