use std::collections::{BTreeMap, HashMap};

use footical_snapshot::{
    Competition, CompetitionKind, Fixture, Outcome, Player, Snapshot, Standing, Team, Venue,
};

use crate::error::{IssueField, PageKind, ParseIssue, Parsed};
use crate::fixture_list::{CompetitionGroup, FixtureListPage, FixtureRow};
use crate::table::TablePage;
use crate::team::TeamPage;
use crate::text::normalised_name;
use crate::venue::VenuePage;

const LEAGUE_GROUP_PREFIX: &str = "1_";
const CUP_GROUP_PREFIX: &str = "2_";
const COUNTY_COMPETITION_KEY_PREFIX: &str = "county-competition:";
const TEAM_NAME_KEY_PREFIX: &str = "team-name:";
pub fn team_name_key(name: &str) -> String {
    format!("{TEAM_NAME_KEY_PREFIX}{}", normalised_name(name))
}

pub fn is_county_competition(competition: &Competition) -> bool {
    competition.key.starts_with(COUNTY_COMPETITION_KEY_PREFIX)
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Pages<'pages> {
    pub fixture_lists: &'pages [FixtureListPage],
    pub tables: &'pages [TablePage],
    pub teams: &'pages [TeamPage],
    pub venues: &'pages [VenuePage],
}

pub fn assemble(pages: Pages<'_>) -> Parsed<Snapshot> {
    let mut builder = SnapshotBuilder::default();
    pages
        .tables
        .iter()
        .for_each(|table| builder.add_table(table));
    pages
        .teams
        .iter()
        .for_each(|team| builder.add_team_page(team));
    pages
        .fixture_lists
        .iter()
        .flat_map(|list| &list.groups)
        .for_each(|group| builder.add_group(group));
    pages
        .venues
        .iter()
        .for_each(|venue| builder.add_venue_page(venue));
    for page in [PageKind::Fixtures, PageKind::Results] {
        pages
            .fixture_lists
            .iter()
            .filter(|list| list.page == page)
            .flat_map(|list| &list.rows)
            .for_each(|row| builder.add_fixture_row(page, row));
    }
    builder.finish()
}

#[derive(Default)]
struct SnapshotBuilder {
    competitions: BTreeMap<String, Competition>,
    teams: BTreeMap<String, Team>,
    venues: BTreeMap<String, Venue>,
    fixtures: BTreeMap<String, Fixture>,
    standings: Vec<Standing>,
    players: Vec<Player>,
    team_keys_by_name: HashMap<String, String>,
    groups_by_name: HashMap<String, Competition>,
    issues: Vec<ParseIssue>,
}

impl SnapshotBuilder {
    fn add_table(&mut self, table: &TablePage) {
        let competition_key = format!("group:{LEAGUE_GROUP_PREFIX}{}", table.division_season);
        self.add_competition(Competition {
            key: competition_key.clone(),
            name: table.competition_name.clone(),
            kind: CompetitionKind::League,
        });
        for row in &table.rows {
            let team_key = self.register_team_id(&row.team_id, &row.team_name);
            self.standings.push(Standing {
                competition_key: competition_key.clone(),
                team_key,
                position: row.position,
                played: row.played,
                won: row.won,
                drawn: row.drawn,
                lost: row.lost,
                goal_difference: row.goal_difference,
                points: row.points,
            });
        }
    }

    fn add_team_page(&mut self, team: &TeamPage) {
        let team_key = self.register_team_id(&team.team_id, &team.team_name);
        self.players
            .extend(team.player_names.iter().map(|name| Player {
                team_key: team_key.clone(),
                name: name.clone(),
            }));
    }

    fn add_group(&mut self, group: &CompetitionGroup) {
        let kind = if group.key.starts_with(LEAGUE_GROUP_PREFIX) {
            CompetitionKind::League
        } else if group.key.starts_with(CUP_GROUP_PREFIX) {
            CompetitionKind::Cup
        } else {
            self.issues.push(ParseIssue::new(
                PageKind::Fixtures,
                &group.name,
                IssueField::Competition,
                &group.key,
            ));
            return;
        };
        self.groups_by_name.insert(
            normalised_name(&group.name),
            Competition {
                key: format!("group:{}", group.key),
                name: group.name.clone(),
                kind,
            },
        );
    }

    fn add_venue_page(&mut self, venue: &VenuePage) {
        let key = self.venue_key(&venue.name);
        if let Some(stored) = self.venues.get_mut(&key) {
            stored.address = venue.address.clone();
            stored.coordinates = venue.coordinates;
        }
    }

    fn add_fixture_row(&mut self, page: PageKind, row: &FixtureRow) {
        let Some(competition_key) = self.competition_key(row) else {
            self.issues.push(ParseIssue::new(
                page,
                &row.fixture_key,
                IssueField::Competition,
                &row.competition_name,
            ));
            return;
        };
        let fixture = Fixture {
            key: row.fixture_key.clone(),
            competition_key,
            home_team_key: self.team_key(&row.home_team_name),
            away_team_key: self.team_key(&row.away_team_name),
            kicks_off_at: row.kicks_off_at,
            venue_key: row.venue_name.as_deref().map(|name| self.venue_key(name)),
            outcome: row.outcome.clone(),
        };
        match self.fixtures.get_mut(&fixture.key) {
            Some(stored) => merge_fixture(stored, fixture),
            None => {
                self.fixtures.insert(fixture.key.clone(), fixture);
            }
        }
    }

    fn competition_key(&mut self, row: &FixtureRow) -> Option<String> {
        let normalised = normalised_name(&row.competition_name);
        let competition = if row.is_county_cup {
            Competition {
                key: format!("{COUNTY_COMPETITION_KEY_PREFIX}{normalised}"),
                name: row.competition_name.clone(),
                kind: CompetitionKind::Cup,
            }
        } else {
            self.groups_by_name.get(&normalised)?.clone()
        };
        let key = competition.key.clone();
        self.add_competition(competition);
        Some(key)
    }

    fn add_competition(&mut self, competition: Competition) {
        self.competitions
            .entry(competition.key.clone())
            .or_insert(competition);
    }

    fn register_team_id(&mut self, team_id: &str, name: &str) -> String {
        let key = format!("team:{team_id}");
        self.team_keys_by_name
            .insert(normalised_name(name), key.clone());
        self.insert_team(&key, name);
        key
    }

    fn team_key(&mut self, name: &str) -> String {
        let normalised = normalised_name(name);
        let key = self
            .team_keys_by_name
            .get(&normalised)
            .cloned()
            .unwrap_or_else(|| team_name_key(name));
        self.insert_team(&key, name);
        key
    }

    fn insert_team(&mut self, key: &str, name: &str) {
        self.teams.entry(key.to_owned()).or_insert_with(|| Team {
            key: key.to_owned(),
            name: name.to_owned(),
        });
    }

    fn venue_key(&mut self, name: &str) -> String {
        let key = format!("venue:{}", normalised_name(name));
        self.venues.entry(key.clone()).or_insert_with(|| Venue {
            key: key.clone(),
            name: name.to_owned(),
            address: None,
            coordinates: None,
        });
        key
    }

    fn finish(self) -> Parsed<Snapshot> {
        Parsed {
            content: Snapshot {
                competitions: self.competitions.into_values().collect(),
                teams: self.teams.into_values().collect(),
                venues: self.venues.into_values().collect(),
                fixtures: self.fixtures.into_values().collect(),
                standings: self.standings,
                players: self.players,
            },
            issues: self.issues,
        }
    }
}

fn merge_fixture(stored: &mut Fixture, incoming: Fixture) {
    if incoming.outcome != Outcome::Scheduled {
        stored.outcome = incoming.outcome;
    }
    if stored.venue_key.is_none() {
        stored.venue_key = incoming.venue_key;
    }
}
