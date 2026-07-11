use std::collections::HashMap;
use reqwest::blocking::Client;

pub fn embed_text_batch(client: &Client, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
    // create a hashmap with the batch for the request
    let mut json_map = std::collections::HashMap::new();
        json_map.insert("model", serde_json::json!("nomic-embed-text"));
        json_map.insert("input", serde_json::json!(texts));
        
    let res = client.post("http://localhost:11434/api/embeddings")
        .json(&json_map)
        .send()?;

    let parsed: serde_json::Value = res.json()?;
    
    let embeddings: Vec<Vec<f32>> = serde_json::from_value(parsed["embeddings"].clone())?;
    
    Ok(embeddings)
}
