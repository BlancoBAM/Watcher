CREATE TABLE IF NOT EXISTS movies (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    imdb_id TEXT UNIQUE,
    tmdb_id INTEGER UNIQUE,
    poster TEXT,
    tagline TEXT,
    overview TEXT,
    original_language TEXT,
    runtime INTEGER,
    release_date TEXT,
    tmdb_average REAL,
    tmdb_vote_count INTEGER,
    revenue INTEGER,
    budget INTEGER,
    genres TEXT,
    original_title TEXT,
    imdb_rating REAL,
    imdb_rating_updated_at TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS persons (
    id INTEGER PRIMARY KEY,
    tmdb_person_id INTEGER UNIQUE NOT NULL,
    name TEXT NOT NULL,
    profile_path TEXT,
    known_for TEXT,
    biography TEXT,
    birthday TEXT,
    place_of_birth TEXT,
    deathday TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS credits (
    id INTEGER PRIMARY KEY,
    movie_id INTEGER NOT NULL,
    person_id INTEGER NOT NULL,
    role_type TEXT NOT NULL,
    character_name TEXT,
    display_order INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (movie_id) REFERENCES movies (id) ON DELETE CASCADE,
    FOREIGN KEY (person_id) REFERENCES persons (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS watchlist (
    id INTEGER PRIMARY KEY,
    movie_id INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (movie_id) REFERENCES movies (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS places (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    is_cinema INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS plays (
    id INTEGER PRIMARY KEY,
    movie_id INTEGER NOT NULL,
    watched_at TEXT,
    watch_order INTEGER NOT NULL DEFAULT 1,
    place_id INTEGER,
    comment TEXT,
    FOREIGN KEY (movie_id) REFERENCES movies (id) ON DELETE CASCADE,
    FOREIGN KEY (place_id) REFERENCES places (id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS tv_shows (
    id INTEGER PRIMARY KEY,
    tmdb_id INTEGER NOT NULL UNIQUE,
    imdb_id TEXT,
    name TEXT NOT NULL,
    original_name TEXT,
    poster_path TEXT,
    backdrop_path TEXT,
    tagline TEXT,
    overview TEXT,
    original_language TEXT,
    genres TEXT,
    first_air_date TEXT,
    last_air_date TEXT,
    status TEXT,
    number_of_seasons INTEGER,
    number_of_episodes INTEGER,
    tmdb_average REAL,
    tmdb_vote_count INTEGER,
    imdb_rating REAL,
    imdb_rating_updated_at TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS tv_seasons (
    id INTEGER PRIMARY KEY,
    show_id INTEGER NOT NULL,
    tmdb_season_id INTEGER,
    season_number INTEGER NOT NULL,
    name TEXT,
    overview TEXT,
    air_date TEXT,
    poster_path TEXT,
    episode_count INTEGER,
    vote_average REAL,
    vote_count INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (show_id) REFERENCES tv_shows (id) ON DELETE CASCADE,
    UNIQUE (show_id, season_number)
);

CREATE TABLE IF NOT EXISTS tv_episodes (
    id INTEGER PRIMARY KEY,
    show_id INTEGER NOT NULL,
    season_id INTEGER NOT NULL,
    tmdb_episode_id INTEGER,
    season_number INTEGER NOT NULL,
    episode_number INTEGER NOT NULL,
    name TEXT NOT NULL,
    overview TEXT,
    air_date TEXT,
    runtime INTEGER,
    still_path TEXT,
    director_names TEXT,
    writer_names TEXT,
    vote_average REAL,
    vote_count INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (show_id) REFERENCES tv_shows (id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES tv_seasons (id) ON DELETE CASCADE,
    UNIQUE (show_id, season_number, episode_number)
);

CREATE TABLE IF NOT EXISTS tv_watchlist (
    id INTEGER PRIMARY KEY,
    show_id INTEGER NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (show_id) REFERENCES tv_shows (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS tv_episode_plays (
    id INTEGER PRIMARY KEY,
    show_id INTEGER NOT NULL,
    episode_id INTEGER NOT NULL,
    watched_at TEXT,
    watch_order INTEGER NOT NULL DEFAULT 1,
    place_id INTEGER,
    comment TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (show_id) REFERENCES tv_shows (id) ON DELETE CASCADE,
    FOREIGN KEY (episode_id) REFERENCES tv_episodes (id) ON DELETE CASCADE,
    FOREIGN KEY (place_id) REFERENCES places (id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS tv_credits (
    id INTEGER PRIMARY KEY,
    show_id INTEGER NOT NULL,
    person_id INTEGER NOT NULL,
    role_type TEXT NOT NULL,
    character_name TEXT,
    display_order INTEGER NOT NULL DEFAULT 0,
    episode_count INTEGER,
    FOREIGN KEY (show_id) REFERENCES tv_shows (id) ON DELETE CASCADE,
    FOREIGN KEY (person_id) REFERENCES persons (id) ON DELETE CASCADE
);
