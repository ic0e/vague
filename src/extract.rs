use std::fs;
use pdf_oxide::PdfDocument;
use std::path::{PathBuf};
use std::error::Error;

/// Extracts the text from a text file.
pub fn extract_text(path: &std::path::Path) -> anyhow::Result<String> {
    let text: String = fs::read_to_string(path)?;

    Ok(text)
}

/// Extracts the text from a PDF file.
pub fn extract_pdf_text(path: &PathBuf) -> Result<String, Box<dyn Error>> {
    let doc = PdfDocument::open(path)?;
    let mut text = String::new();

    for page_index in 0..doc.page_count().unwrap() {
            text.push_str(&doc.extract_text(page_index)?);
            text.push('\n'); // separate pages
    }
    
    Ok(text)
}
