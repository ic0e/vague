use ocrs::{OcrEngine, OcrEngineParams};
use rten::Model;
use std::env;
use std::fs;

/// Initiates the OCR model for global use.
pub fn init_global_ocr() -> anyhow::Result<OcrEngine> {
    let exe_path = env::current_exe()
        .expect("Failed to get executable path")
        .parent()
        .expect("Failed to get parent directory")
        .to_path_buf();
    
    println!("Loading detection model...");
    let detection_path = exe_path.join("models/text-detection.rten");
    let detection_bytes = fs::read(&detection_path)?;
    let detection_model = Model::load(detection_bytes)?;
    
    println!("Loading recognition model...");
    let recognition_path = exe_path.join("models/text-recognition.rten");
    let recognition_bytes = fs::read(&recognition_path)?;
    let recognition_model = Model::load(recognition_bytes)?;
    println!("Recognition model loaded!");
    
    let params = OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    };
    let engine = OcrEngine::new(params)?;
    
    Ok(engine)
}
