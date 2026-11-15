use footical_snapshot::{Outcome, PlayedScore, Score, Side};

const FIXTURE_STATUS_POSTPONED: &str = "Postponed";

pub(crate) fn parse_fixture_status(status: &str) -> Option<Outcome> {
    match status {
        "" => Some(Outcome::Scheduled),
        FIXTURE_STATUS_POSTPONED => Some(Outcome::Postponed),
        _ => None,
    }
}

pub(crate) fn parse_result_score(text: &str) -> Option<Outcome> {
    match text {
        "Postponed P - P" | "P - P" => Some(Outcome::Postponed),
        "Home Walkover H - W" | "H - W" => Some(Outcome::Walkover { winner: Side::Home }),
        "Away Walkover A - W" | "A - W" => Some(Outcome::Walkover { winner: Side::Away }),
        _ => parse_played_score(text).map(Outcome::Played),
    }
}

fn parse_played_score(text: &str) -> Option<PlayedScore> {
    let (full_time_text, details_text) = match text.split_once(" (") {
        Some((score, details)) => (score, details.strip_suffix(')')?),
        None => (text, ""),
    };
    let mut played = PlayedScore {
        full_time: parse_score(full_time_text, " - ")?,
        half_time: None,
        penalties: None,
    };
    for detail in details_text.split(", ").filter(|detail| !detail.is_empty()) {
        apply_score_detail(&mut played, detail)?;
    }
    Some(played)
}

fn apply_score_detail(played: &mut PlayedScore, detail: &str) -> Option<()> {
    let (label, score_text) = detail.split_once(' ')?;
    let score = parse_score(score_text, "-")?;
    match label {
        "HT" => played.half_time = Some(score),
        "Pens" => played.penalties = Some(score),
        _ => return None,
    }
    Some(())
}

fn parse_score(text: &str, separator: &str) -> Option<Score> {
    let (home, away) = text.split_once(separator)?;
    Some(Score {
        home_goals: home.trim().parse().ok()?,
        away_goals: away.trim().parse().ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn score(home_goals: u32, away_goals: u32) -> Score {
        Score {
            home_goals,
            away_goals,
        }
    }

    #[test]
    fn full_time_scores_parse() {
        assert_eq!(
            parse_result_score("2 - 3"),
            Some(Outcome::Played(PlayedScore {
                full_time: score(2, 3),
                half_time: None,
                penalties: None,
            }))
        );
    }

    #[test]
    fn half_time_and_penalties_parse_in_either_combination() {
        assert_eq!(
            parse_result_score("2 - 6 (HT 0-0)"),
            Some(Outcome::Played(PlayedScore {
                full_time: score(2, 6),
                half_time: Some(score(0, 0)),
                penalties: None,
            }))
        );
        assert_eq!(
            parse_result_score("2 - 2 (Pens 2-4, HT 2-2)"),
            Some(Outcome::Played(PlayedScore {
                full_time: score(2, 2),
                half_time: Some(score(2, 2)),
                penalties: Some(score(2, 4)),
            }))
        );
        assert_eq!(
            parse_result_score("3 - 3 (Pens 3-0)"),
            Some(Outcome::Played(PlayedScore {
                full_time: score(3, 3),
                half_time: None,
                penalties: Some(score(3, 0)),
            }))
        );
    }

    #[test]
    fn postponements_and_walkovers_parse() {
        assert_eq!(
            parse_result_score("Postponed P - P"),
            Some(Outcome::Postponed)
        );
        assert_eq!(
            parse_result_score("Home Walkover H - W"),
            Some(Outcome::Walkover { winner: Side::Home })
        );
        assert_eq!(
            parse_result_score("Away Walkover A - W"),
            Some(Outcome::Walkover { winner: Side::Away })
        );
    }

    #[test]
    fn anything_else_is_unrecognised() {
        assert_eq!(parse_result_score("Abandoned A - A"), None);
        assert_eq!(parse_result_score("2 - 2 (AET 3-3)"), None);
        assert_eq!(parse_result_score("VS"), None);
        assert_eq!(parse_fixture_status("Awarded"), None);
    }

    #[test]
    fn fixture_statuses_parse() {
        assert_eq!(parse_fixture_status(""), Some(Outcome::Scheduled));
        assert_eq!(parse_fixture_status("Postponed"), Some(Outcome::Postponed));
    }
}
