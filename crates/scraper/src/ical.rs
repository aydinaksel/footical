use std::collections::HashMap;
use std::path::Path;

use chrono::NaiveDateTime;
use sqlx::PgPool;

#[derive(sqlx::FromRow)]
struct TeamRow {
    team_id: i32,
    name: String,
}

#[derive(sqlx::FromRow)]
struct FixtureRow {
    fixture_id: i32,
    home_team_id: i32,
    away_team_id: i32,
    home_team_name: String,
    away_team_name: String,
    scheduled_at: NaiveDateTime,
    status: String,
    venue_name: Option<String>,
    venue_address: Option<String>,
}

pub async fn regenerate_icals(pool: &PgPool, output_directory: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(output_directory)?;

    let teams = sqlx::query_as::<_, TeamRow>(
        "SELECT team_id, name FROM team ORDER BY name",
    )
    .fetch_all(pool)
    .await?;

    let fixtures = sqlx::query_as::<_, FixtureRow>(
        "SELECT
             fixture.fixture_id,
             fixture.home_team_id,
             fixture.away_team_id,
             home_team.name AS home_team_name,
             away_team.name AS away_team_name,
             fixture.scheduled_at,
             fixture.status,
             venue.name AS venue_name,
             venue.address AS venue_address
         FROM fixture
         JOIN team home_team ON home_team.team_id = fixture.home_team_id
         JOIN team away_team ON away_team.team_id = fixture.away_team_id
         JOIN division ON division.division_id = fixture.division_id
         JOIN league ON league.league_id = division.league_id
         LEFT JOIN venue ON venue.venue_id = league.venue_id
         ORDER BY fixture.scheduled_at",
    )
    .fetch_all(pool)
    .await?;

    let mut fixtures_by_team: HashMap<i32, Vec<&FixtureRow>> = HashMap::new();
    for fixture in &fixtures {
        fixtures_by_team
            .entry(fixture.home_team_id)
            .or_default()
            .push(fixture);
        fixtures_by_team
            .entry(fixture.away_team_id)
            .or_default()
            .push(fixture);
    }

    let generation_timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();

    for team in &teams {
        let team_fixtures = fixtures_by_team
            .get(&team.team_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let ical = build_ical(&team.name, team.team_id, team_fixtures, &generation_timestamp);
        let path = output_directory.join(format!("{}.ics", team.team_id));
        atomic_write(&path, ical.as_bytes())?;
    }

    tracing::info!(
        teams = teams.len(),
        fixtures = fixtures.len(),
        "regenerated iCal files",
    );

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
    push_folded(
        &mut output,
        &format!("X-WR-CALNAME:{} Fixtures", team_name),
    );
    output.push_str("BEGIN:VTIMEZONE\r\n");
    output.push_str("TZID:Europe/London\r\n");
    output.push_str("BEGIN:STANDARD\r\n");
    output.push_str("TZOFFSETFROM:+0100\r\n");
    output.push_str("TZOFFSETTO:+0000\r\n");
    output.push_str("TZNAME:GMT\r\n");
    output.push_str("DTSTART:19701025T020000\r\n");
    output.push_str("RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10\r\n");
    output.push_str("END:STANDARD\r\n");
    output.push_str("BEGIN:DAYLIGHT\r\n");
    output.push_str("TZOFFSETFROM:+0000\r\n");
    output.push_str("TZOFFSETTO:+0100\r\n");
    output.push_str("TZNAME:BST\r\n");
    output.push_str("DTSTART:19700329T010000\r\n");
    output.push_str("RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=3\r\n");
    output.push_str("END:DAYLIGHT\r\n");
    output.push_str("END:VTIMEZONE\r\n");

    for fixture in fixtures {
        let start = fixture.scheduled_at.format("%Y%m%dT%H%M%S").to_string();
        let end = (fixture.scheduled_at + chrono::Duration::minutes(35))
            .format("%Y%m%dT%H%M%S")
            .to_string();
        let ical_status = match fixture.status.as_str() {
            "cancelled" | "postponed" => "CANCELLED",
            _ => "CONFIRMED",
        };

        output.push_str("BEGIN:VEVENT\r\n");
        push_folded(
            &mut output,
            &format!(
                "UID:fixture-{}-{}@footical.club",
                fixture.fixture_id, team_id
            ),
        );
        push_folded(&mut output, &format!("DTSTAMP:{}", generation_timestamp));
        push_folded(
            &mut output,
            &format!("DTSTART;TZID=Europe/London:{}", start),
        );
        push_folded(
            &mut output,
            &format!("DTEND;TZID=Europe/London:{}", end),
        );
        let opponent_name = if fixture.home_team_id == team_id {
            escape_ical_text(&fixture.away_team_name)
        } else {
            escape_ical_text(&fixture.home_team_name)
        };
        push_folded(&mut output, &format!("SUMMARY:Versus {}", opponent_name));
        if let Some(address) = &fixture.venue_address {
            push_folded(
                &mut output,
                &format!("LOCATION:{}", escape_ical_text(address)),
            );
        } else if let Some(name) = &fixture.venue_name {
            push_folded(
                &mut output,
                &format!("LOCATION:{}", escape_ical_text(name)),
            );
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

fn push_folded(output: &mut String, line: &str) {
    const FIRST_LINE_MAX_BYTES: usize = 75;
    const CONTINUATION_MAX_BYTES: usize = 74;

    let bytes = line.as_bytes();

    if bytes.len() <= FIRST_LINE_MAX_BYTES {
        output.push_str(line);
        output.push_str("\r\n");
        return;
    }

    let first_split = utf8_boundary(bytes, FIRST_LINE_MAX_BYTES);
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

fn utf8_boundary(bytes: &[u8], max_bytes: usize) -> usize {
    let end = max_bytes.min(bytes.len());
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
