use ocrs::{OcrEngine, OcrEngineParams};
use rten::Model;
use std::fs;

pub fn init_global_ocr() -> anyhow::Result<OcrEngine> {
    println!("Loading detection model...");
    let detection_bytes = fs::read("models/text-detection.rten")?;
    let detection_model = Model::load(detection_bytes)?;
    
    println!("Loading recognition model...");
    let recognition_bytes = fs::read("models/text-recognition.rten")?;
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
