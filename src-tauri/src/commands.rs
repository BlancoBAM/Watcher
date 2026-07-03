use tauri::State;
use crate::db::Database;
use crate::recommend::{Candidate, Recommendation, Recommender};
use crate::tmdb::TmdbClient;
use crate::models::*;
use std::collections::HashSet;

#[tauri::command]
pub async fn search_titles(query: String, tmdb: State<'_, TmdbClient>) -> Result<Vec<TmdbSearchResult>, String> {
    let results = tmdb.search_multi(&query).await?;
    let items: Vec<TmdbSearchResult> = results.as_array()
        .map(|arr| arr.iter().filter_map(|v| {
            let media_type = v["media_type"].as_str()?;
            if media_type != "movie" && media_type != "tv" { return None; }
            Some(TmdbSearchResult {
                id: v["id"].as_i64().unwrap_or(0),
                title: v["title"].as_str().or(v["name"].as_str()).map(String::from),
                name: v["name"].as_str().map(String::from),
                media_type: media_type.to_string(),
                poster_path: v["poster_path"].as_str().map(String::from),
                release_date: v["release_date"].as_str().or(v["first_air_date"].as_str()).map(String::from),
                first_air_date: v["first_air_date"].as_str().map(String::from),
                vote_average: v["vote_average"].as_f64(),
                overview: v["overview"].as_str().map(String::from),
            })
        }).collect())
        .unwrap_or_default();
    Ok(items)
}

#[tauri::command]
pub async fn get_movie_full(tmdb_id: i64, db: State<'_, Database>, tmdb: State<'_, TmdbClient>, recommender: State<'_, Recommender>) -> Result<serde_json::Value, String> {
    let details = tmdb.get_movie_details(tmdb_id).await?;
    let credits_json = tmdb.get_movie_credits(tmdb_id).await?;
    let movie_id = db.upsert_movie(&details, tmdb_id).map_err(|e| e.to_string())?;

    let mut credits = Vec::new();
    if let Some(crew) = credits_json["crew"].as_array() {
        for member in crew {
            let role = member["job"].as_str().unwrap_or("");
            let role_type = match role {
                "Director" => "director",
                "Producer" => "producer",
                "Director of Photography" | "Cinematography" => "cinematographer",
                "Original Music Composer" | "Music" | "Composer" => "music_composer",
                _ => continue,
            };
            let pid = member["id"].as_i64().unwrap_or(0);
            let person_id = db.upsert_person(pid, member["name"].as_str().unwrap_or(""), member["profile_path"].as_str())
                .map_err(|e| e.to_string())?;
            credits.push(CreditEntry {
                person_id, tmdb_person_id: pid,
                person_name: member["name"].as_str().unwrap_or("").to_string(),
                profile_path: member["profile_path"].as_str().map(String::from),
                role_type: role_type.to_string(),
                character_name: None,
            });
        }
    }
    if let Some(cast) = credits_json["cast"].as_array() {
        for member in cast {
            let pid = member["id"].as_i64().unwrap_or(0);
            let person_id = db.upsert_person(pid, member["name"].as_str().unwrap_or(""), member["profile_path"].as_str())
                .map_err(|e| e.to_string())?;
            credits.push(CreditEntry {
                person_id, tmdb_person_id: pid,
                person_name: member["name"].as_str().unwrap_or("").to_string(),
                profile_path: member["profile_path"].as_str().map(String::from),
                role_type: "actor".to_string(),
                character_name: member["character"].as_str().map(String::from),
            });
        }
    }
    db.save_movie_credits(movie_id, &credits).map_err(|e| e.to_string())?;

    let movie = db.get_movie_by_tmdb_id(tmdb_id).map_err(|e| e.to_string())?.unwrap();
    let db_credits = db.get_movie_credits(movie_id).map_err(|e| e.to_string())?;
    let plays = db.get_movie_plays(movie_id).map_err(|e| e.to_string())?;
    let in_watchlist = db.is_in_watchlist(tmdb_id).map_err(|e| e.to_string())?;

    if recommender.is_loaded() {
        let text = build_embedding_text_from_json(&details);
        if let Ok(emb) = recommender.encode(&text) {
            let _ = db.save_embedding(tmdb_id, "movie", &emb);
        }
    }

    Ok(serde_json::json!({
        "movie": movie,
        "credits": db_credits,
        "plays": plays,
        "in_watchlist": in_watchlist,
        "tmdb_id": tmdb_id,
    }))
}

#[tauri::command]
pub async fn get_tv_full(tmdb_id: i64, db: State<'_, Database>, tmdb: State<'_, TmdbClient>, recommender: State<'_, Recommender>) -> Result<serde_json::Value, String> {
    let details = tmdb.get_tv_details(tmdb_id).await?;
    let credits_json = tmdb.get_tv_credits(tmdb_id).await?;
    let show_id = db.upsert_tv_show(&details, tmdb_id).map_err(|e| e.to_string())?;

    if let Some(seasons) = details["seasons"].as_array() {
        db.save_seasons(show_id, seasons).map_err(|e| e.to_string())?;
        for s in seasons {
            let sn = s["season_number"].as_i64().unwrap_or(0);
            if sn < 0 { continue; }
            if let Ok(season_details) = tmdb.get_tv_season(tmdb_id, sn).await {
                if let Some(episodes) = season_details["episodes"].as_array() {
                    db.save_episodes(show_id, sn, episodes).map_err(|e| e.to_string())?;
                }
            }
        }
    }

    let mut credits = Vec::new();
    let mut seen = std::collections::HashSet::new();
    if let Some(crew) = credits_json["crew"].as_array() {
        for member in crew {
            let role = member["job"].as_str().unwrap_or("");
            let role_type = match role {
                "Director" => "director",
                "Producer" => "producer",
                "Director of Photography" | "Cinematography" => "cinematographer",
                "Original Music Composer" | "Music" | "Composer" => "music_composer",
                _ => continue,
            };
            let pid = member["id"].as_i64().unwrap_or(0);
            let person_id = db.upsert_person(pid, member["name"].as_str().unwrap_or(""), member["profile_path"].as_str())
                .map_err(|e| e.to_string())?;
            let key = (person_id, role_type.to_string());
            if seen.contains(&key) { continue; }
            seen.insert(key);
            credits.push(CreditEntry {
                person_id, tmdb_person_id: pid,
                person_name: member["name"].as_str().unwrap_or("").to_string(),
                profile_path: member["profile_path"].as_str().map(String::from),
                role_type: role_type.to_string(),
                character_name: None,
            });
        }
    }
    if let Some(cast) = credits_json["cast"].as_array() {
        for member in cast {
            let pid = member["id"].as_i64().unwrap_or(0);
            let person_id = db.upsert_person(pid, member["name"].as_str().unwrap_or(""), member["profile_path"].as_str())
                .map_err(|e| e.to_string())?;
            let char_name = member["character"].as_str().or_else(|| {
                member["roles"].as_array().and_then(|roles| roles.first()?.get("character").and_then(|c| c.as_str()))
            });
            credits.push(CreditEntry {
                person_id, tmdb_person_id: pid,
                person_name: member["name"].as_str().unwrap_or("").to_string(),
                profile_path: member["profile_path"].as_str().map(String::from),
                role_type: "actor".to_string(),
                character_name: char_name.map(String::from),
            });
        }
    }
    db.save_tv_credits(show_id, &credits).map_err(|e| e.to_string())?;

    let show = db.get_tv_show_by_tmdb_id(tmdb_id).map_err(|e| e.to_string())?.unwrap();
    let db_credits = db.get_tv_credits(show_id).map_err(|e| e.to_string())?;
    let plays = db.get_tv_plays(show_id).map_err(|e| e.to_string())?;
    let seasons = db.get_seasons(show_id).map_err(|e| e.to_string())?;
    let progress = db.get_episode_progress(show_id).map_err(|e| e.to_string())?;
    let in_watchlist = db.is_in_tv_watchlist(tmdb_id).map_err(|e| e.to_string())?;

    if recommender.is_loaded() {
        let text = build_embedding_text_from_json(&details);
        if let Ok(emb) = recommender.encode(&text) {
            let _ = db.save_embedding(tmdb_id, "tv", &emb);
        }
    }

    Ok(serde_json::json!({
        "show": show,
        "credits": db_credits,
        "plays": plays,
        "seasons": seasons,
        "episode_progress": progress,
        "in_watchlist": in_watchlist,
        "tmdb_id": tmdb_id,
    }))
}

#[tauri::command]
pub async fn add_play(input: PlayInput, db: State<'_, Database>) -> Result<(), String> {
    db.add_play(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_tv_episode_play(input: EpisodePlayInput, db: State<'_, Database>) -> Result<(), String> {
    db.add_tv_episode_play(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_season_plays(input: SeasonPlayInput, db: State<'_, Database>) -> Result<(), String> {
    db.add_season_plays(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_play(play_id: i64, source_type: String, db: State<'_, Database>) -> Result<(), String> {
    if source_type == "tv" {
        db.delete_tv_episode_play(play_id).map_err(|e| e.to_string())
    } else {
        db.delete_play(play_id).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn toggle_watchlist(tmdb_id: i64, media_type: String, db: State<'_, Database>) -> Result<bool, String> {
    if media_type == "tv" {
        db.toggle_tv_watchlist(tmdb_id).map_err(|e| e.to_string())
    } else {
        db.toggle_watchlist(tmdb_id).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn get_all_plays(db: State<'_, Database>) -> Result<Vec<Play>, String> {
    db.get_all_plays().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_dashboard_stats(db: State<'_, Database>) -> Result<DashboardStats, String> {
    db.get_dashboard_stats().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_watchlist(db: State<'_, Database>) -> Result<Vec<WatchlistItem>, String> {
    db.get_watchlist().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_places(db: State<'_, Database>) -> Result<Vec<Place>, String> {
    db.get_places().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_place(name: String, is_cinema: bool, db: State<'_, Database>) -> Result<(), String> {
    db.add_place(&name, is_cinema).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_place(id: i64, db: State<'_, Database>) -> Result<(), String> {
    db.delete_place(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_favorites(min_plays: i64, db: State<'_, Database>) -> Result<Vec<FavoriteItem>, String> {
    db.get_favorites(min_plays).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_favorite_people(role: String, min_plays: i64, db: State<'_, Database>) -> Result<Vec<FavoritePerson>, String> {
    db.get_favorite_people(&role, min_plays).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_rating(play_id: i64, rating: Option<f64>, source_type: String, db: State<'_, Database>) -> Result<(), String> {
    if source_type == "tv" {
        db.update_tv_play_rating(play_id, rating).map_err(|e| e.to_string())
    } else {
        db.update_play_rating(play_id, rating).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn export_data(db: State<'_, Database>) -> Result<ExportData, String> {
    db.export_data().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_data(data: ExportData, db: State<'_, Database>) -> Result<(), String> {
    db.import_data(&data).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_movie_keywords(tmdb_id: i64, tmdb: State<'_, TmdbClient>) -> Result<Vec<String>, String> {
    tmdb.get_movie_keywords(tmdb_id).await
}

#[tauri::command]
pub async fn get_tv_keywords(tmdb_id: i64, tmdb: State<'_, TmdbClient>) -> Result<Vec<String>, String> {
    tmdb.get_tv_keywords(tmdb_id).await
}

#[tauri::command]
pub async fn get_person_details(person_id: i64, tmdb: State<'_, TmdbClient>) -> Result<serde_json::Value, String> {
    let details = tmdb.get_person_details(person_id).await?;
    let credits = tmdb.get_person_combined_credits(person_id).await?;
    Ok(serde_json::json!({
        "details": details,
        "credits": credits,
    }))
}

#[tauri::command]
pub async fn get_recommendations(
    prompt: Option<String>,
    reference_tmdb_id: Option<i64>,
    reference_media_type: Option<String>,
    db: State<'_, Database>,
    tmdb: State<'_, TmdbClient>,
    recommender: State<'_, Recommender>,
) -> Result<Vec<Recommendation>, String> {
    let watched: Vec<(i64, String)> = db.get_all_watched_keys().map_err(|e| e.to_string())?;
    let watched_set: HashSet<(i64, String)> = watched.into_iter().collect();

    let favs = db.get_favorites(1).unwrap_or_default();
    let top_rated_ids: Vec<i64> = favs.iter()
        .filter(|f| f.avg_rating >= 7.0)
        .take(5)
        .map(|f| f.tmdb_id)
        .collect();

    let mut seen_ids = watched_set.clone();
    let mut candidates: Vec<Candidate> = Vec::new();

    // Strategy 1: If reference title provided, get TMDB recommendations for it
    if let Some(ref_id) = reference_tmdb_id {
        let mt = reference_media_type.as_deref().unwrap_or("movie");
        let recs = if mt == "tv" {
            tmdb.get_tv_recommendations(ref_id).await
        } else {
            tmdb.get_movie_recommendations(ref_id).await
        };
        if let Ok(items) = recs {
            if let Some(arr) = items.as_array() {
                for item in arr {
                    let id = item["id"].as_i64().unwrap_or(0);
                    let key = (id, mt.to_string());
                    if seen_ids.contains(&key) { continue; }
                    seen_ids.insert(key);
                    candidates.push(Candidate {
                        tmdb_id: id,
                        media_type: mt.to_string(),
                        title: item["title"].as_str().or(item["name"].as_str()).unwrap_or("").to_string(),
                        poster: item["poster_path"].as_str().map(String::from),
                        release_date: item["release_date"].as_str().or(item["first_air_date"].as_str()).map(String::from),
                        overview: item["overview"].as_str().map(String::from),
                        tmdb_vote_avg: item["vote_average"].as_f64().unwrap_or(0.0) / 10.0,
                        score: 0.0,
                        match_reason: "Similar to what you're watching".to_string(),
                    });
                }
            }
        }
    }

    // Strategy 2: Get TMDB recommendations for user's top-rated movies
    let mut rec_count = 0;
    for movie_id in &top_rated_ids {
        if rec_count >= 20 { break; }
        if let Ok(items) = tmdb.get_movie_recommendations(*movie_id).await {
            if let Some(arr) = items.as_array() {
                for item in arr {
                    if rec_count >= 20 { break; }
                    let id = item["id"].as_i64().unwrap_or(0);
                    let key = (id, "movie".to_string());
                    if seen_ids.contains(&key) { continue; }
                    seen_ids.insert(key);
                    candidates.push(Candidate {
                        tmdb_id: id,
                        media_type: "movie".to_string(),
                        title: item["title"].as_str().unwrap_or("").to_string(),
                        poster: item["poster_path"].as_str().map(String::from),
                        release_date: item["release_date"].as_str().map(String::from),
                        overview: item["overview"].as_str().map(String::from),
                        tmdb_vote_avg: item["vote_average"].as_f64().unwrap_or(0.0) / 10.0,
                        score: 0.0,
                        match_reason: "Recommended based on your favorites".to_string(),
                    });
                    rec_count += 1;
                }
            }
        }
    }

    // Strategy 3: If text prompt given, search TMDB for matching content
    if let Some(ref text) = prompt {
        if !text.is_empty() {
            let search_results = tmdb.search_multi(text).await;
            if let Ok(items) = search_results {
                if let Some(arr) = items.as_array() {
                    for item in arr.iter().take(5) {
                        let id = item["id"].as_i64().unwrap_or(0);
                        let mt = item["media_type"].as_str().unwrap_or("movie");
                        if mt != "movie" && mt != "tv" { continue; }
                        let key = (id, mt.to_string());
                        if seen_ids.contains(&key) { continue; }
                        seen_ids.insert(key);
                        candidates.push(Candidate {
                            tmdb_id: id,
                            media_type: mt.to_string(),
                            title: item["title"].as_str().or(item["name"].as_str()).unwrap_or("").to_string(),
                            poster: item["poster_path"].as_str().map(String::from),
                            release_date: item["release_date"].as_str().or(item["first_air_date"].as_str()).map(String::from),
                            overview: item["overview"].as_str().map(String::from),
                            tmdb_vote_avg: item["vote_average"].as_f64().unwrap_or(0.0) / 10.0,
                            score: 0.0,
                            match_reason: "Matches your search".to_string(),
                        });
                    }
                }
            }
        }
    }

    // Strategy 4: Discover based on favorite directors/actors
    let high_rated = db.get_high_rated_person_ids().unwrap_or_default();
    for person_id in &high_rated.0 {
        if let Ok(items) = tmdb.discover_by_person(*person_id, "movie", "director").await {
            if let Some(arr) = items.as_array() {
                for item in arr.iter().take(3) {
                    let id = item["id"].as_i64().unwrap_or(0);
                    let key = (id, "movie".to_string());
                    if seen_ids.contains(&key) { continue; }
                    seen_ids.insert(key);
                    candidates.push(Candidate {
                        tmdb_id: id,
                        media_type: "movie".to_string(),
                        title: item["title"].as_str().unwrap_or("").to_string(),
                        poster: item["poster_path"].as_str().map(String::from),
                        release_date: item["release_date"].as_str().map(String::from),
                        overview: item["overview"].as_str().map(String::from),
                        tmdb_vote_avg: item["vote_average"].as_f64().unwrap_or(0.0) / 10.0,
                        score: 0.0,
                        match_reason: "From a director you love".to_string(),
                    });
                }
            }
        }
    }

    // BERT re-ranking
    let search_text = prompt.as_deref().unwrap_or("");
    let has_prompt = !search_text.is_empty();
    let has_bert = recommender.is_loaded();

    if has_prompt && has_bert {
        let cached = db.get_all_embedding_data().unwrap_or_default();
        recommender.rank_with_bert(search_text, &mut candidates, &cached);
    }

    // Convert to final output, sort, truncate
    let mut results: Vec<Recommendation> = candidates.into_iter().map(|c| Recommendation {
        tmdb_id: c.tmdb_id,
        media_type: c.media_type,
        title: c.title,
        poster: c.poster,
        release_date: c.release_date,
        score: c.score,
        match_reason: c.match_reason,
    }).collect();

    let mut seen_ids_dedup = HashSet::new();
    results.retain(|r| seen_ids_dedup.insert(r.tmdb_id));

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(24);

    Ok(results)
}

#[tauri::command]
pub async fn get_recommender_status(recommender: State<'_, Recommender>) -> Result<bool, String> {
    Ok(recommender.is_loaded())
}

fn build_embedding_text_from_json(details: &serde_json::Value) -> String {
    let mut parts = Vec::new();
    if let Some(t) = details["title"].as_str().or(details["name"].as_str()) {
        parts.push(t.to_string());
    }
    if let Some(t) = details["tagline"].as_str() {
        if !t.is_empty() { parts.push(t.to_string()); }
    }
    if let Some(o) = details["overview"].as_str() {
        if !o.is_empty() { parts.push(o.to_string()); }
    }
    parts.join(" ")
}
