//! Measure replay retention and save its complete report for before/after comparison.
use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(debug_assertions) {
        return Err("use --release".into());
    }
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err("INSTALL CAPTURE LIMIT NEW_REPORT_PATH".into());
    }
    let output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[3])?;
    let mut output = BufWriter::new(output);
    let loaded = rondata::load::load(&rondata::Install::new(&args[0]))?;
    let text = std::fs::read_to_string(&args[1])?;
    let log = rondata::gamelog::Log::parse(&text);
    let start = std::time::Instant::now();
    let report = rondata::diff::run_traced(
        &loaded,
        &log,
        sim::Tuning::RON,
        Some(args[2].parse()?),
        None,
        &[],
        None,
    )
    .ok_or("no initial state")?;
    eprintln!(
        "{} frames, replay {:.3}s",
        report.frames.len(),
        start.elapsed().as_secs_f64()
    );
    write!(output, "{report:#?}")?;
    output.flush()?;
    Ok(())
}
