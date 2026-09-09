//! Isolated indexed-reader phase timing, excluding install loading and simulation.
use std::time::Instant;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(debug_assertions) {
        return Err("use --release".into());
    }
    let path = std::env::args().nth(1).ok_or("CAPTURE")?;
    let start = Instant::now();
    let mut source = rondata::capture::indexed::IndexedCapture::open(path)?;
    eprintln!("index {:.3}s", start.elapsed().as_secs_f64());
    let start = Instant::now();
    let text = source.read_replay_setup()?;
    eprintln!(
        "setup selection {:.3}s, {} bytes",
        start.elapsed().as_secs_f64(),
        text.len()
    );
    drop(text);
    let start = Instant::now();
    source.with_replay_initial(|init| eprintln!("{} clock frames", init.frame_guys.len()))?;
    eprintln!(
        "complete initialization {:.3}s",
        start.elapsed().as_secs_f64()
    );
    let start = Instant::now();
    let mut count = 0;
    for frame in source.frame_states() {
        std::hint::black_box(frame?);
        count += 1;
    }
    eprintln!(
        "decode {count} frames {:.3}s",
        start.elapsed().as_secs_f64()
    );
    Ok(())
}
