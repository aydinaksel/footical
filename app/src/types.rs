use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct League {
    pub league_id: i32,
    pub name: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Division {
    pub division_id: i32,
    pub league_id: i32,
    pub name: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Team {
    pub team_id: i32,
    pub division_id: i32,
    pub name: String,
}

pub fn read_tracked_team_id() -> Option<i32> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item("footical_team_id")
        .ok()?
        .and_then(|value| value.parse::<i32>().ok())
}

pub fn save_tracked_team_id(team_id: i32) {
    if let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item("footical_team_id", &team_id.to_string());
    }
}

pub fn clear_tracked_team_id() {
    if let Some(storage) = web_sys::window()
        .and_then(|window| window.local_storage().ok())
        .flatten()
    {
        let _ = storage.remove_item("footical_team_id");
    }
}
