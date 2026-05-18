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

#[cfg(feature = "hydrate")]
pub fn read_tracked_team_id() -> Option<i32> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item("footical_team_id")
        .ok()?
        .and_then(|value| value.parse::<i32>().ok())
}

#[cfg(feature = "hydrate")]
pub fn save_tracked_team_id(team_id: i32) {
    if let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item("footical_team_id", &team_id.to_string());
    }
}

#[cfg(feature = "hydrate")]
pub fn clear_tracked_team_id() {
    if let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
    {
        let _ = storage.remove_item("footical_team_id");
    }
}

#[cfg(not(feature = "hydrate"))]
pub fn read_tracked_team_id() -> Option<i32> {
    None
}

#[cfg(not(feature = "hydrate"))]
pub fn save_tracked_team_id(_team_id: i32) {}

#[cfg(not(feature = "hydrate"))]
pub fn clear_tracked_team_id() {}
