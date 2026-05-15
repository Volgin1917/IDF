-- ICE DATA FORGE — initial schema

-- Trigger function for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Teams
CREATE TABLE teams (
    id SERIAL PRIMARY KEY,
    nhl_team_id INTEGER UNIQUE NOT NULL,
    abbreviation TEXT UNIQUE NOT NULL,
    team_name TEXT NOT NULL,
    location TEXT NOT NULL,
    short_name TEXT,
    conference TEXT CHECK (conference IN ('Eastern', 'Western')),
    division TEXT,
    venue_name TEXT,
    venue_city TEXT,
    venue_capacity INTEGER,
    founded_year INTEGER,
    first_year_of_play INTEGER,
    twitter TEXT,
    official_site_url TEXT,
    active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_teams_abbreviation ON teams(abbreviation);
CREATE INDEX idx_teams_conference ON teams(conference);

-- Players
CREATE TABLE players (
    id SERIAL PRIMARY KEY,
    nhl_player_id INTEGER UNIQUE NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    full_name TEXT NOT NULL,
    position TEXT NOT NULL CHECK (position IN ('C', 'LW', 'RW', 'D', 'G')),
    shoot_catches TEXT CHECK (shoot_catches IN ('L', 'R')),
    height_cm INTEGER,
    height_display TEXT,
    weight_kg INTEGER,
    weight_lbs INTEGER,
    birth_date DATE,
    birth_city TEXT,
    birth_state_province TEXT,
    birth_country TEXT,
    nationality TEXT,
    current_team_id INTEGER REFERENCES teams(id),
    current_team_abbreviation TEXT,
    jersey_number INTEGER,
    draft_year INTEGER,
    draft_round INTEGER,
    draft_pick INTEGER,
    draft_team_abbreviation TEXT,
    photo_url TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    last_api_sync TIMESTAMPTZ,
    search_vector TSVECTOR,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_players_nhl_id ON players(nhl_player_id);
CREATE INDEX idx_players_full_name ON players(full_name);
CREATE INDEX idx_players_position ON players(position);
CREATE INDEX idx_players_current_team ON players(current_team_abbreviation);
CREATE INDEX idx_players_search_vector ON players USING GIN(search_vector);

CREATE TRIGGER players_search_vector_update
    BEFORE INSERT OR UPDATE ON players
    FOR EACH ROW
    EXECUTE FUNCTION tsvector_update_trigger(
        search_vector, 'pg_catalog.english', first_name, last_name, full_name
    );

CREATE TRIGGER update_players_updated_at
    BEFORE UPDATE ON players
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Player season stats
CREATE TABLE player_season_stats (
    id SERIAL PRIMARY KEY,
    player_id INTEGER REFERENCES players(id) ON DELETE CASCADE,
    nhl_player_id INTEGER NOT NULL,
    season TEXT NOT NULL,
    league TEXT DEFAULT 'NHL',
    team_abbreviation TEXT,
    games_played INTEGER DEFAULT 0,
    games_started INTEGER DEFAULT 0,
    goals INTEGER DEFAULT 0,
    assists INTEGER DEFAULT 0,
    points INTEGER DEFAULT 0,
    plus_minus INTEGER DEFAULT 0,
    penalty_minutes INTEGER DEFAULT 0,
    power_play_goals INTEGER DEFAULT 0,
    power_play_assists INTEGER DEFAULT 0,
    power_play_points INTEGER DEFAULT 0,
    short_handed_goals INTEGER DEFAULT 0,
    short_handed_assists INTEGER DEFAULT 0,
    short_handed_points INTEGER DEFAULT 0,
    game_winning_goals INTEGER DEFAULT 0,
    overtime_goals INTEGER DEFAULT 0,
    shots INTEGER DEFAULT 0,
    shooting_percentage DECIMAL(5,2),
    time_on_ice_per_game TEXT,
    shifts_per_game DECIMAL(4,1),
    face_off_wins INTEGER,
    face_off_losses INTEGER,
    face_off_percentage DECIMAL(5,2),
    advanced_metrics JSONB DEFAULT '{}'::jsonb,
    points_per_game DECIMAL(4,2),
    goals_per_60 DECIMAL(4,2),
    assists_per_60 DECIMAL(4,2),
    points_per_60 DECIMAL(4,2),
    raw_api_data JSONB,
    data_source TEXT DEFAULT 'NHL_API',
    calculated_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(nhl_player_id, season, team_abbreviation)
);
CREATE INDEX idx_stats_player_season ON player_season_stats(nhl_player_id, season);
CREATE INDEX idx_stats_season ON player_season_stats(season);
CREATE INDEX idx_stats_team ON player_season_stats(team_abbreviation);
CREATE INDEX idx_stats_advanced_gin ON player_season_stats USING GIN(advanced_metrics jsonb_path_ops);

CREATE TRIGGER update_stats_updated_at
    BEFORE UPDATE ON player_season_stats
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- AI Analysis
CREATE TABLE ai_analysis (
    id SERIAL PRIMARY KEY,
    player_id INTEGER REFERENCES players(id) ON DELETE CASCADE,
    nhl_player_id INTEGER NOT NULL,
    analysis_date TIMESTAMPTZ DEFAULT NOW(),
    season TEXT,
    analysis_type TEXT CHECK (analysis_type IN ('full', 'potential', 'strengths', 'comparison')),
    ai_model TEXT DEFAULT 'gpt-4',
    language TEXT DEFAULT 'en',
    confidence_score DECIMAL(3,2),
    strengths JSONB,
    weaknesses JSONB,
    potential JSONB,
    development_areas JSONB,
    statistical_insights JSONB,
    full_report TEXT,
    sources JSONB,
    validated_by TEXT,
    validated_at TIMESTAMPTZ,
    cache_key TEXT,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_ai_player ON ai_analysis(nhl_player_id);
CREATE INDEX idx_ai_date ON ai_analysis(analysis_date DESC);
CREATE INDEX idx_ai_type ON ai_analysis(analysis_type);
CREATE INDEX idx_ai_strengths_gin ON ai_analysis USING GIN(strengths jsonb_path_ops);

-- News events
CREATE TABLE news_events (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    summary TEXT,
    full_text TEXT,
    url TEXT NOT NULL,
    source TEXT NOT NULL,
    author TEXT,
    category TEXT CHECK (category IN ('general', 'injury', 'trade', 'contract', 'milestone', 'performance', 'suspension', 'call_up', 'send_down')),
    sentiment TEXT CHECK (sentiment IN ('positive', 'neutral', 'negative')),
    player_ids INTEGER[],
    team_abbreviations TEXT[],
    published_at TIMESTAMPTZ NOT NULL,
    collected_at TIMESTAMPTZ DEFAULT NOW(),
    language TEXT DEFAULT 'en',
    image_url TEXT,
    tags TEXT[],
    processed BOOLEAN DEFAULT FALSE,
    ai_summary TEXT,
    UNIQUE(url)
);
CREATE INDEX idx_news_published ON news_events(published_at DESC);
CREATE INDEX idx_news_category ON news_events(category);
CREATE INDEX idx_news_player_ids ON news_events USING GIN(player_ids);
CREATE INDEX idx_news_team_abbreviations ON news_events USING GIN(team_abbreviations);

-- System config
CREATE TABLE system_config (
    key TEXT PRIMARY KEY,
    value JSONB NOT NULL,
    description TEXT,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    updated_by TEXT
);

-- API keys
CREATE TABLE api_keys (
    id SERIAL PRIMARY KEY,
    key_hash TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    owner_email TEXT,
    permissions JSONB DEFAULT '{"read": true, "write": false, "admin": false}',
    rate_limit INTEGER DEFAULT 100,
    active BOOLEAN DEFAULT TRUE,
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_api_keys_hash ON api_keys(key_hash);
CREATE INDEX idx_api_keys_active ON api_keys(active) WHERE active = TRUE;

-- Audit log
CREATE TABLE audit_log (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    user_id TEXT,
    action TEXT NOT NULL,
    resource_type TEXT,
    resource_id INTEGER,
    old_value JSONB,
    new_value JSONB,
    ip_address INET,
    user_agent TEXT,
    metadata JSONB
);
CREATE INDEX idx_audit_timestamp ON audit_log(timestamp DESC);
CREATE INDEX idx_audit_user ON audit_log(user_id);
CREATE INDEX idx_audit_resource ON audit_log(resource_type, resource_id);

-- Views
CREATE VIEW v_player_career_summary AS
SELECT
    p.id,
    p.nhl_player_id,
    p.full_name,
    p.position,
    p.current_team_abbreviation,
    COUNT(DISTINCT pss.season) as seasons_played,
    SUM(pss.games_played) as career_games,
    SUM(pss.goals) as career_goals,
    SUM(pss.assists) as career_assists,
    SUM(pss.points) as career_points,
    AVG(pss.points_per_game) as career_ppg,
    MAX(pss.points) as best_season_points
FROM players p
LEFT JOIN player_season_stats pss ON p.nhl_player_id = pss.nhl_player_id
WHERE pss.league = 'NHL'
GROUP BY p.id, p.nhl_player_id, p.full_name, p.position, p.current_team_abbreviation;

-- Seed teams
INSERT INTO teams (nhl_team_id, abbreviation, team_name, location, conference, division) VALUES
(1, 'NJD', 'Devils', 'New Jersey', 'Eastern', 'Metropolitan'),
(2, 'NYI', 'Islanders', 'New York', 'Eastern', 'Metropolitan'),
(3, 'NYR', 'Rangers', 'New York', 'Eastern', 'Metropolitan'),
(4, 'PHI', 'Flyers', 'Philadelphia', 'Eastern', 'Metropolitan'),
(5, 'PIT', 'Penguins', 'Pittsburgh', 'Eastern', 'Metropolitan'),
(6, 'BOS', 'Bruins', 'Boston', 'Eastern', 'Atlantic'),
(7, 'BUF', 'Sabres', 'Buffalo', 'Eastern', 'Atlantic'),
(8, 'MTL', 'Canadiens', 'Montreal', 'Eastern', 'Atlantic'),
(9, 'OTT', 'Senators', 'Ottawa', 'Eastern', 'Atlantic'),
(10, 'TOR', 'Maple Leafs', 'Toronto', 'Eastern', 'Atlantic'),
(12, 'CAR', 'Hurricanes', 'Carolina', 'Eastern', 'Metropolitan'),
(13, 'FLA', 'Panthers', 'Florida', 'Eastern', 'Atlantic'),
(14, 'TBL', 'Lightning', 'Tampa Bay', 'Eastern', 'Atlantic'),
(15, 'WSH', 'Capitals', 'Washington', 'Eastern', 'Metropolitan'),
(16, 'CHI', 'Blackhawks', 'Chicago', 'Western', 'Central'),
(17, 'DET', 'Red Wings', 'Detroit', 'Eastern', 'Atlantic'),
(18, 'NSH', 'Predators', 'Nashville', 'Western', 'Central'),
(19, 'STL', 'Blues', 'St. Louis', 'Western', 'Central'),
(20, 'CGY', 'Flames', 'Calgary', 'Western', 'Pacific'),
(21, 'COL', 'Avalanche', 'Colorado', 'Western', 'Central'),
(22, 'EDM', 'Oilers', 'Edmonton', 'Western', 'Pacific'),
(23, 'VAN', 'Canucks', 'Vancouver', 'Western', 'Pacific'),
(24, 'ANA', 'Ducks', 'Anaheim', 'Western', 'Pacific'),
(25, 'DAL', 'Stars', 'Dallas', 'Western', 'Central'),
(26, 'LAK', 'Kings', 'Los Angeles', 'Western', 'Pacific'),
(28, 'SJS', 'Sharks', 'San Jose', 'Western', 'Pacific'),
(29, 'CBJ', 'Blue Jackets', 'Columbus', 'Eastern', 'Metropolitan'),
(30, 'MIN', 'Wild', 'Minnesota', 'Western', 'Central'),
(52, 'WPG', 'Jets', 'Winnipeg', 'Western', 'Central'),
(53, 'UTA', 'Hockey Club', 'Utah', 'Western', 'Central'),
(54, 'VGK', 'Golden Knights', 'Vegas', 'Western', 'Pacific'),
(55, 'SEA', 'Kraken', 'Seattle', 'Western', 'Pacific');

-- Seed system config
INSERT INTO system_config (key, value, description) VALUES
('nhl_api.rate_limit', '{"requests_per_minute": 60}'::jsonb, 'NHL API rate limiting'),
('cache.ttl_seconds', '{"player_stats": 300, "analysis": 604800, "search": 3600}'::jsonb, 'Cache TTL by type'),
('feature.flags', '{"ai_analysis": true, "real_time_updates": true, "beta_features": false}'::jsonb, 'Feature toggles');
