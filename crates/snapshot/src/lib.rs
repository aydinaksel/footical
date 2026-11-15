use chrono::NaiveDateTime;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapshot {
    pub competitions: Vec<Competition>,
    pub teams: Vec<Team>,
    pub venues: Vec<Venue>,
    pub fixtures: Vec<Fixture>,
    pub standings: Vec<Standing>,
    pub players: Vec<Player>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Competition {
    pub key: String,
    pub name: String,
    pub kind: CompetitionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompetitionKind {
    League,
    Cup,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Team {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Venue {
    pub key: String,
    pub name: String,
    pub address: Option<String>,
    pub coordinates: Option<Coordinates>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    pub latitude_degrees: f64,
    pub longitude_degrees: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fixture {
    pub key: String,
    pub competition_key: String,
    pub home_team_key: String,
    pub away_team_key: String,
    pub kicks_off_at: NaiveDateTime,
    pub venue_key: Option<String>,
    pub outcome: Outcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Scheduled,
    Postponed,
    Played(PlayedScore),
    Walkover { winner: Side },
    Unrecognised { raw: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayedScore {
    pub full_time: Score,
    pub half_time: Option<Score>,
    pub penalties: Option<Score>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Score {
    pub home_goals: u32,
    pub away_goals: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Home,
    Away,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Standing {
    pub competition_key: String,
    pub team_key: String,
    pub position: u32,
    pub played: u32,
    pub won: u32,
    pub drawn: u32,
    pub lost: u32,
    pub goal_difference: i32,
    pub points: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub team_key: String,
    pub name: String,
}
