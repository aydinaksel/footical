use chrono::NaiveDateTime;
use sqlx::SqlitePool;

const FIXTURE_DURATION_MINUTES: i64 = 35;

#[derive(sqlx::FromRow)]
#[expect(
    dead_code,
    reason = "sqlx::FromRow populates every column; not all are read"
)]
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

pub async fn generate_for_team(pool: &SqlitePool, team_id: i32) -> anyhow::Result<Option<String>> {
    let team_name: Option<String> = sqlx::query_scalar("SELECT name FROM team WHERE team_id = ?")
        .bind(team_id)
        .fetch_optional(pool)
        .await?;

    let team_name = match team_name {
        Some(name) => name,
        None => return Ok(None),
    };

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
         WHERE fixture.home_team_id = ?1 OR fixture.away_team_id = ?1
         ORDER BY fixture.scheduled_at",
    )
    .bind(team_id)
    .fetch_all(pool)
    .await?;

    let generation_timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let fixture_refs: Vec<&FixtureRow> = fixtures.iter().collect();
    let ical = build_ical(&team_name, team_id, &fixture_refs, &generation_timestamp);

    Ok(Some(ical))
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
        let end = fixture
            .scheduled_at
            .checked_add_signed(chrono::Duration::minutes(FIXTURE_DURATION_MINUTES))
            .unwrap_or(fixture.scheduled_at)
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
        push_folded(&mut output, &format!("DTEND;TZID=Europe/London:{}", end));
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

fn push_folded(output: &mut String, line: &str) {
    const FIRST_LINE_MAX_BYTES: usize = 75;
    const CONTINUATION_MAX_BYTES: usize = 74;

    let mut remaining = line;
    let mut max_bytes = FIRST_LINE_MAX_BYTES;

    loop {
        let Some(split) = fold_point(remaining, max_bytes) else {
            output.push_str(remaining);
            output.push_str("\r\n");
            return;
        };

        let (folded, rest) = remaining.split_at_checked(split).unwrap_or((remaining, ""));
        output.push_str(folded);
        output.push_str("\r\n");
        output.push(' ');

        remaining = rest;
        max_bytes = CONTINUATION_MAX_BYTES;
    }
}

fn fold_point(line: &str, max_bytes: usize) -> Option<usize> {
    if line.len() <= max_bytes {
        return None;
    }
    line.char_indices()
        .map(|(index, character)| index.saturating_add(character.len_utf8()))
        .take_while(|end| *end <= max_bytes)
        .last()
        .filter(|split| *split > 0)
}
