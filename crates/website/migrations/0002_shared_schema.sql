CREATE TABLE source (
    source_id INTEGER PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
);

INSERT INTO source (source_id, code, name) VALUES
    (1, 'football-mundial', 'Football Mundial'),
    (2, 'fa-fulltime', 'FA Full-Time');

CREATE TABLE new_venue (
    venue_id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL REFERENCES source (source_id),
    source_key TEXT NOT NULL,
    name TEXT NOT NULL,
    address TEXT,
    latitude_degrees REAL,
    longitude_degrees REAL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (source_id, source_key)
);

INSERT INTO new_venue (venue_id, source_id, source_key, name, address, created_at, updated_at)
SELECT venue_id, 1, source_key, name, address, created_at, updated_at
FROM venue;

CREATE TABLE new_league (
    league_id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL REFERENCES source (source_id),
    venue_id INTEGER REFERENCES new_venue (venue_id),
    name TEXT NOT NULL,
    day_of_week INTEGER,
    source_key TEXT NOT NULL,
    number_of_players INTEGER,
    starts_at TEXT,
    ends_at TEXT,
    price_pence INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (source_id, source_key)
);

INSERT INTO new_league (league_id, source_id, venue_id, name, day_of_week, source_key,
                        number_of_players, starts_at, ends_at, price_pence, created_at, updated_at)
SELECT league_id, 1, venue_id, name, day_of_week, source_key,
       number_of_players, starts_at, ends_at, price_pence, created_at, updated_at
FROM league;

CREATE TABLE competition (
    competition_id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL REFERENCES source (source_id),
    league_id INTEGER REFERENCES new_league (league_id),
    name TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('league', 'cup')),
    source_key TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (source_id, source_key)
);

INSERT INTO competition (competition_id, source_id, league_id, name, kind, source_key,
                         created_at, updated_at)
SELECT division_id, 1, league_id, name, 'league', source_key, created_at, updated_at
FROM division;

CREATE TABLE new_team (
    team_id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL REFERENCES source (source_id),
    competition_id INTEGER REFERENCES competition (competition_id),
    name TEXT NOT NULL,
    source_key TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (source_id, source_key)
);

INSERT INTO new_team (team_id, source_id, competition_id, name, source_key, created_at, updated_at)
SELECT team.team_id, 1, team.division_id, team.name,
       division.source_key || '|' || team.source_key, team.created_at, team.updated_at
FROM team
JOIN division ON division.division_id = team.division_id;

CREATE TABLE new_fixture (
    fixture_id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL REFERENCES source (source_id),
    competition_id INTEGER NOT NULL REFERENCES competition (competition_id),
    home_team_id INTEGER NOT NULL REFERENCES new_team (team_id),
    away_team_id INTEGER NOT NULL REFERENCES new_team (team_id),
    venue_id INTEGER REFERENCES new_venue (venue_id),
    scheduled_at TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'scheduled'
        CHECK (status IN ('scheduled', 'postponed', 'played', 'walkover', 'unrecognised', 'removed')),
    home_goals INTEGER,
    away_goals INTEGER,
    half_time_home_goals INTEGER,
    half_time_away_goals INTEGER,
    penalty_home_goals INTEGER,
    penalty_away_goals INTEGER,
    walkover_winner TEXT CHECK (walkover_winner IN ('home', 'away')),
    unrecognised_outcome TEXT,
    source_key TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (source_id, source_key),
    CHECK (status <> 'played' OR (home_goals IS NOT NULL AND away_goals IS NOT NULL)),
    CHECK (status <> 'walkover' OR walkover_winner IS NOT NULL),
    CHECK (status <> 'unrecognised' OR unrecognised_outcome IS NOT NULL)
);

INSERT INTO new_fixture (fixture_id, source_id, competition_id, home_team_id, away_team_id,
                         venue_id, scheduled_at, status, source_key, created_at, updated_at)
SELECT fixture.fixture_id, 1, fixture.division_id, fixture.home_team_id, fixture.away_team_id,
       league.venue_id, fixture.scheduled_at,
       CASE fixture.status
           WHEN 'postponed' THEN 'postponed'
           WHEN 'cancelled' THEN 'removed'
           ELSE 'scheduled'
       END,
       fixture.source_key, fixture.created_at, fixture.updated_at
FROM fixture
JOIN division ON division.division_id = fixture.division_id
JOIN league ON league.league_id = division.league_id;

DROP TABLE fixture;
DROP TABLE team;
DROP TABLE division;
DROP TABLE league;
DROP TABLE venue;
DROP TABLE organisation;

ALTER TABLE new_venue RENAME TO venue;
ALTER TABLE new_league RENAME TO league;
ALTER TABLE new_team RENAME TO team;
ALTER TABLE new_fixture RENAME TO fixture;

CREATE INDEX league_venue_id_index ON league (venue_id);
CREATE INDEX competition_league_id_index ON competition (league_id);
CREATE INDEX team_competition_id_index ON team (competition_id);
CREATE INDEX fixture_competition_id_index ON fixture (competition_id);
CREATE INDEX fixture_home_team_id_index ON fixture (home_team_id);
CREATE INDEX fixture_away_team_id_index ON fixture (away_team_id);
CREATE INDEX fixture_venue_id_index ON fixture (venue_id);
CREATE INDEX fixture_scheduled_at_index ON fixture (scheduled_at);

CREATE TABLE standing (
    competition_id INTEGER NOT NULL REFERENCES competition (competition_id),
    team_id INTEGER NOT NULL REFERENCES team (team_id),
    position INTEGER NOT NULL,
    played INTEGER NOT NULL,
    won INTEGER NOT NULL,
    drawn INTEGER NOT NULL,
    lost INTEGER NOT NULL,
    goal_difference INTEGER NOT NULL,
    points INTEGER NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (competition_id, team_id)
);

CREATE INDEX standing_team_id_index ON standing (team_id);

CREATE TABLE source_page (
    source_page_id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL REFERENCES source (source_id),
    page_kind TEXT NOT NULL,
    page_key TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    body TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (source_id, page_kind, page_key)
);

CREATE TABLE parse_issue (
    parse_issue_id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL REFERENCES source (source_id),
    page_kind TEXT NOT NULL,
    subject TEXT NOT NULL,
    field TEXT NOT NULL,
    raw TEXT NOT NULL,
    first_seen_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    resolved_at TEXT,
    UNIQUE (source_id, page_kind, subject, field, raw)
);
