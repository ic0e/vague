use std::path::Path;

mod embedder;
mod extract;

fn main() -> anyhow::Result<()> {
    let vector = embedder::embed_text("a fluffy cat")?;
    println!("Got a vector with {} numbers", vector.len());
    println!("First 5 values: {:?}", &vector[..5]);

    let path = Path::new("testdata/cat.txt");
    let contents = extract::extract_text(path)?;

    println!("File contents: {}", contents);

    let file_vector = embedder::embed_text(&contents)?;
    println!("Got a vector with {} numbers", file_vector.len());
    println!("First 5 values: {:?}", &file_vector[..5]);
    
    Ok(())
}
