use anyhow::Result;
mod embedder;

fn main() -> anyhow::Result<()> {
    let vector = embedder::embed_text("a fluffy cat")?;
    println!("Got a vector with {} numbers", vector.len());
    println!("First 5 values: {:?}", &vector[..5]);
    
    Ok(())
}
