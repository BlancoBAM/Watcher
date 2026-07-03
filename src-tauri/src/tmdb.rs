use reqwest::Client;
use serde_json::Value;

const TMDB_BASE_URL: &str = "https://api.themoviedb.org/3";

pub struct TmdbClient {
    client: Client,
    api_key: String,
}

impl TmdbClient {
    pub fn new(api_key: String) -> Self {
        TmdbClient {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn search_multi(&self, query: &str) -> Result<Value, String> {
        let url = format!(
            "{}/search/multi?query={}&include_adult=false&language=en-US&page=1&api_key={}",
            TMDB_BASE_URL, urlencoding(query), self.api_key
        );
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        let json: Value = resp.json().await.map_err(|e| e.to_string())?;
        Ok(json["results"].clone())
    }

    pub async fn get_movie_details(&self, tmdb_id: i64) -> Result<Value, String> {
        let url = format!(
            "{}/movie/{}?language=en-US&api_key={}",
            TMDB_BASE_URL, tmdb_id, self.api_key
        );
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn get_tv_details(&self, tmdb_id: i64) -> Result<Value, String> {
        let url = format!(
            "{}/tv/{}?language=en-US&append_to_response=external_ids&api_key={}",
            TMDB_BASE_URL, tmdb_id, self.api_key
        );
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn get_movie_credits(&self, tmdb_id: i64) -> Result<Value, String> {
        let url = format!(
            "{}/movie/{}/credits?language=en-US&api_key={}",
            TMDB_BASE_URL, tmdb_id, self.api_key
        );
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn get_tv_credits(&self, tmdb_id: i64) -> Result<Value, String> {
        let url = format!(
            "{}/tv/{}/aggregate_credits?language=en-US&api_key={}",
            TMDB_BASE_URL, tmdb_id, self.api_key
        );
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        let json: Value = resp.json().await.map_err(|e| e.to_string())?;
        // Fallback to regular credits if aggregate is empty
        if json["cast"].as_array().map(|a| a.is_empty()).unwrap_or(true)
            && json["crew"].as_array().map(|a| a.is_empty()).unwrap_or(true) {
            let url2 = format!(
                "{}/tv/{}/credits?language=en-US&api_key={}",
                TMDB_BASE_URL, tmdb_id, self.api_key
            );
            let resp2 = self.client.get(&url2).send().await.map_err(|e| e.to_string())?;
            return resp2.json().await.map_err(|e| e.to_string());
        }
        Ok(json)
    }

    pub async fn get_tv_season(&self, tmdb_id: i64, season: i64) -> Result<Value, String> {
        let url = format!(
            "{}/tv/{}/season/{}?language=en-US&api_key={}",
            TMDB_BASE_URL, tmdb_id, season, self.api_key
        );
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn get_person_details(&self, person_id: i64) -> Result<Value, String> {
        let url = format!(
            "{}/person/{}?language=en-US&api_key={}",
            TMDB_BASE_URL, person_id, self.api_key
        );
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn get_movie_keywords(&self, tmdb_id: i64) -> Result<Vec<String>, String> {
        let url = format!(
            "{}/movie/{}/keywords?api_key={}",
            TMDB_BASE_URL, tmdb_id, self.api_key
        );
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        let json: Value = resp.json().await.map_err(|e| e.to_string())?;
        let keywords = json["keywords"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|k| k["name"].as_str().map(String::from))
                .collect())
            .unwrap_or_default();
        Ok(keywords)
    }

    pub async fn get_tv_keywords(&self, tmdb_id: i64) -> Result<Vec<String>, String> {
        let url = format!(
            "{}/tv/{}/keywords?api_key={}",
            TMDB_BASE_URL, tmdb_id, self.api_key
        );
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        let json: Value = resp.json().await.map_err(|e| e.to_string())?;
        let keywords = json["results"].as_array()
            .map(|arr| arr.iter()
                .filter_map(|k| k["name"].as_str().map(String::from))
                .collect())
            .unwrap_or_default();
        Ok(keywords)
    }

    pub async fn get_person_combined_credits(&self, person_id: i64) -> Result<Value, String> {
        let url = format!(
            "{}/person/{}/combined_credits?language=en-US&api_key={}",
            TMDB_BASE_URL, person_id, self.api_key
        );
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn get_movie_recommendations(&self, tmdb_id: i64) -> Result<Value, String> {
        let url = format!(
            "{}/movie/{}/recommendations?language=en-US&page=1&api_key={}",
            TMDB_BASE_URL, tmdb_id, self.api_key
        );
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        let json: Value = resp.json().await.map_err(|e| e.to_string())?;
        Ok(json["results"].clone())
    }

    pub async fn get_tv_recommendations(&self, tmdb_id: i64) -> Result<Value, String> {
        let url = format!(
            "{}/tv/{}/recommendations?language=en-US&page=1&api_key={}",
            TMDB_BASE_URL, tmdb_id, self.api_key
        );
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        let json: Value = resp.json().await.map_err(|e| e.to_string())?;
        Ok(json["results"].clone())
    }

    pub async fn discover_by_person(&self, person_id: i64, media_type: &str, role: &str) -> Result<Value, String> {
        let param = if role == "actor" { "with_cast" } else { "with_crew" };
        let url = format!(
            "{}/discover/{}?{}={}&sort_by=vote_average.desc&vote_count.gte=50&language=en-US&page=1&api_key={}",
            TMDB_BASE_URL, media_type, param, person_id, self.api_key
        );
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        let json: Value = resp.json().await.map_err(|e| e.to_string())?;
        Ok(json["results"].clone())
    }

}

fn urlencoding(s: &str) -> String {
    s.split(' ').collect::<Vec<_>>().join("%20")
}
