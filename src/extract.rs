use std::fs;
use pdf_oxide::PdfDocument;
use std::path::{PathBuf};
use std::error::Error;
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;

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

pub fn extract_image_text(path: &std::path::Path) -> anyhow::Result<String> {
    // init the rten models
    let detection_model = Model::load_file(".fastembed_cache/text-detection.rten")?;
    let recognition_model = Model::load_file(".fastembed_cache/text-recognition.rten")?;

    let engine = OcrEngine::new(OcrEngineParams {
            detection_model: Some(detection_model),
            recognition_model: Some(recognition_model),
            ..Default::default()
        })?;

    let img = image::ImageReader::open(path)?.decode()?.into_rgb8();
    let img_source = ImageSource::from_bytes(img.as_raw(), img.dimensions())?;
    
    let input = engine.prepare_input(img_source)?;
    let text = engine.get_text(&input)?;
    
    Ok(text)
}
