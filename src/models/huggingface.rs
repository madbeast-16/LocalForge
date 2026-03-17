use crate::models::SearchResult;
use crate::error::{Result, LocalForgeError};

const HF_API_BASE: &str = "https://huggingface.co/api";

/// Search HuggingFace for GGUF models matching the query.
pub async fn search_models(query: &str) -> Result<Vec<SearchResult>> {
    let url = format!(
        "{}/models?search={}&filter=gguf&sort=downloads&direction=-1&limit=20",
        HF_API_BASE, query
    );
    
    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await
        .map_err(|e| LocalForgeError::Network(e.to_string()))?;
    
    if !resp.status().is_success() {
        return Err(LocalForgeError::Network(format!("HF API error: {}", resp.status())));
    }
    
    let body = resp.text().await
        .map_err(|e| LocalForgeError::Network(e.to_string()))?;
    
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap_or_default();
    
    let results = parsed
        .into_iter()
        .filter_map(|m| {
            Some(SearchResult {
                id: m.get("id")?.as_str()?.to_string(),
                downloads: m.get("downloads")?.as_u64().unwrap_or(0),
            })
        })
        .collect();
    
    Ok(results)
}

/// List GGUF files available in a given HuggingFace repo.
pub async fn list_repo_files(repo: &str) -> Result<Vec<String>> {
    let url = format!("{}/models/{}", HF_API_BASE, repo);
    
    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await
        .map_err(|e| LocalForgeError::Network(e.to_string()))?;
    
    if !resp.status().is_success() {
        return Err(LocalForgeError::Network(format!("HF repo error: {}", resp.status())));
    }
    
    let body = resp.text().await
        .map_err(|e| LocalForgeError::Network(e.to_string()))?;
    
    let parsed: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| LocalForgeError::Serialization(e.to_string()))?;
    
    let siblings = parsed
        .get("siblings")
        .and_then(|s| s.as_array())
        .cloned()
        .unwrap_or_default();
    
    let gguf_files: Vec<String> = siblings
        .into_iter()
        .filter_map(|s| {
            let fname = s.get("rfilename")?.as_str()?.to_string();
            if fname.ends_with(".gguf") {
                Some(fname)
            } else {
                None
            }
        })
        .collect();
    
    Ok(gguf_files)
}

/// Construct a direct download URL for a GGUF file in a repo.
pub fn download_url(repo: &str, filename: &str) -> String {
    format!("https://huggingface.co/{}/resolve/main/{}", repo, filename)
}