use candle_core::{safetensors, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub tmdb_id: i64,
    pub media_type: String,
    pub title: String,
    pub poster: Option<String>,
    pub release_date: Option<String>,
    pub score: f64,
    pub match_reason: String,
}

const MODEL_OWNER: &str = "sentence-transformers";
const MODEL_NAME: &str = "all-MiniLM-L6-v2";

struct EmbeddingEngine {
    model: BertModel,
    tokenizer: tokenizers::Tokenizer,
    device: Device,
}

impl EmbeddingEngine {
    fn load(model_dir: &PathBuf) -> Result<Self, String> {
        let device = Device::Cpu;
        let model_cache = model_dir.join("models").join(format!("{}/{}", MODEL_OWNER, MODEL_NAME));
        std::fs::create_dir_all(&model_cache)
            .map_err(|e| format!("Cannot create model cache: {}", e))?;

        let safetensors_path = model_cache.join("model.safetensors");
        let tokenizer_path = model_cache.join("tokenizer.json");
        let config_path = model_cache.join("config.json");

        if !safetensors_path.exists() || !tokenizer_path.exists() || !config_path.exists() {
            let client = hf_hub::HFClientSync::new()
                .map_err(|e| format!("Cannot connect to HuggingFace Hub: {}", e))?;
            let repo = client.model(MODEL_OWNER, MODEL_NAME);

            repo.download_file().filename("model.safetensors").send()
                .map_err(|e| format!("Downloading model failed: {}", e))?;
            repo.download_file().filename("tokenizer.json").send()
                .map_err(|e| format!("Downloading tokenizer failed: {}", e))?;
            repo.download_file().filename("config.json").send()
                .map_err(|e| format!("Downloading config failed: {}", e))?;
        }

        let config_raw = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Reading config: {}", e))?;
        let config: BertConfig = serde_json::from_str(&config_raw)
            .map_err(|e| format!("Parsing config: {}", e))?;

        let tensors = safetensors::load(&safetensors_path, &device)
            .map_err(|e| format!("Loading weights: {}", e))?;
        let vb = VarBuilder::from_tensors(tensors, candle_core::DType::F32, &device);
        let model = BertModel::load(vb, &config)
            .map_err(|e| format!("Building model: {}", e))?;

        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| format!("Loading tokenizer: {}", e))?;

        Ok(EmbeddingEngine { model, tokenizer, device })
    }

    fn encode(&self, text: &str) -> Result<Vec<f32>, String> {
        let tokens = self.tokenizer.encode(text, true)
            .map_err(|e| format!("Tokenization: {}", e))?;

        let token_ids = tokens.get_ids().to_vec();
        let token_type_ids = vec![0u32; token_ids.len()];
        let attention_mask = tokens.get_attention_mask().to_vec();

        if token_ids.is_empty() {
            return Err("Empty tokens".into());
        }

        let ids = Tensor::new(&token_ids[..], &self.device)
            .map_err(|e| format!("Tensor: {}", e))?.unsqueeze(0)
            .map_err(|e| format!("Unsqueeze: {}", e))?;
        let types = Tensor::new(&token_type_ids[..], &self.device)
            .map_err(|e| format!("Tensor: {}", e))?.unsqueeze(0)
            .map_err(|e| format!("Unsqueeze: {}", e))?;
        let mask = Tensor::new(&attention_mask[..], &self.device)
            .map_err(|e| format!("Tensor: {}", e))?.unsqueeze(0)
            .map_err(|e| format!("Unsqueeze: {}", e))?;

        let hidden = self.model.forward(&ids, &types, Some(&mask))
            .map_err(|e| format!("Forward: {}", e))?;

        let mf = mask.to_dtype(candle_core::DType::F32)
            .map_err(|e| format!("Dtype: {}", e))?;
        let me = mf.unsqueeze(2)
            .map_err(|e| format!("Unsqueeze: {}", e))?;

        let sum_emb = hidden.broadcast_mul(&me)
            .map_err(|e| format!("Mul: {}", e))?
            .sum(1)
            .map_err(|e| format!("Sum: {}", e))?;
        let sum_mask = me.sum(1)
            .map_err(|e| format!("Sum: {}", e))?;
        let mean = sum_emb.broadcast_div(&sum_mask)
            .map_err(|e| format!("Div: {}", e))?;

        let norms = mean.sqr()
            .map_err(|e| format!("Sqr: {}", e))?.sum(1)
            .map_err(|e| format!("Sum: {}", e))?.sqrt()
            .map_err(|e| format!("Sqrt: {}", e))?;
        let normalized = mean.broadcast_div(&norms)
            .map_err(|e| format!("Div: {}", e))?;

        let result = normalized.squeeze(0)
            .map_err(|e| format!("Squeeze: {}", e))?.to_vec1::<f32>()
            .map_err(|e| format!("ToVec: {}", e))?;

        Ok(result)
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() { return 0.0; }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 { 0.0 } else { (dot / (na * nb)) as f64 }
}

fn build_embedding_text(title: &str, overview: Option<&str>) -> String {
    let mut parts = vec![title.to_string()];
    if let Some(o) = overview { if !o.is_empty() { parts.push(o.to_string()); } }
    parts.join(" ")
}

pub struct Recommender {
    engine: Option<Arc<EmbeddingEngine>>,
}

impl Recommender {
    pub fn new(model_dir: PathBuf) -> Self {
        let engine = match EmbeddingEngine::load(&model_dir) {
            Ok(e) => Some(Arc::new(e)),
            Err(err) => {
                eprintln!("BERT model failed to load: {}; recommendations will use TMDB only", err);
                None
            }
        };
        Recommender { engine }
    }

    pub fn is_loaded(&self) -> bool {
        self.engine.is_some()
    }

    pub fn encode(&self, text: &str) -> Result<Vec<f32>, String> {
        match &self.engine {
            Some(e) => e.encode(text),
            None => Err("BERT model not loaded".into()),
        }
    }

    pub fn rank_with_bert(
        &self,
        prompt: &str,
        candidates: &mut [Candidate],
        cached_embeddings: &[(i64, Vec<f32>)],
    ) {
        let prompt_emb = match self.encode(prompt) {
            Ok(e) => e,
            Err(_) => return,
        };

        for c in candidates.iter_mut() {
            let dir_bonus = if c.match_reason.contains("director") { 0.3 } else { 0.0 };
            let cast_bonus = if c.match_reason.contains("favorites") { 0.2 } else { 0.0 };

            let sim = if let Some((_, emb)) = cached_embeddings.iter().find(|(id, _)| *id == c.tmdb_id) {
                cosine_similarity(&prompt_emb, emb)
            } else {
                let text = build_embedding_text(&c.title, c.overview.as_deref());
                if let Ok(emb) = self.encode(&text) {
                    cosine_similarity(&prompt_emb, &emb)
                } else {
                    c.tmdb_vote_avg
                }
            };

            c.score = (sim * 0.7) + (c.tmdb_vote_avg * 0.3) + dir_bonus + cast_bonus;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub tmdb_id: i64,
    pub media_type: String,
    pub title: String,
    pub poster: Option<String>,
    pub release_date: Option<String>,
    pub overview: Option<String>,
    pub tmdb_vote_avg: f64,
    pub score: f64,
    pub match_reason: String,
}
