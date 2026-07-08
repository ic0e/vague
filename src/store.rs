use crate::indexer::IndexEntry;

fn dot_product(a: &[f32], b:&[f32]) -> f32 {
    let mut sum = 0.0;
    for i in 0..a.len() {
        sum += a[i] * b[i];
    }
    sum
}

fn magnitude(v: &[f32]) -> f32 {
    let mut sum = 0.0;
    for x in v {
        sum += x * x;
    }
    sum.sqrt()
}

// returns the similarity of the 2 vectors directions
pub fn cosine_similarity(a: &[f32], b:&[f32]) -> f32 {
    let dot = dot_product(a, b);
    let mag_a = magnitude(a);
    let mag_b = magnitude(b);

    dot / (mag_a * mag_b)
}

pub fn search<'a>(entries: &'a [IndexEntry], query_vector: &[f32], top_n: usize) -> Vec<(&'a IndexEntry, f32)> {
    let mut scored: Vec<(&IndexEntry, f32)> = entries
        .iter()
        .map(|e| (e, cosine_similarity(&e.vector, query_vector)))
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    scored.truncate(top_n);
    scored
}

use std::fs;

pub fn save_index(entries: &[IndexEntry], path: &str) -> anyhow::Result<()> {
    let json = serde_json::to_string(entries)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn load_index(path: &str) -> anyhow::Result<Vec<IndexEntry>> {
    let json = fs::read_to_string(path)?;
    let entries: Vec<IndexEntry> = serde_json::from_str(&json)?;
    Ok(entries)
}
