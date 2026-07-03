use rusqlite::{Connection, Result, params};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::models::*;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_data_dir).ok();
        let db_path = app_data_dir.join("watcher.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Database { conn: Mutex::new(conn) };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0)).unwrap_or(0);

        if version < 1 {
            conn.execute_batch(include_str!("../migrations/001_initial.sql"))?;
            conn.pragma_update(None, "user_version", 1)?;
        }
        if version < 2 {
            conn.execute_batch(include_str!("../migrations/002_user_ratings.sql"))?;
            conn.pragma_update(None, "user_version", 2)?;
        }
        if version < 3 {
            conn.execute_batch(include_str!("../migrations/003_embeddings.sql"))?;
            conn.pragma_update(None, "user_version", 3)?;
        }

        Ok(())
    }

    pub fn upsert_movie(&self, details: &serde_json::Value, tmdb_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let title = details["title"].as_str().unwrap_or("Untitled");
        let original_title = details["original_title"].as_str().or(details["title"].as_str());
        let imdb_id = details["imdb_id"].as_str();
        let poster = details["poster_path"].as_str();
        let tagline = details["tagline"].as_str();
        let overview = details["overview"].as_str();
        let lang = details["original_language"].as_str();
        let runtime = details["runtime"].as_i64();
        let release = details["release_date"].as_str();
        let budget = details["budget"].as_i64();
        let revenue = details["revenue"].as_i64();
        let vote_avg = details["vote_average"].as_f64();
        let vote_count = details["vote_count"].as_i64();
        let genres: String = details["genres"].as_array()
            .map(|a| a.iter()
                .filter_map(|g| g["name"].as_str().map(String::from))
                .collect::<Vec<_>>()
                .join(", "))
            .unwrap_or_default();

        conn.execute(
            "INSERT INTO movies (title, original_title, imdb_id, tmdb_id, poster, tagline, overview, original_language, genres, runtime, release_date, budget, tmdb_average, tmdb_vote_count, revenue)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
             ON CONFLICT(tmdb_id) DO UPDATE SET
                title=excluded.title, original_title=excluded.original_title, imdb_id=excluded.imdb_id,
                poster=excluded.poster, tagline=excluded.tagline, overview=excluded.overview,
                original_language=excluded.original_language, genres=excluded.genres,
                runtime=excluded.runtime, release_date=excluded.release_date,
                budget=excluded.budget, tmdb_average=excluded.tmdb_average,
                tmdb_vote_count=excluded.tmdb_vote_count, revenue=excluded.revenue,
                updated_at=CURRENT_TIMESTAMP",
            params![title, original_title, imdb_id, tmdb_id, poster, tagline, overview, lang, genres, runtime, release, budget, vote_avg, vote_count, revenue],
        )?;
        let id: i64 = conn.query_row(
            "SELECT id FROM movies WHERE tmdb_id = ?1", params![tmdb_id], |r| r.get(0)
        )?;
        Ok(id)
    }

    pub fn upsert_tv_show(&self, details: &serde_json::Value, tmdb_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let name = details["name"].as_str().unwrap_or("Untitled");
        let original_name = details["original_name"].as_str().or(details["name"].as_str());
        let imdb_id = details["imdb_id"].as_str()
            .or_else(|| details["external_ids"]["imdb_id"].as_str());
        let poster = details["poster_path"].as_str();
        let backdrop = details["backdrop_path"].as_str();
        let tagline = details["tagline"].as_str();
        let overview = details["overview"].as_str();
        let lang = details["original_language"].as_str();
        let first_air = details["first_air_date"].as_str();
        let last_air = details["last_air_date"].as_str();
        let status = details["status"].as_str();
        let num_seasons = details["number_of_seasons"].as_i64();
        let num_episodes = details["number_of_episodes"].as_i64();
        let vote_avg = details["vote_average"].as_f64();
        let vote_count = details["vote_count"].as_i64();
        let genres: String = details["genres"].as_array()
            .map(|a| a.iter()
                .filter_map(|g| g["name"].as_str().map(String::from))
                .collect::<Vec<_>>()
                .join(", "))
            .unwrap_or_default();

        conn.execute(
            "INSERT INTO tv_shows (tmdb_id, imdb_id, name, original_name, poster_path, backdrop_path, tagline, overview, original_language, genres, first_air_date, last_air_date, status, number_of_seasons, number_of_episodes, tmdb_average, tmdb_vote_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
             ON CONFLICT(tmdb_id) DO UPDATE SET
                imdb_id=excluded.imdb_id, name=excluded.name, original_name=excluded.original_name,
                poster_path=excluded.poster_path, backdrop_path=excluded.backdrop_path,
                tagline=excluded.tagline, overview=excluded.overview,
                original_language=excluded.original_language, genres=excluded.genres,
                first_air_date=excluded.first_air_date, last_air_date=excluded.last_air_date,
                status=excluded.status, number_of_seasons=excluded.number_of_seasons,
                number_of_episodes=excluded.number_of_episodes, tmdb_average=excluded.tmdb_average,
                tmdb_vote_count=excluded.tmdb_vote_count, updated_at=CURRENT_TIMESTAMP",
            params![tmdb_id, imdb_id, name, original_name, poster, backdrop, tagline, overview, lang, genres, first_air, last_air, status, num_seasons, num_episodes, vote_avg, vote_count],
        )?;
        let id: i64 = conn.query_row(
            "SELECT id FROM tv_shows WHERE tmdb_id = ?1", params![tmdb_id], |r| r.get(0)
        )?;
        Ok(id)
    }

    pub fn upsert_person(&self, tmdb_person_id: i64, name: &str, profile_path: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO persons (tmdb_person_id, name, profile_path)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(tmdb_person_id) DO UPDATE SET
                name=excluded.name, profile_path=excluded.profile_path, updated_at=CURRENT_TIMESTAMP",
            params![tmdb_person_id, name, profile_path],
        )?;
        let id: i64 = conn.query_row(
            "SELECT id FROM persons WHERE tmdb_person_id = ?1", params![tmdb_person_id], |r| r.get(0)
        )?;
        Ok(id)
    }

    pub fn save_movie_credits(&self, movie_id: i64, credits: &[CreditEntry]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM credits WHERE movie_id = ?1", params![movie_id])?;
        let mut order = 0i64;
        for c in credits {
            conn.execute(
                "INSERT INTO credits (movie_id, person_id, role_type, character_name, display_order)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![movie_id, c.person_id, c.role_type, c.character_name, order],
            )?;
            order += 1;
        }
        Ok(())
    }

    pub fn save_tv_credits(&self, show_id: i64, credits: &[CreditEntry]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM tv_credits WHERE show_id = ?1", params![show_id])?;
        let mut order = 0i64;
        for c in credits {
            conn.execute(
                "INSERT INTO tv_credits (show_id, person_id, role_type, character_name, display_order)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![show_id, c.person_id, c.role_type, c.character_name, order],
            )?;
            order += 1;
        }
        Ok(())
    }

    pub fn save_seasons(&self, show_id: i64, seasons: &[serde_json::Value]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        for s in seasons {
            let sn = s["season_number"].as_i64().unwrap_or(0);
            conn.execute(
                "INSERT INTO tv_seasons (show_id, tmdb_season_id, season_number, name, overview, air_date, poster_path, episode_count, vote_average, vote_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(show_id, season_number) DO UPDATE SET
                    name=excluded.name, overview=excluded.overview, air_date=excluded.air_date,
                    poster_path=excluded.poster_path, episode_count=excluded.episode_count,
                    vote_average=excluded.vote_average, vote_count=excluded.vote_count,
                    updated_at=CURRENT_TIMESTAMP",
                params![
                    show_id, s["id"].as_i64(), sn, s["name"].as_str(),
                    s["overview"].as_str(), s["air_date"].as_str(), s["poster_path"].as_str(),
                    s["episode_count"].as_i64(), s["vote_average"].as_f64(), s["vote_count"].as_i64()
                ],
            )?;
        }
        Ok(())
    }

    pub fn save_episodes(&self, show_id: i64, season_number: i64, episodes: &[serde_json::Value]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let season_id: Option<i64> = conn.query_row(
            "SELECT id FROM tv_seasons WHERE show_id = ?1 AND season_number = ?2",
            params![show_id, season_number], |r| r.get(0)
        ).ok();

        let Some(sid) = season_id else { return Ok(()) };

        for ep in episodes {
            conn.execute(
                "INSERT INTO tv_episodes (show_id, season_id, tmdb_episode_id, season_number, episode_number, name, overview, air_date, runtime, still_path, vote_average, vote_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                 ON CONFLICT(show_id, season_number, episode_number) DO UPDATE SET
                    name=excluded.name, overview=excluded.overview, air_date=excluded.air_date,
                    runtime=excluded.runtime, still_path=excluded.still_path,
                    vote_average=excluded.vote_average, vote_count=excluded.vote_count,
                    updated_at=CURRENT_TIMESTAMP",
                params![
                    show_id, sid, ep["id"].as_i64(), season_number, ep["episode_number"].as_i64(),
                    ep["name"].as_str(), ep["overview"].as_str(), ep["air_date"].as_str(),
                    ep["runtime"].as_i64(), ep["still_path"].as_str(),
                    ep["vote_average"].as_f64(), ep["vote_count"].as_i64()
                ],
            )?;
        }
        Ok(())
    }

    pub fn add_play(&self, input: &PlayInput) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let watch_order = self.next_watch_order(&conn, input.watched_at.as_deref())?;
        conn.execute(
            "INSERT INTO plays (movie_id, watched_at, watch_order, place_id, comment, user_rating)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![input.title_id, input.watched_at, watch_order, input.place_id, input.comment, input.user_rating],
        )?;
        Ok(())
    }

    pub fn add_tv_episode_play(&self, input: &EpisodePlayInput) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let watch_order = self.next_watch_order(&conn, input.watched_at.as_deref())?;
        conn.execute(
            "INSERT INTO tv_episode_plays (show_id, episode_id, watched_at, watch_order, place_id, comment, user_rating)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![input.show_id, input.episode_id, input.watched_at, watch_order, input.place_id, input.comment, input.user_rating],
        )?;
        Ok(())
    }

    pub fn add_season_plays(&self, input: &SeasonPlayInput) -> Result<()> {
        let episodes: Vec<i64> = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id FROM tv_episodes WHERE show_id = ?1 AND season_number = ?2"
            )?;
            let rows = stmt.query_map(params![input.show_id, input.season_number], |r| r.get(0))?
                .filter_map(|r| r.ok())
                .collect::<Vec<i64>>();
            rows
        };
        for ep_id in episodes {
            self.add_tv_episode_play(&EpisodePlayInput {
                show_id: input.show_id,
                episode_id: ep_id,
                watched_at: input.watched_at.clone(),
                place_id: input.place_id,
                comment: input.comment.clone(),
                user_rating: input.user_rating,
            })?;
        }
        Ok(())
    }

    fn next_watch_order(&self, conn: &Connection, watched_date: Option<&str>) -> Result<i64> {
        let max: i64 = if let Some(date) = watched_date {
            conn.query_row(
                "SELECT COALESCE(MAX(watch_order), 0) FROM (
                    SELECT watch_order FROM plays WHERE watched_at = ?1
                    UNION ALL
                    SELECT watch_order FROM tv_episode_plays WHERE watched_at = ?1
                )",
                params![date], |r| r.get(0)
            ).unwrap_or(0)
        } else {
            conn.query_row(
                "SELECT COALESCE(MAX(watch_order), 0) FROM (
                    SELECT watch_order FROM plays WHERE watched_at IS NULL
                    UNION ALL
                    SELECT watch_order FROM tv_episode_plays WHERE watched_at IS NULL
                )",
                [], |r| r.get(0)
            ).unwrap_or(0)
        };
        Ok(max + 1)
    }

    pub fn delete_play(&self, play_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM plays WHERE id = ?1", params![play_id])?;
        Ok(())
    }

    pub fn delete_tv_episode_play(&self, play_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM tv_episode_plays WHERE id = ?1", params![play_id])?;
        Ok(())
    }

    pub fn get_all_plays(&self) -> Result<Vec<Play>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT plays.id, plays.id as source_play_id, 'movie' as source_type,
                    plays.movie_id as title_id, NULL as episode_id,
                    plays.watched_at, plays.watch_order,
                    plays.place_id, places.name as place_name, places.is_cinema,
                    plays.comment, movies.title, movies.original_title,
                    movies.poster, NULL as season_poster,
                    movies.release_date, movies.tmdb_id, 'movie' as media_type,
                    NULL as season_number, NULL as episode_number, NULL as episode_name,
                    plays.user_rating
             FROM plays JOIN movies ON movies.id = plays.movie_id
             LEFT JOIN places ON plays.place_id = places.id
             UNION ALL
             SELECT tv_episode_plays.id + 2000000000, tv_episode_plays.id, 'tv',
                    tv_shows.id + 1000000000, tv_episodes.id,
                    tv_episode_plays.watched_at, tv_episode_plays.watch_order,
                    tv_episode_plays.place_id, places.name, places.is_cinema,
                    tv_episode_plays.comment, tv_shows.name, tv_shows.original_name,
                    tv_shows.poster_path, tv_seasons.poster_path,
                    tv_shows.first_air_date, tv_shows.tmdb_id, 'tv',
                    tv_episodes.season_number, tv_episodes.episode_number, tv_episodes.name,
                    tv_episode_plays.user_rating
             FROM tv_episode_plays
             JOIN tv_shows ON tv_episode_plays.show_id = tv_shows.id
             JOIN tv_episodes ON tv_episode_plays.episode_id = tv_episodes.id
             LEFT JOIN tv_seasons ON tv_episodes.season_id = tv_seasons.id
             LEFT JOIN places ON tv_episode_plays.place_id = places.id
             ORDER BY watched_at DESC, watch_order DESC"
        )?;

        let plays = stmt.query_map([], |r| {
            Ok(Play {
                id: r.get(0)?, source_play_id: r.get(1)?, source_type: r.get(2)?,
                title_id: r.get(3)?, episode_id: r.get(4)?,
                watched_at: r.get(5)?, watch_order: r.get(6)?,
                place_id: r.get(7)?, place_name: r.get(8)?, is_cinema: r.get(9)?,
                comment: r.get(10)?, title: r.get(11)?, original_title: r.get(12)?,
                poster: r.get(13)?, season_poster: r.get(14)?,
                release_date: r.get(15)?, tmdb_id: r.get(16)?, media_type: r.get(17)?,
                season_number: r.get(18)?, episode_number: r.get(19)?, episode_name: r.get(20)?,
                user_rating: r.get(21)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(plays)
    }

    pub fn get_dashboard_stats(&self) -> Result<DashboardStats> {
        let conn = self.conn.lock().unwrap();
        let movie_plays: i64 = conn.query_row("SELECT COUNT(*) FROM plays", [], |r| r.get(0)).unwrap_or(0);
        let tv_plays: i64 = conn.query_row("SELECT COUNT(*) FROM tv_episode_plays", [], |r| r.get(0)).unwrap_or(0);
        let movie_unique: i64 = conn.query_row("SELECT COUNT(DISTINCT movie_id) FROM plays", [], |r| r.get(0)).unwrap_or(0);
        let tv_unique: i64 = conn.query_row("SELECT COUNT(DISTINCT show_id) FROM tv_episode_plays", [], |r| r.get(0)).unwrap_or(0);
        let movie_wl: i64 = conn.query_row("SELECT COUNT(*) FROM watchlist", [], |r| r.get(0)).unwrap_or(0);
        let tv_wl: i64 = conn.query_row("SELECT COUNT(*) FROM tv_watchlist", [], |r| r.get(0)).unwrap_or(0);
        let movie_runtime: i64 = conn.query_row(
            "SELECT COALESCE(SUM(COALESCE(movies.runtime,0)),0) FROM plays JOIN movies ON movies.id=plays.movie_id",
            [], |r| r.get(0)
        ).unwrap_or(0);
        let tv_runtime: i64 = conn.query_row(
            "SELECT COALESCE(SUM(COALESCE(tv_episodes.runtime,0)),0) FROM tv_episode_plays LEFT JOIN tv_episodes ON tv_episode_plays.episode_id=tv_episodes.id",
            [], |r| r.get(0)
        ).unwrap_or(0);

        Ok(DashboardStats {
            movie_total_plays: movie_plays,
            tv_total_plays: tv_plays,
            movie_unique_titles: movie_unique,
            tv_unique_shows: tv_unique,
            movie_watchlist_count: movie_wl,
            tv_watchlist_count: tv_wl,
            total_runtime_minutes: movie_runtime + tv_runtime,
            total_plays: movie_plays + tv_plays,
        })
    }

    pub fn get_movie_by_tmdb_id(&self, tmdb_id: i64) -> Result<Option<Movie>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, title, original_title, imdb_id, tmdb_id, poster, tagline, overview, original_language, runtime, release_date, genres, tmdb_average, tmdb_vote_count, budget, revenue, imdb_rating FROM movies WHERE tmdb_id = ?1")?;
        let mut rows = stmt.query_map(params![tmdb_id], |r| {
            Ok(Movie {
                id: r.get(0)?, title: r.get(1)?, original_title: r.get(2)?,
                imdb_id: r.get(3)?, tmdb_id: r.get(4)?, poster: r.get(5)?,
                tagline: r.get(6)?, overview: r.get(7)?, original_language: r.get(8)?,
                runtime: r.get(9)?, release_date: r.get(10)?, genres: r.get(11)?,
                tmdb_average: r.get(12)?, tmdb_vote_count: r.get(13)?,
                budget: r.get(14)?, revenue: r.get(15)?, imdb_rating: r.get(16)?,
            })
        })?;
        Ok(rows.next().and_then(|r| r.ok()))
    }

    pub fn get_tv_show_by_tmdb_id(&self, tmdb_id: i64) -> Result<Option<TvShow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, tmdb_id, imdb_id, name, original_name, poster_path, backdrop_path,
                    tagline, overview, original_language, genres, first_air_date, last_air_date,
                    status, number_of_seasons, number_of_episodes, tmdb_average, tmdb_vote_count
             FROM tv_shows WHERE tmdb_id = ?1"
        )?;
        let mut rows = stmt.query_map(params![tmdb_id], |r| {
            Ok(TvShow {
                id: r.get(0)?, tmdb_id: r.get(1)?, imdb_id: r.get(2)?,
                name: r.get(3)?, original_name: r.get(4)?, poster_path: r.get(5)?,
                backdrop_path: r.get(6)?, tagline: r.get(7)?, overview: r.get(8)?,
                original_language: r.get(9)?, genres: r.get(10)?,
                first_air_date: r.get(11)?, last_air_date: r.get(12)?,
                status: r.get(13)?, number_of_seasons: r.get(14)?,
                number_of_episodes: r.get(15)?, tmdb_average: r.get(16)?,
                tmdb_vote_count: r.get(17)?,
            })
        })?;
        Ok(rows.next().and_then(|r| r.ok()))
    }

    pub fn get_movie_credits(&self, movie_id: i64) -> Result<Vec<CreditEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT persons.id, persons.tmdb_person_id, persons.name, persons.profile_path,
                    credits.role_type, credits.character_name
             FROM credits JOIN persons ON credits.person_id = persons.id
             WHERE credits.movie_id = ?1
             ORDER BY credits.display_order"
        )?;
        let rows = stmt.query_map(params![movie_id], |r| {
            Ok(CreditEntry {
                person_id: r.get(0)?, tmdb_person_id: r.get(1)?,
                person_name: r.get(2)?, profile_path: r.get(3)?,
                role_type: r.get(4)?, character_name: r.get(5)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    pub fn get_tv_credits(&self, show_id: i64) -> Result<Vec<CreditEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT persons.id, persons.tmdb_person_id, persons.name, persons.profile_path,
                    tv_credits.role_type, tv_credits.character_name
             FROM tv_credits JOIN persons ON tv_credits.person_id = persons.id
             WHERE tv_credits.show_id = ?1
             ORDER BY tv_credits.display_order"
        )?;
        let rows = stmt.query_map(params![show_id], |r| {
            Ok(CreditEntry {
                person_id: r.get(0)?, tmdb_person_id: r.get(1)?,
                person_name: r.get(2)?, profile_path: r.get(3)?,
                role_type: r.get(4)?, character_name: r.get(5)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    pub fn get_movie_plays(&self, movie_id: i64) -> Result<Vec<Play>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT plays.id, plays.id, 'movie', plays.movie_id, NULL,
                    plays.watched_at, plays.watch_order, plays.place_id,
                    places.name, places.is_cinema, plays.comment,
                    movies.title, movies.original_title, movies.poster,
                    NULL, movies.release_date, movies.tmdb_id, 'movie',
                    NULL, NULL, NULL, plays.user_rating
             FROM plays JOIN movies ON movies.id = plays.movie_id
             LEFT JOIN places ON plays.place_id = places.id
             WHERE plays.movie_id = ?1
             ORDER BY plays.watched_at DESC, plays.watch_order DESC"
        )?;
        let rows = stmt.query_map(params![movie_id], |r| {
            Ok(Play {
                id: r.get(0)?, source_play_id: r.get(1)?, source_type: r.get(2)?,
                title_id: r.get(3)?, episode_id: r.get(4)?,
                watched_at: r.get(5)?, watch_order: r.get(6)?,
                place_id: r.get(7)?, place_name: r.get(8)?, is_cinema: r.get(9)?,
                comment: r.get(10)?, title: r.get(11)?, original_title: r.get(12)?,
                poster: r.get(13)?, season_poster: r.get(14)?,
                release_date: r.get(15)?, tmdb_id: r.get(16)?, media_type: r.get(17)?,
                season_number: r.get(18)?, episode_number: r.get(19)?, episode_name: r.get(20)?,
                user_rating: r.get(21)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    pub fn get_tv_plays(&self, show_id: i64) -> Result<Vec<Play>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT tv_episode_plays.id+2000000000, tv_episode_plays.id, 'tv',
                    tv_shows.id+1000000000, tv_episodes.id,
                    tv_episode_plays.watched_at, tv_episode_plays.watch_order,
                    tv_episode_plays.place_id, places.name, places.is_cinema,
                    tv_episode_plays.comment, tv_shows.name, tv_shows.original_name,
                    tv_shows.poster_path, tv_seasons.poster_path,
                    tv_shows.first_air_date, tv_shows.tmdb_id, 'tv',
                    tv_episodes.season_number, tv_episodes.episode_number, tv_episodes.name,
                    tv_episode_plays.user_rating
             FROM tv_episode_plays
             JOIN tv_shows ON tv_episode_plays.show_id = tv_shows.id
             JOIN tv_episodes ON tv_episode_plays.episode_id = tv_episodes.id
             LEFT JOIN tv_seasons ON tv_episodes.season_id = tv_seasons.id
             LEFT JOIN places ON tv_episode_plays.place_id = places.id
             WHERE tv_episode_plays.show_id = ?1
             ORDER BY tv_episode_plays.watched_at DESC, tv_episode_plays.watch_order DESC"
        )?;
        let rows = stmt.query_map(params![show_id], |r| {
            Ok(Play {
                id: r.get(0)?, source_play_id: r.get(1)?, source_type: r.get(2)?,
                title_id: r.get(3)?, episode_id: r.get(4)?,
                watched_at: r.get(5)?, watch_order: r.get(6)?,
                place_id: r.get(7)?, place_name: r.get(8)?, is_cinema: r.get(9)?,
                comment: r.get(10)?, title: r.get(11)?, original_title: r.get(12)?,
                poster: r.get(13)?, season_poster: r.get(14)?,
                release_date: r.get(15)?, tmdb_id: r.get(16)?, media_type: r.get(17)?,
                season_number: r.get(18)?, episode_number: r.get(19)?, episode_name: r.get(20)?,
                user_rating: r.get(21)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    pub fn is_in_watchlist(&self, tmdb_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let local = conn.query_row(
            "SELECT id FROM movies WHERE tmdb_id = ?1", params![tmdb_id], |r| r.get::<_, i64>(0)
        );
        let Ok(local_id) = local else { return Ok(false) };
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM watchlist WHERE movie_id = ?1",
            params![local_id], |r| r.get(0)
        ).unwrap_or(0);
        Ok(count > 0)
    }

    pub fn is_in_tv_watchlist(&self, tmdb_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let local = conn.query_row(
            "SELECT id FROM tv_shows WHERE tmdb_id = ?1", params![tmdb_id], |r| r.get::<_, i64>(0)
        );
        let Ok(local_id) = local else { return Ok(false) };
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM tv_watchlist WHERE show_id = ?1",
            params![local_id], |r| r.get(0)
        ).unwrap_or(0);
        Ok(count > 0)
    }

    pub fn toggle_watchlist(&self, tmdb_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let local = conn.query_row(
            "SELECT id FROM movies WHERE tmdb_id = ?1", params![tmdb_id], |r| r.get::<_, i64>(0)
        );
        let local_id: i64 = local?;
        let exists: bool = conn.query_row(
            "SELECT 1 FROM watchlist WHERE movie_id = ?1", params![local_id], |r| r.get::<_, i64>(0)
        ).is_ok();
        if exists {
            conn.execute("DELETE FROM watchlist WHERE movie_id = ?1", params![local_id])?;
            Ok(false)
        } else {
            conn.execute("INSERT INTO watchlist (movie_id) VALUES (?1)", params![local_id])?;
            Ok(true)
        }
    }

    pub fn toggle_tv_watchlist(&self, tmdb_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let local_id: i64 = conn.query_row(
            "SELECT id FROM tv_shows WHERE tmdb_id = ?1", params![tmdb_id], |r| r.get::<_, i64>(0)
        )?;
        let exists: bool = conn.query_row(
            "SELECT 1 FROM tv_watchlist WHERE show_id = ?1", params![local_id], |r| r.get::<_, i64>(0)
        ).is_ok();
        if exists {
            conn.execute("DELETE FROM tv_watchlist WHERE show_id = ?1", params![local_id])?;
            Ok(false)
        } else {
            conn.execute("INSERT INTO tv_watchlist (show_id) VALUES (?1)", params![local_id])?;
            Ok(true)
        }
    }

    pub fn get_watchlist(&self) -> Result<Vec<WatchlistItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT movies.id, movies.title, movies.original_title, 'movie',
                    movies.poster, movies.release_date, movies.tmdb_id, movies.tmdb_average,
                    watchlist.created_at
             FROM watchlist JOIN movies ON movies.id = watchlist.movie_id
             UNION ALL
             SELECT tv_shows.id+1000000000, tv_shows.name, tv_shows.original_name, 'tv',
                    tv_shows.poster_path, tv_shows.first_air_date, tv_shows.tmdb_id, tv_shows.tmdb_average,
                    tv_watchlist.created_at
             FROM tv_watchlist JOIN tv_shows ON tv_watchlist.show_id = tv_shows.id
             ORDER BY added_at DESC"
        )?;
        let items = stmt.query_map([], |r| {
            Ok(WatchlistItem {
                id: r.get(0)?, title: r.get(1)?, original_title: r.get(2)?,
                media_type: r.get(3)?, poster: r.get(4)?,
                release_date: r.get(5)?, tmdb_id: r.get(6)?,
                tmdb_average: r.get(7)?, added_at: r.get(8)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(items)
    }

    pub fn get_favorites(&self, min_plays: i64) -> Result<Vec<FavoriteItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT movies.tmdb_id, movies.title, movies.poster, 'movie',
                    AVG(plays.user_rating), COUNT(plays.id), movies.release_date
             FROM plays JOIN movies ON plays.movie_id = movies.id
             WHERE plays.user_rating IS NOT NULL
             GROUP BY movies.tmdb_id
             HAVING COUNT(plays.id) >= ?1 AND AVG(plays.user_rating) >= 5.0
             UNION ALL
             SELECT tv_shows.tmdb_id, tv_shows.name, tv_shows.poster_path, 'tv',
                    AVG(tv_episode_plays.user_rating), COUNT(tv_episode_plays.id),
                    tv_shows.first_air_date
             FROM tv_episode_plays JOIN tv_shows ON tv_episode_plays.show_id = tv_shows.id
             WHERE tv_episode_plays.user_rating IS NOT NULL
             GROUP BY tv_shows.tmdb_id
             HAVING COUNT(tv_episode_plays.id) >= ?1 AND AVG(tv_episode_plays.user_rating) >= 5.0
             ORDER BY avg_rating DESC, play_count DESC"
        )?;
        let items = stmt.query_map(params![min_plays, min_plays], |r| {
            Ok(FavoriteItem {
                tmdb_id: r.get(0)?, title: r.get(1)?, poster: r.get(2)?,
                media_type: r.get(3)?, avg_rating: r.get(4)?,
                play_count: r.get(5)?, release_date: r.get(6)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(items)
    }

    pub fn get_favorite_people(&self, role: &str, min_plays: i64) -> Result<Vec<FavoritePerson>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!(
            "SELECT p.tmdb_person_id, p.name, p.profile_path, '{}',
                    AVG(pl.user_rating), COUNT(DISTINCT pl.title_id)
             FROM (
                SELECT c.person_id, plays.user_rating, plays.movie_id as title_id
                FROM plays JOIN credits c ON c.movie_id = plays.movie_id
                WHERE plays.user_rating IS NOT NULL AND c.role_type = '{}'
                UNION ALL
                SELECT tc.person_id, tep.user_rating, tep.show_id
                FROM tv_episode_plays tep
                JOIN tv_credits tc ON tc.show_id = tep.show_id
                WHERE tep.user_rating IS NOT NULL AND tc.role_type = '{}'
             ) pl
             JOIN persons p ON p.id = pl.person_id
             GROUP BY p.tmdb_person_id
             HAVING COUNT(DISTINCT pl.title_id) >= ?1 AND AVG(pl.user_rating) >= 5.0
             ORDER BY avg_rating DESC, appearance_count DESC",
            role, role, role
        );
        let mut stmt = conn.prepare(&sql)?;
        let items = stmt.query_map(params![min_plays], |r| {
            Ok(FavoritePerson {
                tmdb_person_id: r.get(0)?, name: r.get(1)?, profile_path: r.get(2)?,
                role_type: r.get(3)?, avg_rating: r.get(4)?, appearance_count: r.get(5)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(items)
    }

    pub fn get_seasons(&self, show_id: i64) -> Result<Vec<SeasonInfo>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT ts.id, ts.season_number, ts.name, ts.episode_count,
                    COUNT(DISTINCT te.id) as total_ep,
                    COUNT(DISTINCT tep.episode_id) as watched_ep,
                    ts.air_date
             FROM tv_seasons ts
             LEFT JOIN tv_episodes te ON te.season_id = ts.id
             LEFT JOIN tv_episode_plays tep ON tep.episode_id = te.id AND tep.show_id = ?1
             WHERE ts.show_id = ?1
             GROUP BY ts.id
             ORDER BY ts.season_number"
        )?;
        let seasons = stmt.query_map(params![show_id], |r| {
            Ok(SeasonInfo {
                id: r.get(0)?, season_number: r.get(1)?, name: r.get(2)?,
                episode_count: r.get(3)?, total_episodes: r.get(4)?,
                watched_episodes: r.get(5)?, air_date: r.get(6)?,
            })
        })?.filter_map(|r| r.ok()).collect();
        Ok(seasons)
    }

    pub fn get_episode_progress(&self, show_id: i64) -> Result<EpisodeProgress> {
        let conn = self.conn.lock().unwrap();
        let total_ep: i64 = conn.query_row(
            "SELECT COUNT(*) FROM tv_episodes WHERE show_id = ?1",
            params![show_id], |r| r.get(0)
        ).unwrap_or(0);
        let watched_ep: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT episode_id) FROM tv_episode_plays WHERE show_id = ?1",
            params![show_id], |r| r.get(0)
        ).unwrap_or(0);
        let total_seasons: i64 = conn.query_row(
            "SELECT COUNT(*) FROM tv_seasons WHERE show_id = ?1",
            params![show_id], |r| r.get(0)
        ).unwrap_or(0);
        let completed_seasons: i64 = conn.query_row(
            "SELECT COUNT(*) FROM (
                SELECT ts.id, COUNT(DISTINCT te.id) as total_in_season,
                       COUNT(DISTINCT tep.episode_id) as watched_in_season
                FROM tv_seasons ts
                LEFT JOIN tv_episodes te ON te.season_id = ts.id
                LEFT JOIN tv_episode_plays tep ON tep.episode_id = te.id AND tep.show_id = ?1
                WHERE ts.show_id = ?1
                GROUP BY ts.id
                HAVING total_in_season > 0 AND watched_in_season >= total_in_season
            )", params![show_id, show_id], |r| r.get(0)
        ).unwrap_or(0);
        Ok(EpisodeProgress { total_episodes: total_ep, watched_episodes: watched_ep, total_seasons, completed_seasons })
    }

    pub fn get_places(&self) -> Result<Vec<Place>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, is_cinema FROM places ORDER BY name")?;
        let items = stmt.query_map([], |r| {
            Ok(Place { id: r.get(0)?, name: r.get(1)?, is_cinema: r.get(2)? })
        })?.filter_map(|r| r.ok()).collect();
        Ok(items)
    }

    pub fn add_place(&self, name: &str, is_cinema: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO places (name, is_cinema) VALUES (?1, ?2)",
            params![name, is_cinema])?;
        Ok(())
    }

    pub fn delete_place(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM places WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn export_data(&self) -> Result<ExportData> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare("SELECT tmdb_id, title, original_title, imdb_id FROM movies")?;
        let movies: Vec<MovieExport> = stmt.query_map([], |r| {
            Ok(MovieExport {
                tmdb_id: r.get(0)?, title: r.get(1)?,
                original_title: r.get(2)?, imdb_id: r.get(3)?,
                user_rating: None,
            })
        })?.filter_map(|r| r.ok()).collect();

        let mut stmt = conn.prepare("SELECT tmdb_id, name, original_name, imdb_id FROM tv_shows")?;
        let tv_shows: Vec<TvShowExport> = stmt.query_map([], |r| {
            Ok(TvShowExport {
                tmdb_id: r.get(0)?, name: r.get(1)?,
                original_name: r.get(2)?, imdb_id: r.get(3)?,
            })
        })?.filter_map(|r| r.ok()).collect();

        let mut stmt = conn.prepare(
            "SELECT m.tmdb_id, p.watched_at, pl.name, p.comment, p.user_rating
             FROM plays p JOIN movies m ON p.movie_id = m.id
             LEFT JOIN places pl ON p.place_id = pl.id"
        )?;
        let plays: Vec<PlayExport> = stmt.query_map([], |r| {
            Ok(PlayExport {
                tmdb_id: r.get(0)?, watched_at: r.get(1)?,
                place_name: r.get(2)?, comment: r.get(3)?, user_rating: r.get(4)?,
            })
        })?.filter_map(|r| r.ok()).collect();

        let mut stmt = conn.prepare(
            "SELECT s.tmdb_id, e.season_number, e.episode_number, tep.watched_at, pl.name, tep.comment, tep.user_rating
             FROM tv_episode_plays tep
             JOIN tv_shows s ON tep.show_id = s.id
             JOIN tv_episodes e ON tep.episode_id = e.id
             LEFT JOIN places pl ON tep.place_id = pl.id"
        )?;
        let tv_episode_plays: Vec<TvEpisodePlayExport> = stmt.query_map([], |r| {
            Ok(TvEpisodePlayExport {
                tmdb_id: r.get(0)?, season_number: r.get(1)?, episode_number: r.get(2)?,
                watched_at: r.get(3)?, place_name: r.get(4)?, comment: r.get(5)?,
                user_rating: r.get(6)?,
            })
        })?.filter_map(|r| r.ok()).collect();

        let mut stmt = conn.prepare("SELECT tmdb_id FROM watchlist JOIN movies ON watchlist.movie_id = movies.id")?;
        let watchlist: Vec<i64> = stmt.query_map([], |r| r.get(0))?.filter_map(|r| r.ok()).collect();

        let mut stmt = conn.prepare("SELECT tmdb_id FROM tv_watchlist JOIN tv_shows ON tv_watchlist.show_id = tv_shows.id")?;
        let tv_watchlist: Vec<i64> = stmt.query_map([], |r| r.get(0))?.filter_map(|r| r.ok()).collect();

        let mut stmt = conn.prepare("SELECT id, name, is_cinema FROM places")?;
        let places: Vec<Place> = stmt.query_map([], |r| {
            Ok(Place { id: r.get(0)?, name: r.get(1)?, is_cinema: r.get(2)? })
        })?.filter_map(|r| r.ok()).collect();

        Ok(ExportData { movies, tv_shows, plays, tv_episode_plays, watchlist, tv_watchlist, places })
    }

    pub fn import_data(&self, data: &ExportData) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("BEGIN IMMEDIATE")?;

        // Import places
        for p in &data.places {
            conn.execute(
                "INSERT OR IGNORE INTO places (name, is_cinema) VALUES (?1, ?2)",
                params![p.name, p.is_cinema],
            )?;
        }

        conn.execute_batch("COMMIT")?;
        Ok(())
    }

    pub fn update_play_rating(&self, play_id: i64, rating: Option<f64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE plays SET user_rating = ?1 WHERE id = ?2",
            params![rating, play_id])?;
        Ok(())
    }

    pub fn update_tv_play_rating(&self, play_id: i64, rating: Option<f64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE tv_episode_plays SET user_rating = ?1 WHERE id = ?2",
            params![rating, play_id])?;
        Ok(())
    }

    pub fn get_all_watched_keys(&self) -> Result<Vec<(i64, String)>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut keys = Vec::new();

        let mut stmt = conn.prepare(
            "SELECT DISTINCT m.tmdb_id, 'movie' FROM plays p JOIN movies m ON m.id = p.movie_id WHERE m.tmdb_id IS NOT NULL"
        ).map_err(|e| e.to_string())?;
        keys.extend(stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?.filter_map(|r| r.ok()));

        let mut stmt2 = conn.prepare(
            "SELECT DISTINCT s.tmdb_id, 'tv' FROM tv_episode_plays tep JOIN tv_shows s ON s.id = tep.show_id WHERE s.tmdb_id IS NOT NULL"
        ).map_err(|e| e.to_string())?;
        keys.extend(stmt2.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?.filter_map(|r| r.ok()));

        Ok(keys)
    }

    pub fn get_high_rated_person_ids(&self) -> Result<(Vec<i64>, Vec<i64>), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let directors: Vec<i64> = conn.prepare(
            "SELECT DISTINCT p.tmdb_person_id FROM plays pl
             JOIN credits c ON c.movie_id = pl.movie_id
             JOIN persons p ON p.id = c.person_id
             WHERE pl.user_rating >= 7.0 AND c.role_type = 'director'
             UNION
             SELECT DISTINCT p.tmdb_person_id FROM tv_episode_plays tep
             JOIN tv_credits tc ON tc.show_id = tep.show_id
             JOIN persons p ON p.id = tc.person_id
             WHERE tep.user_rating >= 7.0 AND tc.role_type = 'director'"
        ).map_err(|e| e.to_string())?
            .query_map([], |r| r.get::<_, i64>(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok()).collect();

        let actors: Vec<i64> = conn.prepare(
            "SELECT DISTINCT p.tmdb_person_id FROM plays pl
             JOIN credits c ON c.movie_id = pl.movie_id
             JOIN persons p ON p.id = c.person_id
             WHERE pl.user_rating >= 7.0 AND c.role_type = 'actor'
             UNION
             SELECT DISTINCT p.tmdb_person_id FROM tv_episode_plays tep
             JOIN tv_credits tc ON tc.show_id = tep.show_id
             JOIN persons p ON p.id = tc.person_id
             WHERE tep.user_rating >= 7.0 AND tc.role_type = 'actor'"
        ).map_err(|e| e.to_string())?
            .query_map([], |r| r.get::<_, i64>(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok()).collect();

        Ok((directors, actors))
    }

    pub fn save_embedding(&self, tmdb_id: i64, media_type: &str, embedding: &[f32]) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        conn.execute(
            "INSERT OR REPLACE INTO movie_embeddings (tmdb_id, media_type, embedding) VALUES (?1, ?2, ?3)",
            params![tmdb_id, media_type, bytes],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_all_embedding_data(&self) -> Result<Vec<(i64, Vec<f32>)>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT tmdb_id, embedding FROM movie_embeddings WHERE media_type = 'movie'"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([], |r| {
            let tmdb_id: i64 = r.get(0)?;
            let bytes: Vec<u8> = r.get(1)?;
            let emb = bytes.chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            Ok((tmdb_id, emb))
        }).map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }
}
