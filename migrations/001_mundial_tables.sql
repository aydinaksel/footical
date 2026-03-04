CREATE TABLE IF NOT EXISTS organisation (
  organisation_id SERIAL PRIMARY KEY,
  name            VARCHAR(100) NOT NULL,
  website         VARCHAR(255),
  created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS venue (
  venue_id   SERIAL PRIMARY KEY,
  name       VARCHAR(100) NOT NULL,
  address    VARCHAR(255),
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- A recurring league at a specific venue on a specific day
-- e.g. "York Monday at Huntington School"
CREATE TABLE IF NOT EXISTS league (
  league_id       SERIAL PRIMARY KEY,
  organisation_id INTEGER NOT NULL REFERENCES organisation(organisation_id),
  venue_id        INTEGER REFERENCES venue(venue_id) ON DELETE SET NULL,
  name            VARCHAR(100) NOT NULL,
  day_of_week     SMALLINT CHECK (day_of_week BETWEEN 1 AND 7), -- ISO: 1=Mon, 7=Sun
  source_key      VARCHAR(255),
  created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (organisation_id, source_key)
);

-- A division within a league, e.g. "Division 1", "Premier"
CREATE TABLE IF NOT EXISTS division (
  division_id SERIAL PRIMARY KEY,
  league_id   INTEGER NOT NULL REFERENCES league(league_id) ON DELETE CASCADE,
  name        VARCHAR(100) NOT NULL,
  source_key  VARCHAR(255),
  created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (league_id, source_key)
);

CREATE TABLE IF NOT EXISTS team (
  team_id     SERIAL PRIMARY KEY,
  division_id INTEGER NOT NULL REFERENCES division(division_id) ON DELETE CASCADE,
  name        VARCHAR(100) NOT NULL,
  source_key  VARCHAR(255),
  created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (division_id, source_key)
);

CREATE TABLE IF NOT EXISTS fixture (
  fixture_id   SERIAL PRIMARY KEY,
  division_id  INTEGER NOT NULL REFERENCES division(division_id) ON DELETE CASCADE,
  home_team_id INTEGER NOT NULL REFERENCES team(team_id),
  away_team_id INTEGER NOT NULL REFERENCES team(team_id),
  scheduled_at TIMESTAMP NOT NULL,
  home_score   SMALLINT,
  away_score   SMALLINT,
  status       VARCHAR(20) NOT NULL DEFAULT 'scheduled',
             -- scheduled | played | postponed | cancelled
  source_key   VARCHAR(255) UNIQUE,
  created_at   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
