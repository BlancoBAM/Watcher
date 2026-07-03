mod commands;
mod db;
mod models;
mod recommend;
mod tmdb;

use db::Database;
use recommend::Recommender;
use tmdb::TmdbClient;
use tauri::Manager;

fn get_tmdb_api_key() -> String {
    std::env::var("TMDB_API_KEY").unwrap_or_else(|_| {
        option_env!("TMDB_API_KEY").unwrap_or("").to_string()
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
            let db = Database::new(app_data_dir.clone()).expect("failed to initialize database");

            let recommender = Recommender::new(app_data_dir.clone());

            let tmdb = TmdbClient::new(get_tmdb_api_key());

            app.manage(db);
            app.manage(recommender);
            app.manage(tmdb);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search_titles,
            commands::get_movie_full,
            commands::get_tv_full,
            commands::add_play,
            commands::add_tv_episode_play,
            commands::add_season_plays,
            commands::delete_play,
            commands::toggle_watchlist,
            commands::get_all_plays,
            commands::get_dashboard_stats,
            commands::get_watchlist,
            commands::get_places,
            commands::add_place,
            commands::delete_place,
            commands::get_favorites,
            commands::get_favorite_people,
            commands::update_rating,
            commands::export_data,
            commands::import_data,
            commands::get_movie_keywords,
            commands::get_tv_keywords,
            commands::get_person_details,
            commands::get_recommendations,
            commands::get_recommender_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
