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

CREATE TABLE IF NOT EXISTS squad_player (
    squad_player_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS fine_type (
    fine_type_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    default_amount_pence INTEGER NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS fine (
    fine_id INTEGER PRIMARY KEY AUTOINCREMENT,
    squad_player_id INTEGER NOT NULL REFERENCES squad_player (squad_player_id),
    fine_type_id INTEGER NOT NULL REFERENCES fine_type (fine_type_id),
    amount_pence INTEGER NOT NULL,
    incurred_on TEXT NOT NULL,
    note TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS payment (
    payment_id INTEGER PRIMARY KEY AUTOINCREMENT,
    squad_player_id INTEGER NOT NULL REFERENCES squad_player (squad_player_id),
    amount_pence INTEGER NOT NULL,
    paid_on TEXT NOT NULL,
    note TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS squad_fixture (
    squad_fixture_id INTEGER PRIMARY KEY AUTOINCREMENT,
    competition TEXT NOT NULL,
    kicks_off_at TEXT NOT NULL,
    is_home INTEGER NOT NULL,
    opponent TEXT NOT NULL,
    venue TEXT,
    source_key TEXT NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS fine_squad_player_id_index ON fine (squad_player_id);
CREATE INDEX IF NOT EXISTS payment_squad_player_id_index ON payment (squad_player_id);
CREATE INDEX IF NOT EXISTS squad_fixture_kicks_off_at_index ON squad_fixture (kicks_off_at);

INSERT OR IGNORE INTO squad_player (name) VALUES
    ('Dwight Adams'),
    ('Aydin Aksel'),
    ('Mathew Appleton'),
    ('Will Atherton'),
    ('Connor Attwood'),
    ('Maximillian Rocco Bauer'),
    ('Wolfgang Bauer'),
    ('Christopher Bivens'),
    ('Louis Boyce'),
    ('Jack Brown'),
    ('Charlie Cammidge'),
    ('Cole Carter'),
    ('Jay Carter'),
    ('Ellis Cattaneo'),
    ('Leon Codrai'),
    ('Marco De Jesus'),
    ('Alexander Richard Dunning'),
    ('James Edmond'),
    ('Samuel Gough'),
    ('Jamie Green'),
    ('Daniel Hickey'),
    ('Dale Holding'),
    ('Neale Holmes'),
    ('Sebastian Holowkiewicz'),
    ('Eddie Hopper'),
    ('Ethan Hurren'),
    ('Daniel Jefferson'),
    ('Joseph Johnson'),
    ('Archie Klar'),
    ('Owen Laird'),
    ('Ethan Lloyd-Jones'),
    ('Joshua Luckhurst'),
    ('Jordan McGuigan'),
    ('George Newton'),
    ('Thomas Perry'),
    ('Isaac Pulleyn'),
    ('Harry Rhodes'),
    ('Liam Robertson'),
    ('Dayton Scott'),
    ('Oscar Shaw'),
    ('Finlay Simpson'),
    ('Joe Stamp'),
    ('Ethan Suter'),
    ('William Suter'),
    ('James Sutton'),
    ('Charley Tompkins'),
    ('Dale Waddington'),
    ('Jamie Willstrop'),
    ('Leo Wilson');

INSERT OR IGNORE INTO fine_type (name, default_amount_pence) VALUES
    ('Boozing Night Before', 500),
    ('Dirty Boots', 100),
    ('Dissent Towards Ref', 1000),
    ('Failing Post Match Job', 200),
    ('Late (00 - 05 minutes)', 100),
    ('Late (05 - 10 minutes)', 200),
    ('Late (10 - 15 minutes)', 300),
    ('Late (15 - 20 minutes)', 400),
    ('Match Shirt Not Inside Out', 100),
    ('Missed Availability Deadline', 200),
    ('Missed Match w/o Voice Note', 100),
    ('Missing Shower Item x1', 100),
    ('Missing Shower Item x2', 200),
    ('Missing Shower Item x3', 300),
    ('No Blind Card', 100),
    ('No Post Match Drink', 100),
    ('No Post Match Shower', 100),
    ('No Training Top', 100),
    ('No Voice Note', 100),
    ('Playing for Another Team', 1000),
    ('Pointless Moaning', 50),
    ('Pound a Pint (Underage)', 100),
    ('Short Notice Drop-out', 500),
    ('Throw-in Failure', 100),
    ('Unsympathetic', 100),
    ('Yellow Card', 1000),
    ('Sin Bin', 500),
    ('Other', 100);

INSERT OR IGNORE INTO squad_fixture (competition, kicks_off_at, is_home, opponent, venue, source_key) VALUES
    ('YMRA', '2026-09-19T14:00:00', 0, 'Tadcaster Magnets Reserves', 'TADCASTER COMMUNITY SPORTS TRUST #1', '2026-09-19-tadcaster-magnets-reserves'),
    ('Cup', '2026-09-26T14:00:00', 1, 'Heslington Reserves', 'THE WIGGINTON SPORTS AND PLAYING FIELDS 11 v 11 Pitch 2', '2026-09-26-heslington-reserves'),
    ('CC', '2026-10-03T14:00:00', 0, 'HAMILTON PANTHERS Reserves', 'HAMILTON PANTHERS Reserves', '2026-10-03-hamilton-panthers-reserves'),
    ('YMRA', '2026-10-10T14:00:00', 0, 'OLD MALTON ST MARY''S Second', 'THE GANNOCK 11 v 11 Pitch 2', '2026-10-10-old-malton-st-marys-second'),
    ('YMRA', '2026-10-17T14:00:00', 1, 'POPPLETON FC Poppleton FC reserves', 'THE WIGGINTON SPORTS AND PLAYING FIELDS 11 v 11 Pitch 2', '2026-10-17-poppleton-fc-reserves'),
    ('YMRA', '2026-10-28T14:00:00', 0, 'OSBALDWICK Reserves', 'OSBALDWICK SPORTS CLUB 11 v 11 Pitch 1', '2026-10-28-osbaldwick-reserves'),
    ('YMRA', '2026-10-31T14:00:00', 0, 'Malt Shovel (Selby) Reserves', 'DENISON ROAD FOOTBALL PITCHES 2', '2026-10-31-malt-shovel-selby-reserves'),
    ('YMRA', '2026-11-07T14:00:00', 1, 'HAMILTON PANTHERS Reserves', 'THE WIGGINTON SPORTS AND PLAYING FIELDS 11 v 11 Pitch 2', '2026-11-07-hamilton-panthers-reserves'),
    ('YMRA', '2026-11-14T14:00:00', 0, 'Fulford Football Club Reserves', 'FULFORD SCHOOL 11 v 11 Pitch', '2026-11-14-fulford-football-club-reserves'),
    ('YMRA', '2026-11-21T14:00:00', 1, 'Tadcaster Magnets Reserves', 'THE WIGGINTON SPORTS AND PLAYING FIELDS 11 v 11 Pitch 2', '2026-11-21-tadcaster-magnets-reserves'),
    ('YMRA', '2026-12-12T14:00:00', 1, 'Howden AFC Reserves', 'THE WIGGINTON SPORTS AND PLAYING FIELDS 11 v 11 Pitch 2', '2026-12-12-howden-afc-reserves'),
    ('YMRA', '2026-12-19T14:00:00', 1, 'Fulford Football Club Reserves', 'THE WIGGINTON SPORTS AND PLAYING FIELDS 11 v 11 Pitch 2', '2026-12-19-fulford-football-club-reserves'),
    ('YMRA', '2027-01-09T14:00:00', 1, 'DRINGHOUSES Reserves', 'THE WIGGINTON SPORTS AND PLAYING FIELDS 11 v 11 Pitch 2', '2027-01-09-dringhouses-reserves'),
    ('YMRA', '2027-01-16T14:00:00', 0, 'Dunnington Reserves', 'DUNNINGTON PLAYING FIELDS ASSOCIATION 11v11 Pitch 2', '2027-01-16-dunnington-reserves'),
    ('YMRA', '2027-01-23T14:00:00', 1, 'OLD MALTON ST MARY''S Second', 'THE WIGGINTON SPORTS AND PLAYING FIELDS 11 v 11 Pitch 2', '2027-01-23-old-malton-st-marys-second'),
    ('YMRA', '2027-01-30T14:00:00', 0, 'POPPLETON FC Poppleton FC reserves', 'POPPLETON SPORT PAVILION (POPPLETON FC) 11v11 Pitch 2', '2027-01-30-poppleton-fc-reserves'),
    ('YMRA', '2027-02-13T14:00:00', 1, 'Malt Shovel (Selby) Reserves', 'THE WIGGINTON SPORTS AND PLAYING FIELDS 11 v 11 Pitch 2', '2027-02-13-malt-shovel-selby-reserves'),
    ('YMRA', '2027-02-20T14:00:00', 0, 'HAMILTON PANTHERS Reserves', 'LITTLE KNAVESMIRE (HAMILTON PANTHERS) 11 v 11 Pitch 2', '2027-02-20-hamilton-panthers-reserves');
