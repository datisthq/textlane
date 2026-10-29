use std::error::Error;
use std::time::Instant;
use std::{env, fs};

/// Converts `<input> <repeat>` times and prints `{elapsedMs, outputBytes}`; setup is untimed.
pub fn run_adapter(
    mut convert: impl FnMut(&str) -> Result<String, Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).ok_or("usage: <input> <repeat>")?;
    let repeat: usize = args.get(2).map_or(Ok(1), |value| value.parse())?;
    let markdown = fs::read_to_string(path)?;
    let mut html = String::new();
    let started = Instant::now();
    for _ in 0..repeat {
        html = convert(&markdown)?;
    }
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
    println!(
        "{}",
        serde_json::json!({ "elapsedMs": elapsed_ms, "outputBytes": html.len() })
    );
    Ok(())
}
