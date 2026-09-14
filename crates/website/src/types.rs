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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct SquadPlayer {
    pub squad_player_id: i32,
    pub name: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct FineType {
    pub fine_type_id: i32,
    pub name: String,
    pub default_amount_pence: i64,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct PlayerBalance {
    pub squad_player_id: i32,
    pub name: String,
    pub fines_pence: i64,
    pub payments_pence: i64,
    pub balance_pence: i64,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct SquadFixture {
    pub squad_fixture_id: i32,
    pub competition: String,
    pub kicks_off_at: chrono::NaiveDateTime,
    pub is_home: bool,
    pub opponent: String,
    pub venue: Option<String>,
}

pub fn format_pence(pence: i64) -> String {
    let is_negative = pence < 0;
    let absolute = pence.unsigned_abs();
    let pounds = absolute / 100;
    let remainder = absolute % 100;
    let sign = if is_negative { "-" } else { "" };
    format!("{sign}£{pounds}.{remainder:02}")
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct LedgerEntry {
    pub entry_id: i32,
    pub is_payment: bool,
    pub player_name: String,
    pub description: String,
    pub amount_pence: i64,
    pub happened_on: String,
    pub note: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct SquadRosterEntry {
    pub squad_player_id: i32,
    pub name: String,
    pub is_active: bool,
    pub balance_pence: i64,
}

#[cfg(test)]
mod tests {
    use super::format_pence;

    #[test]
    fn formats_whole_pounds() {
        assert_eq!(format_pence(500), "£5.00");
        assert_eq!(format_pence(1000), "£10.00");
    }

    #[test]
    fn formats_part_pounds() {
        assert_eq!(format_pence(50), "£0.50");
        assert_eq!(format_pence(1050), "£10.50");
        assert_eq!(format_pence(5), "£0.05");
    }

    #[test]
    fn formats_zero_and_credit() {
        assert_eq!(format_pence(0), "£0.00");
        assert_eq!(format_pence(-250), "-£2.50");
    }
}
