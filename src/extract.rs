use std::fs;

pub fn extract_text(path: &std::path::Path) -> anyhow::Result<String> {
    let text: String = fs::read_to_string(path)?;

    Ok(text)
}
