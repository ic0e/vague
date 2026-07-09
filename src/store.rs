use crate::indexer::{IndexEntry, EmbeddingType};

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
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

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot = dot_product(a, b);
    let mag_a = magnitude(a);
    let mag_b = magnitude(b);
    dot / (mag_a * mag_b)
}

/// Score each modality independently, min-max normalize within each group
/// so their raw similarity scales don't dominate each other, then merge
/// and rank together. This prevents CLIP's naturally lower cross-modal
/// scores from always being buried under nomic's higher text-text scores.
pub fn search<'a>(
    entries: &'a [IndexEntry],
    clip_query: &[f32],
    text_query: &[f32],
    top_n: usize,
) -> Vec<(&'a IndexEntry, f32)> {
    // Score each entry against its own embedding space.
    let mut clip_scored: Vec<(&IndexEntry, f32)> = entries
        .iter()
        .filter(|e| e.embedding_type == EmbeddingType::Clip)
        .map(|e| (e, cosine_similarity(&e.vector, clip_query)))
        .collect();

    let mut text_scored: Vec<(&IndexEntry, f32)> = entries
        .iter()
        .filter(|e| e.embedding_type == EmbeddingType::Text)
        .map(|e| (e, cosine_similarity(&e.vector, text_query)))
        .collect();

    // min max normalize each group so scores are on the same 0-1 scale
    // before merging. Without this, nomic text text scores (~0.5-0.7) will
    // always outrank CLIP image text scores (~0.2-0.35).
    normalize_scores(&mut clip_scored);
    normalize_scores(&mut text_scored);

    // Merge both groups and rank together.
    let mut combined: Vec<(&IndexEntry, f32)> = clip_scored
        .into_iter()
        .chain(text_scored.into_iter())
        .collect();

    combined.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    combined.truncate(top_n);
    combined
}

/// Rescale scores in place to [0, 1] using min max normalization to ensure everything is ranked properly
/// if all scores are equal (or only one entry), scores are left as they were
fn normalize_scores(scored: &mut Vec<(&IndexEntry, f32)>) {
    if scored.len() < 2 {
        return;
    }
    let min = scored.iter().map(|(_, s)| *s).fold(f32::INFINITY, f32::min);
    let max = scored.iter().map(|(_, s)| *s).fold(f32::NEG_INFINITY, f32::max);
    let range = max - min;
    if range > 1e-9 {
        for (_, score) in scored.iter_mut() {
            *score = (*score - min) / range;
        }
    }
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
