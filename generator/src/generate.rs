use crate::database::{DivisionRow, FixtureRow, LeagueRow, TeamRow};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Serialize)]
struct LeagueJson {
    league_id: i32,
    name: String,
}

#[derive(Serialize)]
struct DivisionJson {
    division_id: i32,
    league_id: i32,
    name: String,
}

#[derive(Serialize)]
struct TeamJson {
    team_id: i32,
    division_id: i32,
    name: String,
}

pub fn write_all(
    output_directory: &Path,
    leagues: &[LeagueRow],
    divisions: &[DivisionRow],
    teams: &[TeamRow],
    fixtures: &[FixtureRow],
) -> anyhow::Result<()> {
    std::fs::create_dir_all(output_directory)?;

    write_leagues(output_directory, leagues)?;
    write_divisions(output_directory, divisions)?;
    write_teams(output_directory, teams)?;
    write_icals(output_directory, teams, fixtures)?;

    Ok(())
}

fn write_leagues(output_directory: &Path, leagues: &[LeagueRow]) -> anyhow::Result<()> {
    let json: Vec<LeagueJson> = leagues
        .iter()
        .map(|league| LeagueJson { league_id: league.league_id, name: league.name.clone() })
        .collect();
    write_json(output_directory.join("leagues.json"), &json)
}

fn write_divisions(output_directory: &Path, divisions: &[DivisionRow]) -> anyhow::Result<()> {
    let json: Vec<DivisionJson> = divisions
        .iter()
        .map(|division| DivisionJson {
            division_id: division.division_id,
            league_id: division.league_id,
            name: division.name.clone(),
        })
        .collect();
    write_json(output_directory.join("divisions.json"), &json)
}

fn write_teams(output_directory: &Path, teams: &[TeamRow]) -> anyhow::Result<()> {
    let json: Vec<TeamJson> = teams
        .iter()
        .map(|team| TeamJson {
            team_id: team.team_id,
            division_id: team.division_id,
            name: team.name.clone(),
        })
        .collect();
    write_json(output_directory.join("teams.json"), &json)
}

fn write_json<T: Serialize>(path: std::path::PathBuf, value: &T) -> anyhow::Result<()> {
    let content = serde_json::to_string(value)?;
    atomic_write(&path, content.as_bytes())
}

fn write_icals(
    output_directory: &Path,
    teams: &[TeamRow],
    fixtures: &[FixtureRow],
) -> anyhow::Result<()> {
    let mut fixtures_by_team: HashMap<i32, Vec<&FixtureRow>> = HashMap::new();
    for fixture in fixtures {
        fixtures_by_team.entry(fixture.home_team_id).or_default().push(fixture);
        fixtures_by_team.entry(fixture.away_team_id).or_default().push(fixture);
    }

    let generation_timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();

    for team in teams {
        let team_fixtures = fixtures_by_team
            .get(&team.team_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let ical = build_ical(&team.name, team.team_id, team_fixtures, &generation_timestamp);
        let path = output_directory.join(format!("{}.ics", team.team_id));
        atomic_write(&path, ical.as_bytes())?;
    }

    Ok(())
}

fn build_ical(
    team_name: &str,
    team_id: i32,
    fixtures: &[&FixtureRow],
    generation_timestamp: &str,
) -> String {
    let mut output = String::new();

    output.push_str("BEGIN:VCALENDAR\r\n");
    output.push_str("VERSION:2.0\r\n");
    output.push_str("PRODID:-//footical.club//Footical//EN\r\n");
    output.push_str("CALSCALE:GREGORIAN\r\n");
    output.push_str("METHOD:PUBLISH\r\n");
    push_folded(&mut output, &format!("X-WR-CALNAME:{} Fixtures", team_name));

    for fixture in fixtures {
        let start = fixture.scheduled_at.format("%Y%m%dT%H%M%S").to_string();
        let end = (fixture.scheduled_at + chrono::Duration::minutes(90))
            .format("%Y%m%dT%H%M%S")
            .to_string();
        let ical_status = match fixture.status.as_str() {
            "cancelled" | "postponed" => "CANCELLED",
            _ => "CONFIRMED",
        };

        output.push_str("BEGIN:VEVENT\r\n");
        push_folded(
            &mut output,
            &format!("UID:fixture-{}-{}@footical.club", fixture.fixture_id, team_id),
        );
        push_folded(&mut output, &format!("DTSTAMP:{}", generation_timestamp));
        push_folded(&mut output, &format!("DTSTART:{}", start));
        push_folded(&mut output, &format!("DTEND:{}", end));
        push_folded(
            &mut output,
            &format!(
                "SUMMARY:{} vs {}",
                escape_ical_text(&fixture.home_team_name),
                escape_ical_text(&fixture.away_team_name),
            ),
        );
        if let Some(address) = &fixture.venue_address {
            push_folded(&mut output, &format!("LOCATION:{}", escape_ical_text(address)));
        } else if let Some(name) = &fixture.venue_name {
            push_folded(&mut output, &format!("LOCATION:{}", escape_ical_text(name)));
        }
        push_folded(&mut output, &format!("STATUS:{}", ical_status));
        output.push_str("END:VEVENT\r\n");
    }

    output.push_str("END:VCALENDAR\r\n");
    output
}

fn escape_ical_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace(',', "\\,")
        .replace(';', "\\;")
        .replace('\n', "\\n")
}

// Appends a property line to output, folding at 75 octets per RFC 5545.
fn push_folded(output: &mut String, line: &str) {
    const FIRST_LINE_MAX_BYTES: usize = 75;
    const CONTINUATION_MAX_BYTES: usize = 74; // 75 minus the leading space

    let bytes = line.as_bytes();

    if bytes.len() <= FIRST_LINE_MAX_BYTES {
        output.push_str(line);
        output.push_str("\r\n");
        return;
    }

    let first_split = utf8_boundary(bytes, FIRST_LINE_MAX_BYTES);
    // SAFETY: utf8_boundary returns a valid char boundary index
    output.push_str(&line[..first_split]);
    output.push_str("\r\n");

    let mut offset = first_split;
    while offset < bytes.len() {
        let remaining_bytes = &bytes[offset..];
        let split = utf8_boundary(remaining_bytes, CONTINUATION_MAX_BYTES);
        output.push(' ');
        output.push_str(&line[offset..offset + split]);
        output.push_str("\r\n");
        offset += split;
    }
}

// Returns the largest byte index <= max_bytes that falls on a UTF-8 char boundary.
fn utf8_boundary(bytes: &[u8], max_bytes: usize) -> usize {
    let end = max_bytes.min(bytes.len());
    // Walk back from end until we hit a valid UTF-8 char boundary.
    // A byte is a continuation byte if it matches 0b10xxxxxx.
    let mut boundary = end;
    while boundary > 0 && (bytes[boundary - 1] & 0b1100_0000) == 0b1000_0000 {
        boundary -= 1;
    }
    boundary
}

fn atomic_write(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, content)?;
    std::fs::rename(&temp_path, path)?;
    Ok(())
}
