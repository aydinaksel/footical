use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct League {
    pub league_id: i32,
    pub name: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Division {
    pub division_id: i32,
    pub league_id: i32,
    pub name: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Team {
    pub team_id: i32,
    pub division_id: i32,
    pub name: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Fixture {
    pub fixture_id: i32,
    pub home_team_id: i32,
    pub away_team_id: i32,
    pub home_team_name: String,
    pub away_team_name: String,
    pub scheduled_at: chrono::NaiveDateTime,
    pub status: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct TodayFixture {
    pub fixture_id: i32,
    pub scheduled_at: chrono::NaiveDateTime,
    pub status: String,
    pub home_team_name: String,
    pub away_team_name: String,
    pub division_name: String,
    pub league_name: String,
    pub venue_name: Option<String>,
    pub venue_address: Option<String>,
}
