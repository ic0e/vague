use std::collections::HashMap;
use reqwest::blocking::Client;

pub fn embed_text(client: &Client, text: &str) -> anyhow::Result<Vec<f32>> {
    // create a hashmap with the json request
    let mut json_map = HashMap::new();
    json_map.insert("model", "nomic-embed-text");
    json_map.insert("prompt", text);
    
    let res = client.post("http://localhost:11434/api/embeddings")
        .json(&json_map)
        .send()?;

    let parsed: serde_json::Value = res.json()?;
    let embedding: Vec<f32> = serde_json::from_value(parsed["embedding"].clone())?;

    Ok(embedding)
}
