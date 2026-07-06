CREATE TABLE IF NOT EXISTS organisation (
    organisation_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);

INSERT OR IGNORE INTO organisation (organisation_id, name) VALUES (1, 'Footical');

CREATE TABLE IF NOT EXISTS venue (
    venue_id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_key TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    address TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS league (
    league_id INTEGER PRIMARY KEY AUTOINCREMENT,
    organisation_id INTEGER NOT NULL REFERENCES organisation (organisation_id),
    venue_id INTEGER REFERENCES venue (venue_id),
    name TEXT NOT NULL,
    day_of_week INTEGER,
    source_key TEXT NOT NULL,
    number_of_players INTEGER,
    starts_at TEXT,
    ends_at TEXT,
    price_pence INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (organisation_id, source_key)
);

CREATE TABLE IF NOT EXISTS division (
    division_id INTEGER PRIMARY KEY AUTOINCREMENT,
    league_id INTEGER NOT NULL REFERENCES league (league_id),
    name TEXT NOT NULL,
    source_key TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (league_id, source_key)
);

CREATE TABLE IF NOT EXISTS team (
    team_id INTEGER PRIMARY KEY AUTOINCREMENT,
    division_id INTEGER NOT NULL REFERENCES division (division_id),
    name TEXT NOT NULL,
    source_key TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (division_id, source_key)
);

CREATE TABLE IF NOT EXISTS fixture (
    fixture_id INTEGER PRIMARY KEY AUTOINCREMENT,
    division_id INTEGER NOT NULL REFERENCES division (division_id),
    home_team_id INTEGER NOT NULL REFERENCES team (team_id),
    away_team_id INTEGER NOT NULL REFERENCES team (team_id),
    scheduled_at TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'scheduled',
    source_key TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS division_league_id_index ON division (league_id);
CREATE INDEX IF NOT EXISTS team_division_id_index ON team (division_id);
CREATE INDEX IF NOT EXISTS fixture_division_id_index ON fixture (division_id);
CREATE INDEX IF NOT EXISTS fixture_home_team_id_index ON fixture (home_team_id);
CREATE INDEX IF NOT EXISTS fixture_away_team_id_index ON fixture (away_team_id);
CREATE INDEX IF NOT EXISTS fixture_scheduled_at_index ON fixture (scheduled_at);
