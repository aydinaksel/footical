use scraper::{Html, Selector};

fn compile_selector(css: &str) -> anyhow::Result<Selector> {
    Selector::parse(css).map_err(|error| anyhow::anyhow!("invalid CSS selector {css}: {error}"))
}

fn parse_price_pence(text: &str) -> Option<i32> {
    let digits: String = text
        .chars()
        .filter(|character| character.is_ascii_digit() || *character == '.')
        .collect();
    let (pounds_text, pence_text) = match digits.split_once('.') {
        Some((pounds, pence)) => (pounds, pence),
        None => (digits.as_str(), ""),
    };
    let pounds = pounds_text.parse::<i32>().ok()?;
    let pence = match pence_text.len() {
        0 => 0,
        1 => pence_text.parse::<i32>().ok()?.checked_mul(10)?,
        _ => pence_text.get(..2)?.parse::<i32>().ok()?,
    };
    pounds.checked_mul(100)?.checked_add(pence)
}

pub struct LeagueGroupData {
    pub league_group_name: String,
    pub league_group_id: String,
    pub league_endpoints: Vec<LeagueEndpoint>,
    pub venue_source_key: Option<String>,
    pub number_of_players: Option<i32>,
    pub day_of_week: Option<i32>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub price_pence: Option<i32>,
}

pub struct LeagueEndpoint {
    pub league_id: String,
    pub league_name: Option<String>,
}

pub struct VenueData {
    pub name: String,
    pub address: Option<String>,
}

pub struct FixtureData {
    pub fixture_date: String,
    pub mundial_home_team_id: i32,
    pub mundial_home_team_name: String,
    pub mundial_away_team_id: i32,
    pub mundial_away_team_name: String,
}

pub fn parse_league_group_ids(html: &str) -> anyhow::Result<Vec<String>> {
    let document = Html::parse_document(html);
    let selector = compile_selector("option")?;

    let league_group_paths = document
        .select(&selector)
        .filter_map(|element| element.value().attr("value"))
        .filter(|value| value.starts_with("/info/leaguegroups/"))
        .map(String::from)
        .collect();
    Ok(league_group_paths)
}

pub fn parse_league_group(html: &str, league_group_path: &str) -> anyhow::Result<LeagueGroupData> {
    let document = Html::parse_document(html);

    let league_group_id = league_group_path
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_owned();

    let heading_selector = compile_selector("h1")?;
    let league_group_name = document
        .select(&heading_selector)
        .next()
        .map(|element| element.text().collect::<String>().trim().to_uppercase())
        .unwrap_or_default();

    let link_selector = compile_selector("a")?;
    let mut league_endpoints = Vec::new();
    let mut venue_source_key = None;

    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            if href.starts_with("/info/leagues/") {
                let league_id = href.rsplit('/').next().unwrap_or("").to_owned();
                league_endpoints.push(LeagueEndpoint {
                    league_id,
                    league_name: None,
                });
            } else if href.starts_with("/info/venues/") {
                venue_source_key = href.rsplit('/').next().map(String::from);
            }
        }
    }

    let panel_title_selector = compile_selector("h4.panel-title")?;
    let league_name_regex =
        regex::Regex::new(r"^(.+?)\s+View Fixtures & Results\s+\[/info/leagues/(\d+)\]")?;

    for element in document.select(&panel_title_selector) {
        let text = element.text().collect::<String>();
        if let Some(captures) = league_name_regex.captures(&text) {
            let Some(league_name) = captures
                .get(1)
                .map(|group| group.as_str().trim().to_uppercase())
            else {
                continue;
            };
            let Some(league_id) = captures.get(2).map(|group| group.as_str().to_owned()) else {
                continue;
            };

            if let Some(endpoint) = league_endpoints
                .iter_mut()
                .find(|endpoint| endpoint.league_id == league_id)
            {
                endpoint.league_name = Some(league_name);
            } else {
                league_endpoints.push(LeagueEndpoint {
                    league_id,
                    league_name: Some(league_name),
                });
            }
        }
    }

    let day_regex = regex::Regex::new(r"Day:</b>\s*([^<\n]+)")?;
    let time_regex = regex::Regex::new(r"Game Times:</b>\s*([^<\n]+)")?;
    let price_regex = regex::Regex::new(r"Game Price:</b>\s*([\d.,£\s]+)")?;
    let players_regex = regex::Regex::new(r"Number of Players:</b>\s*(\d+)")?;

    let day_map = |name: &str| -> Option<i32> {
        match name.trim() {
            "Monday" => Some(1),
            "Tuesday" => Some(2),
            "Wednesday" => Some(3),
            "Thursday" => Some(4),
            "Friday" => Some(5),
            "Saturday" => Some(6),
            "Sunday" => Some(7),
            _ => None,
        }
    };

    let day_of_week = day_regex
        .captures(html)
        .and_then(|captures| day_map(captures.get(1)?.as_str()));

    let (starts_at, ends_at) = time_regex
        .captures(html)
        .map(|captures| {
            let Some(times) = captures.get(1) else {
                return (None, None);
            };
            let parts: Vec<&str> = times.as_str().split('-').collect();
            (
                parts.first().map(|part| part.trim().to_owned()),
                parts.get(1).map(|part| part.trim().to_owned()),
            )
        })
        .unwrap_or((None, None));

    let price_pence = price_regex
        .captures(html)
        .and_then(|captures| parse_price_pence(captures.get(1)?.as_str()));

    let number_of_players = players_regex
        .captures(html)
        .and_then(|captures| captures.get(1)?.as_str().parse().ok());

    Ok(LeagueGroupData {
        league_group_name,
        league_group_id,
        league_endpoints,
        venue_source_key,
        number_of_players,
        day_of_week,
        starts_at,
        ends_at,
        price_pence,
    })
}

pub fn parse_venue(html: &str) -> anyhow::Result<VenueData> {
    let document = Html::parse_document(html);

    let heading_selector = compile_selector("h2")?;
    let name = document
        .select(&heading_selector)
        .next()
        .map(|element| element.text().collect::<String>().trim().to_uppercase())
        .unwrap_or_default();

    let address_selector = compile_selector(
        "body > div.container_fluid > div.container > div.panel-body > div > div > div > div.col-lg-2.col-md-3.col-sm-3.col-xs-12",
    )?;

    let address = document.select(&address_selector).next().map(|element| {
        let text = element.text().collect::<String>();
        text.split("\n\n")
            .filter(|paragraph| !paragraph.is_empty() && *paragraph != "Address")
            .collect::<Vec<_>>()
            .join(", ")
            .to_uppercase()
    });

    Ok(VenueData { name, address })
}

pub fn parse_league_fixtures(html: &str, _league_id: &str) -> anyhow::Result<Vec<FixtureData>> {
    let document = Html::parse_document(html);
    let group_selector =
        compile_selector("#fixtures_accordion_fixtures > div > div:not(.panel-heading)")?;
    let row_selector = compile_selector("table > tbody > tr")?;
    let link_selector = compile_selector("a[href]")?;
    let date_regex = regex::Regex::new(r"\d{4}-\d{2}-\d{2}")?;
    let time_regex = regex::Regex::new(r"^\d{2}:\d{2}$")?;
    let team_id_regex = regex::Regex::new(r"/info/teams/(\d+)")?;

    let mut fixtures = Vec::new();

    for group in document.select(&group_selector) {
        let group_id = group.value().attr("id").unwrap_or("");
        let date = match date_regex.find(group_id) {
            Some(matched) => matched.as_str(),
            None => continue,
        };

        for row in group.select(&row_selector) {
            let text = row.text().collect::<String>();
            let tokens: Vec<&str> = text.split_whitespace().collect();

            if !tokens.contains(&"v") {
                continue;
            }

            let match_time = tokens
                .iter()
                .find(|token| time_regex.is_match(token))
                .unwrap_or(&"00:00");

            let team_links: Vec<(i32, String)> = row
                .select(&link_selector)
                .filter_map(|link| {
                    let href = link.value().attr("href")?;
                    let team_id = team_id_regex
                        .captures(href)?
                        .get(1)?
                        .as_str()
                        .parse()
                        .ok()?;
                    let team_name = link.text().collect::<String>().trim().to_owned();
                    Some((team_id, team_name))
                })
                .collect();

            if let (Some(home_team), Some(away_team)) = (team_links.first(), team_links.get(1)) {
                fixtures.push(FixtureData {
                    fixture_date: format!("{date}T{match_time}:00"),
                    mundial_home_team_id: home_team.0,
                    mundial_home_team_name: home_team.1.clone(),
                    mundial_away_team_id: away_team.0,
                    mundial_away_team_name: away_team.1.clone(),
                });
            }
        }
    }

    Ok(fixtures)
}
