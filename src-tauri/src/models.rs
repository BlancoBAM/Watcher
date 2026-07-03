use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TmdbSearchResult {
    pub id: i64,
    pub title: Option<String>,
    pub name: Option<String>,
    pub media_type: String,
    pub poster_path: Option<String>,
    pub release_date: Option<String>,
    pub first_air_date: Option<String>,
    pub vote_average: Option<f64>,
    pub overview: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Movie {
    pub id: i64,
    pub title: String,
    pub original_title: Option<String>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<i64>,
    pub poster: Option<String>,
    pub tagline: Option<String>,
    pub overview: Option<String>,
    pub original_language: Option<String>,
    pub runtime: Option<i64>,
    pub release_date: Option<String>,
    pub genres: Option<String>,
    pub tmdb_average: Option<f64>,
    pub tmdb_vote_count: Option<i64>,
    pub budget: Option<i64>,
    pub revenue: Option<i64>,
    pub imdb_rating: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TvShow {
    pub id: i64,
    pub tmdb_id: i64,
    pub imdb_id: Option<String>,
    pub name: String,
    pub original_name: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub tagline: Option<String>,
    pub overview: Option<String>,
    pub original_language: Option<String>,
    pub genres: Option<String>,
    pub first_air_date: Option<String>,
    pub last_air_date: Option<String>,
    pub status: Option<String>,
    pub number_of_seasons: Option<i64>,
    pub number_of_episodes: Option<i64>,
    pub tmdb_average: Option<f64>,
    pub tmdb_vote_count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Play {
    pub id: i64,
    pub source_play_id: i64,
    pub source_type: String,
    pub title_id: i64,
    pub episode_id: Option<i64>,
    pub watched_at: Option<String>,
    pub watch_order: i64,
    pub place_id: Option<i64>,
    pub place_name: Option<String>,
    pub is_cinema: Option<bool>,
    pub comment: Option<String>,
    pub title: String,
    pub original_title: Option<String>,
    pub poster: Option<String>,
    pub season_poster: Option<String>,
    pub release_date: Option<String>,
    pub tmdb_id: Option<i64>,
    pub media_type: String,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub episode_name: Option<String>,
    pub user_rating: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayInput {
    pub title_id: i64,
    pub watched_at: Option<String>,
    pub place_id: Option<i64>,
    pub comment: Option<String>,
    pub user_rating: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodePlayInput {
    pub show_id: i64,
    pub episode_id: i64,
    pub watched_at: Option<String>,
    pub place_id: Option<i64>,
    pub comment: Option<String>,
    pub user_rating: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardStats {
    pub movie_total_plays: i64,
    pub tv_total_plays: i64,
    pub movie_unique_titles: i64,
    pub tv_unique_shows: i64,
    pub movie_watchlist_count: i64,
    pub tv_watchlist_count: i64,
    pub total_runtime_minutes: i64,
    pub total_plays: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FavoriteItem {
    pub tmdb_id: i64,
    pub title: String,
    pub poster: Option<String>,
    pub media_type: String,
    pub avg_rating: f64,
    pub play_count: i64,
    pub release_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FavoritePerson {
    pub tmdb_person_id: i64,
    pub name: String,
    pub profile_path: Option<String>,
    pub role_type: String,
    pub avg_rating: f64,
    pub appearance_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WatchlistItem {
    pub id: i64,
    pub title: String,
    pub original_title: Option<String>,
    pub media_type: String,
    pub poster: Option<String>,
    pub release_date: Option<String>,
    pub tmdb_id: Option<i64>,
    pub tmdb_average: Option<f64>,
    pub added_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Place {
    pub id: i64,
    pub name: String,
    pub is_cinema: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreditEntry {
    pub person_id: i64,
    pub tmdb_person_id: i64,
    pub person_name: String,
    pub profile_path: Option<String>,
    pub role_type: String,
    pub character_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeasonInfo {
    pub id: i64,
    pub season_number: i64,
    pub name: Option<String>,
    pub episode_count: Option<i64>,
    pub total_episodes: i64,
    pub watched_episodes: i64,
    pub air_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodeProgress {
    pub total_episodes: i64,
    pub watched_episodes: i64,
    pub total_seasons: i64,
    pub completed_seasons: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeasonPlayInput {
    pub show_id: i64,
    pub season_number: i64,
    pub watched_at: Option<String>,
    pub place_id: Option<i64>,
    pub comment: Option<String>,
    pub user_rating: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportData {
    pub movies: Vec<MovieExport>,
    pub tv_shows: Vec<TvShowExport>,
    pub plays: Vec<PlayExport>,
    pub tv_episode_plays: Vec<TvEpisodePlayExport>,
    pub watchlist: Vec<i64>,
    pub tv_watchlist: Vec<i64>,
    pub places: Vec<Place>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MovieExport {
    pub tmdb_id: i64,
    pub title: String,
    pub original_title: Option<String>,
    pub imdb_id: Option<String>,
    pub user_rating: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TvShowExport {
    pub tmdb_id: i64,
    pub name: String,
    pub original_name: Option<String>,
    pub imdb_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayExport {
    pub tmdb_id: i64,
    pub watched_at: Option<String>,
    pub place_name: Option<String>,
    pub comment: Option<String>,
    pub user_rating: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TvEpisodePlayExport {
    pub tmdb_id: i64,
    pub season_number: i64,
    pub episode_number: i64,
    pub watched_at: Option<String>,
    pub place_name: Option<String>,
    pub comment: Option<String>,
    pub user_rating: Option<f64>,
}
