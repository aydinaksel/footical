use scraper::Html;

use crate::error::{PageError, PageKind, Parsed};
use crate::text::{element_text, query_value, selector};

const PAGE: PageKind = PageKind::Team;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamPage {
    pub team_id: String,
    pub team_name: String,
    pub player_names: Vec<String>,
}

pub fn parse_team_page(html: &str) -> Result<Parsed<TeamPage>, PageError> {
    let document = Html::parse_document(html);
    let heading = selector(PAGE, "div.team-header h1")?;
    let team_name = document
        .select(&heading)
        .map(element_text)
        .find(|text| !text.is_empty())
        .ok_or_else(|| PageError::missing_element(PAGE, "team heading"))?;
    let team_link = selector(PAGE, "a[href*=\"teamID=\"]")?;
    let team_id = document
        .select(&team_link)
        .filter_map(|link| link.value().attr("href"))
        .find_map(|href| query_value(href, "teamID"))
        .ok_or_else(|| PageError::missing_element(PAGE, "team link"))?
        .to_owned();
    let player_list = selector(PAGE, "div.tab-4 div.player-grid")?;
    if document.select(&player_list).next().is_none() {
        return Err(PageError::missing_element(PAGE, "player list"));
    }
    let player = selector(PAGE, "div.tab-4 div.player-grid p")?;
    let player_names = document
        .select(&player)
        .map(element_text)
        .filter(|name| !name.is_empty())
        .collect();
    Ok(Parsed {
        content: TeamPage {
            team_id,
            team_name,
            player_names,
        },
        issues: Vec::new(),
    })
}
